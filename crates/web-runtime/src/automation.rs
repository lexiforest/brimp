use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use network::{HeaderList, ResourceLoader, ResourceRequest};
use serde::Serialize;
use serde_json::Value;
use web_bindings::{CookieJar, StoredCookie};

use crate::{
    ExtractedDocument, ExtractionOptions, NavigationError, NavigationResponse, Page, PageOptions,
    ScreenshotOptions, Viewport, page::DocumentNetworkScope, worker::WorkerCoordinator,
};

fn render_script(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;
    while let Some((offset, placeholder, value)) = replacements
        .iter()
        .filter_map(|(placeholder, value)| {
            remaining
                .find(placeholder)
                .map(|offset| (offset, *placeholder, *value))
        })
        .min_by_key(|(offset, _, _)| *offset)
    {
        rendered.push_str(&remaining[..offset]);
        rendered.push_str(value);
        remaining = &remaining[offset + placeholder.len()..];
    }
    rendered.push_str(remaining);
    rendered
}

#[derive(Debug, thiserror::Error)]
pub enum AutomationError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("transport failure: {0}")]
    Transport(String),
    #[error("HTTP response status {0}")]
    HttpStatus(u16),
    #[error("navigation failure: {0}")]
    Navigation(String),
    #[error("JavaScript exception: {0}")]
    JavaScript(String),
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    #[error("operation was cancelled")]
    Cancellation,
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    #[error("object is closed")]
    Closed,
    #[error("screenshot failure: {0}")]
    Screenshot(String),
    #[error("extraction failure: {0}")]
    Extraction(String),
    #[error("runtime failure: {0}")]
    Internal(String),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TouchPoint {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation_angle: f64,
    pub force: f64,
    pub tangential_pressure: f64,
}
impl AutomationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
            Self::Transport(_) => "transport",
            Self::HttpStatus(_) => "http_status",
            Self::Navigation(_) => "navigation",
            Self::JavaScript(_) => "javascript",
            Self::Timeout(_) => "timeout",
            Self::Cancellation => "cancelled",
            Self::Unsupported(_) => "unsupported",
            Self::Closed => "closed",
            Self::Screenshot(_) => "screenshot",
            Self::Extraction(_) => "extraction",
            Self::Internal(_) => "internal",
        }
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

pub struct AutomationBrowser {
    loader: Arc<dyn ResourceLoader>,
    persona: Option<persona::ResolvedPersona>,
    pages: Mutex<Vec<Weak<PageControl>>>,
    closed: AtomicBool,
    workers: WorkerCoordinator,
    default_context: AutomationBrowserContext,
    network_config: Option<network::CurlConfig>,
}

#[derive(Clone, Default)]
pub struct AutomationBrowserContext {
    cookies: Arc<CookieJar>,
}

impl AutomationBrowserContext {
    pub fn set_cookie(&self, url: &str, name: &str, value: &str) -> Result<(), AutomationError> {
        self.cookies
            .set(url, name, value)
            .map_err(AutomationError::InvalidInput)
    }

    pub fn store_cookie(&self, url: &str, header: &str) -> Result<(), AutomationError> {
        self.cookies
            .store(url, header)
            .map_err(AutomationError::InvalidInput)
    }

    pub fn cookies(&self) -> Vec<StoredCookie> {
        self.cookies.all()
    }

    pub fn cookies_for_url(&self, url: &str) -> Vec<StoredCookie> {
        self.cookies.matching(url)
    }

    pub fn delete_cookies(
        &self,
        name: &str,
        url: Option<&str>,
        domain: Option<&str>,
        path: Option<&str>,
    ) {
        self.cookies.delete(name, url, domain, path);
    }

