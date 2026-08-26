//! Resource loading boundary and Brimp-owned libcurl-impersonate transport.

mod config;
mod ffi;
mod multi;

use async_trait::async_trait;
pub use config::{CurlConfig, Proxy, ProxyKind, ProxyParseError};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use multi::MultiExecutor;
use std::ops::Index;
use thiserror::Error;

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
}

/// Cloneable transport handle. Clones share exactly one curl multi executor.
#[derive(Debug, Clone)]
pub struct CurlResourceLoader {
    executor: std::sync::Arc<MultiExecutor>,
}
impl CurlResourceLoader {
    pub fn new(config: CurlConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            executor: std::sync::Arc::new(MultiExecutor::new(config)?),
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
