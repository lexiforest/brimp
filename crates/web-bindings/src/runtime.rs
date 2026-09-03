use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use blitz_dom::{LocalName, Namespace, NodeData, Prefix, QualName, ns};
use browser_dom::{
    BrowserDocument, CssomError, HtmlParserSession, NodeId, ParseProgress, parse_xml_at_root,
};
use jsc::{
    JsException, JsObjectIdentity, JsRuntime, NativeCall, NativeError, NativeValue,
    PromiseSettlement, ProtectedJsObject,
};
use selectors::matching::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, SelectorCaches,
    matches_selector_list,
};
use style::dom::{TDocument, TNode};
use style::dom_apis::element_matches;

use crate::{
    PersistentStorage, WrapperCache,
    angle::{AngleStore, UniformValue},
    audio::AudioStore,
    canvas::{
        CanvasColorSpace, CanvasColorType, CanvasDrawEffects, CanvasFilterInput,
        CanvasFilterOperation, CanvasLightSource, CanvasPaintStyle, CanvasShadowStyle, CanvasStore,
        CanvasStrokeStyle,
    },
    gpu::{
        GpuBindGroupEntry, GpuBindGroupLayoutEntry, GpuColorAttachment, GpuColorTarget,
        GpuComputeCommand, GpuDepthStencilAttachment, GpuDepthStencilState, GpuMultisampleState,
        GpuPrimitiveState, GpuRenderBundleEncoderDescriptor, GpuRenderCommand,
        GpuSamplerDescriptor, GpuStore, GpuTextureViewDescriptor, GpuTimestampWrites,
        GpuVertexBufferLayout,
    },
};

mod dispatch_audio;
mod dispatch_canvas;
mod dispatch_dom;
mod dispatch_gpu;
mod dispatch_platform;
mod dispatch_webgl;

const CLASS_DEFINITIONS: &str = concat!(
    include_str!("runtime/bootstrap.js"),
    include_str!("runtime/events_messaging.js"),
    include_str!("runtime/dom_core.js"),
    include_str!("runtime/cssom.js"),
    include_str!("runtime/document_collections.js"),
    include_str!("runtime/elements.js"),
    include_str!("runtime/html_elements.js"),
    include_str!("runtime/window_location.js"),
    include_str!("runtime/navigator_services.js"),
    include_str!("runtime/url_encoding_files.js"),
    include_str!("runtime/storage_fetch.js"),
    include_str!("runtime/css_style.js"),
    include_str!("runtime/observers.js"),
    include_str!("runtime/input.js"),
    include_str!("runtime/install.js"),
);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WebFeatureFlags {
    pub worker_system: bool,
    pub streaming_networking: bool,
    pub persistent_storage: bool,
    pub canvas: bool,
    pub webgl: bool,
    pub webgpu: bool,
    pub webaudio: bool,
    pub webaudio_output: bool,
}

impl WebFeatureFlags {
    pub fn json(self) -> String {
        format!(
            "{{\"workerSystem\":{},\"streamingNetworking\":{},\"persistentStorage\":{},\"canvas\":{},\"webgl\":{},\"webgpu\":{},\"webaudio\":{},\"webaudioOutput\":{}}}",
            self.worker_system,
            self.streaming_networking,
            self.persistent_storage,
            self.canvas,
            self.webgl,
            self.webgpu,
            self.webaudio,
            self.webaudio_output,
        )
    }
}

pub struct TimerQueue {
    next_id: u32,
    timers: Vec<Timer>,
    microtasks: VecDeque<ProtectedJsObject>,
}

pub struct FetchQueue {
    next_id: u64,
    pending: VecDeque<PendingFetch>,
    settlements: HashMap<u64, PromiseSettlement>,
}

pub struct PendingFetch {
    pub id: u64,
    pub url: String,
    pub method: String,
    pub headers_json: String,
    pub body: Option<Vec<u8>>,
    pub streaming: bool,
}

pub enum PendingWorkerOperation {
    Create {
        id: u64,
        url: String,
        kind: String,
        name: String,
        scope: String,
    },
    Post {
        id: u64,
        message_json: String,
    },
    Terminate {
        id: u64,
    },
    Unregister {
        id: u64,
    },
}

pub struct WorkerQueue {
    next_id: u64,
    pending: VecDeque<PendingWorkerOperation>,
}

pub enum PendingWebSocketOperation {
    Create { id: u64, url: String },
    SendText { id: u64, message: String },
    Close { id: u64 },
    CancelFetch { id: u64 },
}

pub struct StreamingQueue {
    next_id: u64,
    pending: VecDeque<PendingWebSocketOperation>,
}

impl Default for StreamingQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            pending: VecDeque::new(),
        }
    }
}

impl StreamingQueue {
    pub fn take_pending(&mut self) -> Vec<PendingWebSocketOperation> {
        self.pending.drain(..).collect()
    }
}

impl Default for WorkerQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            pending: VecDeque::new(),
        }
    }
}

impl WorkerQueue {
    pub fn take_pending(&mut self) -> Vec<PendingWorkerOperation> {
        self.pending.drain(..).collect()
    }
}

impl Default for FetchQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            pending: VecDeque::new(),
            settlements: HashMap::new(),
        }
    }
}

impl FetchQueue {
    fn push(&mut self, request: PendingFetch, settlement: PromiseSettlement) {
        self.settlements.insert(request.id, settlement);
        self.pending.push_back(request);
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    pub fn take_pending(&mut self) -> Vec<PendingFetch> {
        self.pending.drain(..).collect()
    }

    pub fn take_settlement(&mut self, id: u64) -> Option<PromiseSettlement> {
        self.settlements.remove(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.settlements.is_empty()
    }
}

struct Timer {
    id: u32,
    deadline: Instant,
    interval: Option<Duration>,
    callback: ProtectedJsObject,
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self {
            next_id: 1,
            timers: Vec::new(),
            microtasks: VecDeque::new(),
        }
    }
}

impl TimerQueue {
    fn schedule(&mut self, delay_ms: f64, repeat: bool, callback: ProtectedJsObject) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let delay_ms = if delay_ms.is_finite() {
            delay_ms.max(0.0)
        } else {
            0.0
        };
        let delay = Duration::from_secs_f64(delay_ms / 1000.0);
        self.timers.push(Timer {
            id,
            deadline: Instant::now() + delay,
            interval: repeat.then_some(delay.max(Duration::from_millis(1))),
            callback,
        });
        id
    }

    fn clear(&mut self, id: u32) {
        self.timers.retain(|timer| timer.id != id);
    }

    pub fn pop_due(&mut self) -> Option<ProtectedJsObject> {
        let now = Instant::now();
        let index = self
            .timers
            .iter()
            .enumerate()
            .filter(|(_, timer)| timer.deadline <= now)
            .min_by_key(|(_, timer)| timer.deadline)
            .map(|(index, _)| index)?;
        if let Some(interval) = self.timers[index].interval {
            self.timers[index].deadline = now + interval;
            Some(self.timers[index].callback.clone())
        } else {
            Some(self.timers.remove(index).callback)
        }
    }

    fn queue_microtask(&mut self, callback: ProtectedJsObject) {
        self.microtasks.push_back(callback);
    }

    pub fn pop_microtask(&mut self) -> Option<ProtectedJsObject> {
        self.microtasks.pop_front()
    }
}

pub struct BindingRuntime {
    state: Rc<BindingState>,
    input_controller: ProtectedJsObject,
}

pub struct BindingQueues {
    pub timers: Rc<RefCell<TimerQueue>>,
    pub fetches: Rc<RefCell<FetchQueue>>,
    pub workers: Rc<RefCell<WorkerQueue>>,
    pub streaming: Rc<RefCell<StreamingQueue>>,
}

#[derive(Clone, Debug)]
pub struct StoredCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub host_only: bool,
    pub path: String,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
}

