//! Remote browser operations and lifecycle, independent of the message transport.
use crate::connection::Connection;
use crate::extraction_assets::{DEFUDDLE_BUNDLE, INSTALL_EXTRACTOR};
#[cfg(target_os = "macos")]
use crate::transport::{Transport, WebSocketTransport};
use crate::{AutomationError as Error, CancellationToken};
use base64::{Engine, engine::general_purpose::STANDARD};
use brimp_protocol::{ExtractedDocument, ExtractionOptions, WorkerConfig};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

pub struct Browser {
    connection: Arc<Connection>,
    lite: bool,
    version: Value,
    dom_ready: bool,
    context: String,
    closed: AtomicBool,
    #[cfg(target_os = "macos")]
    worker: OwnedWorker,
}
/// A launched worker's protocol. Both modes own the child process.
#[derive(Clone, Copy, Default)]
pub enum WorkerProtocol {
    #[default]
    FramedCdp,
    Cdp,
}

pub struct WorkerOptions {
    pub path: String,
    pub protocol: WorkerProtocol,
    pub arguments: Vec<String>,
}

impl WorkerOptions {
    /// Read exact launch arguments from a JSON array. No shell expansion occurs.
    pub fn read_arguments(path: &std::path::Path) -> Result<Vec<String>, Error> {
        let bytes = std::fs::read(path).map_err(|error| {
            Error::InvalidInput(format!(
                "cannot read worker arguments `{}`: {error}",
                path.display()
            ))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            Error::InvalidInput(format!(
                "worker arguments `{}` must be a JSON array of strings: {error}",
                path.display()
            ))
        })
    }
}

