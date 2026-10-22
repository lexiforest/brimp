//! Typed native API. JavaScript objects never leave the page owner thread.

use brimp_jsc::ProtectedJsObject;

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::{
    AutomationError, CancellationToken, NavigationResponse, Page, PageHandle, PageOptions,
    SessionCookies,
};
use crate::network::{CurlConfig, ResourceLoader};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct IdentityOptions {
    pub user_agent: Option<String>,
    pub languages: Option<Vec<String>>,
    pub platform: Option<String>,
}
impl IdentityOptions {
    pub(crate) fn apply(
        &self,
        persona: &mut crate::persona::PersonaConfig,
    ) -> Result<(), AutomationError> {
        if let Some(value) = &self.user_agent {
            persona.base_headers.user_agent = value.clone();
        }
        if let Some(values) = &self.languages {
            if values.is_empty()
                || values
                    .iter()
                    .any(|value| value.trim().is_empty() || value.chars().any(char::is_control))
            {
                return Err(AutomationError::InvalidInput(
                    "languages must be nonempty header-safe strings".into(),
                ));
            }
            persona.languages = values.clone();
            persona.locale = values[0].clone();
            persona.base_headers.accept_language = values.join(",");
        }
        if let Some(value) = &self.platform {
            persona.platform = value.clone();
        }
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct SessionOptions {
    pub clock: super::clock::ClockOptions,
    pub diagnostics: Option<super::diagnostics::DiagnosticsOptions>,
    pub identity: IdentityOptions,
    pub headers: Vec<(String, String)>,
    pub timeout: Duration,
    pub network: CurlConfig,
    pub plugins: Vec<super::plugins::Plugin>,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            diagnostics: None,
            identity: IdentityOptions::default(),
            headers: Vec::new(),
            timeout: Duration::from_secs(30),
            network: CurlConfig::default(),
            plugins: Vec::new(),
        }
    }
}

/// Owns pages and a cookie jar. Cloning a page does not extend the session lifetime.
pub struct Session {
    pub(super) id: u64,
    pub(super) loader: Arc<dyn ResourceLoader>,
    pub(super) persona: Option<crate::persona::PersonaConfig>,
    pub(super) closed: AtomicBool,
    pub(super) state: std::sync::Mutex<super::session::SessionState>,
    pub(super) cookie_store: SessionCookies,
    pub(super) network_config: Option<CurlConfig>,
    pub(super) options: SessionOptions,
    pub(super) network_executor: super::networking::NetworkExecutor,
}

impl Session {
    pub fn new(options: SessionOptions) -> Result<Self, AutomationError> {
        validate_timeout(options.timeout)?;
        let mut persona = crate::persona::PersonaConfig::default();
        options.identity.apply(&mut persona)?;
        Self::create(options, Some(persona), None)
    }

