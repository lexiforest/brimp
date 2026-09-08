use crate::platform::{self, WorkerProcess};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::handshake::server::{Request, Response};
use tungstenite::{Message, WebSocket};

const CDP_VERSION: &str = "1.3";
const PRODUCT: &str = "Brimp/0.1";
const MAX_IPC_MESSAGE: usize = 64 * 1024 * 1024;

#[derive(Clone)]
struct Config {
    worker_path: String,
    worker_args: Vec<String>,
    worker_protocol: String,
    pool_size: usize,
    max_pending: usize,
    max_jobs: usize,
    max_worker_memory: u64,
    operation_timeout: Duration,
    port: u16,
}

impl Config {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut config = Self {
            worker_path: std::env::var("BRIMP_WORKER_PATH").unwrap_or_default(),
            worker_args: Vec::new(),
            worker_protocol: "framed-cdp".into(),
            pool_size: platform::physical_cpu_count().saturating_mul(2).max(1),
            max_pending: 64,
            max_jobs: 100,
            max_worker_memory: 1024 * 1024 * 1024,
            operation_timeout: Duration::from_secs(30),
            port: 9222,
        };
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            let (name, inline) = argument
                .split_once('=')
                .map_or((argument.as_str(), None), |(name, value)| {
                    (name, Some(value))
                });
            let mut value = || {
                inline
                    .map(str::to_owned)
                    .or_else(|| arguments.next())
                    .ok_or_else(|| format!("{name} requires a value"))
            };
            match name {
                "--worker-path" => config.worker_path = value()?,
                "--worker-arg" => config.worker_args.push(value()?),
                "--worker-protocol" => config.worker_protocol = value()?,
                "--pool-size" => config.pool_size = parse_nonzero(&value()?, name)?,
                "--max-pending" => config.max_pending = parse_number(&value()?, name)?,
                "--max-jobs" => config.max_jobs = parse_nonzero(&value()?, name)?,
                "--max-worker-memory-mb" => {
                    config.max_worker_memory =
                        parse_number::<u64>(&value()?, name)?.saturating_mul(1024 * 1024)
                }
                "--operation-timeout" => {
                    let seconds: f64 = value()?
                        .parse()
                        .map_err(|_| "invalid --operation-timeout")?;
                    config.operation_timeout = Duration::try_from_secs_f64(seconds)
                        .ok()
                        .filter(|duration| !duration.is_zero())
                        .ok_or("--operation-timeout must be positive and finite")?;
                }
                "--port" => config.port = parse_number(&value()?, name)?,
                "--headless" => config.worker_args.push(argument),
                _ if name == "--window-size" => config
                    .worker_args
                    .push(format!("--window-size={}", value()?)),
                _ => return Err(format!("unknown argument: {argument}")),
            }
        }
        if config.worker_path.is_empty() {
            return Err("--worker-path is required".into());
        }
        if config.worker_protocol != "framed-cdp" && config.worker_protocol != "cdp" {
            return Err("--worker-protocol must be framed-cdp or cdp".into());
        }
        Ok(config)
    }
}

fn parse_number<T: std::str::FromStr>(value: &str, name: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("invalid {name}"))
}

fn parse_nonzero(value: &str, name: &str) -> Result<usize, String> {
    let parsed = parse_number(value, name)?;
    if parsed == 0 {
        Err(format!("invalid {name}"))
    } else {
        Ok(parsed)
    }
}

enum WorkerMessage {
    Event(Arc<Worker>, Value),
    Failed(Arc<Worker>),
}

struct Worker {
    process: WorkerProcess,
    writer: Mutex<platform::WorkerStream>,
    request_lock: Mutex<()>,
    responses: Arc<(Mutex<HashMap<i64, Value>>, Condvar)>,
    alive: Arc<AtomicBool>,
    failure_sent: Arc<AtomicBool>,
    next_id: AtomicI64,
    jobs: AtomicUsize,
    timeout: Duration,
    stop: Arc<AtomicBool>,
}

