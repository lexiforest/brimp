use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};
use web_runtime::AutomationBrowser;

use crate::dispatch::ConnectionState;
use crate::protocol::{Request, Response};

const MAX_HTTP_HEADER: usize = 16 * 1024;
const MAX_MESSAGE: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    pub allow_non_loopback: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from(([127, 0, 0, 1], 0)),
            allow_non_loopback: false,
        }
    }
}

pub struct ServerHandle {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), ServerError>>,
}

impl ServerHandle {
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
    pub fn browser_websocket_url(&self) -> String {
        format!("ws://{}/devtools/browser/brimp", self.addr)
    }
    pub async fn shutdown(mut self) -> Result<(), ServerError> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task
            .await
            .map_err(|error| ServerError::Task(error.to_string()))?
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("refusing non-loopback CDP bind {0} without explicit permission")]
    NonLoopback(SocketAddr),
    #[error("failed to create browser: {0}")]
    Browser(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("server task failed: {0}")]
    Task(String),
}

pub async fn start(config: ServerConfig) -> Result<ServerHandle, ServerError> {
    let browser = Arc::new(
        AutomationBrowser::new().map_err(|error| ServerError::Browser(error.to_string()))?,
    );
    start_with_browser(config, browser).await
}

pub async fn start_with_browser(
    config: ServerConfig,
    browser: Arc<AutomationBrowser>,
) -> Result<ServerHandle, ServerError> {
    if !config.bind.ip().is_loopback() && !config.allow_non_loopback {
        return Err(ServerError::NonLoopback(config.bind));
    }
    if !config.bind.ip().is_loopback() {
        eprintln!(
            "WARNING: Brimp CDP is binding to non-loopback address {}; any reachable client can control the browser",
            config.bind
        );
    }
    let listener = TcpListener::bind(config.bind).await?;
    let addr = listener.local_addr()?;
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let (stream, _) = accepted?;
                    let browser = Arc::clone(&browser);
                    tokio::spawn(async move { let _ = handle_connection(stream, addr, browser).await; });
                }
            }
        }
        Ok(())
    });
    Ok(ServerHandle {
        addr,
        shutdown: Some(shutdown_tx),
        task,
    })
}

async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    browser: Arc<AutomationBrowser>,
) -> Result<(), WebSocketError> {
    let mut header = vec![0; MAX_HTTP_HEADER];
    loop {
        let count = stream.peek(&mut header).await.map_err(WebSocketError::Io)?;
        if count == 0 {
            return Ok(());
        }
        let complete = header[..count]
            .windows(4)
            .any(|window| window == b"\r\n\r\n");
        if complete {
            header.truncate(count);
            break;
        }
        if count == MAX_HTTP_HEADER {
            write_http(
                &mut stream,
                "431 Request Header Fields Too Large",
                "text/plain",
                b"request headers too large",
            )
            .await
            .map_err(WebSocketError::Io)?;
            return Ok(());
        }
        tokio::task::yield_now().await;
    }
    let request = String::from_utf8_lossy(&header);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let upgrade = request
        .lines()
        .any(|line| line.eq_ignore_ascii_case("upgrade: websocket"));
    if !upgrade {
        let mut consumed = vec![0; header.len()];
        stream
            .read_exact(&mut consumed)
            .await
            .map_err(WebSocketError::Io)?;
        serve_discovery(&mut stream, addr, path)
            .await
            .map_err(WebSocketError::Io)?;
        return Ok(());
    }
    if path != "/devtools/browser/brimp" {
        write_http(&mut stream, "404 Not Found", "text/plain", b"not found")
            .await
            .map_err(WebSocketError::Io)?;
        return Ok(());
    }
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE))
        .max_frame_size(Some(MAX_MESSAGE));
    let socket = tokio_tungstenite::accept_async_with_config(stream, Some(config)).await?;
    serve_websocket(socket, browser).await
}

async fn serve_discovery(
    stream: &mut TcpStream,
    addr: SocketAddr,
    path: &str,
) -> std::io::Result<()> {
    let websocket = format!("ws://{addr}/devtools/browser/brimp");
    let body = match path {
        "/json/version" => serde_json::to_vec(&json!({
            "Browser": "Brimp/0.1.0",
            "Protocol-Version": "1.3",
            "User-Agent": "Brimp/0.1.0",
            "V8-Version": "JavaScriptCore",
            "webSocketDebuggerUrl": websocket
        }))
        .unwrap(),
        "/json" | "/json/list" => b"[]".to_vec(),
        _ => {
            write_http(stream, "404 Not Found", "text/plain", b"not found").await?;
            return Ok(());
        }
    };
    write_http(stream, "200 OK", "application/json", &body).await
}

async fn write_http(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.shutdown().await
}

async fn serve_websocket(
    mut socket: tokio_tungstenite::WebSocketStream<TcpStream>,
    browser: Arc<AutomationBrowser>,
) -> Result<(), WebSocketError> {
    let mut state = ConnectionState::new(browser);
    while let Some(message) = socket.next().await {
        match message? {
            Message::Text(text) => {
                let (response, events_before_response) =
                    match serde_json::from_str::<Request>(&text) {
                        Ok(request) => {
                            let events_before_response = request.method == "Target.attachToTarget";
                            (state.dispatch(&request).await, events_before_response)
                        }
                        Err(error) => (
                            Response {
                                id: 0,
                                result: None,
                                error: Some(crate::protocol::ProtocolError {
                                    code: -32700,
                                    message: format!("Parse error: {error}"),
                                }),
                                session_id: None,
                            },
                            false,
                        ),
                    };
                if events_before_response && response.error.is_none() {
                    for event in state.take_events() {
                        socket
                            .send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                            .await?;
                    }
                }
                socket
                    .send(Message::Text(
                        serde_json::to_string(&response).unwrap().into(),
                    ))
                    .await?;
                for event in state.take_events() {
                    socket
                        .send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                        .await?;
                }
            }
            Message::Ping(payload) => socket.send(Message::Pong(payload)).await?,
            Message::Close(frame) => {
                socket.close(frame).await?;
                break;
            }
            Message::Binary(_) => {
                socket.close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame { code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Unsupported, reason: "CDP requires JSON text messages".into() })).await?;
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn parse_bind(value: &str) -> Result<SocketAddr, String> {
    value
        .parse()
        .map_err(|error| format!("invalid bind address {value}: {error}"))
}
