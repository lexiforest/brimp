use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base64::Engine;
use http::{HeaderName, HeaderValue, Method, StatusCode};
use network::{
    HeaderList, NetworkError, ResourceInterception, ResourceInterceptionCallback,
    ResourceInterceptor, ResourceRequest, ResourceResponse,
};
use serde_json::{Map, Value, json};
use tokio::sync::mpsc;

use crate::protocol::{Event, Request, Response};

const MAX_FULFILL_BODY: usize = 32 * 1024 * 1024;
const MAX_PENDING_REQUESTS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InterceptionMode {
    Fetch,
    Network,
}

#[derive(Clone)]
struct Configuration {
    session_id: String,
    mode: InterceptionMode,
    patterns: Vec<String>,
}

struct PendingRequest {
    target_id: String,
    session_id: String,
    mode: InterceptionMode,
    request: ResourceRequest,
    callback: ResourceInterceptionCallback,
}

struct State {
    next_id: u64,
    configurations: HashMap<String, Configuration>,
    pending: HashMap<String, PendingRequest>,
}

struct Inner {
    state: Mutex<State>,
    paused: mpsc::UnboundedSender<PausedRequest>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        let pending = self
            .state
            .get_mut()
            .unwrap()
            .pending
            .drain()
            .collect::<Vec<_>>();
        for (_, pending) in pending {
            (pending.callback)(ResourceInterception::Fail(NetworkError::Closed));
        }
    }
}

#[derive(Clone)]
pub(crate) struct InterceptionRegistry(Arc<Inner>);

pub(crate) struct PausedRequest {
    request_id: String,
    target_id: String,
    session_id: String,
    mode: InterceptionMode,
    request: ResourceRequest,
}

impl PausedRequest {
    pub(crate) fn event(self) -> Event {
        let headers = self
            .request
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_owned(),
                    json!(value.to_str().unwrap_or_default()),
                )
            })
            .collect::<Map<_, _>>();
        let mut request = json!({
            "url": self.request.url,
            "method": self.request.method.as_str(),
            "headers": headers,
            "initialPriority": "High",
            "referrerPolicy": "no-referrer-when-downgrade",
        });
        if let Some(body) = self.request.body {
            request["postData"] = json!(String::from_utf8_lossy(&body));
            request["hasPostData"] = json!(true);
        }
        let (method, params) = match self.mode {
            InterceptionMode::Fetch => (
                "Fetch.requestPaused",
                json!({
                    "requestId": self.request_id,
                    "request": request,
                    "frameId": self.target_id,
                    "resourceType": "Other",
                }),
            ),
            InterceptionMode::Network => (
                "Network.requestIntercepted",
                json!({
                    "interceptionId": self.request_id,
                    "request": request,
                    "frameId": self.target_id,
                    "resourceType": "Other",
                    "isNavigationRequest": false,
                }),
            ),
        };
        Event {
            method: method.into(),
            params,
            session_id: Some(self.session_id),
        }
    }
}

pub(crate) struct TargetInterceptor {
    target_id: String,
    registry: InterceptionRegistry,
}

impl ResourceInterceptor for TargetInterceptor {
    fn intercept(&self, request: ResourceRequest, callback: ResourceInterceptionCallback) {
        self.registry
            .intercept(self.target_id.clone(), request, callback);
    }
}

impl InterceptionRegistry {
    pub(crate) fn new() -> (Self, mpsc::UnboundedReceiver<PausedRequest>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (
            Self(Arc::new(Inner {
                state: Mutex::new(State {
                    next_id: 1,
                    configurations: HashMap::new(),
                    pending: HashMap::new(),
                }),
                paused: sender,
            })),
            receiver,
        )
    }

    pub(crate) fn target_interceptor(&self, target_id: String) -> Arc<dyn ResourceInterceptor> {
        Arc::new(TargetInterceptor {
            target_id,
            registry: self.clone(),
        })
    }