impl Worker {
    fn spawn(
        config: &Config,
        events: mpsc::Sender<WorkerMessage>,
        stop: Arc<AtomicBool>,
    ) -> Result<Arc<Self>, String> {
        let arguments = if config.worker_args.is_empty() {
            vec!["--headless".into()]
        } else {
            config.worker_args.clone()
        };
        let (process, stream) = platform::spawn_framed_worker(&config.worker_path, &arguments)
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(config.operation_timeout.min(Duration::from_secs(1))))
            .map_err(|error| error.to_string())?;
        let reader = stream.try_clone().map_err(|error| error.to_string())?;
        let worker = Arc::new(Self {
            process,
            writer: Mutex::new(stream),
            request_lock: Mutex::new(()),
            responses: Arc::new((Mutex::new(HashMap::new()), Condvar::new())),
            alive: Arc::new(AtomicBool::new(true)),
            failure_sent: Arc::new(AtomicBool::new(false)),
            next_id: AtomicI64::new(1),
            jobs: AtomicUsize::new(0),
            timeout: config.operation_timeout,
            stop,
        });
        let weak = Arc::downgrade(&worker);
        thread::spawn(move || {
            read_worker(reader, &weak, &events);
            if let Some(worker) = weak.upgrade() {
                worker.alive.store(false, Ordering::SeqCst);
                worker.responses.1.notify_all();
                if !worker.failure_sent.swap(true, Ordering::SeqCst) {
                    let _ = events.send(WorkerMessage::Failed(worker));
                }
            }
        });
        Ok(worker)
    }

    fn request(
        &self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
    ) -> Result<Value, String> {
        let _request = self.request_lock.lock().unwrap();
        if !self.alive.load(Ordering::SeqCst) {
            return Err("worker is not running".into());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut message = json!({"id": id, "method": method, "params": params});
        if let Some(session) = session_id {
            message["sessionId"] = json!(session)
        }
        let payload = serde_json::to_vec(&message).map_err(|error| error.to_string())?;
        if payload.len() > MAX_IPC_MESSAGE {
            return Err("worker request is too large".into());
        }
        {
            let mut writer = self.writer.lock().unwrap();
            writer
                .write_all(&(payload.len() as u32).to_be_bytes())
                .and_then(|_| writer.write_all(&payload))
                .map_err(|error| error.to_string())?;
        }
        let deadline = Instant::now() + self.timeout;
        let (responses, condition) = &*self.responses;
        let mut responses = responses.lock().unwrap();
        loop {
            if let Some(response) = responses.remove(&id) {
                if let Some(error) = response.get("error") {
                    return Err(error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("worker operation failed")
                        .into());
                }
                return Ok(response.get("result").cloned().unwrap_or_else(|| json!({})));
            }
            if self.stop.load(Ordering::Acquire) {
                drop(responses);
                self.fail();
                return Err("controller was cancelled".into());
            }
            if !self.alive.load(Ordering::SeqCst) {
                return Err("worker closed its IPC connection".into());
            }
            let now = Instant::now();
            if now >= deadline {
                self.fail();
                return Err("worker operation timed out".into());
            }
            (responses, _) = condition
                .wait_timeout(responses, (deadline - now).min(Duration::from_millis(50)))
                .unwrap();
        }
    }

    fn fail(&self) {
        self.alive.store(false, Ordering::SeqCst);
        let _ = self.writer.lock().unwrap().shutdown(Shutdown::Both);
        self.responses.1.notify_all();
        self.process.terminate();
    }
}

fn read_worker(
    mut reader: platform::WorkerStream,
    weak: &std::sync::Weak<Worker>,
    events: &mpsc::Sender<WorkerMessage>,
) {
    loop {
        let mut header = [0_u8; 4];
        if reader.read_exact(&mut header).is_err() {
            return;
        }
        let length = u32::from_be_bytes(header) as usize;
        if length > MAX_IPC_MESSAGE {
            return;
        }
        let mut payload = vec![0; length];
        if reader.read_exact(&mut payload).is_err() {
            return;
        }
        let Ok(message) = serde_json::from_slice::<Value>(&payload) else {
            return;
        };
        let Some(worker) = weak.upgrade() else { return };
        if message.get("method").is_some() {
            let _ = events.send(WorkerMessage::Event(worker, message));
        } else if let Some(id) = message.get("id").and_then(Value::as_i64) {
            worker.responses.0.lock().unwrap().insert(id, message);
            worker.responses.1.notify_all();
        }
    }
}

#[derive(Clone, Default)]
struct Context {
    worker_id: String,
    worker: Option<Arc<Worker>>,
}

#[derive(Clone)]
struct Target {
    id: String,
    worker_session: String,
    url: String,
    context_id: String,
    loader_id: String,
    navigation_id: String,
    worker: Arc<Worker>,
    sessions: HashSet<String>,
}

struct Client {
    sender: mpsc::Sender<Value>,
    discover: bool,
    auto_attach: bool,
    direct_target: String,
    sessions: HashMap<String, String>,
    enabled: HashMap<String, HashSet<String>>,
    next_execution_context: HashMap<String, u64>,
}