#[derive(Default)]
pub struct CookieJar(Mutex<cookie_store::CookieStore>);

impl CookieJar {
    pub fn store(&self, url: &str, header: &str) -> Result<(), String> {
        let url = url::Url::parse(url).map_err(|error| error.to_string())?;
        let cookie =
            cookie_store::RawCookie::parse(header.to_owned()).map_err(|error| error.to_string())?;
        let result = self
            .0
            .lock()
            .expect("cookie store lock poisoned")
            .insert_raw(&cookie, &url);
        match result {
            Ok(_) | Err(cookie_store::CookieError::Expired) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn set(&self, url: &str, name: &str, value: &str) -> Result<(), String> {
        if name.is_empty()
            || name.contains([';', '=', '\r', '\n'])
            || value.contains([';', '\r', '\n'])
        {
            return Err("invalid cookie name or value".into());
        }
        self.store(url, &format!("{name}={value}; Path=/"))
    }

    fn store_document(&self, url: &str, header: &str) -> Result<(), String> {
        let url = url::Url::parse(url).map_err(|error| error.to_string())?;
        let raw =
            cookie_store::RawCookie::parse(header.to_owned()).map_err(|error| error.to_string())?;
        if raw.http_only() == Some(true) || (raw.secure() == Some(true) && url.scheme() != "https")
        {
            return Ok(());
        }
        let cookie = cookie_store::Cookie::try_from_raw_cookie(&raw, &url)
            .map_err(|error| error.to_string())?
            .into_owned();
        let domain = cookie.domain.as_cow().unwrap_or_default().into_owned();
        let path = cookie.path.as_ref().to_owned();
        let mut store = self.0.lock().expect("cookie store lock poisoned");
        if store
            .get_any(&domain, &path, cookie.name())
            .is_some_and(|existing| existing.http_only() == Some(true))
        {
            return Ok(());
        }
        match store.insert(cookie, &url) {
            Ok(_) | Err(cookie_store::CookieError::Expired) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn header(&self, url: &str) -> Option<String> {
        let url = url::Url::parse(url).ok()?;
        let value = self
            .0
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        (!value.is_empty()).then_some(value)
    }

    pub fn for_url(&self, url: &str) -> Vec<(String, String)> {
        let Ok(url) = url::Url::parse(url) else {
            return Vec::new();
        };
        self.0
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }

    pub fn matching(&self, url: &str) -> Vec<StoredCookie> {
        let Ok(url) = url::Url::parse(url) else {
            return Vec::new();
        };
        self.0
            .lock()
            .expect("cookie store lock poisoned")
            .matches(&url)
            .into_iter()
            .map(stored_cookie)
            .collect()
    }

    pub fn all(&self) -> Vec<StoredCookie> {
        self.0
            .lock()
            .expect("cookie store lock poisoned")
            .iter_unexpired()
            .map(stored_cookie)
            .collect()
    }

    pub fn delete(&self, name: &str, url: Option<&str>, domain: Option<&str>, path: Option<&str>) {
        let parsed_url = url.and_then(|value| url::Url::parse(value).ok());
        let mut store = self.0.lock().expect("cookie store lock poisoned");
        let keys = store
            .iter_unexpired()
            .filter(|cookie| cookie.name() == name)
            .filter(|cookie| parsed_url.as_ref().is_none_or(|url| cookie.matches(url)))
            .filter(|cookie| {
                domain.is_none_or(|domain| {
                    cookie.domain.as_cow().as_deref() == Some(domain.trim_start_matches('.'))
                })
            })
            .filter(|cookie| path.is_none_or(|path| cookie.path.as_ref() == path))
            .filter_map(|cookie| {
                Some((
                    cookie.domain.as_cow()?.into_owned(),
                    cookie.path.as_ref().to_owned(),
                    cookie.name().to_owned(),
                ))
            })
            .collect::<Vec<_>>();
        for (domain, path, name) in keys {
            store.remove(&domain, &path, &name);
        }
    }

    pub fn clear(&self) {
        *self.0.lock().expect("cookie store lock poisoned") = cookie_store::CookieStore::default();
    }
}

fn stored_cookie(cookie: &cookie_store::Cookie<'static>) -> StoredCookie {
    let (domain, host_only) = match &cookie.domain {
        cookie_store::CookieDomain::HostOnly(domain) => (domain.clone(), true),
        cookie_store::CookieDomain::Suffix(domain) => (format!(".{domain}"), false),
        domain => (domain.as_cow().unwrap_or_default().into_owned(), false),
    };
    StoredCookie {
        name: cookie.name().to_owned(),
        value: cookie.value().to_owned(),
        domain,
        host_only,
        path: cookie.path.as_ref().to_owned(),
        expires: match cookie.expires {
            cookie_store::CookieExpiration::AtUtc(value) => Some(value.unix_timestamp()),
            cookie_store::CookieExpiration::SessionEnd => None,
        },
        http_only: cookie.http_only() == Some(true),
        secure: cookie.secure() == Some(true),
        same_site: cookie.same_site().map(|value| format!("{value:?}")),
    }
}

pub struct BrowsingContext {
    url: Mutex<Option<String>>,
    pending_navigation: Mutex<Option<String>>,
    cookies: Arc<CookieJar>,
    request_headers: Mutex<Vec<(http::HeaderName, http::HeaderValue)>>,
    resource_cors: Mutex<HashMap<String, ResourceCorsPolicy>>,
}

impl Default for BrowsingContext {
    fn default() -> Self {
        Self::with_cookie_jar(Arc::new(CookieJar::default()))
    }
}

#[derive(Clone, Debug, Default)]
struct ResourceCorsPolicy {
    allow_origin: Option<String>,
    allow_credentials: bool,
    credentials_sent: bool,
}

impl BrowsingContext {
    pub fn with_cookie_jar(cookies: Arc<CookieJar>) -> Self {
        Self {
            url: Mutex::new(None),
            pending_navigation: Mutex::new(None),
            cookies,
            request_headers: Mutex::new(Vec::new()),
            resource_cors: Mutex::new(HashMap::new()),
        }
    }
    pub fn store_resource_cors(
        &self,
        requested_url: &str,
        effective_url: &str,
        headers: &network::HeaderList,
        credentials_sent: bool,
    ) {
        let policy = ResourceCorsPolicy {
            allow_origin: headers
                .get(http::header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .map(str::to_owned),
            allow_credentials: headers
                .get(http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("true")),
            credentials_sent,
        };
        let mut policies = self
            .resource_cors
            .lock()
            .expect("resource CORS policy lock poisoned");
        policies.insert(requested_url.to_owned(), policy.clone());
        policies.insert(effective_url.to_owned(), policy);
    }

    fn resource_origin_clean(&self, source_url: &str, cors_mode: Option<&str>) -> bool {
        let Ok(source) = url::Url::parse(source_url) else {
            return false;
        };
        if source.scheme() == "data" {
            return true;
        }
        let document = self
            .current_url()
            .and_then(|url| url::Url::parse(&url).ok());
        if document
            .as_ref()
            .is_some_and(|document| document.origin() == source.origin())
        {
            return true;
        }
        let Some(cors_mode) = cors_mode else {
            return false;
        };
        let Some(document_origin) = document.map(|url| url.origin().ascii_serialization()) else {
            return false;
        };
        let policies = self
            .resource_cors
            .lock()
            .expect("resource CORS policy lock poisoned");
        let Some(policy) = policies.get(source_url) else {
            return false;
        };
        let exact_origin = policy.allow_origin.as_deref() == Some(document_origin.as_str());
        match cors_mode {
            "use-credentials" => exact_origin && policy.allow_credentials,
            _ if policy.credentials_sent => exact_origin && policy.allow_credentials,
            _ => exact_origin || policy.allow_origin.as_deref() == Some("*"),
        }
    }

    pub fn set_request_headers(
        &self,
        headers: impl IntoIterator<Item = (String, String)>,
    ) -> Result<(), String> {
        let headers = headers
            .into_iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(name, value)| {
                Ok((
                    http::HeaderName::from_bytes(name.as_bytes())
                        .map_err(|error| error.to_string())?,
                    http::HeaderValue::from_str(&value).map_err(|error| error.to_string())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        *self
            .request_headers
            .lock()
            .expect("request identity lock poisoned") = headers;
        Ok(())
    }

    pub fn apply_request_identity(&self, headers: &mut network::HeaderList) {
        for (name, value) in self
            .request_headers
            .lock()
            .expect("request identity lock poisoned")
            .iter()
        {
            if !headers.contains_key(name) {
                headers.insert(name.clone(), value.clone());
            }
        }
    }

    pub fn set_url(&self, url: impl Into<String>) {
        *self.url.lock().expect("browsing URL lock poisoned") = Some(url.into());
    }

    pub fn current_url(&self) -> Option<String> {
        self.url.lock().expect("browsing URL lock poisoned").clone()
    }

    pub fn request_navigation(&self, url: impl Into<String>) {
        *self
            .pending_navigation
            .lock()
            .expect("pending navigation lock poisoned") = Some(url.into());
    }

    pub fn take_pending_navigation(&self) -> Option<String> {
        self.pending_navigation
            .lock()
            .expect("pending navigation lock poisoned")
            .take()
    }

    pub fn store_response_cookie(&self, url: &str, header: &str) {
        let _ = self.cookies.store(url, header);
    }

    pub fn cookie_header(&self, url: &str) -> Option<String> {
        self.cookies.header(url)
    }

    pub fn cookies_for_url(&self, url: &str) -> Vec<(String, String)> {
        self.cookies.for_url(url)
    }

    fn document_cookies(&self) -> String {
        let raw_url = self.url.lock().expect("browsing URL lock poisoned");
        let Some(url) = raw_url.as_deref().and_then(|url| url::Url::parse(url).ok()) else {
            return String::new();
        };
        self.cookies
            .0
            .lock()
            .expect("cookie store lock poisoned")
            .matches(&url)
            .into_iter()
            .filter(|cookie| cookie.http_only() != Some(true))
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn set_document_cookie(&self, header: &str) {
        let url = self.url.lock().expect("browsing URL lock poisoned").clone();
        if let Some(url) = url {
            let _ = self.cookies.store_document(&url, header);
        }
    }
}

struct BindingState {
    document: Rc<RefCell<BrowserDocument>>,
    wrappers: WrapperCache,
    style_wrappers: WrapperCache,
    computed_style_wrappers: WrapperCache,
    prototypes: RefCell<Option<Prototypes>>,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    fetches: Rc<RefCell<FetchQueue>>,
    features: WebFeatureFlags,
    storage: Option<Arc<PersistentStorage>>,
    workers: Rc<RefCell<WorkerQueue>>,
    worker_delivery: RefCell<Option<ProtectedJsObject>>,
    streaming: Rc<RefCell<StreamingQueue>>,
    websocket_delivery: RefCell<Option<ProtectedJsObject>>,
    fetch_stream_delivery: RefCell<Option<ProtectedJsObject>>,
    canvases: RefCell<CanvasStore>,
    audio: RefCell<AudioStore>,
    gpu: RefCell<GpuStore>,
    angles: RefCell<AngleStore>,
}

struct Prototypes {
    document: ProtectedJsObject,
    html_element: ProtectedJsObject,
    html_anchor_element: ProtectedJsObject,
    html_base_element: ProtectedJsObject,
    specialized_html_elements: HashMap<String, ProtectedJsObject>,
    text: ProtectedJsObject,
    comment: ProtectedJsObject,
    document_fragment: ProtectedJsObject,
    css_style: ProtectedJsObject,
}

impl BindingRuntime {
    pub fn canvas_rasters(&self) -> Result<Vec<crate::canvas::CanvasRaster>, String> {
        let _angle_guard = crate::angle::lock();
        let webgl = self.state.canvases.borrow().webgl_dimensions();
        for (id, width, height) in webgl {
            if width == 0 || height == 0 {
                continue;
            }
            let mut pixels = self
                .state
                .angles
                .borrow()
                .read_canvas_rgba(id, 0, 0, width, height)?;
            flip_rows(&mut pixels, width, height);
            self.state
                .canvases
                .borrow_mut()
                .write_rgba(id, width, height, 0, 0, width, height, &pixels)?;
        }
        Ok(self.state.canvases.borrow_mut().rasters())
    }

    pub fn install(
        runtime: &JsRuntime,
        document: Rc<RefCell<BrowserDocument>>,
        browsing_context: Arc<BrowsingContext>,
        cross_origin_isolated: bool,
        features: WebFeatureFlags,
        storage: Option<Arc<PersistentStorage>>,
        queues: BindingQueues,
    ) -> Result<Self, JsException> {
        let state = Rc::new(BindingState {
            document,
            wrappers: WrapperCache::default(),
            style_wrappers: WrapperCache::default(),
            computed_style_wrappers: WrapperCache::default(),
            prototypes: RefCell::new(None),
            timers: queues.timers,
            browsing_context,
            fetches: queues.fetches,
            features,
            storage,
            workers: queues.workers,
            worker_delivery: RefCell::new(None),
            streaming: queues.streaming,
            websocket_delivery: RefCell::new(None),
            fetch_stream_delivery: RefCell::new(None),
            canvases: RefCell::new(CanvasStore::default()),
            audio: RefCell::new(AudioStore::default()),
            gpu: RefCell::new(GpuStore::default()),
            angles: RefCell::new(AngleStore::default()),
        });
        let callback_state = Rc::clone(&state);
        runtime.set_global_function("__brimp", move |call| dispatch(&callback_state, &call))?;
        runtime.eval(CLASS_DEFINITIONS)?;
        if features.worker_system {
            let worker_state = Rc::clone(&state);
            runtime.set_global_function("__brimpWorkerHost", move |call| {
                dispatch(&worker_state, &call)
            })?;
            runtime.eval(include_str!("worker_system.js"))?;
            *state.worker_delivery.borrow_mut() = Some(
                runtime
                    .eval("globalThis.__brimpDeliverWorker")?
                    .to_object()?,
            );
            runtime.eval("delete globalThis.__brimpDeliverWorker")?;
            runtime.eval("delete globalThis.__brimpWorkerHost")?;
        }
        if features.persistent_storage {
            let storage_state = Rc::clone(&state);
            runtime.set_global_function("__brimpStorageHost", move |call| {
                dispatch(&storage_state, &call)
            })?;
            runtime.eval(include_str!("persistent_storage.js"))?;
            runtime.eval("delete globalThis.__brimpStorageHost")?;
        }
        if features.streaming_networking {
            let streaming_state = Rc::clone(&state);
            runtime.set_global_function("__brimpStreamingHost", move |call| {
                dispatch(&streaming_state, &call)
            })?;
            runtime.eval(include_str!("streaming_networking.js"))?;
            *state.websocket_delivery.borrow_mut() = Some(
                runtime
                    .eval("globalThis.__brimpDeliverWebSocket")?
                    .to_object()?,
            );
            *state.fetch_stream_delivery.borrow_mut() = Some(
                runtime
                    .eval("globalThis.__brimpDeliverFetchStream")?
                    .to_object()?,
            );
            runtime.eval("delete globalThis.__brimpDeliverWebSocket")?;
            runtime.eval("delete globalThis.__brimpDeliverFetchStream")?;
            runtime.eval("delete globalThis.__brimpStreamingHost")?;
        }
        if features.canvas || features.webgl || features.webgpu {
            let canvas_state = Rc::clone(&state);
            runtime.set_global_function("__brimpCanvasHost", move |call| {
                dispatch(&canvas_state, &call)
            })?;
            runtime.eval(include_str!("canvas.js"))?;
            runtime.eval("delete globalThis.__brimpCanvasHost")?;
        }
        if features.webaudio {
            let audio_state = Rc::clone(&state);
            runtime.set_global_function("__brimpAudioHost", move |call| {
                dispatch(&audio_state, &call)
            })?;
            runtime.eval(include_str!("audio.js"))?;
            runtime.eval("delete globalThis.__brimpAudioHost")?;
        }
        if features.webgpu {
            let gpu_state = Rc::clone(&state);
            runtime
                .set_global_function("__brimpGpuHost", move |call| dispatch(&gpu_state, &call))?;
            runtime.eval(include_str!("gpu.js"))?;
            runtime.eval("delete globalThis.__brimpGpuHost")?;
        }
        if features.webgl {
            let webgl_state = Rc::clone(&state);
            runtime.set_global_function("__brimpWebGlHost", move |call| {
                dispatch(&webgl_state, &call)
            })?;
            runtime.eval(include_str!("webgl.js"))?;
            runtime.eval("delete globalThis.__brimpWebGlHost")?;
        }
        runtime.eval("delete globalThis.__brimpMarkTrustedEvent")?;
        runtime.eval(if cross_origin_isolated {
            include_str!("cross_origin_isolated.js")
        } else {
            include_str!("cross_origin_unisolated.js")
        })?;

        *state.prototypes.borrow_mut() = Some(Prototypes {
            document: runtime.eval("Document.prototype")?.to_object()?,
            html_element: runtime.eval("HTMLElement.prototype")?.to_object()?,
            html_anchor_element: runtime.eval("HTMLAnchorElement.prototype")?.to_object()?,
            html_base_element: runtime.eval("HTMLBaseElement.prototype")?.to_object()?,
            specialized_html_elements: [
                ("picture", "HTMLPictureElement"),
                ("img", "HTMLImageElement"),
                ("iframe", "HTMLIFrameElement"),
                ("embed", "HTMLEmbedElement"),
                ("object", "HTMLObjectElement"),
                ("param", "HTMLParamElement"),
                ("video", "HTMLVideoElement"),
                ("audio", "HTMLAudioElement"),
                ("source", "HTMLSourceElement"),
                ("track", "HTMLTrackElement"),
                ("canvas", "HTMLCanvasElement"),
                ("map", "HTMLMapElement"),
                ("area", "HTMLAreaElement"),
                ("form", "HTMLFormElement"),
                ("fieldset", "HTMLFieldSetElement"),
                ("legend", "HTMLLegendElement"),
                ("label", "HTMLLabelElement"),
                ("input", "HTMLInputElement"),
                ("button", "HTMLButtonElement"),
                ("select", "HTMLSelectElement"),
                ("datalist", "HTMLDataListElement"),
                ("optgroup", "HTMLOptGroupElement"),
                ("option", "HTMLOptionElement"),
                ("textarea", "HTMLTextAreaElement"),
                ("output", "HTMLOutputElement"),
                ("progress", "HTMLProgressElement"),
                ("meter", "HTMLMeterElement"),
                ("table", "HTMLTableElement"),
                ("caption", "HTMLTableCaptionElement"),
                ("colgroup", "HTMLTableColElement"),
                ("col", "HTMLTableColElement"),
                ("tbody", "HTMLTableSectionElement"),
                ("thead", "HTMLTableSectionElement"),
                ("tfoot", "HTMLTableSectionElement"),
                ("tr", "HTMLTableRowElement"),
                ("td", "HTMLTableCellElement"),
                ("th", "HTMLTableCellElement"),
                ("head", "HTMLHeadElement"),
                ("title", "HTMLTitleElement"),
                ("link", "HTMLLinkElement"),
                ("meta", "HTMLMetaElement"),
                ("style", "HTMLStyleElement"),
                ("p", "HTMLParagraphElement"),
                ("hr", "HTMLHRElement"),
                ("pre", "HTMLPreElement"),
                ("blockquote", "HTMLQuoteElement"),
                ("q", "HTMLQuoteElement"),
                ("ol", "HTMLOListElement"),
                ("ul", "HTMLUListElement"),
                ("li", "HTMLLIElement"),
                ("dl", "HTMLDListElement"),
                ("div", "HTMLDivElement"),
                ("data", "HTMLDataElement"),
                ("time", "HTMLTimeElement"),
                ("br", "HTMLBRElement"),
                ("body", "HTMLBodyElement"),
                ("h1", "HTMLHeadingElement"),
                ("h2", "HTMLHeadingElement"),
                ("h3", "HTMLHeadingElement"),
                ("h4", "HTMLHeadingElement"),
                ("h5", "HTMLHeadingElement"),
                ("h6", "HTMLHeadingElement"),
                ("html", "HTMLHtmlElement"),
                ("script", "HTMLScriptElement"),
                ("template", "HTMLTemplateElement"),
                ("slot", "HTMLSlotElement"),
                ("ins", "HTMLModElement"),
                ("del", "HTMLModElement"),
                ("details", "HTMLDetailsElement"),
                ("menu", "HTMLMenuElement"),
                ("dialog", "HTMLDialogElement"),
                ("marquee", "HTMLMarqueeElement"),
                ("frameset", "HTMLFrameSetElement"),
                ("frame", "HTMLFrameElement"),
                ("dir", "HTMLDirectoryElement"),
                ("font", "HTMLFontElement"),
            ]
            .into_iter()
            .map(|(tag, class)| {
                runtime
                    .eval(&format!("{class}.prototype"))
                    .and_then(|value| value.to_object())
                    .map(|prototype| (tag.to_owned(), prototype))
            })
            .collect::<Result<_, _>>()?,
            text: runtime.eval("Text.prototype")?.to_object()?,
            comment: runtime.eval("Comment.prototype")?.to_object()?,
            document_fragment: runtime.eval("DocumentFragment.prototype")?.to_object()?,
            css_style: runtime.eval("CSSStyleProperties.prototype")?.to_object()?,
        });
        let input_controller = runtime
            .eval("globalThis.__brimpInputController")?
            .to_object()?;
        runtime.eval("delete globalThis.__brimpInputController")?;
        let bindings = Self {
            state,
            input_controller,
        };
        bindings.reset_document(runtime)?;
        Ok(bindings)
    }

    pub fn dispatch_input(
        &self,
        runtime: &JsRuntime,
        serialized_command: &str,
    ) -> Result<String, JsException> {
        runtime
            .call_function_with_string(&self.input_controller, serialized_command)?
            .to_string()
    }

    pub fn dispatch_input_on(
        &self,
        runtime: &JsRuntime,
        serialized_command: &str,
        target_expression: &str,
    ) -> Result<String, JsException> {
        let target = runtime.eval(target_expression)?.to_object()?;
        runtime
            .call_function_with_string_and_object(
                &self.input_controller,
                serialized_command,
                &target,
            )?
            .to_string()
    }

    pub fn reset_document(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        self.state.wrappers.clear();
        self.state.style_wrappers.clear();
        self.state.computed_style_wrappers.clear();
        self.state.canvases.borrow_mut().clear();
        self.state.audio.borrow_mut().clear();
        *self.state.angles.borrow_mut() = AngleStore::default();
        let document_id = self.state.document.borrow().root().id;
        let prototype = self
            .state
            .prototypes
            .borrow()
            .as_ref()
            .expect("bindings are initialized")
            .document
            .identity();
        let document =
            self.state
                .wrappers
                .wrap_with_runtime_prototype(runtime, document_id, prototype);
        runtime.set_global_object("document", &document)?;
        runtime.eval("window.document = document")?;
        Ok(())
    }

    pub fn sync_window_named_properties(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        let names = self.state.document.borrow().window_named_properties();
        let names = serde_json::to_string(&names).expect("window names serialize as JSON");
        let script = include_str!("sync_window_named_properties.js").replace("__NAMES__", &names);
        runtime.eval(&script)?;
        Ok(())
    }

    pub fn wrapper_cache(&self) -> &WrapperCache {
        &self.state.wrappers
    }

    pub fn deliver_worker_event(
        &self,
        runtime: &JsRuntime,
        id: u64,
        serialized_event: &str,
    ) -> Result<(), JsException> {
        let delivery = self.state.worker_delivery.borrow();
        let delivery = delivery
            .as_ref()
            .ok_or_else(|| JsException::from_message("worker delivery is unavailable"))?;
        runtime
            .call_function_with_string(
                delivery,
                &serde_json::json!({"id": id, "event": serialized_event}).to_string(),
            )
            .map(|_| ())
    }

    pub fn deliver_websocket_event(
        &self,
        runtime: &JsRuntime,
        id: u64,
        serialized_event: &str,
    ) -> Result<(), JsException> {
        let delivery = self.state.websocket_delivery.borrow();
        let delivery = delivery
            .as_ref()
            .ok_or_else(|| JsException::from_message("WebSocket delivery is unavailable"))?;
        runtime
            .call_function_with_string(
                delivery,
                &serde_json::json!({"id": id, "event": serialized_event}).to_string(),
            )
            .map(|_| ())
    }

    pub fn deliver_fetch_stream_event(
        &self,
        runtime: &JsRuntime,
        id: u64,
        serialized_event: &str,
    ) -> Result<(), JsException> {
        let delivery = self.state.fetch_stream_delivery.borrow();
        let delivery = delivery
            .as_ref()
            .ok_or_else(|| JsException::from_message("Fetch stream delivery is unavailable"))?;
        runtime
            .call_function_with_string(
                delivery,
                &serde_json::json!({"id": id, "event": serialized_event}).to_string(),
            )
            .map(|_| ())
    }
}

fn dispatch(state: &BindingState, call: &NativeCall<'_>) -> Result<NativeValue, NativeError> {
    let operation = required_string(call, 0, "operation")?;
    let _angle_guard = operation_uses_angle(&operation).then(crate::angle::lock);
    match operation.as_str() {
        "runtimeFeatures" => Ok(NativeValue::String(state.features.json())),
        operation if operation == "canvasFeatures" || operation.starts_with("canvas") => {
            dispatch_canvas::dispatch(state, call, operation)
        }
        operation if operation.starts_with("audio") => {
            dispatch_audio::dispatch(state, call, operation)
        }
        operation if operation.starts_with("gpu") => dispatch_gpu::dispatch(state, call, operation),
        operation if operation.starts_with("webgl") => {
            dispatch_webgl::dispatch(state, call, operation)
        }
        operation if is_platform_operation(operation) => {
            dispatch_platform::dispatch(state, call, operation)
        }
        operation => dispatch_dom::dispatch(state, call, operation),
    }
}

fn is_platform_operation(operation: &str) -> bool {
    operation.starts_with("worker")
        || operation.starts_with("webSocket")
        || operation.starts_with("persistent")
        || operation.starts_with("url")
        || operation.starts_with("encoding")
        || operation.starts_with("legacyQuery")
        || operation.starts_with("base64")
        || operation.starts_with("crypto")
        || operation.starts_with("fetch")
        || matches!(
            operation,
            "setTimeout"
                | "setInterval"
                | "clearTimeout"
                | "clearInterval"
                | "queueMicrotask"
                | "location"
                | "locationNavigate"
                | "historyUpdateUrl"
                | "formUrlEncode"
                | "decodeBytes"
                | "encodeUtf8"
        )
}

fn operation_uses_angle(operation: &str) -> bool {
    operation.starts_with("webgl")
        || matches!(
            operation,
            "canvasReset"
                | "canvas2dDrawCanvas"
                | "canvasCreateImageBitmap"
                | "canvas2dCreatePattern"
                | "canvasEncode"
        )
}

fn decode_bytes(
    encoding: &'static encoding_rs::Encoding,
    bytes: &[u8],
    fatal: bool,
    ignore_bom: bool,
    last: bool,
) -> Result<Option<String>, NativeError> {
    let mut decoder = if ignore_bom {
        encoding.new_decoder_without_bom_handling()
    } else {
        encoding.new_decoder_with_bom_removal()
    };
    let capacity = if fatal {
        decoder.max_utf8_buffer_length_without_replacement(bytes.len())
    } else {
        decoder.max_utf8_buffer_length(bytes.len())
    }
    .ok_or_else(|| NativeError::new("decoded text is too large"))?;
    let mut output = String::with_capacity(capacity);
    if fatal {
        let (result, read) = decoder.decode_to_string_without_replacement(bytes, &mut output, last);
        match result {
            encoding_rs::DecoderResult::InputEmpty if read == bytes.len() => Ok(Some(output)),
            encoding_rs::DecoderResult::Malformed(_, _) => Ok(None),
            encoding_rs::DecoderResult::OutputFull => {
                Err(NativeError::new("decoder output buffer was too small"))
            }
            _ => Err(NativeError::new("decoder did not consume its input")),
        }
    } else {
        let (result, read, _) = decoder.decode_to_string(bytes, &mut output, last);
        if result == encoding_rs::CoderResult::InputEmpty && read == bytes.len() {
            Ok(Some(output))
        } else {
            Err(NativeError::new("decoder did not consume its input"))
        }
    }
}

fn legacy_query_encode(encoding: &'static encoding_rs::Encoding, input: &str) -> String {
    let prepared = prepare_legacy_input(encoding, input);
    let input = prepared.as_deref().unwrap_or(input);
    let (bytes, _, _) = encoding.encode(input);
    let mut output = String::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'&'
            && bytes.get(index + 1) == Some(&b'#')
            && bytes[index + 2..]
                .iter()
                .position(|byte| *byte == b';')
                .is_some_and(|end| {
                    bytes[index + 2..index + 2 + end]
                        .iter()
                        .all(u8::is_ascii_digit)
                })
        {
            let end = index
                + 2
                + bytes[index + 2..]
                    .iter()
                    .position(|byte| *byte == b';')
                    .expect("numeric character reference terminator was checked");
            output.push_str("%26%23");
            output.push_str(std::str::from_utf8(&bytes[index + 2..end]).expect("digits are UTF-8"));
            output.push_str("%3B");
            index = end + 1;
            continue;
        }
        let byte = bytes[index];
        if byte > 0x20 && byte <= 0x7E && !matches!(byte, b'"' | b'#' | b'\'' | b'<' | b'>') {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(output, "%{byte:02X}").expect("writing to a String cannot fail");
        }
        index += 1;
    }
    output
}

fn form_url_encode(encoding: &'static encoding_rs::Encoding, input: &str) -> String {
    let prepared = prepare_legacy_input(encoding, input);
    let input = prepared.as_deref().unwrap_or(input);
    let (bytes, _, _) = encoding.encode(input);
    let mut output = String::with_capacity(bytes.len());
    for byte in bytes.iter().copied() {
        if byte == b' ' {
            output.push('+');
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'*' | b'-' | b'.' | b'_') {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(output, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    output
}

fn prepare_legacy_input(encoding: &'static encoding_rs::Encoding, input: &str) -> Option<String> {
    (encoding == encoding_rs::ISO_2022_JP
        && input
            .chars()
            .any(|character| matches!(character, '\u{000E}' | '\u{000F}' | '\u{001B}')))
    .then(|| {
        input
            .chars()
            .map(|character| {
                if matches!(character, '\u{000E}' | '\u{000F}' | '\u{001B}') {
                    '\u{FFFD}'
                } else {
                    character
                }
            })
            .collect()
    })
}

fn prototypes(state: &BindingState) -> std::cell::Ref<'_, Prototypes> {
    std::cell::Ref::map(state.prototypes.borrow(), |prototypes| {
        prototypes.as_ref().expect("bindings are initialized")
    })
}

fn required_string(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<String, NativeError> {
    call.argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_string()
}

fn required_numbers<const N: usize>(
    call: &NativeCall<'_>,
    start: usize,
    label: &str,
) -> Result<[f64; N], NativeError> {
    let mut values = [0.0; N];
    for (offset, value) in values.iter_mut().enumerate() {
        *value = call
            .argument(start + offset)
            .ok_or_else(|| NativeError::new(format!("missing {label}")))?
            .to_number()?;
    }
    Ok(values)
}

fn required_number(call: &NativeCall<'_>, index: usize, label: &str) -> Result<f64, NativeError> {
    call.argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_number()
}

fn required_boolean(call: &NativeCall<'_>, index: usize, label: &str) -> Result<bool, NativeError> {
    Ok(call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_boolean())
}

fn required_u32(call: &NativeCall<'_>, index: usize, label: &str) -> Result<u32, NativeError> {
    let value = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_number()?;
    if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(value.trunc() as u32)
}

fn required_u64(call: &NativeCall<'_>, index: usize, label: &str) -> Result<u64, NativeError> {
    let value = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_number()?;
    if !value.is_finite() || value < 0.0 || value > u64::MAX as f64 || value.fract() != 0.0 {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(value as u64)
}

fn required_i32(call: &NativeCall<'_>, index: usize, label: &str) -> Result<i32, NativeError> {
    let value = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_number()?;
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(value.trunc() as i32)
}

fn required_f32_array(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<Vec<f32>, NativeError> {
    let bytes = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_bytes()?;
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(bytes
        .chunks_exact(std::mem::size_of::<f32>())
        .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("four-byte f32 chunk")))
        .collect())
}

fn required_i32_array(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<Vec<i32>, NativeError> {
    let bytes = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_bytes()?;
    if bytes.len() % std::mem::size_of::<i32>() != 0 {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(bytes
        .chunks_exact(std::mem::size_of::<i32>())
        .map(|chunk| i32::from_ne_bytes(chunk.try_into().expect("four-byte i32 chunk")))
        .collect())
}

fn required_u32_array(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<Vec<u32>, NativeError> {
    let bytes = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_bytes()?;
    if bytes.len() % std::mem::size_of::<u32>() != 0 {
        return Err(NativeError::new(format!("invalid {label}")));
    }
    Ok(bytes
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("four-byte u32 chunk")))
        .collect())
}

fn canvas_dimensions(call: &NativeCall<'_>) -> Result<(u32, u32), NativeError> {
    Ok((
        required_u32(call, 2, "canvas width")?,
        required_u32(call, 3, "canvas height")?,
    ))
}

fn flip_rows(pixels: &mut [u8], width: u32, height: u32) {
    let row_bytes = width as usize * 4;
    for top in 0..height as usize / 2 {
        let bottom = height as usize - top - 1;
        let (before_bottom, bottom_and_after) = pixels.split_at_mut(bottom * row_bytes);
        before_bottom[top * row_bytes..(top + 1) * row_bytes]
            .swap_with_slice(&mut bottom_and_after[..row_bytes]);
    }
}

fn crop_rgba(
    pixels: &[u8],
    source_width: u32,
    source_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, NativeError> {
    let end_x = x
        .checked_add(width)
        .ok_or_else(|| NativeError::new("external image crop x range overflow"))?;
    let end_y = y
        .checked_add(height)
        .ok_or_else(|| NativeError::new("external image crop y range overflow"))?;
    if end_x > source_width || end_y > source_height {
        return Err(NativeError::new(
            "external image crop exceeds the source dimensions",
        ));
    }
    let source_stride = usize::try_from(source_width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| NativeError::new("external image row size overflow"))?;
    let row_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or_else(|| NativeError::new("external image crop row size overflow"))?;
    let output_len = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(|| NativeError::new("external image crop size overflow"))?;
    let x = usize::try_from(x)
        .ok()
        .and_then(|x| x.checked_mul(4))
        .ok_or_else(|| NativeError::new("external image crop offset overflow"))?;
    let mut output = Vec::with_capacity(output_len);
    for row in y..end_y {
        let start = (row as usize)
            .checked_mul(source_stride)
            .and_then(|start| start.checked_add(x))
            .ok_or_else(|| NativeError::new("external image crop offset overflow"))?;
        let end = start
            .checked_add(row_bytes)
            .ok_or_else(|| NativeError::new("external image crop offset overflow"))?;
        output.extend_from_slice(
            pixels
                .get(start..end)
                .ok_or_else(|| NativeError::new("external image pixels are incomplete"))?,
        );
    }
    Ok(output)
}

fn premultiply_rgba(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);
        for component in &mut pixel[..3] {
            *component = ((u16::from(*component) * alpha + 127) / 255) as u8;
        }
    }
}

fn required_object(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<JsObjectIdentity, NativeError> {
    call.argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .as_object()?
        .ok_or_else(|| NativeError::new(format!("{label} must be an object")))
}

fn required_node_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, 1, "node receiver")?;
    state
        .wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("receiver is not a native Node"))
}

fn required_document_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    if state.document.borrow().is_document(id) {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not a Document"))
    }
}

fn required_parent_node_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    let is_parent_node = state.document.borrow().node(id).is_some_and(|node| {
        matches!(
            node.data,
            NodeData::Document | NodeData::Element(_) | NodeData::AnonymousBlock(_)
        )
    });
    if is_parent_node {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not a ParentNode"))
    }
}

fn required_element_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    if state
        .document
        .borrow()
        .node(id)
        .is_some_and(|node| node.is_element())
    {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not an Element"))
    }
}

fn required_canvas_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_element_target(state, call)?;
    if state.document.borrow().node(id).is_some_and(|node| {
        matches!(&node.data, NodeData::Element(element) if element.name.local.as_ref() == "canvas")
    }) {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not an HTMLCanvasElement"))
    }
}

fn required_canvas_argument(
    state: &BindingState,
    call: &NativeCall<'_>,
    index: usize,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, index, "canvas argument")?;
    let id = state
        .wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("argument is not a native Node"))?;
    if state.document.borrow().node(id).is_some_and(|node| {
        matches!(&node.data, NodeData::Element(element) if element.name.local.as_ref() == "canvas")
    }) {
        Ok(id)
    } else {
        Err(NativeError::new("argument is not an HTMLCanvasElement"))
    }
}

fn resolve_element_attribute_url(
    document: &BrowserDocument,
    browsing_context: &BrowsingContext,
    id: NodeId,
    attribute: &str,
) -> Option<String> {
    let input = document
        .node(id)?
        .element_data()?
        .attr(LocalName::from(attribute))?;
    let document_url = browsing_context
        .current_url()
        .and_then(|url| url::Url::parse(&url).ok());
    let base_url = document
        .query_selector("base[href]")
        .ok()
        .flatten()
        .and_then(|base_id| {
            document
                .node(base_id)
                .and_then(|node| node.element_data())
                .and_then(|base| base.attr(LocalName::from("href")))
        })
        .and_then(|base| {
            url::Url::options()
                .base_url(document_url.as_ref())
                .parse(base)
                .ok()
        })
        .or(document_url);
    url::Url::options()
        .base_url(base_url.as_ref())
        .parse(input)
        .ok()
        .map(|url| url.to_string())
}

fn required_image_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_element_target(state, call)?;
    if state.document.borrow().node(id).is_some_and(|node| {
        matches!(&node.data, NodeData::Element(element) if element.name.local.as_ref() == "img")
    }) {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not an HTMLImageElement"))
    }
}

fn required_image_argument(
    state: &BindingState,
    call: &NativeCall<'_>,
    index: usize,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, index, "image argument")?;
    let id = state
        .wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("argument is not a native Node"))?;
    if state.document.borrow().node(id).is_some_and(|node| {
        matches!(&node.data, NodeData::Element(element) if element.name.local.as_ref() == "img")
    }) {
        Ok(id)
    } else {
        Err(NativeError::new("argument is not an HTMLImageElement"))
    }
}

