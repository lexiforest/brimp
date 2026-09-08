use crate::platform::{self, WorkerProcess};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::WebSocket;
use tungstenite::handshake::server::{Request, Response};

struct Worker {
    process: WorkerProcess,
    port: u16,
    browser_path: String,
    version: Value,
}

struct Lease {
    worker: Arc<Worker>,
    routes: HashMap<String, String>,
    expires: Instant,
}

struct State {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    executable: String,
    arguments: Vec<String>,
    timeout: Duration,
    workers: Vec<Arc<Worker>>,
    available: Vec<Arc<Worker>>,
    leases: HashMap<String, Lease>,
}

impl State {
    fn spawn_worker(&mut self) -> Result<Arc<Worker>, String> {
        if self.stop.load(std::sync::atomic::Ordering::Acquire) {
            return Err("controller was cancelled".into());
        }
        let port = unused_port()?;
        let mut supplied = false;
        let mut arguments = self
            .arguments
            .iter()
            .map(|argument| {
                if argument.contains("{port}") {
                    supplied = true;
                    argument.replace("{port}", &port.to_string())
                } else {
                    argument.clone()
                }
            })
            .collect::<Vec<_>>();
        if !supplied {
            arguments.push(format!("--remote-debugging-port={port}"));
        }
        let process = platform::spawn_process(&self.executable, &arguments)
            .map_err(|error| error.to_string())?;
        let deadline = Instant::now() + self.timeout;
        while Instant::now() < deadline
            && process.running()
            && !self.stop.load(std::sync::atomic::Ordering::Acquire)
        {
            if let Ok(version) = http_json(port, "/json/version")
                && let Some(url) = version.get("webSocketDebuggerUrl").and_then(Value::as_str)
            {
                let browser_path = url_path(url).to_owned();
                let worker = Arc::new(Worker {
                    process,
                    port,
                    browser_path,
                    version,
                });
                self.workers.push(worker.clone());
                return Ok(worker);
            }
            thread::sleep(Duration::from_millis(50));
        }
        process.terminate();
        Err("worker did not publish a valid /json/version endpoint".into())
    }

    fn add_worker(&mut self) -> Result<(), String> {
        let worker = self.spawn_worker()?;
        self.available.push(worker);
        Ok(())
    }

    fn lease(&mut self) -> Result<(String, Arc<Worker>), String> {
        self.expire();
        let worker = self
            .available
            .pop()
            .ok_or("Brimp CDP worker pool is full")?;
        let token = platform::random_token();
        self.leases.insert(
            token.clone(),
            Lease {
                worker: worker.clone(),
                routes: HashMap::new(),
                expires: Instant::now() + Duration::from_secs(10),
            },
        );
        Ok((token, worker))
    }

    fn retire(&mut self, token: &str) {
        let Some(lease) = self.leases.remove(token) else {
            return;
        };
        lease.worker.process.terminate();
        self.workers
            .retain(|worker| !Arc::ptr_eq(worker, &lease.worker));
        if let Err(error) = self.add_worker() {
            eprintln!("failed to replace CDP worker: {error}");
        }
    }

    fn expire(&mut self) {
        let now = Instant::now();
        let expired = self
            .leases
            .iter()
            .filter(|(_, lease)| lease.expires <= now)
            .map(|(token, _)| token.clone())
            .collect::<Vec<_>>();
        for token in expired {
            self.retire(&token);
        }
    }
}

struct ProxyShutdown(Arc<Mutex<State>>);
impl Drop for ProxyShutdown {
    fn drop(&mut self) {
        let state = self.0.lock().unwrap();
        state.stop.store(true, std::sync::atomic::Ordering::Release);
        let workers = state.workers.clone();
        drop(state);
        for worker in workers {
            worker.process.terminate();
        }
    }
}
impl Drop for State {
    fn drop(&mut self) {
        for worker in &self.workers {
            worker.process.terminate();
        }
    }
}

