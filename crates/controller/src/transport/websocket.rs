use super::{Transport, closed, invalid};
use brimp_protocol::MAX_FRAME_SIZE;
use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tungstenite::{Message, WebSocket, protocol::WebSocketConfig};

pub struct WebSocketTransport {
    socket: Mutex<Option<WebSocket<TcpStream>>>,
}
impl WebSocketTransport {
    /// Connect to a `ws://` CDP endpoint. TLS endpoints are not enabled in this build.
    pub fn connect(
        endpoint: &str,
        timeout: Duration,
        cancellation: &crate::CancellationToken,
    ) -> io::Result<Self> {
        if cancellation.is_cancelled() {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let url = url::Url::parse(endpoint).map_err(|error| invalid(&error.to_string()))?;
        if url.scheme() != "ws" {
            return Err(invalid("expected a ws:// CDP endpoint"));
        }
        let host = url
            .host_str()
            .ok_or_else(|| invalid("WebSocket URL has no host"))?;
        let deadline = Instant::now() + timeout;
        let mut last_error = invalid("WebSocket host has no addresses");
        for address in (host, url.port_or_known_default().unwrap_or(80)).to_socket_addrs()? {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|time| !time.is_zero())
                .ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
            let stream = match TcpStream::connect_timeout(&address, remaining) {
                Ok(stream) => stream,
                Err(error) => {
                    last_error = error;
                    continue;
                }
            };
            stream.set_nonblocking(true)?;
            let config = WebSocketConfig::default()
                .max_message_size(Some(MAX_FRAME_SIZE))
                .max_frame_size(Some(MAX_FRAME_SIZE))
                .max_write_buffer_size(MAX_FRAME_SIZE + 1024);
            let mut handshake =
                tungstenite::client::client_with_config(endpoint, stream, Some(config));
            let socket = loop {
                if cancellation.is_cancelled() {
                    return Err(io::ErrorKind::Interrupted.into());
                }
                if Instant::now() >= deadline {
                    return Err(io::ErrorKind::TimedOut.into());
                }
                match handshake {
                    Ok((socket, _)) => break socket,
                    Err(tungstenite::HandshakeError::Failure(error)) => {
                        return Err(invalid(&error.to_string()));
                    }
                    Err(tungstenite::HandshakeError::Interrupted(pending)) => {
                        std::thread::sleep(Duration::from_millis(10));
                        handshake = pending.handshake();
                    }
                }
            };
            return Ok(Self {
                socket: Mutex::new(Some(socket)),
            });
        }
        Err(last_error)
    }
}
fn flushed(result: tungstenite::Result<()>) -> io::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(tungstenite::Error::Io(error)) if error.kind() == io::ErrorKind::WouldBlock => Ok(()),
        Err(error) => Err(io::Error::other(error.to_string())),
    }
}
impl Transport for WebSocketTransport {
    fn send(&self, message: &[u8]) -> io::Result<()> {
        if message.len() > MAX_FRAME_SIZE {
            return Err(invalid("message exceeds 64 MiB"));
        }
        let text = std::str::from_utf8(message).map_err(|error| invalid(&error.to_string()))?;
        let mut guard = self.socket.lock().unwrap();
        let socket = guard.as_mut().ok_or_else(closed)?;
        flushed(socket.send(Message::Text(text.to_owned().into())))
    }
    fn receive(&self) -> io::Result<Option<Vec<u8>>> {
        let mut guard = self.socket.lock().unwrap();
        let socket = guard.as_mut().ok_or_else(closed)?;
        flushed(socket.flush())?;
        match socket.read() {
            Ok(Message::Text(text)) => Ok(Some(text.as_bytes().to_vec())),
            Ok(Message::Close(_)) => {
                let _ = socket.flush();
                Err(closed())
            }
            Ok(Message::Ping(_) | Message::Pong(_)) => Ok(None),
            Ok(_) => Err(invalid("CDP WebSocket messages must be text")),
            Err(tungstenite::Error::Io(error)) if error.kind() == io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(error) => Err(io::Error::other(error.to_string())),
        }
    }
    fn close(&self) {
        if let Some(mut socket) = self.socket.lock().unwrap().take() {
            let _ = socket.close(None);
        }
    }
}