struct Controller {
    stop: Arc<AtomicBool>,
    config: Config,
    events: mpsc::Sender<WorkerMessage>,
    workers: Vec<Arc<Worker>>,
    available: Vec<Arc<Worker>>,
    default_context: Context,
    contexts: HashMap<String, Context>,
    targets: HashMap<String, Target>,
    clients: HashMap<String, Client>,
    browser_id: String,
}

impl Controller {
    fn spawn_worker(&mut self) -> Result<Arc<Worker>, String> {
        let worker = Worker::spawn(&self.config, self.events.clone(), self.stop.clone())?;
        let version = worker.request("Browser.getVersion", json!({}), None)?;
        if version.get("protocolVersion").and_then(Value::as_str) != Some(CDP_VERSION) {
            worker.fail();
            return Err("incompatible worker CDP version".into());
        }
        self.workers.push(worker.clone());
        Ok(worker)
    }

    fn ensure_context(&mut self, context_id: &str) -> Result<Context, String> {
        let current = if context_id.is_empty() {
            self.default_context.clone()
        } else {
            self.contexts
                .get(context_id)
                .cloned()
                .ok_or("Unknown browserContextId")?
        };
        if current.worker.is_some() {
            return Ok(current);
        }
        let worker = self
            .available
            .pop()
            .ok_or("Brimp browser-context pool is full")?;
        let result = worker.request("Target.createBrowserContext", json!({}), None)?;
        let worker_id = result
            .get("browserContextId")
            .and_then(Value::as_str)
            .ok_or("worker did not return browserContextId")?
            .to_owned();
        let context = Context {
            worker_id,
            worker: Some(worker),
        };
        if context_id.is_empty() {
            self.default_context = context.clone()
        } else {
            self.contexts.insert(context_id.into(), context.clone());
        }
        Ok(context)
    }

    fn create_target(&mut self, url: &str, context_id: &str) -> Result<String, String> {
        let context = self.ensure_context(context_id)?;
        let worker = context.worker.clone().unwrap();
        let result = worker.request("Target.createTarget", json!({"url": if url.is_empty() { "about:blank" } else { url }, "browserContextId": context.worker_id}), None)?;
        let id = result
            .get("targetId")
            .and_then(Value::as_str)
            .ok_or("worker did not return targetId")?
            .to_owned();
        let attached = worker.request(
            "Target.attachToTarget",
            json!({"targetId": id, "flatten": true}),
            None,
        )?;
        let worker_session = attached
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or("worker did not return sessionId")?
            .to_owned();
        let target = Target {
            id: id.clone(),
            worker_session,
            url: if url.is_empty() {
                "about:blank".into()
            } else {
                url.into()
            },
            context_id: context_id.into(),
            loader_id: "main".into(),
            navigation_id: String::new(),
            worker,
            sessions: HashSet::new(),
        };
        self.targets.insert(id.clone(), target);
        self.emit(
            "Target.targetCreated",
            json!({"targetInfo": self.target_info(&id)}),
            None,
            None,
        );
        Ok(id)
    }

    fn target_info(&self, id: &str) -> Value {
        let target = &self.targets[id];
        let mut value = json!({"targetId": target.id, "type": "page", "title": "", "url": target.url, "attached": !target.sessions.is_empty(), "canAccessOpener": false});
        if !target.context_id.is_empty() {
            value["browserContextId"] = json!(target.context_id)
        }
        value
    }

    fn emit(&self, method: &str, params: Value, selected: Option<&str>, session: Option<&str>) {
        let mut message = json!({"method": method, "params": params});
        if let Some(session) = session {
            message["sessionId"] = json!(session)
        }
        for (id, client) in &self.clients {
            if selected.is_none_or(|selected| selected == id) {
                let _ = client.sender.send(message.clone());
            }
        }
    }

    fn attach(&mut self, client_id: &str, target_id: &str) -> Result<String, String> {
        if !self.targets.contains_key(target_id) {
            return Err("Unknown targetId".into());
        }
        let session = platform::random_token();
        self.targets
            .get_mut(target_id)
            .unwrap()
            .sessions
            .insert(session.clone());
        let client = self.clients.get_mut(client_id).unwrap();
        client.sessions.insert(session.clone(), target_id.into());
        client.enabled.insert(session.clone(), HashSet::new());
        self.emit("Target.attachedToTarget", json!({"sessionId": session, "targetInfo": self.target_info(target_id), "waitingForDebugger": false}), Some(client_id), None);
        Ok(session)
    }

