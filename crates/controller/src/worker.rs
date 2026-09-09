//! Owned WebSocket CDP worker startup, shared by the browser API and server pool.
use crate::AutomationError as Error;
use crate::platform::{self, WorkerProcess};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

pub(crate) struct CdpWorker {
    pub process: WorkerProcess,
    pub port: u16,
    pub browser_path: String,
    pub version: Value,
    // Keep the profile alive until the process has been terminated and reaped.
    _profile: tempfile::TempDir,
}
impl CdpWorker {
    pub fn spawn(
        path: &str,
        arguments: &[String],
        timeout: Duration,
        stop: Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        let deadline = Instant::now() + timeout;
        check(deadline, timeout, &stop)?;
        for argument in arguments {
            if argument.starts_with("--user-data-dir")
                || argument.starts_with("--remote-debugging-address")
                || argument.starts_with("--remote-debugging-pipe")
                || (argument.starts_with("--remote-debugging-port")
                    && argument != "--remote-debugging-port={port}")
            {
                return Err(Error::InvalidInput(
                    "worker profile and debugging endpoint are managed by the controller".into(),
                ));
            }
        }
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(transport_error)?;
        let port = listener.local_addr().map_err(transport_error)?.port();
        let profile = tempfile::Builder::new()
            .prefix("brimp-cdp-")
            .tempdir()
            .map_err(transport_error)?;
        let mut supplied_port = false;
        let mut args: Vec<String> = arguments
            .iter()
            .map(|argument| {
                supplied_port |= argument.contains("{port}");
                argument.replace("{port}", &port.to_string())
            })
            .collect();
        if !supplied_port {
            args.push(format!("--remote-debugging-port={port}"));
        }
        args.push(format!("--user-data-dir={}", profile.path().display()));
        if !args
            .iter()
            .any(|arg| arg == "--headless" || arg.starts_with("--headless="))
        {
            args.push("--headless".into());
        }
        drop(listener);
        let process = platform::spawn_process(path, &args).map_err(transport_error)?;
        loop {
            check(deadline, timeout, &stop)?;
            if !process.running() {
                return Err(Error::Transport("CDP worker exited during startup".into()));
            }
            let remaining = deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(100));
            if let Ok(version) = http_json(port, "/json/version", remaining)
                && let Some(endpoint) = version.get("webSocketDebuggerUrl").and_then(Value::as_str)
                && let Ok(url) = url::Url::parse(endpoint)
                && url.scheme() == "ws"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
                && url.port_or_known_default() == Some(port)
                && url.path().starts_with("/devtools/browser/")
            {
                check(deadline, timeout, &stop)?;
                // Always connect to the loopback port allocated for this child.
                let browser_path = format!(
                    "{}{}",
                    url.path(),
                    url.query().map(|q| format!("?{q}")).unwrap_or_default()
                );
                return Ok(Self {
                    process,
                    port,
                    browser_path,
                    version,
                    _profile: profile,
                });
            }
            std::thread::sleep(
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(10)),
            );
        }
    }
    pub fn endpoint(&self) -> String {
        format!("ws://127.0.0.1:{}{}", self.port, self.browser_path)
    }
}

fn check(deadline: Instant, timeout: Duration, stop: &AtomicBool) -> Result<(), Error> {
    if stop.load(Ordering::Acquire) {
        return Err(Error::Cancellation);
    }
    if Instant::now() >= deadline {
        return Err(Error::Timeout(timeout));
    }
    Ok(())
}
fn transport_error(error: std::io::Error) -> Error {
    Error::Transport(error.to_string())
}

pub(crate) fn http_json(port: u16, path: &str, timeout: Duration) -> Result<Value, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, timeout).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + timeout;
    let mut bytes = Vec::new();
    let mut expected = None;
    loop {
        if expected.is_none() {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut response = httparse::Response::new(&mut headers);
            if let httparse::Status::Complete(header_length) =
                response.parse(&bytes).map_err(|e| e.to_string())?
            {
                if response.code != Some(200) {
                    return Err("worker discovery did not return HTTP 200".into());
                }
                let length = response
                    .headers
                    .iter()
                    .find(|header| header.name.eq_ignore_ascii_case("content-length"))
                    .ok_or("worker discovery has no Content-Length")?;
                let length = std::str::from_utf8(length.value)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?;
                if length > 1024 * 1024 {
                    return Err("worker discovery response is too large".into());
                }
                expected = Some((header_length, length));
            }
        }
        if let Some((header, length)) = expected
            && bytes.len() >= header + length
        {
            return serde_json::from_slice(&bytes[header..header + length])
                .map_err(|e| e.to_string());
        }
        if bytes.len() >= 1024 * 1024 {
            return Err("worker discovery response is too large".into());
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|time| !time.is_zero())
            .ok_or("worker discovery timed out")?;
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        let mut buffer = [0; 8192];
        let count = stream.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("truncated worker discovery response".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}