    pub(crate) fn enable(
        &self,
        target_id: String,
        session_id: String,
        mode: InterceptionMode,
        patterns: Vec<String>,
    ) {
        self.disable(&target_id, InterceptionMode::Fetch);
        self.disable(&target_id, InterceptionMode::Network);
        self.0.state.lock().unwrap().configurations.insert(
            target_id,
            Configuration {
                session_id,
                mode,
                patterns,
            },
        );
    }

    pub(crate) fn disable(&self, target_id: &str, mode: InterceptionMode) {
        let pending = {
            let mut state = self.0.state.lock().unwrap();
            if state
                .configurations
                .get(target_id)
                .is_some_and(|configuration| configuration.mode == mode)
            {
                state.configurations.remove(target_id);
            }
            let ids = state
                .pending
                .iter()
                .filter_map(|(id, pending)| {
                    (pending.target_id == target_id && pending.mode == mode).then(|| id.clone())
                })
                .collect::<Vec<_>>();
            ids.into_iter()
                .filter_map(|id| state.pending.remove(&id))
                .collect::<Vec<_>>()
        };
        for pending in pending {
            let request = pending.request;
            (pending.callback)(ResourceInterception::Continue(request));
        }
    }

    pub(crate) fn remove_target(&self, target_id: &str) {
        let modes = [InterceptionMode::Fetch, InterceptionMode::Network];
        for mode in modes {
            self.disable(target_id, mode);
        }
    }

    pub(crate) fn remove_session(&self, target_id: &str, session_id: &str) {
        let mode = self
            .0
            .state
            .lock()
            .unwrap()
            .configurations
            .get(target_id)
            .filter(|configuration| configuration.session_id == session_id)
            .map(|configuration| configuration.mode);
        if let Some(mode) = mode {
            self.disable(target_id, mode);
        }
    }

    fn remove_session_configurations(&self, session_id: &str) {
        let configurations = self
            .0
            .state
            .lock()
            .unwrap()
            .configurations
            .iter()
            .filter(|(_, configuration)| configuration.session_id == session_id)
            .map(|(target, configuration)| (target.clone(), configuration.mode))
            .collect::<Vec<_>>();
        for (target, mode) in configurations {
            self.disable(&target, mode);
        }
    }