    pub fn clear_cookies(&self) {
        self.cookies.clear();
    }
}
impl AutomationBrowser {
    pub fn new() -> Result<Self, AutomationError> {
        let persona = persona::PersonaConfig::default();
        Self::with_persona(persona)
    }
    pub fn with_resource_loader(loader: Arc<dyn ResourceLoader>) -> Self {
        Self {
            loader,
            persona: None,
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().expect("shared worker coordinator must start"),
            default_context: AutomationBrowserContext::default(),
            network_config: None,
        }
    }
    pub fn with_persona_and_resource_loader(
        persona: persona::PersonaConfig,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, AutomationError> {
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        Ok(Self {
            loader,
            persona: Some(persona.resolve()),
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().map_err(AutomationError::Internal)?,
            default_context: AutomationBrowserContext::default(),
            network_config: None,
        })
    }
    pub fn with_persona(persona: persona::PersonaConfig) -> Result<Self, AutomationError> {
        Self::with_persona_and_network_config(persona, network::CurlConfig::default())
    }
    pub fn with_persona_and_network_config(
        persona: persona::PersonaConfig,
        mut config: network::CurlConfig,
    ) -> Result<Self, AutomationError> {
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        let persona = persona.resolve();
        config.impersonation_profile = persona.transport_profile.clone();
        config.default_headers = false;
        network::CurlResourceLoader::check_profile(&config)
            .map_err(|error| AutomationError::Transport(error.to_string()))?;
        let loader = Arc::new(
            network::CurlResourceLoader::new(config.clone())
                .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        Ok(Self {
            loader,
            persona: Some(persona),
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().map_err(AutomationError::Internal)?,
            default_context: AutomationBrowserContext::default(),
            network_config: Some(config),
        })
    }
    pub fn new_page(&self, options: PageOptions) -> Result<AutomationPage, AutomationError> {
        self.new_page_in_context(options, &self.default_context)
    }
    pub fn default_context(&self) -> AutomationBrowserContext {
        self.default_context.clone()
    }
    pub fn create_context(&self) -> AutomationBrowserContext {
        AutomationBrowserContext::default()
    }
    pub fn new_page_in_context(
        &self,
        options: PageOptions,
        context: &AutomationBrowserContext,
    ) -> Result<AutomationPage, AutomationError> {
        self.new_page_with_loader(options, Arc::clone(&self.loader), context)
    }
    pub fn new_page_in_context_with_proxy(
        &self,
        options: PageOptions,
        context: &AutomationBrowserContext,
        proxy: Option<&str>,
    ) -> Result<AutomationPage, AutomationError> {
        let Some(proxy) = proxy else {
            return self.new_page_in_context(options, context);
        };
        let mut config = self.network_config.clone().ok_or_else(|| {
            AutomationError::Unsupported(
                "proxy pages require a browser created with CurlConfig".into(),
            )
        })?;
        config.proxy = Some(
            network::Proxy::parse(proxy)
                .map_err(|error| AutomationError::InvalidInput(error.to_string()))?,
        );
        let loader = Arc::new(
            network::CurlResourceLoader::new(config)
                .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        self.new_page_with_loader(options, loader, context)
    }
    pub fn new_page_with_request_interceptor(
        &self,
        options: PageOptions,
        interceptor: Arc<dyn network::ResourceInterceptor>,
    ) -> Result<AutomationPage, AutomationError> {
        let loader = Arc::new(network::InterceptingResourceLoader::new(
            Arc::clone(&self.loader),
            interceptor,
        ));
        self.new_page_with_loader(options, loader, &self.default_context)
    }
    pub fn new_page_in_context_with_request_interceptor(
        &self,
        options: PageOptions,
        context: &AutomationBrowserContext,
        interceptor: Arc<dyn network::ResourceInterceptor>,
    ) -> Result<AutomationPage, AutomationError> {
        let loader = Arc::new(network::InterceptingResourceLoader::new(
            Arc::clone(&self.loader),
            interceptor,
        ));
        self.new_page_with_loader(options, loader, context)
    }
    fn new_page_with_loader(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
        context: &AutomationBrowserContext,
    ) -> Result<AutomationPage, AutomationError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        let options = self
            .persona
            .clone()
            .map_or(options.clone(), |persona| options.with_persona(persona));
        let page = AutomationPage::launch(
            options,
            DocumentNetworkScope::new(loader),
            self.workers.clone(),
            Arc::clone(&context.cookies),
        )?;
        self.pages
            .lock()
            .expect("automation page list poisoned")
            .push(Arc::downgrade(&page.control));
        Ok(page)
    }
    pub fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let pages = std::mem::take(&mut *self.pages.lock().expect("automation page list poisoned"));
        for page in pages.into_iter().filter_map(|page| page.upgrade()) {
            page.close();
        }
    }
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}
impl Drop for AutomationBrowser {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
pub struct AutomationPage {
    control: Arc<PageControl>,
}
impl AutomationPage {
    fn launch(
        options: PageOptions,
        network_scope: DocumentNetworkScope,
        workers: WorkerCoordinator,
        cookies: Arc<CookieJar>,
    ) -> Result<Self, AutomationError> {
        let (commands, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("brimp-page-owner".into())
            .spawn(move || {
                run_page(
                    options,
                    network_scope,
                    workers,
                    cookies,
                    receiver,
                    ready_sender,
                )
            })
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        ready_receiver
            .recv()
            .map_err(|_| AutomationError::Internal("page owner exited during startup".into()))??;
        Ok(Self {
            control: Arc::new(PageControl {
                commands: Mutex::new(Some(commands)),
                worker: Mutex::new(Some(worker)),
                closed: AtomicBool::new(false),
            }),
        })
    }
    pub fn navigate(
        &self,
        url: impl Into<String>,
        timeout: Duration,
    ) -> Result<NavigationResponse, AutomationError> {
        self.navigate_cancellable(url, timeout, CancellationToken::new())
    }
    pub fn navigate_cancellable(
        &self,
        url: impl Into<String>,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<NavigationResponse, AutomationError> {
        self.navigate_with_headers(url, timeout, cancellation, Vec::new())
    }
    pub fn navigate_with_headers(
        &self,
        url: impl Into<String>,
        timeout: Duration,
        cancellation: CancellationToken,
        headers: Vec<(String, String)>,
    ) -> Result<NavigationResponse, AutomationError> {
        self.navigate_request("GET", url, timeout, cancellation, headers, None, true, 20)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn navigate_request(
        &self,
        method: impl AsRef<str>,
        url: impl Into<String>,
        timeout: Duration,
        cancellation: CancellationToken,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
        allow_redirects: bool,
        max_redirects: usize,
    ) -> Result<NavigationResponse, AutomationError> {
        if timeout.is_zero() {
            return Err(AutomationError::InvalidInput(
                "timeout must be positive".into(),
            ));
        }
        let method = http::Method::from_bytes(method.as_ref().as_bytes()).map_err(|error| {
            AutomationError::InvalidInput(format!("invalid HTTP method: {error}"))
        })?;
        let mut request_headers = HeaderList::new();
        for (name, value) in headers {
            let name = name.parse::<http::HeaderName>().map_err(|error| {
                AutomationError::InvalidInput(format!("invalid header name: {error}"))
            })?;
            let value = value.parse::<http::HeaderValue>().map_err(|error| {
                AutomationError::InvalidInput(format!("invalid header value: {error}"))
            })?;
            request_headers.append(name, value);
        }
        self.request(|response| Command::Navigate {
            method,
            url: url.into(),
            timeout,
            cancellation,
            headers: request_headers,
            body,
            allow_redirects,
            max_redirects,
            response,
        })?
    }
    pub fn evaluate(&self, expression: impl Into<String>) -> Result<Value, AutomationError> {
        self.request(|response| Command::Evaluate {
            expression: expression.into(),
            response,
        })?
    }
    pub fn evaluate_remote(
        &self,
        expression: impl Into<String>,
        return_by_value: bool,
        object_group: Option<String>,
        await_promise: bool,
    ) -> Result<Value, AutomationError> {
        self.request(|response| Command::EvaluateRemote {
            expression: expression.into(),
            return_by_value,
            object_group,
            await_promise,
            response,
        })?
    }
    pub fn call_function_remote(
        &self,
        declaration: impl Into<String>,
        receiver: Option<String>,
        arguments: Vec<RemoteArgument>,
        return_by_value: bool,
        object_group: Option<String>,
        await_promise: bool,
    ) -> Result<Value, AutomationError> {
        self.request(|response| Command::CallFunctionRemote {
            declaration: declaration.into(),
            receiver,
            arguments,
            return_by_value,
            object_group,
            await_promise,
            response,
        })?
    }
    pub fn remote_object_properties(
        &self,
        object_id: impl Into<String>,
        own_properties: bool,
        accessor_properties_only: bool,
    ) -> Result<Value, AutomationError> {
        self.request(|response| Command::RemoteObjectProperties {
            object_id: object_id.into(),
            own_properties,
            accessor_properties_only,
            response,
        })?
    }
    pub fn describe_remote_node(
        &self,
        object_id: impl Into<String>,
    ) -> Result<Value, AutomationError> {
        self.request(|response| Command::DescribeRemoteNode {
            object_id: object_id.into(),
            response,
        })?
    }
    pub fn resolve_remote_node(
        &self,
        backend_node_id: u64,
        object_group: Option<String>,
    ) -> Result<Value, AutomationError> {
        self.request(|response| Command::ResolveRemoteNode {
            backend_node_id,
            object_group,
            response,
        })?
    }
    pub fn release_remote_object(
        &self,
        object_id: impl Into<String>,
    ) -> Result<bool, AutomationError> {
        self.request(|response| Command::ReleaseRemoteObject {
            object_id: object_id.into(),
            response,
        })?
    }
    pub fn release_remote_object_group(
        &self,
        object_group: impl Into<String>,
    ) -> Result<usize, AutomationError> {
        self.request(|response| Command::ReleaseRemoteObjectGroup {
            object_group: object_group.into(),
            response,
        })?
    }
    pub fn title(&self) -> Result<String, AutomationError> {
        self.evaluate("document.title")?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AutomationError::Internal("document title was not a string".into()))
    }
    pub fn text_content(&self) -> Result<String, AutomationError> {
        self.request(|response| Command::Text { response })?
    }
    pub fn html(&self) -> Result<String, AutomationError> {
        self.request(|response| Command::Html { response })?
    }
    pub fn viewport(&self) -> Result<Viewport, AutomationError> {
        self.request(|response| Command::Viewport { response })?
    }
    pub fn set_viewport(
        &self,
        width: u32,
        height: u32,
        device_pixel_ratio: f64,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::SetViewport {
            width,
            height,
            device_pixel_ratio,
            response,
        })?
    }
    pub fn set_navigator_identity_override(
        &self,
        identity_override: Value,
    ) -> Result<(), AutomationError> {
        let identity_override = serde_json::to_string(&identity_override)
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        self.request(|response| Command::SetNavigatorIdentityOverride {
            identity_override,
            response,
        })?
    }
    pub fn add_preload_script(
        &self,
        identifier: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::AddPreloadScript {
            identifier: identifier.into(),
            source: source.into(),
            response,
        })?
    }
    pub fn remove_preload_script(
        &self,
        identifier: impl Into<String>,
    ) -> Result<bool, AutomationError> {
        self.request(|response| Command::RemovePreloadScript {
            identifier: identifier.into(),
            response,
        })?
    }
    pub fn screenshot(&self, full_page: bool) -> Result<Vec<u8>, AutomationError> {
        self.request(|response| Command::Screenshot {
            full_page,
            response,
        })?
    }
    pub fn extract(
        &self,
        options: ExtractionOptions,
    ) -> Result<ExtractedDocument, AutomationError> {
        self.request(|response| Command::Extract { options, response })?
    }
    pub fn wait_for_selector(
        &self,
        selector: impl Into<String>,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::WaitForSelector {
            selector: selector.into(),
            timeout,
            cancellation,
            response,
        })?
    }
    pub fn wait_for_network_idle(
        &self,
        quiet_window: Duration,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::WaitForNetworkIdle {
            quiet_window,
            timeout,
            cancellation,
            response,
        })?
    }
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_mouse_event(
        &self,
        event_type: impl Into<String>,
        x: f64,
        y: f64,
        button: u8,
        buttons: u64,
        click_count: u64,
        modifiers: u64,
    ) -> Result<(), AutomationError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AutomationError::InvalidInput(
                "mouse coordinates must be finite".into(),
            ));
        }
        self.input(serde_json::json!({
            "action": "mouse",
            "eventType": event_type.into(),
            "x": x,
            "y": y,
            "button": button,
            "buttons": buttons,
            "clickCount": click_count,
            "modifiers": modifiers,
        }))
    }
    pub fn dispatch_key_event(
        &self,
        event_type: impl Into<String>,
        key: impl Into<String>,
        code: impl Into<String>,
        text: impl Into<String>,
        auto_repeat: bool,
        modifiers: u64,
    ) -> Result<(), AutomationError> {
        self.input(serde_json::json!({
            "action": "key",
            "eventType": event_type.into(),
            "key": key.into(),
            "code": code.into(),
            "text": text.into(),
            "autoRepeat": auto_repeat,
            "modifiers": modifiers,
        }))
    }
    pub fn insert_text(&self, text: impl Into<String>) -> Result<(), AutomationError> {
        self.input(serde_json::json!({"action": "insertText", "text": text.into()}))
    }
    pub fn dispatch_touch_event(
        &self,
        event_type: impl Into<String>,
        touch_points: Vec<TouchPoint>,
        modifiers: u64,
    ) -> Result<(), AutomationError> {
        if touch_points.iter().any(|point| {
            !point.x.is_finite()
                || !point.y.is_finite()
                || !point.radius_x.is_finite()
                || !point.radius_y.is_finite()
                || !point.rotation_angle.is_finite()
                || !point.force.is_finite()
                || !point.tangential_pressure.is_finite()
                || point.radius_x < 0.0
                || point.radius_y < 0.0
                || !(0.0..=1.0).contains(&point.force)
                || !(-1.0..=1.0).contains(&point.tangential_pressure)
        }) {
            return Err(AutomationError::InvalidInput(
                "touch point values are outside their supported ranges".into(),
            ));
        }
        self.input(serde_json::json!({
            "action": "touch",
            "eventType": event_type.into(),
            "touchPoints": touch_points,
            "modifiers": modifiers,
        }))
    }
    pub fn click(&self, selector: impl Into<String>) -> Result<(), AutomationError> {
        self.input(serde_json::json!({"action": "click", "selector": selector.into()}))
    }
    pub fn hover(&self, selector: impl Into<String>) -> Result<(), AutomationError> {
        self.input(serde_json::json!({"action": "hover", "selector": selector.into()}))
    }
    pub fn type_text(
        &self,
        selector: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<(), AutomationError> {
        self.input(serde_json::json!({
            "action": "type",
            "selector": selector.into(),
            "text": text.into(),
        }))
    }
    pub fn tap(&self, selector: impl Into<String>) -> Result<(), AutomationError> {
        self.input(serde_json::json!({"action": "tap", "selector": selector.into()}))
    }
    pub fn focus_remote_node(
        &self,
        target_expression: impl Into<String>,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::FocusRemoteNode {
            target_expression: target_expression.into(),
            response,
        })?
    }
    fn input(&self, command: Value) -> Result<(), AutomationError> {
        let command = serde_json::to_string(&command)
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        self.request(|response| Command::Input { command, response })?
    }
    pub fn close(&self) {
        self.control.close();
    }
    pub fn reset(&self) -> Result<(), AutomationError> {
        self.request(Command::Reset)?
    }
    pub fn is_closed(&self) -> bool {
        self.control.closed.load(Ordering::Acquire)
    }
    fn request<T>(
        &self,
        command: impl FnOnce(mpsc::SyncSender<T>) -> Command,
    ) -> Result<T, AutomationError> {
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let commands = self
            .control
            .commands
            .lock()
            .expect("automation command sender poisoned");
        commands
            .as_ref()
            .ok_or(AutomationError::Closed)?
            .send(command(sender))
            .map_err(|_| AutomationError::Closed)?;
        receiver.recv().map_err(|_| AutomationError::Closed)
    }
}
impl Drop for AutomationPage {
    fn drop(&mut self) {
        if Arc::strong_count(&self.control) == 1 {
            self.close();
        }
    }
}

