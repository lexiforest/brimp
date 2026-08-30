use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::{Value, json};
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken, PageOptions,
    RemoteArgument, TouchPoint,
};

use crate::interception::{InterceptionMode, InterceptionRegistry};
use crate::protocol::{Event, Request, Response};

const MAX_EVENTS: usize = 256;
const MAX_TARGETS: usize = 32;
const MAX_SESSIONS: usize = 64;

fn render_script(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;
    while let Some((offset, placeholder, value)) = replacements
        .iter()
        .filter_map(|(placeholder, value)| {
            remaining
                .find(placeholder)
                .map(|offset| (offset, *placeholder, *value))
        })
        .min_by_key(|(offset, _, _)| *offset)
    {
        rendered.push_str(&remaining[..offset]);
        rendered.push_str(value);
        remaining = &remaining[offset + placeholder.len()..];
    }
    rendered.push_str(remaining);
    rendered
}

pub(crate) struct ConnectionState {
    browser: Arc<AutomationBrowser>,
    page_options: PageOptions,
    pages: HashMap<String, PageTarget>,
    sessions: HashMap<String, String>,
    browser_sessions: HashSet<String>,
    browser_contexts: HashSet<String>,
    events: EventQueue,
    next_page_id: u64,
    next_session_id: u64,
    next_loader_id: u64,
    next_context_id: u64,
    next_script_id: u64,
    next_browser_context_id: u64,
    next_history_entry_id: u64,
    discover_targets: bool,
    enabled_pages: HashSet<String>,
    enabled_runtimes: HashSet<String>,
    enabled_networks: HashSet<String>,
    extra_http_headers: HashMap<String, Vec<(String, String)>>,
    lifecycle_events: HashSet<String>,
    waiting_for_debugger: HashSet<String>,
    auto_attach: HashMap<Option<String>, AutoAttach>,
    interception: InterceptionRegistry,
}

impl ConnectionState {
    pub(crate) fn new(
        browser: Arc<AutomationBrowser>,
        page_options: PageOptions,
        interception: InterceptionRegistry,
    ) -> Self {
        Self {
            browser,
            page_options,
            pages: HashMap::new(),
            sessions: HashMap::new(),
            browser_sessions: HashSet::new(),
            browser_contexts: HashSet::new(),
            events: EventQueue::default(),
            next_page_id: 1,
            next_session_id: 1,
            next_loader_id: 1,
            next_context_id: 1,
            next_script_id: 1,
            next_browser_context_id: 1,
            next_history_entry_id: 1,
            discover_targets: false,
            enabled_pages: HashSet::new(),
            enabled_runtimes: HashSet::new(),
            enabled_networks: HashSet::new(),
            extra_http_headers: HashMap::new(),
            lifecycle_events: HashSet::new(),
            waiting_for_debugger: HashSet::new(),
            auto_attach: HashMap::new(),
            interception,
        }
    }

