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
use style::dom_apis::{MayUseInvalidation, QueryAll, QuerySelectorAllResult, query_selector};

use crate::{PersistentStorage, WrapperCache};

const CLASS_DEFINITIONS: &str = include_str!("runtime.js");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WebFeatureFlags {
    pub worker_system: bool,
    pub streaming_networking: bool,
    pub persistent_storage: bool,
}

impl WebFeatureFlags {
    pub fn json(self) -> String {
        format!(
            "{{\"workerSystem\":{},\"streamingNetworking\":{},\"persistentStorage\":{}}}",
            self.worker_system, self.streaming_networking, self.persistent_storage,
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
    pub body: Option<String>,
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
    fn schedule(&mut self, delay_ms: f64, callback: ProtectedJsObject) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let delay_ms = if delay_ms.is_finite() {
            delay_ms.max(0.0)
        } else {
            0.0
        };
        self.timers.push(Timer {
            id,
            deadline: Instant::now() + Duration::from_secs_f64(delay_ms / 1000.0),
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
        Some(self.timers.remove(index).callback)
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
}

pub struct BindingQueues {
    pub timers: Rc<RefCell<TimerQueue>>,
    pub fetches: Rc<RefCell<FetchQueue>>,
    pub workers: Rc<RefCell<WorkerQueue>>,
    pub streaming: Rc<RefCell<StreamingQueue>>,
}

#[derive(Default)]
pub struct BrowsingContext {
    url: Mutex<Option<String>>,
    cookies: Mutex<cookie_store::CookieStore>,
    request_headers: Mutex<Vec<(http::HeaderName, http::HeaderValue)>>,
}

impl BrowsingContext {
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

    fn current_url(&self) -> Option<String> {
        self.url.lock().expect("browsing URL lock poisoned").clone()
    }

    pub fn store_response_cookie(&self, url: &str, header: &str) {
        let (Ok(url), Ok(cookie)) = (
            url::Url::parse(url),
            cookie_store::RawCookie::parse(header.to_owned()),
        ) else {
            return;
        };
        self.cookies
            .lock()
            .expect("cookie store lock poisoned")
            .store_response_cookies(std::iter::once(cookie), &url);
    }

    pub fn cookie_header(&self, url: &str) -> Option<String> {
        let url = url::Url::parse(url).ok()?;
        let value = self
            .cookies
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        (!value.is_empty()).then_some(value)
    }

    pub fn cookies_for_url(&self, url: &str) -> Vec<(String, String)> {
        let Ok(url) = url::Url::parse(url) else {
            return Vec::new();
        };
        self.cookies
            .lock()
            .expect("cookie store lock poisoned")
            .get_request_values(&url)
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }

    fn document_cookies(&self) -> String {
        let raw_url = self.url.lock().expect("browsing URL lock poisoned");
        let Some(url) = raw_url.as_deref().and_then(|url| url::Url::parse(url).ok()) else {
            return String::new();
        };
        self.cookies
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
            self.store_response_cookie(&url, header);
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
        let bindings = Self { state };
        bindings.reset_document(runtime)?;
        Ok(bindings)
    }

    pub fn reset_document(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        self.state.wrappers.clear();
        self.state.style_wrappers.clear();
        self.state.computed_style_wrappers.clear();
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
    match operation.as_str() {
        "runtimeFeatures" => Ok(NativeValue::String(state.features.json())),
        "workerCreate" => {
            if !state.features.worker_system {
                return Err(NativeError::new("worker system is disabled"));
            }
            let url = required_string(call, 2, "worker script URL")?;
            let kind = required_string(call, 3, "worker kind")?;
            let name = required_string(call, 4, "worker name")?;
            let scope = required_string(call, 5, "worker scope")?;
            let mut workers = state.workers.borrow_mut();
            let id = workers.next_id;
            workers.next_id = workers.next_id.wrapping_add(1).max(1);
            workers.pending.push_back(PendingWorkerOperation::Create {
                id,
                url,
                kind,
                name,
                scope,
            });
            Ok(NativeValue::Number(id as f64))
        }
        "workerPost" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            let message_json = required_string(call, 3, "worker message")?;
            state
                .workers
                .borrow_mut()
                .pending
                .push_back(PendingWorkerOperation::Post { id, message_json });
            Ok(NativeValue::Undefined)
        }
        "workerTerminate" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            let mut workers = state.workers.borrow_mut();
            workers
                .pending
                .push_back(PendingWorkerOperation::Terminate { id });
            Ok(NativeValue::Undefined)
        }
        "workerUnregister" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing worker id"))?
                .to_number()? as u64;
            state
                .workers
                .borrow_mut()
                .pending
                .push_back(PendingWorkerOperation::Unregister { id });
            Ok(NativeValue::Undefined)
        }
        "webSocketCreate" => {
            if !state.features.streaming_networking {
                return Err(NativeError::new("streaming networking is disabled"));
            }
            let url = required_string(call, 2, "WebSocket URL")?;
            let mut streaming = state.streaming.borrow_mut();
            let id = streaming.next_id;
            streaming.next_id = streaming.next_id.wrapping_add(1).max(1);
            streaming
                .pending
                .push_back(PendingWebSocketOperation::Create { id, url });
            Ok(NativeValue::Number(id as f64))
        }
        "webSocketSend" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing WebSocket id"))?
                .to_number()? as u64;
            let message = required_string(call, 3, "WebSocket message")?;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::SendText { id, message });
            Ok(NativeValue::Undefined)
        }
        "webSocketClose" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing WebSocket id"))?
                .to_number()? as u64;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::Close { id });
            Ok(NativeValue::Undefined)
        }
        "fetchStreamCancel" => {
            let id = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing Fetch stream id"))?
                .to_number()? as u64;
            state
                .streaming
                .borrow_mut()
                .pending
                .push_back(PendingWebSocketOperation::CancelFetch { id });
            Ok(NativeValue::Undefined)
        }
        "persistentList" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            let entries = storage.list(&origin, &namespace).map_err(err)?;
            Ok(NativeValue::String(
                serde_json::to_string(&entries).map_err(err)?,
            ))
        }
        "persistentGet" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            Ok(match storage.get(&origin, &namespace, &key).map_err(err)? {
                Some(value) => NativeValue::String(value),
                None => NativeValue::Null,
            })
        }
        "persistentSet" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let value = required_string(call, 4, "storage value")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage
                .set(&origin, &namespace, &key, &value)
                .map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentDelete" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let key = required_string(call, 3, "storage key")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage.delete(&origin, &namespace, &key).map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentClear" => {
            let namespace = required_string(call, 2, "storage namespace")?;
            let storage = persistent_storage(state)?;
            let origin = storage_origin(state)?;
            storage.clear(&origin, &namespace).map_err(err)?;
            Ok(NativeValue::Undefined)
        }
        "persistentEstimate" => {
            let storage = persistent_storage(state)?;
            let usage = match storage_origin(state) {
                Ok(origin) => storage.usage(&origin).map_err(err)?,
                Err(_) => 0,
            };
            Ok(NativeValue::String(format!(
                "{{\"usage\":{},\"quota\":{}}}",
                usage,
                storage.quota()
            )))
        }
        "setTimeout" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing timer callback"))?
                .to_function()?;
            let delay = call
                .argument(3)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0);
            let id = state.timers.borrow_mut().schedule(delay, callback);
            Ok(NativeValue::Number(f64::from(id)))
        }
        "clearTimeout" => {
            let id = call
                .argument(2)
                .map(|value| value.to_number())
                .transpose()?
                .unwrap_or(0.0) as u32;
            state.timers.borrow_mut().clear(id);
            Ok(NativeValue::Undefined)
        }
        "queueMicrotask" => {
            let callback = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing microtask callback"))?
                .to_function()?;
            state.timers.borrow_mut().queue_microtask(callback);
            Ok(NativeValue::Undefined)
        }
        "location" => {
            let property = required_string(call, 2, "location property")?;
            let raw_url = state
                .browsing_context
                .url
                .lock()
                .expect("browsing URL lock poisoned");
            let Some(raw_url) = raw_url.as_deref() else {
                return Ok(NativeValue::String(String::new()));
            };
            let url = url::Url::parse(raw_url).map_err(err)?;
            let value = match property.as_str() {
                "href" => url.as_str().to_string(),
                "protocol" => format!("{}:", url.scheme()),
                "host" => match (url.host_str(), url.port()) {
                    (Some(host), Some(port)) => format!("{host}:{port}"),
                    (Some(host), None) => host.to_string(),
                    (None, _) => String::new(),
                },
                "hostname" => url.host_str().unwrap_or_default().to_string(),
                "port" => url.port().map(|port| port.to_string()).unwrap_or_default(),
                "pathname" => url.path().to_string(),
                "search" => url
                    .query()
                    .map(|query| format!("?{query}"))
                    .unwrap_or_default(),
                "hash" => url
                    .fragment()
                    .map(|hash| format!("#{hash}"))
                    .unwrap_or_default(),
                "origin" => url.origin().ascii_serialization(),
                _ => return Err(NativeError::new("unknown Location property")),
            };
            Ok(NativeValue::String(value))
        }
        "urlParse" => {
            let input = required_string(call, 2, "URL input")?;
            let base = required_string(call, 3, "URL base")?;
            let base = (!base.is_empty())
                .then(|| url::Url::parse(&base).map_err(err))
                .transpose()?;
            let parsed = url::Url::options()
                .base_url(base.as_ref())
                .parse(&input)
                .map_err(err)?;
            Ok(NativeValue::String(url_record_json(&parsed)?))
        }
        "urlSet" => {
            let href = required_string(call, 2, "URL href")?;
            let component = required_string(call, 3, "URL component")?;
            let value = required_string(call, 4, "URL component value")?;
            Ok(NativeValue::String(set_url_component(
                &href, &component, &value,
            )?))
        }
        "urlSearchParamsParse" => {
            let input = required_string(call, 2, "query")?;
            let pairs = url::form_urlencoded::parse(input.trim_start_matches('?').as_bytes())
                .into_owned()
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&pairs).map_err(err)?,
            ))
        }
        "urlSearchParamsSerialize" => {
            let input = required_string(call, 2, "query pairs")?;
            let pairs: Vec<(String, String)> = serde_json::from_str(&input).map_err(err)?;
            let output = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish();
            Ok(NativeValue::String(output))
        }
        "encodingCanonical" => {
            let label = required_string(call, 2, "encoding label")?;
            match encoding_rs::Encoding::for_label_no_replacement(label.as_bytes()) {
                Some(encoding) => Ok(NativeValue::String(encoding.name().to_ascii_lowercase())),
                None => Ok(NativeValue::Null),
            }
        }
        "legacyQueryEncodeBlock" => {
            let label = required_string(call, 2, "encoding label")?;
            let block_start = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing code point block"))?
                .to_number()? as u32;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid document encoding"))?;
            let encoded = (block_start..block_start.saturating_add(256))
                .map(|code_point| {
                    char::from_u32(code_point)
                        .map(|character| legacy_query_encode(encoding, &character.to_string()))
                        .unwrap_or_else(|| "%EF%BF%BD".to_owned())
                })
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&encoded).map_err(err)?,
            ))
        }
        "legacyQueryEncode" => {
            let label = required_string(call, 2, "encoding label")?;
            let input = required_string(call, 3, "query input")?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid document encoding"))?;
            Ok(NativeValue::String(legacy_query_encode(encoding, &input)))
        }
        "formUrlEncode" => {
            let label = required_string(call, 2, "form encoding label")?;
            let input = required_string(call, 3, "form field value")?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .unwrap_or(encoding_rs::UTF_8);
            Ok(NativeValue::String(form_url_encode(encoding, &input)))
        }
        "decodeBytes" => {
            let label = required_string(call, 2, "encoding label")?;
            let bytes_json = required_string(call, 3, "encoded bytes")?;
            let fatal = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing fatal flag"))?
                .to_boolean();
            let ignore_bom = call
                .argument(5)
                .ok_or_else(|| NativeError::new("missing ignoreBOM flag"))?
                .to_boolean();
            let stream = call
                .argument(6)
                .ok_or_else(|| NativeError::new("missing stream flag"))?
                .to_boolean();
            let bytes: Vec<u8> = serde_json::from_str(&bytes_json).map_err(err)?;
            let encoding = encoding_rs::Encoding::for_label_no_replacement(label.as_bytes())
                .ok_or_else(|| NativeError::new("invalid encoding label"))?;
            match decode_bytes(encoding, &bytes, fatal, ignore_bom, !stream)? {
                Some(decoded) => Ok(NativeValue::String(decoded)),
                None => Ok(NativeValue::Null),
            }
        }
        "encodeUtf8" => {
            let input = required_string(call, 2, "text")?;
            Ok(NativeValue::String(
                serde_json::to_string(input.as_bytes()).map_err(err)?,
            ))
        }
        "base64Encode" => {
            use base64::Engine as _;
            let bytes_json = required_string(call, 2, "bytes")?;
            let bytes: Vec<u8> = serde_json::from_str(&bytes_json).map_err(err)?;
            Ok(NativeValue::String(
                base64::engine::general_purpose::STANDARD.encode(bytes),
            ))
        }
        "base64Decode" => {
            use base64::{Engine as _, alphabet, engine};
            let input = required_string(call, 2, "base64 input")?;
            let input = input
                .bytes()
                .filter(|byte| !matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' '))
                .collect::<Vec<_>>();
            let config = engine::general_purpose::GeneralPurposeConfig::new()
                .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent)
                .with_decode_allow_trailing_bits(true);
            let decoder = engine::GeneralPurpose::new(&alphabet::STANDARD, config);
            match decoder.decode(input) {
                Ok(bytes) => Ok(NativeValue::String(
                    serde_json::to_string(&bytes).map_err(err)?,
                )),
                Err(_) => Ok(NativeValue::Null),
            }
        }
        "fetch" | "fetchStream" => {
            let url = required_string(call, 2, "fetch URL")?;
            let method = required_string(call, 3, "fetch method")?;
            let headers_json = required_string(call, 4, "fetch headers")?;
            let body = call
                .argument(5)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| value.to_string())
                .transpose()?;
            let (promise, settlement) = call.make_deferred_promise()?.into_parts();
            let mut fetches = state.fetches.borrow_mut();
            let id = fetches.next_id();
            fetches.push(
                PendingFetch {
                    id,
                    url,
                    method,
                    headers_json,
                    body,
                    streaming: operation == "fetchStream",
                },
                settlement,
            );
            Ok(NativeValue::ProtectedObject(promise))
        }
        "innerWidth" | "innerHeight" | "devicePixelRatio" => {
            let metrics = state.document.borrow().viewport_metrics();
            let index = match operation.as_str() {
                "innerWidth" => 0,
                "innerHeight" => 1,
                _ => 2,
            };
            Ok(NativeValue::Number(metrics[index]))
        }
        "domParserParse" => {
            let input = required_string(call, 2, "input")?;
            let content_type = required_string(call, 3, "type")?;
            let root = state.document.borrow_mut().create_document();
            if content_type == "text/html" {
                let mut parser =
                    HtmlParserSession::new_at_root(Rc::clone(&state.document), &input, root);
                while !matches!(parser.resume(), ParseProgress::Done) {}
            } else if parse_xml_at_root(Rc::clone(&state.document), &input, root) {
                let mut document = state.document.borrow_mut();
                document
                    .blitz_mut()
                    .mutate()
                    .remove_and_drop_all_children(root);
                let name = QualName::new(
                    None,
                    Namespace::from("http://www.mozilla.org/newlayout/xml/parsererror.xml"),
                    LocalName::from("parsererror"),
                );
                let mut mutator = document.blitz_mut().mutate();
                let error = mutator.create_element(name, vec![]);
                let text = mutator.create_text_node("XML parsing error");
                mutator.append_children(error, &[text]);
                mutator.append_children(root, &[error]);
                drop(mutator);
                document.adopt_subtree(error, root);
            }
            node_value(state, call, root)
        }
        "documentElement" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = document.node(root).and_then(|node| {
                node.children
                    .iter()
                    .copied()
                    .find(|id| document.node(*id).is_some_and(|node| node.is_element()))
            });
            drop(document);
            optional_node(state, call, id)
        }
        "title" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let title = subtree_query_selector_all(&document, root, "title")?
                .into_iter()
                .next()
                .and_then(|id| document.node(id))
                .map(|node| node.text_content())
                .unwrap_or_default();
            Ok(NativeValue::String(title))
        }
        "cookie" => {
            required_document_target(state, call)?;
            Ok(NativeValue::String(
                state.browsing_context.document_cookies(),
            ))
        }
        "setCookie" => {
            required_document_target(state, call)?;
            let cookie = required_string(call, 2, "cookie")?;
            state.browsing_context.set_document_cookie(&cookie);
            Ok(NativeValue::Undefined)
        }
        "head" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = subtree_query_selector_all(&document, root, "head")?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, id)
        }
        "body" => {
            let root = required_document_target(state, call)?;
            let document = state.document.borrow();
            let id = subtree_query_selector_all(&document, root, "body")?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, id)
        }
        "createElement" => {
            let owner = required_document_target(state, call)?;
            let tag = required_string(call, 2, "tag name")?.to_ascii_lowercase();
            if tag.is_empty() {
                return Err(NativeError::new("tag name cannot be empty"));
            }
            let name = QualName::new(None, ns!(html), LocalName::from(tag));
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_element(name, vec![]);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createElementNS" => {
            let owner = required_document_target(state, call)?;
            let namespace = call
                .argument(2)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| value.to_string())
                .transpose()?
                .unwrap_or_default();
            let qualified_name = required_string(call, 3, "qualified name")?;
            let mut parts = qualified_name.split(':');
            let first = parts.next().unwrap_or_default();
            let second = parts.next();
            if qualified_name.is_empty()
                || parts.next().is_some()
                || first.is_empty()
                || second.is_some_and(str::is_empty)
            {
                return Err(NativeError::new("invalid qualified name"));
            }
            let (prefix, local_name) = match second {
                Some(local_name) => (Some(Prefix::from(first)), local_name),
                None => (None, first),
            };
            if prefix.is_some() && namespace.is_empty() {
                return Err(NativeError::new("a prefixed name requires a namespace"));
            }
            let name = QualName::new(
                prefix,
                Namespace::from(namespace),
                LocalName::from(local_name),
            );
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_element(name, vec![]);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createTextNode" => {
            let owner = required_document_target(state, call)?;
            let text = required_string(call, 2, "text")?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.blitz_mut().mutate().create_text_node(&text);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createComment" => {
            let owner = required_document_target(state, call)?;
            let data = required_string(call, 2, "comment data")?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.create_comment(&data);
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "createDocumentFragment" => {
            let owner = required_document_target(state, call)?;
            let id = {
                let mut document = state.document.borrow_mut();
                let id = document.create_document_fragment();
                document.set_node_document(id, owner);
                id
            };
            node_value(state, call, id)
        }
        "getElementById" => {
            let root = required_document_target(state, call)?;
            let id = required_string(call, 2, "id")?;
            let document = state.document.borrow();
            let node = descendant_ids(&document, root)?
                .into_iter()
                .find(|node_id| {
                    document
                        .node(*node_id)
                        .and_then(|node| node.element_data())
                        .and_then(|element| element.attr(LocalName::from("id")))
                        == Some(id.as_str())
                });
            drop(document);
            optional_node(state, call, node)
        }
        "elementFromPoint" => {
            required_document_target(state, call)?;
            let x = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing x coordinate"))?
                .to_number()?;
            let y = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing y coordinate"))?
                .to_number()?;
            let viewport = state.document.borrow().viewport_metrics();
            if !x.is_finite()
                || !y.is_finite()
                || x < 0.0
                || y < 0.0
                || x >= viewport[0]
                || y >= viewport[1]
            {
                return Ok(NativeValue::Null);
            }
            resolve_document(state);
            let node = state.document.borrow().element_at_point(x, y);
            optional_node(state, call, node)
        }
        "getElementsByTagName" => {
            let root_id = required_parent_node_target(state, call)?;
            let name = required_string(call, 2, "name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let nodes = descendant_ids(&document, root_id)?
                .into_iter()
                .filter(|id| {
                    document.node(*id).is_some_and(|node| match &node.data {
                        NodeData::Element(element) => {
                            name == "*" || element.name.local.as_ref() == name
                        }
                        _ => false,
                    })
                })
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "getElementsByClassName" => {
            let root_id = required_parent_node_target(state, call)?;
            let names = required_string(call, 2, "class names")?
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let document = state.document.borrow();
            let nodes = if names.is_empty() {
                Vec::new()
            } else {
                descendant_ids(&document, root_id)?
                    .into_iter()
                    .filter(|id| {
                        document
                            .node(*id)
                            .and_then(|node| node.element_data())
                            .and_then(|element| element.attr(LocalName::from("class")))
                            .is_some_and(|classes| {
                                let classes =
                                    classes.split_ascii_whitespace().collect::<HashSet<_>>();
                                names.iter().all(|name| classes.contains(name.as_str()))
                            })
                    })
                    .collect()
            };
            drop(document);
            node_array(state, call, &nodes)
        }
        "getElementsByName" => {
            let root_id = required_document_target(state, call)?;
            let name = required_string(call, 2, "name")?;
            let document = state.document.borrow();
            let nodes = descendant_ids(&document, root_id)?
                .into_iter()
                .filter(|id| {
                    document
                        .node(*id)
                        .and_then(|node| node.element_data())
                        .is_some_and(|element| {
                            element.name.ns == ns!(html)
                                && element.attr(LocalName::from("name")) == Some(name.as_str())
                        })
                })
                .collect::<Vec<_>>();
            drop(document);
            node_array(state, call, &nodes)
        }
        "querySelector" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let node = subtree_query_selector_all(&document, root_id, &selector)?
                .into_iter()
                .next();
            drop(document);
            optional_node(state, call, node)
        }
        "querySelectorAll" => {
            let root_id = required_parent_node_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            let document = state.document.borrow();
            let nodes = subtree_query_selector_all(&document, root_id, &selector)?;
            drop(document);
            node_array(state, call, &nodes)
        }
        "matches" => {
            let id = required_element_target(state, call)?;
            let selector = required_string(call, 2, "selector")?;
            Ok(NativeValue::Boolean(
                state
                    .document
                    .borrow()
                    .query_selector_all(&selector)
                    .map_err(err)?
                    .contains(&id),
            ))
        }
        "ownerDocument" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let owner = if document.is_document(id) {
                None
            } else {
                document.node_document(id)
            };
            drop(document);
            optional_node(state, call, owner)
        }
        "nodeType" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let node_type = if document.is_document_fragment(id) {
                11.0
            } else {
                match node.data {
                    NodeData::Element(_) | NodeData::AnonymousBlock(_) => 1.0,
                    NodeData::Text(_) => 3.0,
                    NodeData::Comment => 8.0,
                    NodeData::Document => 9.0,
                }
            };
            Ok(NativeValue::Number(node_type))
        }
        "nodeName" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let name = if document.is_document_fragment(id) {
                "#document-fragment".to_owned()
            } else {
                match &node.data {
                    NodeData::Element(element) | NodeData::AnonymousBlock(element) => {
                        element.name.local.to_string().to_ascii_uppercase()
                    }
                    NodeData::Text(_) => "#text".to_owned(),
                    NodeData::Comment => "#comment".to_owned(),
                    NodeData::Document => "#document".to_owned(),
                }
            };
            Ok(NativeValue::String(name))
        }
        "parentNode" => {
            let id = required_node_target(state, call)?;
            let parent = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .parent;
            optional_node(state, call, parent)
        }
        "firstChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .first()
                .copied();
            optional_node(state, call, child)
        }
        "lastChild" => {
            let id = required_node_target(state, call)?;
            let child = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .last()
                .copied();
            optional_node(state, call, child)
        }
        "previousSibling" | "nextSibling" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let parent = node.parent.and_then(|parent| document.node(parent));
            let sibling = parent.and_then(|parent| {
                let index = parent.children.iter().position(|child| *child == id)?;
                if operation == "previousSibling" {
                    index.checked_sub(1).map(|index| parent.children[index])
                } else {
                    parent.children.get(index + 1).copied()
                }
            });
            drop(document);
            optional_node(state, call, sibling)
        }
        "childNodes" => {
            let id = required_node_target(state, call)?;
            let children = state
                .document
                .borrow()
                .node(id)
                .ok_or_else(stale_wrapper)?
                .children
                .clone();
            node_array(state, call, &children)
        }
        "textContent" => {
            let id = required_node_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let text = document
                .comment_data(id)
                .map(str::to_owned)
                .unwrap_or_else(|| node.text_content());
            Ok(NativeValue::String(text))
        }
        "setTextContent" => {
            let id = required_node_target(state, call)?;
            let value = required_string(call, 2, "textContent")?;
            set_text_content(state, id, &value)?;
            Ok(NativeValue::Undefined)
        }
        "appendChild" => mutate_child(state, call, ChildMutation::Append),
        "removeChild" => mutate_child(state, call, ChildMutation::Remove),
        "insertBefore" => mutate_child(state, call, ChildMutation::InsertBefore),
        "cloneNode" => {
            let id = required_node_target(state, call)?;
            let deep = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing deep flag"))?
                .to_boolean();
            let clone = {
                let mut document = state.document.borrow_mut();
                let mut mutator = document.blitz_mut().mutate();
                let clone = mutator.deep_clone_node(id);
                if !deep {
                    mutator.remove_and_drop_all_children(clone);
                }
                drop(mutator);
                document.copy_node_metadata(id, clone, deep);
                clone
            };
            node_value(state, call, clone)
        }
        "tagName" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let local_name = element.name.local.to_string();
            let tag_name = if element.name.ns == ns!(html) {
                local_name.to_ascii_uppercase()
            } else if let Some(prefix) = &element.name.prefix {
                format!("{prefix}:{local_name}")
            } else {
                local_name
            };
            Ok(NativeValue::String(tag_name))
        }
        "localName" | "namespaceURI" | "prefix" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            match operation.as_str() {
                "localName" => Ok(NativeValue::String(element.name.local.to_string())),
                "namespaceURI" => {
                    if element.name.ns.is_empty() {
                        Ok(NativeValue::Null)
                    } else {
                        Ok(NativeValue::String(element.name.ns.to_string()))
                    }
                }
                "prefix" => match &element.name.prefix {
                    Some(prefix) => Ok(NativeValue::String(prefix.to_string())),
                    None => Ok(NativeValue::Null),
                },
                _ => unreachable!(),
            }
        }
        "getAttribute" | "getAttributeOrEmpty" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let document = state.document.borrow();
            let value = document
                .node(id)
                .and_then(|node| node.element_data())
                .and_then(|element| element.attr(LocalName::from(name)));
            match (operation.as_str(), value) {
                ("getAttributeOrEmpty", None) => Ok(NativeValue::String(String::new())),
                (_, None) => Ok(NativeValue::Null),
                (_, Some(value)) => Ok(NativeValue::String(value.to_owned())),
            }
        }
        "elementAttributes" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let attributes = element
                .attrs()
                .iter()
                .map(|attribute| {
                    let prefix = attribute.name.prefix.as_ref().map(ToString::to_string);
                    let local_name = attribute.name.local.to_string();
                    let name = prefix
                        .as_ref()
                        .map(|prefix| format!("{prefix}:{local_name}"))
                        .unwrap_or_else(|| local_name.clone());
                    serde_json::json!({
                        "namespaceURI": if attribute.name.ns.is_empty() {
                            None
                        } else {
                            Some(attribute.name.ns.to_string())
                        },
                        "prefix": prefix,
                        "localName": local_name,
                        "name": name,
                        "value": attribute.value,
                    })
                })
                .collect::<Vec<_>>();
            Ok(NativeValue::String(
                serde_json::to_string(&attributes).map_err(err)?,
            ))
        }
        "setAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let value = required_string(call, 3, "attribute value")?;
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_attribute(id, name, &value);
            Ok(NativeValue::Undefined)
        }
        "removeAttribute" => {
            let id = required_element_target(state, call)?;
            let name = required_string(call, 2, "attribute name")?.to_ascii_lowercase();
            let name = QualName::new(None, ns!(), LocalName::from(name));
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .clear_attribute(id, name);
            Ok(NativeValue::Undefined)
        }
        "elementUrl" => {
            let id = required_element_target(state, call)?;
            let property = required_string(call, 2, "URL property")?;
            let document = state.document.borrow();
            let element = document
                .node(id)
                .and_then(|node| node.element_data())
                .ok_or_else(stale_wrapper)?;
            let attribute = if property == "origin" {
                "href"
            } else {
                property.as_str()
            };
            let input = element
                .attr(LocalName::from(attribute))
                .unwrap_or_default()
                .to_owned();
            let document_url = state
                .browsing_context
                .current_url()
                .and_then(|url| url::Url::parse(&url).ok());
            let base_url = if element.name.local.as_ref() == "base" {
                document_url.clone()
            } else {
                document
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
                    .or(document_url)
            };
            let parsed = url::Url::options()
                .base_url(base_url.as_ref())
                .parse(&input);
            let value = match (property.as_str(), parsed) {
                ("origin", Ok(parsed)) => parsed.origin().ascii_serialization(),
                (_, Ok(parsed)) => parsed.as_str().to_owned(),
                (_, Err(_)) => input,
            };
            Ok(NativeValue::String(value))
        }
        "innerHTML" => {
            let id = required_element_target(state, call)?;
            let document = state.document.borrow();
            let node = document.node(id).ok_or_else(stale_wrapper)?;
            let mut html = String::new();
            for child in &node.children {
                document
                    .node(*child)
                    .ok_or_else(stale_wrapper)?
                    .write_outer_html(&mut html);
            }
            Ok(NativeValue::String(html))
        }
        "setInnerHTML" => {
            let id = required_element_target(state, call)?;
            let html = required_string(call, 2, "innerHTML")?;
            let removed = descendant_ids(&state.document.borrow(), id)?;
            state
                .document
                .borrow_mut()
                .blitz_mut()
                .mutate()
                .set_inner_html(id, &html);
            state.wrappers.remove_nodes(&removed);
            state.style_wrappers.remove_nodes(&removed);
            Ok(NativeValue::Undefined)
        }
        "style" => {
            let id = required_element_target(state, call)?;
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "getComputedStyle" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let prototype = prototypes(state).css_style.identity();
            let style = state
                .computed_style_wrappers
                .wrap_with_prototype(call, id, prototype);
            Ok(NativeValue::Object(style))
        }
        "styleSheetElements" => {
            required_document_target(state, call)?;
            let nodes = state.document.borrow().stylesheet_node_ids();
            node_array(state, call, &nodes)
        }
        "styleSheetRules" => {
            let id = required_element_target(state, call)?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .stylesheet_rule_texts(id)
                    .ok_or(CssomError::NotAStyleSheet),
            )
        }
        "parseStyleSheetRule" => {
            let rule = required_string(call, 2, "CSS rule")?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .parse_stylesheet_rule(&rule)
                    .map(|rule| vec![rule]),
            )
        }
        "parseStyleSheetText" => {
            let css = required_string(call, 2, "stylesheet text")?;
            cssom_json(Ok(state.document.borrow().parse_stylesheet_text(&css)))
        }
        "styleRuleDeclarations" => {
            let rule = required_string(call, 2, "CSS style rule")?;
            let declarations = state
                .document
                .borrow()
                .style_rule_declarations(&rule)
                .unwrap_or_default();
            Ok(NativeValue::String(
                serde_json::to_string(&declarations).map_err(err)?,
            ))
        }
        "styleRuleGetProperty" => {
            let rule = required_string(call, 2, "CSS style rule")?;
            let name = required_string(call, 3, "CSS property name")?;
            Ok(NativeValue::String(
                state
                    .document
                    .borrow()
                    .style_rule_property(&rule, &name)
                    .unwrap_or_default(),
            ))
        }
        "nestedRuleTexts" => {
            let rule = required_string(call, 2, "CSS grouping rule")?;
            cssom_json(
                state
                    .document
                    .borrow()
                    .nested_rule_texts(&rule)
                    .ok_or(CssomError::Syntax),
            )
        }
        "styleSheetInsertRule" => {
            let id = required_element_target(state, call)?;
            let rule = required_string(call, 2, "CSS rule")?;
            let index = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .insert_stylesheet_rule(id, &rule, index),
            )
        }
        "styleSheetDeleteRule" => {
            let id = required_element_target(state, call)?;
            let index = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .delete_stylesheet_rule(id, index),
            )
        }
        "styleSheetReplaceRule" => {
            let id = required_element_target(state, call)?;
            let rule = required_string(call, 2, "CSS rule")?;
            let index = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing CSS rule index"))?
                .to_number()? as usize;
            cssom_json(
                state
                    .document
                    .borrow_mut()
                    .replace_stylesheet_rule(id, &rule, index),
            )
        }
        "styleSheetReplace" => {
            let id = required_element_target(state, call)?;
            let css = required_string(call, 2, "stylesheet text")?;
            cssom_json(state.document.borrow_mut().replace_stylesheet(id, &css))
        }
        "styleGetProperty" => {
            let name = required_string(call, 2, "property name")?;
            let object = required_object(call, 1, "style receiver")?;
            if let Some(id) = state.style_wrappers.node_id(object) {
                Ok(NativeValue::String(inline_style_property(state, id, &name)))
            } else if let Some(id) = state.computed_style_wrappers.node_id(object) {
                resolve_document(state);
                Ok(NativeValue::String(
                    state
                        .document
                        .borrow()
                        .computed_style_property(id, &name)
                        .unwrap_or_default(),
                ))
            } else {
                Err(NativeError::new("receiver is not a CSSStyleDeclaration"))
            }
        }
        "styleDeclarations" => {
            let object = required_object(call, 1, "style receiver")?;
            let declarations = if let Some(id) = state.style_wrappers.node_id(object) {
                state
                    .document
                    .borrow()
                    .inline_style_declarations(id)
                    .unwrap_or_default()
            } else if let Some(id) = state.computed_style_wrappers.node_id(object) {
                resolve_document(state);
                state.document.borrow().computed_style_declarations(id)
            } else {
                return Err(NativeError::new("receiver is not a CSSStyleDeclaration"));
            };
            Ok(NativeValue::String(
                serde_json::to_string(&declarations).map_err(err)?,
            ))
        }
        "styleCssText" => {
            let object = required_object(call, 1, "style receiver")?;
            if let Some(id) = state.style_wrappers.node_id(object) {
                Ok(NativeValue::String(
                    state
                        .document
                        .borrow()
                        .inline_style_css(id)
                        .unwrap_or_default(),
                ))
            } else if state.computed_style_wrappers.node_id(object).is_some() {
                Ok(NativeValue::String(String::new()))
            } else {
                Err(NativeError::new("receiver is not a CSSStyleDeclaration"))
            }
        }
        "styleWritable" => {
            let object = required_object(call, 1, "style receiver")?;
            Ok(NativeValue::Boolean(
                state.style_wrappers.node_id(object).is_some(),
            ))
        }
        "styleSetCssText" => {
            let id = required_style_target(state, call)?;
            let css = required_string(call, 2, "declaration text")?;
            state.document.borrow_mut().set_inline_style_css(id, &css);
            Ok(NativeValue::Undefined)
        }
        "styleSetProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let value = required_string(call, 3, "property value")?;
            state
                .document
                .borrow_mut()
                .set_style_property(id, &name, &value);
            Ok(NativeValue::Undefined)
        }
        "styleRemoveProperty" => {
            let id = required_style_target(state, call)?;
            let name = required_string(call, 2, "property name")?;
            let old = inline_style_property(state, id, &name);
            state.document.borrow_mut().remove_style_property(id, &name);
            Ok(NativeValue::String(old))
        }
        "clientWidth" | "clientHeight" | "offsetWidth" | "offsetHeight" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let document = state.document.borrow();
            let size = if operation.starts_with("client") {
                document.client_size(id)
            } else {
                document.offset_size(id)
            }
            .ok_or_else(stale_wrapper)?;
            let index = usize::from(operation.ends_with("Height"));
            Ok(NativeValue::Number(size[index]))
        }
        "boundingRect" => {
            let id = required_element_target(state, call)?;
            resolve_document(state);
            let rect = state
                .document
                .borrow()
                .bounding_rect(id)
                .ok_or_else(stale_wrapper)?;
            let values = rect.into_iter().map(NativeValue::Number).collect();
            Ok(NativeValue::ProtectedObject(call.make_value_array(values)?))
        }
        _ => Err(NativeError::new(format!(
            "unknown native DOM operation: {operation}"
        ))),
    }
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
    let mut results: QuerySelectorAllResult<&blitz_dom::Node> = Default::default();
    query_selector::<&blitz_dom::Node, QueryAll>(
        root,
        &selectors,
        &mut results,
        MayUseInvalidation::Yes,
    );
    Ok(results
        .into_iter()
        .map(|node| node.id)
        .filter(|id| *id != root_id)
        .collect())
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