struct PageControl {
    commands: Mutex<Option<mpsc::Sender<Command>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
    closed: AtomicBool,
}
impl PageControl {
    fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Some(sender) = self
            .commands
            .lock()
            .expect("automation command sender poisoned")
            .take()
        {
            let (done, receiver) = mpsc::sync_channel(1);
            let _ = sender.send(Command::Close(done));
            let _ = receiver.recv();
        }
        if let Some(worker) = self
            .worker
            .lock()
            .expect("automation worker lock poisoned")
            .take()
        {
            let _ = worker.join();
        }
    }
}
impl Drop for PageControl {
    fn drop(&mut self) {
        self.close();
    }
}

enum Command {
    Reset(mpsc::SyncSender<Result<(), AutomationError>>),
    Navigate {
        method: http::Method,
        url: String,
        timeout: Duration,
        cancellation: CancellationToken,
        headers: HeaderList,
        body: Option<Vec<u8>>,
        allow_redirects: bool,
        max_redirects: usize,
        response: mpsc::SyncSender<Result<NavigationResponse, AutomationError>>,
    },
    Evaluate {
        expression: String,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    EvaluateRemote {
        expression: String,
        return_by_value: bool,
        object_group: Option<String>,
        await_promise: bool,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    CallFunctionRemote {
        declaration: String,
        receiver: Option<String>,
        arguments: Vec<RemoteArgument>,
        return_by_value: bool,
        object_group: Option<String>,
        await_promise: bool,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    RemoteObjectProperties {
        object_id: String,
        own_properties: bool,
        accessor_properties_only: bool,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    DescribeRemoteNode {
        object_id: String,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    ResolveRemoteNode {
        backend_node_id: u64,
        object_group: Option<String>,
        response: mpsc::SyncSender<Result<Value, AutomationError>>,
    },
    ReleaseRemoteObject {
        object_id: String,
        response: mpsc::SyncSender<Result<bool, AutomationError>>,
    },
    ReleaseRemoteObjectGroup {
        object_group: String,
        response: mpsc::SyncSender<Result<usize, AutomationError>>,
    },
    Text {
        response: mpsc::SyncSender<Result<String, AutomationError>>,
    },
    Html {
        response: mpsc::SyncSender<Result<String, AutomationError>>,
    },
    Viewport {
        response: mpsc::SyncSender<Result<Viewport, AutomationError>>,
    },
    SetViewport {
        width: u32,
        height: u32,
        device_pixel_ratio: f64,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    SetNavigatorIdentityOverride {
        identity_override: String,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    AddPreloadScript {
        identifier: String,
        source: String,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    RemovePreloadScript {
        identifier: String,
        response: mpsc::SyncSender<Result<bool, AutomationError>>,
    },
    Screenshot {
        full_page: bool,
        response: mpsc::SyncSender<Result<Vec<u8>, AutomationError>>,
    },
    Extract {
        options: ExtractionOptions,
        response: mpsc::SyncSender<Result<ExtractedDocument, AutomationError>>,
    },
    WaitForSelector {
        selector: String,
        timeout: Duration,
        cancellation: CancellationToken,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    WaitForNetworkIdle {
        quiet_window: Duration,
        timeout: Duration,
        cancellation: CancellationToken,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    Input {
        command: String,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    FocusRemoteNode {
        target_expression: String,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    Close(mpsc::SyncSender<()>),
}

fn run_page(
    options: PageOptions,
    network_scope: DocumentNetworkScope,
    workers: WorkerCoordinator,
    cookies: Arc<CookieJar>,
    commands: mpsc::Receiver<Command>,
    ready: mpsc::SyncSender<Result<(), AutomationError>>,
) {
    let mut page = match Page::new(options, network_scope, workers, cookies) {
        Ok(page) => page,
        Err(error) => {
            let _ = ready.send(Err(AutomationError::Internal(error.to_string())));
            return;
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready.send(Err(AutomationError::Internal(error.to_string())));
            return;
        }
    };
    let _ = ready.send(Ok(()));
    loop {
        let command = match commands.recv_timeout(Duration::from_millis(1)) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = page.run_pending_tasks();
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        match command {
            Command::Reset(response) => {
                let result = page
                    .reset()
                    .map_err(|error| AutomationError::JavaScript(error.to_string()));
                let _ = response.send(result);
            }
            Command::Navigate {
                method,
                url,
                timeout,
                cancellation,
                headers,
                body,
                allow_redirects,
                max_redirects,
                response,
            } => {
                let mut request = ResourceRequest::new(method, url);
                request.headers = headers;
                request.body = body;
                let result = runtime.block_on(async {
                    tokio::select! {
                        result = page.goto_request(request, allow_redirects, max_redirects) => result.map_err(AutomationError::from),
                        () = wait_for_cancellation(cancellation) => Err(AutomationError::Cancellation),
                        () = tokio::time::sleep(timeout) => Err(AutomationError::Timeout(timeout)),
                    }
                });
                let _ = response.send(result);
            }
            Command::Evaluate {
                expression,
                response,
            } => {
                let _ = response.send(evaluate(&page, &expression));
            }
            Command::EvaluateRemote {
                expression,
                return_by_value,
                object_group,
                await_promise,
                response,
            } => {
                let _ = response.send(evaluate_remote(
                    &mut page,
                    &expression,
                    return_by_value,
                    object_group.as_deref(),
                    await_promise,
                ));
            }
            Command::CallFunctionRemote {
                declaration,
                receiver,
                arguments,
                return_by_value,
                object_group,
                await_promise,
                response,
            } => {
                let _ = response.send(call_function_remote(
                    &mut page,
                    &declaration,
                    receiver.as_deref(),
                    &arguments,
                    return_by_value,
                    object_group.as_deref(),
                    await_promise,
                ));
            }
            Command::RemoteObjectProperties {
                object_id,
                own_properties,
                accessor_properties_only,
                response,
            } => {
                let _ = response.send(remote_object_properties(
                    &page,
                    &object_id,
                    own_properties,
                    accessor_properties_only,
                ));
            }
            Command::DescribeRemoteNode {
                object_id,
                response,
            } => {
                let _ = response.send(describe_remote_node(&page, &object_id));
            }
            Command::ResolveRemoteNode {
                backend_node_id,
                object_group,
                response,
            } => {
                let _ = response.send(resolve_remote_node(
                    &page,
                    backend_node_id,
                    object_group.as_deref(),
                ));
            }
            Command::ReleaseRemoteObject {
                object_id,
                response,
            } => {
                let _ = response.send(release_remote_object(&page, &object_id));
            }
            Command::ReleaseRemoteObjectGroup {
                object_group,
                response,
            } => {
                let _ = response.send(release_remote_object_group(&page, &object_group));
            }
            Command::Text { response } => {
                let result =
                    evaluate(&page, "document.documentElement.textContent").and_then(|value| {
                        value.as_str().map(str::to_owned).ok_or_else(|| {
                            AutomationError::Internal("document text was not a string".into())
                        })
                    });
                let _ = response.send(result);
            }
            Command::Html { response } => {
                let _ = response.send(Ok(page.html()));
            }
            Command::Viewport { response } => {
                let _ = response.send(Ok(page.viewport()));
            }
            Command::SetViewport {
                width,
                height,
                device_pixel_ratio,
                response,
            } => {
                page.set_viewport(width, height, device_pixel_ratio);
                let _ = response.send(Ok(()));
            }
            Command::SetNavigatorIdentityOverride {
                identity_override,
                response,
            } => {
                let result = page
                    .set_navigator_identity_override(identity_override)
                    .map_err(|error| AutomationError::JavaScript(error.to_string()));
                let _ = response.send(result);
            }
            Command::AddPreloadScript {
                identifier,
                source,
                response,
            } => {
                page.add_preload_script(identifier, source);
                let _ = response.send(Ok(()));
            }
            Command::RemovePreloadScript {
                identifier,
                response,
            } => {
                let removed = page.remove_preload_script(&identifier);
                let _ = response.send(Ok(removed));
            }
            Command::Screenshot {
                full_page,
                response,
            } => {
                let viewport = page.viewport();
                let mut options =
                    ScreenshotOptions::new(viewport.width as u32, viewport.height as u32);
                options.full_page = full_page;
                let result = page
                    .screenshot_png(options)
                    .map_err(|error| AutomationError::Screenshot(error.to_string()));
                let _ = response.send(result);
            }
            Command::Extract { options, response } => {
                let result = page
                    .extract(options)
                    .map_err(|error| AutomationError::Extraction(error.to_string()));
                let _ = response.send(result);
            }
            Command::WaitForSelector {
                selector,
                timeout,
                cancellation,
                response,
            } => {
                let started = Instant::now();
                let encoded = serde_json::to_string(&selector)
                    .map_err(|error| AutomationError::InvalidInput(error.to_string()));
                let result = encoded.and_then(|encoded| {
                    loop {
                        if cancellation.is_cancelled() {
                            break Err(AutomationError::Cancellation);
                        }
                        if started.elapsed() >= timeout {
                            break Err(AutomationError::Timeout(timeout));
                        }
                        if evaluate(
                            &page,
                            &format!("document.querySelector({encoded}) !== null"),
                        )? == Value::Bool(true)
                        {
                            break Ok(());
                        }
                        page.run_pending_tasks()
                            .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
                        std::thread::sleep(Duration::from_millis(5));
                    }
                });
                let _ = response.send(result);
            }
            Command::WaitForNetworkIdle {
                quiet_window,
                timeout,
                cancellation,
                response,
            } => {
                let started = Instant::now();
                let mut idle_since = None;
                let result = loop {
                    if cancellation.is_cancelled() {
                        break Err(AutomationError::Cancellation);
                    }
                    if started.elapsed() >= timeout {
                        break Err(AutomationError::Timeout(timeout));
                    }
                    if let Err(error) = page.run_pending_tasks() {
                        break Err(AutomationError::JavaScript(error.to_string()));
                    }
                    if page.is_network_idle() {
                        let since = idle_since.get_or_insert_with(Instant::now);
                        if since.elapsed() >= quiet_window {
                            break Ok(());
                        }
                    } else {
                        idle_since = None;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                };
                let _ = response.send(result);
            }
            Command::Input { command, response } => {
                let result = page.dispatch_input(&command).map(|_| ()).map_err(|error| {
                    let message = error.to_string();
                    if let Some((_, selector)) = message.split_once("BRIMP_INPUT_NOT_FOUND:") {
                        AutomationError::InvalidInput(format!(
                            "selector did not match an element: {}",
                            selector.trim_end_matches(']')
                        ))
                    } else if message.contains("BRIMP_INPUT_NO_FOCUS") {
                        AutomationError::InvalidInput(
                            "text input requires a focused editable control".into(),
                        )
                    } else {
                        AutomationError::JavaScript(message)
                    }
                });
                let _ = response.send(result);
            }
            Command::FocusRemoteNode {
                target_expression,
                response,
            } => {
                let result = page
                    .dispatch_input_on(r#"{"action":"focusTarget"}"#, &target_expression)
                    .map(|_| ())
                    .map_err(|error| AutomationError::JavaScript(error.to_string()));
                let _ = response.send(result);
            }
            Command::Close(done) => {
                drop(page);
                let _ = done.send(());
                break;
            }
        }
    }
}

async fn wait_for_cancellation(cancellation: CancellationToken) {
    while !cancellation.is_cancelled() {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn evaluate(page: &Page, expression: &str) -> Result<Value, AutomationError> {
    let encoded = serde_json::to_string(expression)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/evaluate_json.js"),
        &[("__EXPRESSION__", &encoded)],
    );
    let json = page
        .eval(&source)
        .map_err(|error| {
            let message = error.to_string();
            message.split_once("BRIMP_UNSUPPORTED_RESULT:").map_or_else(
                || AutomationError::JavaScript(message.clone()),
                |(_, detail)| {
                    AutomationError::Unsupported(detail.trim_end_matches(']').to_string())
                },
            )
        })?
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    serde_json::from_str(&json).map_err(|error| {
        AutomationError::Internal(format!("invalid structured JavaScript result: {error}"))
    })
}

#[derive(Clone, Debug)]
pub enum RemoteArgument {
    Value(Value),
    ObjectId(String),
    UnserializableValue(String),
}

fn evaluate_remote(
    page: &mut Page,
    expression: &str,
    return_by_value: bool,
    object_group: Option<&str>,
    await_promise: bool,
) -> Result<Value, AutomationError> {
    let expression = serde_json::to_string(expression)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let body = render_script(
        include_str!("automation/evaluate_expression.js"),
        &[("__EXPRESSION__", &expression)],
    );
    evaluate_remote_maybe_await(page, &body, return_by_value, object_group, await_promise)
}

fn call_function_remote(
    page: &mut Page,
    declaration: &str,
    receiver: Option<&str>,
    arguments: &[RemoteArgument],
    return_by_value: bool,
    object_group: Option<&str>,
    await_promise: bool,
) -> Result<Value, AutomationError> {
    let declaration = serde_json::to_string(declaration)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let receiver = serde_json::to_string(&receiver)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let arguments = arguments
        .iter()
        .map(|argument| match argument {
            RemoteArgument::Value(value) => serde_json::json!({"value": value}),
            RemoteArgument::ObjectId(object_id) => serde_json::json!({"objectId": object_id}),
            RemoteArgument::UnserializableValue(value) => {
                serde_json::json!({"unserializableValue": value})
            }
        })
        .collect::<Vec<_>>();
    let arguments = serde_json::to_string(&arguments)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let body = render_script(
        include_str!("automation/call_function.js"),
        &[
            ("__DECLARATION__", &declaration),
            ("__RECEIVER__", &receiver),
            ("__ARGUMENTS__", &arguments),
        ],
    );
    evaluate_remote_maybe_await(page, &body, return_by_value, object_group, await_promise)
}

fn evaluate_remote_maybe_await(
    page: &mut Page,
    body: &str,
    return_by_value: bool,
    object_group: Option<&str>,
    await_promise: bool,
) -> Result<Value, AutomationError> {
    if !await_promise {
        return evaluate_remote_source(page, body, return_by_value, object_group);
    }
    let setup = render_script(
        include_str!("automation/await_promise.js"),
        &[("__BODY__", body)],
    );
    page.eval(&setup)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    let timeout = Duration::from_secs(30);
    let started = Instant::now();
    loop {
        let settled = page
            .eval(include_str!("automation/promise_settled.js"))
            .map_err(|error| AutomationError::JavaScript(error.to_string()))?
            .to_string()
            .map_err(|error| AutomationError::JavaScript(error.to_string()))?
            == "true";
        if settled {
            break;
        }
        if started.elapsed() >= timeout {
            return Err(AutomationError::Timeout(timeout));
        }
        page.run_pending_tasks()
            .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
        std::thread::sleep(Duration::from_millis(1));
    }
    evaluate_remote_source(
        page,
        include_str!("automation/promise_result.js"),
        return_by_value,
        object_group,
    )
}

fn evaluate_remote_source(
    page: &Page,
    body: &str,
    return_by_value: bool,
    object_group: Option<&str>,
) -> Result<Value, AutomationError> {
    let object_group = serde_json::to_string(&object_group)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/evaluate_remote.js"),
        &[
            ("__BODY__", body),
            ("__OBJECT_GROUP__", &object_group),
            (
                "__RETURN_BY_VALUE__",
                if return_by_value { "true" } else { "false" },
            ),
        ],
    );
    let json = page
        .eval(&source)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    serde_json::from_str(&json)
        .map_err(|error| AutomationError::Internal(format!("invalid remote object: {error}")))
}

fn remote_object_properties(
    page: &Page,
    object_id: &str,
    own_properties: bool,
    accessor_properties_only: bool,
) -> Result<Value, AutomationError> {
    let object_id = serde_json::to_string(object_id)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/remote_object_properties.js"),
        &[
            ("__OBJECT_ID__", &object_id),
            (
                "__OWN_PROPERTIES__",
                if own_properties { "true" } else { "false" },
            ),
            (
                "__ACCESSOR_PROPERTIES_ONLY__",
                if accessor_properties_only {
                    "true"
                } else {
                    "false"
                },
            ),
        ],
    );
    let json = page
        .eval(&source)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    serde_json::from_str(&json).map_err(|error| {
        AutomationError::Internal(format!("invalid property descriptors: {error}"))
    })
}

fn describe_remote_node(page: &Page, object_id: &str) -> Result<Value, AutomationError> {
    let object_id = serde_json::to_string(object_id)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/describe_remote_node.js"),
        &[("__OBJECT_ID__", &object_id)],
    );
    let json = page
        .eval(&source)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    serde_json::from_str(&json)
        .map_err(|error| AutomationError::Internal(format!("invalid DOM node: {error}")))
}

fn resolve_remote_node(
    page: &Page,
    backend_node_id: u64,
    object_group: Option<&str>,
) -> Result<Value, AutomationError> {
    let backend_node_id = backend_node_id.to_string();
    let body = render_script(
        include_str!("automation/resolve_remote_node.js"),
        &[("__BACKEND_NODE_ID__", &backend_node_id)],
    );
    evaluate_remote_source(page, &body, false, object_group)
}

fn release_remote_object(page: &Page, object_id: &str) -> Result<bool, AutomationError> {
    let object_id = serde_json::to_string(object_id)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/release_remote_object.js"),
        &[("__OBJECT_ID__", &object_id)],
    );
    let value = page
        .eval(&source)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    Ok(value
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?
        == "true")
}

fn release_remote_object_group(page: &Page, object_group: &str) -> Result<usize, AutomationError> {
    let object_group = serde_json::to_string(object_group)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("automation/release_remote_object_group.js"),
        &[("__OBJECT_GROUP__", &object_group)],
    );
    let value = page
        .eval(&source)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    let count = value
        .to_number()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    usize::try_from(count as u64)
        .map_err(|_| AutomationError::Internal("invalid released object count".into()))
}

impl From<NavigationError> for AutomationError {
    fn from(error: NavigationError) -> Self {
        match error {
            NavigationError::Network(error) => Self::Transport(error.to_string()),
            NavigationError::HttpStatus(status) => Self::HttpStatus(status),
            NavigationError::InvalidUrl(error) => {
                Self::InvalidInput(format!("invalid URL: {error}"))
            }
            NavigationError::InvalidRequest(error) => Self::InvalidInput(error),
            other => Self::Navigation(other.to_string()),
        }
    }
}
