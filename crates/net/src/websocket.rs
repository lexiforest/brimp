use std::ffi::{CString, c_long, c_void};
use std::ptr;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::time::Duration;

use crate::ffi::*;
use crate::{Config as CurlConfig, Error as NetworkError};
use http::HeaderMap as HeaderList;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Socket handshake, message and lifecycle observations.
pub enum WebSocketEvent {
    /// HandshakeRequest observation.
    HandshakeRequest {
        /// headers data.
        headers: Vec<(String, String)>,
    },
    /// HandshakeResponse observation.
    HandshakeResponse {
        /// status data.
        status: u16,
        /// headers data.
        headers: Vec<(String, String)>,
    },
    /// FrameSent observation.
    FrameSent {
        /// opcode data.
        opcode: u8,
        /// payload data.
        payload: Vec<u8>,
    },
    /// Open observation.
    Open,
    /// Text observation.
    Text(String),
    /// Binary observation.
    Binary(Vec<u8>),
    /// Close observation.
    Close {
        /// code data.
        code: u16,
        /// reason data.
        reason: String,
    },
    /// Error observation.
    Error(String),
}

enum Command {
    Text(String),
    Binary(Vec<u8>),
    Close,
}

/// Owned socket connection; dropping it requests closure.
pub struct WebSocketHandle {
    sender: Sender<Command>,
}

impl WebSocketHandle {
    /// Queue a text message.
    pub fn send_text(&self, message: impl Into<String>) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Text(message.into()))
            .map_err(|_| NetworkError::Network("socket closed".into()))
    }

    /// Queue a binary message.
    pub fn send_binary(&self, message: Vec<u8>) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Binary(message))
            .map_err(|_| NetworkError::Network("socket closed".into()))
    }

    /// Request a normal close handshake.
    pub fn close(&self) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Close)
            .map_err(|_| NetworkError::Network("socket closed".into()))
    }
}

impl Drop for WebSocketHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Close);
    }
}

/// Open a socket on its dedicated transport thread.
pub fn open_websocket(
    config: CurlConfig,
    url: String,
    headers: HeaderList,
    callback: Box<dyn Fn(WebSocketEvent) + Send>,
) -> Result<WebSocketHandle, NetworkError> {
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("brimp-curl-websocket".into())
        .spawn(move || {
            if let Err(error) = run(config, &url, headers, &receiver, callback.as_ref()) {
                callback(WebSocketEvent::Error(error.to_string()));
            }
        })
        .map_err(|error| NetworkError::Initialization(error.to_string()))?;
    Ok(WebSocketHandle { sender })
}

fn parse_headers(bytes: &[u8]) -> Vec<(String, String)> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}
struct DebugHeaders<'a> {
    callback: &'a (dyn Fn(WebSocketEvent) + Send),
}
unsafe extern "C" fn debug_headers(
    _curl: *mut Curl,
    kind: i32,
    data: *mut std::ffi::c_char,
    size: usize,
    user: *mut c_void,
) -> i32 {
    if kind == 2 && size <= 1024 * 1024 {
        // libcurl keeps both pointers valid for the duration of this callback.
        let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) };
        let debug = unsafe { &*(user.cast::<DebugHeaders<'_>>()) };
        (debug.callback)(WebSocketEvent::HandshakeRequest {
            headers: parse_headers(bytes),
        });
    }
    0
}
unsafe extern "C" fn response_headers(
    data: *mut std::ffi::c_char,
    size: usize,
    count: usize,
    user: *mut c_void,
) -> usize {
    let Some(len) = size.checked_mul(count) else {
        return 0;
    };
    // libcurl owns the input and the run function owns the output vector.
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) };
    let headers = unsafe { &mut *user.cast::<Vec<u8>>() };
    if bytes.starts_with(b"HTTP/") {
        headers.clear();
    }
    if headers.len() + len > 1024 * 1024 {
        return 0;
    }
    headers.extend_from_slice(bytes);
    len
}

