use std::ffi::{CString, c_long, c_void};
use std::ptr;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::time::Duration;

use crate::ffi::*;
use crate::{CurlConfig, HeaderList, NetworkError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketEvent {
    Open,
    Text(String),
    Binary(Vec<u8>),
    Close { code: u16, reason: String },
    Error(String),
}

enum Command {
    Text(String),
    Binary(Vec<u8>),
    Close,
}

pub struct WebSocketHandle {
    sender: Sender<Command>,
}

impl WebSocketHandle {
    pub fn send_text(&self, message: impl Into<String>) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Text(message.into()))
            .map_err(|_| NetworkError::Closed)
    }

    pub fn send_binary(&self, message: Vec<u8>) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Binary(message))
            .map_err(|_| NetworkError::Closed)
    }

    pub fn close(&self) -> Result<(), NetworkError> {
        self.sender
            .send(Command::Close)
            .map_err(|_| NetworkError::Closed)
    }
}

impl Drop for WebSocketHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Close);
    }
}

pub(crate) fn open(
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
        .map_err(|error| NetworkError::WorkerStart(error.to_string()))?;
    Ok(WebSocketHandle { sender })
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
        return Err(NetworkError::Transport(
            "failed to create curl handle".into(),
        ));
    }
    struct Easy(*mut Curl);
    impl Drop for Easy {
        fn drop(&mut self) {
            unsafe { curl_easy_cleanup(self.0) };
        }
    }
    let easy = Easy(handle);
    let url = CString::new(url).map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
    let profile = CString::new(config.impersonation_profile)
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
        return Err(NetworkError::Transport(error(impersonate)));
    }
    unsafe {
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
                return Err(NetworkError::Transport(error(code)));
            }
        } else {
            let code = curl_easy_perform(easy.0);
            if code != CURLE_OK {
                return Err(NetworkError::Transport(error(code)));
            }
        }
    }
    callback(WebSocketEvent::Open);
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        match receiver.try_recv() {
            Ok(Command::Text(message)) => send(easy.0, message.as_bytes(), CURLWS_TEXT)?,
            Ok(Command::Binary(message)) => send(easy.0, &message, CURLWS_BINARY)?,
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
            return Err(NetworkError::Transport(error(code)));
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
            return Err(NetworkError::Transport(error(code)));
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
        let socket = open(
            CurlConfig::default(),
            format!("ws://{address}/echo"),
            HeaderList::new(),
            Box::new(move |event| {
                let _ = events_sender.send(event);
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