#[cfg(target_os = "macos")]
enum OwnedWorker {
    Framed(crate::platform::WorkerProcess),
    Cdp(crate::worker::CdpWorker),
}
#[cfg(target_os = "macos")]
impl OwnedWorker {
    fn terminate(&self) {
        match self {
            Self::Framed(process) => process.terminate(),
            Self::Cdp(worker) => worker.process.terminate(),
        }
    }
}
impl Browser {
    /// Launch and own a browser worker, discovering its connection automatically.
    pub fn launch(
        options: &WorkerOptions,
        config: WorkerConfig,
        timeout: Duration,
        cancellation: CancellationToken,
        dom_ready: bool,
    ) -> Result<Self, Error> {
        if cancellation.is_cancelled() {
            return Err(Error::Cancellation);
        }
        #[cfg(target_os = "macos")]
        {
            let started = Instant::now();
            let remaining = || {
                timeout
                    .checked_sub(started.elapsed())
                    .filter(|time| !time.is_zero())
                    .ok_or(Error::Timeout(timeout))
            };
            let (worker, transport): (OwnedWorker, Arc<dyn Transport>) = match options.protocol {
                WorkerProtocol::FramedCdp => {
                    let arguments = if options.arguments.is_empty() {
                        vec!["--headless".into()]
                    } else {
                        options.arguments.clone()
                    };
                    let (process, stream) =
                        crate::platform::spawn_framed_worker(&options.path, &arguments).map_err(
                            |error| {
                                Error::Transport(format!(
                                    "could not launch worker `{}`: {error}",
                                    options.path
                                ))
                            },
                        )?;
                    let transport = crate::transport::SocketPair::from_stream(stream)
                        .map_err(|error| Error::Transport(error.to_string()))?;
                    (OwnedWorker::Framed(process), Arc::new(transport))
                }
                WorkerProtocol::Cdp => {
                    let worker = crate::worker::CdpWorker::spawn(
                        &options.path,
                        &options.arguments,
                        remaining()?,
                        cancellation.flag(),
                    )?;
                    let transport = WebSocketTransport::connect(
                        &worker.endpoint(),
                        remaining()?,
                        &cancellation,
                    )
                    .map_err(|error| match error.kind() {
                        std::io::ErrorKind::Interrupted => Error::Cancellation,
                        std::io::ErrorKind::TimedOut => Error::Timeout(timeout),
                        _ => Error::Transport(error.to_string()),
                    })?;
                    (OwnedWorker::Cdp(worker), Arc::new(transport))
                }
            };
            let connection = Arc::new(Connection::new(transport, remaining()?, cancellation));
            Self::initialize(connection, worker, config, dom_ready)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (options, config, timeout, cancellation, dom_ready);
            Err(Error::Unsupported(
                "worker launch is supported only on macOS".into(),
            ))
        }
    }
    #[cfg(target_os = "macos")]
    fn initialize(
        connection: Arc<Connection>,
        worker: OwnedWorker,
        config: WorkerConfig,
        dom_ready: bool,
    ) -> Result<Self, Error> {
        let version = connection.command("Browser.getVersion", json!({}), None)?;
        if version["protocolVersion"] != "1.3" {
            return Err(Error::Unsupported("worker must speak CDP 1.3".into()));
        }
        let lite = version["brimpWorker"] == "lite";
        if lite {
            connection.command(
                "Brimp.configure",
                serde_json::to_value(&config).map_err(|e| Error::InvalidInput(e.to_string()))?,
                None,
            )?;
        } else if config.requires_lite() || !config.headers.is_empty() {
            return Err(Error::Unsupported("selected worker does not support custom headers, lite persona, proxy, trust-root, subsystem, or persistent-storage options".into()));
        }
        let context = field(
            &connection.command(
                "Target.createBrowserContext",
                json!({"disposeOnDetach": true}),
                None,
            )?,
            "browserContextId",
        )?;
        Ok(Self {
            connection,
            lite,
            version,
            dom_ready,
            context,
            closed: AtomicBool::new(false),
            #[cfg(target_os = "macos")]
            worker,
        })
    }
    pub fn doctor(&self) -> Result<Value, Error> {
        if self.lite {
            self.connection.command("Brimp.doctor", json!({}), None)
        } else {
            Ok(json!({"worker":self.version,"protocol":"ok"}))
        }
    }
    pub fn default_context(&self) -> Context {
        Context(self.connection.clone(), self.context.clone(), self.lite)
    }
    pub fn new_page(&self) -> Result<Page, Error> {
        let target = self.connection.command(
            "Target.createTarget",
            json!({"url":"about:blank", "browserContextId":self.context}),
            None,
        )?;
        let target = field(&target, "targetId")?;
        let attached = self.connection.command(
            "Target.attachToTarget",
            json!({"targetId":target,"flatten":true}),
            None,
        )?;
        let session = field(&attached, "sessionId")?;
        let page = Page {
            connection: self.connection.clone(),
            target,
            session,
            lite: self.lite,
            dom_ready: self.dom_ready,
            closed: AtomicBool::new(false),
        };
        for method in ["Page.enable", "Runtime.enable", "Network.enable"] {
            page.command(method, json!({}))?;
        }
        Ok(page)
    }
    pub fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let _ = self.connection.command(
            "Target.disposeBrowserContext",
            json!({"browserContextId": self.context}),
            None,
        );
        self.connection.close();
        #[cfg(target_os = "macos")]
        self.worker.terminate();
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        self.close();
    }
}
pub struct Context(Arc<Connection>, String, bool);
impl Context {
    pub fn set_cookie(&self, url: &str, name: &str, value: &str) -> Result<(), Error> {
        if !self.2 {
            return Err(Error::Unsupported(
                "selected WebKit worker does not support setting cookies".into(),
            ));
        }
        self.0.command(
            "Storage.setCookies",
            json!({"browserContextId":self.1, "cookies":[{"url":url,"name":name,"value":value}]}),
            None,
        )?;
        Ok(())
    }
}
pub struct NavigationResponse {
    pub url: String,
    pub status_code: u16,
    pub content: Vec<u8>,
}
pub struct Page {
    connection: Arc<Connection>,
    target: String,
    session: String,
    lite: bool,
    dom_ready: bool,
    closed: AtomicBool,
}
impl Page {
    fn command(&self, method: &str, params: Value) -> Result<Value, Error> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        self.connection.command(method, params, Some(&self.session))
    }
    pub fn navigate_cancellable(
        &self,
        url: impl Into<String>,
        cancellation: CancellationToken,
        capture_body: bool,
    ) -> Result<NavigationResponse, Error> {
        if cancellation.is_cancelled() {
            return Err(Error::Cancellation);
        }
        if capture_body && !self.lite {
            return Err(Error::Unsupported("selected WebKit worker does not support raw response bodies (including robots.txt); use rendered output and --ignore-robots for crawling".into()));
        }
        self.connection.drain_events(&self.session);
        let result = self.command("Page.navigate", json!({"url":url.into()}))?;
        if let Some(error) = result["errorText"].as_str() {
            return Err(Error::Navigation(error.into()));
        }
        let event = if self.dom_ready {
            "Page.domContentEventFired"
        } else {
            "Page.loadEventFired"
        };
        let navigation_id = result.get("navigationId");
        let ready = self.connection.event(&self.session, |e| {
            (e["method"] == event
                || (e["method"] == "Network.loadingFailed" && e["params"]["type"] == "Document"))
                && navigation_id.is_none_or(|id| {
                    e["params"]
                        .get("navigationId")
                        .is_none_or(|event_id| event_id == id)
                })
        })?;
        if ready["method"] == "Network.loadingFailed" {
            return Err(Error::Transport(
                ready["params"]["errorText"]
                    .as_str()
                    .unwrap_or("navigation failed")
                    .into(),
            ));
        }
        let response = self.connection.event(&self.session, |e| {
            e["method"] == "Network.responseReceived" && e["params"]["type"] == "Document"
        })?;
        let request_id = &response["params"]["requestId"];
        let content = if capture_body {
            let body = self.command("Network.getResponseBody", json!({"requestId":request_id}))?;
            decode(&body, "body", "base64Encoded")?
        } else {
            Vec::new()
        };
        Ok(NavigationResponse {
            url: field(&response["params"]["response"], "url")?,
            status_code: response["params"]["response"]["status"]
                .as_u64()
                .ok_or_else(|| Error::Transport("response has no status".into()))?
                as u16,
            content,
        })
    }
    pub fn evaluate(&self, expression: impl Into<String>) -> Result<Value, Error> {
        let result = self.command(
            "Runtime.evaluate",
            json!({"expression":expression.into(),"returnByValue":true,"awaitPromise":true}),
        )?;
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::JavaScript(exception.to_string()));
        }
        let object = &result["result"];
        if object.get("unserializableValue").is_some() {
            return Err(Error::Unsupported(
                "JavaScript result is not JSON serializable".into(),
            ));
        }
        if object["type"] == "undefined" {
            return Ok(Value::Null);
        }
        object
            .get("value")
            .cloned()
            .ok_or_else(|| Error::Unsupported("JavaScript result is not JSON serializable".into()))
    }
    pub fn extract(&self, options: ExtractionOptions) -> Result<ExtractedDocument, Error> {
        if self.lite {
            let params =
                serde_json::to_value(options).map_err(|e| Error::Extraction(e.to_string()))?;
            return serde_json::from_value(self.command("Brimp.extract", params)?)
                .map_err(|e| Error::Extraction(e.to_string()));
        }
        // External WebKit workers expose extraction through standard evaluation.

        let mut options =
            serde_json::to_value(options).map_err(|e| Error::Extraction(e.to_string()))?;
        options["separateMarkdown"] = true.into();
        options["useAsync"] = false.into();
        let encoded = serde_json::to_string(&options.to_string()).unwrap();
        let expression = format!(
            "(() => {{ {DEFUDDLE_BUNDLE}; const extract = {INSTALL_EXTRACTOR}; return extract({encoded}); }})()"
        );
        let result = self
            .evaluate(expression)
            .map_err(|e| Error::Extraction(e.to_string()))?;
        serde_json::from_str(
            result
                .as_str()
                .ok_or_else(|| Error::Extraction("extractor returned no JSON".into()))?,
        )
        .map_err(|e| Error::Extraction(e.to_string()))
    }
    pub fn screenshot(&self, full_page: bool) -> Result<Vec<u8>, Error> {
        let result = self.command(
            "Page.captureScreenshot",
            json!({"format":"png","captureBeyondViewport":full_page}),
        )?;
        STANDARD
            .decode(field(&result, "data")?)
            .map_err(|e| Error::Screenshot(e.to_string()))
    }
    pub fn wait_for_selector(
        &self,
        selector: String,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), Error> {
        let deadline = Instant::now() + timeout;
        let expression = format!(
            "document.querySelector({}) !== null",
            serde_json::to_string(&selector).unwrap()
        );
        loop {
            if cancellation.is_cancelled() {
                return Err(Error::Cancellation);
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(timeout));
            }
            if self.evaluate(expression.clone())? == true {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn wait_for_network_idle(
        &self,
        quiet: Duration,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), Error> {
        if self.lite {
            self.command(
                "Brimp.waitForNetworkIdle",
                json!({"quietMs":quiet.as_millis(),"timeoutMs":timeout.as_millis()}),
            )?;
            return Ok(());
        }
        let deadline = Instant::now() + timeout;
        let mut idle = Instant::now();
        let mut pending = HashSet::new();
        loop {
            self.connection.check()?;
            if cancellation.is_cancelled() {
                return Err(Error::Cancellation);
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(timeout));
            }
            for event in self.connection.drain_events(&self.session) {
                let id = event["params"]["requestId"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                match event["method"].as_str() {
                    Some("Network.requestWillBeSent") => {
                        pending.insert(id);
                        idle = Instant::now();
                    }
                    Some("Network.loadingFinished" | "Network.loadingFailed") => {
                        pending.remove(&id);
                        idle = Instant::now();
                    }
                    _ => {}
                }
            }
            if pending.is_empty() && idle.elapsed() >= quiet {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn close(&self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            let _ = self.connection.command(
                "Target.closeTarget",
                json!({"targetId":self.target}),
                None,
            );
            self.connection.drain_events(&self.session);
        }
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        self.close();
    }
}
fn field(value: &Value, name: &str) -> Result<String, Error> {
    value[name]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::Transport(format!("worker response missing {name}")))
}
fn decode(value: &Value, data: &str, base64: &str) -> Result<Vec<u8>, Error> {
    let data = field(value, data)?;
    if value[base64] == true {
        STANDARD
            .decode(data)
            .map_err(|e| Error::Transport(e.to_string()))
    } else {
        Ok(data.into_bytes())
    }
}
