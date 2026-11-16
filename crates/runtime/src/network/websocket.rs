//! Socket connection policy is independent of HTTP resource loading.
use super::{CurlConfig, NetworkError, ResourceRequest};
pub use brimp_net::{WebSocketEvent, WebSocketHandle};
pub trait WebSocketConnector: Send + Sync {
    fn open_websocket(
        &self,
        request: ResourceRequest,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError>;
}
#[derive(Clone, Debug)]
pub struct CurlWebSocketConnector(pub CurlConfig);
impl WebSocketConnector for CurlWebSocketConnector {
    fn open_websocket(
        &self,
        request: ResourceRequest,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        let mut fields = http::HeaderMap::new();
        for (name, value) in request.headers.iter() {
            fields.append(name.clone(), value.clone());
        }
        let selected = request
            .proxy_options
            .resolve(&request.url, &self.0.proxy_options)?;
        let mut config = super::loader::transport_config(&self.0);
        config.proxy = selected.proxy;
        config.no_proxy = Some(selected.no_proxy);
        config.trust_env = false;
        brimp_net::open_websocket(config, request.url, fields, callback).map_err(NetworkError::from)
    }
}