fn decoded_raster_image(
    state: &BindingState,
    image: NodeId,
) -> Result<(u32, u32, Vec<u8>), NativeError> {
    let document = state.document.borrow();
    let image = document
        .node(image)
        .and_then(|node| node.element_data())
        .and_then(|element| element.raster_image_data())
        .ok_or_else(|| NativeError::new("HTMLImageElement has no decoded raster image"))?;
    Ok((image.width, image.height, image.data.as_ref().to_vec()))
}

include!("runtime_graphics.rs");

fn required_style_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, 1, "style receiver")?;
    state
        .style_wrappers
        .node_id(object)
        .ok_or_else(|| NativeError::new("receiver is not a CSSStyleDeclaration"))
}

fn optional_node(
    state: &BindingState,
    call: &NativeCall<'_>,
    node: Option<NodeId>,
) -> Result<NativeValue, NativeError> {
    match node {
        Some(node) => node_value(state, call, node),
        None => Ok(NativeValue::Null),
    }
}

fn node_value(
    state: &BindingState,
    call: &NativeCall<'_>,
    node_id: NodeId,
) -> Result<NativeValue, NativeError> {
    let prototype = {
        let document = state.document.borrow();
        let node = document.node(node_id).ok_or_else(stale_wrapper)?;
        let prototypes = prototypes(state);
        if document.is_document_fragment(node_id) {
            prototypes.document_fragment.identity()
        } else {
            match node.data {
                NodeData::Document => prototypes.document.identity(),
                NodeData::Element(ref element) | NodeData::AnonymousBlock(ref element) => {
                    match element.name.local.as_ref() {
                        "a" => prototypes.html_anchor_element.identity(),
                        "base" => prototypes.html_base_element.identity(),
                        tag => prototypes
                            .specialized_html_elements
                            .get(tag)
                            .unwrap_or(&prototypes.html_element)
                            .identity(),
                    }
                }
                NodeData::Text(_) => prototypes.text.identity(),
                NodeData::Comment => prototypes.comment.identity(),
            }
        }
    };
    Ok(NativeValue::Object(
        state.wrappers.wrap_with_prototype(call, node_id, prototype),
    ))
}

