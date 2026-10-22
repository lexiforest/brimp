use std::fmt::Display;
use std::sync::{Arc, OnceLock};

use bytes::Bytes;
use http::{Request, Response};
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use tokio::sync::{mpsc, oneshot};

use crate::body::Body;
use crate::config::Config;
use crate::error::Error;
use crate::multi::{MultiExecutor, StreamingTransfer};
use crate::pool::CurlHandlePool;
use crate::response::CollectedResponse;
use crate::transfer::CurlTransfer;

const BODY_CHANNEL_CAPACITY: usize = 16;

/// HTTP client that executes requests with `libcurl-impersonate`.
///
/// Clones share a curl multi coordinator and an easy-handle pool. Keep a clone
/// alive while reading responses: dropping the last client cancels active sends.
#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    handle_pool: Arc<CurlHandlePool>,
    multi: Arc<MultiExecutor>,
    fingerprint: Arc<OnceLock<Result<Option<crate::fingerprint::Fingerprint>, Error>>>,
}

impl Client {
    /// Creates a new client with the provided configuration.
    pub fn new(config: Config) -> Self {
        Self {
            multi: Arc::new(MultiExecutor::new(config.queue_capacity)),
            config,
            handle_pool: Arc::new(CurlHandlePool::default()),
            fingerprint: Arc::new(OnceLock::new()),
        }
    }

    /// Validate the configured impersonation profile without issuing a request.
    pub fn check_profile(&self) -> Result<(), Error> {
        let request = Request::builder()
            .uri("https://profile.invalid/")
            .body(None)
            .map_err(|e| Error::InvalidRequest(e.to_string()))?;
        CurlTransfer::new(
            request,
            self.config.clone(),
            self.handle_pool.clone(),
            self.resolved_fingerprint()?,
        )
        .map(|_| ())
    }

    /// Sends a request and returns a streaming response body.
    ///
    /// The response resolves once final response headers are available. Body
    /// bytes continue to arrive through [`Body`]. Request bodies are collected
    /// before submission; uploads are not streamed. Dropping the send future
    /// or its response body cancels the transfer.
    pub async fn send<B>(&self, request: Request<B>) -> Result<Response<Body>, Error>
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: Display,
    {
        self.send_buffered(collect_request_body(request).await?)
            .await
    }

    async fn send_buffered(
        &self,
        request: Request<Option<Vec<u8>>>,
    ) -> Result<Response<Body>, Error> {
        let transfer = Box::new(CurlTransfer::new(
            request,
            self.config.clone(),
            Arc::clone(&self.handle_pool),
            self.resolved_fingerprint()?,
        )?);
        let (response_sender, response_receiver) = oneshot::channel();
        let (body_sender, body_receiver) = mpsc::channel(BODY_CHANNEL_CAPACITY);
        let (completion_sender, completion_receiver) = oneshot::channel();

        self.multi.submit(StreamingTransfer {
            transfer,
            wake: Arc::clone(&self.multi.wake),
            response_sender,
            body_sender,
            body_receiver,
            completion_sender,
            completion_receiver,
        })?;

        response_receiver
            .await
            .map_err(|_| Error::Network("curl worker exited before response headers".to_string()))?
    }

    /// Sends a request and collects the full response body into memory.
    ///
    /// This blocks the calling thread using curl easy-perform. In async code,
    /// use [`Self::send`] and `http_body_util::BodyExt::collect` instead.
    /// Collection has no body-size limit; use incremental consumption for large
    /// or untrusted responses.
    pub fn send_collect(
        &self,
        request: Request<Option<Vec<u8>>>,
    ) -> Result<CollectedResponse, Error> {
        CurlTransfer::new(
            request,
            self.config.clone(),
            Arc::clone(&self.handle_pool),
            self.resolved_fingerprint()?,
        )?
        .perform_collect()
    }

    fn resolved_fingerprint(&self) -> Result<Option<&crate::fingerprint::Fingerprint>, Error> {
        self.fingerprint
            .get_or_init(|| crate::fingerprint::load_named(&self.config.impersonation_target))
            .as_ref()
            .map(Option::as_ref)
            .map_err(Clone::clone)
    }
}

async fn collect_request_body<B>(request: Request<B>) -> Result<Request<Option<Vec<u8>>>, Error>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Display,
{
    let (parts, body) = request.into_parts();
    let bytes = body
        .collect()
        .await
        .map_err(|error| Error::InvalidRequest(format!("failed to collect request body: {error}")))?
        .to_bytes();
    let body = (!bytes.is_empty()).then(|| bytes.to_vec());
    Ok(Request::from_parts(parts, body))
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use http::Request;
    use http_body_util::{Empty, Full};
    use tokio::runtime::Builder;

    use super::collect_request_body;

    #[test]
    fn collects_empty_hyper_body_as_none() {
        let runtime = Builder::new_current_thread().build().unwrap();
        let request = Request::builder()
            .uri("https://example.com/")
            .body(Empty::<Bytes>::new())
            .unwrap();

        let request = runtime.block_on(collect_request_body(request)).unwrap();

        assert_eq!(request.uri(), "https://example.com/");
        assert!(request.body().is_none());
    }

    #[test]
    fn collects_full_hyper_body_as_bytes() {
        let runtime = Builder::new_current_thread().build().unwrap();
        let request = Request::builder()
            .method("POST")
            .uri("https://example.com/")
            .body(Full::new(Bytes::from_static(b"hello")))
            .unwrap();

        let request = runtime.block_on(collect_request_body(request)).unwrap();

        assert_eq!(request.method(), "POST");
        assert_eq!(request.body().as_deref(), Some(&b"hello"[..]));
    }
}
