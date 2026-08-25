use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::{Value, json};
use web_runtime::{AutomationBrowser, AutomationError, AutomationPage, PageOptions};

use crate::protocol::{Event, Request, Response};

const MAX_EVENTS: usize = 256;
const MAX_TARGETS: usize = 32;
const MAX_SESSIONS: usize = 64;

pub(crate) struct ConnectionState {
    browser: Arc<AutomationBrowser>,
    pages: HashMap<String, PageTarget>,
    sessions: HashMap<String, String>,
    events: EventQueue,
    next_page_id: u64,
    next_session_id: u64,
    discover_targets: bool,
    enabled_pages: HashSet<String>,
    enabled_runtimes: HashSet<String>,
    lifecycle_events: HashSet<String>,
    waiting_for_debugger: HashSet<String>,
    auto_attach: HashMap<Option<String>, AutoAttach>,
}

impl ConnectionState {
    pub(crate) fn new(browser: Arc<AutomationBrowser>) -> Self {
        Self {
            browser,
            pages: HashMap::new(),
            sessions: HashMap::new(),
            events: EventQueue::default(),
            next_page_id: 1,
            next_session_id: 1,
            discover_targets: false,
            enabled_pages: HashSet::new(),
            enabled_runtimes: HashSet::new(),
            lifecycle_events: HashSet::new(),
            waiting_for_debugger: HashSet::new(),
            auto_attach: HashMap::new(),
        }
    }

    pub(crate) async fn dispatch(&mut self, request: &Request) -> Response {
        let result = match request.method.as_str() {
            "Browser.getVersion" => Ok(json!({
                "protocolVersion": "1.3",
                "product": "Brimp/0.1.0",
                "revision": env!("CARGO_PKG_VERSION"),
                "userAgent": "Brimp/0.1.0",
                "jsVersion": "JavaScriptCore"
            })),
            "Target.getBrowserContexts" => Ok(json!({"browserContextIds": []})),
            "Target.setDiscoverTargets" => self.set_discover_targets(request),
            "Target.setAutoAttach" => self.set_auto_attach(request),
            "Target.getTargets" => Ok(json!({"targetInfos": self.target_infos()})),
            "Target.getTargetInfo" => self.get_target_info(request),
            "Target.createTarget" => self.create_target(request).await,
            "Target.attachToTarget" => self.attach_to_target(request),
            "Target.detachFromTarget" => self.detach_from_target(request),
            "Target.closeTarget" => self.close_target(request),
            "Page.enable" => self.enable_page(request),
            "Page.setLifecycleEventsEnabled" => self.set_lifecycle_events(request),
            "Page.navigate" => self.navigate(request).await,
            "Page.captureScreenshot" => self.capture_screenshot(request).await,
            "Runtime.enable" => self.enable_runtime(request),
            "Runtime.evaluate" => self.evaluate(request).await,
            "Runtime.callFunctionOn" => self.call_function(request).await,
            "Runtime.runIfWaitingForDebugger" => self.run_if_waiting(request),
            _ => Err(DispatchError::method_not_found(format!(
                "Method not found: {}",
                request.method
            ))),
        };
        match result {
            Ok(value) => Response::success(request, value),
            Err(error) => Response::error(request, error.code, error.message),
        }
    }

    fn set_auto_attach(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let auto_attach = bool_param(&request.params, "autoAttach")?;
        let wait_for_debugger = bool_param(&request.params, "waitForDebuggerOnStart")?;
        if request.params.get("flatten").and_then(Value::as_bool) != Some(true) {
            return Err(DispatchError::invalid_params("flatten must be true"));
        }
        let excluded_page = request
            .params
            .get("filter")
            .and_then(Value::as_array)
            .is_some_and(|filters| {
                filters.iter().any(|filter| {
                    filter.get("type").and_then(Value::as_str) == Some("page")
                        && filter.get("exclude").and_then(Value::as_bool) == Some(true)
                })
            });
        self.auto_attach.insert(
            request.session_id.clone(),
            AutoAttach {
                auto_attach,
                wait_for_debugger,
                excluded_page,
            },
        );
        Ok(json!({}))
    }

    fn get_target_info(&self, request: &Request) -> Result<Value, DispatchError> {
        let target_id = request
            .params
            .get("targetId")
            .and_then(Value::as_str)
            .ok_or_else(|| DispatchError::invalid_params("targetId must be a string"))?;
        if !self.pages.contains_key(target_id) {
            return Err(DispatchError::invalid_params("unknown targetId"));
        }
        let target = &self.pages[target_id];
        Ok(
            json!({"targetInfo": target_info(target_id, &target.url, &target.title, self.sessions.values().any(|candidate| candidate == target_id))}),
        )
    }

