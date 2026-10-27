use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::dom::{BrowserDocument, HtmlParserSession, NodeId, ParseProgress};
use crate::web_apis::WrapperCache;
use brimp_jsc::{
    JsException, JsObjectIdentity, JsRuntime, NativeCall, NativeError, NativeValue,
    PromiseSettlement, ProtectedJsObject,
};

mod crypto;
mod dispatch_dom;
mod dispatch_platform;

const CLASS_DEFINITIONS: &str = concat!(
    "(() => {\n",
    include_str!("../../js/web_apis/bootstrap.js"),
    include_str!("../../js/vendor/input-color.bundle.js"),
    include_str!("../../js/web_apis/events_messaging.js"),
    include_str!("../../js/web_apis/dom_core.js"),
    include_str!("../../js/web_apis/document_mode.js"),
    include_str!("../../js/web_apis/document_collections.js"),
    include_str!("../../js/web_apis/token_list.js"),
    include_str!("../../js/web_apis/traversal.js"),
    include_str!("../../js/web_apis/elements.js"),
    include_str!("../../js/web_apis/input_decimal.js"),
    include_str!("../../js/web_apis/input_values.js"),
    include_str!("../../js/web_apis/input_checked.js"),
    include_str!("../../js/web_apis/input_attributes.js"),
    include_str!("../../js/web_apis/html_elements.js"),
    include_str!("../../js/web_apis/namespaces.js"),
    include_str!("../../js/web_apis/window_location.js"),
    include_str!("../../js/web_apis/shadow_dom.js"),
    include_str!("../../js/web_apis/navigator_services.js"),
    include_str!("../../js/web_apis/crypto.js"),
    include_str!("../../js/web_apis/url_encoding_files.js"),
    include_str!("../../js/web_apis/form_disabled.js"),
    include_str!("../../js/web_apis/file_reader.js"),
    include_str!("../../js/web_apis/storage_fetch.js"),
    include_str!("../../js/web_apis/forms.js"),
    include_str!("../../js/web_apis/input_value_conversions.js"),
    include_str!("../../js/web_apis/input_stepping.js"),
    include_str!("../../js/web_apis/input_editing.js"),
    include_str!("../../js/web_apis/input_validity.js"),
    include_str!("../../js/web_apis/input_validation.js"),
    include_str!("../../js/web_apis/label_associations.js"),
    include_str!("../../js/web_apis/control_selection.js"),
    include_str!("../../js/web_apis/element_internals.js"),
    include_str!("../../js/web_apis/css_property_names.js"),
    include_str!("../../js/web_apis/css_style.js"),
    include_str!("../../js/web_apis/stylesheets.js"),
    include_str!("../../js/web_apis/observers.js"),
    include_str!("../../js/web_apis/document_write.js"),
    include_str!("../../js/web_apis/drag_drop.js"),
    include_str!("../../js/web_apis/input.js"),
    include_str!("../../js/web_apis/dom_placeholders.js"),
    include_str!("../../js/web_apis/intl_profile.js"),
    include_str!("../../js/web_apis/media_query.js"),
    include_str!("../../js/web_apis/fonts.js"),
    include_str!("../../js/web_apis/window_environment.js"),
    include_str!("../../js/web_apis/storage_host.js"),
    include_str!("../../js/web_apis/scheduling.js"),
    include_str!("../../js/web_apis/install.js"),
    "\n})();\n",
    include_str!("../../js/web_apis/geometry.js"),
    include_str!("../../js/web_apis/css-typed-om.js"),
    include_str!("../../js/web_apis/value_objects.js"),
    include_str!("../../js/web_apis/canvas_contexts.js"),
);