    pub(crate) fn prepare_queued_command(&self, request: &Request) {
        match request.method.as_str() {
            "Fetch.enable" | "Fetch.disable" => {
                if let Some(session_id) = request.session_id.as_deref() {
                    self.remove_session_configurations(session_id);
                }
            }
            "Network.setRequestInterception"
                if request
                    .params
                    .get("patterns")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty) =>
            {
                if let Some(session_id) = request.session_id.as_deref() {
                    self.remove_session_configurations(session_id);
                }
            }
            "Target.detachFromTarget" => {
                if let Some(session_id) = request.params.get("sessionId").and_then(Value::as_str) {
                    self.remove_session_configurations(session_id);
                }
            }
            "Target.closeTarget" => {
                if let Some(target_id) = request.params.get("targetId").and_then(Value::as_str) {
                    self.remove_target(target_id);
                }
            }
            _ => {}
        }
    }

    fn intercept(
        &self,
        target_id: String,
        request: ResourceRequest,
        callback: ResourceInterceptionCallback,
    ) {
        let paused = {
            let mut state = self.0.state.lock().unwrap();
            let Some(configuration) = state.configurations.get(&target_id).cloned() else {
                drop(state);
                callback(ResourceInterception::Continue(request));
                return;
            };
            if !configuration
                .patterns
                .iter()
                .any(|pattern| wildcard(pattern, &request.url))
            {
                drop(state);
                callback(ResourceInterception::Continue(request));
                return;
            }
            if state.pending.len() == MAX_PENDING_REQUESTS {
                drop(state);
                callback(ResourceInterception::Fail(NetworkError::QueueFull));
                return;
            }
            let request_id = format!("interception-{}", state.next_id);
            state.next_id += 1;
            state.pending.insert(
                request_id.clone(),
                PendingRequest {
                    target_id: target_id.clone(),
                    session_id: configuration.session_id.clone(),
                    mode: configuration.mode,
                    request: request.clone(),
                    callback,
                },
            );
            PausedRequest {
                request_id,
                target_id,
                session_id: configuration.session_id,
                mode: configuration.mode,
                request,
            }
        };
        if self.0.paused.send(paused).is_err() {
            self.fail_all(NetworkError::Closed);
        }
    }

    fn fail_all(&self, error: NetworkError) {
        let pending = self
            .0
            .state
            .lock()
            .unwrap()
            .pending
            .drain()
            .map(|(_, pending)| pending)
            .collect::<Vec<_>>();
        for pending in pending {
            (pending.callback)(ResourceInterception::Fail(error.clone()));
        }
    }

    pub(crate) fn shutdown(&self) {
        self.fail_all(NetworkError::Closed);
        self.0.state.lock().unwrap().configurations.clear();
    }

    pub(crate) fn handle_control(&self, request: &Request) -> Option<Response> {
        let result = match request.method.as_str() {
            "Fetch.continueRequest" => {
                self.continue_request(&request.params, request.session_id.as_deref(), false)
            }
            "Fetch.failRequest" => {
                self.fail_request(&request.params, request.session_id.as_deref(), false)
            }
            "Fetch.fulfillRequest" => {
                self.fulfill_request(&request.params, request.session_id.as_deref())
            }
            "Network.continueInterceptedRequest" => {
                self.continue_legacy_request(&request.params, request.session_id.as_deref())
            }
            _ => return None,
        };
        Some(match result {
            Ok(()) => Response::success(request, json!({})),
            Err(message) => Response::error(request, -32602, message),
        })
    }

    fn take_pending(
        &self,
        params: &Value,
        session_id: Option<&str>,
        legacy: bool,
    ) -> Result<PendingRequest, String> {
        let name = if legacy {
            "interceptionId"
        } else {
            "requestId"
        };
        let id = params
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name} must be a string"))?;
        let mut state = self.0.state.lock().unwrap();
        let expected_mode = if legacy {
            InterceptionMode::Network
        } else {
            InterceptionMode::Fetch
        };
        let pending = state
            .pending
            .get(id)
            .ok_or_else(|| format!("unknown intercepted request {id}"))?;
        if pending.mode != expected_mode {
            return Err(format!(
                "intercepted request {id} belongs to another domain"
            ));
        }
        if Some(pending.session_id.as_str()) != session_id {
            return Err(format!(
                "intercepted request {id} belongs to another session"
            ));
        }
        Ok(state.pending.remove(id).unwrap())
    }

    fn continue_request(
        &self,
        params: &Value,
        session_id: Option<&str>,
        legacy: bool,
    ) -> Result<(), String> {
        let mut pending = self.take_pending(params, session_id, legacy)?;
        if let Err(message) = apply_request_overrides(&mut pending.request, params, legacy) {
            (pending.callback)(ResourceInterception::Fail(NetworkError::InvalidRequest(
                message.clone(),
            )));
            return Err(message);
        }
        let request = pending.request;
        (pending.callback)(ResourceInterception::Continue(request));
        Ok(())
    }

    fn fail_request(
        &self,
        params: &Value,
        session_id: Option<&str>,
        legacy: bool,
    ) -> Result<(), String> {
        let pending = self.take_pending(params, session_id, legacy)?;
        let reason = params
            .get("errorReason")
            .and_then(Value::as_str)
            .unwrap_or("Failed");
        let error = if reason == "Aborted" {
            NetworkError::Cancelled
        } else {
            NetworkError::Transport(format!("request interception failed: {reason}"))
        };
        (pending.callback)(ResourceInterception::Fail(error));
        Ok(())
    }

    fn fulfill_request(&self, params: &Value, session_id: Option<&str>) -> Result<(), String> {
        let status = params
            .get("responseCode")
            .and_then(Value::as_u64)
            .and_then(|status| u16::try_from(status).ok())
            .and_then(|status| StatusCode::from_u16(status).ok())
            .ok_or_else(|| "responseCode must be a valid HTTP status".to_owned())?;
        let headers = header_entries(params.get("responseHeaders"))?;
        let body = params
            .get("body")
            .and_then(Value::as_str)
            .map(|body| {
                base64::engine::general_purpose::STANDARD
                    .decode(body)
                    .map_err(|error| format!("body must be base64: {error}"))
            })
            .transpose()?
            .unwrap_or_default();
        if body.len() > MAX_FULFILL_BODY {
            return Err("fulfilled response body exceeds 32 MiB".into());
        }
        let pending = self.take_pending(params, session_id, false)?;
        let effective_url = pending.request.url.clone();
        (pending.callback)(ResourceInterception::Fulfill(ResourceResponse {
            status,
            headers,
            body,
            effective_url,
        }));
        Ok(())
    }

    fn continue_legacy_request(
        &self,
        params: &Value,
        session_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(raw_response) = params.get("rawResponse") {
            let response = parse_raw_response(raw_response)?;
            let pending = self.take_pending(params, session_id, true)?;
            (pending.callback)(ResourceInterception::Fulfill(ResourceResponse {
                status: response.0,
                headers: response.1,
                body: response.2,
                effective_url: pending.request.url,
            }));
            return Ok(());
        }
        if params.get("authChallengeResponse").is_some() {
            return Err("authentication interception is not supported".into());
        }
        if params.get("errorReason").is_some() {
            self.fail_request(params, session_id, true)
        } else {
            self.continue_request(params, session_id, true)
        }
    }
}

