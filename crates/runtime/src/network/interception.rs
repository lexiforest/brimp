use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use super::{
    NetworkError, ResourceCallback, ResourceLoader, ResourceRequest, ResourceResponse,
    ResourceStreamCallback, ResourceStreamEvent, ResourceStreamHandle,
};

pub const MAX_INTERCEPTED_BODY: usize = 32 * 1024 * 1024;

pub enum ResourceInterception {
    Continue(ResourceRequest),
    Fulfill(ResourceResponse),
    Fail(NetworkError),
}

pub type ResourceInterceptionCallback = Box<dyn FnOnce(ResourceInterception) + Send>;

pub trait ResourceInterceptor: Send + Sync {
    fn intercept(&self, request: ResourceRequest, callback: ResourceInterceptionCallback);

    fn intercepts_response(&self, _request: &ResourceRequest) -> bool {
        false
    }

    fn intercept_response(
        &self,
        _request: ResourceRequest,
        response: ResourceResponse,
        callback: ResourceInterceptionCallback,
    ) {
        callback(ResourceInterception::Fulfill(response));
    }
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
        let mut decision = receiver.await.map_err(|_| NetworkError::Closed)?;
        for _ in 0..=16 {
            match decision {
                ResourceInterception::Continue(mut request) => {
                    if self.interceptor.intercepts_response(&request) {
                        request.max_response_bytes = Some(MAX_INTERCEPTED_BODY);
                    }
                    let response = self.inner.fetch(request.clone()).await?;
                    if !self.interceptor.intercepts_response(&request) {
                        return Ok(response);
                    }
                    let (sender, receiver) = tokio::sync::oneshot::channel();
                    self.interceptor.intercept_response(
                        request,
                        response,
                        Box::new(move |decision| {
                            let _ = sender.send(decision);
                        }),
                    );
                    decision = receiver.await.map_err(|_| NetworkError::Closed)?;
                }
                ResourceInterception::Fulfill(response) => return Ok(response),
                ResourceInterception::Fail(error) => return Err(error),
            }
        }
        Err(NetworkError::Transport(
            "too many authentication attempts".into(),
        ))
    }

    fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        let inner = Arc::clone(&self.inner);
        let interceptor = Arc::clone(&self.interceptor);
        self.interceptor.intercept(
            request,
            Box::new(move |decision| {
                deliver(inner, interceptor, decision, callback, 0);
            }),
        );
        Ok(())
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::proxy();
        let decision_handle = handle.clone();
        let inner = Arc::clone(&self.inner);
        let interceptor = Arc::clone(&self.interceptor);
        let callback = Arc::new(Mutex::new(callback));
        self.interceptor.intercept(
            request,
            Box::new(move |decision| {
                if decision_handle.is_cancelled() {
                    callback.lock().unwrap()(
                        ResourceStreamEvent::Error(NetworkError::Cancelled),
                        &decision_handle,
                    );
                    return;
                }
                match decision {
                    ResourceInterception::Continue(request)
                        if interceptor.intercepts_response(&request) =>
                    {
                        deliver(
                            inner,
                            interceptor,
                            ResourceInterception::Continue(request),
                            Box::new(move |result| match result {
                                Ok(response) => super::stream::deliver_buffered(
                                    response,
                                    Box::new(move |event, control| {
                                        callback.lock().unwrap()(event, control)
                                    }),
                                    decision_handle,
                                ),
                                Err(error) => {
                                    callback.lock().unwrap()(
                                        ResourceStreamEvent::Error(error),
                                        &decision_handle,
                                    );
                                }
                            }),
                            0,
                        );
                    }
                    ResourceInterception::Continue(request) => {
                        let stream_callback = Arc::clone(&callback);
                        match inner.fetch_stream_callback(
                            request,
                            Box::new(move |event, control| {
                                stream_callback.lock().unwrap()(event, control)
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
                    ResourceInterception::Fulfill(response) => super::stream::deliver_buffered(
                        response,
                        Box::new(move |event, control| callback.lock().unwrap()(event, control)),
                        decision_handle,
                    ),
                    ResourceInterception::Fail(error) => {
                        let _ = callback.lock().unwrap()(
                            ResourceStreamEvent::Error(error),
                            &decision_handle,
                        );
                    }
                }
            }),
        );
        Ok(handle)
    }
}

fn deliver(
    inner: Arc<dyn ResourceLoader>,
    interceptor: Arc<dyn ResourceInterceptor>,
    decision: ResourceInterception,
    callback: ResourceCallback,
    retries: usize,
) {
    match decision {
        ResourceInterception::Continue(mut request) => {
            if interceptor.intercepts_response(&request) {
                request.max_response_bytes = Some(MAX_INTERCEPTED_BODY);
            }
            if retries > 16 {
                callback(Err(NetworkError::Transport(
                    "too many authentication attempts".into(),
                )));
                return;
            }
            let callback = Arc::new(Mutex::new(Some(callback)));
            let delivered = Arc::clone(&callback);
            let next_inner = Arc::clone(&inner);
            let result = inner.fetch_callback(
                request.clone(),
                Box::new(move |result| {
                    let Some(callback) = delivered.lock().unwrap().take() else {
                        return;
                    };
                    match result {
                        Ok(response) if interceptor.intercepts_response(&request) => {
                            let next_interceptor = Arc::clone(&interceptor);
                            interceptor.intercept_response(
                                request,
                                response,
                                Box::new(move |decision| {
                                    deliver(
                                        next_inner,
                                        next_interceptor,
                                        decision,
                                        callback,
                                        retries + 1,
                                    );
                                }),
                            );
                        }
                        result => callback(result),
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
    }
}
