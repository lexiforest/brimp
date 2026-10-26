mod scripts;

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use crate::dom::BrowserDocument;
use crate::network::{HeaderList, NetworkError, ResourceLoader, ResourceRequest};
use crate::web_apis::{
    BindingQueues, BindingRuntime, BrowsingContext, CookieJar, FetchQueue, PendingFetch,
    PendingWebSocketOperation, StreamingQueue, TimerQueue, WrapperCache,
};
use brimp_jsc::{JsException, JsRuntime, JsValue, ProtectedJsObject};

use crate::runtime::{
    ExtractedDocument, ExtractionError, ExtractionOptions, TaskQueue, TaskSender,
};

type ConsoleCallback = Rc<dyn Fn(&str)>;

pub struct Page {
    session_id: u64,
    pub(super) operation_deadline: Option<Instant>,
    pub(super) cancellation: super::CancellationToken,
    wait_until: NavigationWait,
    pending_navigation_scripts: std::cell::Cell<usize>,
    navigation_jobs: RefCell<Vec<tokio::task::AbortHandle>>,
    clock_options: super::clock::ClockOptions,
    diagnostics: Option<super::diagnostics::Diagnostics>,
    realm_id: u64,
    owner_id: u64,
    events: super::events::EventHub,
    network: Option<Arc<super::networking::NetworkRuntime>>,
    plugins: Vec<super::plugins::Plugin>,
    plugin_runtime: Option<super::plugins::PluginRuntime>,
    pub(crate) bridge: super::bridge::BridgeRuntime,
    plugin_timeout: Duration,
    pub(crate) shutdown: Arc<AtomicBool>,
    // Bindings must drop before `js`, because their objects are protected in that context.
    bindings: BindingRuntime,
    persona_identity_setter: ProtectedJsObject,
    module_loader: ProtectedJsObject,
    defuddle_extractor: Option<ProtectedJsObject>,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    fetches: Rc<RefCell<FetchQueue>>,
    js: JsRuntime,
    document: Rc<RefCell<BrowserDocument>>,
    tasks: TaskQueue,
    network_scope: DocumentNetworkScope,
    load_state: LoadState,
    document_ready: bool,
    async_error: Option<JsException>,
    persona: crate::persona::PersonaConfig,
    navigator_identity_override: serde_json::Map<String, serde_json::Value>,
    network_offline: bool,
    console_callback: Option<ConsoleCallback>,
    preload_scripts: Vec<(String, String)>,
    streaming_queue: Rc<RefCell<StreamingQueue>>,
    websockets: HashMap<u64, crate::network::WebSocketHandle>,
    fetch_streams: RefCell<HashMap<u64, crate::network::ResourceStreamHandle>>,
    fetch_buffered: RefCell<HashMap<u64, Arc<AtomicUsize>>>,
    script_execution_deadline: Option<Instant>,
}

#[derive(Clone)]
pub(crate) struct DocumentNetworkScope {
    loader: Arc<dyn ResourceLoader>,
    executor: tokio::runtime::Handle,
    pub(super) sockets: Arc<dyn crate::network::WebSocketConnector>,
}

impl DocumentNetworkScope {
    pub(crate) fn new(
        loader: Arc<dyn ResourceLoader>,
        sockets: Arc<dyn crate::network::WebSocketConnector>,
        executor: tokio::runtime::Handle,
    ) -> Self {
        Self {
            loader,
            sockets,
            executor,
        }
    }
}

