use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
use std::time::Duration;

use network::{HeaderList, ResourceLoader};
use serde_json::Value;

use crate::{NavigationError, NavigationResponse, Page, PageOptions, ScreenshotOptions};

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
    persona: Option<persona::Persona>,
    pages: Mutex<Vec<Weak<PageControl>>>,
    closed: AtomicBool,
}
impl AutomationBrowser {
    pub fn new() -> Result<Self, AutomationError> {
        let persona = persona::Persona::default();
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
    pub fn with_persona(persona: persona::Persona) -> Result<Self, AutomationError> {
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        let config = network::CurlConfig {
            impersonation_profile: persona.transport_profile.clone(),
            default_headers: false,
            ..network::CurlConfig::default()
        };
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
    pub fn title(&self) -> Result<String, AutomationError> {
        self.evaluate("document.title")?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AutomationError::Internal("document title was not a string".into()))
    }
    pub fn text_content(&self) -> Result<String, AutomationError> {
        self.request(|response| Command::Text { response })?
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
    Text {
        response: mpsc::SyncSender<Result<String, AutomationError>>,
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
    while let Ok(command) = commands.recv() {
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
            Command::Text { response } => {
                let result =
                    evaluate(&page, "document.documentElement.textContent").and_then(|value| {
                        value.as_str().map(str::to_owned).ok_or_else(|| {
                            AutomationError::Internal("document text was not a string".into())
                        })
                    });
                let _ = response.send(result);
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