    fn close_target(&mut self, id: &str) -> bool {
        let Some(target) = self.targets.get(id).cloned() else {
            return false;
        };
        let healthy = target
            .worker
            .request("Target.closeTarget", json!({"targetId": id}), None)
            .is_ok();
        for client in self.clients.values_mut() {
            client.sessions.retain(|session, target_id| {
                if target_id == id {
                    client.enabled.remove(session);
                    false
                } else {
                    true
                }
            });
        }
        self.targets.remove(id);
        target.worker.jobs.fetch_add(1, Ordering::Relaxed);
        self.emit(
            "Target.targetDestroyed",
            json!({"targetId": id}),
            None,
            None,
        );
        healthy
    }

    fn release_context(&mut self, context_id: &str) {
        let context = if context_id.is_empty() {
            std::mem::take(&mut self.default_context)
        } else {
            self.contexts.remove(context_id).unwrap_or_default()
        };
        let Some(worker) = context.worker else { return };
        let healthy = worker
            .request(
                "Target.disposeBrowserContext",
                json!({"browserContextId": context.worker_id}),
                None,
            )
            .is_ok()
            && worker.alive.load(Ordering::SeqCst)
            && (self.config.max_worker_memory == 0
                || worker.process.physical_memory_bytes() < self.config.max_worker_memory)
            && worker.jobs.load(Ordering::Relaxed) < self.config.max_jobs;
        if healthy {
            self.available.push(worker)
        } else {
            worker.fail()
        }
    }