impl Page {
    pub(crate) fn new(
        options: PageOptions,
        network_scope: DocumentNetworkScope,
        cookies: Arc<CookieJar>,
    ) -> Result<Self, JsException> {
        let realm_id = next_realm_id();
        let diagnostics = options
            .diagnostics
            .clone()
            .map(|options| super::diagnostics::Diagnostics::new(options, realm_id))
            .transpose()?;
        let js = JsRuntime::new()?;
        install_interrupt(
            &js,
            options.operation_deadline,
            options.cancellation.clone(),
            options.shutdown.clone(),
        );
        let browsing_context = Arc::new(BrowsingContext::with_cookie_jar(cookies));
        let mut request_headers = persona_request_headers(&options.persona);
        for (name, value) in &options.request_headers {
            request_headers.retain(|(old, _)| !old.eq_ignore_ascii_case(name));
            request_headers.push((name.clone(), value.clone()));
        }
        browsing_context
            .set_request_headers(request_headers)
            .map_err(JsException::from_message)?;
        let initial_document = BrowserDocument::parse("<html><head></head><body></body></html>");
        let document = Rc::new(RefCell::new(initial_document));
        let timers = Rc::new(RefCell::new(TimerQueue::new(&options.clock)));
        let fetches = Rc::new(RefCell::new(FetchQueue::default()));
        let streaming_queue = Rc::new(RefCell::new(StreamingQueue::default()));
        let bindings = BindingRuntime::install(
            &js,
            Rc::clone(&document),
            Arc::clone(&browsing_context),
            false,
            options.events.clone(),
            BindingQueues {
                timers: Rc::clone(&timers),
                fetches: Rc::clone(&fetches),
                streaming: Rc::clone(&streaming_queue),
            },
            diagnostics.clone(),
        )?;
        let module_loader = js
            .eval(include_str!("../../js/runtime/module_loader.js"))?
            .to_object()?;
        let persona_identity_setter = install_persona(&js, &options.persona)?;
        let bridge = super::bridge::BridgeRuntime::new(
            &js,
            realm_id,
            options.shutdown.clone(),
            options.plugin_timeout,
        )?;
        let mut page = Self {
            bridge,
            clock_options: options.clock,
            diagnostics,
            session_id: options.session_id,
            operation_deadline: options.operation_deadline,
            cancellation: options.cancellation,
            wait_until: options.wait_until,
            pending_navigation_scripts: std::cell::Cell::new(0),
            navigation_jobs: RefCell::new(Vec::new()),
            realm_id,
            owner_id: realm_id,
            events: options.events,
            network: options.network,
            plugins: options.plugins,
            plugin_runtime: None,
            plugin_timeout: options.plugin_timeout,
            shutdown: options.shutdown,
            bindings,
            persona_identity_setter,
            module_loader,
            defuddle_extractor: None,
            timers,
            browsing_context,
            fetches,
            js,
            document,
            tasks: TaskQueue::default(),
            network_scope,
            load_state: LoadState::Idle,
            document_ready: false,
            async_error: None,
            persona: options.persona,
            navigator_identity_override: serde_json::Map::new(),
            network_offline: false,
            console_callback: None,
            preload_scripts: Vec::new(),
            streaming_queue,
            websockets: HashMap::new(),
            fetch_streams: RefCell::new(HashMap::new()),
            fetch_buffered: RefCell::new(HashMap::new()),
            script_execution_deadline: None,
        };
        page.install_console_events()?;
        page.initialize_plugins()?;
        page.bindings.finalize_exposure(&page.js)?;
        page.install_diagnostics()?;
        Ok(page)
    }

    pub(crate) fn bridge_retain(
        &self,
        value: &ProtectedJsObject,
    ) -> Result<super::bridge::Reference, super::AutomationError> {
        self.bridge.retain(&self.js, value)
    }
    pub(crate) fn bridge_get(
        &self,
        token: &super::bridge::Reference,
    ) -> Result<ProtectedJsObject, super::AutomationError> {
        self.bridge.get(&self.js, token)
    }
    pub fn set_content(&mut self, html: &str) -> Result<(), JsException> {
        self.set_content_at(html, None)
    }

    fn set_content_at(&mut self, html: &str, _base_url: Option<&str>) -> Result<(), JsException> {
        let document = BrowserDocument::parse(html);
        *self.document.borrow_mut() = document;
        self.document_ready = true;
        self.bindings.reset_document(&self.js)
    }

    fn reset_page_at(
        &mut self,
        base_url: &str,
        cross_origin_isolated: bool,
        encoding: &'static encoding_rs::Encoding,
        content_type: &str,
    ) -> Result<(), JsException> {
        self.close_plugins();
        self.browsing_context.set_url(base_url);
        self.realm_id = next_realm_id();
        if let Some(log) = &self.diagnostics {
            log.realm(self.realm_id);
        }
        self.close_websockets();
        self.cancel_fetch_streams();
        let document = BrowserDocument::empty();
        document.set_encoding(document.root(), encoding);
        document.set_content_type(document.root(), content_type);
        *self.document.borrow_mut() = document;
        self.document_ready = false;
        for job in self.navigation_jobs.borrow_mut().drain(..) {
            job.abort();
        }
        self.pending_navigation_scripts.set(0);
        let js = JsRuntime::new()?;
        install_interrupt(
            &js,
            self.operation_deadline,
            self.cancellation.clone(),
            self.shutdown.clone(),
        );
        if let Some(deadline) = self.script_execution_deadline {
            js.set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        }
        let timers = Rc::new(RefCell::new(TimerQueue::new(&self.clock_options)));
        let fetches = Rc::new(RefCell::new(FetchQueue::default()));
        let streaming_queue = Rc::new(RefCell::new(StreamingQueue::default()));
        let bindings = BindingRuntime::install(
            &js,
            Rc::clone(&self.document),
            Arc::clone(&self.browsing_context),
            cross_origin_isolated,
            self.events.clone(),
            BindingQueues {
                timers: Rc::clone(&timers),
                fetches: Rc::clone(&fetches),
                streaming: Rc::clone(&streaming_queue),
            },
            self.diagnostics.clone(),
        )?;
        let module_loader = js
            .eval(include_str!("../../js/runtime/module_loader.js"))?
            .to_object()?;
        let persona_identity_setter = install_persona(&js, &self.persona)?;
        if !self.navigator_identity_override.is_empty() {
            js.call_function_with_string(
                &persona_identity_setter,
                &serde_json::Value::Object(self.navigator_identity_override.clone()).to_string(),
            )?;
        }
        js.call_function_with_string(
            &persona_identity_setter,
            &serde_json::json!({"onLine": !self.network_offline}).to_string(),
        )?;
        if let Some(callback) = self.console_callback.clone() {
            js.set_console_callback(move |message| callback(message))?;
        }

        self.bridge = super::bridge::BridgeRuntime::new(
            &js,
            self.realm_id,
            self.shutdown.clone(),
            self.plugin_timeout,
        )?;
        self.bindings = bindings;
        self.persona_identity_setter = persona_identity_setter;
        self.module_loader = module_loader;
        self.defuddle_extractor = None;
        self.timers = timers;
        self.fetches = fetches;
        self.streaming_queue = streaming_queue;
        self.tasks = TaskQueue::default();
        self.async_error = None;
        self.js = js;

        self.install_console_events()?;
        self.initialize_plugins()?;
        self.bindings.finalize_exposure(&self.js)?;
        self.install_diagnostics()?;
        Ok(())
    }

