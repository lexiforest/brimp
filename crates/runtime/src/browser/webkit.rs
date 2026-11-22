//! WebKit adapter. The browser owns document, layout, input and network state.
use super::{Result, transport::Transport, unsupported};
use crate::runtime::bridge::{BridgeValue, Reference};
use crate::runtime::events::{
    EventHandler, EventHub, PageEvent, RequestRecord, ResponseRecord, Subscription,
};
use crate::runtime::native::{RequestOptions, SessionOptions};
use crate::runtime::{
    AutomationError, CancellationToken, NavigationHistoryEntry, NavigationRequestInfo,
    NavigationResponse,
};
use base64::Engine;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1 << 48);
fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|e| AutomationError::Internal(e.to_string()))
}
fn pairs(value: &Value) -> Vec<(String, String)> {
    value
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").into()))
                .collect()
        })
        .unwrap_or_default()
}
fn check_deadline(start: Instant, timeout: Duration) -> Result<Duration> {
    if CancellationToken::current().is_cancelled() {
        return Err(AutomationError::Cancellation);
    }
    if timeout.is_zero() {
        return Ok(timeout);
    }
    timeout
        .checked_sub(start.elapsed())
        .filter(|d| !d.is_zero())
        .ok_or(AutomationError::Timeout(timeout))
}

pub struct Session {
    transport: Transport,
    options: SessionOptions,
    pages: Mutex<HashMap<String, Page>>,
}
impl Session {
    pub fn new(options: SessionOptions, executable: &Path) -> Result<Arc<Self>> {
        if !options.plugins.is_empty() {
            return unsupported("runtime subsystem plugins; use plugins=[]");
        }
        if options.profile.is_some()
            || options.clock.mode != crate::runtime::clock::TimeMode::System
            || options.clock.epoch_ms.is_some()
            || options.diagnostics.is_some()
            || options.storage_path.is_some()
            || options.identity.user_agent.is_some()
            || options.identity.languages.is_some()
            || options.identity.platform.is_some()
        {
            return unsupported(
                "profile, identity, logical clock, diagnostics or storage_path options",
            );
        }
        let proxy = &options.network.proxy_options;
        if proxy.proxy.is_some() || proxy.proxies.is_some() || proxy.proxy_auth.is_some() {
            return unsupported("curl proxy options");
        }
        crate::runtime::routes::parse_headers(options.headers.clone())?;
        let session = Arc::new(Self {
            transport: Transport::launch(executable)?,
            options,
            pages: Mutex::new(HashMap::new()),
        });
        let weak = Arc::downgrade(&session);
        session.transport.on_event(Arc::new(move |event| {
            if let Some(session) = weak.upgrade() {
                session.event(event);
            }
        }));
        Ok(session)
    }
    fn wrap(self: &Arc<Self>, handle: &str) -> Page {
        self.pages
            .lock()
            .unwrap()
            .entry(handle.into())
            .or_insert_with(|| {
                Page(Arc::new(PageState {
                    session: Arc::downgrade(self),
                    handle: handle.into(),
                    id: next_id(),
                    timeout: self.options.timeout,
                    events: EventHub::default(),
                    closed: AtomicBool::new(false),
                    epoch: AtomicU64::new(next_id()),
                    controller: Mutex::new(None),
                    operation: Mutex::new(()),
                    headers: Mutex::new(self.options.headers.clone()),
                    network: Mutex::new(NetworkState::default()),
                    initialized: AtomicBool::new(false),
                    popup_ready: AtomicBool::new(false),
                    popup_opener: Mutex::new(None),
                }))
            })
            .clone()
    }
    pub fn new_page(self: &Arc<Self>) -> Result<Page> {
        let result = self.transport.call(
            "Automation.createBrowsingContext",
            json!({}),
            self.options.timeout,
        )?;
        let handle = result["handle"]
            .as_str()
            .ok_or_else(|| AutomationError::Internal("missing WebKit page handle".into()))?;
        let page = self.wrap(handle);
        if let Err(error) = page.initialize() {
            let _ = page.close();
            return Err(error);
        }
        Ok(page)
    }
    pub fn pages(&self) -> Vec<Page> {
        self.pages
            .lock()
            .unwrap()
            .values()
            .filter(|p| !p.is_closed())
            .cloned()
            .collect()
    }
    pub fn close(&self) {
        self.transport.close();
        let pages: Vec<_> = self.pages.lock().unwrap().values().cloned().collect();
        for page in pages {
            page.closed();
        }
        self.pages.lock().unwrap().clear();
    }
    fn event(self: &Arc<Self>, event: Value) {
        let params = &event["params"];
        if event["method"] == "Brimp.disconnected" {
            let pages: Vec<_> = self.pages.lock().unwrap().values().cloned().collect();
            for page in pages {
                page.closed();
            }
            return;
        }
        if event["method"] == "Brimp.popup" {
            let handle = params["handle"].as_str().unwrap_or("");
            let child = self.wrap(handle);
            let opener = self
                .pages
                .lock()
                .unwrap()
                .get(params["opener"].as_str().unwrap_or(""))
                .cloned();
            if let Some(opener) = opener {
                *child.0.popup_opener.lock().unwrap() = Some(opener.clone());
                // Initialization may itself emit events; never block the event dispatcher.
                std::thread::spawn(move || match child.initialize() {
                    Ok(()) => child.emit_popup_if_ready(),
                    Err(error) => opener.emit(PageEvent::Error {
                        message: error.to_string(),
                    }),
                });
            }
            return;
        }
        let page = self
            .pages
            .lock()
            .unwrap()
            .get(params["handle"].as_str().unwrap_or(""))
            .cloned();
        let Some(page) = page else { return };
        match event["method"].as_str() {
            Some("Brimp.closed" | "Brimp.crashed") => {
                page.closed();
                self.pages.lock().unwrap().remove(&page.0.handle);
            }
            Some("Brimp.inspectorEvent") => page.inspector_event(&params["message"]),
            Some("Brimp.popupReady") => {
                page.0.popup_ready.store(true, Ordering::Release);
                page.emit_popup_if_ready();
            }
            _ => (),
        }
    }
    pub fn set_cookie(&self, url: &str, name: &str, value: &str) -> Result<()> {
        self.transport.call(
            "Brimp.setCookie",
            json!({"url":url,"name":name,"value":value}),
            self.options.timeout,
        )?;
        Ok(())
    }
    pub fn cookies(&self) -> Result<Vec<crate::web_apis::StoredCookie>> {
        decode(
            self.transport
                .call("Brimp.cookies", json!({}), self.options.timeout)?,
        )
    }
    pub fn clear_cookies(&self) -> Result<()> {
        self.transport
            .call("Brimp.clearCookies", json!({}), self.options.timeout)?;
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.transport.close();
    }
}