fn node_array(
    state: &BindingState,
    call: &NativeCall<'_>,
    nodes: &[NodeId],
) -> Result<NativeValue, NativeError> {
    let wrappers = nodes
        .iter()
        .map(|node| match node_value(state, call, *node)? {
            NativeValue::Object(object) => Ok(object),
            _ => unreachable!(),
        })
        .collect::<Result<Vec<_>, NativeError>>()?;
    Ok(NativeValue::ProtectedObject(call.make_array(&wrappers)?))
}

fn cssom_json(result: Result<Vec<String>, CssomError>) -> Result<NativeValue, NativeError> {
    let value = match result {
        Ok(value) => serde_json::json!({ "value": value }),
        Err(error) => {
            let (name, message) = match error {
                CssomError::Syntax => ("SyntaxError", "The CSS rule is invalid"),
                CssomError::IndexSize => ("IndexSizeError", "The rule index is out of range"),
                CssomError::HierarchyRequest => (
                    "HierarchyRequestError",
                    "The CSS rule cannot be inserted at this position",
                ),
                CssomError::InvalidState => (
                    "InvalidStateError",
                    "The stylesheet is not in a state that permits this mutation",
                ),
                CssomError::NotAStyleSheet => ("InvalidStateError", "The node has no stylesheet"),
            };
            serde_json::json!({ "error": name, "message": message })
        }
    };
    Ok(NativeValue::String(
        serde_json::to_string(&value).map_err(err)?,
    ))
}

