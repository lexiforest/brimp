use async_trait::async_trait;
use http::{HeaderValue, Method, StatusCode};

use super::{
    CurlConfig, HeaderList, NetworkError, ResourceCallback, ResourceLoader, ResourceRequest,
    ResourceResponse, ResourceStreamCallback, ResourceStreamHandle, ResponseMetadata,
    WebSocketEvent, WebSocketHandle, ffi, multi::MultiExecutor, websocket,
};

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
        if let Some(response) = data_url_response(&request) {
            return response;
        }
        self.executor.fetch(request).await
    }
    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        if let Some(response) = data_url_response(&request) {
            callback(response);
            return Ok(());
        }
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

fn data_url_response(request: &ResourceRequest) -> Option<Result<ResourceResponse, NetworkError>> {
    if !request
        .url
        .trim_start_matches(|character: char| character <= ' ')
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
    {
        return None;
    }
    Some((|| {
        if request.method != Method::GET || request.body.is_some() {
            return Err(NetworkError::InvalidRequest(
                "data URL resources require GET without a body".into(),
            ));
        }
        let parsed = data_url::DataUrl::process(&request.url)
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let content_type = parsed.mime_type().to_string();
        let (body, _) = parsed
            .decode_to_vec()
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let mut headers = HeaderList::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str(&content_type)
                .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?,
        );
        let downloaded_bytes = body.len() as u64;
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body,
            effective_url: request.url.clone(),
            metadata: ResponseMetadata {
                downloaded_bytes,
                ..ResponseMetadata::default()
            },
        })
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::multi;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, mpsc};
    use std::time::{Duration, Instant};

    static EXECUTOR_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn data_urls_are_decoded_without_entering_curl() {
        // Loader construction starts an executor even when this request is a
        // data URL. Keep it out of the other tests' process-wide thread counts.
        let _guard = EXECUTOR_TEST_LOCK.lock().unwrap();
        let loader = CurlResourceLoader::default();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let response = runtime
            .block_on(loader.fetch(ResourceRequest::get(
                "data:text/javascript;charset=utf-8;base64,Y29uc29sZS5sb2coJ29rJyk=",
            )))
            .unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body, b"console.log('ok')");
        assert_eq!(
            response.headers["content-type"].to_str().unwrap(),
            "text/javascript;charset=utf-8"
        );
        assert_eq!(response.metadata.downloaded_bytes, 17);
    }

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
