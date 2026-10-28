use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::network::{HeaderList, ResourceRequest};

use serde_json::Value;

use crate::runtime::{
    ExtractedDocument, ExtractionOptions, NavigationError, NavigationResponse, Page,
    session::PageSetup,
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

pub use super::CancellationToken;
pub use super::error::AutomationError;

#[derive(Clone)]
pub struct PageHandle {
    pub(super) control: Arc<PageControl>,
}
impl PageHandle {
    pub fn id(&self) -> u64 {
        self.control.id
    }
    pub(crate) fn native_operation(
        &self,
        operation: super::native::Operation,
        timeout: Duration,
    ) -> Result<super::native::OperationResult, AutomationError> {
        self.request(|response| Command::Native {
            operation,
            timeout,
            epoch: self.control.interrupt.load(Ordering::Acquire),
            response,
        })?
    }
    pub(super) fn launch(setup: PageSetup) -> Result<Self, AutomationError> {
        let (commands, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let closed = setup.shutdown();
        let owner_closed = closed.clone();
        let interrupt = Arc::new(AtomicU64::new(0));
        let owner_interrupt = interrupt.clone();
        let worker = std::thread::Builder::new()
            .name("brimp-page-owner".into())
            .spawn(move || run_page(setup, receiver, ready_sender, owner_closed, owner_interrupt))
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        let id = ready_receiver
            .recv()
            .map_err(|_| AutomationError::Internal("page owner exited during startup".into()))??;
        Ok(Self {
            control: Arc::new(PageControl {
                id,
                interrupt,
                commands: Mutex::new(Some(commands)),
                wake: worker.thread().clone(),
                worker: Mutex::new(Some(worker)),
                closed,
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
        self.request(|response| {
            // Serialize the interrupt with command submission, so a concurrent
            // evaluation cannot capture the new epoch ahead of this navigation.
            self.control.interrupt.fetch_add(1, Ordering::AcqRel);
            Command::Navigate {
                method,
                url: url.into(),
                timeout,
                cancellation,
                headers: request_headers,
                body,
                allow_redirects,
                max_redirects,
                response,
            }
        })?
    }
    pub(crate) fn set_network_offline(&self, offline: bool) -> Result<(), AutomationError> {
        self.request(|response| Command::NetworkOffline { offline, response })?
    }

    pub fn evaluate(&self, expression: impl Into<String>) -> Result<Value, AutomationError> {
        match self.native_operation(
            super::native::Operation::Evaluate {
                source: expression.into(),
                by_value: true,
            },
            Duration::from_secs(30),
        )? {
            super::native::OperationResult::Value(value) => Ok(value),
            _ => unreachable!(),
        }
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
    pub fn load_html(
        &self,
        html: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::LoadHtml {
            html: html.into(),
            url: url.into(),
            response,
        })?
    }
    pub(crate) fn set_identity(
        &self,
        identity: super::native::IdentityOptions,
    ) -> Result<(), AutomationError> {
        self.request(|response| Command::Identity { identity, response })?
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
    pub(crate) fn input(&self, command: Value) -> Result<(), AutomationError> {
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
    pub(crate) fn ensure_access(&self) -> Result<(), AutomationError> {
        if super::plugins::is_callback_page(self.id()) {
            return Err(AutomationError::Reentry);
        }
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        Ok(())
    }
    fn request<T>(
        &self,
        command: impl FnOnce(mpsc::SyncSender<T>) -> Command,
    ) -> Result<T, AutomationError> {
        self.ensure_access()?;
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
        drop(commands);
        self.control.wake.unpark();
        receiver.recv().map_err(|_| AutomationError::Closed)
    }
}
impl Drop for PageHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.control) == 1 {
            self.close();
        }
    }
}

pub(super) struct PageControl {
    wake: std::thread::Thread,
    id: u64,
    interrupt: Arc<AtomicU64>,
    commands: Mutex<Option<mpsc::Sender<Command>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
    pub(super) closed: Arc<AtomicBool>,
}
impl PageControl {
    pub(super) fn close(&self) {
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
            self.wake.unpark();
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
    Identity {
        identity: super::native::IdentityOptions,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
    Native {
        operation: super::native::Operation,
        timeout: Duration,
        epoch: u64,
        response: mpsc::SyncSender<Result<super::native::OperationResult, AutomationError>>,
    },
    NetworkOffline {
        offline: bool,
        response: mpsc::SyncSender<Result<(), AutomationError>>,
    },
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

    Text {
        response: mpsc::SyncSender<Result<String, AutomationError>>,
    },
    Html {
        response: mpsc::SyncSender<Result<String, AutomationError>>,
    },
    LoadHtml {
        html: String,
        url: String,
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
    Close(mpsc::SyncSender<()>),
}

fn run_page(
    setup: PageSetup,
    commands: mpsc::Receiver<Command>,
    ready: mpsc::SyncSender<Result<u64, AutomationError>>,
    closed: Arc<AtomicBool>,
    interrupt: Arc<AtomicU64>,
) {
    let mut page = match setup.create() {
        Ok(page) => page,
        Err(error) => {
            let _ = ready.send(Err(error));
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
    let _ = ready.send(Ok(page.id()));
    loop {
        let command = match commands.try_recv() {
            Ok(command) => command,
            Err(mpsc::TryRecvError::Empty) => {
                let _ = page.run_pending_tasks();
                runtime.block_on(page.process_dynamic_scripts());
                if page.advance_bridge_timer() {
                    continue;
                }
                page.wait_for_work(None);
                continue;
            }
            Err(mpsc::TryRecvError::Disconnected) => break,
        };
        match command {
            Command::Identity { identity, response } => {
                let _ = response.send(page.set_identity(identity));
            }
            Command::Native {
                operation,
                timeout,
                epoch,
                response,
            } => {
                let started = Instant::now();
                page.set_script_execution_deadline(timeout);
                let result = page.check_operation().and_then(|()| {
                    super::native::execute_operation(
                        &mut page, operation, timeout, &closed, &interrupt, epoch,
                    )
                });
                page.clear_script_execution_deadline();
                let result = page.check_operation().and_then(|()| {
                    if started.elapsed() >= timeout {
                        Err(AutomationError::Timeout(timeout))
                    } else {
                        result
                    }
                });
                let _ = response.send(result);
            }
            Command::NetworkOffline { offline, response } => {
                let result = page
                    .set_network_offline(offline)
                    .map_err(|error| AutomationError::JavaScript(error.to_string()));
                let _ = response.send(result);
            }

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
                let started = Instant::now();
                page.set_script_execution_deadline(timeout);
                let session_cancel = page.cancellation.clone();
                let session_deadline = page.operation_deadline;
                let mut result = runtime.block_on(async {
                    tokio::select! {
                        biased;
                        error = wait_for_operation(session_cancel, session_deadline) => Err(error),
                        () = wait_for_close(&closed) => Err(AutomationError::Closed),
                        () = wait_for_cancellation(cancellation) => Err(AutomationError::Cancellation),
                        () = tokio::time::sleep(timeout) => Err(AutomationError::Timeout(timeout)),
                        result = page.goto_request(request, allow_redirects, max_redirects) => result.map_err(AutomationError::from),
                    }
                });
                page.clear_script_execution_deadline();
                if started.elapsed() >= timeout
                    && !matches!(
                        result,
                        Err(AutomationError::Cancellation | AutomationError::Closed)
                    )
                {
                    result = Err(AutomationError::Timeout(timeout));
                }
                let result = page.check_operation().and(result);
                let _ = response.send(result);
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
            Command::LoadHtml {
                html,
                url,
                response,
            } => {
                let result = page
                    .load_html(&html, &url)
                    .map_err(|error| AutomationError::JavaScript(error.to_string()));
                let _ = response.send(result);
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
                        page.check_operation()?;
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
                        runtime.block_on(page.process_dynamic_scripts());
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
                    if let Err(error) = page.check_operation() {
                        break Err(error);
                    }
                    if cancellation.is_cancelled() {
                        break Err(AutomationError::Cancellation);
                    }
                    if started.elapsed() >= timeout {
                        break Err(AutomationError::Timeout(timeout));
                    }
                    if let Err(error) = page.run_pending_tasks() {
                        break Err(AutomationError::JavaScript(error.to_string()));
                    }
                    runtime.block_on(page.process_dynamic_scripts());
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
            Command::Close(done) => {
                drop(page);
                let _ = done.send(());
                break;
            }
        }
    }
}

async fn wait_for_operation(
    cancellation: CancellationToken,
    deadline: Option<Instant>,
) -> AutomationError {
    loop {
        if cancellation.is_cancelled() {
            return AutomationError::Cancellation;
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return AutomationError::Timeout(Duration::ZERO);
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn wait_for_cancellation(cancellation: CancellationToken) {
    while !cancellation.is_cancelled() {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn wait_for_close(closed: &AtomicBool) {
    while !closed.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn evaluate(page: &Page, expression: &str) -> Result<Value, AutomationError> {
    let encoded = serde_json::to_string(expression)
        .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
    let source = render_script(
        include_str!("../../js/internal/page_handle/evaluate_json.js"),
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