fn parse_raw_response(value: &Value) -> Result<(StatusCode, HeaderList, Vec<u8>), String> {
    let encoded = value
        .as_str()
        .ok_or_else(|| "rawResponse must be a base64 string".to_owned())?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| format!("rawResponse must be base64: {error}"))?;
    if raw.len() > MAX_FULFILL_BODY {
        return Err("rawResponse exceeds 32 MiB".into());
    }
    let boundary = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "rawResponse is missing the HTTP header boundary".to_owned())?;
    let head = std::str::from_utf8(&raw[..boundary])
        .map_err(|_| "rawResponse headers must be UTF-8".to_owned())?;
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .and_then(|status| StatusCode::from_u16(status).ok())
        .ok_or_else(|| "rawResponse has an invalid HTTP status line".to_owned())?;
    let mut headers = HeaderList::new();
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "rawResponse contains an invalid header".to_owned())?;
        headers.append(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| format!("invalid rawResponse header name: {error}"))?,
            HeaderValue::from_str(value.trim())
                .map_err(|error| format!("invalid rawResponse header value: {error}"))?,
        );
    }
    Ok((status, headers, raw[boundary + 4..].to_vec()))
}

fn apply_request_overrides(
    request: &mut ResourceRequest,
    params: &Value,
    legacy: bool,
) -> Result<(), String> {
    if let Some(url) = params.get("url") {
        request.url = url
            .as_str()
            .ok_or_else(|| "url must be a string".to_owned())?
            .to_owned();
        url::Url::parse(&request.url).map_err(|error| format!("invalid URL: {error}"))?;
    }
    if let Some(method) = params.get("method") {
        request.method = Method::from_bytes(
            method
                .as_str()
                .ok_or_else(|| "method must be a string".to_owned())?
                .as_bytes(),
        )
        .map_err(|error| format!("invalid method: {error}"))?;
    }
    if let Some(post_data) = params.get("postData") {
        let post_data = post_data
            .as_str()
            .ok_or_else(|| "postData must be a base64 string".to_owned())?;
        request.body = Some(
            base64::engine::general_purpose::STANDARD
                .decode(post_data)
                .map_err(|error| format!("postData must be base64: {error}"))?,
        );
    }
    if let Some(headers) = params.get("headers") {
        request.headers = if legacy {
            header_object(Some(headers))?
        } else {
            header_entries(Some(headers))?
        };
    }
    Ok(())
}

