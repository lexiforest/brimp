use async_trait::async_trait;
use http::{Method, StatusCode};
use thiserror::Error;

use super::{
    HeaderList, ResourceStreamCallback, ResourceStreamEvent, ResourceStreamHandle, WebSocketEvent,
    WebSocketHandle,
};

#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderList,
    pub body: Option<Vec<u8>>,
}
impl ResourceRequest {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HeaderList::new(),
            body: None,
        }
    }
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::GET, url)
    }
}
#[derive(Debug)]
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
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError>;

    /// Submits without adding a caller-owned coordination thread. Custom test
    /// loaders whose futures complete immediately can use this default.
    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        use std::future::Future;
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake, Waker};
        struct Noop;
        impl Wake for Noop {
            fn wake(self: Arc<Self>) {}
        }
        let mut future = Box::pin(self.fetch(request));
        let waker = Waker::from(Arc::new(Noop));
        match Future::poll(future.as_mut(), &mut Context::from_waker(&waker)) {
            Poll::Ready(result) => {
                callback(result);
                Ok(())
            }
            Poll::Pending => Err(NetworkError::Transport(
                "this resource loader does not support callback submission".into(),
            )),
        }
    }

    fn open_websocket(
        &self,
        _url: String,
        _headers: HeaderList,
        _callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        Err(NetworkError::Transport(
            "this resource loader does not support WebSocket".into(),
        ))
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
                    let _ = callback(
                        ResourceStreamEvent::Headers {
                            status: response.status,
                            headers: response.headers,
                            url: response.effective_url,
                        },
                        &callback_handle,
                    );
                    if !callback_handle.is_cancelled() {
                        let _ =
                            callback(ResourceStreamEvent::Chunk(response.body), &callback_handle);
                    }
                    if !callback_handle.is_cancelled() {
                        let _ = callback(ResourceStreamEvent::Complete, &callback_handle);
                    }
                }
                Err(error) => {
                    let _ = callback(ResourceStreamEvent::Error(error), &callback_handle);
                }
            }),
        )?;
        Ok(handle)
    }
}