    pub fn with_resource_loader(
        options: SessionOptions,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, AutomationError> {
        validate_timeout(options.timeout)?;
        let mut persona = crate::persona::PersonaConfig::default();
        options.identity.apply(&mut persona)?;
        Self::create(options, Some(persona), Some(loader))
    }

    pub fn new_page(&self) -> Result<NativePage, AutomationError> {
        let events = super::events::EventHub::default();
        let network = Arc::new(super::networking::NetworkRuntime::new(
            self.resource_loader(),
            events.clone(),
            self.network_executor.handle(),
            self.websocket_connector(),
        ));
        let page = self.new_page_with_loader(
            PageOptions::builder()
                .clock(self.options.clock.clone())
                .diagnostics(self.options.diagnostics.clone())
                .events(events.clone())
                .network(network.clone())
                .request_headers(self.options.headers.clone())
                .plugins(self.options.plugins.clone())
                .plugin_timeout(self.options.timeout)
                .build(),
            network.clone(),
        )?;
        network.routes.set_page(page.id());
        Ok(NativePage {
            page,
            network,
            events,
            timeout: self.options.timeout,
        })
    }

    pub fn cookies(&self) -> SessionCookies {
        self.cookie_store.clone()
    }
}

#[derive(Clone, Debug)]
pub struct RequestOptions {
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
    pub allow_redirects: bool,
    pub max_redirects: usize,
    pub cancellation: CancellationToken,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            body: None,
            timeout: None,
            allow_redirects: true,
            max_redirects: 20,
            cancellation: CancellationToken::new(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub name: String,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub last_modified: u64,
}

#[derive(Clone)]
pub struct NativePage {
    page: PageHandle,
    network: Arc<super::networking::NetworkRuntime>,
    events: super::events::EventHub,
    timeout: Duration,
}

impl NativePage {
    pub fn advance_time(
        &self,
        milliseconds: f64,
        max_tasks: usize,
    ) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::AdvanceTime {
            milliseconds,
            max_tasks,
        })?
        else {
            unreachable!()
        };
        Ok(value)
    }
    pub fn run_pending_tasks(&self, max_tasks: usize) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::DrainTasks { max_tasks })?
        else {
            unreachable!()
        };
        Ok(value)
    }
    pub fn take_diagnostics(&self) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::Diagnostics)? else {
            unreachable!()
        };
        Ok(value)
    }

    pub fn stream(
        &self,
        method: &str,
        url: &str,
        options: RequestOptions,
    ) -> Result<super::streams::StreamingResponse, AutomationError> {
        let method = http::Method::from_bytes(method.as_bytes())
            .map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        let url = url::Url::parse(url).map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        if !["http", "https"].contains(&url.scheme()) {
            return Err(AutomationError::InvalidInput(
                "streams require HTTP(S)".into(),
            ));
        }
        let mut request = crate::network::ResourceRequest::new(method, url.to_string());
        request.headers = super::routes::parse_headers(options.headers.clone())?;
        request.body = options.body.clone();
        // Capture before obtaining the context: navigation at either boundary must
        // invalidate this stream, including a gap between redirect hops.
        let generation = self.network.stream_generation();
        let OperationResult::NetworkContext(context) = self.operation(Operation::NetworkContext)?
        else {
            unreachable!()
        };
        let timeout = options.timeout.unwrap_or(self.timeout);
        validate_timeout(timeout)?;
        super::streams::open(
            &self.network,
            context,
            self.id(),
            generation,
            request,
            options,
            timeout,
        )
    }
    pub fn on(
        &self,
        event: &str,
        handler: super::events::EventHandler,
    ) -> Result<super::events::Subscription, AutomationError> {
        self.page.ensure_access()?;
        let page_id = self.id();
        self.events.on(
            event,
            Arc::new(move |event| {
                let _guard = super::plugins::enter_page_callback(page_id);
                handler(event);
            }),
        )
    }
    pub fn route(
        &self,
        pattern: &str,
        stage: super::routes::RouteStage,
        handler: super::routes::RouteHandler,
    ) -> Result<super::routes::RouteRegistration, AutomationError> {
        self.page.ensure_access()?;
        self.network.routes.register(pattern, stage, handler)
    }
    pub fn configure_network(
        &self,
        options: super::networking::NetworkOptions,
    ) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        let offline = options.offline;
        self.network.configure(options)?;
        self.page.set_network_offline(offline)
    }
    pub fn set_identity(&self, identity: IdentityOptions) -> Result<(), AutomationError> {
        self.page.set_identity(identity)
    }
    pub fn set_extra_headers(&self, headers: Vec<(String, String)>) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        self.network.extra_headers(headers)
    }
    pub fn clear_cache(&self) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        self.network.clear_cache();
        Ok(())
    }
    pub fn response_body(&self, request_id: u64) -> Result<Vec<u8>, AutomationError> {
        self.page.ensure_access()?;
        self.network.body(request_id)
    }
    pub fn id(&self) -> u64 {
        self.page.id()
    }
    pub fn check_callback(&self) -> Result<(), AutomationError> {
        if super::plugins::is_callback_page(self.id()) {
            return Err(AutomationError::Reentry);
        }
        Ok(())
    }
    pub fn request(
        &self,
        method: &str,
        url: &str,
        options: RequestOptions,
    ) -> Result<NavigationResponse, AutomationError> {
        self.page.navigate_request(
            method,
            url,
            options.timeout.unwrap_or(self.timeout),
            options.cancellation,
            options.headers,
            options.body,
            options.allow_redirects,
            options.max_redirects,
        )
    }

    pub fn evaluate(&self, source: &str) -> Result<Value, AutomationError> {
        match self.operation(Operation::Evaluate {
            source: source.into(),
            by_value: true,
        })? {
            OperationResult::Value(value) => Ok(value),
            _ => unreachable!(),
        }
    }

    pub fn evaluate_handle(&self, source: &str) -> Result<JSHandle, AutomationError> {
        match self.operation(Operation::Evaluate {
            source: source.into(),
            by_value: false,
        })? {
            OperationResult::Handle(token) => Ok(self.handle(token)),
            _ => unreachable!(),
        }
    }

    pub fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>, AutomationError> {
        Ok(self.query(None, selector, false)?.into_iter().next())
    }

    pub fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        self.query(None, selector, true)
    }

    fn query(
        &self,
        root: Option<HandleToken>,
        selector: &str,
        all: bool,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        match self.operation(Operation::Query {
            root,
            selector: selector.into(),
            all,
        })? {
            OperationResult::Handles(tokens) => Ok(tokens
                .into_iter()
                .map(|token| ElementHandle(self.handle(token)))
                .collect()),
            _ => unreachable!(),
        }
    }

    fn handle(&self, token: HandleToken) -> JSHandle {
        JSHandle(Arc::new(Handle {
            page: self.clone(),
            token,
            disposed: AtomicBool::new(false),
        }))
    }

    fn operation(&self, operation: Operation) -> Result<OperationResult, AutomationError> {
        self.page.native_operation(operation, self.timeout)
    }

    pub fn add_init_script(&self, source: &str) -> Result<(), AutomationError> {
        self.page.add_preload_script(source, source)
    }
    fn input(&self, command: Value) -> Result<(), AutomationError> {
        self.operation(Operation::Input {
            target: None,
            command,
        })?;
        Ok(())
    }
    pub fn click(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"click","selector":selector}))
    }
    pub fn hover(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"hover","selector":selector}))
    }
    pub fn tap(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"tap","selector":selector}))
    }
    pub fn focus(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"focus","selector":selector}))
    }
    pub fn type_text(&self, selector: &str, text: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"type","selector":selector,"text":text}))
    }
    pub fn fill(&self, selector: &str, text: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"fill","selector":selector,"text":text}))
    }
    pub fn press(&self, selector: &str, key: &str, modifiers: u8) -> Result<(), AutomationError> {
        self.input(json!({"action":"press","selector":selector,"key":key,"modifiers":modifiers}))
    }
    pub fn drag_to(&self, source: &str, destination: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"dragTo","selector":source,"destination":destination}))
    }
    pub fn set_input_files(
        &self,
        selector: &str,
        files: Vec<FileUpload>,
    ) -> Result<(), AutomationError> {
        self.input(json!({"action":"files","selector":selector,"files":files}))
    }
    pub fn extract(
        &self,
        options: super::ExtractionOptions,
    ) -> Result<super::ExtractedDocument, AutomationError> {
        self.page.extract(options)
    }
    pub fn html(&self) -> Result<String, AutomationError> {
        self.page.html()
    }
    pub fn close(&self) -> Result<(), AutomationError> {
        self.check_callback()?;
        self.page.close();
        Ok(())
    }
    pub fn is_closed(&self) -> bool {
        self.page.is_closed()
    }
}