#[derive(Clone, Copy)]
enum ChildMutation {
    Append,
    Remove,
    InsertBefore,
}

fn mutate_child(
    state: &BindingState,
    call: &NativeCall<'_>,
    mutation: ChildMutation,
) -> Result<NativeValue, NativeError> {
    let parent = required_node_target(state, call)?;
    let child_object = required_object(call, 2, "child")?;
    let child = state
        .wrappers
        .node_id(child_object)
        .ok_or_else(|| NativeError::new("child is not a native Node"))?;
    let fragment_children = {
        let document = state.document.borrow();
        document.is_document_fragment(child).then(|| {
            document
                .node(child)
                .map(|node| node.children.clone())
                .unwrap_or_default()
        })
    };
    if let Some(children) = fragment_children.filter(|_| {
        matches!(
            mutation,
            ChildMutation::Append | ChildMutation::InsertBefore
        )
    }) {
        for child in &children {
            ensure_can_parent(&state.document.borrow(), parent, *child)?;
        }
        match mutation {
            ChildMutation::Append => {
                state
                    .document
                    .borrow_mut()
                    .blitz_mut()
                    .mutate()
                    .reparent_children(child, parent);
            }
            ChildMutation::InsertBefore => {
                let reference = call
                    .argument(3)
                    .ok_or_else(|| NativeError::new("missing reference node"))?;
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                if reference.is_null_or_undefined() {
                    mutator.reparent_children(child, parent);
                } else {
                    let reference = reference
                        .as_object()?
                        .and_then(|object| state.wrappers.node_id(object))
                        .ok_or_else(|| NativeError::new("reference is not a native Node"))?;
                    if mutator.parent_id(reference) != Some(parent) {
                        return Err(NativeError::new(
                            "reference node is not a child of this parent",
                        ));
                    }
                    for child in &children {
                        mutator.remove_node(*child);
                    }
                    mutator.insert_nodes_before(reference, &children);
                }
            }
            ChildMutation::Remove => unreachable!(),
        }
        let owner = state.document.borrow().node_document(parent).unwrap_or(0);
        let mut document = state.document.borrow_mut();
        for child in children {
            document.adopt_subtree(child, owner);
        }
        return Ok(NativeValue::Object(child_object));
    }
    ensure_can_parent(&state.document.borrow(), parent, child)?;

    match mutation {
        ChildMutation::Append => {
            let mut document = state.document.borrow_mut();
            let mut mutator = document.blitz_mut().mutate();
            if mutator.node_has_parent(child) {
                mutator.remove_node(child);
            }
            mutator.append_children(parent, &[child]);
        }
        ChildMutation::Remove => {
            let mut document = state.document.borrow_mut();
            let mut mutator = document.blitz_mut().mutate();
            if mutator.parent_id(child) != Some(parent) {
                return Err(NativeError::new("node is not a child of this parent"));
            }
            mutator.remove_node(child);
        }
        ChildMutation::InsertBefore => {
            let reference = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing reference node"))?;
            if reference.is_null_or_undefined() {
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                if mutator.node_has_parent(child) {
                    mutator.remove_node(child);
                }
                mutator.append_children(parent, &[child]);
            } else {
                let reference = reference
                    .as_object()?
                    .and_then(|object| state.wrappers.node_id(object))
                    .ok_or_else(|| NativeError::new("reference is not a native Node"))?;
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                if mutator.parent_id(reference) != Some(parent) {
                    return Err(NativeError::new(
                        "reference node is not a child of this parent",
                    ));
                }
                if reference != child {
                    if mutator.node_has_parent(child) {
                        mutator.remove_node(child);
                    }
                    mutator.insert_nodes_before(reference, &[child]);
                }
            }
        }
    }
    if matches!(
        mutation,
        ChildMutation::Append | ChildMutation::InsertBefore
    ) {
        let owner = state.document.borrow().node_document(parent).unwrap_or(0);
        state.document.borrow_mut().adopt_subtree(child, owner);
    }
    Ok(NativeValue::Object(child_object))
}

