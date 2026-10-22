//! Request and response interception with owned values and exactly-once completion.
use super::{
    AutomationError,
    events::{RequestRecord, ResponseRecord, headers},
};
pub use crate::network::MAX_INTERCEPTED_BODY;
use crate::network::{
    self, NetworkError, ResourceInterception, ResourceInterceptionCallback, ResourceInterceptor,
    ResourceRequest, ResourceResponse,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteStage {
    Request,
    Response,
    Auth,
}
pub type RouteHandler = Arc<dyn Fn(Route) + Send + Sync>;
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RouteOverrides {
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<Vec<u8>>,
    pub status: Option<u16>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteInfo {
    pub stage: RouteStage,
    pub request: RequestRecord,
    pub response: Option<ResponseRecord>,
    pub auth_challenge: Option<String>,
}
struct Pending {
    callback: Mutex<Option<ResourceInterceptionCallback>>,
    request: ResourceRequest,
    response: Option<ResourceResponse>,
    info: RouteInfo,
    page_id: u64,
}
impl Pending {
    fn finish(&self, decision: ResourceInterception) -> Result<(), AutomationError> {
        let callback = self.callback.lock().unwrap().take().ok_or_else(|| {
            AutomationError::InvalidInput("route already completed or cancelled".into())
        })?;
        callback(decision);
        Ok(())
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.get_mut().unwrap().take() {
            callback(ResourceInterception::Fail(NetworkError::Cancelled));
        }
    }
}
#[derive(Clone)]
pub struct Route(Arc<Pending>);
impl Route {
    pub fn info(&self) -> &RouteInfo {
        &self.0.info
    }
    pub fn page_id(&self) -> u64 {
        self.0.page_id
    }
    pub fn is_pending(&self) -> bool {
        self.0.callback.lock().unwrap().is_some()
    }
    pub fn body(&self) -> Option<Vec<u8>> {
        self.0
            .response
            .as_ref()
            .map(|response| response.body.clone())
            .or_else(|| self.0.request.body.clone())
    }
    pub fn continue_request(&self, overrides: RouteOverrides) -> Result<(), AutomationError> {
        if let Some(original) = &self.0.response {
            if overrides.url.is_some() || overrides.method.is_some() {
                return Err(AutomationError::InvalidInput(
                    "response continuation cannot change the request URL or method".into(),
                ));
            }
            let mut response = original.clone();
            if let Some(status) = overrides.status {
                response.status = http::StatusCode::from_u16(status).map_err(invalid)?;
            }
            if let Some(headers) = overrides.headers {
                response.headers = parse_headers(headers)?;
            }
            if let Some(body) = overrides.body {
                check_body(&body)?;
                response.body = body;
            }
            self.0.finish(ResourceInterception::Fulfill(response))
        } else {
            if overrides.status.is_some() {
                return Err(AutomationError::InvalidInput(
                    "request continuation cannot change response status".into(),
                ));
            }
            let mut request = self.0.request.clone();
            if let Some(url) = overrides.url {
                url::Url::parse(&url).map_err(invalid)?;
                request.url = url;
            }
            if let Some(method) = overrides.method {
                request.method = http::Method::from_bytes(method.as_bytes()).map_err(invalid)?;
            }
            if let Some(headers) = overrides.headers {
                request.headers = parse_headers(headers)?;
            }
            if let Some(body) = overrides.body {
                check_body(&body)?;
                request.body = Some(body);
            }
            self.0.finish(ResourceInterception::Continue(request))
        }
    }
    pub fn fulfill(
        &self,
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<(), AutomationError> {
        check_body(&body)?;
        self.0
            .finish(ResourceInterception::Fulfill(ResourceResponse {
                status: http::StatusCode::from_u16(status).map_err(invalid)?,
                headers: parse_headers(headers)?,
                body,
                effective_url: self.0.request.url.clone(),
                metadata: Default::default(),
            }))
    }
    pub fn fail(&self, message: impl Into<String>) -> Result<(), AutomationError> {
        self.0
            .finish(ResourceInterception::Fail(NetworkError::Transport(
                message.into(),
            )))
    }
    pub fn authenticate(&self, username: String, password: String) -> Result<(), AutomationError> {
        let response = self
            .0
            .response
            .as_ref()
            .ok_or_else(|| invalid("authentication requires a response challenge"))?;
        if ![401, 407].contains(&response.status.as_u16()) {
            return Err(invalid("response is not an authentication challenge"));
        }
        let proxy = response.status.as_u16() == 407;
        let mut request = self.0.request.clone();
        request.auth.retain(|auth| auth.proxy != proxy);
        request.auth.push(network::HttpCredentials {
            username,
            password,
            proxy,
        });
        self.0.finish(ResourceInterception::Continue(request))
    }
}
fn invalid(error: impl std::fmt::Display) -> AutomationError {
    AutomationError::InvalidInput(error.to_string())
}
fn check_body(body: &[u8]) -> Result<(), AutomationError> {
    if body.len() > MAX_INTERCEPTED_BODY {
        return Err(invalid(format!(
            "intercepted body exceeds {MAX_INTERCEPTED_BODY} bytes"
        )));
    }
    Ok(())
}
pub(crate) fn parse_headers(
    pairs: Vec<(String, String)>,
) -> Result<network::HeaderList, AutomationError> {
    let mut result = network::HeaderList::new();
    for (name, value) in pairs {
        result.append(
            http::HeaderName::from_bytes(name.as_bytes()).map_err(invalid)?,
            http::HeaderValue::from_str(&value).map_err(invalid)?,
        );
    }
    Ok(result)
}
struct Registration {
    pattern: glob::Pattern,
    stage: RouteStage,
    handler: RouteHandler,
}
#[derive(Default)]
struct State {
    routes: BTreeMap<u64, Registration>,
    pending: Vec<Weak<Pending>>,
}
#[derive(Clone, Default)]
pub(crate) struct Routes {
    state: Arc<Mutex<State>>,
    next: Arc<AtomicU64>,
    page_id: Arc<AtomicU64>,
    closed: Arc<AtomicBool>,
}
impl Routes {
    pub fn matches(&self, url: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .routes
            .values()
            .any(|route| route.pattern.matches(url))
    }
    pub fn set_page(&self, id: u64) {
        self.page_id.store(id, Ordering::Release);
    }
    pub fn register(
        &self,
        pattern: &str,
        stage: RouteStage,
        handler: RouteHandler,
    ) -> Result<RouteRegistration, AutomationError> {
        let pattern = glob::Pattern::new(pattern).map_err(invalid)?;
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.lock().unwrap();
        if self.closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        state.routes.insert(
            id,
            Registration {
                pattern,
                stage,
                handler,
            },
        );
        Ok(RouteRegistration {
            routes: self.clone(),
            id,
        })
    }
    pub fn cancel_request(&self, id: u64) {
        let pending = self
            .state
            .lock()
            .unwrap()
            .pending
            .iter()
            .filter_map(|call| call.upgrade())
            .filter(|call| call.info.request.id == id)
            .collect::<Vec<_>>();
        for call in pending {
            let _ = call.finish(ResourceInterception::Fail(NetworkError::Cancelled));
        }
    }
    pub fn cancel_pending(&self) {
        let pending = std::mem::take(&mut self.state.lock().unwrap().pending);
        for call in pending.into_iter().filter_map(|call| call.upgrade()) {
            let _ = call.finish(ResourceInterception::Fail(NetworkError::Cancelled));
        }
    }
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.cancel_pending();
        self.state.lock().unwrap().routes.clear();
    }
    fn handler(&self, url: &str, stage: RouteStage) -> Option<RouteHandler> {
        self.state
            .lock()
            .unwrap()
            .routes
            .values()
            .rev()
            .find(|route| route.stage == stage && route.pattern.matches(url))
            .map(|route| route.handler.clone())
    }
    fn dispatch(
        &self,
        request: ResourceRequest,
        response: Option<ResourceResponse>,
        stage: RouteStage,
        callback: ResourceInterceptionCallback,
    ) {
        if self.closed.load(Ordering::Acquire) {
            callback(ResourceInterception::Fail(NetworkError::Closed));
            return;
        }
        let Some(handler) = self.handler(&request.url, stage) else {
            callback(match response {
                Some(response) => ResourceInterception::Fulfill(response),
                None => ResourceInterception::Continue(request),
            });
            return;
        };
        if response
            .as_ref()
            .is_some_and(|response| response.body.len() > MAX_INTERCEPTED_BODY)
            || request
                .body
                .as_ref()
                .is_some_and(|body| body.len() > MAX_INTERCEPTED_BODY)
        {
            callback(ResourceInterception::Fail(NetworkError::ResponseTooLarge {
                limit: MAX_INTERCEPTED_BODY,
            }));
            return;
        }
        let id = request
            .request_id
            .unwrap_or_else(|| self.next.fetch_add(1, Ordering::Relaxed));
        let info = RouteInfo {
            stage,
            request: RequestRecord {
                id,
                method: request.method.to_string(),
                url: request.url.clone(),
                headers: headers(&request.headers),
                body: request.body.clone(),
                resource_type: request.resource_type.clone(),
            },
            response: response.as_ref().map(|response| ResponseRecord {
                request_id: id,
                url: response.effective_url.clone(),
                status: response.status.as_u16(),
                headers: headers(&response.headers),
                from_cache: false,
            }),
            auth_challenge: response
                .as_ref()
                .and_then(|response| {
                    response.headers.get(if response.status.as_u16() == 407 {
                        "proxy-authenticate"
                    } else {
                        "www-authenticate"
                    })
                })
                .map(|value| String::from_utf8_lossy(value.as_bytes()).into_owned()),
        };
        let pending = Arc::new(Pending {
            callback: Mutex::new(Some(callback)),
            request,
            response,
            info,
            page_id: self.page_id.load(Ordering::Acquire),
        });
        {
            let mut state = self.state.lock().unwrap();
            if self.closed.load(Ordering::Acquire) {
                drop(state);
                let _ = pending.finish(ResourceInterception::Fail(NetworkError::Closed));
                return;
            }
            state.pending.retain(|call| call.strong_count() > 0);
            state.pending.push(Arc::downgrade(&pending));
        }
        let _guard = super::plugins::enter_page_callback(pending.page_id);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(Route(pending))));
    }
}
impl ResourceInterceptor for Routes {
    fn intercept(&self, request: ResourceRequest, callback: ResourceInterceptionCallback) {
        self.dispatch(request, None, RouteStage::Request, callback);
    }
    fn intercepts_response(&self, request: &ResourceRequest) -> bool {
        self.handler(&request.url, RouteStage::Response).is_some()
            || self.handler(&request.url, RouteStage::Auth).is_some()
    }
    fn intercept_response(
        &self,
        request: ResourceRequest,
        response: ResourceResponse,
        callback: ResourceInterceptionCallback,
    ) {
        let stage = if [401, 407].contains(&response.status.as_u16())
            && self.handler(&request.url, RouteStage::Auth).is_some()
        {
            RouteStage::Auth
        } else {
            RouteStage::Response
        };
        self.dispatch(request, Some(response), stage, callback);
    }
}
pub struct RouteRegistration {
    routes: Routes,
    id: u64,
}
impl RouteRegistration {
    pub fn remove(&self) {
        self.routes.state.lock().unwrap().routes.remove(&self.id);
    }
}
impl Drop for RouteRegistration {
    fn drop(&mut self) {
        self.remove();
    }
}
