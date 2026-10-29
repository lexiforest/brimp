//! Page fetch, streaming and WebSocket operations.

use super::{NavigationError, Page};
use crate::network::{HeaderList, ResourceRequest};
use crate::web_apis::{BrowsingContext, PendingFetch, PendingWebSocketOperation};
use brimp_jsc::JsException;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

impl Page {
    pub(crate) fn set_network_offline(&mut self, offline: bool) -> Result<(), JsException> {
        self.js.call_function_with_string(
            &self.profile_identity_setter,
            &serde_json::json!({"onLine": !offline}).to_string(),
        )?;
        self.network_offline = offline;
        Ok(())
    }

    pub(super) async fn fetch_success(
        &self,
        url: &str,
    ) -> Result<crate::network::ResourceResponse, NavigationError> {
        let request = self.resource_request(url)?;
        let response = crate::runtime::request::fetch(
            self.network_scope.loader.as_ref(),
            &self.browsing_context,
            request,
        )
        .await?;
        self.accept_success_response(response)
    }

    pub(super) fn resource_request(&self, url: &str) -> Result<ResourceRequest, NavigationError> {
        Ok(ResourceRequest::get(url))
    }

    pub(super) fn accept_success_response(
        &self,
        response: crate::network::ResourceResponse,
    ) -> Result<crate::network::ResourceResponse, NavigationError> {
        if response.status.is_success() {
            Ok(response)
        } else {
            Err(NavigationError::HttpStatus(response.status.as_u16()))
        }
    }

    pub(crate) fn network_context(&self) -> Arc<BrowsingContext> {
        self.browsing_context.clone()
    }

    pub(crate) fn is_network_idle(&mut self) -> bool {
        self.fetches.borrow().is_empty()
            && self.fetch_streams.borrow().is_empty()
            && self.tasks.is_empty()
    }

    pub(super) fn start_pending_fetches(&self) -> Result<(), JsException> {
        let pending_fetches = self.fetches.borrow_mut().take_pending();
        for pending in pending_fetches {
            let id = pending.id;
            let streaming = pending.streaming;
            let request = match self.prepare_fetch_request(pending) {
                Ok(request) => request,
                Err(error) => {
                    self.reject_fetch(id, &error)?;
                    continue;
                }
            };
            let requested_url = request.url.clone();
            let loader = Arc::clone(&self.network_scope.loader);
            let browsing_context = Arc::clone(&self.browsing_context);
            let sender = self.task_sender();
            let buffered_bytes = Arc::new(AtomicUsize::new(0));
            let callback_buffered_bytes = Arc::clone(&buffered_bytes);
            if streaming {
                let result = crate::runtime::request::streaming::fetch_stream_callback(
                    loader,
                    browsing_context,
                    request,
                    true,
                    Box::new(move |event, control| {
                        const HIGH_WATER_MARK: usize = 256 * 1024;
                        const LOW_WATER_MARK: usize = 128 * 1024;
                        let chunk_length = match &event {
                            crate::network::ResourceStreamEvent::Chunk(bytes) => bytes.len(),
                            _ => 0,
                        };
                        if chunk_length != 0 {
                            let reserved = callback_buffered_bytes.try_update(
                                Ordering::AcqRel,
                                Ordering::Acquire,
                                |current| {
                                    (current.saturating_add(chunk_length) <= HIGH_WATER_MARK)
                                        .then_some(current + chunk_length)
                                },
                            );
                            if reserved.is_err() {
                                let resume_control = control.clone();
                                let resume_buffered_bytes = Arc::clone(&callback_buffered_bytes);
                                if sender
                                    .post(move |_| {
                                        if resume_buffered_bytes.load(Ordering::Acquire)
                                            <= LOW_WATER_MARK
                                        {
                                            resume_control.resume();
                                        }
                                    })
                                    .is_err()
                                {
                                    return crate::network::ResourceStreamDirective::Cancel;
                                }
                                return crate::network::ResourceStreamDirective::Pause;
                            }
                        }
                        if sender
                            .post(move |page| {
                                if let Err(error) = page.handle_fetch_stream_event(id, event) {
                                    page.async_error = Some(error);
                                }
                            })
                            .is_err()
                        {
                            if chunk_length != 0 {
                                callback_buffered_bytes.fetch_sub(chunk_length, Ordering::AcqRel);
                            }
                            return crate::network::ResourceStreamDirective::Cancel;
                        }
                        crate::network::ResourceStreamDirective::Continue
                    }),
                );
                match result {
                    Ok(handle) => {
                        self.fetch_buffered.borrow_mut().insert(id, buffered_bytes);
                        self.fetch_streams.borrow_mut().insert(id, handle);
                    }
                    Err(error) => self.reject_fetch(id, &error.to_string())?,
                }
                continue;
            }
            let result = crate::runtime::request::fetch_callback(
                loader,
                browsing_context,
                request,
                Box::new(move |result| {
                    let result = result
                        .map_err(|error| error.to_string())
                        .map(|response| (requested_url, response));
                    let _ = sender.post(move |page| {
                        if let Err(error) = page.complete_fetch(id, result) {
                            page.async_error = Some(error);
                        }
                    });
                }),
            );
            if let Err(error) = result {
                self.reject_fetch(id, &error.to_string())?;
            }
        }
        Ok(())
    }

