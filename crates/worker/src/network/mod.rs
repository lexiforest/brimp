//! Resource loading boundary and Brimp-owned libcurl-impersonate transport.

mod config;
mod ffi;
mod headers;
mod interception;
mod loader;
mod multi;
mod resource;
mod stream;
mod websocket;

pub use config::{CurlConfig, Proxy, ProxyKind, ProxyParseError};
pub use headers::HeaderList;
pub use interception::{
    InterceptingResourceLoader, ResourceInterception, ResourceInterceptionCallback,
    ResourceInterceptor,
};
pub use loader::CurlResourceLoader;
pub use resource::{
    NetworkError, ResourceCallback, ResourceLoader, ResourceRequest, ResourceResponse,
    ResponseMetadata,
};
pub use stream::{
    ResourceStreamCallback, ResourceStreamDirective, ResourceStreamEvent, ResourceStreamHandle,
};
pub use websocket::{WebSocketEvent, WebSocketHandle};