fn run(
    config: CurlConfig,
    url: &str,
    headers: HeaderList,
    receiver: &mpsc::Receiver<Command>,
    callback: &(dyn Fn(WebSocketEvent) + Send),
) -> Result<(), NetworkError> {
    global_init();
    let handle = unsafe { curl_easy_init() };
    if handle.is_null() {
        return Err(NetworkError::Network("failed to create curl handle".into()));
    }
    struct Easy(*mut Curl);
    impl Drop for Easy {
        fn drop(&mut self) {
            unsafe { curl_easy_cleanup(self.0) };
        }
    }
    let easy = Easy(handle);
    let url = CString::new(url).map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
    let profile = CString::new(config.impersonation_target)
        .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
    let mut header_list: *mut CurlSlist = ptr::null_mut();
    for (name, value) in headers.iter() {
        let value = value
            .to_str()
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let line = CString::new(format!("{}: {value}", name.as_str()))
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        header_list = unsafe { curl_slist_append(header_list, line.as_ptr()) };
    }
    struct Headers(*mut CurlSlist);
    impl Drop for Headers {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { curl_slist_free_all(self.0) }
            }
        }
    }
    let header_list = Headers(header_list);
    let impersonate =
        unsafe { curl_easy_impersonate(easy.0, profile.as_ptr(), config.default_headers as i32) };
    if impersonate != CURLE_OK {
        return Err(NetworkError::Network(curl_error(easy.0, impersonate)));
    }
    let mut incoming_headers = Vec::<u8>::new();
    let mut debug = DebugHeaders { callback };
    unsafe {
        setopt(easy.0, CURLOPT_VERBOSE, 1 as c_long)?; // CURLOPT_VERBOSE routes only to our debug callback.
        setopt(
            easy.0,
            CURLOPT_DEBUGFUNCTION,
            debug_headers as *const c_void,
        )?;
        setopt(
            easy.0,
            CURLOPT_DEBUGDATA,
            (&mut debug as *mut DebugHeaders<'_>).cast::<c_void>(),
        )?;
        setopt(
            easy.0,
            CURLOPT_HEADERFUNCTION,
            response_headers as *const c_void,
        )?;
        setopt(
            easy.0,
            CURLOPT_HEADERDATA,
            (&mut incoming_headers as *mut Vec<u8>).cast::<c_void>(),
        )?;
        setopt(easy.0, CURLOPT_URL, url.as_ptr())?;
        setopt(easy.0, CURLOPT_CONNECT_ONLY, 2 as c_long)?;
        setopt(
            easy.0,
            CURLOPT_CONNECTTIMEOUT_MS,
            config.connect_timeout.as_millis() as c_long,
        )?;
        if !header_list.0.is_null() {
            setopt(easy.0, CURLOPT_HTTPHEADER, header_list.0)?;
        }
        let proxy = config
            .proxy
            .as_ref()
            .map(|proxy| CString::new(proxy.url()).map(|value| (value, proxy.curl_proxy_type())))
            .transpose()
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        if let Some((proxy, kind)) = &proxy {
            setopt(easy.0, CURLOPT_PROXY, proxy.as_ptr())?;
            setopt(easy.0, CURLOPT_PROXYTYPE, *kind as c_long)?;
        }
        if let Some(path) = config.ca_bundle {
            let path = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
            setopt(easy.0, CURLOPT_CAINFO, path.as_ptr())?;
            let code = curl_easy_perform(easy.0);
            if code != CURLE_OK {
                return Err(NetworkError::Network(curl_error(handle, code)));
            }
        } else {
            let code = curl_easy_perform(easy.0);
            if code != CURLE_OK {
                return Err(NetworkError::Network(curl_error(handle, code)));
            }
        }
    }
    let status = String::from_utf8_lossy(&incoming_headers)
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    callback(WebSocketEvent::HandshakeResponse {
        status,
        headers: parse_headers(&incoming_headers),
    });
    callback(WebSocketEvent::Open);
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        match receiver.try_recv() {
            Ok(Command::Text(message)) => {
                send(easy.0, message.as_bytes(), CURLWS_TEXT)?;
                callback(WebSocketEvent::FrameSent {
                    opcode: 1,
                    payload: message.into_bytes(),
                });
            }
            Ok(Command::Binary(message)) => {
                send(easy.0, &message, CURLWS_BINARY)?;
                callback(WebSocketEvent::FrameSent {
                    opcode: 2,
                    payload: message,
                });
            }
            Ok(Command::Close) | Err(TryRecvError::Disconnected) => {
                let _ = send(easy.0, &1000u16.to_be_bytes(), CURLWS_CLOSE);
                callback(WebSocketEvent::Close {
                    code: 1000,
                    reason: String::new(),
                });
                return Ok(());
            }
            Err(TryRecvError::Empty) => {}
        }
        let mut received = 0usize;
        let mut metadata: *const CurlWsFrame = ptr::null();
        let code = unsafe {
            curl_ws_recv(
                easy.0,
                buffer.as_mut_ptr().cast::<c_void>(),
                buffer.len(),
                &mut received,
                &mut metadata,
            )
        };
        if code == CURLE_AGAIN {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        if code != CURLE_OK {
            return Err(NetworkError::Network(curl_error(handle, code)));
        }
        if metadata.is_null() {
            continue;
        }
        let flags = unsafe { (*metadata).flags } as u32;
        let bytes = &buffer[..received];
        if flags & CURLWS_TEXT != 0 {
            callback(WebSocketEvent::Text(
                String::from_utf8_lossy(bytes).into_owned(),
            ));
        } else if flags & CURLWS_BINARY != 0 {
            callback(WebSocketEvent::Binary(bytes.to_vec()));
        } else if flags & CURLWS_CLOSE != 0 {
            let code = bytes
                .get(..2)
                .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
                .unwrap_or(1005);
            let reason = String::from_utf8_lossy(bytes.get(2..).unwrap_or_default()).into_owned();
            callback(WebSocketEvent::Close { code, reason });
            return Ok(());
        }
    }
}

fn send(handle: *mut Curl, bytes: &[u8], flags: u32) -> Result<(), NetworkError> {
    let mut offset = 0;
    while offset < bytes.len() {
        let mut sent = 0usize;
        let code = unsafe {
            curl_ws_send(
                handle,
                bytes[offset..].as_ptr().cast::<c_void>(),
                bytes.len() - offset,
                &mut sent,
                0,
                flags,
            )
        };
        if code == CURLE_AGAIN {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        if code != CURLE_OK {
            return Err(NetworkError::Network(curl_error(handle, code)));
        }
        offset += sent;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn curl_impersonate_websocket_round_trip() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            let message = socket.read().unwrap();
            socket.send(message).unwrap();
            socket.close(None).unwrap();
        });
        let (events_sender, events_receiver) = mpsc::channel();
        let socket = open_websocket(
            CurlConfig::default(),
            format!("ws://{address}/echo"),
            HeaderList::new(),
            Box::new(move |event| {
                if !matches!(
                    event,
                    WebSocketEvent::HandshakeRequest { .. }
                        | WebSocketEvent::HandshakeResponse { .. }
                        | WebSocketEvent::FrameSent { .. }
                ) {
                    let _ = events_sender.send(event);
                }
            }),
        )
        .unwrap();
        assert_eq!(
            events_receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap(),
            WebSocketEvent::Open
        );
        socket.send_text("hello").unwrap();
        assert_eq!(
            events_receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap(),
            WebSocketEvent::Text("hello".into())
        );
        drop(socket);
        server.join().unwrap();
    }
}
