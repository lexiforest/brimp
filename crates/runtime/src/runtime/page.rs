mod navigation;
mod network;
mod options;
mod plugins;
mod profile;
mod scripts;
mod tasks;

pub use navigation::{
    LoadState, NavigationError, NavigationHistoryEntry, NavigationRequestInfo, NavigationResponse,
    NavigationWait,
};
pub use options::{PageOptions, PageOptionsBuilder};
use profile::install_profile;

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
use crate::network::ResourceLoader;
use crate::web_apis::{
    BindingQueues, BindingRuntime, BrowsingContext, CookieJar, FetchQueue, StreamingQueue,
    TimerQueue, WrapperCache,
};
use brimp_jsc::{JsException, JsRuntime, JsValue, ProtectedJsObject};

use crate::runtime::{ExtractedDocument, ExtractionError, ExtractionOptions, TaskQueue};

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
    profile_identity_setter: ProtectedJsObject,
    profile_controller: ProtectedJsObject,
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
    profile: crate::profile::BrowserProfile,
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
        let mut browsing_context = BrowsingContext::with_cookie_jar(cookies);
        browsing_context.storage = options.storage.clone();
        let browsing_context = Arc::new(browsing_context);
        let mut request_headers = options
            .profile
            .request_headers()
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect::<Vec<_>>();
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
            .eval(include_str!("../../js/internal/module_loader.js"))?
            .to_object()?;
        let (profile_identity_setter, profile_controller) = install_profile(&js, &options.profile)?;
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
            profile_identity_setter,
            profile_controller,
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
            profile: options.profile,
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
        page.profile_command("finish")?;
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
            .eval(include_str!("../../js/internal/module_loader.js"))?
            .to_object()?;
        let (profile_identity_setter, profile_controller) = install_profile(&js, &self.profile)?;
        if !self.navigator_identity_override.is_empty() {
            js.call_function_with_string(
                &profile_identity_setter,
                &serde_json::Value::Object(self.navigator_identity_override.clone()).to_string(),
            )?;
        }
        js.call_function_with_string(
            &profile_identity_setter,
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
        self.profile_identity_setter = profile_identity_setter;
        self.profile_controller = profile_controller;
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
        self.profile_command("finish")?;
        self.bindings.finalize_exposure(&self.js)?;
        self.install_diagnostics()?;
        Ok(())
    }

    pub(crate) fn id(&self) -> u64 {
        self.owner_id
    }

    /// Replaces the current document and JavaScript realm with a clean,
    /// unnavigated page while retaining page-scoped transport and profile
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

    pub fn html(&self) -> String {
        self.document.borrow().outer_html()
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
                    .eval("globalThis[Symbol.for('extract')]")
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

    pub fn document(&self) -> Ref<'_, BrowserDocument> {
        self.document.borrow()
    }

    pub fn document_mut(&self) -> RefMut<'_, BrowserDocument> {
        self.document.borrow_mut()
    }

    pub fn wrapper_cache(&self) -> &WrapperCache {
        self.bindings.wrapper_cache()
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
            include_str!("../../js/internal/diagnostics.js"),
            if recursive {
                include_str!("../../js/internal/recursive-tracing.js")
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