    pub(crate) fn id(&self) -> u64 {
        self.owner_id
    }

    fn initialize_plugins(&mut self) -> Result<(), JsException> {
        if self.plugins.is_empty() {
            return Ok(());
        }
        self.js.set_execution_time_limit(self.plugin_timeout);
        let result = (|| {
            self.plugin_runtime = Some(super::plugins::PluginRuntime::install(
                &self.js,
                &self.plugins,
                self.owner_id,
                self.realm_id,
                self.session_id,
                self.browsing_context.clone(),
                self.events.clone(),
                &self.bridge,
            )?);
            let start = Instant::now();
            loop {
                self.check_operation()
                    .map_err(|error| JsException::from_message(error.to_string()))?;
                if self.shutdown.load(Ordering::Acquire) {
                    return Err(JsException::from_message("page initialization cancelled"));
                }
                if self
                    .plugin_runtime
                    .as_ref()
                    .expect("installed plugins")
                    .ready(&self.js)?
                {
                    return Ok(());
                }
                if start.elapsed() >= self.plugin_timeout {
                    return Err(JsException::from_message("plugin installation timed out"));
                }
                if !self.run_one_task()? && !self.advance_to_next_timer() {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        })();
        self.js.clear_execution_time_limit();
        if let Some(deadline) = self.script_execution_deadline {
            self.js
                .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        }
        result
    }

    fn close_plugins(&mut self) {
        let Some(runtime) = self.plugin_runtime.take() else {
            return;
        };
        self.js.set_execution_time_limit(self.plugin_timeout);
        let _ = runtime.cancel(&self.js);
        let start = Instant::now();
        while !runtime.disposed(&self.js).unwrap_or(true) && start.elapsed() < self.plugin_timeout {
            if runtime.pump(&self.js).is_err() || self.run_one_task().is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        self.js.clear_execution_time_limit();
    }

    pub(crate) fn check_operation(&self) -> Result<(), super::AutomationError> {
        if self.cancellation.is_cancelled() {
            return Err(super::AutomationError::Cancellation);
        }
        if self.shutdown.load(Ordering::Acquire) {
            return Err(super::AutomationError::Closed);
        }
        if self
            .operation_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(super::AutomationError::Timeout(Duration::ZERO));
        }
        Ok(())
    }

    pub(crate) fn set_script_execution_deadline(&mut self, timeout: Duration) {
        self.script_execution_deadline = Instant::now().checked_add(timeout);
        if let Some(deadline) = self.script_execution_deadline {
            self.js
                .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        }
    }

    pub(crate) fn clear_script_execution_deadline(&mut self) {
        self.js.clear_execution_time_limit();
        self.script_execution_deadline = None;
    }

    /// Replaces the current document and JavaScript realm with a clean,
    /// unnavigated page while retaining page-scoped transport and persona
    /// configuration.
    pub fn reset(&mut self) -> Result<(), JsException> {
        const BLANK_URL: &str = "about:blank";
        const BLANK_DOCUMENT: &str = "<!doctype html><html><head></head><body></body></html>";
        let _ = self.browsing_context.take_pending_navigation();
        self.reset_page_at(BLANK_URL, false, encoding_rs::UTF_8, "text/html")?;
        self.set_content_at(BLANK_DOCUMENT, Some(BLANK_URL))?;
        self.load_state = LoadState::Idle;
        Ok(())
    }

    pub async fn goto(&mut self, url: &str) -> Result<NavigationResponse, NavigationError> {
        self.goto_with_headers(url, HeaderList::new()).await
    }

    pub async fn goto_with_headers(
        &mut self,
        url: &str,
        headers: HeaderList,
    ) -> Result<NavigationResponse, NavigationError> {
        let mut request = self.resource_request(url)?;
        request.headers = headers;
        self.goto_request(request, true, 20).await
    }

    pub async fn goto_request(
        &mut self,
        request: ResourceRequest,
        allow_redirects: bool,
        max_redirects: usize,
    ) -> Result<NavigationResponse, NavigationError> {
        if (request.method == http::Method::GET || request.method == http::Method::HEAD)
            && request.body.is_some()
        {
            return Err(NavigationError::InvalidRequest(format!(
                "{} navigation cannot have a body",
                request.method
            )));
        }
        let mut request = request;
        request.resource_type = Some("Document".into());
        self.close_plugins();
        if let Some(network) = &self.network {
            network.routes.cancel_pending();
            network.cancel_streams();
        }
        self.events.emit(super::events::PageEvent::Lifecycle {
            name: "navigation".into(),
            url: request.url.clone(),
        });
        let started = Instant::now();
        self.load_state = LoadState::Loading;
        let fetched = match crate::runtime::request::fetch_with_redirects(
            self.network_scope.loader.as_ref(),
            &self.browsing_context,
            request,
            crate::runtime::request::RedirectOptions {
                include_credentials: true,
                follow: allow_redirects,
                limit: max_redirects,
            },
        )
        .await
        {
            Ok(response) => response,
            Err(error) => {
                self.load_state = LoadState::Failed;
                return Err(error.into());
            }
        };
        let request = navigation_request_info(&fetched.request);
        let history = fetched
            .history
            .iter()
            .map(|hop| NavigationHistoryEntry {
                status_code: hop.status.as_u16(),
                reason: hop.status.canonical_reason().unwrap_or_default().to_owned(),
                url: hop.url.clone(),
                headers: header_pairs(&hop.headers),
                request: navigation_request_info(&hop.request),
            })
            .collect();
        let response = fetched.response;
        let cross_origin_isolated = response_is_cross_origin_isolated(&response.headers);
        let status_code = response.status.as_u16();
        let reason = response
            .status
            .canonical_reason()
            .unwrap_or_default()
            .to_owned();
        let headers = header_pairs(&response.headers);
        let metadata = response.metadata;
        let is_html = response
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .is_none_or(|mime| {
                mime.trim().eq_ignore_ascii_case("text/html")
                    || mime.trim().eq_ignore_ascii_case("application/xhtml+xml")
            });
        let effective_url = response.effective_url;
        let content = response.body;
        let decoded = if is_html {
            Some(
                crate::dom::decode_html(
                    &content,
                    response
                        .headers
                        .get(http::header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok()),
                )
                .map_err(|error| {
                    self.load_state = LoadState::Failed;
                    JsException::from_message(error.to_string())
                })?,
            )
        } else {
            None
        };
        let document_encoding = decoded
            .as_ref()
            .map_or(encoding_rs::UTF_8, |(_, encoding)| *encoding);
        if let Err(error) = self.reset_page_at(
            &effective_url,
            cross_origin_isolated,
            document_encoding,
            response
                .headers
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("text/html"),
        ) {
            self.load_state = LoadState::Failed;
            return Err(error.into());
        }
        self.bindings.script_control(
            &self.js,
            serde_json::json!({"op":"ready", "state":"loading"}),
        )?;
        for (_, source) in &self.preload_scripts {
            self.execute_page_script(source);
        }
        let html = if let Some((source, _)) = decoded {
            if let Err(error) = self
                .parse_navigation_document(&source, &effective_url)
                .await
            {
                self.load_state = LoadState::Failed;
                return Err(error);
            }
            self.document_ready = true;
            self.execute_page_script(
                "document.dispatchEvent(new Event('DOMContentLoaded', {bubbles:true}))",
            );
            self.events.emit(super::events::PageEvent::Lifecycle {
                name: "dom_content_loaded".into(),
                url: effective_url.clone(),
            });
            if self.wait_until == NavigationWait::Load {
                while self.pending_navigation_scripts.get() > 0 {
                    self.run_pending_tasks()?;
                    tokio::task::yield_now().await;
                    self.wait_for_work(Some(Duration::from_millis(5)));
                }
                self.finish_navigation_load()?;
            } else if self.pending_navigation_scripts.get() == 0 {
                // Leave the load milestone for the next owner turn.
                let realm = self.realm_id;
                self.tasks.push(move |page| {
                    if page.realm_id == realm {
                        let _ = page.finish_navigation_load();
                    }
                });
            }
            if self.wait_until == NavigationWait::Load {
                self.process_dynamic_scripts().await;
                let _ = self.run_pending_tasks();
                self.process_dynamic_scripts().await;
            }
            Some(self.document.borrow().outer_html())
        } else {
            None
        };
        let cookies = response_cookie_pairs(&response.headers);
        if !is_html {
            self.finish_navigation_load()?;
        }
        if let Some(destination) = self.browsing_context.take_pending_navigation() {
            return Box::pin(self.goto(&destination)).await;
        }
        Ok(NavigationResponse {
            status_code,
            reason,
            url: effective_url,
            headers,
            content,
            html,
            cookies,
            elapsed: started.elapsed(),
            request,
            history,
            http_version: metadata.http_version,
            downloaded_bytes: metadata.downloaded_bytes,
            uploaded_bytes: metadata.uploaded_bytes,
            header_bytes: metadata.header_bytes,
        })
    }

    fn finish_navigation_load(&mut self) -> Result<(), JsException> {
        if self.load_state == LoadState::Complete {
            return Ok(());
        }
        self.bindings.script_control(
            &self.js,
            serde_json::json!({"op":"ready", "state":"complete"}),
        )?;
        self.load_state = LoadState::Complete;
        self.execute_page_script("window.dispatchEvent(new Event('load'))");
        self.events.emit(super::events::PageEvent::Lifecycle {
            name: "load".into(),
            url: self.url().unwrap_or_default(),
        });
        Ok(())
    }

    pub fn html(&self) -> String {
        self.document.borrow().outer_html()
    }

    pub async fn wait_for_load(&self) -> Result<(), NavigationError> {
        match self.load_state {
            LoadState::Complete => Ok(()),
            LoadState::Failed => Err(NavigationError::LoadFailed),
            state => Err(NavigationError::NotLoaded(state)),
        }
    }

    pub fn load_state(&self) -> LoadState {
        self.load_state
    }

    pub fn url(&self) -> Option<String> {
        self.browsing_context.current_url()
    }

    pub fn add_preload_script(&mut self, identifier: String, source: String) {
        self.preload_scripts.push((identifier, source));
    }

    pub fn remove_preload_script(&mut self, identifier: &str) -> bool {
        let original_len = self.preload_scripts.len();
        self.preload_scripts
            .retain(|(candidate, _)| candidate != identifier);
        self.preload_scripts.len() != original_len
    }

    pub(crate) fn set_network_offline(&mut self, offline: bool) -> Result<(), JsException> {
        self.js.call_function_with_string(
            &self.persona_identity_setter,
            &serde_json::json!({"onLine": !offline}).to_string(),
        )?;
        self.network_offline = offline;
        Ok(())
    }

    pub(crate) fn set_identity(
        &mut self,
        identity: super::native::IdentityOptions,
    ) -> Result<(), super::AutomationError> {
        let mut persona = self.persona.clone();
        identity.apply(&mut persona)?;
        let mut headers = Vec::new();
        let mut values = serde_json::Map::new();
        if identity.user_agent.is_some() {
            headers.push(("user-agent".into(), persona.base_headers.user_agent.clone()));
            values.insert(
                "userAgent".into(),
                serde_json::json!(persona.base_headers.user_agent),
            );
            values.insert(
                "appVersion".into(),
                serde_json::json!(
                    persona
                        .base_headers
                        .user_agent
                        .trim_start_matches("Mozilla/")
                ),
            );
        }
        if identity.languages.is_some() {
            headers.push((
                "accept-language".into(),
                persona.base_headers.accept_language.clone(),
            ));
            values.insert("language".into(), serde_json::json!(persona.locale));
            values.insert("languages".into(), serde_json::json!(persona.languages));
        }
        if identity.platform.is_some() {
            values.insert("platform".into(), serde_json::json!(persona.platform));
        }
        self.browsing_context
            .update_identity_headers(headers)
            .map_err(super::AutomationError::InvalidInput)?;
        self.persona = persona;
        self.set_navigator_identity_override(serde_json::Value::Object(values).to_string())
            .map_err(|error| super::AutomationError::JavaScript(error.to_string()))
    }

    pub(crate) fn set_navigator_identity_override(
        &mut self,
        identity_override: String,
    ) -> Result<(), JsException> {
        self.js
            .call_function_with_string(&self.persona_identity_setter, &identity_override)?;
        let values =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&identity_override)
                .map_err(|error| JsException::from_message(error.to_string()))?;
        for (name, value) in values {
            if value.is_null() {
                self.navigator_identity_override.remove(&name);
            } else {
                self.navigator_identity_override.insert(name, value);
            }
        }
        Ok(())
    }

    async fn fetch_success(
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

    fn resource_request(&self, url: &str) -> Result<ResourceRequest, NavigationError> {
        Ok(ResourceRequest::get(url))
    }

    fn accept_success_response(
        &self,
        response: crate::network::ResourceResponse,
    ) -> Result<crate::network::ResourceResponse, NavigationError> {
        if response.status.is_success() {
            Ok(response)
        } else {
            Err(NavigationError::HttpStatus(response.status.as_u16()))
        }
    }

    pub fn eval(&self, source: &str) -> Result<JsValue<'_>, JsException> {
        let value = self.js.eval(source)?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub(crate) fn call_native(
        &self,
        source: &str,
        input: &str,
        object: &ProtectedJsObject,
    ) -> Result<JsValue<'_>, JsException> {
        let function = self.js.eval(&format!("({source})"))?.to_object()?;
        let value = self
            .js
            .call_function_with_string_and_object(&function, input, object)?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub fn extract(
        &mut self,
        options: ExtractionOptions,
    ) -> Result<ExtractedDocument, ExtractionError> {
        if self.defuddle_extractor.is_none() {
            if !self.plugins.iter().any(|plugin| plugin.name == "defuddle") {
                return Err(ExtractionError::JavaScript(
                    "page.extract requires the defuddle plugin".into(),
                ));
            }
            self.defuddle_extractor = Some(
                self.js
                    .eval("globalThis[Symbol.for('brimp.extract')]")
                    .and_then(|value| value.to_object())
                    .map_err(|error| ExtractionError::JavaScript(error.to_string()))?,
            );
        }
        crate::runtime::extraction::extract(
            &self.js,
            self.defuddle_extractor
                .as_ref()
                .expect("Defuddle extractor was installed"),
            &options,
            Some(
                self.browsing_context
                    .current_url()
                    .as_deref()
                    .unwrap_or("about:blank"),
            ),
        )
    }

    pub(crate) fn dispatch_input(&self, serialized_command: &str) -> Result<String, JsException> {
        let value = self.bindings.dispatch_input(&self.js, serialized_command)?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub(crate) fn dispatch_input_object(
        &self,
        command: &str,
        target: &ProtectedJsObject,
    ) -> Result<String, JsException> {
        let result = self
            .bindings
            .dispatch_input_object(&self.js, command, target)?;
        self.start_pending_fetches()?;
        Ok(result)
    }

    fn install_console_events(&self) -> Result<(), JsException> {
        let events = self.events.clone();
        let callback = self.console_callback.clone();
        self.js.set_console_event_callback(move |level, message| {
            events.emit(super::events::PageEvent::Console {
                level: level.into(),
                message: message.into(),
            });
            if let Some(callback) = &callback {
                callback(message);
            }
        })
    }

    pub fn set_console_callback<F>(&mut self, callback: F) -> Result<(), JsException>
    where
        F: Fn(&str) + 'static,
    {
        let callback: Rc<dyn Fn(&str)> = Rc::new(callback);
        self.console_callback = Some(callback);
        self.install_console_events()
    }

    pub(crate) fn network_context(&self) -> Arc<BrowsingContext> {
        self.browsing_context.clone()
    }

    pub fn document(&self) -> Ref<'_, BrowserDocument> {
        self.document.borrow()
    }

    pub fn document_mut(&self) -> RefMut<'_, BrowserDocument> {
        self.document.borrow_mut()
    }

    pub fn wrapper_cache(&self) -> &WrapperCache {
        self.bindings.wrapper_cache()
    }

    pub fn tasks(&mut self) -> &mut TaskQueue {
        &mut self.tasks
    }

    pub fn task_sender(&self) -> TaskSender {
        self.tasks.sender()
    }

    fn install_diagnostics(&mut self) -> Result<(), JsException> {
        let Some(log) = self.diagnostics.clone() else {
            return Ok(());
        };
        let stack = log.stack();
        let recursive = log.mode() == super::diagnostics::TraceMode::Recursive;
        self.js.set_global_function("__brimpTrace", move |call| {
            let text = |index| {
                call.argument(index)
                    .ok_or_else(|| brimp_jsc::NativeError::new("missing trace field"))?
                    .to_string()
            };
            let stack = text(3)?;
            let details = serde_json::from_str(&text(4)?).map_err(brimp_jsc::NativeError::new)?;
            log.record(
                &text(0)?,
                &text(1)?,
                &text(2)?,
                (!stack.is_empty()).then_some(stack),
                Some(details),
            );
            Ok(brimp_jsc::NativeValue::Undefined)
        })?;
        let controls = self.js.eval(&format!(
            "({})({stack}, {})",
            include_str!("../../js/runtime/diagnostics.js"),
            if recursive {
                include_str!("../../js/runtime/recursive-tracing.js")
            } else {
                "null"
            },
        ))?;
        if recursive {
            self.bindings
                .trace_internal_controls(&self.js, &controls.to_object()?)?;
        }
        Ok(())
    }

    pub(crate) fn diagnostics(
        &self,
        view: super::diagnostics::DiagnosticsView,
    ) -> serde_json::Value {
        self.diagnostics
            .as_ref()
            .map(|log| log.read(view))
            .unwrap_or_else(|| super::diagnostics::disabled(view))
    }

    pub(crate) fn wait_for_work(&self, maximum: Option<Duration>) {
        // Engine worker completions target the platform loop, not Rust's park
        // token. Bound the wait so engine jobs also progress without host traffic.
        let maximum = Some(
            maximum
                .unwrap_or(Duration::from_millis(10))
                .min(Duration::from_millis(10)),
        );
        let timer = self.timers.borrow().wait_duration();
        match (timer, maximum) {
            (Some(a), Some(b)) => std::thread::park_timeout(a.min(b)),
            (Some(a), None) | (None, Some(a)) => std::thread::park_timeout(a),
            (None, None) => std::thread::park(),
        }
    }

    pub(crate) fn advance_bridge_timer(&mut self) -> bool {
        self.bridge.has_pending() && self.advance_to_next_timer()
    }

    pub(crate) fn advance_to_next_timer(&mut self) -> bool {
        let mut timers = self.timers.borrow_mut();
        if !timers.clock.logical() {
            return false;
        }
        if let Some(next) = timers.next_deadline() {
            timers.clock.advance_to(next);
            true
        } else {
            false
        }
    }

    pub(crate) fn drain_tasks(
        &mut self,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        self.drive_clock(None, max_tasks)
    }
    pub(crate) fn advance_time(
        &mut self,
        milliseconds: f64,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        if !self.timers.borrow().clock.logical() {
            return Err(JsException::from_message(
                "advance_time requires logical time",
            ));
        }
        let delta = Duration::try_from_secs_f64(milliseconds / 1000.0).map_err(|_| {
            JsException::from_message("time advance must be finite and nonnegative")
        })?;
        let target = self
            .timers
            .borrow()
            .clock
            .now()
            .checked_add(delta)
            .ok_or_else(|| JsException::from_message("clock overflow"))?;
        self.drive_clock(Some(target), max_tasks)
    }
    fn drive_clock(
        &mut self,
        target: Option<Duration>,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        if max_tasks == 0 || max_tasks > 1_000_000 {
            return Err(JsException::from_message("max_tasks must be 1..1000000"));
        }
        let mut tasks = 0;
        loop {
            if self.shutdown.load(Ordering::Acquire) {
                return Err(JsException::from_message("page closed"));
            }
            if self
                .script_execution_deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return Err(JsException::from_message("clock operation timed out"));
            }
            if tasks == max_tasks {
                return Err(JsException::from_message(
                    "task budget exhausted; clock stopped at last processed task",
                ));
            }
            if self.run_one_task()? {
                tasks += 1;
                continue;
            }
            let Some(target) = target else {
                break;
            };
            let next = self.timers.borrow().next_deadline();
            if let Some(next) = next.filter(|next| *next <= target) {
                self.timers.borrow_mut().clock.advance_to(next);
            } else {
                self.timers.borrow_mut().clock.advance_to(target);
                break;
            }
        }
        Ok(
            serde_json::json!({"now": self.timers.borrow().clock.now().as_secs_f64() * 1000.0, "tasks": tasks}),
        )
    }