fn ensure_can_parent(
    document: &BrowserDocument,
    parent: NodeId,
    child: NodeId,
) -> Result<(), NativeError> {
    if parent == child {
        return Err(NativeError::new("a node cannot contain itself"));
    }
    let mut ancestor = document.node(parent).and_then(|node| node.parent);
    while let Some(id) = ancestor {
        if id == child {
            return Err(NativeError::new("operation would create a DOM cycle"));
        }
        ancestor = document.node(id).and_then(|node| node.parent);
    }
    Ok(())
}

fn set_text_content(state: &BindingState, node_id: NodeId, value: &str) -> Result<(), NativeError> {
    if state.document.borrow_mut().set_comment_data(node_id, value) {
        return Ok(());
    }
    let is_text = state
        .document
        .borrow()
        .node(node_id)
        .ok_or_else(stale_wrapper)?
        .is_text_node();
    if is_text {
        state
            .document
            .borrow_mut()
            .blitz_mut()
            .mutate()
            .set_node_text(node_id, value);
        return Ok(());
    }

    let removed = descendant_ids(&state.document.borrow(), node_id)?;
    let mut document = state.document.borrow_mut();
    let mut mutator = document.blitz_mut().mutate();
    mutator.remove_and_drop_all_children(node_id);
    if !value.is_empty() {
        let text = mutator.create_text_node(value);
        mutator.append_children(node_id, &[text]);
    }
    drop(mutator);
    document.remove_node_metadata(&removed);
    drop(document);
    state.wrappers.remove_nodes(&removed);
    state.style_wrappers.remove_nodes(&removed);
    Ok(())
}