    fn command(
        &mut self,
        client_id: &str,
        method: &str,
        params: Value,
        session: Option<&str>,
    ) -> Result<Value, (i64, String)> {
        if let Some(session) = session {
            let target = self
                .clients
                .get(client_id)
                .and_then(|client| client.sessions.get(session))
                .cloned()
                .ok_or((-32602, "Unknown sessionId".into()))?;
            return self.target_command(client_id, method, params, &target, session);
        }
        let direct = self
            .clients
            .get(client_id)
            .map(|client| client.direct_target.clone())
            .unwrap_or_default();
        if !direct.is_empty()
            && ["Page.", "Runtime.", "Network.", "Emulation.", "Input."]
                .iter()
                .any(|prefix| method.starts_with(prefix))
        {
            return self.target_command(client_id, method, params, &direct, "direct");
        }
        match method {
            "Browser.getVersion" => Ok(
                json!({"protocolVersion": CDP_VERSION, "product": PRODUCT, "revision": "", "userAgent": PRODUCT, "jsVersion": ""}),
            ),
            "Browser.close" => Ok(json!({})),
            "Target.getTargets" => Ok(
                json!({"targetInfos": self.targets.keys().map(|id| self.target_info(id)).collect::<Vec<_>>() }),
            ),
            "Target.setDiscoverTargets" => {
                let discover = params
                    .get("discover")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                self.clients.get_mut(client_id).unwrap().discover = discover;
                if discover {
                    for id in self.targets.keys().cloned().collect::<Vec<_>>() {
                        self.emit(
                            "Target.targetCreated",
                            json!({"targetInfo": self.target_info(&id)}),
                            Some(client_id),
                            None,
                        );
                    }
                }
                Ok(json!({}))
            }
            "Target.setAutoAttach" => {
                let enabled = params
                    .get("autoAttach")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                self.clients.get_mut(client_id).unwrap().auto_attach = enabled;
                if enabled {
                    let targets = self.targets.keys().cloned().collect::<Vec<_>>();
                    for target in targets {
                        let attached = self.clients[client_id]
                            .sessions
                            .values()
                            .any(|value| value == &target);
                        if !attached {
                            self.attach(client_id, &target)
                                .map_err(|error| (-32000, error))?;
                        }
                    }
                }
                Ok(json!({}))
            }
            "Target.createTarget" => {
                let url = params
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("about:blank");
                let context = params
                    .get("browserContextId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let target = self
                    .create_target(url, context)
                    .map_err(|error| (-32000, error))?;
                if self.clients[client_id].auto_attach {
                    self.attach(client_id, &target)
                        .map_err(|error| (-32000, error))?;
                }
                Ok(json!({"targetId": target}))
            }
            "Target.closeTarget" => {
                let id = params.get("targetId").and_then(Value::as_str).unwrap_or("");
                if !self.close_target(id) {
                    return Err((-32602, "Unknown targetId".into()));
                }
                Ok(json!({"success": true}))
            }
            "Target.attachToTarget" => {
                if params.get("flatten").and_then(Value::as_bool) != Some(true) {
                    return Err((-32602, "Only flattened sessions are supported".into()));
                }
                let target = params.get("targetId").and_then(Value::as_str).unwrap_or("");
                Ok(
                    json!({"sessionId": self.attach(client_id, target).map_err(|error| (-32602, error))?}),
                )
            }
            "Target.detachFromTarget" => {
                let session = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let client = self.clients.get_mut(client_id).unwrap();
                let target = client
                    .sessions
                    .remove(&session)
                    .ok_or((-32602, "Unknown sessionId".into()))?;
                client.enabled.remove(&session);
                self.targets
                    .get_mut(&target)
                    .unwrap()
                    .sessions
                    .remove(&session);
                self.emit(
                    "Target.detachedFromTarget",
                    json!({"sessionId": session, "targetId": target}),
                    Some(client_id),
                    None,
                );
                Ok(json!({}))
            }
            "Target.getBrowserContexts" => {
                Ok(json!({"browserContextIds": self.contexts.keys().collect::<Vec<_>>() }))
            }
            "Target.createBrowserContext" => {
                if params.get("verificationIdentity").is_some() {
                    return Err((-32602, "verificationIdentity is no longer supported; browser contexts are always ephemeral".into()));
                }
                let id = platform::random_token();
                self.contexts.insert(id.clone(), Context::default());
                Ok(json!({"browserContextId": id}))
            }
            "Target.disposeBrowserContext" => {
                let id = params
                    .get("browserContextId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                if !self.contexts.contains_key(&id) {
                    return Err((-32602, "Unknown browserContextId".into()));
                }
                let targets = self
                    .targets
                    .values()
                    .filter(|target| target.context_id == id)
                    .map(|target| target.id.clone())
                    .collect::<Vec<_>>();
                for target in targets {
                    self.close_target(&target);
                }
                self.release_context(&id);
                Ok(json!({}))
            }
            _ => Err((-32601, format!("'{method}' wasn't found"))),
        }
    }

    fn target_command(
        &mut self,
        client_id: &str,
        method: &str,
        params: Value,
        target_id: &str,
        routing: &str,
    ) -> Result<Value, (i64, String)> {
        const LOCAL: &[&str] = &[
            "Page.setLifecycleEventsEnabled",
            "Audits.enable",
            "Performance.enable",
            "Log.enable",
            "Emulation.setTouchEmulationEnabled",
        ];
        if LOCAL.contains(&method) {
            return Ok(json!({}));
        }
        if method == "Page.addScriptToEvaluateOnNewDocument" {
            return Ok(json!({"identifier": platform::random_token()}));
        }
        if method == "Page.createIsolatedWorld" {
            let client = self.clients.get_mut(client_id).unwrap();
            let next = client
                .next_execution_context
                .entry(routing.into())
                .or_insert(2);
            let context_id = *next;
            *next += 1;
            let context = json!({"id": context_id, "uniqueId": format!("{target_id}-{context_id}"), "origin": "", "name": params.get("worldName").and_then(Value::as_str).unwrap_or(""), "auxData": {"isDefault": false, "type": "isolated", "frameId": params.get("frameId").and_then(Value::as_str).unwrap_or("main")}});
            self.emit(
                "Runtime.executionContextCreated",
                json!({"context": context}),
                Some(client_id),
                (routing != "direct").then_some(routing),
            );
            return Ok(json!({"executionContextId": context_id}));
        }
        const FORWARDED: &[&str] = &[
            "Page.enable",
            "Page.disable",
            "Page.navigate",
            "Page.reload",
            "Page.getFrameTree",
            "Page.captureScreenshot",
            "Runtime.enable",
            "Runtime.disable",
            "Runtime.evaluate",
            "Runtime.callFunctionOn",
            "Runtime.releaseObject",
            "Network.enable",
            "Network.disable",
            "Emulation.setDeviceMetricsOverride",
            "Input.dispatchMouseEvent",
            "Input.dispatchKeyEvent",
        ];
        if !FORWARDED.contains(&method) {
            return Err((-32601, format!("'{method}' wasn't found")));
        }
        let target = self
            .targets
            .get(target_id)
            .cloned()
            .ok_or((-32602, "Unknown targetId".into()))?;
        if method == "Page.navigate" || method == "Page.reload" {
            let target = self.targets.get_mut(target_id).unwrap();
            target.loader_id = platform::random_token();
            target.navigation_id.clear();
        }
        let mut result = target
            .worker
            .request(method, params.clone(), Some(&target.worker_session))
            .map_err(|error| (-32000, error))?;
        if method == "Page.navigate" || method == "Page.reload" {
            let target = self.targets.get_mut(target_id).unwrap();
            target.navigation_id = result
                .get("navigationId")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into();
            result
                .as_object_mut()
                .map(|object| object.remove("navigationId"));
            if method == "Page.navigate" {
                result["loaderId"] = json!(target.loader_id);
                if let Some(url) = params.get("url").and_then(Value::as_str) {
                    target.url = url.into();
                }
                self.emit(
                    "Target.targetInfoChanged",
                    json!({"targetInfo": self.target_info(target_id)}),
                    None,
                    None,
                );
            }
        }
        if let Some(command) = method.split('.').nth(1) {
            let domain = method.split('.').next().unwrap();
            let client = self.clients.get_mut(client_id).unwrap();
            if command == "enable" {
                client
                    .enabled
                    .entry(routing.into())
                    .or_default()
                    .insert(domain.into());
                if method == "Runtime.enable" {
                    client.next_execution_context.insert(routing.into(), 2);
                    let context = json!({"id": 1, "uniqueId": format!("{target_id}-1"), "origin": "", "name": "", "auxData": {"isDefault": true, "type": "default", "frameId": "main"}});
                    self.emit(
                        "Runtime.executionContextCreated",
                        json!({"context": context}),
                        Some(client_id),
                        (routing != "direct").then_some(routing),
                    );
                }
            } else if command == "disable" {
                client
                    .enabled
                    .entry(routing.into())
                    .or_default()
                    .remove(domain);
            }
        }
        Ok(result)
    }

    fn worker_event(&mut self, worker: &Arc<Worker>, message: Value) {
        let worker_session = message
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(target) = self
            .targets
            .values()
            .find(|target| {
                target.worker_session == worker_session && Arc::ptr_eq(&target.worker, worker)
            })
            .cloned()
        else {
            return;
        };
        let target_id = target.id.as_str();
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let mut params = message.get("params").cloned().unwrap_or_else(|| json!({}));
        if let Some(navigation) = params.get("navigationId").and_then(Value::as_str) {
            if !target.navigation_id.is_empty() && navigation != target.navigation_id {
                return;
            }
            params
                .as_object_mut()
                .map(|object| object.remove("navigationId"));
        }
        if method == "Page.frameNavigated"
            && let Some(frame) = params.get_mut("frame")
        {
            frame["loaderId"] = json!(target.loader_id);
        }
        let domain = method.split('.').next().unwrap_or("");
        for (client_id, client) in &self.clients {
            if client.direct_target == target_id
                && client
                    .enabled
                    .get("direct")
                    .is_some_and(|set| set.contains(domain))
            {
                let _ = client
                    .sender
                    .send(json!({"method": method, "params": params}));
            }
            for (session, session_target) in &client.sessions {
                if session_target == target_id
                    && client
                        .enabled
                        .get(session)
                        .is_some_and(|set| set.contains(domain))
                {
                    let _ = client
                        .sender
                        .send(json!({"method": method, "params": params, "sessionId": session}));
                }
            }
            let _ = client_id;
        }
        let lifecycle = match method {
            "Page.frameNavigated" => Some("init"),
            "Page.domContentEventFired" => Some("DOMContentLoaded"),
            "Page.loadEventFired" => Some("load"),
            _ => None,
        };
        if let Some(name) = lifecycle {
            let lifecycle = json!({"frameId": "main", "loaderId": target.loader_id, "name": name, "timestamp": params.get("timestamp").cloned().unwrap_or(json!(0))});
            for client in self.clients.values() {
                if client.direct_target == target_id
                    && client
                        .enabled
                        .get("direct")
                        .is_some_and(|set| set.contains("Page"))
                {
                    let _ = client
                        .sender
                        .send(json!({"method": "Page.lifecycleEvent", "params": lifecycle}));
                }
                for (session, session_target) in &client.sessions {
                    if session_target == target_id
                        && client
                            .enabled
                            .get(session)
                            .is_some_and(|set| set.contains("Page"))
                    {
                        let _ = client.sender.send(json!({"method": "Page.lifecycleEvent", "params": lifecycle, "sessionId": session}));
                    }
                }
            }
        }
    }
}

pub fn run(arguments: impl Iterator<Item = String>, stop: Arc<AtomicBool>) -> Result<(), String> {
    let config = Config::parse(arguments)?;
    if config.worker_protocol == "cdp" {
        return crate::proxy::run(
            config.worker_path,
            config.worker_args,
            config.pool_size,
            config.port,
            config.operation_timeout,
            stop,
        );
    }
    let (event_tx, event_rx) = mpsc::channel();
    let mut controller = Controller {
        stop: stop.clone(),
        config: config.clone(),
        events: event_tx,
        workers: Vec::new(),
        available: Vec::new(),
        default_context: Context::default(),
        contexts: HashMap::new(),
        targets: HashMap::new(),
        clients: HashMap::new(),
        browser_id: platform::random_token(),
    };
    for _ in 0..config.pool_size {
        let worker = controller.spawn_worker()?;
        controller.available.push(worker);
    }
    controller.create_target("about:blank", "")?;
    let controller = Arc::new(Mutex::new(controller));
    let _shutdown = ControllerShutdown(controller.clone());
    let weak_controller = Arc::downgrade(&controller);
    let event_stop = stop.clone();
    thread::spawn(move || {
        while !event_stop.load(Ordering::Acquire) {
            let message = match event_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(message) => message,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => break,
            };
            let Some(event_controller) = weak_controller.upgrade() else {
                break;
            };
            match message {
                WorkerMessage::Event(worker, message) => event_controller
                    .lock()
                    .unwrap()
                    .worker_event(&worker, message),
                WorkerMessage::Failed(worker) => handle_worker_failure(&event_controller, &worker),
            }
        }
    });
    let listener =
        TcpListener::bind(("127.0.0.1", config.port)).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    println!("Brimp listening on http://127.0.0.1:{port}");
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let controller = controller.clone();
                thread::spawn(move || {
                    if let Err(error) = serve_connection(stream, controller, port) {
                        eprintln!("connection error: {error}");
                    }
                });
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    let workers = controller.lock().unwrap().workers.clone();
    for worker in workers {
        worker.fail();
    }
    Ok(())
}