    pub(crate) fn take_events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events.drain()
    }

    fn set_discover_targets(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let discover = bool_param(&request.params, "discover")?;
        self.discover_targets = discover;
        if discover {
            for target_info in self.target_infos() {
                self.push_event(Event {
                    method: "Target.targetCreated".into(),
                    params: json!({"targetInfo": target_info}),
                    session_id: None,
                })?;
            }
        }
        Ok(json!({}))
    }

    async fn create_target(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let url = request
            .params
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("about:blank");
        if url != "about:blank" {
            return Err(DispatchError::invalid_params(
                "Target.createTarget currently accepts only about:blank; navigate with Page.navigate",
            ));
        }
        if self.pages.len() == MAX_TARGETS {
            return Err(DispatchError::invalid_request("page target limit exceeded"));
        }
        let browser = Arc::clone(&self.browser);
        let page = tokio::task::spawn_blocking(move || browser.new_page(PageOptions::default()))
            .await
            .map_err(internal_join)??;
        let target_id = format!("page-{}", self.next_page_id);
        self.next_page_id += 1;
        self.pages.insert(
            target_id.clone(),
            PageTarget {
                page,
                url: "about:blank".into(),
                title: String::new(),
            },
        );
        if self.discover_targets {
            self.push_event(Event {
                method: "Target.targetCreated".into(),
                params: json!({"targetInfo": target_info(&target_id, "about:blank", "", false)}),
                session_id: None,
            })?;
        }
        if self
            .auto_attach
            .get(&None)
            .is_some_and(|config| config.auto_attach && !config.excluded_page)
        {
            let wait = self.auto_attach[&None].wait_for_debugger;
            self.attach(&target_id, wait)?;
        }
        Ok(json!({"targetId": target_id}))
    }

    fn attach_to_target(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let target_id = string_param(&request.params, "targetId")?;
        if !self.pages.contains_key(target_id) {
            return Err(DispatchError::invalid_params("unknown targetId"));
        }
        if request.params.get("flatten").and_then(Value::as_bool) != Some(true) {
            return Err(DispatchError::invalid_params("flatten must be true"));
        }
        let session_id = self.attach(target_id, false)?;
        Ok(json!({"sessionId": session_id}))
    }

    fn attach(
        &mut self,
        target_id: &str,
        waiting_for_debugger: bool,
    ) -> Result<String, DispatchError> {
        if self.sessions.len() == MAX_SESSIONS {
            return Err(DispatchError::invalid_request("session limit exceeded"));
        }
        let target = self
            .pages
            .get(target_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown targetId"))?;
        let target_info = target_info(target_id, &target.url, &target.title, true);
        let session_id = format!("session-{}", self.next_session_id);
        self.next_session_id += 1;
        self.sessions
            .insert(session_id.clone(), target_id.to_owned());
        if waiting_for_debugger {
            self.waiting_for_debugger.insert(session_id.clone());
        }
        self.push_event(Event {
            method: "Target.attachedToTarget".into(),
            params: json!({
                "sessionId": session_id,
                "targetInfo": target_info,
                "waitingForDebugger": waiting_for_debugger
            }),
            session_id: None,
        })?;
        Ok(session_id)
    }

    fn detach_from_target(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session_id = string_param(&request.params, "sessionId")?.to_owned();
        let target_id = self
            .sessions
            .remove(&session_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown sessionId"))?;
        self.enabled_pages.remove(&session_id);
        self.enabled_runtimes.remove(&session_id);
        self.lifecycle_events.remove(&session_id);
        self.waiting_for_debugger.remove(&session_id);
        self.auto_attach.remove(&Some(session_id.clone()));
        self.push_event(Event {
            method: "Target.detachedFromTarget".into(),
            params: json!({"sessionId": session_id, "targetId": target_id}),
            session_id: request.session_id.clone(),
        })?;
        Ok(json!({}))
    }

    fn close_target(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let target_id = string_param(&request.params, "targetId")?.to_owned();
        let target = self
            .pages
            .remove(&target_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown targetId"))?;
        target.page.close();
        let detached: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, page)| **page == target_id)
            .map(|(id, _)| id.clone())
            .collect();
        for session_id in detached {
            self.sessions.remove(&session_id);
            self.enabled_pages.remove(&session_id);
            self.enabled_runtimes.remove(&session_id);
            self.lifecycle_events.remove(&session_id);
            self.waiting_for_debugger.remove(&session_id);
            self.auto_attach.remove(&Some(session_id.clone()));
            self.push_event(Event {
                method: "Target.detachedFromTarget".into(),
                params: json!({"sessionId": session_id, "targetId": target_id}),
                session_id: None,
            })?;
        }
        if self.discover_targets {
            self.push_event(Event {
                method: "Target.targetDestroyed".into(),
                params: json!({"targetId": target_id}),
                session_id: None,
            })?;
        }
        Ok(json!({"success": true}))
    }

    fn enable_page(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        self.enabled_pages.insert(session.to_owned());
        Ok(json!({}))
    }

    fn set_lifecycle_events(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        if bool_param(&request.params, "enabled")? {
            self.lifecycle_events.insert(session);
        } else {
            self.lifecycle_events.remove(&session);
        }
        Ok(json!({}))
    }

    async fn navigate(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        if !self.enabled_pages.contains(&session) {
            return Err(DispatchError::invalid_request(
                "Page.enable must be called first",
            ));
        }
        let url = string_param(&request.params, "url")?.to_owned();
        let navigated_url = url.clone();
        let page = self.page_for_session(&session)?.clone();
        let title = tokio::task::spawn_blocking(move || {
            page.navigate(url, Duration::from_secs(30))?;
            page.title()
        })
        .await
        .map_err(internal_join)??;
        let frame_id = self.target_for_session(&session)?.to_owned();
        if let Some(target) = self.pages.get_mut(&frame_id) {
            target.url = navigated_url;
            target.title = title;
        }
        let loader_id = format!("loader-{frame_id}");
        if self.lifecycle_events.contains(&session) {
            for name in ["init", "DOMContentLoaded", "load"] {
                self.push_event(Event {
                    method: "Page.lifecycleEvent".into(),
                    params: json!({"frameId": frame_id, "loaderId": loader_id, "name": name, "timestamp": timestamp()}),
                    session_id: Some(session.clone()),
                })?;
            }
        }
        self.push_event(Event {
            method: "Page.loadEventFired".into(),
            params: json!({"timestamp": timestamp()}),
            session_id: Some(session),
        })?;
        Ok(json!({"frameId": frame_id, "loaderId": loader_id}))
    }

    async fn capture_screenshot(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let format = request
            .params
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("png");
        if format != "png" {
            return Err(DispatchError::invalid_params(
                "only PNG screenshots are supported",
            ));
        }
        let page = self.page_for_session(session)?.clone();
        let bytes = tokio::task::spawn_blocking(move || page.screenshot(false))
            .await
            .map_err(internal_join)??;
        Ok(json!({"data": base64::engine::general_purpose::STANDARD.encode(bytes)}))
    }

    fn enable_runtime(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_runtimes.insert(session.clone());
        let target = self.target_for_session(&session)?.to_owned();
        self.push_event(Event {
            method: "Runtime.executionContextCreated".into(),
            params: json!({"context": {
                "id": 1,
                "origin": "",
                "name": "",
                "uniqueId": format!("context-{target}"),
                "auxData": {"isDefault": true, "type": "default", "frameId": target}
            }}),
            session_id: Some(session),
        })?;
        Ok(json!({}))
    }

    async fn evaluate(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        if !self.enabled_runtimes.contains(session) {
            return Err(DispatchError::invalid_request(
                "Runtime.enable must be called first",
            ));
        }
        let expression = string_param(&request.params, "expression")?.to_owned();
        self.evaluate_expression(session, expression).await
    }

    async fn call_function(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        if !self.enabled_runtimes.contains(session) {
            return Err(DispatchError::invalid_request(
                "Runtime.enable must be called first",
            ));
        }
        let declaration = string_param(&request.params, "functionDeclaration")?;
        let arguments = request
            .params
            .get("arguments")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let values = arguments
            .into_iter()
            .map(|argument| {
                argument.get("value").cloned().ok_or_else(|| {
                    DispatchError::invalid_params("only by-value function arguments are supported")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let arguments = values
            .into_iter()
            .map(|value| {
                serde_json::to_string(&value)
                    .map_err(|error| DispatchError::internal(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(",");
        self.evaluate_expression(session, format!("({declaration})({arguments})"))
            .await
    }

    fn run_if_waiting(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.waiting_for_debugger.remove(&session);
        Ok(json!({}))
    }

    async fn evaluate_expression(
        &self,
        session: &str,
        expression: String,
    ) -> Result<Value, DispatchError> {
        let page = self.page_for_session(session)?.clone();
        let value = tokio::task::spawn_blocking(move || page.evaluate(expression))
            .await
            .map_err(internal_join)??;
        Ok(json!({"result": remote_object(value)}))
    }

    fn session<'a>(&self, request: &'a Request) -> Result<&'a str, DispatchError> {
        let session = request
            .session_id
            .as_deref()
            .ok_or_else(|| DispatchError::invalid_request("sessionId is required"))?;
        if !self.sessions.contains_key(session) {
            return Err(DispatchError::invalid_request("unknown sessionId"));
        }
        Ok(session)
    }

    fn target_for_session(&self, session: &str) -> Result<&str, DispatchError> {
        self.sessions
            .get(session)
            .map(String::as_str)
            .ok_or_else(|| DispatchError::invalid_request("unknown sessionId"))
    }

    fn page_for_session(&self, session: &str) -> Result<&AutomationPage, DispatchError> {
        let target = self.target_for_session(session)?;
        self.pages
            .get(target)
            .map(|target| &target.page)
            .ok_or_else(|| DispatchError::invalid_request("session target is closed"))
    }

    fn target_infos(&self) -> Vec<Value> {
        self.pages
            .iter()
            .map(|(id, target)| {
                target_info(
                    id,
                    &target.url,
                    &target.title,
                    self.sessions.values().any(|candidate| candidate == id),
                )
            })
            .collect()
    }

    fn push_event(&mut self, event: Event) -> Result<(), DispatchError> {
        self.events.push(event)
    }
}

#[derive(Clone, Copy)]
struct AutoAttach {
    auto_attach: bool,
    wait_for_debugger: bool,
    excluded_page: bool,
}

struct PageTarget {
    page: AutomationPage,
    url: String,
    title: String,
}

#[derive(Default)]
struct EventQueue(VecDeque<Event>);
impl EventQueue {
    fn push(&mut self, event: Event) -> Result<(), DispatchError> {
        if self.0.len() == MAX_EVENTS {
            return Err(DispatchError::internal("pending event limit exceeded"));
        }
        self.0.push_back(event);
        Ok(())
    }
    fn drain(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.0.drain(..)
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        for target in self.pages.values() {
            target.page.close();
        }
    }
}

#[derive(Debug)]
struct DispatchError {
    code: i64,
    message: String,
}
impl DispatchError {
    fn method_not_found(message: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: message.into(),
        }
    }
    fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
        }
    }
    fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }
    fn internal(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
        }
    }
}
impl From<AutomationError> for DispatchError {
    fn from(error: AutomationError) -> Self {
        Self::internal(format!("{}: {error}", error.code()))
    }
}

fn internal_join(error: tokio::task::JoinError) -> DispatchError {
    DispatchError::internal(error.to_string())
}
fn string_param<'a>(params: &'a Value, name: &str) -> Result<&'a str, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be a string")))
}
fn bool_param(params: &Value, name: &str) -> Result<bool, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_bool)
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be a boolean")))
}
fn target_info(id: &str, url: &str, title: &str, attached: bool) -> Value {
    json!({"targetId": id, "type": "page", "title": title, "url": url, "attached": attached})
}
fn remote_object(value: Value) -> Value {
    match value {
        Value::Null => json!({"type": "object", "subtype": "null", "value": null}),
        Value::Bool(value) => json!({"type": "boolean", "value": value}),
        Value::Number(value) => json!({"type": "number", "value": value}),
        Value::String(value) => json!({"type": "string", "value": value}),
        value @ (Value::Array(_) | Value::Object(_)) => json!({"type": "object", "value": value}),
    }
}
fn timestamp() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_queue_applies_backpressure_at_its_fixed_limit() {
        let mut queue = EventQueue::default();
        for index in 0..MAX_EVENTS {
            queue
                .push(Event {
                    method: format!("event-{index}"),
                    params: json!({}),
                    session_id: None,
                })
                .unwrap();
        }
        let error = queue
            .push(Event {
                method: "overflow".into(),
                params: json!({}),
                session_id: None,
            })
            .unwrap_err();
        assert_eq!(error.code, -32603);
        assert_eq!(queue.drain().count(), MAX_EVENTS);
    }
}
