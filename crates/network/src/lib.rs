//! Resource loading boundary and Brimp-owned libcurl-impersonate transport.

mod config;
mod ffi;
mod multi;
mod websocket;

use async_trait::async_trait;
pub use config::{CurlConfig, Proxy, ProxyKind, ProxyParseError};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use multi::MultiExecutor;
use std::ops::Index;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
};
use thiserror::Error;
pub use websocket::{WebSocketEvent, WebSocketHandle};

/// An insertion-ordered HTTP header list. Duplicate fields remain distinct.
#[derive(Debug, Clone, Default)]
pub struct HeaderList(Vec<(HeaderName, HeaderValue)>);
impl HeaderList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn append(&mut self, name: impl TryInto<HeaderName>, value: HeaderValue) {
        if let Ok(name) = name.try_into() {
            self.0.push((name, value));
        }
    }
    pub fn insert(&mut self, name: impl TryInto<HeaderName>, value: HeaderValue) {
        if let Ok(name) = name.try_into() {
            self.0.retain(|(candidate, _)| candidate != name);
            self.0.push((name, value));
        }
    }
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.get(name).is_some()
    }
    pub fn remove(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref();
        self.0
            .retain(|(candidate, _)| !candidate.as_str().eq_ignore_ascii_case(name));
    }
    pub fn get(&self, name: impl AsRef<str>) -> Option<&HeaderValue> {
        let name = name.as_ref();
        self.0
            .iter()
            .find(|(candidate, _)| candidate.as_str().eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }
    pub fn get_all(&self, name: impl AsRef<str>) -> impl Iterator<Item = &HeaderValue> {
        let name = name.as_ref().to_string();
        self.0
            .iter()
            .filter(move |(candidate, _)| candidate.as_str().eq_ignore_ascii_case(&name))
            .map(|(_, value)| value)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.0.iter().map(|(name, value)| (name, value))
    }
}
impl From<HeaderMap> for HeaderList {
    fn from(headers: HeaderMap) -> Self {
        let mut result = Self::new();
        let mut current_name = None;
        for (name, value) in headers {
            if let Some(name) = name {
                current_name = Some(name);
            }
            if let Some(name) = current_name.clone() {
                result.append(name, value);
            }
        }
        result
    }
}
impl Index<&str> for HeaderList {
    type Output = HeaderValue;
    fn index(&self, name: &str) -> &Self::Output {
        self.get(name).expect("header not found")
    }
}

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
}

#[derive(Debug, Clone)]
pub enum ResourceStreamEvent {
    Headers {
        status: StatusCode,
        headers: HeaderList,
        url: String,
    },
    Chunk(Vec<u8>),
    Complete,
    Error(NetworkError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceStreamDirective {
    Continue,
    Pause,
    Cancel,
}

const STREAM_RUNNING: u8 = 0;
const STREAM_PAUSED: u8 = 1;
const STREAM_CANCELLED: u8 = 2;

/// Controls one in-flight streaming request. Clones refer to the same curl
/// transfer and may safely be used by the page task that consumes a chunk.
#[derive(Debug)]
pub struct ResourceStreamHandle {
    state: Arc<AtomicU8>,
    delegate: Option<Arc<Mutex<Option<ResourceStreamHandle>>>>,
    owner: bool,
}

impl ResourceStreamHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            delegate: None,
            owner: true,
        }
    }

    fn proxy() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            delegate: Some(Arc::new(Mutex::new(None))),
            owner: true,
        }
    }

    fn attach(&self, handle: ResourceStreamHandle) {
        if self.is_cancelled() {
            handle.cancel();
        }
        *self.delegate.as_ref().unwrap().lock().unwrap() = Some(handle);
    }

    pub fn resume(&self) {
        let _ = self.state.compare_exchange(
            STREAM_PAUSED,
            STREAM_RUNNING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.resume();
        }
    }

    pub fn cancel(&self) {
        self.state.store(STREAM_CANCELLED, Ordering::Release);
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.cancel();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.state.load(Ordering::Acquire) == STREAM_CANCELLED
    }

    pub(crate) fn pause(&self) {
        let _ = self.state.compare_exchange(
            STREAM_RUNNING,
            STREAM_PAUSED,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.state.load(Ordering::Acquire) == STREAM_PAUSED
    }
}

