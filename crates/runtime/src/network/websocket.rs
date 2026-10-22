//! Socket connection policy is independent of HTTP resource loading.
use super::{CurlConfig, HeaderList, NetworkError};
pub use brimp_net::{WebSocketEvent, WebSocketHandle};
pub trait WebSocketConnector: Send + Sync {
    fn open_websocket(
        &self,
        url: String,
        headers: HeaderList,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError>;
}
#[derive(Clone, Debug)]
pub struct CurlWebSocketConnector(pub CurlConfig);
impl WebSocketConnector for CurlWebSocketConnector {
    fn open_websocket(
        &self,
        url: String,
        headers: HeaderList,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        let mut fields = http::HeaderMap::new();
        for (name, value) in headers.iter() {
            fields.append(name.clone(), value.clone());
        }
        brimp_net::open_websocket(
            super::loader::transport_config(&self.0),
            url,
            fields,
            callback,
        )
        .map_err(|e| NetworkError::Transport(e.to_string()))
    }
}
