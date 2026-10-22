//! Browser-like HTTP transport backed by `libcurl-impersonate`.
//!
//! `brimp-net` exposes a small Rust API around `curl_easy_impersonate`, making
//! it possible to issue requests with browser-oriented TLS and HTTP fingerprints
//! while still receiving responses through standard `http` messages and Hyper-compatible `http_body::Body` streams.
//!
//! # Native dependency
//!
//! This crate links to `libcurl-impersonate`. Local builds expect one of these
//! files in `BRIMP_CURL_LIB_DIR` (default `/usr/local/lib`):
//!
//! - `libcurl-impersonate.dylib`
//! - `libcurl-impersonate.so`
//! - `libcurl-impersonate_imp.lib` on Windows
//!
//! docs.rs builds skip native linking so API documentation can render without
//! the library installed.
//!
//! # Collecting a response
//!
//! ```no_run
//! use brimp_net::{Client, Config, RedirectPolicy};
//! use http::{Method, Request};
//!
//! let client = Client::new(Config {
//!     impersonation_target: "chrome150".to_string(),
//!     redirect_policy: RedirectPolicy::Follow,
//!     ..Config::default()
//! });
//!
//! let request = Request::builder()
//!     .method(Method::GET)
//!     .uri("https://example.com/")
//!     .body(None)?;
//! let response = client.send_collect(request)?;
//! println!("{} bytes from {}", response.body.len(), response.effective_url);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Streaming a response
//!
//! ```no_run
//! use brimp_net::{Client, Config};
//! use http::Request;
//! use http_body_util::{BodyExt, Empty};
//! use bytes::Bytes;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new(Config::default());
//! let request = Request::builder()
//!     .uri("https://example.com/")
//!     .body(Empty::<Bytes>::new())?;
//! let mut response = client.send(request).await?;
//! while let Some(frame) = response.body_mut().frame().await {
//!     if let Some(bytes) = frame?.data_ref() {
//!         println!("received {} bytes", bytes.len());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

mod body;
mod client;
mod config;
mod error;
mod ffi;
mod fingerprint;
mod headers;
mod multi;
mod pool;
mod response;
mod transfer;

pub use body::Body;
pub use client::Client;
pub use config::{
    Config, Credential, Credentials, OrderedHeaders, Proxy, ProxyKind, ProxyParseError,
    RedirectPolicy,
};
pub use error::{Error, Result};
pub use response::{CollectedResponse, TransferInfo, TransferMetadata};

mod websocket;
pub use websocket::{WebSocketEvent, WebSocketHandle, open_websocket};