fn header_entries(value: Option<&Value>) -> Result<HeaderList, String> {
    let mut headers = HeaderList::new();
    let Some(entries) = value else {
        return Ok(headers);
    };
    let entries = entries
        .as_array()
        .ok_or_else(|| "headers must be an array".to_owned())?;
    for entry in entries {
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "header name must be a string".to_owned())?;
        let value = entry
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| "header value must be a string".to_owned())?;
        headers.append(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| format!("invalid header name: {error}"))?,
            HeaderValue::from_str(value)
                .map_err(|error| format!("invalid header value: {error}"))?,
        );
    }
    Ok(headers)
}

fn header_object(value: Option<&Value>) -> Result<HeaderList, String> {
    let mut headers = HeaderList::new();
    let Some(entries) = value else {
        return Ok(headers);
    };
    let entries = entries
        .as_object()
        .ok_or_else(|| "headers must be an object".to_owned())?;
    for (name, value) in entries {
        let value = value
            .as_str()
            .ok_or_else(|| "header value must be a string".to_owned())?;
        headers.append(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| format!("invalid header name: {error}"))?,
            HeaderValue::from_str(value)
                .map_err(|error| format!("invalid header value: {error}"))?,
        );
    }
    Ok(headers)
}

fn wildcard(pattern: &str, value: &str) -> bool {
    let (mut pattern_index, mut value_index) = (0, 0);
    let (mut star, mut matched) = (None, 0);
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star = Some(pattern_index);
            matched = value_index;
            pattern_index += 1;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            matched += 1;
            value_index = matched;
        } else {
            return false;
        }
    }
    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc as std_mpsc;

    use base64::Engine;
    use http::Method;
    use network::{ResourceInterception, ResourceRequest};
    use serde_json::json;

    use super::{InterceptionMode, InterceptionRegistry, wildcard};
    use crate::protocol::Request;

    #[test]
    fn matches_cdp_url_patterns() {
        assert!(wildcard("*", "https://example.test/a"));
        assert!(wildcard("https://*.test/*", "https://example.test/a"));
        assert!(!wildcard("https://*.test/api", "https://example.test/a"));
    }

    #[test]
    fn continue_request_applies_fetch_overrides() {
        let (registry, mut paused) = InterceptionRegistry::new();
        registry.enable(
            "page-1".into(),
            "session-1".into(),
            InterceptionMode::Fetch,
            vec!["*".into()],
        );
        let (sender, receiver) = std_mpsc::channel();
        registry.intercept(
            "page-1".into(),
            ResourceRequest::get("https://example.test/original"),
            Box::new(move |decision| sender.send(decision).unwrap()),
        );
        let request_id = paused.try_recv().unwrap().request_id;
        let response = registry
            .handle_control(&Request {
                id: 1,
                method: "Fetch.continueRequest".into(),
                params: json!({
                    "requestId": request_id,
                    "url": "https://example.test/modified",
                    "method": "POST",
                    "postData": base64::engine::general_purpose::STANDARD.encode("agent body"),
                    "headers": [{"name": "x-agent", "value": "ready"}]
                }),
                session_id: Some("session-1".into()),
            })
            .unwrap();
        assert!(response.error.is_none());
        let ResourceInterception::Continue(request) = receiver.recv().unwrap() else {
            panic!("request was not continued");
        };
        assert_eq!(request.url, "https://example.test/modified");
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.body.as_deref(), Some(b"agent body".as_slice()));
        assert_eq!(request.headers["x-agent"], "ready");
    }

    #[test]
    fn disabling_fetch_releases_paused_requests() {
        let (registry, mut paused) = InterceptionRegistry::new();
        registry.enable(
            "page-1".into(),
            "session-1".into(),
            InterceptionMode::Fetch,
            vec!["*".into()],
        );
        let (sender, receiver) = std_mpsc::channel();
        registry.intercept(
            "page-1".into(),
            ResourceRequest::get("https://example.test/"),
            Box::new(move |decision| sender.send(decision).unwrap()),
        );
        paused.try_recv().unwrap();
        registry.prepare_queued_command(&Request {
            id: 1,
            method: "Fetch.disable".into(),
            params: json!({}),
            session_id: Some("session-1".into()),
        });
        assert!(matches!(
            receiver.recv().unwrap(),
            ResourceInterception::Continue(_)
        ));
    }
}