struct Handle {
    page: NativePage,
    token: HandleToken,
    disposed: AtomicBool,
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.disposed.swap(true, Ordering::AcqRel) {
            let _ = self.page.operation(Operation::Dispose(self.token));
        }
    }
}

/// A rooted value, including primitives. Clones share explicit disposal.
#[derive(Clone)]
pub struct JSHandle(Arc<Handle>);

impl JSHandle {
    fn token(&self) -> Result<HandleToken, AutomationError> {
        if self.0.disposed.load(Ordering::Acquire) {
            return Err(AutomationError::StaleHandle);
        }
        Ok(self.0.token)
    }

    pub fn json_value(&self) -> Result<Value, AutomationError> {
        match self.0.page.operation(Operation::Value(self.token()?))? {
            OperationResult::Value(value) => Ok(value),
            _ => unreachable!(),
        }
    }

    /// Invokes `function(value, ...arguments)` in the owning realm.
    pub fn call(&self, function: &str, arguments: Vec<Value>) -> Result<JSHandle, AutomationError> {
        match self.0.page.operation(Operation::Call {
            token: self.token()?,
            function: function.into(),
            arguments,
        })? {
            OperationResult::Handle(token) => Ok(self.0.page.handle(token)),
            _ => unreachable!(),
        }
    }