struct ControllerShutdown(Arc<Mutex<Controller>>);
impl Drop for ControllerShutdown {
    fn drop(&mut self) {
        let state = self.0.lock().unwrap();
        state.stop.store(true, Ordering::Release);
        let workers = state.workers.clone();
        drop(state);
        for worker in workers {
            worker.fail();
        }
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        for worker in &self.workers {
            worker.fail();
        }
    }
}

fn handle_worker_failure(controller: &Arc<Mutex<Controller>>, worker: &Arc<Worker>) {
    let mut state = controller.lock().unwrap();
    state
        .available
        .retain(|candidate| !Arc::ptr_eq(candidate, worker));
    state
        .workers
        .retain(|candidate| !Arc::ptr_eq(candidate, worker));
    if state
        .default_context
        .worker
        .as_ref()
        .is_some_and(|candidate| Arc::ptr_eq(candidate, worker))
    {
        state.default_context.worker = None;
        state.default_context.worker_id.clear();
    }
    for context in state.contexts.values_mut() {
        if context
            .worker
            .as_ref()
            .is_some_and(|candidate| Arc::ptr_eq(candidate, worker))
        {
            context.worker = None;
            context.worker_id.clear();
        }
    }
    let failed = state
        .targets
        .values()
        .filter(|target| Arc::ptr_eq(&target.worker, worker))
        .map(|target| target.id.clone())
        .collect::<Vec<_>>();
    for id in failed {
        for client in state.clients.values_mut() {
            let detached = client
                .sessions
                .iter()
                .filter(|(_, target)| *target == &id)
                .map(|(session, _)| session.clone())
                .collect::<Vec<_>>();
            for session in detached {
                client.sessions.remove(&session);
                client.enabled.remove(&session);
                let _ = client.sender.send(json!({"method": "Target.detachedFromTarget", "params": {"sessionId": session, "targetId": id}}));
            }
        }
        state.targets.remove(&id);
        state.emit(
            "Target.targetCrashed",
            json!({"targetId": id, "status": "crashed", "errorCode": 0}),
            None,
            None,
        );
        state.emit(
            "Target.targetDestroyed",
            json!({"targetId": id}),
            None,
            None,
        );
    }
    if !state.stop.load(Ordering::Acquire)
        && let Ok(replacement) = state.spawn_worker()
    {
        state.available.push(replacement);
    }
}

