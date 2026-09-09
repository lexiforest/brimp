use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use super::{
    HeaderList, NetworkError, ResourceCallback, ResourceLoader, ResourceRequest, ResourceResponse,
    ResourceStreamCallback, ResourceStreamDirective, ResourceStreamEvent, ResourceStreamHandle,
    WebSocketEvent, WebSocketHandle,
};

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