#[derive(Default)]
struct NetworkState {
    requests: HashMap<String, RequestState>,
    document: Option<String>,
    url: String,
    main_frame: String,
}
struct RequestState {
    id: u64,
    request: Value,
    response: Value,
    metrics: Value,
    frame: String,
    document: bool,
    history: Vec<NavigationHistoryEntry>,
    done: bool,
    bytes: usize,
}
struct PageState {
    session: Weak<Session>,
    handle: String,
    id: u64,
    timeout: Duration,
    events: EventHub,
    closed: AtomicBool,
    epoch: AtomicU64,
    controller: Mutex<Option<(u64, String)>>,
    operation: Mutex<()>,
    headers: Mutex<Vec<(String, String)>>,
    network: Mutex<NetworkState>,
    initialized: AtomicBool,
    popup_ready: AtomicBool,
    popup_opener: Mutex<Option<Page>>,
}
#[derive(Clone)]
pub struct Page(Arc<PageState>);
impl Page {
    fn session(&self) -> Result<Arc<Session>> {
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        self.0.session.upgrade().ok_or(AutomationError::Closed)
    }
    pub fn id(&self) -> u64 {
        self.0.id
    }
    pub fn default_timeout(&self) -> Duration {
        self.0.timeout
    }
    pub fn is_closed(&self) -> bool {
        self.0.closed.load(Ordering::Acquire)
            || self
                .0
                .session
                .upgrade()
                .is_none_or(|s| s.transport.is_closed())
    }
    fn closed(&self) {
        if !self.0.closed.swap(true, Ordering::AcqRel) {
            self.0.network.lock().unwrap().requests.clear();
            *self.0.controller.lock().unwrap() = None;
            self.0.popup_opener.lock().unwrap().take();
            self.emit(PageEvent::Close);
        }
    }
    fn emit(&self, event: PageEvent) {
        let _guard = crate::runtime::plugins::enter_page_callback(self.id());
        self.0.events.emit(event);
    }
    pub fn check_callback(&self) -> Result<()> {
        if crate::runtime::plugins::is_callback_page(self.id()) {
            return Err(AutomationError::Reentry);
        }
        self.session().map(|_| ())
    }
    pub fn on(&self, event: &str, handler: EventHandler) -> Result<Subscription> {
        self.session()?;
        if matches!(
            event,
            "mutation" | "websocket" | "eventsource" | "popup_requested"
        ) {
            return unsupported(&format!("{event} events"));
        }
        self.0.events.on(event, handler)
    }
    fn lock_operation(
        &self,
        started: Instant,
        timeout: Duration,
    ) -> Result<std::sync::MutexGuard<'_, ()>> {
        loop {
            check_deadline(started, timeout)?;
            match self.0.operation.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(AutomationError::Internal(
                        "page operation lock poisoned".into(),
                    ));
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    std::thread::sleep(Duration::from_millis(5))
                }
            }
        }
    }
    fn call(&self, method: &str, mut params: Value, timeout: Duration) -> Result<Value> {
        params["handle"] = json!(self.0.handle);
        self.session()?.transport.call(method, params, timeout)
    }
    fn inspector(&self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        self.call(
            "Brimp.inspector",
            json!({"method":method,"params":params}),
            timeout,
        )
    }
    fn emit_popup_if_ready(&self) {
        if self.0.initialized.load(Ordering::Acquire) && self.0.popup_ready.load(Ordering::Acquire)
        {
            let opener = self.0.popup_opener.lock().unwrap().take();
            if let Some(opener) = opener {
                opener.emit(PageEvent::Popup { page_id: self.id() });
            }
        }
    }
    fn initialize(&self) -> Result<()> {
        self.inspector(
            "Target.setPauseOnStart",
            json!({"pauseOnStart":true}),
            self.0.timeout,
        )?;
        self.inspector("Page.enable", json!({}), self.0.timeout)?;
        let tree = self.inspector("Page.getResourceTree", json!({}), self.0.timeout)?;
        self.0.network.lock().unwrap().main_frame = tree["frameTree"]["frame"]["id"]
            .as_str()
            .unwrap_or("")
            .into();
        self.inspector("Runtime.enable", json!({}), self.0.timeout)?;
        self.inspector("Console.enable", json!({}), self.0.timeout)?;
        self.inspector("Network.enable", json!({}), self.0.timeout)?;
        self.inspector(
            "Network.setClearResourceDataOnNavigate",
            json!({"clearResourceDataOnNavigate":false}),
            self.0.timeout,
        )?;
        self.set_extra_headers(self.session()?.options.headers.clone())?;
        self.0.initialized.store(true, Ordering::Release);
        Ok(())
    }
    fn inspector_event(&self, event: &Value) {
        let p = &event["params"];
        let method = event["method"].as_str().unwrap_or("");
        if method == "Target.targetCreated" && p["targetInfo"]["isPaused"] == true {
            let page = self.clone();
            let target = p["targetInfo"]["targetId"]
                .as_str()
                .unwrap_or("")
                .to_owned();
            let is_page = p["targetInfo"]["type"] == "page";
            std::thread::spawn(move || {
                let initialize = || -> Result<()> {
                    if is_page {
                        let headers: serde_json::Map<String, Value> = page
                            .0
                            .headers
                            .lock()
                            .unwrap()
                            .iter()
                            .map(|(name, value)| (name.clone(), json!(value)))
                            .collect();
                        for (method, params) in [
                            ("Page.enable", json!({})),
                            ("Runtime.enable", json!({})),
                            ("Console.enable", json!({})),
                            ("Network.enable", json!({})),
                            ("Network.setExtraHTTPHeaders", json!({"headers":headers})),
                            (
                                "Network.setClearResourceDataOnNavigate",
                                json!({"clearResourceDataOnNavigate":false}),
                            ),
                        ] {
                            page.call(
                                "Brimp.inspector",
                                json!({"target":target,"method":method,"params":params}),
                                page.0.timeout,
                            )?;
                        }
                    }
                    Ok(())
                };
                let result = initialize();
                let resumed =
                    page.inspector("Target.resume", json!({"targetId":target}), page.0.timeout);
                if let Err(error) = result.and(resumed.map(|_| ())) {
                    page.emit(PageEvent::Error {
                        message: error.to_string(),
                    });
                }
            });
            return;
        }
        let mut network = self.0.network.lock().unwrap();
        let observation = match method {
            "Console.messageAdded" => {
                let message = &p["message"];
                if message["source"] == "javascript" && message["level"] == "error" {
                    Some(PageEvent::Error {
                        message: message["text"].as_str().unwrap_or("").into(),
                    })
                } else {
                    Some(PageEvent::Console {
                        level: match message["level"].as_str().unwrap_or("log") {
                            "warning" => "warn",
                            other => other,
                        }
                        .into(),
                        message: message["text"].as_str().unwrap_or("").into(),
                    })
                }
            }
            "Page.frameNavigated" => {
                if p["frame"]["parentId"].is_null() {
                    network.main_frame = p["frame"]["id"].as_str().unwrap_or("").into();
                    network.document = network
                        .requests
                        .iter()
                        .filter(|(_, request)| {
                            request.document && request.frame == network.main_frame
                        })
                        .max_by_key(|(_, request)| request.id)
                        .map(|(key, _)| key.clone());
                    network.url = p["frame"]["url"].as_str().unwrap_or("about:blank").into();
                    self.0.epoch.store(next_id(), Ordering::Release);
                    *self.0.controller.lock().unwrap() = None;
                }
                None
            }
            "Page.domContentEventFired" | "Page.loadEventFired" => Some(PageEvent::Lifecycle {
                name: if method == "Page.loadEventFired" {
                    "load"
                } else {
                    "dom_content_loaded"
                }
                .into(),
                url: network.url.clone(),
            }),
            "Network.requestWillBeSent" => {
                let key = p["requestId"].as_str().unwrap_or("").to_owned();
                let id = next_id();
                let mut history = Vec::new();
                if let Some(previous) = network.requests.get(&key) {
                    history = previous.history.clone();
                    if p["redirectResponse"].is_object() {
                        let response = &p["redirectResponse"];
                        history.push(NavigationHistoryEntry {
                            status_code: response["status"].as_u64().unwrap_or(0) as u16,
                            reason: response["statusText"].as_str().unwrap_or("").into(),
                            url: response["url"].as_str().unwrap_or("").into(),
                            headers: pairs(&response["headers"]),
                            request: NavigationRequestInfo {
                                method: previous.request["method"].as_str().unwrap_or("GET").into(),
                                url: previous.request["url"].as_str().unwrap_or("").into(),
                                headers: pairs(&previous.request["headers"]),
                                body: previous.request["postData"]
                                    .as_str()
                                    .map(|s| s.as_bytes().to_vec()),
                            },
                        });
                    }
                }
                network.requests.insert(
                    key,
                    RequestState {
                        id,
                        request: p["request"].clone(),
                        response: Value::Null,
                        metrics: Value::Null,
                        frame: p["frameId"].as_str().unwrap_or("").into(),
                        document: false,
                        history,
                        done: false,
                        bytes: 0,
                    },
                );
                Some(PageEvent::Request {
                    request: RequestRecord {
                        id,
                        method: p["request"]["method"].as_str().unwrap_or("GET").into(),
                        url: p["request"]["url"].as_str().unwrap_or("").into(),
                        headers: pairs(&p["request"]["headers"]),
                        body: p["request"]["postData"]
                            .as_str()
                            .map(|v| v.as_bytes().to_vec()),
                        resource_type: p["type"].as_str().map(|v| v.to_ascii_lowercase()),
                    },
                })
            }
            "Network.responseReceived" => {
                let key = p["requestId"].as_str().unwrap_or("");
                if p["type"] == "Document"
                    && p["frameId"].as_str() == Some(network.main_frame.as_str())
                {
                    network.document = Some(key.into());
                }
                network.requests.get_mut(key).map(|request| {
                    request.response = p["response"].clone();
                    request.document = p["type"] == "Document";
                    PageEvent::Response {
                        response: ResponseRecord {
                            request_id: request.id,
                            url: p["response"]["url"].as_str().unwrap_or("").into(),
                            status: p["response"]["status"].as_f64().unwrap_or(0.0) as u16,
                            headers: pairs(&p["response"]["headers"]),
                            from_cache: matches!(
                                p["response"]["source"].as_str(),
                                Some("disk-cache" | "memory-cache")
                            ),
                        },
                    }
                })
            }
            "Network.dataReceived" => {
                if let Some(r) = network
                    .requests
                    .get_mut(p["requestId"].as_str().unwrap_or(""))
                {
                    r.bytes += p["dataLength"].as_u64().unwrap_or(0) as usize;
                }
                None
            }
            "Network.loadingFinished" | "Network.loadingFailed" => network
                .requests
                .get_mut(p["requestId"].as_str().unwrap_or(""))
                .map(|r| {
                    r.done = true;
                    r.metrics = p["metrics"].clone();
                    if method == "Network.loadingFinished" {
                        PageEvent::RequestFinished {
                            request_id: r.id,
                            bytes: r.bytes,
                        }
                    } else {
                        PageEvent::RequestFailed {
                            request_id: r.id,
                            message: p["errorText"].as_str().unwrap_or("network error").into(),
                            cancelled: p["canceled"].as_bool().unwrap_or(false),
                        }
                    }
                }),
            _ => None,
        };
        drop(network);
        if let Some(event) = observation {
            self.emit(event);
        }
    }
    fn controller(&self, timeout: Duration) -> Result<String> {
        let mut cache = self.0.controller.lock().unwrap();
        let epoch = self.0.epoch.load(Ordering::Acquire);
        if let Some((generation, id)) = &*cache
            && *generation == epoch
        {
            return Ok(id.clone());
        }
        let source = format!(
            "({})(({})({epoch}), ({}))",
            include_str!("controller.js"),
            include_str!("../../js/internal/bridge.js"),
            include_str!("../../js/internal/automation.js")
        );
        let result = self.inspector(
            "Runtime.evaluate",
            json!({"expression":source,"objectGroup":"brimp"}),
            timeout,
        )?;
        let object = remote_result(result)?;
        let id = object["objectId"]
            .as_str()
            .ok_or_else(|| AutomationError::Internal("missing controller object".into()))?
            .to_owned();
        *cache = Some((epoch, id.clone()));
        Ok(id)
    }
    fn execute(&self, mut command: Value, timeout: Duration) -> Result<BridgeValue> {
        self.check_callback()?;
        let started = Instant::now();
        let controller = match self.controller(timeout) {
            Ok(controller) => controller,
            Err(error @ (AutomationError::Timeout(_) | AutomationError::Cancellation)) => {
                let _scope = CancellationToken::new().enter();
                let _ = self.call("Brimp.terminatePage", json!({}), Duration::from_secs(1));
                self.closed();
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        let id = next_id();
        command["id"] = json!(id);
        command["timeout"] = json!(check_deadline(started, timeout)?.as_secs_f64() * 1000.0);
        let result = self.inspector("Runtime.callFunctionOn", json!({"objectId":controller,"functionDeclaration":"function(command) { return this(command); }","arguments":[{"value":command}],"returnByValue":true,"awaitPromise":true}), check_deadline(started,timeout)?);
        if matches!(
            result,
            Err(AutomationError::Timeout(_) | AutomationError::Cancellation)
        ) {
            let _scope = CancellationToken::new().enter();
            let cancelled = self.inspector("Runtime.callFunctionOn", json!({"objectId":controller,"functionDeclaration":"function(id) { return this({op:'cancel', id}); }","arguments":[{"value":id}],"returnByValue":true}), Duration::from_millis(250));
            if matches!(cancelled, Err(AutomationError::Timeout(_))) {
                // A synchronous script cannot run its cancellation callback. Stop
                // its process so an infinite loop cannot outlive the operation.
                let _ = self.call("Brimp.terminatePage", json!({}), Duration::from_secs(1));
                self.closed();
            }
        }
        let result = result?;
        if result["wasThrown"] == true
            && result["result"]["description"]
                .as_str()
                .is_some_and(|message| message.contains("timeout: operation timed out"))
        {
            return Err(AutomationError::Timeout(timeout));
        }
        let object = remote_result(result)?;
        decode(object["value"].clone())
    }
    pub fn automate(&self, command: Value, timeout: Option<Duration>) -> Result<BridgeValue> {
        let timeout = timeout.unwrap_or(self.0.timeout);
        let started = Instant::now();
        let _lock = self.lock_operation(started, timeout)?;
        let result = self.execute(
            json!({"op":"automate","command":command}),
            check_deadline(started, timeout)?,
        )?;
        if command["op"] == "input" {
            let (x, y) = coordinates(result)?;
            self.input(&command["input"], x, y, check_deadline(started, timeout)?)?;
            return Ok(BridgeValue::Null);
        }
        Ok(result)
    }
    pub fn current_url(&self) -> Result<String> {
        copied_string(self.execute(
            json!({"op":"evaluate","source":"location.href"}),
            self.0.timeout,
        )?)
    }
    pub fn html(&self) -> Result<String> {
        copied_string(self.execute(json!({"op":"evaluate","source":"(document.doctype ? new XMLSerializer().serializeToString(document.doctype) : '') + document.documentElement.outerHTML"}),self.0.timeout)?)
    }
    pub fn load_html(&self, html: &str, url: &str) -> Result<()> {
        self.check_callback()?;
        let started = Instant::now();
        let _lock = self.lock_operation(started, self.0.timeout)?;
        url::Url::parse(url).map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        self.call(
            "Brimp.loadHTML",
            json!({"html":html,"url":url}),
            check_deadline(started, self.0.timeout)?,
        )?;
        self.0.epoch.store(next_id(), Ordering::Release);
        *self.0.controller.lock().unwrap() = None;
        Ok(())
    }
    pub fn request(
        &self,
        method: &str,
        url: &str,
        options: RequestOptions,
    ) -> Result<NavigationResponse> {
        self.check_callback()?;
        let parsed = url::Url::parse(url)
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return unsupported("HTTP response snapshots for non-HTTP URLs");
        }
        let parsed_method = http::Method::from_bytes(method.as_bytes())
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        if matches!(parsed_method, http::Method::GET | http::Method::HEAD) && options.body.is_some()
        {
            return Err(AutomationError::InvalidInput(format!(
                "{method} navigation cannot have a body"
            )));
        }
        if !options.allow_redirects
            || options.max_redirects != 20
            || options.proxy_options.proxy.is_some()
            || options.proxy_options.proxies.is_some()
            || options.proxy_options.proxy_auth.is_some()
        {
            return unsupported("custom redirect or proxy policy");
        }
        let _scope = options.cancellation.enter();
        let started = Instant::now();
        let timeout = options.timeout.unwrap_or(self.0.timeout);
        let _lock = self.lock_operation(started, timeout)?;
        let mut headers = self.0.headers.lock().unwrap().clone();
        if options.headers.iter().any(|(name, value)| {
            headers.iter().any(|(default_name, default_value)| {
                default_name.eq_ignore_ascii_case(name) && default_value != value
            })
        }) {
            return unsupported(
                "per-request overrides of page extra headers; update the page headers first",
            );
        }
        for (name, value) in &options.headers {
            headers.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
            headers.push((name.clone(), value.clone()));
        }
        crate::runtime::routes::parse_headers(headers.clone())?;
        self.0.network.lock().unwrap().document = None;
        let wait_until = options
            .wait_until
            .unwrap_or(self.session()?.options.wait_until);
        let navigated = self.call("Brimp.request", json!({"waitUntil": if wait_until == crate::runtime::NavigationWait::Load {"load"} else {"domcontentloaded"},"method":method,"url":url,"headers":headers,"body":options.body.as_ref().map(|v| base64::engine::general_purpose::STANDARD.encode(v))}),check_deadline(started, timeout)?);
        if navigated.is_err() {
            let _scope = CancellationToken::new().enter();
            let _ = self.call("Brimp.stopLoading", json!({}), Duration::from_millis(250));
        }
        navigated?;
        let (request_id, response, request, metrics, history) = loop {
            let record = {
                let network = self.0.network.lock().unwrap();
                network.document.as_ref().and_then(|key| {
                    network.requests.get(key).filter(|r| r.done).map(|r| {
                        (
                            key.clone(),
                            r.response.clone(),
                            r.request.clone(),
                            r.metrics.clone(),
                            r.history.clone(),
                        )
                    })
                })
            };
            if let Some(record) = record {
                break record;
            }
            check_deadline(started, timeout)?;
            if self.is_closed() {
                return Err(AutomationError::Closed);
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let content = self.body(&request_id, check_deadline(started, timeout)?)?;
        let status = response["status"].as_f64().unwrap_or(0.0) as u16;
        let response_headers = pairs(&response["headers"]);
        Ok(NavigationResponse {
            status_code: status,
            reason: response["statusText"].as_str().unwrap_or("").into(),
            url: response["url"].as_str().unwrap_or(url).into(),
            headers: response_headers,
            downloaded_bytes: metrics["responseBodyBytesReceived"]
                .as_u64()
                .unwrap_or(content.len() as u64),
            uploaded_bytes: metrics["requestBodyBytesSent"].as_u64().unwrap_or(0),
            header_bytes: metrics["responseHeaderBytesReceived"].as_u64().unwrap_or(0),
            content,
            html: None,
            cookies: pairs(&response["headers"])
                .into_iter()
                .filter(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
                .flat_map(|(_, value)| {
                    value
                        .split('\n')
                        .filter_map(|line| {
                            line.split(';')
                                .next()?
                                .split_once('=')
                                .map(|(name, value)| {
                                    (name.trim().to_owned(), value.trim().to_owned())
                                })
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            elapsed: started.elapsed(),
            history,
            http_version: metrics["protocol"].as_str().map(str::to_owned),
            request: NavigationRequestInfo {
                method: request["method"].as_str().unwrap_or(method).into(),
                url: request["url"].as_str().unwrap_or(url).into(),
                headers: pairs(&request["headers"]),
                body: if matches!(request["method"].as_str(), Some("GET" | "HEAD")) {
                    None
                } else {
                    options.body
                },
            },
        })
    }
    fn body(&self, id: &str, timeout: Duration) -> Result<Vec<u8>> {
        let result = self.inspector(
            "Network.getResponseBody",
            json!({"requestId":id,"raw":true}),
            timeout,
        )?;
        let content = result["body"].as_str().unwrap_or("");
        if result["base64Encoded"] == true {
            base64::engine::general_purpose::STANDARD
                .decode(content)
                .map_err(|e| AutomationError::Internal(e.to_string()))
        } else {
            Ok(content.as_bytes().to_vec())
        }
    }
    pub fn response_body(&self, id: u64) -> Result<Vec<u8>> {
        let key = self
            .0
            .network
            .lock()
            .unwrap()
            .requests
            .iter()
            .find(|(_, v)| v.id == id)
            .map(|(k, _)| k.clone())
            .ok_or_else(|| AutomationError::InvalidInput("unknown request".into()))?;
        self.body(&key, self.0.timeout)
    }
    pub fn set_extra_headers(&self, headers: Vec<(String, String)>) -> Result<()> {
        crate::runtime::routes::parse_headers(headers.clone())?;
        self.inspector("Network.setExtraHTTPHeaders",json!({"headers":headers.iter().map(|(key,value)|(key.clone(),json!(value))).collect::<serde_json::Map<_,_>>()}),self.0.timeout)?;
        *self.0.headers.lock().unwrap() = headers;
        Ok(())
    }
    pub fn clear_cache(&self) -> Result<()> {
        self.call("Brimp.clearCache", json!({}), self.0.timeout)?;
        Ok(())
    }
    pub fn add_init_script(&self, source: &str) -> Result<()> {
        self.call(
            "Brimp.addInitScript",
            json!({"source": source}),
            self.0.timeout,
        )?;
        Ok(())
    }
    pub fn wait_for_network_idle(
        &self,
        quiet: Duration,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<()> {
        let _scope = cancellation.enter();
        let start = Instant::now();
        let mut idle = None;
        loop {
            check_deadline(start, timeout)?;
            if self.is_closed() {
                return Err(AutomationError::Closed);
            }
            if self
                .0
                .network
                .lock()
                .unwrap()
                .requests
                .values()
                .all(|r| r.done)
            {
                if idle.get_or_insert_with(Instant::now).elapsed() >= quiet {
                    return Ok(());
                }
            } else {
                idle = None;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn close(&self) -> Result<()> {
        if !self.is_closed() {
            self.call("Brimp.closePage", json!({}), Duration::from_secs(2))?;
        }
        self.closed();
        Ok(())
    }
    pub fn evaluate_handle(&self, source: &str) -> Result<Handle> {
        let wire = self.execute(
            json!({"op":"evaluate","source":source,"handle":true}),
            self.0.timeout,
        )?;
        self.handle(wire)
    }
    fn handle(&self, wire: BridgeValue) -> Result<Handle> {
        match wire {
            BridgeValue::Reference(token) => Ok(Handle(Arc::new(HandleState {
                page: self.clone(),
                token,
                disposed: AtomicBool::new(false),
            }))),
            _ => Err(AutomationError::Internal("expected handle".into())),
        }
    }
    fn query(&self, selector: &str, token: Option<Reference>, all: bool) -> Result<Vec<Handle>> {
        let wire = self.execute(
            json!({"op":"query","selector":selector,"token":token,"all":all}),
            self.0.timeout,
        )?;
        match wire {
            BridgeValue::Array(values) => values.into_iter().map(|v| self.handle(v)).collect(),
            _ => Err(AutomationError::Internal("expected handles".into())),
        }
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<Handle>> {
        Ok(self.query(selector, None, false)?.into_iter().next())
    }
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<Handle>> {
        self.query(selector, None, true)
    }
    fn input(&self, input: &Value, x: f64, y: f64, timeout: Duration) -> Result<()> {
        match input["action"].as_str().unwrap_or("") {
            "click" | "tap" => self.mouse_with_timeout("SingleClick", x, y, "left", 0, timeout),
            "hover" => self.mouse_with_timeout("Move", x, y, "none", 0, timeout),
            "focus" => Ok(()),
            "fill" | "type" => {
                if input["text"] == "" {
                    return if input["action"] == "fill" {
                        self.keyboard("Backspace", timeout)
                    } else {
                        Ok(())
                    };
                }
                self.call(
                    "Automation.performKeyboardInteractions",
                    json!({"interactions":[{"type":"InsertByKey","text":input["text"]}]}),
                    timeout,
                )?;
                Ok(())
            }
            "press" => self.keyboard(input["key"].as_str().unwrap_or(""), timeout),
            _ => unsupported("input action"),
        }
    }
    fn mouse(&self, interaction: &str, x: f64, y: f64, button: &str, modifiers: u8) -> Result<()> {
        self.mouse_with_timeout(interaction, x, y, button, modifiers, self.0.timeout)
    }
    fn mouse_with_timeout(
        &self,
        interaction: &str,
        x: f64,
        y: f64,
        button: &str,
        modifiers: u8,
        timeout: Duration,
    ) -> Result<()> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AutomationError::InvalidInput(
                "mouse coordinates must be finite".into(),
            ));
        }
        let button = match button {
            "left" => "Left",
            "right" => "Right",
            "middle" => "Middle",
            "none" => "None",
            _ => return Err(AutomationError::InvalidInput("invalid mouse button".into())),
        };
        let keys: Vec<_> = [(1, "Shift"), (2, "Control"), (4, "Alt"), (8, "Meta")]
            .into_iter()
            .filter(|(mask, _)| modifiers & mask != 0)
            .map(|(_, name)| name)
            .collect();
        self.call("Automation.performMouseInteraction",json!({"position":{"x":x,"y":y},"button":button,"interaction":interaction,"modifiers":keys}),timeout)?;
        Ok(())
    }
    fn keyboard(&self, key: &str, timeout: Duration) -> Result<()> {
        if key.chars().count() == 1 {
            self.call(
                "Automation.performKeyboardInteractions",
                json!({"interactions":[{"type":"InsertByKey","text":key}]}),
                timeout,
            )?;
            return Ok(());
        }
        let key = match key {
            "ArrowLeft" => "LeftArrow",
            "ArrowRight" => "RightArrow",
            "ArrowUp" => "UpArrow",
            "ArrowDown" => "DownArrow",
            "Alt" => "Alternate",
            "F1" => "Function1",
            "F2" => "Function2",
            "F3" => "Function3",
            "F4" => "Function4",
            "F5" => "Function5",
            "F6" => "Function6",
            "F7" => "Function7",
            "F8" => "Function8",
            "F9" => "Function9",
            "F10" => "Function10",
            "F11" => "Function11",
            "F12" => "Function12",
            other => other,
        };
        self.call(
            "Automation.performKeyboardInteractions",
            json!({"interactions":[{"type":"KeyPress","key":key},{"type":"KeyRelease","key":key}]}),
            timeout,
        )?;
        Ok(())
    }
    fn selector_input(&self, selector: &str, input: Value) -> Result<()> {
        self.automate(
            json!({"op":"input","selectors":[selector],"input":input}),
            None,
        )?;
        Ok(())
    }
    pub fn click(&self, selector: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"click"}))
    }
    pub fn hover(&self, selector: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"hover"}))
    }
    pub fn tap(&self, selector: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"tap"}))
    }
    pub fn focus(&self, selector: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"focus"}))
    }
    pub fn fill(&self, selector: &str, text: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"fill","text":text}))
    }
    pub fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        self.selector_input(selector, json!({"action":"type","text":text}))
    }
    pub fn press(&self, selector: &str, key: &str, modifiers: u8) -> Result<()> {
        if modifiers != 0 {
            return unsupported("keyboard modifiers");
        }
        self.selector_input(selector, json!({"action":"press","key":key}))
    }
    pub fn mouse_move(&self, x: f64, y: f64, steps: u32) -> Result<()> {
        if steps != 1 {
            return unsupported("interpolated mouse movement");
        }
        self.mouse("Move", x, y, "none", 0)
    }
    pub fn mouse_down(
        &self,
        x: f64,
        y: f64,
        button: &str,
        count: u32,
        modifiers: u8,
    ) -> Result<()> {
        if count != 1 {
            return unsupported("mouse-down click count");
        }
        self.mouse("Down", x, y, button, modifiers)
    }
    pub fn mouse_up(&self, x: f64, y: f64, button: &str, count: u32, modifiers: u8) -> Result<()> {
        if count != 1 {
            return unsupported("mouse-up click count");
        }
        self.mouse("Up", x, y, button, modifiers)
    }
    pub fn mouse_click(
        &self,
        x: f64,
        y: f64,
        button: &str,
        count: u32,
        modifiers: u8,
    ) -> Result<()> {
        if !(1..=2).contains(&count) {
            return unsupported("click counts other than one or two");
        }
        self.mouse(
            if count == 2 {
                "DoubleClick"
            } else {
                "SingleClick"
            },
            x,
            y,
            button,
            modifiers,
        )
    }
    pub fn mouse_wheel(&self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        if ![x, y, dx, dy].iter().all(|value| value.is_finite()) {
            return Err(AutomationError::InvalidInput(
                "wheel coordinates and deltas must be finite".into(),
            ));
        }
        self.call("Automation.performInteractionSequence", json!({
            "inputSources":[{"sourceId":"wheel","sourceType":"Wheel"}],
            "steps":[{"states":[{"sourceId":"wheel","location":{"x":x,"y":y},"delta":{"width":dx.round() as i64,"height":dy.round() as i64}}]}]
        }),self.0.timeout)?;
        Ok(())
    }
}