#[allow(clippy::result_large_err)]
fn serve_connection(
    mut stream: TcpStream,
    controller: Arc<Mutex<Controller>>,
    port: u16,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut preview = [0_u8; 65536];
    let count = stream
        .peek(&mut preview)
        .map_err(|error| error.to_string())?;
    let request = String::from_utf8_lossy(&preview[..count]);
    let first = request.lines().next().unwrap_or("");
    let path = first.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let upgrade = request
        .lines()
        .any(|line| line.to_ascii_lowercase().starts_with("upgrade: websocket"));
    if !upgrade {
        let mut consumed = vec![0; count];
        stream
            .read_exact(&mut consumed)
            .map_err(|error| error.to_string())?;
        let body = discovery(&controller.lock().unwrap(), &path, port);
        let status = if body.is_some() {
            "200 OK"
        } else {
            "404 Not Found"
        };
        let body =
            serde_json::to_vec(&body.unwrap_or_else(|| json!({"error": "Not found"}))).unwrap();
        write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).and_then(|_| stream.write_all(&body)).map_err(|error| error.to_string())?;
        return Ok(());
    }
    let captured = Arc::new(Mutex::new(String::new()));
    let callback_path = captured.clone();
    let mut socket =
        tungstenite::accept_hdr(stream, move |request: &Request, response: Response| {
            *callback_path.lock().unwrap() = request.uri().path().to_owned();
            Ok(response)
        })
        .map_err(|error| error.to_string())?;
    let path = captured.lock().unwrap().clone();
    let direct = path
        .strip_prefix("/devtools/page/")
        .map(str::to_owned)
        .unwrap_or_default();
    let browser = controller.lock().unwrap().browser_id.clone();
    if direct.is_empty() && path != format!("/devtools/browser/{browser}") {
        return Err("unknown WebSocket route".into());
    }
    serve_websocket(&mut socket, controller, direct)
}