    pub(crate) async fn dispatch(&mut self, request: &Request) -> Response {
        if let Some(response) = self.interception.handle_control(request) {
            return response;
        }
        let result = match request.method.as_str() {
            "Browser.getVersion" => Ok(json!({
                "protocolVersion": "1.3",
                "product": "Brimp/0.1.0",
                "revision": env!("CARGO_PKG_VERSION"),
                "userAgent": "Brimp/0.1.0",
                "jsVersion": "JavaScriptCore"
            })),
            "Browser.getWindowForTarget" => self.get_window_for_target(request),
            "Browser.getWindowBounds" => Ok(default_window_bounds()),
            "Browser.setDownloadBehavior" => Ok(json!({})),
            "Target.getBrowserContexts" => {
                Ok(json!({"browserContextIds": self.browser_contexts.iter().collect::<Vec<_>>()}))
            }
            "Target.createBrowserContext" => self.create_browser_context(),
            "Target.disposeBrowserContext" => self.dispose_browser_context(request),
            "Target.setDiscoverTargets" => self.set_discover_targets(request),
            "Target.setAutoAttach" => self.set_auto_attach(request),
            "Target.getTargets" => Ok(json!({"targetInfos": self.target_infos()})),
            "Target.getTargetInfo" => self.get_target_info(request),
            "Target.createTarget" => self.create_target(request).await,
            "Target.attachToBrowserTarget" => self.attach_to_browser_target(),
            "Target.attachToTarget" => self.attach_to_target(request),
            "Target.detachFromTarget" => self.detach_from_target(request),
            "Target.closeTarget" => self.close_target(request),
            "Target.activateTarget" => self.activate_target(request),
            "Page.enable" => self.enable_page(request),
            "Page.disable" => self.disable_page(request),
            "Page.getFrameTree" => self.get_frame_tree(request),
            "Page.setLifecycleEventsEnabled" => self.set_lifecycle_events(request),
            "Page.addScriptToEvaluateOnNewDocument" => self.add_preload_script(request).await,
            "Page.removeScriptToEvaluateOnNewDocument" => self.remove_preload_script(request).await,
            "Page.createIsolatedWorld" => self.create_isolated_world(request),
            "Page.navigate" => self.navigate(request).await,
            "Page.reload" => self.reload(request).await,
            "Page.getNavigationHistory" => self.get_navigation_history(request),
            "Page.navigateToHistoryEntry" => self.navigate_to_history_entry(request).await,
            "Page.getLayoutMetrics" => self.get_layout_metrics(request).await,
            "Page.captureScreenshot" => self.capture_screenshot(request).await,
            "DOM.describeNode" => self.describe_node(request).await,
            "DOM.resolveNode" => self.resolve_node(request).await,
            "DOM.getDocument" => self.get_document(request).await,
            "DOM.querySelector" => self.query_selector(request).await,
            "DOM.querySelectorAll" => self.query_selector_all(request).await,
            "DOM.getAttributes" => self.get_attributes(request).await,
            "DOM.getContentQuads" => self.get_content_quads(request).await,
            "DOM.getBoxModel" => self.get_box_model(request).await,
            "DOM.scrollIntoViewIfNeeded" => self.scroll_node_into_view(request).await,
            "DOM.focus" => self.focus_node(request).await,
            "Runtime.enable" => self.enable_runtime(request),
            "Runtime.disable" => self.disable_runtime(request),
            "Runtime.evaluate" => self.evaluate(request).await,
            "Runtime.callFunctionOn" => self.call_function(request).await,
            "Runtime.getProperties" => self.get_properties(request).await,
            "Runtime.releaseObject" => self.release_object(request).await,
            "Runtime.releaseObjectGroup" => self.release_object_group(request).await,
            "Runtime.runIfWaitingForDebugger" => self.run_if_waiting(request),
            "Network.enable" => self.enable_network(request),
            "Network.disable" => self.disable_network(request),
            "Network.setCacheDisabled" => self.acknowledge_session(request),
            "Network.setExtraHTTPHeaders" => self.set_extra_http_headers(request),
            "Network.setRequestInterception" => self.set_request_interception(request),
            "Network.getResponseBody" => self.get_response_body(request),
            "Fetch.enable" => self.enable_fetch(request),
            "Fetch.disable" => self.disable_fetch(request),
            "Network.setUserAgentOverride" | "Emulation.setUserAgentOverride" => {
                self.set_user_agent_override(request).await
            }
            "Emulation.setDeviceMetricsOverride" => self.set_device_metrics(request).await,
            "Emulation.clearDeviceMetricsOverride" => self.clear_device_metrics(request).await,
            "Emulation.setTouchEmulationEnabled" => self.set_touch_emulation(request),
            "Emulation.setFocusEmulationEnabled" => self.acknowledge_session(request),
            "Emulation.setEmulatedMedia" => self.set_emulated_media(request),
            "Input.dispatchMouseEvent" => self.dispatch_mouse_event(request).await,
            "Input.dispatchKeyEvent" => self.dispatch_key_event(request).await,
            "Input.dispatchTouchEvent" => self.dispatch_touch_event(request).await,
            "Input.insertText" => self.insert_text(request).await,
            "Log.enable"
            | "Log.disable"
            | "Performance.enable"
            | "Performance.disable"
            | "Audits.enable"
            | "Audits.disable"
            | "Security.enable"
            | "Security.disable"
            | "CSS.enable"
            | "CSS.disable" => self.acknowledge_session(request),
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
        self.auto_attach.insert(
            request.session_id.clone(),
            AutoAttach {
                auto_attach,
                wait_for_debugger,
            },
        );
        Ok(json!({}))
    }

    fn get_target_info(&self, request: &Request) -> Result<Value, DispatchError> {
        let Some(target_id) = request.params.get("targetId").and_then(Value::as_str) else {
            return Ok(json!({"targetInfo": browser_target_info()}));
        };
        if target_id == "browser" {
            return Ok(json!({"targetInfo": browser_target_info()}));
        }
        if !self.pages.contains_key(target_id) {
            return Err(DispatchError::invalid_params("unknown targetId"));
        }
        let target = &self.pages[target_id];
        Ok(
            json!({"targetInfo": target_info(target_id, &target.url, &target.title, self.sessions.values().any(|candidate| candidate == target_id), Some(&target.browser_context_id))}),
        )
    }

    fn create_browser_context(&mut self) -> Result<Value, DispatchError> {
        let context_id = format!("context-{}", self.next_browser_context_id);
        self.next_browser_context_id += 1;
        self.browser_contexts.insert(context_id.clone());
        Ok(json!({"browserContextId": context_id}))
    }

    fn dispose_browser_context(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let context_id = string_param(&request.params, "browserContextId")?.to_owned();
        if !self.browser_contexts.remove(&context_id) {
            return Err(DispatchError::invalid_params("unknown browserContextId"));
        }
        let targets: Vec<_> = self
            .pages
            .iter()
            .filter(|(_, target)| target.browser_context_id == context_id)
            .map(|(target_id, _)| target_id.clone())
            .collect();
        for target_id in targets {
            self.close_target_id(&target_id)?;
        }
        Ok(json!({}))
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
        let browser_context_id = request
            .params
            .get("browserContextId")
            .and_then(Value::as_str)
            .unwrap_or("default")
            .to_owned();
        if browser_context_id != "default" && !self.browser_contexts.contains(&browser_context_id) {
            return Err(DispatchError::invalid_params("unknown browserContextId"));
        }
        let target_id = format!("page-{}", self.next_page_id);
        self.next_page_id += 1;
        let interceptor = self.interception.target_interceptor(target_id.clone());
        let browser = Arc::clone(&self.browser);
        let options = self.page_options.clone();
        let viewport = options.viewport();
        let page = tokio::task::spawn_blocking(move || {
            browser.new_page_with_request_interceptor(options, interceptor)
        })
        .await
        .map_err(internal_join)??;
        let history_entry_id = self.next_history_entry_id;
        self.next_history_entry_id += 1;
        self.pages.insert(
            target_id.clone(),
            PageTarget {
                page,
                url: "about:blank".into(),
                title: String::new(),
                loader_id: format!("loader-{}", self.next_loader_id),
                context_id: self.next_context_id,
                viewport_width: viewport.width as u32,
                viewport_height: viewport.height as u32,
                device_pixel_ratio: viewport.device_pixel_ratio,
                browser_context_id,
                history: vec![NavigationHistoryEntry {
                    id: history_entry_id,
                    url: "about:blank".into(),
                    title: String::new(),
                }],
                history_index: 0,
                isolated_worlds: HashMap::new(),
                response_body: None,
                user_agent_headers: Vec::new(),
            },
        );
        self.next_loader_id += 1;
        self.next_context_id += 1;
        if self.discover_targets {
            self.push_event(Event {
                method: "Target.targetCreated".into(),
                params: json!({"targetInfo": target_info(&target_id, "about:blank", "", false, Some(&self.pages[&target_id].browser_context_id))}),
                session_id: None,
            })?;
        }
        if self
            .auto_attach
            .get(&None)
            .is_some_and(|config| config.auto_attach)
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

    fn attach_to_browser_target(&mut self) -> Result<Value, DispatchError> {
        if self.sessions.len() + self.browser_sessions.len() == MAX_SESSIONS {
            return Err(DispatchError::invalid_request("session limit exceeded"));
        }
        let session_id = format!("browser-session-{}", self.next_session_id);
        self.next_session_id += 1;
        self.browser_sessions.insert(session_id.clone());
        self.push_event(Event {
            method: "Target.attachedToTarget".into(),
            params: json!({
                "sessionId": session_id,
                "targetInfo": browser_target_info(),
                "waitingForDebugger": false
            }),
            session_id: None,
        })?;
        Ok(json!({"sessionId": session_id}))
    }

    fn attach(
        &mut self,
        target_id: &str,
        waiting_for_debugger: bool,
    ) -> Result<String, DispatchError> {
        if self.sessions.len() + self.browser_sessions.len() == MAX_SESSIONS {
            return Err(DispatchError::invalid_request("session limit exceeded"));
        }
        let target = self
            .pages
            .get(target_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown targetId"))?;
        let target_info = target_info(
            target_id,
            &target.url,
            &target.title,
            true,
            Some(&target.browser_context_id),
        );
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
        if self.browser_sessions.remove(&session_id) {
            self.auto_attach.remove(&Some(session_id.clone()));
            self.push_event(Event {
                method: "Target.detachedFromTarget".into(),
                params: json!({"sessionId": session_id, "targetId": "browser"}),
                session_id: request.session_id.clone(),
            })?;
            return Ok(json!({}));
        }
        let target_id = self
            .sessions
            .remove(&session_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown sessionId"))?;
        self.interception.remove_session(&target_id, &session_id);
        self.enabled_pages.remove(&session_id);
        self.enabled_runtimes.remove(&session_id);
        self.enabled_networks.remove(&session_id);
        self.extra_http_headers.remove(&session_id);
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
        let target_id = string_param(&request.params, "targetId")?;
        self.close_target_id(target_id)?;
        Ok(json!({"success": true}))
    }

    fn close_target_id(&mut self, target_id: &str) -> Result<(), DispatchError> {
        self.interception.remove_target(target_id);
        let target = self
            .pages
            .remove(target_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown targetId"))?;
        target.page.close();
        let detached: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, page)| page.as_str() == target_id)
            .map(|(id, _)| id.clone())
            .collect();
        for session_id in detached {
            self.sessions.remove(&session_id);
            self.enabled_pages.remove(&session_id);
            self.enabled_runtimes.remove(&session_id);
            self.enabled_networks.remove(&session_id);
            self.extra_http_headers.remove(&session_id);
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
        Ok(())
    }

    fn get_window_for_target(&self, request: &Request) -> Result<Value, DispatchError> {
        if let Some(target_id) = request.params.get("targetId").and_then(Value::as_str)
            && !self.pages.contains_key(target_id)
        {
            return Err(DispatchError::invalid_params("unknown targetId"));
        }
        Ok(json!({"windowId": 1, "bounds": default_window_bounds()["bounds"]}))
    }

    fn activate_target(&self, request: &Request) -> Result<Value, DispatchError> {
        let target_id = string_param(&request.params, "targetId")?;
        if !self.pages.contains_key(target_id) {
            return Err(DispatchError::invalid_params("unknown targetId"));
        }
        Ok(json!({}))
    }

    fn enable_page(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_pages.insert(session.to_owned());
        Ok(json!({}))
    }

    fn disable_page(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_pages.remove(&session);
        self.lifecycle_events.remove(&session);
        Ok(json!({}))
    }

    fn get_frame_tree(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let frame_id = self.target_for_session(session)?;
        let target = self.page_target_for_session(session)?;
        Ok(json!({"frameTree": {"frame": frame_value(frame_id, target)}}))
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

    async fn add_preload_script(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let source = string_param(&request.params, "source")?.to_owned();
        let identifier = format!("preload-{}", self.next_script_id);
        self.next_script_id += 1;
        let page = self.page_for_session(session)?.clone();
        let installed = identifier.clone();
        tokio::task::spawn_blocking(move || page.add_preload_script(installed, source))
            .await
            .map_err(internal_join)??;
        Ok(json!({"identifier": identifier}))
    }

    async fn remove_preload_script(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let identifier = string_param(&request.params, "identifier")?.to_owned();
        let page = self.page_for_session(session)?.clone();
        let removed = tokio::task::spawn_blocking(move || page.remove_preload_script(identifier))
            .await
            .map_err(internal_join)??;
        if !removed {
            return Err(DispatchError::invalid_params("unknown script identifier"));
        }
        Ok(json!({}))
    }

    fn create_isolated_world(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let frame_id = string_param(&request.params, "frameId")?;
        if self.target_for_session(&session)? != frame_id {
            return Err(DispatchError::invalid_params(
                "frameId is not the session frame",
            ));
        }
        let world_name = request
            .params
            .get("worldName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if world_name.is_empty() {
            return Ok(json!({"executionContextId": self.pages[frame_id].context_id}));
        }
        if let Some(context_id) = self.pages[frame_id].isolated_worlds.get(&world_name) {
            return Ok(json!({"executionContextId": context_id}));
        }
        let context_id = self.next_context_id;
        self.next_context_id += 1;
        self.pages
            .get_mut(frame_id)
            .expect("validated frame")
            .isolated_worlds
            .insert(world_name.clone(), context_id);
        if self.enabled_runtimes.contains(&session) {
            self.push_execution_context_named(&session, frame_id, context_id, &world_name, false)?;
        }
        Ok(json!({"executionContextId": context_id}))
    }

    async fn navigate(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let url = string_param(&request.params, "url")?.to_owned();
        self.navigate_session(session, url, HistoryAction::Push)
            .await
    }

    async fn reload(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let url = self.page_target_for_session(&session)?.url.clone();
        self.navigate_session(session, url, HistoryAction::Reload)
            .await
    }

    fn get_navigation_history(&self, request: &Request) -> Result<Value, DispatchError> {
        let target = self.page_target_for_session(self.session(request)?)?;
        Ok(json!({
            "currentIndex": target.history_index,
            "entries": target.history.iter().map(|entry| json!({
                "id": entry.id,
                "url": entry.url,
                "userTypedURL": entry.url,
                "title": entry.title,
                "transitionType": "typed"
            })).collect::<Vec<_>>()
        }))
    }

    async fn navigate_to_history_entry(
        &mut self,
        request: &Request,
    ) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let entry_id = u64_param(&request.params, "entryId")?;
        let target = self.page_target_for_session(&session)?;
        let index = target
            .history
            .iter()
            .position(|entry| entry.id == entry_id)
            .ok_or_else(|| DispatchError::invalid_params("unknown history entryId"))?;
        let url = target.history[index].url.clone();
        self.navigate_session(session, url, HistoryAction::Select(index))
            .await?;
        Ok(json!({}))
    }

    async fn navigate_session(
        &mut self,
        session: String,
        url: String,
        history_action: HistoryAction,
    ) -> Result<Value, DispatchError> {
        if !self.enabled_pages.contains(&session) {
            return Err(DispatchError::invalid_request(
                "Page.enable must be called first",
            ));
        }
        let navigated_url = url.clone();
        let target = self.page_target_for_session(&session)?;
        let page = target.page.clone();
        let viewport = (
            target.viewport_width,
            target.viewport_height,
            target.device_pixel_ratio,
        );
        let mut request_headers = target.user_agent_headers.clone();
        for (name, value) in self
            .extra_http_headers
            .get(&session)
            .cloned()
            .unwrap_or_default()
        {
            request_headers.retain(|(existing, _)| !existing.eq_ignore_ascii_case(&name));
            request_headers.push((name, value));
        }
        let event_request_headers = request_headers.clone();
        let (navigation, title) = tokio::task::spawn_blocking(move || {
            let navigation = page.navigate_with_headers(
                url,
                Duration::from_secs(30),
                CancellationToken::new(),
                request_headers,
            )?;
            page.set_viewport(viewport.0, viewport.1, viewport.2)?;
            Ok::<_, AutomationError>((navigation, page.title()?))
        })
        .await
        .map_err(internal_join)??;
        let frame_id = self.target_for_session(&session)?.to_owned();
        let loader_id = format!("loader-{}", self.next_loader_id);
        self.next_loader_id += 1;
        let context_id = self.next_context_id;
        self.next_context_id += 1;
        let world_names = self.pages[&frame_id]
            .isolated_worlds
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let isolated_contexts = world_names
            .into_iter()
            .map(|name| {
                let id = self.next_context_id;
                self.next_context_id += 1;
                (name, id)
            })
            .collect::<Vec<_>>();
        if let Some(target) = self.pages.get_mut(&frame_id) {
            target.url = navigated_url;
            target.title = title;
            target.loader_id.clone_from(&loader_id);
            target.context_id = context_id;
            target.isolated_worlds = isolated_contexts.iter().cloned().collect();
            target.response_body = Some(navigation.content.clone());
            match history_action {
                HistoryAction::Push => {
                    target.history.truncate(target.history_index + 1);
                    target.history.push(NavigationHistoryEntry {
                        id: self.next_history_entry_id,
                        url: target.url.clone(),
                        title: target.title.clone(),
                    });
                    self.next_history_entry_id += 1;
                    target.history_index = target.history.len() - 1;
                }
                HistoryAction::Reload => {
                    let entry = &mut target.history[target.history_index];
                    entry.url.clone_from(&target.url);
                    entry.title.clone_from(&target.title);
                }
                HistoryAction::Select(index) => {
                    target.history_index = index;
                    let entry = &mut target.history[index];
                    entry.url.clone_from(&target.url);
                    entry.title.clone_from(&target.title);
                }
            }
        }
        if self.enabled_networks.contains(&session) {
            let headers: serde_json::Map<String, Value> = navigation
                .headers
                .iter()
                .map(|(name, value)| (name.clone(), json!(value)))
                .collect();
            self.push_event(Event {
                method: "Network.requestWillBeSent".into(),
                params: json!({"requestId": loader_id, "loaderId": loader_id, "documentURL": navigation.url, "request": {"url": navigation.url, "method": "GET", "headers": event_request_headers.iter().cloned().map(|(name, value)| (name, json!(value))).collect::<serde_json::Map<_, _>>()}, "timestamp": timestamp(), "wallTime": timestamp(), "initiator": {"type": "other"}, "type": "Document", "frameId": frame_id}),
                session_id: Some(session.clone()),
            })?;
            self.push_event(Event {
                method: "Network.responseReceived".into(),
                params: json!({"requestId": loader_id, "loaderId": loader_id, "timestamp": timestamp(), "type": "Document", "response": {"url": navigation.url, "status": navigation.status_code, "statusText": navigation.reason, "headers": headers, "mimeType": "text/html", "connectionReused": false, "encodedDataLength": navigation.content.len()}, "frameId": frame_id}),
                session_id: Some(session.clone()),
            })?;
            self.push_event(Event {
                method: "Network.loadingFinished".into(),
                params: json!({"requestId": loader_id, "timestamp": timestamp(), "encodedDataLength": navigation.content.len()}),
                session_id: Some(session.clone()),
            })?;
        }
        self.push_event(Event {
            method: "Runtime.executionContextsCleared".into(),
            params: json!({}),
            session_id: Some(session.clone()),
        })?;
        self.push_event(Event {
            method: "Page.frameNavigated".into(),
            params: json!({"frame": frame_value(&frame_id, &self.pages[&frame_id]), "type": "Navigation"}),
            session_id: Some(session.clone()),
        })?;
        if self.enabled_runtimes.contains(&session) {
            self.push_execution_context(&session, &frame_id, context_id)?;
            for (name, isolated_context_id) in isolated_contexts {
                self.push_execution_context_named(
                    &session,
                    &frame_id,
                    isolated_context_id,
                    &name,
                    false,
                )?;
            }
        }
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
            method: "Page.domContentEventFired".into(),
            params: json!({"timestamp": timestamp()}),
            session_id: Some(session.clone()),
        })?;
        self.push_event(Event {
            method: "Page.loadEventFired".into(),
            params: json!({"timestamp": timestamp()}),
            session_id: Some(session),
        })?;
        Ok(json!({"frameId": frame_id, "loaderId": loader_id}))
    }

    async fn get_layout_metrics(&self, request: &Request) -> Result<Value, DispatchError> {
        let target = self.page_target_for_session(self.session(request)?)?;
        let width = target.viewport_width;
        let height = target.viewport_height;
        let rect = json!({"x": 0, "y": 0, "width": width, "height": height});
        Ok(json!({
            "layoutViewport": {"pageX": 0, "pageY": 0, "clientWidth": width, "clientHeight": height},
            "visualViewport": {"offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0, "clientWidth": width, "clientHeight": height, "scale": 1, "zoom": 1},
            "contentSize": rect.clone(),
            "cssLayoutViewport": {"pageX": 0, "pageY": 0, "clientWidth": width, "clientHeight": height},
            "cssVisualViewport": {"offsetX": 0, "offsetY": 0, "pageX": 0, "pageY": 0, "clientWidth": width, "clientHeight": height, "scale": 1, "zoom": 1},
            "cssContentSize": rect
        }))
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
        let capture_beyond_viewport =
            optional_bool_param(&request.params, "captureBeyondViewport")?.unwrap_or(false);
        if let Some(clip) = request.params.get("clip") {
            let target = self.page_target_for_session(session)?;
            let x = finite_number_param(clip, "x")?;
            let y = finite_number_param(clip, "y")?;
            let width = finite_number_param(clip, "width")?;
            let height = finite_number_param(clip, "height")?;
            let scale = finite_number_param(clip, "scale")?;
            let viewport_clip = x == 0.0
                && y == 0.0
                && width == f64::from(target.viewport_width)
                && height == f64::from(target.viewport_height)
                && scale == 1.0;
            let full_page_clip = capture_beyond_viewport
                && x == 0.0
                && y == 0.0
                && width == f64::from(target.viewport_width)
                && height >= f64::from(target.viewport_height)
                && scale == 1.0;
            if !viewport_clip && !full_page_clip {
                return Err(DispatchError::invalid_params(
                    "screenshot clip must cover the viewport or a full page from (0, 0) at scale 1",
                ));
            }
        }
        let page = self.page_for_session(session)?.clone();
        let bytes = tokio::task::spawn_blocking(move || page.screenshot(capture_beyond_viewport))
            .await
            .map_err(internal_join)??;
        Ok(json!({"data": base64::engine::general_purpose::STANDARD.encode(bytes)}))
    }

    async fn describe_node(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let object_id = string_param(&request.params, "objectId")?.to_owned();
        let page = self.page_for_session(session)?.clone();
        let node = tokio::task::spawn_blocking(move || page.describe_remote_node(object_id))
            .await
            .map_err(internal_join)??;
        Ok(json!({"node": node}))
    }

    async fn resolve_node(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let backend_node_id = u64_param(&request.params, "backendNodeId")?;
        let object_group = optional_string_param(&request.params, "objectGroup")?;
        let page = self.page_for_session(session)?.clone();
        let object = tokio::task::spawn_blocking(move || {
            page.resolve_remote_node(backend_node_id, object_group)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"object": object}))
    }

    async fn get_document(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let page = self.page_for_session(session)?.clone();
        let node = tokio::task::spawn_blocking(move || {
            let remote = page.evaluate_remote("document", false, None, false)?;
            let object_id = remote["objectId"]
                .as_str()
                .ok_or_else(|| AutomationError::Internal("document had no objectId".into()))?
                .to_owned();
            let node = page.describe_remote_node(object_id.clone())?;
            page.release_remote_object(object_id)?;
            Ok::<_, AutomationError>(node)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"root": node}))
    }

    async fn query_selector(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node_id = u64_param(&request.params, "nodeId")?;
        let selector = string_param(&request.params, "selector")?;
        let selector = serde_json::to_string(selector).map_err(internal_json)?;
        let node_id = node_id.to_string();
        let source = render_script(
            include_str!("scripts/query_selector.js"),
            &[("__NODE_ID__", &node_id), ("__SELECTOR__", &selector)],
        );
        let page = self.page_for_session(session)?.clone();
        let result = tokio::task::spawn_blocking(move || {
            let remote = page.evaluate_remote(source, false, None, false)?;
            let Some(object_id) = remote["objectId"].as_str() else {
                return Ok::<_, AutomationError>(0);
            };
            let object_id = object_id.to_owned();
            let node = page.describe_remote_node(object_id.clone())?;
            page.release_remote_object(object_id)?;
            node["nodeId"]
                .as_u64()
                .ok_or_else(|| AutomationError::Internal("DOM node had no nodeId".into()))
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"nodeId": result}))
    }