    fn handle_fetch_stream_event(
        &self,
        id: u64,
        event: crate::network::ResourceStreamEvent,
    ) -> Result<(), JsException> {
        match event {
            crate::network::ResourceStreamEvent::Headers {
                status,
                mut headers,
                url,
            } => {
                headers.remove(http::header::CONTENT_ENCODING);
                headers.remove(http::header::CONTENT_LENGTH);
                let headers = headers
                    .iter()
                    .filter_map(|(name, value)| {
                        value.to_str().ok().map(|value| [name.as_str(), value])
                    })
                    .collect::<Vec<_>>();
                if let Some(settlement) = self.fetches.borrow_mut().take_settlement(id) {
                    settlement.resolve(
                        &self.js,
                        &serde_json::json!({
                            "streamId": id,
                            "status": status.as_u16(),
                            "statusText": status.canonical_reason().unwrap_or_default(),
                            "headers": headers,
                            "url": url,
                            "redirected": false,
                        })
                        .to_string(),
                    )?;
                }
            }
            crate::network::ResourceStreamEvent::Chunk(bytes) => {
                self.bindings.deliver_fetch_stream_event(
                    &self.js,
                    id,
                    &serde_json::json!({"type": "chunk", "bytes": bytes}).to_string(),
                )?;
            }
            crate::network::ResourceStreamEvent::Complete => {
                self.fetch_buffered.borrow_mut().remove(&id);
                self.fetch_streams.borrow_mut().remove(&id);
                self.bindings.deliver_fetch_stream_event(
                    &self.js,
                    id,
                    &serde_json::json!({"type": "complete"}).to_string(),
                )?;
            }
            crate::network::ResourceStreamEvent::Error(error) => {
                self.fetch_buffered.borrow_mut().remove(&id);
                self.fetch_streams.borrow_mut().remove(&id);
                if let Some(settlement) = self.fetches.borrow_mut().take_settlement(id) {
                    settlement.reject(&self.js, &error.to_string())?;
                } else {
                    self.bindings.deliver_fetch_stream_event(
                        &self.js,
                        id,
                        &serde_json::json!({"type": "error", "message": error.to_string()})
                            .to_string(),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn start_pending_websockets(&mut self) -> Result<(), JsException> {
        let operations = self.streaming_queue.borrow_mut().take_pending();
        for operation in operations {
            match operation {
                PendingWebSocketOperation::Create { id, url } => {
                    let mut headers = HeaderList::new();
                    self.browsing_context.apply_request_identity(&mut headers);
                    let sender = self.task_sender();
                    let handle = self.network_scope.sockets.open_websocket(
                        url,
                        headers,
                        Box::new(move |event| {
                            let event = match event {
                                crate::network::WebSocketEvent::HandshakeRequest{..} | crate::network::WebSocketEvent::HandshakeResponse{..} | crate::network::WebSocketEvent::FrameSent{..} => return,
                                crate::network::WebSocketEvent::Open => serde_json::json!({"type": "open"}),
                                crate::network::WebSocketEvent::Text(data) => serde_json::json!({"type": "message", "data": data}),
                                crate::network::WebSocketEvent::Binary(data) => serde_json::json!({"type": "message", "data": data, "binary": true}),
                                crate::network::WebSocketEvent::Close { code, reason } => serde_json::json!({"type": "close", "code": code, "reason": reason, "wasClean": code == 1000}),
                                crate::network::WebSocketEvent::Error(message) => serde_json::json!({"type": "error", "message": message}),
                            }.to_string();
                            let _ = sender.post(move |page| {
                                if let Err(error) = page.bindings.deliver_websocket_event(&page.js, id, &event) {
                                    page.async_error = Some(error);
                                }
                            });
                        }),
                    );
                    match handle {
                        Ok(handle) => {
                            self.websockets.insert(id, handle);
                        }
                        Err(error) => {
                            self.bindings.deliver_websocket_event(
                                &self.js,
                                id,
                                &serde_json::json!({"type": "error", "message": error.to_string()})
                                    .to_string(),
                            )?;
                        }
                    }
                }
                PendingWebSocketOperation::SendText { id, message } => {
                    if let Some(socket) = self.websockets.get(&id) {
                        let _ = socket.send_text(message);
                    }
                }
                PendingWebSocketOperation::SendBinary { id, message } => {
                    if let Some(socket) = self.websockets.get(&id) {
                        let _ = socket.send_binary(message);
                    }
                }
                PendingWebSocketOperation::Close { id } => {
                    if let Some(socket) = self.websockets.remove(&id) {
                        let _ = socket.close();
                    }
                }
                PendingWebSocketOperation::ConsumeFetch { id, bytes } => {
                    if let Some(buffered) = self.fetch_buffered.borrow().get(&id) {
                        let _ = buffered.try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                            n.checked_sub(bytes)
                        });
                        if buffered.load(Ordering::Acquire) <= 128 * 1024
                            && let Some(stream) = self.fetch_streams.borrow().get(&id)
                        {
                            stream.resume();
                        }
                    }
                }
                PendingWebSocketOperation::CancelFetch { id } => {
                    self.fetch_buffered.borrow_mut().remove(&id);
                    if let Some(stream) = self.fetch_streams.borrow_mut().remove(&id) {
                        stream.cancel();
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn close_websockets(&mut self) {
        for (_, socket) in self.websockets.drain() {
            let _ = socket.close();
        }
    }

    pub(super) fn cancel_fetch_streams(&self) {
        self.fetch_buffered.borrow_mut().clear();
        for (_, stream) in self.fetch_streams.borrow_mut().drain() {
            stream.cancel();
        }
    }

    fn prepare_fetch_request(&self, pending: PendingFetch) -> Result<ResourceRequest, String> {
        let url = match url::Url::parse(&pending.url) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => self
                .browsing_context
                .current_url()
                .ok_or_else(|| "relative fetch URL has no document base URL".to_string())?
                .parse::<url::Url>()
                .and_then(|base| base.join(&pending.url))
                .map_err(|error| error.to_string())?,
            Err(error) => return Err(error.to_string()),
        };
        let method = http::Method::from_bytes(pending.method.as_bytes())
            .map_err(|error| error.to_string())?;
        if (method == http::Method::GET || method == http::Method::HEAD) && pending.body.is_some() {
            return Err(format!("{method} request cannot have a body"));
        }
        let mut request = ResourceRequest::new(method, url.as_str());
        request.resource_type = Some("Fetch".into());
        let headers = serde_json::from_str::<Vec<(String, String)>>(&pending.headers_json)
            .map_err(|error| error.to_string())?;
        for (name, value) in headers {
            let name =
                http::HeaderName::from_bytes(name.as_bytes()).map_err(|error| error.to_string())?;
            let value = http::HeaderValue::from_str(&value).map_err(|error| error.to_string())?;
            request.headers.append(name, value);
        }
        self.add_fetch_metadata(&mut request, pending.referrer.as_deref());
        request.body = pending.body;
        Ok(request)
    }

    /// Headers a browser attaches to a script-issued request (fetch and XHR).
    /// Session identity defaults only fill gaps, so these take precedence over
    /// the document-oriented Accept and leave explicit script values alone.
    fn add_fetch_metadata(&self, request: &mut ResourceRequest, referrer: Option<&str>) {
        // Frame scripts issue requests for their own document, not the page's.
        let document_url = referrer
            .and_then(|url| url::Url::parse(url).ok())
            .or_else(|| {
                self.browsing_context
                    .current_url()
                    .and_then(|url| url::Url::parse(&url).ok())
            });
        let target = url::Url::parse(&request.url).ok();
        let site = match (document_url.as_ref(), target.as_ref()) {
            (None, _) => "none",
            (Some(_), None) => "none",
            (Some(document), Some(target)) if document.origin() == target.origin() => "same-origin",
            (Some(document), Some(target)) if document.host_str() == target.host_str() => {
                "same-site"
            }
            _ => "cross-site",
        };
        let own_origin = document_url
            .as_ref()
            .map(|url| url.origin().ascii_serialization());
        let same_origin = site == "same-origin";
        let mut fields: Vec<(&'static str, String)> = vec![
            ("sec-fetch-site", site.to_owned()),
            ("sec-fetch-mode", "cors".to_owned()),
            ("sec-fetch-dest", "empty".to_owned()),
        ];
        if !request.headers.contains_key("accept") {
            fields.push(("accept", "*/*".to_owned()));
        }
        // Default referrer policy: full URL same-origin, origin only cross-origin.
        if let (Some(document), Some(referrer)) = (document_url.as_ref(), document_url.as_ref()) {
            let value = if same_origin {
                let mut trimmed = referrer.clone();
                trimmed.set_fragment(None);
                trimmed.to_string()
            } else {
                document.origin().ascii_serialization()
            };
            if !request.headers.contains_key("referer") {
                fields.push(("referer", value));
            }
        }
        if request.method != http::Method::GET
            && request.method != http::Method::HEAD
            && !request.headers.contains_key("origin")
            && let Some(origin) = own_origin
        {
            fields.push(("origin", origin));
        }
        for (name, value) in fields {
            let Ok(name) = http::HeaderName::from_bytes(name.as_bytes()) else {
                continue;
            };
            if request.headers.contains_key(name.as_str()) {
                continue;
            }
            if let Ok(value) = http::HeaderValue::from_str(&value) {
                request.headers.append(name, value);
            }
        }
    }

    fn complete_fetch(
        &self,
        id: u64,
        result: Result<(String, crate::network::ResourceResponse), String>,
    ) -> Result<(), JsException> {
        let Some(settlement) = self.fetches.borrow_mut().take_settlement(id) else {
            return Ok(());
        };
        match result {
            Ok((requested_url, response)) => {
                let effective_url = if response.effective_url.is_empty() {
                    requested_url.clone()
                } else {
                    response.effective_url
                };
                let headers = response
                    .headers
                    .iter()
                    .filter_map(|(name, value)| {
                        value.to_str().ok().map(|value| [name.as_str(), value])
                    })
                    .collect::<Vec<_>>();
                let payload = serde_json::json!({
                    "bytes": response.body,
                    "status": response.status.as_u16(),
                    "statusText": response.status.canonical_reason().unwrap_or_default(),
                    "headers": headers,
                    "url": effective_url,
                    "redirected": effective_url != requested_url,
                });
                settlement.resolve(&self.js, &payload.to_string())?;
            }
            Err(error) => settlement.reject(&self.js, &error)?,
        }
        Ok(())
    }

    fn reject_fetch(&self, id: u64, error: &str) -> Result<(), JsException> {
        if let Some(settlement) = self.fetches.borrow_mut().take_settlement(id) {
            settlement.reject(&self.js, error)?;
        }
        Ok(())
    }
}