pub fn run(
    worker_path: String,
    worker_args: Vec<String>,
    pool_size: usize,
    port: u16,
    timeout: Duration,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    let mut state = State {
        stop: stop.clone(),
        executable: worker_path,
        arguments: worker_args,
        timeout,
        workers: Vec::new(),
        available: Vec::new(),
        leases: HashMap::new(),
    };
    for _ in 0..pool_size {
        state.add_worker()?;
    }
    let state = Arc::new(Mutex::new(state));
    let _shutdown = ProxyShutdown(state.clone());
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    println!("Brimp CDP proxy listening on http://127.0.0.1:{port}");
    while !stop.load(std::sync::atomic::Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let state = state.clone();
                thread::spawn(move || {
                    if let Err(error) = serve(stream, state, port) {
                        eprintln!("proxy connection error: {error}");
                    }
                });
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    let workers = state.lock().unwrap().workers.clone();
    for worker in workers {
        worker.process.terminate();
    }
    Ok(())
}

#[allow(clippy::result_large_err)]
fn serve(
    mut stream: TcpStream,
    state: Arc<Mutex<State>>,
    controller_port: u16,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut preview = [0_u8; 65536];
    let count = stream
        .peek(&mut preview)
        .map_err(|error| error.to_string())?;
    let request = String::from_utf8_lossy(&preview[..count]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_owned();
    if !request
        .lines()
        .any(|line| line.to_ascii_lowercase().starts_with("upgrade: websocket"))
    {
        let mut consumed = vec![0; count];
        stream
            .read_exact(&mut consumed)
            .map_err(|error| error.to_string())?;
        let body = proxy_discovery(&state, &path, controller_port)?;
        let encoded = serde_json::to_vec(&body).map_err(|error| error.to_string())?;
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", encoded.len()).and_then(|_| stream.write_all(&encoded)).map_err(|error| error.to_string())?;
        return Ok(());
    }
    let captured = Arc::new(Mutex::new(String::new()));
    let callback_path = captured.clone();
    let frontend = tungstenite::accept_hdr(stream, move |request: &Request, response: Response| {
        *callback_path.lock().unwrap() = request.uri().path().to_owned();
        Ok(response)
    })
    .map_err(|error| error.to_string())?;
    let route = captured.lock().unwrap().clone();
    let mut parts = route.trim_start_matches('/').split('/');
    if parts.next() != Some("devtools") {
        return Err("unknown proxy route".into());
    }
    let _kind = parts.next();
    let token = parts.next().ok_or("missing lease token")?.to_owned();
    let (worker, backend_path) = {
        let mut state = state.lock().unwrap();
        let lease = state
            .leases
            .get_mut(&token)
            .ok_or("unknown or expired lease")?;
        lease.expires = Instant::now() + Duration::from_secs(10);
        let path = lease
            .routes
            .get(&route)
            .cloned()
            .unwrap_or_else(|| lease.worker.browser_path.clone());
        (lease.worker.clone(), path)
    };
    let backend_stream =
        TcpStream::connect(("127.0.0.1", worker.port)).map_err(|error| error.to_string())?;
    let url = format!("ws://127.0.0.1:{}{}", worker.port, backend_path);
    let (backend, _) =
        tungstenite::client(url, backend_stream).map_err(|error| error.to_string())?;
    bridge(frontend, backend);
    state.lock().unwrap().retire(&token);
    Ok(())
}

fn proxy_discovery(state: &Arc<Mutex<State>>, path: &str, port: u16) -> Result<Value, String> {
    let mut state = state.lock().unwrap();
    let (token, worker) = state.lease()?;
    if path == "/json/version" || path == "/json/version/" {
        let mut version = worker.version.clone();
        version["webSocketDebuggerUrl"] =
            json!(format!("ws://127.0.0.1:{port}/devtools/browser/{token}"));
        return Ok(version);
    }
    if path == "/json" || path == "/json/list" || path == "/json/list/" {
        let mut targets = http_json(worker.port, "/json/list")?;
        for target in targets
            .as_array_mut()
            .ok_or("worker returned invalid /json/list")?
        {
            let backend = target
                .get("webSocketDebuggerUrl")
                .and_then(Value::as_str)
                .map(url_path)
                .unwrap_or("/")
                .to_owned();
            let id = target.get("id").and_then(Value::as_str).unwrap_or("page");
            let route = format!("/devtools/page/{token}/{id}");
            state
                .leases
                .get_mut(&token)
                .unwrap()
                .routes
                .insert(route.clone(), backend);
            target["webSocketDebuggerUrl"] = json!(format!("ws://127.0.0.1:{port}{route}"));
        }
        return Ok(targets);
    }
    state.retire(&token);
    Ok(json!({"error": "Not found"}))
}

fn bridge(mut frontend: WebSocket<TcpStream>, mut backend: WebSocket<TcpStream>) {
    let _ = frontend
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(50)));
    let _ = backend
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(50)));
    loop {
        let mut progressed = false;
        match frontend.read() {
            Ok(message) => {
                progressed = true;
                if message.is_close() || backend.send(message).is_err() {
                    break;
                }
            }
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => break,
        }
        match backend.read() {
            Ok(message) => {
                progressed = true;
                if message.is_close() || frontend.send(message).is_err() {
                    break;
                }
            }
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => break,
        }
        if !progressed {
            thread::yield_now();
        }
    }
    let _ = frontend.close(None);
    let _ = backend.close(None);
    let _ = frontend.get_mut().shutdown(Shutdown::Both);
}

fn unused_port() -> Result<u16, String> {
    TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(|error| error.to_string())
}

fn http_json(port: u16, path: &str) -> Result<Value, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| error.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|error| error.to_string())?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| error.to_string())?;
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("invalid HTTP response")?
        + 4;
    serde_json::from_slice(&response[split..]).map_err(|error| error.to_string())
}

fn url_path(url: &str) -> &str {
    url.find("://")
        .and_then(|scheme| url[scheme + 3..].find('/').map(|slash| scheme + 3 + slash))
        .map(|slash| &url[slash..])
        .unwrap_or("/")
}