pub struct TimerQueue {
    pub(crate) clock: crate::runtime::clock::Clock,
    next_id: u32,
    timers: Vec<Timer>,
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

pub enum PendingWebSocketOperation {
    Create { id: u64, url: String },
    SendText { id: u64, message: String },
    SendBinary { id: u64, message: Vec<u8> },
    Close { id: u64 },
    CancelFetch { id: u64 },
    ConsumeFetch { id: u64, bytes: usize },
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
    deadline: Duration,
    interval: Option<Duration>,
    priority: u8,
    callback: ProtectedJsObject,
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self::new(&Default::default())
    }
}
impl TimerQueue {
    pub(crate) fn new(options: &crate::runtime::clock::ClockOptions) -> Self {
        Self {
            clock: crate::runtime::clock::Clock::new(options),
            next_id: 1,
            timers: Vec::new(),
        }
    }
}

impl TimerQueue {
    pub(crate) fn next_deadline(&self) -> Option<Duration> {
        self.timers.iter().map(|t| t.deadline).min()
    }
    pub(crate) fn wait_duration(&self) -> Option<Duration> {
        if self.clock.logical() {
            None
        } else {
            self.next_deadline()
                .map(|time| time.saturating_sub(self.clock.now()))
        }
    }
    fn schedule(&mut self, delay_ms: f64, repeat: bool, callback: ProtectedJsObject) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let delay_ms = if delay_ms.is_finite() {
            delay_ms.clamp(0.0, 2_147_483_647.0)
        } else {
            0.0
        };
        let delay = Duration::from_secs_f64(delay_ms / 1000.0);
        self.timers.push(Timer {
            id,
            deadline: self.clock.now() + delay,
            interval: repeat.then_some(delay.max(Duration::from_millis(1))),
            priority: 1,
            callback,
        });
        id
    }

    fn clear(&mut self, id: u32) {
        self.timers.retain(|timer| timer.id != id);
    }

    pub fn pop_due(&mut self) -> Option<ProtectedJsObject> {
        let now = self.clock.now();
        let index = self
            .timers
            .iter()
            .enumerate()
            .filter(|(_, timer)| timer.deadline <= now)
            .min_by_key(|(_, timer)| (timer.priority, timer.deadline, timer.id))
            .map(|(index, _)| index)?;
        if let Some(interval) = self.timers[index].interval {
            self.timers[index].deadline = now + interval;
            Some(self.timers[index].callback.clone())
        } else {
            Some(self.timers.remove(index).callback)
        }
    }
}

pub struct BindingRuntime {
    state: Rc<BindingState>,
    input_controller: ProtectedJsObject,
    parser_mutations: ProtectedJsObject,
    script_controller: ProtectedJsObject,
    exposure_finalizer: ProtectedJsObject,
    window_names: ProtectedJsObject,
}

pub struct BindingQueues {
    pub timers: Rc<RefCell<TimerQueue>>,
    pub fetches: Rc<RefCell<FetchQueue>>,
    pub streaming: Rc<RefCell<StreamingQueue>>,
}

#[derive(Clone, Debug, serde::Serialize)]
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
    pub(crate) storage: Arc<super::storage::Storage>,
    pub(crate) session_storage: Mutex<super::storage::Partitions>,
}

impl Default for BrowsingContext {
    fn default() -> Self {
        Self::with_cookie_jar(Arc::new(CookieJar::default()))
    }
}

