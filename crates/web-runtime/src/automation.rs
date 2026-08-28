use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use network::{HeaderList, ResourceLoader};
use serde_json::Value;

use crate::{NavigationError, NavigationResponse, Page, PageOptions, ScreenshotOptions, Viewport};

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
    #[error("runtime failure: {0}")]
    Internal(String),
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
            network::CurlResourceLoader::new(config)
                .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        Ok(Self {
            loader,
            persona: Some(persona),
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
        })
    }
    pub fn new_page(&self, options: PageOptions) -> Result<AutomationPage, AutomationError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        let options = self
            .persona
            .clone()
            .map_or(options.clone(), |persona| options.with_persona(persona));
        let page = AutomationPage::launch(options, Arc::clone(&self.loader))?;
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
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, AutomationError> {
        let (commands, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("brimp-page-owner".into())
            .spawn(move || run_page(options, loader, receiver, ready_sender))
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
        if timeout.is_zero() {
            return Err(AutomationError::InvalidInput(
                "timeout must be positive".into(),
            ));
        }
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
            url: url.into(),
            timeout,
            cancellation,
            headers: request_headers,
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
    pub fn close(&self) {
        self.control.close();
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
    Navigate {
        url: String,
        timeout: Duration,
        cancellation: CancellationToken,
        headers: HeaderList,
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
    Viewport {
        response: mpsc::SyncSender<Result<Viewport, AutomationError>>,
    },
    SetViewport {
        width: u32,
        height: u32,
        device_pixel_ratio: f64,
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
    Close(mpsc::SyncSender<()>),
}

fn run_page(
    options: PageOptions,
    loader: Arc<dyn ResourceLoader>,
    commands: mpsc::Receiver<Command>,
    ready: mpsc::SyncSender<Result<(), AutomationError>>,
) {
    let mut page = match Page::new(options, loader) {
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
            Command::Navigate {
                url,
                timeout,
                cancellation,
                headers,
                response,
            } => {
                let result = runtime.block_on(async {
                    tokio::select! {
                        result = page.goto_with_headers(&url, headers) => result.map_err(AutomationError::from),
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
    let source = format!(
        r#"(() => {{ const value = (0, eval)({encoded}); const kind = typeof value; if (kind === "undefined" || kind === "function" || kind === "symbol" || kind === "bigint") throw new Error("BRIMP_UNSUPPORTED_RESULT:" + kind); let json; try {{ json = JSON.stringify(value); }} catch (error) {{ throw new Error("BRIMP_UNSUPPORTED_RESULT:" + error.message); }} if (json === undefined) throw new Error("BRIMP_UNSUPPORTED_RESULT:unserializable"); return json; }})()"#
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
    evaluate_remote_maybe_await(
        page,
        &format!("const value = (0, eval)({expression});"),
        return_by_value,
        object_group,
        await_promise,
    )
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
    let body = format!(
        "const declarationSource = {declaration};\
         const declaration = (0, eval)('(' + declarationSource + ')');\
         const receiverId = {receiver};\
         const receiver = receiverId === null ? globalThis : state.objects.get(receiverId);\
         if (receiverId !== null && receiver === undefined) throw new Error('Unknown remote object: ' + receiverId);\
         const specs = {arguments};\
         const args = specs.map(spec => {{\
             if ('objectId' in spec) {{\
                 const value = state.objects.get(spec.objectId);\
                 if (value === undefined) throw new Error('Unknown remote object: ' + spec.objectId);\
                 return value;\
             }}\
             if ('unserializableValue' in spec) {{\
                 if (spec.unserializableValue === 'NaN') return NaN;\
                 if (spec.unserializableValue === 'Infinity') return Infinity;\
                 if (spec.unserializableValue === '-Infinity') return -Infinity;\
                 if (spec.unserializableValue === '-0') return -0;\
                 if (spec.unserializableValue.endsWith('n')) return BigInt(spec.unserializableValue.slice(0, -1));\
             }}\
             return spec.value;\
         }});\
         const value = declaration.apply(receiver, args);"
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
    let setup = format!(
        r#"(() => {{
            const state = globalThis.__brimpCdpRemoteObjects ||
                (globalThis.__brimpCdpRemoteObjects = {{next: 1, objects: new Map(), groups: new Map(), objectGroups: new Map()}});
            {body}
            const pending = {{settled: false, rejected: false, value: undefined}};
            state.pendingPromise = pending;
            Promise.resolve(value).then(
                result => {{ pending.value = result; pending.settled = true; }},
                error => {{ pending.value = error; pending.rejected = true; pending.settled = true; }}
            );
            return true;
        }})()"#
    );
    page.eval(&setup)
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    let timeout = Duration::from_secs(30);
    let started = Instant::now();
    loop {
        let settled = page
            .eval("Boolean(globalThis.__brimpCdpRemoteObjects?.pendingPromise?.settled)")
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
        "const pending = state.pendingPromise; delete state.pendingPromise; if (pending.rejected) throw pending.value; const value = pending.value;",
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
    let source = format!(
        r#"(() => {{
            const state = globalThis.__brimpCdpRemoteObjects ||
                (globalThis.__brimpCdpRemoteObjects = {{next: 1, objects: new Map(), groups: new Map(), objectGroups: new Map()}});
            {body}
            const type = typeof value;
            const group = {object_group};
            let result;
            if (value === null) result = {{type: 'object', subtype: 'null', value: null}};
            else if (type === 'undefined') result = {{type: 'undefined'}};
            else if (type === 'number' && (!Number.isFinite(value) || Object.is(value, -0))) result = {{type: 'number', unserializableValue: Object.is(value, -0) ? '-0' : String(value)}};
            else if (type === 'bigint') result = {{type: 'bigint', unserializableValue: String(value) + 'n'}};
            else if (type !== 'object' && type !== 'function' && type !== 'symbol') result = {{type, value}};
            else if ({return_by_value}) result = {{type, value}};
            else {{
                const objectId = 'object-' + state.next++;
                state.objects.set(objectId, value);
                if (group !== null && group !== '') {{
                    let ids = state.groups.get(group);
                    if (ids === undefined) state.groups.set(group, ids = new Set());
                    ids.add(objectId);
                    state.objectGroups.set(objectId, group);
                }}
                result = {{
                    type,
                    subtype: Array.isArray(value) ? 'array' : (value && typeof value.nodeType === 'number' ? 'node' : undefined),
                    className: value && value.constructor ? value.constructor.name : undefined,
                    description: type === 'function' ? String(value) : Object.prototype.toString.call(value),
                    objectId
                }};
            }}
            return JSON.stringify(result);
        }})()"#
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
    let source = format!(
        r#"(() => {{
            const state = globalThis.__brimpCdpRemoteObjects;
            const objectId = {object_id};
            if (!state || !state.objects.has(objectId)) throw new Error('Unknown remote object: ' + objectId);
            const object = state.objects.get(objectId);
            const group = state.objectGroups.get(objectId) ?? null;
            const remote = value => {{
                const type = typeof value;
                if (value === null) return {{type: 'object', subtype: 'null', value: null}};
                if (type === 'undefined') return {{type: 'undefined'}};
                if (type === 'number' && (!Number.isFinite(value) || Object.is(value, -0)))
                    return {{type: 'number', unserializableValue: Object.is(value, -0) ? '-0' : String(value)}};
                if (type === 'bigint') return {{type: 'bigint', unserializableValue: String(value) + 'n'}};
                if (type !== 'object' && type !== 'function' && type !== 'symbol') return {{type, value}};
                const childId = 'object-' + state.next++;
                state.objects.set(childId, value);
                if (group !== null) {{
                    let ids = state.groups.get(group);
                    if (ids === undefined) state.groups.set(group, ids = new Set());
                    ids.add(childId);
                    state.objectGroups.set(childId, group);
                }}
                return {{
                    type,
                    subtype: Array.isArray(value) ? 'array' : (value && typeof value.nodeType === 'number' ? 'node' : undefined),
                    className: value && value.constructor ? value.constructor.name : undefined,
                    description: type === 'function' || type === 'symbol' ? String(value) : Object.prototype.toString.call(value),
                    objectId: childId
                }};
            }};
            const result = [];
            const seen = new Set();
            for (let current = object; current !== null; current = {own_properties} ? null : Object.getPrototypeOf(current)) {{
                for (const key of Reflect.ownKeys(Object(current))) {{
                    const name = typeof key === 'symbol' ? String(key) : key;
                    if (seen.has(name)) continue;
                    seen.add(name);
                    const descriptor = Object.getOwnPropertyDescriptor(current, key);
                    if ({accessor_properties_only} && !descriptor.get && !descriptor.set) continue;
                    const property = {{
                        name,
                        configurable: descriptor.configurable,
                        enumerable: descriptor.enumerable,
                        isOwn: current === object
                    }};
                    if ('value' in descriptor) {{
                        property.value = remote(descriptor.value);
                        property.writable = descriptor.writable;
                    }} else {{
                        if (descriptor.get) property.get = remote(descriptor.get);
                        if (descriptor.set) property.set = remote(descriptor.set);
                    }}
                    result.push(property);
                }}
            }}
            return JSON.stringify({{result, internalProperties: [], privateProperties: []}});
        }})()"#
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
    let source = format!(
        r#"(() => {{
            const state = globalThis.__brimpCdpRemoteObjects;
            const objectId = {object_id};
            if (!state || !state.objects.has(objectId)) throw new Error('Unknown remote object: ' + objectId);
            const node = state.objects.get(objectId);
            if (!node || typeof node.nodeType !== 'number') throw new Error('Remote object is not a DOM node: ' + objectId);
            if (!state.nodeIds) {{ state.nextNode = 1; state.nodeIds = new WeakMap(); state.backendNodes = new Map(); }}
            let backendNodeId = state.nodeIds.get(node);
            if (backendNodeId === undefined) {{
                backendNodeId = state.nextNode++;
                state.nodeIds.set(node, backendNodeId);
                state.backendNodes.set(backendNodeId, node);
            }}
            return JSON.stringify({{
                nodeId: backendNodeId,
                backendNodeId,
                nodeType: node.nodeType,
                nodeName: node.nodeName || '',
                localName: node.localName || '',
                nodeValue: node.nodeValue || '',
                childNodeCount: node.childNodes ? node.childNodes.length : 0
            }});
        }})()"#
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
    evaluate_remote_source(
        page,
        &format!(
            "if (!state.backendNodes || !state.backendNodes.has({backend_node_id})) throw new Error('Unknown backend node: {backend_node_id}'); const value = state.backendNodes.get({backend_node_id});"
        ),
        false,
        object_group,
    )
}

fn release_remote_object(page: &Page, object_id: &str) -> Result<bool, AutomationError> {
    let object_id = serde_json::to_string(object_id)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let value = page
        .eval(&format!(
            "(() => {{ const state = globalThis.__brimpCdpRemoteObjects; if (!state) return false; const group = state.objectGroups.get({object_id}); if (group !== undefined) {{ state.groups.get(group)?.delete({object_id}); state.objectGroups.delete({object_id}); }} return state.objects.delete({object_id}); }})()"
        ))
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?;
    Ok(value
        .to_string()
        .map_err(|error| AutomationError::JavaScript(error.to_string()))?
        == "true")
}

fn release_remote_object_group(page: &Page, object_group: &str) -> Result<usize, AutomationError> {
    let object_group = serde_json::to_string(object_group)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let value = page
        .eval(&format!(
            "(() => {{ const state = globalThis.__brimpCdpRemoteObjects; if (!state) return 0; const ids = state.groups.get({object_group}); if (!ids) return 0; for (const id of ids) {{ state.objects.delete(id); state.objectGroups.delete(id); }} state.groups.delete({object_group}); return ids.size; }})()"
        ))
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
            NavigationError::InvalidUrl(error) => Self::InvalidInput(error.to_string()),
            other => Self::Navigation(other.to_string()),
        }
    }
}