impl Default for ResourceStreamHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ResourceStreamHandle {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            delegate: self.delegate.clone(),
            owner: false,
        }
    }
}

impl Drop for ResourceStreamHandle {
    fn drop(&mut self) {
        if self.owner {
            self.cancel();
        }
    }
}

pub type ResourceStreamCallback =
    Box<dyn FnMut(ResourceStreamEvent, &ResourceStreamHandle) -> ResourceStreamDirective + Send>;

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

pub enum ResourceInterception {
    Continue(ResourceRequest),
    Fulfill(ResourceResponse),
    Fail(NetworkError),
}

pub type ResourceInterceptionCallback = Box<dyn FnOnce(ResourceInterception) + Send>;

pub trait ResourceInterceptor: Send + Sync {
    fn intercept(&self, request: ResourceRequest, callback: ResourceInterceptionCallback);
}

pub struct InterceptingResourceLoader {
    inner: Arc<dyn ResourceLoader>,
    interceptor: Arc<dyn ResourceInterceptor>,
}

impl InterceptingResourceLoader {
    pub fn new(inner: Arc<dyn ResourceLoader>, interceptor: Arc<dyn ResourceInterceptor>) -> Self {
        Self { inner, interceptor }
    }
}

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

#[async_trait]
impl ResourceLoader for InterceptingResourceLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.interceptor.intercept(
            request,
            Box::new(move |decision| {
                let _ = sender.send(decision);
            }),
        );
        match receiver.await.map_err(|_| NetworkError::Closed)? {
            ResourceInterception::Continue(request) => self.inner.fetch(request).await,
            ResourceInterception::Fulfill(response) => Ok(response),
            ResourceInterception::Fail(error) => Err(error),
        }
    }

    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        let inner = Arc::clone(&self.inner);
        self.interceptor.intercept(
            request,
            Box::new(move |decision| match decision {
                ResourceInterception::Continue(request) => {
                    let callback = Arc::new(Mutex::new(Some(callback)));
                    let delivered = Arc::clone(&callback);
                    let result = inner.fetch_callback(
                        request,
                        Box::new(move |result| {
                            if let Some(callback) = delivered.lock().unwrap().take() {
                                callback(result);
                            }
                        }),
                    );
                    if let Err(error) = result
                        && let Some(callback) = callback.lock().unwrap().take()
                    {
                        callback(Err(error));
                    }
                }
                ResourceInterception::Fulfill(response) => callback(Ok(response)),
                ResourceInterception::Fail(error) => callback(Err(error)),
            }),
        );
        Ok(())
    }

    fn open_websocket(
        &self,
        url: String,
        headers: HeaderList,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        self.inner.open_websocket(url, headers, callback)
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::proxy();
        let decision_handle = handle.clone();
        let inner = Arc::clone(&self.inner);
        let callback = Arc::new(Mutex::new(callback));
        self.interceptor.intercept(
            request,
            Box::new(move |decision| match decision {
                ResourceInterception::Continue(request) => {
                    let callback_handle = decision_handle.clone();
                    let stream_callback = Arc::clone(&callback);
                    match inner.fetch_stream_callback(
                        request,
                        Box::new(move |event, _| {
                            stream_callback.lock().unwrap()(event, &callback_handle)
                        }),
                    ) {
                        Ok(inner_handle) => decision_handle.attach(inner_handle),
                        Err(error) => {
                            let _ = callback.lock().unwrap()(
                                ResourceStreamEvent::Error(error),
                                &decision_handle,
                            );
                        }
                    }
                }
                ResourceInterception::Fulfill(response) => {
                    if callback.lock().unwrap()(
                        ResourceStreamEvent::Headers {
                            status: response.status,
                            headers: response.headers,
                            url: response.effective_url,
                        },
                        &decision_handle,
                    ) == ResourceStreamDirective::Cancel
                    {
                        return;
                    }
                    if !response.body.is_empty()
                        && callback.lock().unwrap()(
                            ResourceStreamEvent::Chunk(response.body),
                            &decision_handle,
                        ) == ResourceStreamDirective::Cancel
                    {
                        return;
                    }
                    let _ =
                        callback.lock().unwrap()(ResourceStreamEvent::Complete, &decision_handle);
                }
                ResourceInterception::Fail(error) => {
                    let _ = callback.lock().unwrap()(
                        ResourceStreamEvent::Error(error),
                        &decision_handle,
                    );
                }
            }),
        );
        Ok(handle)
    }
}