fn coordinates(value: BridgeValue) -> Result<(f64, f64)> {
    if let BridgeValue::Array(values) = value
        && let [BridgeValue::Number(x), BridgeValue::Number(y)] = values.as_slice()
        && x.is_finite()
        && y.is_finite()
    {
        return Ok((*x, *y));
    }
    Err(AutomationError::Internal("invalid element geometry".into()))
}
fn copied_string(value: BridgeValue) -> Result<String> {
    match value {
        BridgeValue::String(s) => Ok(s),
        _ => Err(AutomationError::Internal("expected string".into())),
    }
}
fn remote_result(result: Value) -> Result<Value> {
    if result["wasThrown"] == true {
        let message = result["result"]["description"]
            .as_str()
            .unwrap_or("JavaScript evaluation failed");
        if message.contains("stale_handle:") {
            return Err(AutomationError::StaleHandle);
        }
        if message.contains("invalid_input:") {
            return Err(AutomationError::InvalidInput(message.into()));
        }
        if message.contains("unsupported:") {
            return unsupported(message);
        }
        return Err(AutomationError::JavaScript(message.into()));
    }
    Ok(result["result"].clone())
}

struct HandleState {
    page: Page,
    token: Reference,
    disposed: AtomicBool,
}
impl Drop for HandleState {
    fn drop(&mut self) {
        if !self.disposed.load(Ordering::Acquire)
            && !self.page.is_closed()
            && self.token.realm == self.page.0.epoch.load(Ordering::Acquire)
        {
            let _scope = CancellationToken::new().enter();
            let _ = self.page.execute(
                json!({"op":"bridge","command":{"op":"dispose","token":self.token}}),
                Duration::from_millis(250),
            );
        }
    }
}
#[derive(Clone)]
pub struct Handle(Arc<HandleState>);
impl Handle {
    pub fn click(&self) -> Result<()> {
        let value = self.bridge_command(json!({"op":"call","source":"node => { if (!node.isConnected) throw new Error('stale_handle: detached node'); node.scrollIntoView({block:'center',inline:'center'}); const r = node.getBoundingClientRect(); return [r.x+r.width/2,r.y+r.height/2]; }"}))?;
        let (x, y) = coordinates(value)?;
        self.0.page.mouse_click(x, y, "left", 1, 0)
    }
    pub fn reference(&self) -> Result<Reference> {
        if self.0.disposed.load(Ordering::Acquire)
            || self.0.token.realm != self.0.page.0.epoch.load(Ordering::Acquire)
        {
            return Err(AutomationError::StaleHandle);
        }
        self.0.page.session()?;
        Ok(self.0.token)
    }
    pub fn peer_handle(&self, token: Reference) -> Self {
        Self(Arc::new(HandleState {
            page: self.0.page.clone(),
            token,
            disposed: AtomicBool::new(false),
        }))
    }
    pub fn bridge_command(&self, mut command: Value) -> Result<BridgeValue> {
        command["token"] = json!(self.reference()?);
        self.0.page.execute(
            json!({"op":"bridge","command":command}),
            self.0.page.0.timeout,
        )
    }
    pub fn value(&self) -> Result<BridgeValue> {
        self.bridge_command(json!({"op":"value"}))
    }
    pub fn json_value(&self) -> Result<Value> {
        serde_json::from_str(&copied_string(self.bridge_command(json!({"op":"json"}))?)?)
            .map_err(|e| AutomationError::Internal(e.to_string()))
    }
    pub fn dispose(&self) -> Result<()> {
        if !self.0.disposed.swap(true, Ordering::AcqRel)
            && !self.0.page.is_closed()
            && self.0.token.realm == self.0.page.0.epoch.load(Ordering::Acquire)
        {
            self.0.page.execute(
                json!({"op":"bridge","command":{"op":"dispose","token":self.0.token}}),
                self.0.page.0.timeout,
            )?;
        }
        Ok(())
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<Handle>> {
        Ok(self
            .0
            .page
            .query(selector, Some(self.reference()?), false)?
            .into_iter()
            .next())
    }
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<Handle>> {
        self.0.page.query(selector, Some(self.reference()?), true)
    }
}