fn descendant_ids(document: &BrowserDocument, node_id: NodeId) -> Result<Vec<NodeId>, NativeError> {
    fn collect(document: &BrowserDocument, node_id: NodeId, output: &mut Vec<NodeId>) {
        if let Some(node) = document.node(node_id) {
            for child in &node.children {
                output.push(*child);
                collect(document, *child, output);
            }
        }
    }
    if document.node(node_id).is_none() {
        return Err(stale_wrapper());
    }
    let mut output = Vec::new();
    collect(document, node_id, &mut output);
    Ok(output)
}

fn subtree_query_selector_all(
    document: &BrowserDocument,
    root_id: NodeId,
    selector: &str,
) -> Result<Vec<NodeId>, NativeError> {
    let root = document.node(root_id).ok_or_else(stale_wrapper)?;
    let selectors = document
        .blitz()
        .try_parse_selector_list(selector)
        .map_err(|error| NativeError::new(format!("{error:?}")))?;
    let mut selector_caches = SelectorCaches::default();
    let mut context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        &mut selector_caches,
        root.owner_doc().quirks_mode(),
        NeedsSelectorFlags::No,
        MatchingForInvalidation::No,
    );
    context.scope_element = root
        .as_element()
        .map(|element| selectors::Element::opaque(&element));

    fn collect(
        document: &BrowserDocument,
        node: &blitz_dom::Node,
        selectors: &selectors::SelectorList<style::selector_parser::SelectorImpl>,
        context: &mut MatchingContext<style::selector_parser::SelectorImpl>,
        output: &mut Vec<NodeId>,
    ) {
        for child_id in &node.children {
            let Some(child) = document.node(*child_id) else {
                continue;
            };
            if let Some(element) = child.as_element()
                && matches_selector_list(selectors, &element, context)
            {
                output.push(*child_id);
            }
            collect(document, child, selectors, context, output);
        }
    }

    let mut results = Vec::new();
    collect(document, root, &selectors, &mut context, &mut results);
    Ok(results)
}