impl BrowsingContext {
    pub fn with_cookie_jar(cookies: Arc<CookieJar>) -> Self {
        Self {
            url: Mutex::new(None),
            pending_navigation: Mutex::new(None),
            cookies,
            request_headers: Mutex::new(Vec::new()),
            storage: Arc::default(),
            session_storage: Mutex::default(),
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

    pub(crate) fn update_identity_headers(
        &self,
        headers: Vec<(String, String)>,
    ) -> Result<(), String> {
        let parsed = headers
            .into_iter()
            .map(|(name, value)| {
                Ok((
                    name.parse::<http::HeaderName>()
                        .map_err(|error| error.to_string())?,
                    value
                        .parse::<http::HeaderValue>()
                        .map_err(|error| error.to_string())?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut current = self.request_headers.lock().unwrap();
        for (name, value) in parsed {
            current.retain(|(old, _)| old != name);
            current.push((name, value));
        }
        Ok(())
    }
    pub fn apply_request_identity(&self, headers: &mut crate::network::HeaderList) {
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
    crypto: RefCell<crypto::CryptoState>,
    storage_realm: super::storage::Realm,
    diagnostics: Option<crate::runtime::diagnostics::Diagnostics>,
    events: crate::runtime::events::EventHub,
    document: Rc<RefCell<BrowserDocument>>,
    wrappers: WrapperCache,
    style_wrappers: WrapperCache,
    document_writers: RefCell<HashMap<usize, brimp_lexbor::HtmlWriter>>,
    prototypes: RefCell<Option<Prototypes>>,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    fetches: Rc<RefCell<FetchQueue>>,
    streaming: Rc<RefCell<StreamingQueue>>,
    websocket_delivery: RefCell<Option<ProtectedJsObject>>,
    fetch_stream_delivery: RefCell<Option<ProtectedJsObject>>,
}

struct Prototypes {
    node: ProtectedJsObject,
    element: ProtectedJsObject,
    svg_element: ProtectedJsObject,
    svg_graphics_element: ProtectedJsObject,
    document: ProtectedJsObject,
    document_type: ProtectedJsObject,
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
    pub(crate) fn pump_storage(&self, js: &JsRuntime) -> Result<bool, JsException> {
        self.state.storage_realm.pump(js)
    }
    pub(crate) fn install(
        runtime: &JsRuntime,
        document: Rc<RefCell<BrowserDocument>>,
        browsing_context: Arc<BrowsingContext>,
        cross_origin_isolated: bool,
        events: crate::runtime::events::EventHub,
        queues: BindingQueues,
        diagnostics: Option<crate::runtime::diagnostics::Diagnostics>,
    ) -> Result<Self, JsException> {
        let state = Rc::new(BindingState {
            crypto: RefCell::new(crypto::CryptoState::default()),
            storage_realm: super::storage::Realm::new(browsing_context.storage.clone()),
            diagnostics,
            events,
            document,
            wrappers: WrapperCache::default(),
            style_wrappers: WrapperCache::default(),
            document_writers: RefCell::new(HashMap::new()),
            prototypes: RefCell::new(None),
            timers: queues.timers,
            browsing_context,
            fetches: queues.fetches,
            streaming: queues.streaming,
            websocket_delivery: RefCell::new(None),
            fetch_stream_delivery: RefCell::new(None),
        });
        let callback_state = Rc::clone(&state);
        runtime.set_global_function("__brimp", move |call| dispatch(&callback_state, &call))?;
        runtime.eval(CLASS_DEFINITIONS)?;
        runtime.eval(include_str!("../../js/vendor/core.bundle.js"))?;
        runtime.eval(include_str!("../../js/web_apis/locks.js"))?;
        runtime.eval(include_str!("../../js/vendor/storage.bundle.js"))?;
        let streaming_state = Rc::clone(&state);
        runtime.set_global_function("__brimpStreamingHost", move |call| {
            dispatch(&streaming_state, &call)
        })?;
        runtime.eval(include_str!("../../js/web_apis/streaming_networking.js"))?;
        runtime.eval(include_str!("../../js/web_apis/webassembly.js"))?;
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
        runtime.eval("delete globalThis.__brimpMarkTrustedEvent")?;
        runtime.eval(if cross_origin_isolated {
            include_str!("../../js/web_apis/cross_origin_isolated.js")
        } else {
            include_str!("../../js/web_apis/cross_origin_unisolated.js")
        })?;

        *state.prototypes.borrow_mut() = Some(Prototypes {
            node: runtime.eval("Node.prototype")?.to_object()?,
            element: runtime.eval("Element.prototype")?.to_object()?,
            svg_element: runtime.eval("SVGElement.prototype")?.to_object()?,
            svg_graphics_element: runtime.eval("SVGGraphicsElement.prototype")?.to_object()?,
            document: runtime.eval("Document.prototype")?.to_object()?,
            document_type: runtime.eval("DocumentType.prototype")?.to_object()?,
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
                ("span", "HTMLSpanElement"),
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
        let parser_mutations = runtime
            .eval("globalThis.__brimpParserMutations")?
            .to_object()?;
        runtime.eval("delete globalThis.__brimpParserMutations")?;
        let input_controller = runtime
            .eval("globalThis.__brimpInputController")?
            .to_object()?;
        runtime.eval("delete globalThis.__brimpInputController")?;
        let script_controller = runtime
            .eval("globalThis.__brimpScriptController")?
            .to_object()?;
        runtime.eval("delete globalThis.__brimpScriptController")?;
        let exposure_finalizer = runtime
            .eval("globalThis.__brimpFinalizeWebIdl")?
            .to_object()?;
        runtime.eval("delete globalThis.__brimpFinalizeWebIdl")?;
        let window_names = runtime
            .eval(include_str!(
                "../../js/web_apis/sync_window_named_properties.js"
            ))?
            .to_object()?;
        let bindings = Self {
            state,
            input_controller,
            parser_mutations,
            script_controller,
            exposure_finalizer,
            window_names,
        };
        bindings.reset_document(runtime)?;
        Ok(bindings)
    }

    pub(crate) fn script_control(
        &self,
        runtime: &JsRuntime,
        command: serde_json::Value,
    ) -> Result<String, JsException> {
        runtime
            .call_function_with_string(&self.script_controller, &command.to_string())?
            .to_string()
    }

    pub(crate) fn finalize_exposure(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        runtime.call_function(&self.exposure_finalizer)?;
        Ok(())
    }

    pub(crate) fn trace_internal_controls(
        &mut self,
        runtime: &JsRuntime,
        wrap: &ProtectedJsObject,
    ) -> Result<(), JsException> {
        for controller in [
            &mut self.input_controller,
            &mut self.parser_mutations,
            &mut self.script_controller,
            &mut self.window_names,
        ] {
            *controller = runtime
                .call_function_with_string_and_object(wrap, "", controller)?
                .to_object()?;
        }
        Ok(())
    }

    pub fn deliver_parser_mutations(
        &self,
        runtime: &JsRuntime,
        records: &[serde_json::Value],
    ) -> Result<(), JsException> {
        runtime.call_function_with_string(
            &self.parser_mutations,
            &serde_json::to_string(records).unwrap(),
        )?;
        Ok(())
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

    pub fn dispatch_input_object(
        &self,
        runtime: &JsRuntime,
        command: &str,
        target: &ProtectedJsObject,
    ) -> Result<String, JsException> {
        runtime
            .call_function_with_string_and_object(&self.input_controller, command, target)?
            .to_string()
    }

    pub fn reset_document(&self, runtime: &JsRuntime) -> Result<(), JsException> {
        self.state.wrappers.clear();
        self.state.style_wrappers.clear();
        let document_id = self.state.document.borrow().root();
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
        let names: Vec<_> = names
            .iter()
            .map(brimp_lexbor::DomString::to_utf16)
            .collect();
        let names = serde_json::to_string(&names).expect("window names serialize as JSON");
        runtime.call_function_with_string(&self.window_names, &names)?;
        Ok(())
    }

    pub fn wrapper_cache(&self) -> &WrapperCache {
        &self.state.wrappers
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
    if operation == "clockNow" {
        return Ok(NativeValue::Number(
            state.timers.borrow().clock.now().as_secs_f64() * 1000.0,
        ));
    }
    if operation == "clockOrigin" {
        return Ok(NativeValue::Number(state.timers.borrow().clock.origin_ms()));
    }
    if operation == "clockLogical" {
        return Ok(NativeValue::Boolean(state.timers.borrow().clock.logical()));
    }
    let result = dispatch_operation(state, call, &operation);
    if let Some(log) = &state.diagnostics {
        log.record(
            "native",
            &format!("native.{operation}"),
            if result.is_ok() { "return" } else { "throw" },
            None,
            Some(serde_json::json!({"argumentCount": call.argument_count().saturating_sub(1)})),
        );
    }
    result
}
fn dispatch_operation(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "mutationObserved" => Ok(NativeValue::Boolean(state.events.has_listener("mutation"))),
        "mutationNode" => node_value(
            state,
            call,
            required_string(call, 2, "node id")?
                .parse::<usize>()
                .map_err(err)?,
        ),
        "mutationNodeId" => Ok(NativeValue::Number(
            required_node_target(state, call)? as f64
        )),
        "mutationEvent" => {
            let record =
                serde_json::from_str(&required_string(call, 2, "mutation record")?).map_err(err)?;
            state
                .events
                .emit(crate::runtime::events::PageEvent::Mutation { record });
            Ok(NativeValue::Undefined)
        }

        operation if is_platform_operation(operation) => {
            dispatch_platform::dispatch(state, call, operation)
        }
        operation => dispatch_dom::dispatch(state, call, operation),
    }
}

fn is_platform_operation(operation: &str) -> bool {
    operation.starts_with("storage")
        || operation.starts_with("webSocket")
        || operation.starts_with("url")
        || operation.starts_with("encoding")
        || operation.starts_with("base64")
        || operation.starts_with("crypto")
        || operation.starts_with("fetch")
        || matches!(
            operation,
            "scheduleTask"
                | "taskPriority"
                | "setTimeout"
                | "setInterval"
                | "clearTimeout"
                | "clearInterval"
                | "location"
                | "locationNavigate"
                | "historyUpdateUrl"
                | "formUrlEncode"
                | "decodeBytes"
                | "encodeUtf8"
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

fn required_dom_string(
    call: &NativeCall<'_>,
    index: usize,
    label: &str,
) -> Result<brimp_lexbor::DomString, NativeError> {
    let units = call
        .argument(index)
        .ok_or_else(|| NativeError::new(format!("missing {label}")))?
        .to_utf16()?;
    Ok(brimp_lexbor::DomString::from_utf16(&units))
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
        .node_id(&object)
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

fn document_url(state: &BindingState, id: NodeId) -> String {
    if id == state.document.borrow().root() {
        state
            .browsing_context
            .current_url()
            .unwrap_or_else(|| "about:blank".into())
    } else {
        state.document.borrow().document_url(id)
    }
}

fn required_parent_node_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let id = required_node_target(state, call)?;
    let is_parent_node = matches!(state.document.borrow().kind(id).ok(), Some(1 | 9 | 11));
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
    if state.document.borrow().is_element(id) {
        Ok(id)
    } else {
        Err(NativeError::new("receiver is not an Element"))
    }
}

fn resolve_document_base_url(
    document: &BrowserDocument,
    owner: NodeId,
    fallback: &str,
    options: url::ParseOptions<'_>,
) -> Option<url::Url> {
    let fallback = url::Url::parse(fallback).ok();
    let base = document.select(owner, "base[href]").ok().and_then(|nodes| {
        nodes
            .into_iter()
            .filter(|id| {
                document.namespace(*id).ok().as_deref() == Some(crate::dom::HTML_NAMESPACE)
            })
            .find_map(|id| {
                document
                    .attributes(id)
                    .ok()?
                    .into_iter()
                    .find(|attribute| {
                        attribute.namespace.is_none() && attribute.local_name == "href"
                    })
                    .map(|attribute| attribute.value)
            })
    });
    base.and_then(|base| {
        options
            .base_url(fallback.as_ref())
            .parse(&base.to_string_lossy())
            .ok()
            .filter(|url| !matches!(url.scheme(), "data" | "javascript"))
    })
    .or(fallback)
}

fn resolve_element_attribute_url(
    document: &BrowserDocument,
    id: NodeId,
    attribute: &str,
    document_url: &str,
    encoding: Option<&str>,
) -> Option<String> {
    let encoding = encoding.and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()));
    let encode: &dyn for<'a> Fn(&'a str) -> std::borrow::Cow<'a, [u8]> = &|input| {
        std::borrow::Cow::Owned(legacy_query_encode(encoding.unwrap(), input).into_bytes())
    };
    let options = url::Url::options().encoding_override(encoding.map(|_| encode));
    let content_attribute = |id, name: &str| {
        document
            .attributes(id)
            .ok()?
            .into_iter()
            .find(|attribute| attribute.namespace.is_none() && attribute.local_name == name)
            .map(|attribute| attribute.value)
    };
    let input = content_attribute(id, attribute)?;
    let base_url = if document.local_name(id).ok().as_deref() == Some("base") {
        url::Url::parse(document_url).ok()
    } else {
        resolve_document_base_url(document, document.node_document(id)?, document_url, options)
    };
    options
        .base_url(base_url.as_ref())
        .parse(&input.to_string_lossy())
        .ok()
        .map(|url| {
            // Safari escapes the final opaque-path space before a query/fragment.
            if encoding.is_some()
                && url.cannot_be_a_base()
                && url.path().ends_with(' ')
                && (url.query().is_some() || url.fragment().is_some())
            {
                let end = url[..url::Position::AfterPath].len();
                format!("{}%20{}", &url.as_str()[..end - 1], &url.as_str()[end..])
            } else {
                url.to_string()
            }
        })
}

fn required_style_target(
    state: &BindingState,
    call: &NativeCall<'_>,
) -> Result<NodeId, NativeError> {
    let object = required_object(call, 1, "style receiver")?;
    state
        .style_wrappers
        .node_id(&object)
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
        let prototypes = prototypes(state);
        match document.kind(node_id).map_err(err)? {
            9 => prototypes.document.identity(),
            10 => prototypes.document_type.identity(),
            11 => prototypes.document_fragment.identity(),
            3 => prototypes.text.identity(),
            8 => prototypes.comment.identity(),
            1 if document.namespace(node_id).ok().as_deref()
                == Some(crate::dom::HTML_NAMESPACE) =>
            {
                let name = document.local_name(node_id).map_err(err)?;
                match name.as_str() {
                    "a" => prototypes.html_anchor_element.identity(),
                    "base" => prototypes.html_base_element.identity(),
                    tag => prototypes
                        .specialized_html_elements
                        .get(tag)
                        .unwrap_or(&prototypes.html_element)
                        .identity(),
                }
            }
            1 if document.namespace(node_id).ok().as_deref()
                == Some("http://www.w3.org/2000/svg") =>
            {
                match document.local_name(node_id).map_err(err)?.as_str() {
                    "svg" | "g" | "a" | "circle" | "ellipse" | "rect" | "path" | "line"
                    | "polyline" | "polygon" | "text" | "tspan" | "textPath" | "image" | "use"
                    | "foreignObject" | "switch" => prototypes.svg_graphics_element.identity(),
                    _ => prototypes.svg_element.identity(),
                }
            }
            1 => prototypes.element.identity(),
            _ => prototypes.node.identity(),
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
    let parent = required_parent_node_target(state, call)?;
    let object = required_object(call, 2, "child")?;
    let child = state.wrappers.node_id(&object).ok_or_else(stale_wrapper)?;
    let doc = state.document.borrow();
    match mutation {
        ChildMutation::Remove => {
            if doc.parent(child).map_err(err)? != Some(parent) {
                return Err(NativeError::new("node is not a child of this parent"));
            }
            doc.detach(child).map_err(err)?;
        }
        ChildMutation::Append => doc.insert(parent, child, None).map_err(err)?,
        ChildMutation::InsertBefore => {
            let before = call
                .argument(3)
                .filter(|value| !value.is_null_or_undefined())
                .map(|value| {
                    value
                        .as_object()?
                        .and_then(|object| state.wrappers.node_id(&object))
                        .ok_or_else(stale_wrapper)
                })
                .transpose()?;
            doc.insert(parent, child, before).map_err(err)?;
        }
    }
    Ok(NativeValue::Object(object))
}

fn inline_style_property(state: &BindingState, node_id: NodeId, name: &str) -> String {
    state
        .document
        .borrow()
        .inline_style_property(node_id, name)
        .unwrap_or_default()
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
        "search": url::quirks::search(url),
        "hash": url::quirks::hash(url),
    }))
    .map_err(err)
}

fn set_url_component(href: &str, component: &str, value: &str) -> Result<String, NativeError> {
    if component == "href" {
        return url::Url::parse(value).map(|url| url.into()).map_err(err);
    }
    let mut url = url::Url::parse(href).map_err(err)?;
    mutate_url_component(&mut url, component, value);
    Ok(url.into())
}

// Return whether an anchor setter updates its content attribute. Safari writes
// unchanged protocol/opaque-host values, but aborts rejected host/port writes.
fn mutate_url_component(url: &mut url::Url, component: &str, value: &str) -> bool {
    use url::quirks;
    match component {
        "protocol" => {
            let scheme = value
                .split(':')
                .next()
                .unwrap_or("")
                .replace(['\t', '\n', '\r'], "");
            if scheme.eq_ignore_ascii_case("file")
                && matches!(url.scheme(), "http" | "https" | "ftp" | "ws" | "wss")
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
            {
                if let Ok(next) =
                    url::Url::parse(&format!("file{}", &url[url::Position::AfterScheme..]))
                {
                    *url = next;
                }
            } else {
                let _ = quirks::set_protocol(url, value);
            }
        }
        "username" => return quirks::set_username(url, value).is_ok(),
        "password" => return quirks::set_password(url, value).is_ok(),
        "host" | "hostname" => return mutate_anchor_host(url, component == "host", value),
        "port" => return quirks::set_port(url, value).is_ok(),
        "pathname" => {
            if url.cannot_be_a_base() {
                return false;
            }
            quirks::set_pathname(url, value);
        }
        "search" => quirks::set_search(url, value),
        "hash" => quirks::set_hash(url, value),
        _ => return false,
    }
    true
}

// Adapt the dependency's setters to measured Safari host mutation behavior.
// See WebKit Source/WebCore/html/URLDecomposition.cpp and Source/WTF/wtf/URL.cpp.
fn mutate_anchor_host(url: &mut url::Url, include_port: bool, value: &str) -> bool {
    if url.cannot_be_a_base() {
        return true;
    }
    let special = matches!(
        url.scheme(),
        "http" | "https" | "ftp" | "ws" | "wss" | "file"
    );
    let value = value
        .split(|c| matches!(c, '/' | '?' | '#') || (special && c == '\\'))
        .next()
        .unwrap_or("");
    if value.contains('@') {
        return true;
    }
    let colon = value
        .rfind(':')
        .filter(|&i| value.rfind(']').is_none_or(|end| end < i));
    if let Some(colon) = colon {
        if !include_port && value.starts_with('[') {
            let candidate = format!(
                "{}{}{}{}",
                &url[..url::Position::BeforeHost],
                if url.has_authority() { "" } else { "//" },
                value,
                &url[url::Position::AfterHost..],
            );
            return match url::Url::parse(&candidate) {
                Ok(next) => {
                    *url = next;
                    true
                }
                Err(_) => false,
            };
        }
        if !include_port || colon == 0 || (value[..colon].contains(':') && !value.starts_with('['))
        {
            return true;
        }
        let host = &value[..colon];
        let digits: String = value[colon + 1..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits.is_empty() {
            return url::quirks::set_hostname(url, host).is_ok();
        }
        if digits.parse::<u16>().is_err() {
            if url::quirks::set_hostname(url, host).is_err() {
                return false;
            }
            let _ = url.set_port(None);
            return true;
        }
        return url::quirks::set_host(url, &format!("{host}:{digits}")).is_ok();
    }
    if include_port {
        url::quirks::set_host(url, value).is_ok()
    } else {
        url::quirks::set_hostname(url, value).is_ok()
    }
}

fn stale_wrapper() -> NativeError {
    NativeError::new("native node no longer exists")
}

fn err(error: impl ToString) -> NativeError {
    NativeError::new(error)
}

#[cfg(test)]
mod tests {
    use super::CookieJar;

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