/// Cloneable transport handle. Clones share exactly one curl multi executor.
#[derive(Debug, Clone)]
pub struct CurlResourceLoader {
    executor: std::sync::Arc<MultiExecutor>,
    config: CurlConfig,
}
impl CurlResourceLoader {
    pub fn new(config: CurlConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            executor: std::sync::Arc::new(MultiExecutor::new(config.clone())?),
            config,
        })
    }
    pub fn check_profile(config: &CurlConfig) -> Result<(), NetworkError> {
        use std::ffi::CString;
        ffi::global_init();
        let handle = unsafe { ffi::curl_easy_init() };
        if handle.is_null() {
            return Err(NetworkError::Transport(
                "failed to initialize curl easy handle".into(),
            ));
        }
        let profile = CString::new(config.impersonation_profile.as_str())
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let code = unsafe {
            ffi::curl_easy_impersonate(handle, profile.as_ptr(), config.default_headers as i32)
        };
        unsafe {
            ffi::curl_easy_cleanup(handle);
        }
        if code == ffi::CURLE_OK {
            Ok(())
        } else {
            Err(NetworkError::Transport(format!(
                "impersonation profile `{}` is unavailable: {}",
                config.impersonation_profile,
                ffi::error(code)
            )))
        }
    }
}
impl Default for CurlResourceLoader {
    fn default() -> Self {
        Self::new(CurlConfig::default()).expect("curl executor must start")
    }
}
#[async_trait]
impl ResourceLoader for CurlResourceLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        self.executor.fetch(request).await
    }
    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        self.executor.fetch_callback(request, callback)
    }
    fn open_websocket(
        &self,
        url: String,
        headers: HeaderList,
        callback: Box<dyn Fn(WebSocketEvent) + Send>,
    ) -> Result<WebSocketHandle, NetworkError> {
        websocket::open(self.config.clone(), url, headers, callback)
    }
    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        self.executor.fetch_stream_callback(request, callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, mpsc};
    use std::time::{Duration, Instant};

    static EXECUTOR_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn many_loader_clones_share_exactly_one_executor_thread() {
        let _guard = EXECUTOR_TEST_LOCK.lock().unwrap();
        let baseline = multi::executor_thread_count();
        let loader = CurlResourceLoader::default();
        wait_for_threads(baseline + 1);
        let clones = (0..128).map(|_| loader.clone()).collect::<Vec<_>>();
        assert!(
            clones
                .iter()
                .all(|clone| std::sync::Arc::ptr_eq(&loader.executor, &clone.executor))
        );
        assert_eq!(multi::executor_thread_count(), baseline + 1);
        drop(clones);
        drop(loader);
        wait_for_threads(baseline);
    }

    #[test]
    fn final_loader_can_be_released_by_its_worker_callback() {
        let _guard = EXECUTOR_TEST_LOCK.lock().unwrap();
        let baseline = multi::executor_thread_count();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 512];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let count = stream.read(&mut chunk).unwrap();
                request.extend_from_slice(&chunk[..count]);
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .unwrap();
        });
        let loader = CurlResourceLoader::default();
        wait_for_threads(baseline + 1);
        let callback_owner = loader.clone();
        let (done_sender, done_receiver) = mpsc::sync_channel(1);
        loader
            .fetch_callback(
                ResourceRequest::get(format!("http://{address}/")),
                Box::new(move |result| {
                    assert_eq!(result.unwrap().body, b"ok");
                    drop(callback_owner);
                    done_sender.send(()).unwrap();
                }),
            )
            .unwrap();
        drop(loader);
        done_receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        server.join().unwrap();
        wait_for_threads(baseline);
    }

    fn wait_for_threads(expected: usize) {
        let deadline = Instant::now() + Duration::from_secs(1);
        while multi::executor_thread_count() != expected && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(multi::executor_thread_count(), expected);
    }
}
