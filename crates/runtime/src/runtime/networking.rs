//! Per-page network observation, bounded response retention, caching, and controls.
use super::{
    AutomationError,
    events::{EventHub, PageEvent, RequestRecord, ResponseRecord, headers},
    routes::Routes,
};
use crate::network::*;
use async_trait::async_trait;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(crate) struct NetworkExecutor(Option<tokio::runtime::Runtime>);
impl NetworkExecutor {
    pub fn new() -> Result<Self, AutomationError> {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_time()
            .thread_name("brimp-network-dispatch")
            .build()
            .map(|runtime| Self(Some(runtime)))
            .map_err(|error| AutomationError::Internal(error.to_string()))
    }
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.0.as_ref().unwrap().handle().clone()
    }
}
impl Drop for NetworkExecutor {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_background();
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkOptions {
    pub offline: bool,
    pub latency: Duration,
    pub bypass_cache: bool,
    pub max_retained_bytes: usize,
    pub max_retained_body_bytes: usize,
    /// Delays delivery of a fully buffered response. Does not throttle the transport or streams.
    pub buffered_download_bytes_per_second: Option<usize>,
}
impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            offline: false,
            latency: Duration::ZERO,
            bypass_cache: false,
            max_retained_bytes: 32 * 1024 * 1024,
            max_retained_body_bytes: 8 * 1024 * 1024,
            buffered_download_bytes_per_second: None,
        }
    }
}
struct Record {
    request: ResourceRequest,
    response: Option<ResourceResponse>,
    body: Option<Vec<u8>>,
    bytes: usize,
    complete: bool,
    failed: bool,
    sse: Sse,
}
struct Cached {
    response: ResourceResponse,
    expires: Instant,
}
#[derive(Default)]
struct Streams {
    generation: u64,
    closed: bool,
    active: Vec<std::sync::Weak<super::streams::BodyState>>,
}
struct State {
    next: u64,
    options: NetworkOptions,
    extra_headers: HeaderList,
    records: BTreeMap<u64, Record>,
    order: VecDeque<u64>,
    cache: BTreeMap<String, Cached>,
}
#[derive(Clone)]
pub(crate) struct NetworkRuntime {
    inner: Arc<dyn ResourceLoader>,
    sockets: Arc<dyn WebSocketConnector>,
    pub routes: Routes,
    pub events: EventHub,
    state: Arc<Mutex<State>>,
    executor: tokio::runtime::Handle,
    streams: Arc<Mutex<Streams>>,
}
impl std::fmt::Debug for NetworkRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkRuntime").finish_non_exhaustive()
    }
}
impl NetworkRuntime {
    pub fn new(
        inner: Arc<dyn ResourceLoader>,
        events: EventHub,
        executor: tokio::runtime::Handle,
        sockets: Arc<dyn WebSocketConnector>,
    ) -> Self {
        let routes = Routes::default();
        Self {
            sockets,
            streams: Arc::new(Mutex::new(Streams::default())),
            inner: Arc::new(InterceptingResourceLoader::new(
                inner,
                Arc::new(routes.clone()),
            )),
            routes,
            events,
            state: Arc::new(Mutex::new(State {
                next: 1,
                options: NetworkOptions::default(),
                extra_headers: HeaderList::new(),
                records: BTreeMap::new(),
                order: VecDeque::new(),
                cache: BTreeMap::new(),
            })),
            executor,
        }
    }
    pub fn stream_generation(&self) -> u64 {
        self.streams.lock().unwrap().generation
    }
    pub fn register_stream(
        &self,
        state: &Arc<super::streams::BodyState>,
        generation: u64,
    ) -> Result<(), AutomationError> {
        let mut streams = self.streams.lock().unwrap();
        if streams.closed {
            return Err(AutomationError::Closed);
        }
        if streams.generation != generation {
            return Err(AutomationError::Cancellation);
        }
        streams.active.retain(|stream| stream.strong_count() > 0);
        streams.active.push(Arc::downgrade(state));
        Ok(())
    }
    pub fn cancel_streams(&self) {
        let active = {
            let mut streams = self.streams.lock().unwrap();
            streams.generation += 1;
            std::mem::take(&mut streams.active)
        };
        for stream in active.into_iter().filter_map(|stream| stream.upgrade()) {
            stream.cancel();
        }
    }
    pub fn watch_cancellation(
        &self,
        state: std::sync::Weak<super::streams::BodyState>,
        token: super::CancellationToken,
    ) {
        self.executor.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(5)).await;
                let Some(state) = state.upgrade() else {
                    break;
                };
                if token.is_cancelled() {
                    state.cancel();
                    break;
                }
            }
        });
    }
    pub fn configure(&self, options: NetworkOptions) -> Result<(), AutomationError> {
        if options.buffered_download_bytes_per_second == Some(0) {
            return Err(AutomationError::InvalidInput(
                "buffered download rate must be positive".into(),
            ));
        }
        let mut state = self.state.lock().unwrap();
        state.options = options;
        trim(&mut state);
        Ok(())
    }
    pub fn extra_headers(&self, headers: Vec<(String, String)>) -> Result<(), AutomationError> {
        self.state.lock().unwrap().extra_headers = super::routes::parse_headers(headers)?;
        Ok(())
    }
    pub fn clear_cache(&self) {
        self.state.lock().unwrap().cache.clear();
    }
    pub fn body(&self, id: u64) -> Result<Vec<u8>, AutomationError> {
        let state = self.state.lock().unwrap();
        let record = state
            .records
            .get(&id)
            .ok_or_else(|| AutomationError::InvalidInput("unknown or evicted request".into()))?;
        if !record.complete || record.failed {
            return Err(AutomationError::InvalidInput(
                "response body is not available".into(),
            ));
        }
        record.body.clone().ok_or_else(|| {
            AutomationError::InvalidInput(
                "response body exceeded retention limits or was evicted".into(),
            )
        })
    }
    pub fn close(&self) {
        self.streams.lock().unwrap().closed = true;
        self.cancel_streams();
        self.routes.close();
        self.events.clear();
        let mut state = self.state.lock().unwrap();
        state.records.clear();
        state.order.clear();
        state.cache.clear();
    }
    fn begin(
        &self,
        mut request: ResourceRequest,
    ) -> (
        u64,
        ResourceRequest,
        NetworkOptions,
        Option<ResourceResponse>,
    ) {
        let mut state = self.state.lock().unwrap();
        for (name, value) in state.extra_headers.iter() {
            request.headers.insert(name.clone(), value.clone());
        }
        let id = state.next;
        state.next += 1;
        request.request_id = Some(id);
        let cache = if !state.options.bypass_cache
            && !self.routes.matches(&request.url)
            && request.method == http::Method::GET
            && !request.headers.contains_key(http::header::CACHE_CONTROL)
        {
            state
                .cache
                .get(&cache_key(&request))
                .filter(|cached| cached.expires > Instant::now())
                .map(|cached| cached.response.clone())
        } else {
            None
        };
        let event = PageEvent::Request {
            request: RequestRecord {
                id,
                method: request.method.to_string(),
                url: request.url.clone(),
                headers: headers(&request.headers),
                body: request
                    .body
                    .as_ref()
                    .filter(|body| body.len() <= state.options.max_retained_body_bytes)
                    .cloned(),
                resource_type: request.resource_type.clone(),
            },
        };
        let mut kept = request.clone();
        kept.body = None;
        state.records.insert(
            id,
            Record {
                request: kept,
                response: None,
                body: Some(Vec::new()),
                bytes: 0,
                complete: false,
                failed: false,
                sse: Sse::default(),
            },
        );
        state.order.push_back(id);
        while state.order.len() > 256 {
            if let Some(old) = state.order.pop_front() {
                state.records.remove(&old);
            }
        }
        let options = state.options.clone();
        drop(state);
        self.events.emit(event);
        (id, request, options, cache)
    }
    fn response(&self, id: u64, response: &ResourceResponse, cached: bool) {
        let mut metadata = response.clone();
        metadata.body.clear();
        if let Some(record) = self.state.lock().unwrap().records.get_mut(&id) {
            record.response = Some(metadata);
        }
        self.events.emit(PageEvent::Response {
            response: ResponseRecord {
                request_id: id,
                url: response.effective_url.clone(),
                status: response.status.as_u16(),
                headers: headers(&response.headers),
                from_cache: cached,
            },
        });
    }
    fn chunk(&self, id: u64, bytes: &[u8]) {
        let mut state = self.state.lock().unwrap();
        let limit = state.options.max_retained_body_bytes;
        let Some(record) = state.records.get_mut(&id) else {
            return;
        };
        record.bytes = record.bytes.saturating_add(bytes.len());
        if record.bytes > limit {
            record.body = None;
        } else if let Some(body) = &mut record.body {
            body.extend_from_slice(bytes);
        }
        let messages = if record
            .response
            .as_ref()
            .and_then(|response| response.headers.get(http::header::CONTENT_TYPE))
            .is_some_and(|value| value.as_bytes().starts_with(b"text/event-stream"))
        {
            record.sse.feed(id, bytes, limit)
        } else {
            Vec::new()
        };
        trim(&mut state);
        drop(state);
        for message in messages {
            self.events.emit(message);
        }
    }
    fn finish(&self, id: u64, error: Option<&NetworkError>) {
        let mut state = self.state.lock().unwrap();
        let Some(record) = state.records.get_mut(&id) else {
            return;
        };
        if record.complete {
            return;
        }
        record.complete = true;
        record.failed = error.is_some();
        let bytes = record.bytes;
        let cache = if error.is_none() {
            record
                .response
                .as_ref()
                .zip(record.body.as_ref())
                .map(|(response, body)| {
                    let mut response = response.clone();
                    response.body = body.clone();
                    (record.request.clone(), response)
                })
        } else {
            None
        };
        if let Some((request, response)) = cache
            && !state.options.bypass_cache
            && request.method == http::Method::GET
            && response.status == http::StatusCode::OK
            && !response.headers.contains_key(http::header::SET_COOKIE)
            && !response.headers.contains_key(http::header::VARY)
        {
            let control = response
                .headers
                .get(http::header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("");
            if !control.contains("no-store")
                && !control.contains("no-cache")
                && let Some(age) = control
                    .split(',')
                    .find_map(|field| {
                        field
                            .trim()
                            .strip_prefix("max-age=")
                            .and_then(|value| value.parse::<u64>().ok())
                    })
                    .filter(|age| *age > 0 && *age <= 31536000)
            {
                if state
                    .cache
                    .values()
                    .map(|value| value.response.body.len())
                    .sum::<usize>()
                    + response.body.len()
                    > state.options.max_retained_bytes
                {
                    state.cache.clear();
                }
                if response.body.len() <= state.options.max_retained_bytes {
                    state.cache.insert(
                        cache_key(&request),
                        Cached {
                            response,
                            expires: Instant::now() + Duration::from_secs(age),
                        },
                    );
                }
            }
        }
        drop(state);
        self.events.emit(match error {
            Some(error) => PageEvent::RequestFailed {
                request_id: id,
                message: error.to_string(),
                cancelled: matches!(error, NetworkError::Cancelled | NetworkError::Closed),
            },
            None => PageEvent::RequestFinished {
                request_id: id,
                bytes,
            },
        });
    }
}
fn trim(state: &mut State) {
    let mut used = state
        .records
        .values()
        .filter_map(|record| record.body.as_ref())
        .map(Vec::len)
        .sum::<usize>();
    for id in &state.order {
        if let Some(record) = state.records.get_mut(id)
            && (record
                .body
                .as_ref()
                .is_some_and(|body| body.len() > state.options.max_retained_body_bytes)
                || used > state.options.max_retained_bytes)
        {
            used -= record.body.take().map_or(0, |body| body.len());
        }
    }
}
fn cache_key(request: &ResourceRequest) -> String {
    format!("{}\n{:?}", request.url, headers(&request.headers))
}
struct CompletionGuard(NetworkRuntime, u64);
impl Drop for CompletionGuard {
    fn drop(&mut self) {
        self.0.routes.cancel_request(self.1);
        self.0.finish(self.1, Some(&NetworkError::Cancelled));
    }
}
async fn stream_ready(handle: &ResourceStreamHandle, directive: ResourceStreamDirective) -> bool {
    match directive {
        ResourceStreamDirective::Continue => handle.resume(),
        ResourceStreamDirective::Cancel | ResourceStreamDirective::Redirect => handle.cancel(),
        ResourceStreamDirective::Pause => {}
    }
    while handle.is_paused() && !handle.is_cancelled() {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    !handle.is_cancelled()
}
#[async_trait]
impl ResourceLoader for NetworkRuntime {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (id, request, options, cache) = self.begin(request);
        let _guard = CompletionGuard(self.clone(), id);
        let result = if options.offline {
            Err(NetworkError::Transport("network is offline".into()))
        } else {
            tokio::time::sleep(options.latency).await;
            match cache.clone() {
                Some(response) => Ok(response),
                None => self.inner.fetch(request).await,
            }
        };
        if let Ok(response) = &result {
            self.response(id, response, cache.is_some());
            if let Some(rate) = options.buffered_download_bytes_per_second {
                tokio::time::sleep(Duration::from_secs_f64(
                    response.body.len() as f64 / rate as f64,
                ))
                .await;
            }
            self.chunk(id, &response.body);
        }
        self.finish(id, result.as_ref().err());
        result
    }
    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        let loader = self.clone();
        self.executor.spawn(async move {
            callback(loader.fetch(request).await);
        });
        Ok(())
    }
    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let (id, request, options, cache) = self.begin(request);
        let proxy = ResourceStreamHandle::proxy();
        let control = proxy.clone();
        let loader = self.clone();
        let monitor = self.clone();
        let cancellation = control.clone();
        self.executor.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(5)).await;
                if cancellation.is_cancelled() {
                    monitor.routes.cancel_request(id);
                    monitor.finish(id, Some(&NetworkError::Cancelled));
                    break;
                }
                if monitor
                    .state
                    .lock()
                    .unwrap()
                    .records
                    .get(&id)
                    .is_none_or(|record| record.complete)
                {
                    break;
                }
            }
        });
        self.executor.spawn(async move {
            tokio::time::sleep(options.latency).await;
            if control.is_cancelled() {
                loader.finish(id, Some(&NetworkError::Cancelled));
                return;
            }
            if options.offline {
                let error = NetworkError::Transport("network is offline".into());
                loader.finish(id, Some(&error));
                callback(ResourceStreamEvent::Error(error), &control);
                return;
            }
            if let Some(response) = cache {
                let _guard = CompletionGuard(loader.clone(), id);
                loader.response(id, &response, true);
                control.pause();
                let directive = callback(
                    ResourceStreamEvent::Headers {
                        status: response.status,
                        headers: response.headers,
                        url: response.effective_url,
                    },
                    &control,
                );
                if directive == ResourceStreamDirective::Redirect {
                    loader.finish(id, None);
                }
                if !stream_ready(&control, directive).await {
                    return;
                }
                // Cached bodies are sliced so a bounded consumer never needs to buffer the whole resource.
                for bytes in response.body.chunks(16 * 1024) {
                    loop {
                        control.pause();
                        let directive =
                            callback(ResourceStreamEvent::Chunk(bytes.to_vec()), &control);
                        if !stream_ready(&control, directive).await {
                            return;
                        }
                        if directive == ResourceStreamDirective::Continue {
                            loader.chunk(id, bytes);
                            break;
                        }
                    }
                }
                loader.finish(id, None);
                callback(ResourceStreamEvent::Complete, &control);
                return;
            }
            let user = Arc::new(Mutex::new(callback));
            let consumer = user.clone();
            let observed = loader.clone();
            let guard = CompletionGuard(loader.clone(), id);
            let wrapped: ResourceStreamCallback = Box::new(move |event, handle| {
                let _ = &guard;
                match &event {
                    ResourceStreamEvent::Headers {
                        status,
                        headers,
                        url,
                    } => observed.response(
                        id,
                        &ResourceResponse {
                            status: *status,
                            headers: headers.clone(),
                            body: Vec::new(),
                            effective_url: url.clone(),
                            metadata: Default::default(),
                        },
                        false,
                    ),
                    ResourceStreamEvent::Chunk(_) => {}
                    ResourceStreamEvent::Complete => observed.finish(id, None),
                    ResourceStreamEvent::Error(error) => observed.finish(id, Some(error)),
                }
                let bytes = match &event {
                    ResourceStreamEvent::Chunk(bytes) => Some(bytes.clone()),
                    _ => None,
                };
                let directive = consumer.lock().unwrap()(event, handle);
                if directive == ResourceStreamDirective::Continue
                    && let Some(bytes) = bytes
                {
                    observed.chunk(id, &bytes);
                }
                if directive == ResourceStreamDirective::Cancel {
                    observed.finish(id, Some(&NetworkError::Cancelled));
                } else if directive == ResourceStreamDirective::Redirect {
                    observed.finish(id, None);
                }
                directive
            });
            match loader.inner.fetch_stream_callback(request, wrapped) {
                Ok(handle) => control.attach(handle),
                Err(error) => {
                    loader.finish(id, Some(&error));
                    user.lock().unwrap()(ResourceStreamEvent::Error(error), &control);
                }
            }
        });
        Ok(proxy)
    }
}
impl WebSocketConnector for NetworkRuntime {
    fn open_websocket(
        &self,
        url: String,
        mut headers: HeaderList,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        let mut state = self.state.lock().unwrap();
        if state.options.offline {
            return Err(NetworkError::Transport("network is offline".into()));
        }
        for (name, value) in state.extra_headers.iter() {
            headers.insert(name.clone(), value.clone());
        }
        let id = state.next;
        state.next += 1;
        drop(state);
        let events = self.events.clone();
        let event_url = url.clone();
        self.sockets.open_websocket(
            url,
            headers,
            Box::new(move |event| {
                let (kind, data, message) = match &event {
                    WebSocketEvent::Open => ("open", vec![], None),
                    WebSocketEvent::Text(value) => ("message", value.as_bytes().to_vec(), None),
                    WebSocketEvent::Binary(value) => ("message", value.clone(), None),
                    WebSocketEvent::FrameSent { payload, .. } => ("sent", payload.clone(), None),
                    WebSocketEvent::Close { .. } => ("close", vec![], None),
                    WebSocketEvent::Error(message) => ("error", vec![], Some(message.clone())),
                    WebSocketEvent::HandshakeRequest { .. } => ("handshake_request", vec![], None),
                    WebSocketEvent::HandshakeResponse { .. } => {
                        ("handshake_response", vec![], None)
                    }
                };
                events.emit(PageEvent::WebSocket {
                    id,
                    url: event_url.clone(),
                    kind: kind.into(),
                    data,
                    message,
                });
                callback(event);
            }),
        )
    }
}
#[derive(Default)]
struct Sse {
    line: Vec<u8>,
    data: Vec<String>,
    event: String,
    id: String,
    size: usize,
    overflow: bool,
    cr: bool,
}
impl Sse {
    fn feed(&mut self, request_id: u64, bytes: &[u8], limit: usize) -> Vec<PageEvent> {
        let mut output = Vec::new();
        for &byte in bytes {
            if self.cr && byte == b'\n' {
                self.cr = false;
                continue;
            }
            self.cr = false;
            if byte == b'\r' || byte == b'\n' {
                self.cr = byte == b'\r';
                let line = String::from_utf8_lossy(&std::mem::take(&mut self.line)).into_owned();
                if line.is_empty() {
                    if !self.overflow && !self.data.is_empty() {
                        output.push(PageEvent::EventSource {
                            request_id,
                            event_name: if self.event.is_empty() {
                                "message".into()
                            } else {
                                self.event.clone()
                            },
                            event_id: self.id.clone(),
                            data: self.data.join("\n"),
                        });
                    }
                    self.data.clear();
                    self.event.clear();
                    self.size = 0;
                    self.overflow = false;
                } else if !self.overflow {
                    let (name, value) = line.split_once(':').unwrap_or((&line, ""));
                    let value = value.strip_prefix(' ').unwrap_or(value);
                    match name {
                        "data" => self.data.push(value.into()),
                        "event" => self.event = value.into(),
                        "id" if !value.contains('\0') => self.id = value.into(),
                        _ => {}
                    }
                }
            } else {
                self.size += 1;
                if self.size > limit {
                    self.overflow = true;
                    self.line.clear();
                    self.data.clear();
                } else {
                    self.line.push(byte);
                }
            }
        }
        output
    }
}