    pub fn property(&self, name: &str) -> Result<JSHandle, AutomationError> {
        self.call("(value, name) => value[name]", vec![json!(name)])
    }

    pub fn property_names(&self) -> Result<Vec<String>, AutomationError> {
        let value = self
            .call("value => Object.getOwnPropertyNames(value)", vec![])?
            .json_value()?;
        serde_json::from_value(value).map_err(|error| AutomationError::Internal(error.to_string()))
    }

    pub fn dispose(&self) -> Result<(), AutomationError> {
        if !self.0.disposed.swap(true, Ordering::AcqRel) {
            self.0.page.operation(Operation::Dispose(self.0.token))?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ElementHandle(pub JSHandle);

impl ElementHandle {
    fn input(&self, command: Value) -> Result<(), AutomationError> {
        self.0.0.page.operation(Operation::Input {
            target: Some(self.0.token()?),
            command,
        })?;
        Ok(())
    }
    pub fn focus(&self) -> Result<(), AutomationError> {
        self.input(json!({"action":"focus"}))
    }
    pub fn click(&self) -> Result<(), AutomationError> {
        self.input(json!({"action":"click"}))
    }
    pub fn set_input_files(&self, files: Vec<FileUpload>) -> Result<(), AutomationError> {
        self.input(json!({"action":"files","files":files}))
    }

    pub fn get_attribute(&self, name: &str) -> Result<Option<String>, AutomationError> {
        let value = self
            .0
            .call("(node, name) => node.getAttribute(name)", vec![json!(name)])?
            .json_value()?;
        serde_json::from_value(value).map_err(|error| AutomationError::Internal(error.to_string()))
    }
    pub fn set_attribute(&self, name: &str, value: &str) -> Result<(), AutomationError> {
        self.0.call(
            "(node, name, value) => { node.setAttribute(name, value); return null; }",
            vec![json!(name), json!(value)],
        )?;
        Ok(())
    }
    pub fn text_content(&self) -> Result<String, AutomationError> {
        self.string_property("textContent")
    }
    pub fn html(&self) -> Result<String, AutomationError> {
        self.string_property("outerHTML")
    }
    fn string_property(&self, name: &str) -> Result<String, AutomationError> {
        let value = self.0.property(name)?.json_value()?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AutomationError::Internal(format!("{name} is not a string")))
    }
    pub fn set_text_content(&self, text: &str) -> Result<(), AutomationError> {
        self.0.call(
            "(node, text) => { node.textContent = text; return null; }",
            vec![json!(text)],
        )?;
        Ok(())
    }
    pub fn remove(&self) -> Result<(), AutomationError> {
        self.0
            .call("node => { node.remove(); return null; }", vec![])?;
        Ok(())
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>, AutomationError> {
        Ok(self
            .0
            .0
            .page
            .query(Some(self.0.token()?), selector, false)?
            .into_iter()
            .next())
    }
    pub fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        self.0.0.page.query(Some(self.0.token()?), selector, true)
    }
}

fn validate_timeout(timeout: Duration) -> Result<(), AutomationError> {
    if timeout.is_zero() {
        return Err(AutomationError::InvalidInput(
            "timeout must be positive".into(),
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct HandleToken {
    realm: u64,
    id: u64,
}

pub(crate) enum Operation {
    AdvanceTime {
        milliseconds: f64,
        max_tasks: usize,
    },
    DrainTasks {
        max_tasks: usize,
    },
    Diagnostics,
    NetworkContext,
    Input {
        target: Option<HandleToken>,
        command: Value,
    },
    Evaluate {
        source: String,
        by_value: bool,
    },
    Call {
        token: HandleToken,
        function: String,
        arguments: Vec<Value>,
    },
    Value(HandleToken),
    Dispose(HandleToken),
    Query {
        root: Option<HandleToken>,
        selector: String,
        all: bool,
    },
}

pub(crate) enum OperationResult {
    NetworkContext(Arc<crate::web_apis::BrowsingContext>),
    Value(Value),
    Handle(HandleToken),
    Handles(Vec<HandleToken>),
}

/// Owner-thread GC roots. The table is cleared as soon as a realm is replaced.
#[derive(Default)]
pub(crate) struct NativeValues {
    realm: u64,
    next_id: u64,
    values: HashMap<HandleToken, ProtectedJsObject>,
}

impl NativeValues {
    pub(crate) fn synchronize(&mut self, page: &Page) {
        if self.realm != page.realm_id() {
            self.values.clear();
            self.realm = page.realm_id();
        }
    }
    fn insert(&mut self, value: ProtectedJsObject) -> HandleToken {
        self.next_id += 1;
        let token = HandleToken {
            realm: self.realm,
            id: self.next_id,
        };
        self.values.insert(token, value);
        token
    }
    fn get(&self, token: HandleToken) -> Result<&ProtectedJsObject, AutomationError> {
        self.values.get(&token).ok_or(AutomationError::StaleHandle)
    }
    pub(crate) fn execute(
        &mut self,
        page: &mut Page,
        operation: Operation,
        timeout: Duration,
        closed: &AtomicBool,
        interrupt: &AtomicU64,
        epoch: u64,
    ) -> Result<OperationResult, AutomationError> {
        if interrupt.load(Ordering::Acquire) != epoch {
            return Err(AutomationError::Cancellation);
        }
        self.synchronize(page);
        let result = match operation {
            Operation::Diagnostics => OperationResult::Value(page.take_diagnostics()),
            Operation::AdvanceTime {
                milliseconds,
                max_tasks,
            } => OperationResult::Value(page.advance_time(milliseconds, max_tasks)?),
            Operation::DrainTasks { max_tasks } => {
                OperationResult::Value(page.drain_tasks(max_tasks)?)
            }
            Operation::NetworkContext => OperationResult::NetworkContext(page.network_context()),
            Operation::Input { target, command } => {
                let command = command.to_string();
                match target {
                    Some(token) => {
                        let target = page
                            .call_native("(_, box) => box.value", "", self.get(token)?)?
                            .to_object()?;
                        page.dispatch_input_object(&command, &target)?;
                    }
                    None => {
                        page.dispatch_input(&command)?;
                    }
                }
                OperationResult::Value(Value::Null)
            }
            Operation::Dispose(token) => {
                self.values.remove(&token);
                OperationResult::Value(Value::Null)
            }
            Operation::Value(token) => OperationResult::Value(json_value(page, self.get(token)?)?),
            Operation::Evaluate { source, by_value } => {
                let source = serde_json::to_string(&source).expect("string serialization");
                let pending = page.eval(&format!(
                    "(() => {{ const box = {{ done: false }}; Promise.resolve((0, eval)({source})).then(value => {{ box.value = value; box.done = true; }}, error => {{ box.error = String(error); box.failed = true; box.done = true; }}); return box; }})()"
                ))?.to_object()?;
                wait(page, &pending, timeout, closed, interrupt, epoch)?;
                if by_value {
                    OperationResult::Value(json_value(page, &pending)?)
                } else {
                    OperationResult::Handle(self.insert(pending))
                }
            }
            Operation::Call {
                token,
                function,
                arguments,
            } => {
                let pending = page.call_native(
                    "(input, receiver) => { const { source, args } = JSON.parse(input); const box = { done: false }; Promise.resolve((0, eval)('(' + source + ')')(receiver.value, ...args)).then(value => { box.value = value; box.done = true; }, error => { box.error = String(error); box.failed = true; box.done = true; }); return box; }",
                    &json!({ "source": function, "args": arguments }).to_string(), self.get(token)?,
                )?.to_object()?;
                wait(page, &pending, timeout, closed, interrupt, epoch)?;
                OperationResult::Handle(self.insert(pending))
            }
            Operation::Query {
                root,
                selector,
                all,
            } => {
                let document;
                let root = match root {
                    Some(token) => self.get(token)?,
                    None => {
                        document = page.eval("({value: document})")?.to_object()?;
                        &document
                    }
                };
                let nodes = page.call_native(
                    "(input, root) => { const { selector, all } = JSON.parse(input); return { value: all ? Array.from(root.value.querySelectorAll(selector)) : [root.value.querySelector(selector)].filter(Boolean) }; }",
                    &json!({ "selector": selector, "all": all }).to_string(), root,
                )?.to_object()?;
                let len = page
                    .call_native("(_, nodes) => nodes.value.length", "", &nodes)?
                    .to_number()? as usize;
                let mut tokens = Vec::with_capacity(len);
                for index in 0..len {
                    let node = page
                        .call_native(
                            "(index, nodes) => ({value: nodes.value[Number(index)]})",
                            &index.to_string(),
                            &nodes,
                        )?
                        .to_object()?;
                    tokens.push(self.insert(node));
                }
                OperationResult::Handles(tokens)
            }
        };
        Ok(result)
    }
}

fn json_value(page: &Page, value: &ProtectedJsObject) -> Result<Value, AutomationError> {
    let json = page.call_native(
        "(_, box) => { const json = JSON.stringify(box.value); if (json === undefined) throw new TypeError('Value is not JSON serializable'); return json; }",
        "", value,
    )?.to_string()?;
    serde_json::from_str(&json).map_err(|error| AutomationError::Internal(error.to_string()))
}

fn wait(
    page: &mut Page,
    pending: &ProtectedJsObject,
    timeout: Duration,
    closed: &AtomicBool,
    interrupt: &AtomicU64,
    epoch: u64,
) -> Result<(), AutomationError> {
    let started = Instant::now();
    loop {
        if interrupt.load(Ordering::Acquire) != epoch {
            return Err(AutomationError::Cancellation);
        }
        if closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        let state = page.call_native(
            "(_, box) => JSON.stringify({done: box.done, failed: box.failed, error: box.error})", "", pending,
        )?.to_string()?;
        let state: Value = serde_json::from_str(&state)
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        if state["done"] == true {
            if state["failed"] == true {
                return Err(AutomationError::JavaScript(
                    state["error"].as_str().unwrap_or("Promise rejected").into(),
                ));
            }
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(AutomationError::Timeout(timeout));
        }
        if !page.run_one_task()? && !page.advance_to_next_timer() {
            page.wait_for_work(Some(timeout.saturating_sub(started.elapsed())));
        }
    }
}

impl From<brimp_jsc::JsException> for AutomationError {
    fn from(error: brimp_jsc::JsException) -> Self {
        Self::JavaScript(error.to_string())
    }
}

#[cfg(test)]
mod stream_lifecycle_tests {
    use super::*;

    #[test]
    fn navigation_or_close_between_context_capture_and_stream_start_cancels_it() {
        let session = Session::new(SessionOptions::default()).unwrap();
        let page = session.new_page().unwrap();
        let generation = page.network.stream_generation();
        let OperationResult::NetworkContext(context) =
            page.operation(Operation::NetworkContext).unwrap()
        else {
            unreachable!()
        };
        page.request("GET", "data:text/html,new", RequestOptions::default())
            .unwrap();
        let start = || {
            super::super::streams::open(
                &page.network,
                context.clone(),
                page.id(),
                generation,
                crate::network::ResourceRequest::get("https://unused.test/"),
                RequestOptions::default(),
                Duration::from_secs(1),
            )
        };
        assert!(matches!(start(), Err(AutomationError::Cancellation)));
        page.close().unwrap();
        assert!(matches!(start(), Err(AutomationError::Closed)));
    }
}