    async fn query_selector_all(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node_id = u64_param(&request.params, "nodeId")?;
        let selector = string_param(&request.params, "selector")?;
        let selector = serde_json::to_string(selector).map_err(internal_json)?;
        let node_id = node_id.to_string();
        let source = render_script(
            include_str!("scripts/query_selector_all.js"),
            &[("__NODE_ID__", &node_id), ("__SELECTOR__", &selector)],
        );
        let page = self.page_for_session(session)?.clone();
        let node_ids = tokio::task::spawn_blocking(move || {
            let remote = page.evaluate_remote(source, false, None, false)?;
            let array_id = remote["objectId"]
                .as_str()
                .ok_or_else(|| AutomationError::Internal("query result had no objectId".into()))?
                .to_owned();
            let properties = page.remote_object_properties(array_id.clone(), true, false)?;
            let mut node_ids = Vec::new();
            for property in properties["result"].as_array().into_iter().flatten() {
                if property["name"]
                    .as_str()
                    .is_none_or(|name| name.parse::<usize>().is_err())
                {
                    continue;
                }
                let Some(object_id) = property["value"]["objectId"].as_str() else {
                    continue;
                };
                let object_id = object_id.to_owned();
                let node = page.describe_remote_node(object_id.clone())?;
                page.release_remote_object(object_id)?;
                if let Some(node_id) = node["nodeId"].as_u64() {
                    node_ids.push(node_id);
                }
            }
            page.release_remote_object(array_id)?;
            Ok::<_, AutomationError>(node_ids)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"nodeIds": node_ids}))
    }

    async fn get_attributes(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node_id = u64_param(&request.params, "nodeId")?;
        let node_id = node_id.to_string();
        let source = render_script(
            include_str!("scripts/get_attributes.js"),
            &[("__NODE_ID__", &node_id)],
        );
        let page = self.page_for_session(session)?.clone();
        let attributes = tokio::task::spawn_blocking(move || page.evaluate(source))
            .await
            .map_err(internal_join)??;
        Ok(json!({"attributes": attributes}))
    }

    async fn get_content_quads(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node = remote_node_expression(&request.params)?;
        let source = render_script(
            include_str!("scripts/get_content_quads.js"),
            &[("__NODE__", &node)],
        );
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.evaluate(source))
            .await
            .map_err(internal_join)?
            .map_err(DispatchError::from)
    }

    async fn get_box_model(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node = remote_node_expression(&request.params)?;
        let source = render_script(
            include_str!("scripts/get_box_model.js"),
            &[("__NODE__", &node)],
        );
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.evaluate(source))
            .await
            .map_err(internal_join)?
            .map_err(DispatchError::from)
    }

    async fn scroll_node_into_view(&self, request: &Request) -> Result<Value, DispatchError> {
        self.invoke_node_method(request, "scrollIntoView").await
    }

    async fn focus_node(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node = remote_node_expression(&request.params)?;
        let node = format!(
            "(() => {{ const state = globalThis.__brimpCdpRemoteObjects; return {node}; }})()"
        );
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.focus_remote_node(node))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn invoke_node_method(
        &self,
        request: &Request,
        method: &str,
    ) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let node = remote_node_expression(&request.params)?;
        let method = serde_json::to_string(method).map_err(internal_json)?;
        let source = render_script(
            include_str!("scripts/invoke_node_method.js"),
            &[("__NODE__", &node), ("__METHOD__", &method)],
        );
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.evaluate(source))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    fn enable_runtime(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_runtimes.insert(session.clone());
        let target = self.target_for_session(&session)?.to_owned();
        let context_id = self.pages[&target].context_id;
        self.push_execution_context(&session, &target, context_id)?;
        Ok(json!({}))
    }

    fn disable_runtime(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_runtimes.remove(&session);
        Ok(json!({}))
    }

    fn enable_network(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_networks.insert(session);
        Ok(json!({}))
    }

    fn disable_network(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.enabled_networks.remove(&session);
        Ok(json!({}))
    }

    fn enable_fetch(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let patterns = interception_patterns(&request.params, false)?;
        let target = self.target_for_session(&session)?.to_owned();
        self.interception
            .enable(target, session, InterceptionMode::Fetch, patterns);
        Ok(json!({}))
    }

    fn disable_fetch(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let target = self.target_for_session(session)?.to_owned();
        self.interception.disable(&target, InterceptionMode::Fetch);
        Ok(json!({}))
    }

    fn set_request_interception(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let patterns = interception_patterns(&request.params, true)?;
        let target = self.target_for_session(&session)?.to_owned();
        if patterns.is_empty() {
            self.interception
                .disable(&target, InterceptionMode::Network);
        } else {
            self.interception
                .enable(target, session, InterceptionMode::Network, patterns);
        }
        Ok(json!({}))
    }

    fn set_extra_http_headers(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let headers = request
            .params
            .get("headers")
            .and_then(Value::as_object)
            .ok_or_else(|| DispatchError::invalid_params("headers must be an object"))?;
        let headers = headers
            .iter()
            .map(|(name, value)| {
                value
                    .as_str()
                    .map(|value| (name.clone(), value.to_owned()))
                    .ok_or_else(|| DispatchError::invalid_params("header values must be strings"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.extra_http_headers.insert(session, headers);
        Ok(json!({}))
    }

    fn get_response_body(&self, request: &Request) -> Result<Value, DispatchError> {
        let target = self.page_target_for_session(self.session(request)?)?;
        let request_id = string_param(&request.params, "requestId")?;
        if request_id != target.loader_id {
            return Err(DispatchError::invalid_params("unknown requestId"));
        }
        let body = target
            .response_body
            .as_deref()
            .ok_or_else(|| DispatchError::invalid_request("response body is unavailable"))?;
        match std::str::from_utf8(body) {
            Ok(body) => Ok(json!({"body": body, "base64Encoded": false})),
            Err(_) => Ok(json!({
                "body": base64::engine::general_purpose::STANDARD.encode(body),
                "base64Encoded": true
            })),
        }
    }

    async fn set_user_agent_override(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let user_agent = string_param(&request.params, "userAgent")?.to_owned();
        if !valid_header_value(&user_agent) {
            return Err(DispatchError::invalid_params(
                "userAgent contains invalid HTTP header characters",
            ));
        }
        let accept_language = optional_string_param(&request.params, "acceptLanguage")?;
        if accept_language
            .as_deref()
            .is_some_and(|value| !valid_header_value(value))
        {
            return Err(DispatchError::invalid_params(
                "acceptLanguage contains invalid HTTP header characters",
            ));
        }
        let platform = optional_string_param(&request.params, "platform")?;
        if request
            .params
            .get("userAgentMetadata")
            .is_some_and(|value| !value.is_null())
        {
            return Err(DispatchError::invalid_params(
                "userAgentMetadata override is not supported",
            ));
        }
        let language = accept_language
            .as_deref()
            .and_then(|value| value.split(',').next())
            .and_then(|value| value.split(';').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        let languages = accept_language.as_deref().map(|value| {
            value
                .split(',')
                .filter_map(|item| item.split(';').next().map(str::trim))
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
        let identity_override = json!({
            "userAgent": &user_agent,
            "platform": &platform,
            "language": languages.as_ref().map(|_| language),
            "languages": &languages,
        });
        let target_id = self.target_for_session(&session)?.to_owned();
        let target = self
            .pages
            .get_mut(&target_id)
            .ok_or_else(|| DispatchError::invalid_request("session target is closed"))?;
        target.user_agent_headers.clear();
        target
            .user_agent_headers
            .push(("User-Agent".into(), user_agent));
        if let Some(accept_language) = accept_language {
            target
                .user_agent_headers
                .push(("Accept-Language".into(), accept_language));
        }
        let page = target.page.clone();
        tokio::task::spawn_blocking(move || {
            page.set_navigator_identity_override(identity_override)?;
            Ok::<_, AutomationError>(())
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }

    fn acknowledge_session(&self, request: &Request) -> Result<Value, DispatchError> {
        self.session(request)?;
        Ok(json!({}))
    }

    async fn set_device_metrics(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let width = u32_param(&request.params, "width")?;
        let height = u32_param(&request.params, "height")?;
        let device_pixel_ratio = request
            .params
            .get("deviceScaleFactor")
            .and_then(Value::as_f64)
            .ok_or_else(|| DispatchError::invalid_params("deviceScaleFactor must be a number"))?;
        if !device_pixel_ratio.is_finite() || device_pixel_ratio < 0.0 {
            return Err(DispatchError::invalid_params(
                "deviceScaleFactor must be finite and non-negative",
            ));
        }
        let target_id = self.target_for_session(&session)?.to_owned();
        let target = self
            .pages
            .get_mut(&target_id)
            .ok_or_else(|| DispatchError::invalid_request("session target is closed"))?;
        target.viewport_width = width;
        target.viewport_height = height;
        target.device_pixel_ratio = device_pixel_ratio;
        let page = target.page.clone();
        tokio::task::spawn_blocking(move || page.set_viewport(width, height, device_pixel_ratio))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn clear_device_metrics(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        let viewport = PageOptions::default().viewport();
        let target_id = self.target_for_session(&session)?.to_owned();
        let target = self
            .pages
            .get_mut(&target_id)
            .ok_or_else(|| DispatchError::invalid_request("session target is closed"))?;
        target.viewport_width = viewport.width as u32;
        target.viewport_height = viewport.height as u32;
        target.device_pixel_ratio = viewport.device_pixel_ratio;
        let page = target.page.clone();
        tokio::task::spawn_blocking(move || {
            page.set_viewport(
                viewport.width as u32,
                viewport.height as u32,
                viewport.device_pixel_ratio,
            )
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }

    fn set_touch_emulation(&self, request: &Request) -> Result<Value, DispatchError> {
        self.session(request)?;
        bool_param(&request.params, "enabled")?;
        if let Some(max_touch_points) = request.params.get("maxTouchPoints") {
            let max_touch_points = max_touch_points.as_u64().ok_or_else(|| {
                DispatchError::invalid_params("maxTouchPoints must be a non-negative integer")
            })?;
            if max_touch_points > u32::MAX as u64 {
                return Err(DispatchError::invalid_params(
                    "maxTouchPoints exceeds the supported range",
                ));
            }
        }
        Ok(json!({}))
    }

    fn set_emulated_media(&self, request: &Request) -> Result<Value, DispatchError> {
        self.session(request)?;
        if request
            .params
            .get("media")
            .and_then(Value::as_str)
            .is_some_and(|media| !media.is_empty())
        {
            return Err(DispatchError::invalid_params(
                "non-default media emulation is not supported",
            ));
        }
        let supported_defaults = [
            ("prefers-color-scheme", "light"),
            ("prefers-reduced-motion", "no-preference"),
            ("forced-colors", "none"),
            ("prefers-contrast", "no-preference"),
        ];
        if let Some(features) = request.params.get("features").and_then(Value::as_array) {
            for feature in features {
                let name = feature.get("name").and_then(Value::as_str).unwrap_or("");
                let value = feature.get("value").and_then(Value::as_str).unwrap_or("");
                if !supported_defaults.contains(&(name, value)) {
                    return Err(DispatchError::invalid_params(format!(
                        "unsupported media feature {name}={value}"
                    )));
                }
            }
        }
        Ok(json!({}))
    }

    async fn dispatch_mouse_event(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let event_type = string_param(&request.params, "type")?;
        if !matches!(event_type, "mouseMoved" | "mousePressed" | "mouseReleased") {
            return Err(DispatchError::invalid_params(
                "supported mouse event types are mouseMoved, mousePressed, and mouseReleased",
            ));
        }
        let x = finite_number_param(&request.params, "x")?;
        let y = finite_number_param(&request.params, "y")?;
        let button = match request
            .params
            .get("button")
            .and_then(Value::as_str)
            .unwrap_or("none")
        {
            "none" | "left" => 0,
            "middle" => 1,
            "right" => 2,
            "back" => 3,
            "forward" => 4,
            _ => return Err(DispatchError::invalid_params("unsupported mouse button")),
        };
        let buttons = request
            .params
            .get("buttons")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let click_count = request
            .params
            .get("clickCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let modifiers = request
            .params
            .get("modifiers")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let page = self.page_for_session(session)?.clone();
        let event_type = event_type.to_owned();
        tokio::task::spawn_blocking(move || {
            page.dispatch_mouse_event(event_type, x, y, button, buttons, click_count, modifiers)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn dispatch_key_event(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let event_type = string_param(&request.params, "type")?;
        if !matches!(event_type, "keyDown" | "rawKeyDown" | "keyUp" | "char") {
            return Err(DispatchError::invalid_params("unsupported key event type"));
        }
        let key = request
            .params
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("");
        let code = request
            .params
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("");
        let text = request
            .params
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("");
        let modifiers = request
            .params
            .get("modifiers")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let repeat = request
            .params
            .get("autoRepeat")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let event_type = event_type.to_owned();
        let key = key.to_owned();
        let code = code.to_owned();
        let text = text.to_owned();
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || {
            page.dispatch_key_event(event_type, key, code, text, repeat, modifiers)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn dispatch_touch_event(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let event_type = string_param(&request.params, "type")?;
        if !matches!(
            event_type,
            "touchStart" | "touchMove" | "touchEnd" | "touchCancel"
        ) {
            return Err(DispatchError::invalid_params(
                "unsupported touch event type",
            ));
        }
        let points = request
            .params
            .get("touchPoints")
            .and_then(Value::as_array)
            .ok_or_else(|| DispatchError::invalid_params("touchPoints must be an array"))?;
        if matches!(event_type, "touchEnd" | "touchCancel") && !points.is_empty() {
            return Err(DispatchError::invalid_params(
                "touchPoints must be empty for touchEnd and touchCancel",
            ));
        }
        if matches!(event_type, "touchStart" | "touchMove") && points.is_empty() {
            return Err(DispatchError::invalid_params(
                "touchPoints must not be empty for touchStart and touchMove",
            ));
        }
        let mut touch_points = Vec::with_capacity(points.len());
        let mut touch_ids = HashSet::new();
        for point in points {
            let x = finite_number_param(point, "x")?;
            let y = finite_number_param(point, "y")?;
            let finite_optional = |name: &str, default: f64| {
                point.get(name).map_or(Ok(default), |value| {
                    value
                        .as_f64()
                        .filter(|number| number.is_finite())
                        .ok_or_else(|| {
                            DispatchError::invalid_params(format!(
                                "touch point {name} must be finite"
                            ))
                        })
                })
            };
            let id = point.get("id").and_then(Value::as_u64).unwrap_or(0);
            let id = u32::try_from(id)
                .map_err(|_| DispatchError::invalid_params("touch point id is too large"))?;
            if !touch_ids.insert(id) {
                return Err(DispatchError::invalid_params(
                    "touch point ids must be unique",
                ));
            }
            let touch_point = TouchPoint {
                id,
                x,
                y,
                radius_x: finite_optional("radiusX", 1.0)?,
                radius_y: finite_optional("radiusY", 1.0)?,
                rotation_angle: finite_optional("rotationAngle", 0.0)?,
                force: finite_optional("force", 1.0)?,
                tangential_pressure: finite_optional("tangentialPressure", 0.0)?,
            };
            if touch_point.radius_x < 0.0
                || touch_point.radius_y < 0.0
                || !(0.0..=1.0).contains(&touch_point.force)
                || !(-1.0..=1.0).contains(&touch_point.tangential_pressure)
            {
                return Err(DispatchError::invalid_params(
                    "touch point radius, force, or tangentialPressure is outside its supported range",
                ));
            }
            touch_points.push(touch_point);
        }
        let modifiers = request
            .params
            .get("modifiers")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let event_type = event_type.to_owned();
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || {
            page.dispatch_touch_event(event_type, touch_points, modifiers)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn insert_text(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let text = string_param(&request.params, "text")?.to_owned();
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.insert_text(text))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    fn push_execution_context(
        &mut self,
        session: &str,
        target: &str,
        context_id: u64,
    ) -> Result<(), DispatchError> {
        self.push_execution_context_named(session, target, context_id, "", true)
    }

    fn push_execution_context_named(
        &mut self,
        session: &str,
        target: &str,
        context_id: u64,
        name: &str,
        is_default: bool,
    ) -> Result<(), DispatchError> {
        let origin = self.pages[target].url.clone();
        self.push_event(Event {
            method: "Runtime.executionContextCreated".into(),
            params: json!({"context": {
                "id": context_id,
                "origin": origin,
                "name": name,
                "uniqueId": format!("context-{target}-{context_id}"),
                "auxData": {"isDefault": is_default, "type": if is_default { "default" } else { "isolated" }, "frameId": target}
            }}),
            session_id: Some(session.to_owned()),
        })
    }

    async fn evaluate(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        if !self.enabled_runtimes.contains(session) {
            return Err(DispatchError::invalid_request(
                "Runtime.enable must be called first",
            ));
        }
        let expression = string_param(&request.params, "expression")?.to_owned();
        let return_by_value = request
            .params
            .get("returnByValue")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let object_group = optional_string_param(&request.params, "objectGroup")?;
        let await_promise = optional_bool_param(&request.params, "awaitPromise")?.unwrap_or(false);
        let page = self.page_for_session(session)?.clone();
        let result = tokio::task::spawn_blocking(move || {
            page.evaluate_remote(expression, return_by_value, object_group, await_promise)
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"result": result}))
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
        let arguments = arguments
            .into_iter()
            .map(|argument| {
                if let Some(object_id) = argument.get("objectId").and_then(Value::as_str) {
                    Ok(RemoteArgument::ObjectId(object_id.to_owned()))
                } else if let Some(value) =
                    argument.get("unserializableValue").and_then(Value::as_str)
                {
                    Ok(RemoteArgument::UnserializableValue(value.to_owned()))
                } else if argument.get("value").is_some() {
                    Ok(RemoteArgument::Value(argument["value"].clone()))
                } else {
                    Err(DispatchError::invalid_params(
                        "function argument requires value, unserializableValue, or objectId",
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let receiver = request
            .params
            .get("objectId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let return_by_value = request
            .params
            .get("returnByValue")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let object_group = optional_string_param(&request.params, "objectGroup")?;
        let await_promise = optional_bool_param(&request.params, "awaitPromise")?.unwrap_or(false);
        let declaration = declaration.to_owned();
        let page = self.page_for_session(session)?.clone();
        let result = tokio::task::spawn_blocking(move || {
            page.call_function_remote(
                declaration,
                receiver,
                arguments,
                return_by_value,
                object_group,
                await_promise,
            )
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({"result": result}))
    }

    async fn get_properties(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        if !self.enabled_runtimes.contains(session) {
            return Err(DispatchError::invalid_request(
                "Runtime.enable must be called first",
            ));
        }
        let object_id = string_param(&request.params, "objectId")?.to_owned();
        let own_properties = request
            .params
            .get("ownProperties")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let accessor_properties_only = request
            .params
            .get("accessorPropertiesOnly")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || {
            page.remote_object_properties(object_id, own_properties, accessor_properties_only)
        })
        .await
        .map_err(internal_join)?
        .map_err(DispatchError::from)
    }

    async fn release_object(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let object_id = string_param(&request.params, "objectId")?.to_owned();
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.release_remote_object(object_id))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    async fn release_object_group(&self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?;
        let object_group = string_param(&request.params, "objectGroup")?.to_owned();
        let page = self.page_for_session(session)?.clone();
        tokio::task::spawn_blocking(move || page.release_remote_object_group(object_group))
            .await
            .map_err(internal_join)??;
        Ok(json!({}))
    }

    fn run_if_waiting(&mut self, request: &Request) -> Result<Value, DispatchError> {
        let session = self.session(request)?.to_owned();
        self.waiting_for_debugger.remove(&session);
        Ok(json!({}))
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
        Ok(&self.page_target_for_session(session)?.page)
    }

    fn page_target_for_session(&self, session: &str) -> Result<&PageTarget, DispatchError> {
        let target = self.target_for_session(session)?;
        self.pages
            .get(target)
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
                    Some(&target.browser_context_id),
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
}

struct PageTarget {
    page: AutomationPage,
    url: String,
    title: String,
    loader_id: String,
    context_id: u64,
    viewport_width: u32,
    viewport_height: u32,
    device_pixel_ratio: f64,
    browser_context_id: String,
    history: Vec<NavigationHistoryEntry>,
    history_index: usize,
    isolated_worlds: HashMap<String, u64>,
    response_body: Option<Vec<u8>>,
    user_agent_headers: Vec<(String, String)>,
}

struct NavigationHistoryEntry {
    id: u64,
    url: String,
    title: String,
}

#[derive(Clone, Copy)]
enum HistoryAction {
    Push,
    Reload,
    Select(usize),
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
fn internal_json(error: serde_json::Error) -> DispatchError {
    DispatchError::internal(error.to_string())
}
fn string_param<'a>(params: &'a Value, name: &str) -> Result<&'a str, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be a string")))
}
fn optional_string_param(params: &Value, name: &str) -> Result<Option<String>, DispatchError> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(DispatchError::invalid_params(format!(
            "{name} must be a string"
        ))),
    }
}
fn optional_bool_param(params: &Value, name: &str) -> Result<Option<bool>, DispatchError> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(DispatchError::invalid_params(format!(
            "{name} must be a boolean"
        ))),
    }
}
fn finite_number_param(params: &Value, name: &str) -> Result<f64, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be a finite number")))
}

fn interception_patterns(params: &Value, legacy: bool) -> Result<Vec<String>, DispatchError> {
    let Some(patterns) = params.get("patterns") else {
        return Ok(if legacy { Vec::new() } else { vec!["*".into()] });
    };
    let patterns = patterns
        .as_array()
        .ok_or_else(|| DispatchError::invalid_params("patterns must be an array"))?;
    patterns
        .iter()
        .map(|pattern| {
            if pattern.get("resourceType").is_some() {
                return Err(DispatchError::invalid_params(
                    "resourceType interception filters are not supported",
                ));
            }
            let stage_name = if legacy {
                "interceptionStage"
            } else {
                "requestStage"
            };
            if pattern
                .get(stage_name)
                .and_then(Value::as_str)
                .is_some_and(|stage| stage != "Request")
            {
                return Err(DispatchError::invalid_params(
                    "only request-stage interception is supported",
                ));
            }
            Ok(pattern
                .get("urlPattern")
                .and_then(Value::as_str)
                .filter(|pattern| !pattern.is_empty())
                .unwrap_or("*")
                .to_owned())
        })
        .collect()
}
fn remote_node_expression(params: &Value) -> Result<String, DispatchError> {
    if let Some(object_id) = params.get("objectId").and_then(Value::as_str) {
        let object_id = serde_json::to_string(object_id).map_err(internal_json)?;
        return Ok(format!("state?.objects.get({object_id})"));
    }
    if let Some(backend_node_id) = params.get("backendNodeId").and_then(Value::as_u64) {
        return Ok(format!("state?.backendNodes?.get({backend_node_id})"));
    }
    if let Some(node_id) = params.get("nodeId").and_then(Value::as_u64) {
        return Ok(format!("state?.backendNodes?.get({node_id})"));
    }
    Err(DispatchError::invalid_params(
        "objectId or backendNodeId is required",
    ))
}
fn valid_header_value(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte == b'\t' || (0x20..=0x7e).contains(&byte))
}
fn bool_param(params: &Value, name: &str) -> Result<bool, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_bool)
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be a boolean")))
}
fn u32_param(params: &Value, name: &str) -> Result<u32, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| {
            DispatchError::invalid_params(format!("{name} must be a non-negative integer"))
        })
}
fn u64_param(params: &Value, name: &str) -> Result<u64, DispatchError> {
    params
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| DispatchError::invalid_params(format!("{name} must be an integer")))
}
fn target_info(
    id: &str,
    url: &str,
    title: &str,
    attached: bool,
    browser_context_id: Option<&str>,
) -> Value {
    let mut value = json!({
        "targetId": id,
        "type": "page",
        "title": title,
        "url": url,
        "attached": attached,
        "canAccessOpener": false
    });
    if let Some(browser_context_id) = browser_context_id {
        value["browserContextId"] = json!(browser_context_id);
    }
    value
}
fn browser_target_info() -> Value {
    json!({
        "targetId": "browser",
        "type": "browser",
        "title": "",
        "url": "",
        "attached": true,
        "canAccessOpener": false
    })
}
fn frame_value(id: &str, target: &PageTarget) -> Value {
    json!({
        "id": id,
        "loaderId": target.loader_id,
        "url": target.url,
        "domainAndRegistry": "",
        "securityOrigin": target.url,
        "mimeType": "text/html",
        "adFrameStatus": {"adFrameType": "none"},
        "secureContextType": "Secure",
        "crossOriginIsolatedContextType": "NotIsolated",
        "gatedAPIFeatures": []
    })
}
fn default_window_bounds() -> Value {
    json!({"bounds": {
        "left": 0,
        "top": 0,
        "width": 1280,
        "height": 720,
        "windowState": "normal"
    }})
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
