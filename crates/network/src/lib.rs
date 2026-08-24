//! Resource loading boundary and libcurl-impersonate transport.

use std::thread;

use async_trait::async_trait;
use bimp_net::{Client, Config, RedirectPolicy};
use http::{HeaderMap, Method, StatusCode};
use thiserror::Error;
use tokio::sync::oneshot;

/// A complete request for a browser resource.
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
}

impl ResourceRequest {
    /// Creates a request with no headers or body.
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// Creates a GET request.
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::GET, url)
    }
}

/// A fully collected browser resource response.
#[derive(Debug)]
pub struct ResourceResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub effective_url: String,
}

/// Failure returned by a resource loader.
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("invalid resource request: {0}")]
    InvalidRequest(String),
    #[error("resource transfer failed: {0}")]
    Transport(String),
    #[error("resource worker stopped before returning a response")]
    WorkerStopped,
    #[error("failed to start resource worker: {0}")]
    WorkerStart(String),
}

/// The transport boundary used by navigation and all subresource loading.
#[async_trait]
pub trait ResourceLoader: Send + Sync {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError>;
}

/// Browser-like HTTP transport backed by `libcurl-impersonate`.
///
/// The default Chrome impersonation profile negotiates HTTP/2 and supplies
/// matching browser headers. Each transfer runs away from the page owner thread.
#[derive(Debug, Clone)]
pub struct CurlResourceLoader {
    client: Client,
}

impl CurlResourceLoader {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(config),
        }
    }
}

impl Default for CurlResourceLoader {
    fn default() -> Self {
        Self::new(Config {
            impersonation_target: "chrome136".to_string(),
            redirect_policy: RedirectPolicy::Follow,
            default_headers: true,
            ..Config::default()
        })
    }
}

#[async_trait]
impl ResourceLoader for CurlResourceLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut builder = http::Request::builder()
            .method(request.method)
            .uri(&request.url);
        *builder.headers_mut().expect("request builder has headers") = request.headers;
        let request = builder
            .body(request.body)
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;

        let client = self.client.clone();
        let (sender, receiver) = oneshot::channel();
        thread::Builder::new()
            .name("brimp-resource".to_string())
            .spawn(move || {
                let response = client
                    .send_collect(request)
                    .map(|response| ResourceResponse {
                        status: response.status,
                        headers: response.headers,
                        body: response.body,
                        effective_url: response.effective_url,
                    })
                    .map_err(|error| NetworkError::Transport(error.to_string()));
                let _ = sender.send(response);
            })
            .map_err(|error| NetworkError::WorkerStart(error.to_string()))?;

        receiver.await.map_err(|_| NetworkError::WorkerStopped)?
    }
}
