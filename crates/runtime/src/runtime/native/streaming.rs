//! Bounded, pull-based response bodies. No DOM or JavaScript value leaves its owner thread.
use super::RequestOptions;
use crate::runtime::{AutomationError, CancellationToken, networking::NetworkRuntime};
use crate::{network::*, web_apis::BrowsingContext};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};
const CAPACITY: usize = 256 * 1024;
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamMetadata {
    pub status: u16,
    pub url: String,
    pub headers: Vec<(String, String)>,
}
struct Queue {
    headers: Option<StreamMetadata>,
    chunks: VecDeque<Vec<u8>>,
    bytes: usize,
    done: bool,
    error: Option<NetworkError>,
    handle: Option<ResourceStreamHandle>,
}
pub(crate) struct BodyState {
    queue: Mutex<Queue>,
    changed: Condvar,
    cancellation: CancellationToken,
    deadline: Instant,
    timeout: Duration,
    page_id: u64,
}
impl BodyState {
    pub(crate) fn cancel(&self) {
        self.cancellation.cancel();
        let mut queue = self.queue.lock().unwrap();
        if let Some(handle) = &queue.handle {
            handle.cancel();
        }
        queue.chunks.clear();
        queue.bytes = 0;
        self.changed.notify_all();
    }
    fn check(&self) -> Result<(), AutomationError> {
        if crate::runtime::plugins::is_callback_page(self.page_id) {
            return Err(AutomationError::Reentry);
        }
        if self.cancellation.is_cancelled() {
            return Err(AutomationError::Cancellation);
        }
        if Instant::now() >= self.deadline {
            return Err(AutomationError::Timeout(self.timeout));
        }
        Ok(())
    }
}
pub struct BodyStream(Arc<BodyState>);
impl BodyStream {
    /// One chunk at a time. A consumer may cancel from another thread.
    pub fn next_chunk(&self) -> Result<Option<Vec<u8>>, AutomationError> {
        let mut queue = self.0.queue.lock().unwrap();
        loop {
            if let Err(error) = self.0.check() {
                drop(queue);
                self.0.cancel();
                return Err(error);
            }
            if let Some(bytes) = queue.chunks.pop_front() {
                queue.bytes -= bytes.len();
                if let Some(handle) = &queue.handle {
                    handle.resume();
                }
                return Ok(Some(bytes));
            }
            if let Some(error) = queue.error.take() {
                return Err(AutomationError::Transport(error.to_string()));
            }
            if queue.done {
                return Ok(None);
            }
            queue = self
                .0
                .changed
                .wait_timeout(queue, Duration::from_millis(5))
                .unwrap()
                .0;
        }
    }
    pub fn cancel(&self) {
        self.0.cancel();
    }
    fn headers(&self) -> Result<StreamMetadata, AutomationError> {
        let mut queue = self.0.queue.lock().unwrap();
        loop {
            self.0.check()?;
            if let Some(headers) = &queue.headers {
                return Ok(headers.clone());
            }
            if let Some(error) = queue.error.take() {
                return Err(AutomationError::Transport(error.to_string()));
            }
            if queue.done {
                return Err(AutomationError::Transport(
                    "stream ended before response headers".into(),
                ));
            }
            queue = self
                .0
                .changed
                .wait_timeout(queue, Duration::from_millis(5))
                .unwrap()
                .0;
        }
    }
}
impl Drop for BodyStream {
    fn drop(&mut self) {
        self.cancel();
    }
}
pub struct StreamingResponse {
    pub metadata: StreamMetadata,
    pub body: BodyStream,
}

pub(crate) fn open(
    network: &Arc<NetworkRuntime>,
    context: Arc<BrowsingContext>,
    page_id: u64,
    generation: u64,
    mut request: ResourceRequest,
    options: RequestOptions,
    timeout: Duration,
) -> Result<StreamingResponse, AutomationError> {
    let deadline = Instant::now() + timeout;
    context.apply_request_identity(&mut request.headers);
    let mut supplied = crate::runtime::request::supplied_cookie(&request);
    for redirects in 0..=options.max_redirects {
        crate::runtime::request::refresh_cookie_header(&context, &mut request, supplied.as_deref())
            .map_err(|e| AutomationError::Transport(e.to_string()))?;
        let state = Arc::new(BodyState {
            queue: Mutex::new(Queue {
                headers: None,
                chunks: VecDeque::new(),
                bytes: 0,
                done: false,
                error: None,
                handle: None,
            }),
            changed: Condvar::new(),
            cancellation: CancellationToken::new(),
            deadline,
            timeout,
            page_id,
        });
        network.register_stream(&state, generation)?;
        let stream = BodyStream(state.clone());
        let callback_state = state.clone();
        let allow_redirects = options.allow_redirects;
        let external = options.cancellation.clone();
        // External abort is checked in the callback and consumer waits, including before headers.
        let cancel_state = Arc::downgrade(&state);
        network.watch_cancellation(cancel_state, external);
        let control = network
            .fetch_stream_callback(
                request.clone(),
                Box::new(move |event, control| {
                    if callback_state.cancellation.is_cancelled() {
                        return ResourceStreamDirective::Cancel;
                    }
                    let mut queue = callback_state.queue.lock().unwrap();
                    let mut directive = ResourceStreamDirective::Continue;
                    match event {
                        ResourceStreamEvent::Headers {
                            status,
                            headers,
                            url,
                        } => {
                            if allow_redirects
                                && matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
                                && headers.contains_key(http::header::LOCATION)
                            {
                                directive = ResourceStreamDirective::Redirect;
                            }
                            queue.headers = Some(StreamMetadata {
                                status: status.as_u16(),
                                headers: crate::runtime::events::headers(&headers),
                                url,
                            });
                        }
                        ResourceStreamEvent::Chunk(bytes) => {
                            if queue.bytes.saturating_add(bytes.len()) > CAPACITY {
                                return ResourceStreamDirective::Pause;
                            }
                            queue.bytes += bytes.len();
                            queue.chunks.push_back(bytes);
                        }
                        ResourceStreamEvent::Complete => queue.done = true,
                        ResourceStreamEvent::Error(error) => {
                            queue.error = Some(error);
                            queue.done = true;
                        }
                    }
                    // Save a non-owning control before waking readers, even for synchronous loaders.
                    if queue.handle.is_none() {
                        queue.handle = Some(control.clone());
                    }
                    callback_state.changed.notify_all();
                    directive
                }),
            )
            .map_err(|e| AutomationError::Transport(e.to_string()))?;
        {
            let mut queue = state.queue.lock().unwrap();
            queue.handle = Some(control);
            if state.cancellation.is_cancelled() {
                queue.handle.as_ref().unwrap().cancel();
            }
        }
        let metadata = stream.headers()?;
        let headers = crate::runtime::routes::parse_headers(metadata.headers.clone())?;
        for cookie in headers.get_all(http::header::SET_COOKIE) {
            if let Ok(cookie) = cookie.to_str() {
                context.store_response_cookie(&request.url, cookie);
            }
        }
        if !options.allow_redirects
            || !crate::runtime::request::redirect_request(
                &mut request,
                metadata.status,
                &headers,
                &mut supplied,
            )
            .map_err(|e| AutomationError::Transport(e.to_string()))?
        {
            return Ok(StreamingResponse {
                metadata,
                body: stream,
            });
        }
        stream.cancel();
        if redirects == options.max_redirects {
            return Err(AutomationError::Transport(format!(
                "redirect limit of {} exceeded",
                options.max_redirects
            )));
        }
    }
    unreachable!()
}
