use async_trait::async_trait;
use http::{Method, StatusCode};
use thiserror::Error;

use super::{HeaderList, ResourceStreamCallback, ResourceStreamEvent, ResourceStreamHandle};

#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub request_id: Option<u64>,
    pub max_response_bytes: Option<usize>,
    pub resource_type: Option<String>,
    pub method: Method,
    pub url: String,
    pub headers: HeaderList,
    pub body: Option<Vec<u8>>,
    pub auth: Vec<HttpCredentials>,
}

#[derive(Clone)]
pub struct HttpCredentials {
    pub username: String,
    pub password: String,
    pub proxy: bool,
}

impl std::fmt::Debug for HttpCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpCredentials")
            .field("proxy", &self.proxy)
            .finish_non_exhaustive()
    }
}
impl ResourceRequest {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            request_id: None,
            max_response_bytes: None,
            resource_type: None,
            method,
            url: url.into(),
            headers: HeaderList::new(),
            body: None,
            auth: Vec::new(),
        }
    }
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::GET, url)
    }
}
#[derive(Debug, Clone)]
pub struct ResourceResponse {
    pub status: StatusCode,
    pub headers: HeaderList,
    pub body: Vec<u8>,
    pub effective_url: String,
    pub metadata: ResponseMetadata,
}

#[derive(Clone, Debug, Default)]
pub struct ResponseMetadata {
    pub http_version: Option<String>,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub header_bytes: u64,
}

#[derive(Debug, Error, Clone)]
pub enum NetworkError {
    #[error("invalid resource request: {0}")]
    InvalidRequest(String),
    #[error("resource transfer failed: {0}")]
    Transport(String),
    #[error("resource response exceeded the {limit}-byte limit")]
    ResponseTooLarge { limit: usize },
    #[error("resource request was cancelled")]
    Cancelled,
    #[error("resource loader is shutting down")]
    Closed,
    #[error("resource queue is full")]
    QueueFull,
    #[error("failed to start resource worker: {0}")]
    WorkerStart(String),
}

pub type ResourceCallback = Box<dyn FnOnce(Result<ResourceResponse, NetworkError>) + Send>;

#[async_trait]
pub trait ResourceLoader: Send + Sync {
    /// Records a response supplied by an engine subsystem without a transport request.
    fn observe_response(&self, _request: &ResourceRequest, _response: &ResourceResponse) {}

    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError>;

    /// Submits without adding a caller-owned coordination thread. Custom test
    /// loaders whose futures complete immediately can use this default.
    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        let mut future = Box::pin(self.fetch(request));
        match Future::poll(future.as_mut(), &mut Context::from_waker(Waker::noop())) {
            Poll::Ready(result) => {
                callback(result);
                Ok(())
            }
            Poll::Pending => Err(NetworkError::Transport(
                "this resource loader does not support callback submission".into(),
            )),
        }
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::new();
        let callback_handle = handle.clone();
        self.fetch_callback(
            request,
            Box::new(move |result| match result {
                Ok(response) => {
                    super::stream::deliver_buffered(response, callback, callback_handle)
                }
                Err(error) => {
                    let _ = callback(ResourceStreamEvent::Error(error), &callback_handle);
                }
            }),
        )?;
        Ok(handle)
    }
}

impl From<String> for NetworkError {
    fn from(value: String) -> Self {
        Self::Transport(value)
    }
}
