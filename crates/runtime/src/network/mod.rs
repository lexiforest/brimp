//! Resource loading boundary and Brimp-owned libcurl-impersonate transport.

mod config;
mod headers;
mod interception;
mod loader;
mod resource;
mod stream;
mod websocket;

pub use config::{CurlConfig, Proxy, ProxyKind, ProxyParseError};
pub use headers::HeaderList;
pub use interception::{
    InterceptingResourceLoader, MAX_INTERCEPTED_BODY, ResourceInterception,
    ResourceInterceptionCallback, ResourceInterceptor,
};
pub use loader::CurlResourceLoader;
pub use resource::{
    HttpCredentials, NetworkError, ResourceCallback, ResourceLoader, ResourceRequest,
    ResourceResponse, ResponseMetadata,
};
pub use stream::{
    ResourceStreamCallback, ResourceStreamDirective, ResourceStreamEvent, ResourceStreamHandle,
};
pub use websocket::{CurlWebSocketConnector, WebSocketConnector, WebSocketEvent, WebSocketHandle};