    pub fn run_one_task(&mut self) -> Result<bool, JsException> {
        let deadline = self
            .script_execution_deadline
            .unwrap_or_else(|| Instant::now() + self.plugin_timeout);
        self.js
            .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        self.js.pump_jobs();
        let bridge_progress = self.bridge.pump(&self.js);
        if let Some(deadline) = self.script_execution_deadline {
            self.js
                .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        } else {
            self.js.clear_execution_time_limit();
        }
        let bridge_progress = bridge_progress?;
        let plugin_progress = self
            .plugin_runtime
            .as_ref()
            .map(|runtime| runtime.pump(&self.js))
            .transpose()?
            .unwrap_or(false);
        self.start_pending_fetches()?;
        self.start_pending_websockets()?;
        let timer_callback = self.timers.borrow_mut().pop_due();
        if let Some(callback) = timer_callback {
            self.js.call_function(&callback)?;
            self.start_pending_fetches()?;
            self.start_pending_websockets()?;
            return Ok(true);
        }
        let Some(task) = self.tasks.pop() else {
            return Ok(plugin_progress || bridge_progress);
        };
        task(self);
        if let Some(error) = self.async_error.take() {
            return Err(error);
        }
        self.start_pending_fetches()?;
        self.start_pending_websockets()?;
        Ok(true)
    }