fn inline_style_property(state: &BindingState, node_id: NodeId, name: &str) -> String {
    state
        .document
        .borrow()
        .inline_style_property(node_id, name)
        .unwrap_or_default()
}

fn resolve_document(state: &BindingState) {
    state.document.borrow_mut().resolve();
}

fn url_record_json(url: &url::Url) -> Result<String, NativeError> {
    let host = match (url.host_str(), url.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        (None, _) => String::new(),
    };
    serde_json::to_string(&serde_json::json!({
        "href": url.as_str(),
        "origin": url.origin().ascii_serialization(),
        "protocol": format!("{}:", url.scheme()),
        "username": url.username(),
        "password": url.password().unwrap_or_default(),
        "host": host,
        "hostname": url.host_str().unwrap_or_default(),
        "port": url.port().map(|port| port.to_string()).unwrap_or_default(),
        "pathname": url.path(),
        "search": url.query().map(|query| format!("?{query}")).unwrap_or_default(),
        "hash": url.fragment().map(|fragment| format!("#{fragment}")).unwrap_or_default(),
    }))
    .map_err(err)
}

fn set_url_component(href: &str, component: &str, value: &str) -> Result<String, NativeError> {
    if component == "href" {
        return url::Url::parse(value).map(|url| url.into()).map_err(err);
    }
    let mut url = url::Url::parse(href).map_err(err)?;
    match component {
        "protocol" => {
            let _ = url.set_scheme(value.trim_end_matches(':'));
        }
        "username" => {
            let _ = url.set_username(value);
        }
        "password" => {
            let _ = url.set_password(Some(value));
        }
        "host" => {
            if let Ok(host_url) = url::Url::parse(&format!("{}://{value}/", url.scheme())) {
                let _ = url.set_host(host_url.host_str());
                let _ = url.set_port(host_url.port());
            }
        }
        "hostname" => {
            let _ = url.set_host(Some(value));
        }
        "port" => {
            let port = if value.is_empty() {
                None
            } else if let Ok(port) = value.parse::<u16>() {
                Some(port)
            } else {
                return Ok(url.into());
            };
            let _ = url.set_port(port);
        }
        "pathname" => url.set_path(value),
        "search" => url.set_query((!value.is_empty()).then(|| value.trim_start_matches('?'))),
        "hash" => url.set_fragment((!value.is_empty()).then(|| value.trim_start_matches('#'))),
        _ => return Err(NativeError::new("unknown URL component")),
    }
    Ok(url.into())
}

fn stale_wrapper() -> NativeError {
    NativeError::new("native node no longer exists")
}

fn err(error: impl ToString) -> NativeError {
    NativeError::new(error)
}

fn persistent_storage(state: &BindingState) -> Result<&PersistentStorage, NativeError> {
    state
        .storage
        .as_deref()
        .ok_or_else(|| NativeError::new("persistent storage is disabled"))
}

fn storage_origin(state: &BindingState) -> Result<String, NativeError> {
    let url = state
        .browsing_context
        .current_url()
        .ok_or_else(|| NativeError::new("persistent storage requires a document URL"))?;
    let url = url::Url::parse(&url).map_err(err)?;
    let origin = url.origin().ascii_serialization();
    if origin == "null" {
        return Err(NativeError::new(
            "persistent storage is unavailable for opaque origins",
        ));
    }
    Ok(origin)
}

#[cfg(test)]
mod tests {
    use super::{CookieJar, flip_rows, premultiply_rgba};

    #[test]
    fn webgl_source_unpack_flips_and_premultiplies_rgba() {
        let mut pixels = vec![200, 100, 50, 128, 10, 20, 30, 255];
        flip_rows(&mut pixels, 1, 2);
        premultiply_rgba(&mut pixels);
        assert_eq!(pixels, [10, 20, 30, 255, 100, 50, 25, 128]);
    }

    #[test]
    fn document_cookie_cannot_create_or_overwrite_http_only_cookies() {
        let jar = CookieJar::default();
        jar.store("https://example.test/", "session=secret; Path=/; HttpOnly")
            .unwrap();
        jar.store_document("https://example.test/", "session=visible; Path=/")
            .unwrap();
        jar.store_document("https://example.test/", "created=hidden; Path=/; HttpOnly")
            .unwrap();
        assert_eq!(
            jar.header("https://example.test/").as_deref(),
            Some("session=secret")
        );
    }
}