fn discovery(controller: &Controller, path: &str, port: u16) -> Option<Value> {
    if path == "/json/version" || path == "/json/version/" {
        return Some(
            json!({"Browser": PRODUCT, "Protocol-Version": CDP_VERSION, "User-Agent": PRODUCT, "V8-Version": "", "WebKit-Version": "", "webSocketDebuggerUrl": format!("ws://127.0.0.1:{port}/devtools/browser/{}", controller.browser_id)}),
        );
    }
    if path == "/json" || path == "/json/list" || path == "/json/list/" {
        return Some(Value::Array(controller.targets.values().map(|target| json!({"id": target.id, "type": "page", "title": "", "url": target.url, "webSocketDebuggerUrl": format!("ws://127.0.0.1:{port}/devtools/page/{}", target.id)})).collect()));
    }
    None
}

fn serve_websocket(
    socket: &mut WebSocket<TcpStream>,
    controller: Arc<Mutex<Controller>>,
    direct: String,
) -> Result<(), String> {
    socket
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| error.to_string())?;
    let (sender, receiver) = mpsc::channel();
    let client_id = platform::random_token();
    controller.lock().unwrap().clients.insert(
        client_id.clone(),
        Client {
            sender,
            discover: false,
            auto_attach: false,
            direct_target: direct,
            sessions: HashMap::new(),
            enabled: HashMap::new(),
            next_execution_context: HashMap::new(),
        },
    );
    while !controller.lock().unwrap().stop.load(Ordering::Acquire) {
        while let Ok(event) = receiver.try_recv() {
            socket
                .send(Message::Text(event.to_string().into()))
                .map_err(|error| error.to_string())?;
        }
        match socket.read() {
            Ok(Message::Text(text)) => {
                let request: Value =
                    serde_json::from_str(&text).map_err(|error| error.to_string())?;
                let id = request.get("id").cloned().unwrap_or(Value::Null);
                let method = request.get("method").and_then(Value::as_str).unwrap_or("");
                let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
                let session = request.get("sessionId").and_then(Value::as_str);
                let result = controller
                    .lock()
                    .unwrap()
                    .command(&client_id, method, params, session);
                let response = match result {
                    Ok(result) => json!({"id": id, "result": result}),
                    Err((code, message)) => {
                        json!({"id": id, "error": {"code": code, "message": message}})
                    }
                };
                let mut response = response;
                if let Some(session) = session {
                    response["sessionId"] = json!(session)
                }
                socket
                    .send(Message::Text(response.to_string().into()))
                    .map_err(|error| error.to_string())?;
            }
            Ok(Message::Ping(data)) => socket
                .send(Message::Pong(data))
                .map_err(|error| error.to_string())?,
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(tungstenite::Error::ConnectionClosed) => break,
            Err(error) => return Err(error.to_string()),
        }
    }
    let mut state = controller.lock().unwrap();
    if let Some(client) = state.clients.remove(&client_id) {
        for (session, target) in client.sessions {
            if let Some(target) = state.targets.get_mut(&target) {
                target.sessions.remove(&session);
            }
        }
    }
    Ok(())
}
