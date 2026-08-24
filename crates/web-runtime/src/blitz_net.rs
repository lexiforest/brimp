use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use blitz_traits::net::{Body, Bytes, NetHandler, NetProvider, Request};
use network::{ResourceLoader, ResourceRequest};
use tokio::sync::Notify;
use web_bindings::BrowsingContext;

#[derive(Clone)]
pub(crate) struct BlitzResourceProvider {
    shared: Arc<Shared>,
}

struct Shared {
    loader: Arc<dyn ResourceLoader>,
    browsing_context: Arc<BrowsingContext>,
    outstanding: AtomicUsize,
    idle: Notify,
}

impl BlitzResourceProvider {
    pub(crate) fn new(
        loader: Arc<dyn ResourceLoader>,
        browsing_context: Arc<BrowsingContext>,
    ) -> Self {
        Self {
            shared: Arc::new(Shared {
                loader,
                browsing_context,
                outstanding: AtomicUsize::new(0),
                idle: Notify::new(),
            }),
        }
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.shared.outstanding.load(Ordering::Acquire) == 0
    }

    pub(crate) async fn wait_idle(&self) {
        loop {
            let notified = self.shared.idle.notified();
            if self.is_idle() {
                return;
            }
            notified.await;
        }
    }

    fn complete(&self) {
        if self.shared.outstanding.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.shared.idle.notify_waiters();
        }
    }
}

impl NetProvider for BlitzResourceProvider {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url = request.url.to_string();
        let mut resource_request = ResourceRequest::new(request.method, &url);
        resource_request.headers = request.headers;
        if let Some(content_type) = request.content_type
            && !resource_request
                .headers
                .contains_key(http::header::CONTENT_TYPE)
            && let Ok(content_type) = http::HeaderValue::from_str(&content_type)
        {
            resource_request
                .headers
                .insert(http::header::CONTENT_TYPE, content_type);
        }
        resource_request.body = match request.body {
            Body::Bytes(bytes) => Some(bytes.to_vec()),
            Body::Form(form) => serde_json::to_vec(&form).ok(),
            Body::Empty => None,
        };
        if !resource_request.headers.contains_key(http::header::COOKIE)
            && let Some(cookies) = self.shared.browsing_context.cookie_header(&url)
            && let Ok(cookies) = http::HeaderValue::from_str(&cookies)
        {
            resource_request
                .headers
                .insert(http::header::COOKIE, cookies);
        }

        self.shared.outstanding.fetch_add(1, Ordering::AcqRel);
        let provider = self.clone();
        let spawn = std::thread::Builder::new()
            .name("brimp-blitz-resource".to_string())
            .spawn(move || {
                let response = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .ok()
                    .and_then(|runtime| {
                        runtime
                            .block_on(provider.shared.loader.fetch(resource_request))
                            .ok()
                    });
                match response {
                    Some(response) => {
                        let effective_url = if response.effective_url.is_empty() {
                            url
                        } else {
                            response.effective_url
                        };
                        for header in response.headers.get_all(http::header::SET_COOKIE) {
                            if let Ok(header) = header.to_str() {
                                provider
                                    .shared
                                    .browsing_context
                                    .store_response_cookie(&effective_url, header);
                            }
                        }
                        handler.bytes(effective_url, Bytes::from(response.body));
                    }
                    None => handler.bytes(url, Bytes::new()),
                }
                provider.complete();
            });
        if spawn.is_err() {
            self.complete();
        }
    }
}
