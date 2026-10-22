use super::{
    CurlConfig, HeaderList, NetworkError, ResourceCallback, ResourceLoader, ResourceRequest,
    ResourceResponse, ResourceStreamCallback, ResourceStreamDirective, ResourceStreamEvent,
    ResourceStreamHandle, ResponseMetadata,
};
use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderValue, Method, StatusCode};
use http_body_util::{BodyExt, Full};
use std::sync::Arc;

/// Browser resource adapter over the standard HTTP transport in brimp-net.
#[derive(Clone)]
pub struct CurlResourceLoader {
    executor: Arc<Executor>,
    config: CurlConfig,
}
struct Executor {
    client: brimp_net::Client,
    runtime: Option<tokio::runtime::Runtime>,
    handle: tokio::runtime::Handle,
}
impl Drop for Executor {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
impl std::fmt::Debug for CurlResourceLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurlResourceLoader").finish_non_exhaustive()
    }
}
impl CurlResourceLoader {
    pub fn new(config: CurlConfig) -> Result<Self, NetworkError> {
        if config.queue_capacity == 0 || config.max_response_bytes == 0 {
            return Err(NetworkError::InvalidRequest(
                "queue and response limits must be positive".into(),
            ));
        }
        let client = brimp_net::Client::new(transport_config(&config));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_time()
            .thread_name("brimp-resource-dispatch")
            .build()
            .map_err(|e| NetworkError::WorkerStart(e.to_string()))?;
        let handle = runtime.handle().clone();
        Ok(Self {
            executor: Arc::new(Executor {
                client,
                runtime: Some(runtime),
                handle,
            }),
            config,
        })
    }
    pub(crate) fn with_dispatcher(
        config: CurlConfig,
        handle: tokio::runtime::Handle,
    ) -> Result<Self, NetworkError> {
        if config.queue_capacity == 0 || config.max_response_bytes == 0 {
            return Err(NetworkError::InvalidRequest(
                "queue and response limits must be positive".into(),
            ));
        }
        Ok(Self {
            executor: Arc::new(Executor {
                client: brimp_net::Client::new(transport_config(&config)),
                runtime: None,
                handle,
            }),
            config,
        })
    }
    pub fn check_profile(config: &CurlConfig) -> Result<(), NetworkError> {
        brimp_net::Client::new(transport_config(config))
            .check_profile()
            .map_err(net_error)
    }
}
impl Default for CurlResourceLoader {
    fn default() -> Self {
        Self::new(CurlConfig::default()).expect("resource dispatcher must start")
    }
}
pub(crate) fn transport_config(config: &CurlConfig) -> brimp_net::Config {
    brimp_net::Config {
        impersonation_target: config.impersonation_profile.clone(),
        default_headers: config.default_headers,
        prefer_http3: config.prefer_http3,
        ca_bundle: config.ca_bundle.clone(),
        queue_capacity: config.queue_capacity,
        connect_timeout: config.connect_timeout,
        request_timeout: config.request_timeout,
        proxy: config
            .proxy
            .as_ref()
            .map(|p| brimp_net::Proxy::parse(p.url()).expect("validated proxy")),
        redirect_policy: brimp_net::RedirectPolicy::None,
    }
}
fn net_error(error: brimp_net::Error) -> NetworkError {
    match error {
        brimp_net::Error::InvalidRequest(message) => NetworkError::InvalidRequest(message),
        other => NetworkError::Transport(other.to_string()),
    }
}
fn http_request(request: &ResourceRequest) -> Result<http::Request<Full<Bytes>>, NetworkError> {
    let mut result = http::Request::builder()
        .method(request.method.clone())
        .uri(&request.url)
        .body(Full::new(Bytes::from(
            request.body.clone().unwrap_or_default(),
        )))
        .map_err(|e| NetworkError::InvalidRequest(e.to_string()))?;
    let headers = request
        .headers
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect::<Vec<_>>();
    for (name, value) in &headers {
        result.headers_mut().append(name.clone(), value.clone());
    }
    result
        .extensions_mut()
        .insert(brimp_net::OrderedHeaders(headers));
    result.extensions_mut().insert(brimp_net::Credentials(
        request
            .auth
            .iter()
            .map(|a| brimp_net::Credential {
                username: a.username.clone(),
                password: a.password.clone(),
                proxy: a.proxy,
            })
            .collect(),
    ));
    Ok(result)
}
fn limit(request: &ResourceRequest, config: &CurlConfig) -> usize {
    request
        .max_response_bytes
        .unwrap_or(config.max_response_bytes)
        .min(config.max_response_bytes)
}
async fn collect(
    client: &brimp_net::Client,
    request: ResourceRequest,
    config: &CurlConfig,
) -> Result<ResourceResponse, NetworkError> {
    let maximum = limit(&request, config);
    let response = client
        .send(http_request(&request)?)
        .await
        .map_err(net_error)?;
    let info = response
        .extensions()
        .get::<brimp_net::TransferInfo>()
        .cloned()
        .unwrap_or_default();
    let (parts, mut stream) = response.into_parts();
    let mut body = Vec::new();
    while let Some(frame) = stream.frame().await {
        let frame = frame.map_err(net_error)?;
        if let Some(bytes) = frame.data_ref() {
            if body.len().saturating_add(bytes.len()) > maximum {
                return Err(NetworkError::ResponseTooLarge { limit: maximum });
            }
            body.extend_from_slice(bytes);
        }
    }
    let meta = info.snapshot();
    Ok(ResourceResponse {
        status: parts.status,
        headers: parts.headers.into(),
        body,
        effective_url: meta.effective_url,
        metadata: ResponseMetadata {
            http_version: meta.http_version,
            downloaded_bytes: meta.downloaded_bytes,
            uploaded_bytes: meta.uploaded_bytes,
            header_bytes: meta.header_bytes,
        },
    })
}
struct Callback(Option<ResourceCallback>);
impl Drop for Callback {
    fn drop(&mut self) {
        if let Some(callback) = self.0.take() {
            callback(Err(NetworkError::Closed));
        }
    }
}
#[async_trait]
impl ResourceLoader for CurlResourceLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        if let Some(response) = data_url_response(&request) {
            return response;
        }
        collect(&self.executor.client, request, &self.config).await
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
        let client = self.executor.client.clone();
        let config = self.config.clone();
        let mut callback = Callback(Some(callback));
        self.executor.handle.spawn(async move {
            let result = collect(&client, request, &config).await;
            if let Some(callback) = callback.0.take() {
                callback(result);
            }
        });
        Ok(())
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::new();
        let control = handle.clone();
        if let Some(response) = data_url_response(&request) {
            match response {
                Ok(response) => super::stream::deliver_buffered(response, callback, control),
                Err(error) => {
                    callback(ResourceStreamEvent::Error(error), &control);
                }
            }
            return Ok(handle);
        }
        let client = self.executor.client.clone();
        let config = self.config.clone();
        self.executor.handle.spawn(async move {
            let result=tokio::select!{biased; ()=control.cancelled()=>Err(NetworkError::Cancelled), result=stream(&client,request,&config,&control,&mut callback)=>result};
            if let Err(error)=result {callback(ResourceStreamEvent::Error(error),&control);}
        });
        Ok(handle)
    }
}
async fn deliver(
    event: ResourceStreamEvent,
    handle: &ResourceStreamHandle,
    callback: &mut ResourceStreamCallback,
) -> bool {
    loop {
        if handle.is_cancelled() {
            return false;
        }
        handle.pause();
        match callback(event.clone(), handle) {
            ResourceStreamDirective::Continue => {
                handle.resume();
                return true;
            }
            ResourceStreamDirective::Cancel | ResourceStreamDirective::Redirect => return false,
            ResourceStreamDirective::Pause => handle.wait_resumed().await,
        }
    }
}
async fn stream(
    client: &brimp_net::Client,
    request: ResourceRequest,
    config: &CurlConfig,
    handle: &ResourceStreamHandle,
    callback: &mut ResourceStreamCallback,
) -> Result<(), NetworkError> {
    let maximum = limit(&request, config);
    let response = client
        .send(http_request(&request)?)
        .await
        .map_err(net_error)?;
    let (parts, mut body) = response.into_parts();
    if !deliver(
        ResourceStreamEvent::Headers {
            status: parts.status,
            headers: parts.headers.into(),
            url: request.url,
        },
        handle,
        callback,
    )
    .await
    {
        return Ok(());
    }
    let mut received = 0usize;
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(net_error)?;
        if let Some(bytes) = frame.data_ref() {
            received = received.saturating_add(bytes.len());
            if received > maximum {
                return Err(NetworkError::ResponseTooLarge { limit: maximum });
            }
            if !deliver(ResourceStreamEvent::Chunk(bytes.to_vec()), handle, callback).await {
                return Ok(());
            }
        }
    }
    deliver(ResourceStreamEvent::Complete, handle, callback).await;
    Ok(())
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
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, mpsc};
    use std::time::Duration;

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
    fn many_loader_clones_share_one_transport_and_dispatcher() {
        let _guard = EXECUTOR_TEST_LOCK.lock().unwrap();
        let loader = CurlResourceLoader::default();
        let clones = (0..128).map(|_| loader.clone()).collect::<Vec<_>>();
        assert!(
            clones
                .iter()
                .all(|clone| std::sync::Arc::ptr_eq(&loader.executor, &clone.executor))
        );
        let weak = Arc::downgrade(&loader.executor);
        drop(clones);
        drop(loader);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn final_loader_can_be_released_by_its_worker_callback() {
        let _guard = EXECUTOR_TEST_LOCK.lock().unwrap();
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
    }
}