    pub fn run_pending_tasks(&mut self) -> Result<(), JsException> {
        for _ in 0..1000 {
            if self.shutdown.load(Ordering::Acquire) {
                break;
            }
            if !self.run_one_task()? {
                break;
            }
        }
        Ok(())
    }

    pub fn run_until_idle_for(&mut self, timeout: Duration) -> Result<bool, JsException> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut progressed = false;
            while self.run_one_task()? {
                progressed = true;
            }
            if self.fetches.borrow().is_empty()
                && self.fetch_streams.borrow().is_empty()
                && self.tasks.is_empty()
            {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            if !progressed {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    pub(crate) fn is_network_idle(&mut self) -> bool {
        self.fetches.borrow().is_empty()
            && self.fetch_streams.borrow().is_empty()
            && self.tasks.is_empty()
    }

    fn start_pending_fetches(&self) -> Result<(), JsException> {
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
                let result = super::request_stream::fetch_stream_callback(
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

    fn start_pending_websockets(&mut self) -> Result<(), JsException> {
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

    fn close_websockets(&mut self) {
        for (_, socket) in self.websockets.drain() {
            let _ = socket.close();
        }
    }

    fn cancel_fetch_streams(&self) {
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
        request.body = pending.body;
        Ok(request)
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

fn next_realm_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl Drop for Page {
    fn drop(&mut self) {
        for job in self.navigation_jobs.borrow_mut().drain(..) {
            job.abort();
        }
        self.close_plugins();
        self.close_websockets();
        self.cancel_fetch_streams();
        self.events.emit(super::events::PageEvent::Lifecycle {
            name: "closed".into(),
            url: self.browsing_context.current_url().unwrap_or_default(),
        });
        if let Some(network) = &self.network {
            network.close();
        }
        self.events.clear();
    }
}

fn response_is_cross_origin_isolated(headers: &HeaderList) -> bool {
    let opener = headers
        .get("cross-origin-opener-policy")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("same-origin"));
    let embedder = headers
        .get("cross-origin-embedder-policy")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            let value = value.trim();
            value.eq_ignore_ascii_case("require-corp")
                || value.eq_ignore_ascii_case("credentialless")
        });
    opener && embedder
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadState {
    Idle,
    Loading,
    Complete,
    Failed,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationResponse {
    pub status_code: u16,
    pub reason: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    #[serde(skip)]
    pub content: Vec<u8>,
    pub html: Option<String>,
    pub cookies: Vec<(String, String)>,
    pub elapsed: Duration,
    pub request: NavigationRequestInfo,
    pub history: Vec<NavigationHistoryEntry>,
    pub http_version: Option<String>,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub header_bytes: u64,
}

fn header_pairs(headers: &HeaderList) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_owned(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect()
}

fn response_cookie_pairs(headers: &HeaderList) -> Vec<(String, String)> {
    headers
        .get_all(http::header::SET_COOKIE)
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

fn navigation_request_info(request: &ResourceRequest) -> NavigationRequestInfo {
    NavigationRequestInfo {
        method: request.method.as_str().to_owned(),
        url: request.url.clone(),
        headers: header_pairs(&request.headers),
        body: request.body.clone(),
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationRequestInfo {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationHistoryEntry {
    pub status_code: u16,
    pub reason: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub request: NavigationRequestInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error("navigation returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("invalid resource URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid navigation request: {0}")]
    InvalidRequest(String),
    #[error("script response is not UTF-8: {0}")]
    InvalidScriptUtf8(#[from] std::string::FromUtf8Error),
    #[error("document query failed: {0}")]
    Document(String),
    #[error("invalid cookie header: {0}")]
    Cookie(String),
    #[error("script loading task failed: {0}")]
    ScriptTask(#[from] tokio::task::JoinError),
    #[error(transparent)]
    JavaScript(#[from] JsException),
    #[error("page load failed")]
    LoadFailed,
    #[error("page is not loaded (state: {0:?})")]
    NotLoaded(LoadState),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationWait {
    DomContentLoaded,
    #[default]
    Load,
}

fn install_interrupt(
    js: &JsRuntime,
    deadline: Option<Instant>,
    cancellation: super::CancellationToken,
    shutdown: Arc<AtomicBool>,
) {
    js.set_interrupt_handler(Arc::new(move || {
        cancellation.is_cancelled()
            || shutdown.load(Ordering::Acquire)
            || deadline.is_some_and(|deadline| Instant::now() >= deadline)
    }));
}

#[derive(Clone, Debug)]
pub struct PageOptions {
    pub(crate) session_id: u64,
    pub(super) operation_deadline: Option<Instant>,
    pub(super) cancellation: super::CancellationToken,
    wait_until: NavigationWait,
    clock: super::clock::ClockOptions,
    diagnostics: Option<super::diagnostics::DiagnosticsOptions>,
    events: super::events::EventHub,
    network: Option<Arc<super::networking::NetworkRuntime>>,
    plugins: Vec<super::plugins::Plugin>,
    plugin_timeout: Duration,
    pub(crate) shutdown: Arc<AtomicBool>,
    persona: crate::persona::PersonaConfig,
    request_headers: Vec<(String, String)>,
}

impl Default for PageOptions {
    fn default() -> Self {
        let persona = crate::persona::PersonaConfig::default();
        Self {
            session_id: 0,
            operation_deadline: None,
            cancellation: Default::default(),
            wait_until: Default::default(),
            clock: Default::default(),
            diagnostics: None,
            events: super::events::EventHub::default(),
            network: None,
            plugins: Vec::new(),
            plugin_timeout: Duration::from_secs(30),
            shutdown: Arc::new(AtomicBool::new(false)),
            persona,
            request_headers: Vec::new(),
        }
    }
}

impl PageOptions {
    pub(crate) fn network_runtime(&self) -> Option<Arc<super::networking::NetworkRuntime>> {
        self.network.clone()
    }
    pub fn builder() -> PageOptionsBuilder {
        PageOptionsBuilder::default()
    }

    pub fn persona(&self) -> &crate::persona::PersonaConfig {
        &self.persona
    }

    pub fn request_headers(&self) -> &[(String, String)] {
        &self.request_headers
    }

    pub(crate) fn with_persona(mut self, persona: crate::persona::PersonaConfig) -> Self {
        self.persona = persona;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct PageOptionsBuilder {
    options: PageOptions,
}

impl PageOptionsBuilder {
    pub fn operation_control(
        mut self,
        deadline: Option<Instant>,
        cancellation: super::CancellationToken,
    ) -> Self {
        self.options.operation_deadline = deadline;
        self.options.cancellation = cancellation;
        self
    }
    pub fn wait_until(mut self, wait_until: NavigationWait) -> Self {
        self.options.wait_until = wait_until;
        self
    }
    pub fn clock(mut self, options: super::clock::ClockOptions) -> Self {
        self.options.clock = options;
        self
    }
    pub fn diagnostics(mut self, options: Option<super::diagnostics::DiagnosticsOptions>) -> Self {
        self.options.diagnostics = options;
        self
    }

    pub(crate) fn events(mut self, events: super::events::EventHub) -> Self {
        self.options.events = events;
        self
    }
    pub(crate) fn network(mut self, network: Arc<super::networking::NetworkRuntime>) -> Self {
        self.options.network = Some(network);
        self
    }
    pub fn plugins(mut self, plugins: Vec<super::plugins::Plugin>) -> Self {
        self.options.plugins = plugins;
        self
    }
    pub fn plugin_timeout(mut self, timeout: Duration) -> Self {
        self.options.plugin_timeout = timeout;
        self
    }
    pub fn request_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.options.request_headers = headers;
        self
    }

    pub fn build(self) -> PageOptions {
        self.options
    }
}

fn install_persona(
    runtime: &JsRuntime,
    persona: &crate::persona::PersonaConfig,
) -> Result<ProtectedJsObject, JsException> {
    let persona = serde_json::to_string(persona)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let installer = include_str!("../../js/runtime/persona.js");
    runtime
        .eval(&format!("({installer})({persona})"))?
        .to_object()
}

fn persona_request_headers(persona: &crate::persona::PersonaConfig) -> Vec<(String, String)> {
    persona
        .base_headers
        .headers()
        .into_iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}
