use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use blitz_traits::net::NetProvider;
use browser_dom::{BrowserDocument, HtmlParserSession, NodeId, ParseProgress};
use jsc::{JsException, JsRuntime, JsValue, ProtectedJsObject};
use network::{HeaderList, NetworkError, ResourceLoader, ResourceRequest};
use screenshot::{ScreenshotError, ScreenshotOptions};
use web_bindings::{
    BindingQueues, BindingRuntime, BrowsingContext, CookieJar, FetchQueue, PendingFetch,
    PendingWebSocketOperation, PendingWorkerOperation,
    PersistentStorage as BindingPersistentStorage, StreamingQueue, TimerQueue, WebFeatureFlags,
    WorkerQueue, WrapperCache,
};

use crate::{
    ExtractedDocument, ExtractionError, ExtractionOptions, TaskQueue, TaskSender, Viewport,
    blitz_net::BlitzResourceProvider,
    worker::{ServiceWorkerResponse, WorkerCoordinator, WorkerRealm},
};

pub struct Page {
    // Bindings must drop before `js`, because their objects are protected in that context.
    bindings: BindingRuntime,
    persona_identity_setter: ProtectedJsObject,
    defuddle_extractor: Option<ProtectedJsObject>,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    blitz_network: Arc<BlitzResourceProvider>,
    fetches: Rc<RefCell<FetchQueue>>,
    js: JsRuntime,
    document: Rc<RefCell<BrowserDocument>>,
    tasks: TaskQueue,
    viewport: Viewport,
    network_scope: DocumentNetworkScope,
    load_state: LoadState,
    document_ready: bool,
    async_error: Option<JsException>,
    persona: persona::ResolvedPersona,
    subsystems: BrowserSubsystemOptions,
    navigator_identity_override: Option<String>,
    preload_scripts: Vec<(String, String)>,
    worker_queue: Rc<RefCell<WorkerQueue>>,
    workers: HashMap<u64, WorkerInstance>,
    worker_coordinator: WorkerCoordinator,
    streaming_queue: Rc<RefCell<StreamingQueue>>,
    websockets: HashMap<u64, network::WebSocketHandle>,
    fetch_streams: RefCell<HashMap<u64, network::ResourceStreamHandle>>,
    service_worker: Option<(u64, String)>,
}

#[derive(Clone)]
pub(crate) struct DocumentNetworkScope {
    loader: Arc<dyn ResourceLoader>,
}

impl DocumentNetworkScope {
    pub(crate) fn new(loader: Arc<dyn ResourceLoader>) -> Self {
        Self { loader }
    }
}

impl Page {
    pub(crate) fn new(
        options: PageOptions,
        network_scope: DocumentNetworkScope,
        worker_coordinator: WorkerCoordinator,
        cookies: Arc<CookieJar>,
    ) -> Result<Self, JsException> {
        let js = JsRuntime::new()?;
        let browsing_context = Arc::new(BrowsingContext::with_cookie_jar(cookies));
        let mut request_headers = persona_request_headers(&options.persona);
        let persona_header_names = request_headers
            .iter()
            .map(|(name, _)| name.to_ascii_lowercase())
            .collect::<std::collections::HashSet<_>>();
        if let Some((name, _)) = options
            .request_headers
            .iter()
            .find(|(name, _)| persona_header_names.contains(&name.to_ascii_lowercase()))
        {
            return Err(JsException::from_message(format!(
                "request header `{name}` is owned by the persona"
            )));
        }
        request_headers.extend(options.request_headers.clone());
        browsing_context
            .set_request_headers(request_headers)
            .map_err(JsException::from_message)?;
        let blitz_network = Arc::new(BlitzResourceProvider::new(
            Arc::clone(&network_scope.loader),
            Arc::clone(&browsing_context),
        ));
        let net_provider: Arc<dyn NetProvider> = blitz_network.clone();
        let initial_document = BrowserDocument::empty_at_with_net(None, Some(net_provider));
        let document = Rc::new(RefCell::new(initial_document));
        let timers = Rc::new(RefCell::new(TimerQueue::default()));
        let fetches = Rc::new(RefCell::new(FetchQueue::default()));
        let worker_queue = Rc::new(RefCell::new(WorkerQueue::default()));
        let streaming_queue = Rc::new(RefCell::new(StreamingQueue::default()));
        let bindings = BindingRuntime::install(
            &js,
            Rc::clone(&document),
            Arc::clone(&browsing_context),
            false,
            options.subsystems.web_features(),
            options
                .subsystems
                .persistent_storage
                .as_ref()
                .map(|options| {
                    Arc::new(BindingPersistentStorage::new(
                        options.root.clone(),
                        options.quota_bytes,
                    ))
                }),
            BindingQueues {
                timers: Rc::clone(&timers),
                fetches: Rc::clone(&fetches),
                workers: Rc::clone(&worker_queue),
                streaming: Rc::clone(&streaming_queue),
            },
        )?;
        let persona_identity_setter = install_persona(&js, &options.persona, &options.subsystems)?;
        Ok(Self {
            bindings,
            persona_identity_setter,
            defuddle_extractor: None,
            timers,
            browsing_context,
            blitz_network,
            fetches,
            js,
            document,
            tasks: TaskQueue::default(),
            viewport: options.viewport,
            network_scope,
            load_state: LoadState::Idle,
            document_ready: false,
            async_error: None,
            persona: options.persona,
            subsystems: options.subsystems,
            navigator_identity_override: None,
            preload_scripts: Vec::new(),
            worker_queue,
            workers: HashMap::new(),
            worker_coordinator,
            streaming_queue,
            websockets: HashMap::new(),
            fetch_streams: RefCell::new(HashMap::new()),
            service_worker: None,
        })
    }

    pub fn set_content(&mut self, html: &str) -> Result<(), JsException> {
        self.set_content_at(html, None)
    }

    fn set_content_at(&mut self, html: &str, base_url: Option<&str>) -> Result<(), JsException> {
        let net_provider: Arc<dyn NetProvider> = self.blitz_network.clone();
        let mut document = BrowserDocument::parse_at_with_net(html, base_url, Some(net_provider));
        document.set_viewport(
            self.viewport.width as u32,
            self.viewport.height as u32,
            self.viewport.device_pixel_ratio as f32,
        );
        *self.document.borrow_mut() = document;
        self.document_ready = true;
        self.bindings.reset_document(&self.js)
    }

    fn reset_page_at(
        &mut self,
        base_url: &str,
        cross_origin_isolated: bool,
    ) -> Result<(), JsException> {
        self.terminate_workers();
        self.close_websockets();
        self.cancel_fetch_streams();
        let net_provider: Arc<dyn NetProvider> = self.blitz_network.clone();
        let document = BrowserDocument::empty_at_with_net(Some(base_url), Some(net_provider));
        *self.document.borrow_mut() = document;
        self.document_ready = false;
        let js = JsRuntime::new()?;
        let timers = Rc::new(RefCell::new(TimerQueue::default()));
        let fetches = Rc::new(RefCell::new(FetchQueue::default()));
        let worker_queue = Rc::new(RefCell::new(WorkerQueue::default()));
        let streaming_queue = Rc::new(RefCell::new(StreamingQueue::default()));
        let bindings = BindingRuntime::install(
            &js,
            Rc::clone(&self.document),
            Arc::clone(&self.browsing_context),
            cross_origin_isolated,
            self.subsystems.web_features(),
            self.subsystems.persistent_storage.as_ref().map(|options| {
                Arc::new(BindingPersistentStorage::new(
                    options.root.clone(),
                    options.quota_bytes,
                ))
            }),
            BindingQueues {
                timers: Rc::clone(&timers),
                fetches: Rc::clone(&fetches),
                workers: Rc::clone(&worker_queue),
                streaming: Rc::clone(&streaming_queue),
            },
        )?;
        let persona_identity_setter = install_persona(&js, &self.persona, &self.subsystems)?;
        if let Some(identity_override) = &self.navigator_identity_override {
            js.call_function_with_string(&persona_identity_setter, identity_override)?;
        }

        self.bindings = bindings;
        self.persona_identity_setter = persona_identity_setter;
        self.defuddle_extractor = None;
        self.js = js;
        self.timers = timers;
        self.fetches = fetches;
        self.worker_queue = worker_queue;
        self.streaming_queue = streaming_queue;
        self.service_worker = None;
        self.tasks = TaskQueue::default();
        self.async_error = None;
        Ok(())
    }

    /// Replaces the current document and JavaScript realm with a clean,
    /// unnavigated page while retaining page-scoped transport and persona
    /// configuration.
    pub fn reset(&mut self) -> Result<(), JsException> {
        const BLANK_URL: &str = "about:blank";
        const BLANK_DOCUMENT: &str = "<!doctype html><html><head></head><body></body></html>";
        self.browsing_context.set_url(BLANK_URL);
        self.reset_page_at(BLANK_URL, false)?;
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
        let started = Instant::now();
        self.load_state = LoadState::Loading;
        let fetched = match crate::request::fetch_with_redirects(
            self.network_scope.loader.as_ref(),
            &self.browsing_context,
            request,
            crate::request::RedirectOptions {
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
        self.browsing_context.set_url(&effective_url);
        if let Err(error) = self.reset_page_at(&effective_url, cross_origin_isolated) {
            self.load_state = LoadState::Failed;
            return Err(error.into());
        }
        for (_, source) in &self.preload_scripts {
            self.execute_page_script(source);
        }
        let html = if is_html {
            let source = String::from_utf8_lossy(&content);
            if let Err(error) = self
                .parse_navigation_document(&source, &effective_url)
                .await
            {
                self.load_state = LoadState::Failed;
                return Err(error);
            }
            self.document_ready = true;
            self.process_blitz_resources().await;
            self.execute_page_script(
                "document.dispatchEvent(new Event('DOMContentLoaded'));\
                 window.dispatchEvent(new Event('load'));",
            );
            let _ = self.run_pending_tasks();
            Some(self.document.borrow().outer_html())
        } else {
            None
        };
        let cookies = response_cookie_pairs(&response.headers);
        self.load_state = LoadState::Complete;
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

    pub(crate) fn set_navigator_identity_override(
        &mut self,
        identity_override: String,
    ) -> Result<(), JsException> {
        self.js
            .call_function_with_string(&self.persona_identity_setter, &identity_override)?;
        self.navigator_identity_override = Some(identity_override);
        Ok(())
    }

    async fn parse_navigation_document(
        &self,
        html: &str,
        document_url: &str,
    ) -> Result<(), NavigationError> {
        let base_url = url::Url::parse(document_url)?;
        let mut parser = HtmlParserSession::new(Rc::clone(&self.document), html);
        let mut viewport_installed = false;
        let mut script_order = 0;
        let mut deferred_order = Vec::new();
        let mut pending_loads = tokio::task::JoinSet::new();
        let mut loaded_deferred = BTreeMap::new();

        loop {
            let progress = parser.resume();
            if !viewport_installed {
                self.document.borrow_mut().set_viewport(
                    self.viewport.width as u32,
                    self.viewport.height as u32,
                    self.viewport.device_pixel_ratio as f32,
                );
                viewport_installed = true;
            }
            let ParseProgress::Script(node_id) = progress else {
                break;
            };
            let order = script_order;
            script_order += 1;
            let Some(script) = self.script_from_node(node_id, order) else {
                continue;
            };
            match (script.mode, script.source) {
                (ScriptMode::Blocking, ScriptSource::Inline(source)) => {
                    self.process_blitz_resources().await;
                    self.execute_page_script(&source);
                }
                (ScriptMode::Blocking, ScriptSource::External(src)) => {
                    self.process_blitz_resources().await;
                    let script_url = base_url.join(&src)?;
                    let source =
                        String::from_utf8(self.fetch_success(script_url.as_str()).await?.body)?;
                    self.execute_page_script(&source);
                }
                (mode, ScriptSource::External(src)) => {
                    if mode == ScriptMode::Defer {
                        deferred_order.push(script.order);
                    }
                    let script_url = base_url.join(&src)?.to_string();
                    let request = self.resource_request(&script_url)?;
                    let loader = Arc::clone(&self.network_scope.loader);
                    let browsing_context = Arc::clone(&self.browsing_context);
                    pending_loads.spawn(async move {
                        let result =
                            crate::request::fetch(loader.as_ref(), &browsing_context, request)
                                .await;
                        LoadedScript {
                            order: script.order,
                            mode,
                            result,
                        }
                    });
                }
                (_, ScriptSource::Inline(_)) => unreachable!("inline scripts are blocking"),
            }

            while let Some(result) = pending_loads.try_join_next() {
                self.handle_loaded_script(result?, &mut loaded_deferred)?;
            }
        }

        self.process_blitz_resources().await;
        let mut next_deferred = 0;
        self.execute_ready_deferred(&deferred_order, &mut next_deferred, &mut loaded_deferred)?;
        while let Some(result) = pending_loads.join_next().await {
            self.handle_loaded_script(result?, &mut loaded_deferred)?;
            self.execute_ready_deferred(&deferred_order, &mut next_deferred, &mut loaded_deferred)?;
        }
        Ok(())
    }

    fn script_from_node(&self, node_id: NodeId, order: usize) -> Option<Script> {
        let document = self.document.borrow();
        let node = document.node(node_id)?;
        let script_type = node
            .attr(blitz_dom::local_name!("type"))
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !matches!(
            script_type.as_str(),
            "" | "text/javascript" | "application/javascript"
        ) {
            return None;
        }
        let source = match node.attr(blitz_dom::local_name!("src")) {
            Some(src) => ScriptSource::External(src.to_owned()),
            None => ScriptSource::Inline(node.text_content()),
        };
        let mode = match source {
            ScriptSource::Inline(_) => ScriptMode::Blocking,
            ScriptSource::External(_)
                if node.attr(blitz_dom::LocalName::from("async")).is_some() =>
            {
                ScriptMode::Async
            }
            ScriptSource::External(_)
                if node.attr(blitz_dom::LocalName::from("defer")).is_some() =>
            {
                ScriptMode::Defer
            }
            ScriptSource::External(_) => ScriptMode::Blocking,
        };
        Some(Script {
            order,
            source,
            mode,
        })
    }

    fn handle_loaded_script(
        &self,
        loaded: LoadedScript,
        deferred: &mut BTreeMap<usize, String>,
    ) -> Result<(), NavigationError> {
        let response = self.accept_success_response(loaded.result?)?;
        let source = String::from_utf8(response.body)?;
        match loaded.mode {
            ScriptMode::Async => {
                self.execute_page_script(&source);
            }
            ScriptMode::Defer => {
                deferred.insert(loaded.order, source);
            }
            ScriptMode::Blocking => unreachable!("blocking scripts are not spawned"),
        }
        Ok(())
    }

    fn execute_ready_deferred(
        &self,
        order: &[usize],
        next: &mut usize,
        loaded: &mut BTreeMap<usize, String>,
    ) -> Result<(), NavigationError> {
        while let Some(script_order) = order.get(*next)
            && let Some(source) = loaded.remove(script_order)
        {
            self.execute_page_script(&source);
            *next += 1;
        }
        Ok(())
    }

    fn execute_page_script(&self, source: &str) {
        let _ = self.bindings.sync_window_named_properties(&self.js);
        let _ = self.js.eval(source);
        let _ = self.perform_microtask_checkpoint();
        let _ = self.start_pending_fetches();
    }

    async fn fetch_success(&self, url: &str) -> Result<network::ResourceResponse, NavigationError> {
        let request = self.resource_request(url)?;
        let response = crate::request::fetch(
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
        response: network::ResourceResponse,
    ) -> Result<network::ResourceResponse, NavigationError> {
        if response.status.is_success() {
            Ok(response)
        } else {
            Err(NavigationError::HttpStatus(response.status.as_u16()))
        }
    }

    pub fn eval(&self, source: &str) -> Result<JsValue<'_>, JsException> {
        let value = self.js.eval(source)?;
        self.perform_microtask_checkpoint()?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub fn extract(
        &mut self,
        options: ExtractionOptions,
    ) -> Result<ExtractedDocument, ExtractionError> {
        if self.defuddle_extractor.is_none() {
            self.defuddle_extractor = Some(crate::extraction::install(&self.js)?);
        }
        crate::extraction::extract(
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
        self.perform_microtask_checkpoint()?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub(crate) fn dispatch_input_on(
        &self,
        serialized_command: &str,
        target_expression: &str,
    ) -> Result<String, JsException> {
        let value =
            self.bindings
                .dispatch_input_on(&self.js, serialized_command, target_expression)?;
        self.perform_microtask_checkpoint()?;
        self.start_pending_fetches()?;
        Ok(value)
    }

    pub fn set_console_callback<F>(&self, callback: F) -> Result<(), JsException>
    where
        F: Fn(&str) + 'static,
    {
        self.js.set_console_callback(callback)
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

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn set_viewport(&mut self, width: u32, height: u32, device_pixel_ratio: f64) {
        self.viewport.width = f64::from(width);
        self.viewport.height = f64::from(height);
        self.viewport.device_pixel_ratio = device_pixel_ratio;
        if self.document_ready {
            self.document
                .borrow_mut()
                .set_viewport(width, height, device_pixel_ratio as f32);
        }
    }

    pub fn screenshot(&mut self, path: impl AsRef<Path>) -> Result<(), ScreenshotError> {
        let options =
            ScreenshotOptions::new(self.viewport.width as u32, self.viewport.height as u32);
        let png = self.screenshot_png(options)?;
        screenshot::save_png(path, &png)
    }

    pub fn screenshot_png(
        &mut self,
        options: ScreenshotOptions,
    ) -> Result<Vec<u8>, ScreenshotError> {
        let original = self.viewport;
        self.document.borrow_mut().set_viewport(
            options.width,
            options.height,
            original.device_pixel_ratio as f32,
        );
        self.document.borrow_mut().resolve();
        let result = (|| {
            let rasters = self
                .bindings
                .canvas_rasters()
                .map_err(ScreenshotError::Render)?;
            let layers = {
                let document = self.document.borrow();
                rasters
                    .into_iter()
                    .filter_map(|raster| {
                        document
                            .bounding_rect(raster.node)
                            .map(|rect| (raster, rect))
                    })
                    .collect::<Vec<_>>()
            };
            let mut rendered = screenshot::render_rgba(
                self.document.borrow_mut().blitz_mut(),
                options,
                original.device_pixel_ratio,
            )?;
            composite_canvas_rasters(
                &mut rendered.pixels,
                rendered.width,
                rendered.height,
                &layers,
                original.device_pixel_ratio,
            );
            screenshot::encode_rgba(&rendered.pixels, rendered.width, rendered.height)
        })();
        self.document.borrow_mut().set_viewport(
            original.width as u32,
            original.height as u32,
            original.device_pixel_ratio as f32,
        );
        result
    }

    pub fn tasks(&mut self) -> &mut TaskQueue {
        &mut self.tasks
    }

    pub fn task_sender(&self) -> TaskSender {
        self.tasks.sender()
    }

    pub fn run_one_task(&mut self) -> Result<bool, JsException> {
        self.start_pending_workers()?;
        self.start_pending_websockets()?;
        let timer_callback = self.timers.borrow_mut().pop_due();
        if let Some(callback) = timer_callback {
            self.js.call_function(&callback)?;
            self.perform_microtask_checkpoint()?;
            self.start_pending_fetches()?;
            self.start_pending_workers()?;
            self.start_pending_websockets()?;
            return Ok(true);
        }
        let Some(task) = self.tasks.pop() else {
            return Ok(false);
        };
        task(self);
        if let Some(error) = self.async_error.take() {
            return Err(error);
        }
        self.perform_microtask_checkpoint()?;
        self.start_pending_fetches()?;
        self.start_pending_workers()?;
        self.start_pending_websockets()?;
        Ok(true)
    }

    pub fn run_pending_tasks(&mut self) -> Result<(), JsException> {
        while self.run_one_task()? {}
        Ok(())
    }

    pub fn run_until_idle_for(&mut self, timeout: Duration) -> Result<bool, JsException> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut progressed = false;
            while self.run_one_task()? {
                progressed = true;
            }
            self.process_blitz_messages();
            if self.fetches.borrow().is_empty()
                && self.tasks.is_empty()
                && self.blitz_network.is_idle()
            {
                self.process_blitz_messages();
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
        self.fetches.borrow().is_empty() && self.tasks.is_empty() && self.blitz_network.is_idle()
    }

    fn perform_microtask_checkpoint(&self) -> Result<(), JsException> {
        loop {
            let callback = self.timers.borrow_mut().pop_microtask();
            let Some(callback) = callback else {
                return Ok(());
            };
            self.js.call_function(&callback)?;
        }
    }

    async fn process_blitz_resources(&self) {
        loop {
            self.blitz_network.wait_idle().await;
            self.process_blitz_messages();
            if self.blitz_network.is_idle() {
                return;
            }
        }
    }

    fn process_blitz_messages(&self) {
        let mut document = self.document.borrow_mut();
        document.blitz_mut().handle_messages();
        document.resolve();
    }

    fn start_pending_fetches(&self) -> Result<(), JsException> {
        let pending_fetches = self.fetches.borrow_mut().take_pending();
        for pending in pending_fetches {
            let id = pending.id;
            let streaming = pending.streaming;
            let mut request = match self.prepare_fetch_request(pending) {
                Ok(request) => request,
                Err(error) => {
                    self.reject_fetch(id, &error)?;
                    continue;
                }
            };
            if let Some((worker_id, key)) = &self.service_worker
                && let Some(response) = self
                    .worker_coordinator
                    .dispatch_fetch(key.clone(), request.clone())
            {
                self.complete_service_worker_fetch(id, *worker_id, streaming, response)?;
                continue;
            }
            let requested_url = request.url.clone();
            let loader = Arc::clone(&self.network_scope.loader);
            let browsing_context = Arc::clone(&self.browsing_context);
            let credentials_sent = request.headers.contains_key(http::header::COOKIE)
                || browsing_context.cookie_header(&request.url).is_some();
            let sender = self.task_sender();
            let buffered_bytes = Arc::new(AtomicUsize::new(0));
            let callback_buffered_bytes = Arc::clone(&buffered_bytes);
            if streaming {
                browsing_context.apply_request_identity(&mut request.headers);
                if let Some(cookie) = browsing_context.cookie_header(&request.url)
                    && let Ok(value) = http::HeaderValue::from_str(&cookie)
                {
                    request.headers.insert(http::header::COOKIE, value);
                }
                let result = loader.fetch_stream_callback(
                    request,
                    Box::new(move |event, control| {
                        const HIGH_WATER_MARK: usize = 256 * 1024;
                        const LOW_WATER_MARK: usize = 128 * 1024;
                        let chunk_length = match &event {
                            network::ResourceStreamEvent::Chunk(bytes) => bytes.len(),
                            _ => 0,
                        };
                        if chunk_length != 0 {
                            let reserved = callback_buffered_bytes.fetch_update(
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
                                    return network::ResourceStreamDirective::Cancel;
                                }
                                return network::ResourceStreamDirective::Pause;
                            }
                        }
                        let task_buffered_bytes = Arc::clone(&callback_buffered_bytes);
                        let task_control = control.clone();
                        if sender
                            .post(move |page| {
                                if let Err(error) = page.handle_fetch_stream_event(id, event) {
                                    page.async_error = Some(error);
                                }
                                if chunk_length != 0 {
                                    let remaining = task_buffered_bytes
                                        .fetch_sub(chunk_length, Ordering::AcqRel)
                                        .saturating_sub(chunk_length);
                                    if remaining <= LOW_WATER_MARK {
                                        task_control.resume();
                                    }
                                }
                            })
                            .is_err()
                        {
                            if chunk_length != 0 {
                                callback_buffered_bytes.fetch_sub(chunk_length, Ordering::AcqRel);
                            }
                            return network::ResourceStreamDirective::Cancel;
                        }
                        network::ResourceStreamDirective::Continue
                    }),
                );
                match result {
                    Ok(handle) => {
                        self.fetch_streams.borrow_mut().insert(id, handle);
                    }
                    Err(error) => self.reject_fetch(id, &error.to_string())?,
                }
                continue;
            }
            let cors_context = Arc::clone(&browsing_context);
            let result = crate::request::fetch_callback(
                loader,
                browsing_context,
                request,
                Box::new(move |result| {
                    if let Ok(response) = &result {
                        let effective_url = if response.effective_url.is_empty() {
                            requested_url.as_str()
                        } else {
                            response.effective_url.as_str()
                        };
                        cors_context.store_resource_cors(
                            &requested_url,
                            effective_url,
                            &response.headers,
                            credentials_sent,
                        );
                    }
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

    fn complete_service_worker_fetch(
        &self,
        id: u64,
        _worker_id: u64,
        streaming: bool,
        response: ServiceWorkerResponse,
    ) -> Result<(), JsException> {
        let status = http::StatusCode::from_u16(response.status)
            .map_err(|error| JsException::from_message(error.to_string()))?;
        let mut headers = HeaderList::new();
        for [name, value] in response.headers {
            if let (Ok(name), Ok(value)) = (
                http::HeaderName::from_bytes(name.as_bytes()),
                http::HeaderValue::from_str(&value),
            ) {
                headers.append(name, value);
            }
        }
        if streaming {
            let headers_json = headers
                .iter()
                .filter_map(|(name, value)| value.to_str().ok().map(|value| [name.as_str(), value]))
                .collect::<Vec<_>>();
            if let Some(settlement) = self.fetches.borrow_mut().take_settlement(id) {
                settlement.resolve(
                    &self.js,
                    &serde_json::json!({
                        "streamId": id,
                        "status": status.as_u16(),
                        "statusText": response.status_text,
                        "headers": headers_json,
                        "url": "",
                        "redirected": false,
                    })
                    .to_string(),
                )?;
            }
            self.bindings.deliver_fetch_stream_event(
                &self.js,
                id,
                &serde_json::json!({"type": "chunk", "bytes": response.body.into_bytes()})
                    .to_string(),
            )?;
            self.bindings.deliver_fetch_stream_event(
                &self.js,
                id,
                &serde_json::json!({"type": "complete"}).to_string(),
            )?;
        } else {
            self.complete_fetch(
                id,
                Ok((
                    String::new(),
                    network::ResourceResponse {
                        status,
                        headers,
                        body: response.body.into_bytes(),
                        effective_url: String::new(),
                        metadata: network::ResponseMetadata::default(),
                    },
                )),
            )?;
        }
        Ok(())
    }

    fn handle_fetch_stream_event(
        &self,
        id: u64,
        event: network::ResourceStreamEvent,
    ) -> Result<(), JsException> {
        match event {
            network::ResourceStreamEvent::Headers {
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
            network::ResourceStreamEvent::Chunk(bytes) => {
                self.bindings.deliver_fetch_stream_event(
                    &self.js,
                    id,
                    &serde_json::json!({"type": "chunk", "bytes": bytes}).to_string(),
                )?;
            }
            network::ResourceStreamEvent::Complete => {
                self.fetch_streams.borrow_mut().remove(&id);
                self.bindings.deliver_fetch_stream_event(
                    &self.js,
                    id,
                    &serde_json::json!({"type": "complete"}).to_string(),
                )?;
            }
            network::ResourceStreamEvent::Error(error) => {
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

    fn start_pending_workers(&mut self) -> Result<(), JsException> {
        let operations = self.worker_queue.borrow_mut().take_pending();
        for operation in operations {
            match operation {
                PendingWorkerOperation::Create {
                    id,
                    url,
                    kind,
                    name,
                    scope,
                } => {
                    let url = match url::Url::parse(&url) {
                        Ok(url) => url,
                        Err(error) => {
                            self.deliver_worker_error(id, &error.to_string())?;
                            continue;
                        }
                    };
                    self.workers
                        .insert(id, WorkerInstance::Starting(Vec::new()));
                    let loader = Arc::clone(&self.network_scope.loader);
                    let context = Arc::clone(&self.browsing_context);
                    let sender = self.task_sender();
                    let result = crate::request::fetch_callback(
                        loader,
                        context,
                        ResourceRequest::get(url.as_str()),
                        Box::new(move |result| {
                            let result = result.map_err(|error| error.to_string());
                            let _ = sender.post(move |page| {
                                page.finish_worker_start(
                                    id,
                                    url.to_string(),
                                    kind,
                                    name,
                                    scope,
                                    result,
                                )
                            });
                        }),
                    );
                    if let Err(error) = result {
                        self.deliver_worker_error(id, &error.to_string())?;
                    }
                }
                PendingWorkerOperation::Post { id, message_json } => {
                    if let Some(worker) = self.workers.get_mut(&id) {
                        match worker {
                            WorkerInstance::Starting(messages) => messages.push(message_json),
                            WorkerInstance::Running(WorkerBackend::Local(worker)) => {
                                let events = worker.borrow_mut().post_message(&message_json);
                                for event in events {
                                    self.bindings.deliver_worker_event(&self.js, id, &event)?;
                                }
                            }
                            WorkerInstance::Running(WorkerBackend::Coordinated { key }) => {
                                let events = self
                                    .worker_coordinator
                                    .post(key.clone(), message_json)
                                    .map_err(JsException::from_message)?;
                                for event in events {
                                    self.bindings.deliver_worker_event(&self.js, id, &event)?;
                                }
                            }
                        }
                    }
                }
                PendingWorkerOperation::Terminate { id } => {
                    self.workers.remove(&id);
                }
                PendingWorkerOperation::Unregister { id } => {
                    if let Some(WorkerInstance::Running(WorkerBackend::Coordinated { key })) =
                        self.workers.remove(&id)
                    {
                        self.worker_coordinator.remove(key);
                    }
                    if self
                        .service_worker
                        .as_ref()
                        .is_some_and(|(worker_id, _)| *worker_id == id)
                    {
                        self.service_worker = None;
                    }
                }
            }
        }
        Ok(())
    }

    fn finish_worker_start(
        &mut self,
        id: u64,
        url: String,
        kind: String,
        name: String,
        scope: String,
        result: Result<network::ResourceResponse, String>,
    ) {
        let pending = match self.workers.remove(&id) {
            Some(WorkerInstance::Starting(messages)) => messages,
            _ => return,
        };
        let response = match result {
            Ok(response) if response.status.is_success() => response,
            Ok(response) => {
                let _ = self.deliver_worker_error(id, &format!("HTTP {}", response.status));
                return;
            }
            Err(error) => {
                let _ = self.deliver_worker_error(id, &error);
                return;
            }
        };
        let source = match String::from_utf8(response.body) {
            Ok(source) => source,
            Err(error) => {
                let _ = self.deliver_worker_error(id, &error.to_string());
                return;
            }
        };
        if kind == "shared" || kind == "service" {
            let key = if kind == "shared" {
                format!("shared\0{url}\0{name}")
            } else {
                format!("service\0{scope}")
            };
            match self
                .worker_coordinator
                .connect(key.clone(), source, kind.clone())
            {
                Ok(mut events) => {
                    for message in pending {
                        match self.worker_coordinator.post(key.clone(), message) {
                            Ok(outputs) => events.extend(outputs),
                            Err(error) => {
                                let _ = self.deliver_worker_error(id, &error);
                            }
                        }
                    }
                    for event in events {
                        let _ = self.bindings.deliver_worker_event(&self.js, id, &event);
                    }
                    let _ = self.bindings.deliver_worker_event(
                        &self.js,
                        id,
                        &serde_json::json!({"type": "ready"}).to_string(),
                    );
                    if kind == "service" {
                        self.service_worker = Some((id, key.clone()));
                    }
                    self.workers.insert(
                        id,
                        WorkerInstance::Running(WorkerBackend::Coordinated { key }),
                    );
                }
                Err(error) => {
                    let _ = self.deliver_worker_error(id, &error);
                }
            }
            return;
        }
        match WorkerRealm::new(source, &kind) {
            Ok(mut worker) => {
                for message in pending {
                    for event in worker.post_message(&message) {
                        let _ = self.bindings.deliver_worker_event(&self.js, id, &event);
                    }
                }
                for event in worker.take_outputs() {
                    let _ = self.bindings.deliver_worker_event(&self.js, id, &event);
                }
                let _ = self.bindings.deliver_worker_event(
                    &self.js,
                    id,
                    &serde_json::json!({"type": "ready"}).to_string(),
                );
                let worker = Rc::new(RefCell::new(worker));
                self.workers
                    .insert(id, WorkerInstance::Running(WorkerBackend::Local(worker)));
            }
            Err(error) => {
                let _ = self.deliver_worker_error(id, &error);
            }
        }
    }

    fn deliver_worker_error(&self, id: u64, message: &str) -> Result<(), JsException> {
        let event = serde_json::json!({"type": "error", "message": message}).to_string();
        self.bindings.deliver_worker_event(&self.js, id, &event)
    }

    fn terminate_workers(&mut self) {
        self.service_worker = None;
        self.workers.clear();
    }

    fn start_pending_websockets(&mut self) -> Result<(), JsException> {
        let operations = self.streaming_queue.borrow_mut().take_pending();
        for operation in operations {
            match operation {
                PendingWebSocketOperation::Create { id, url } => {
                    let mut headers = HeaderList::new();
                    self.browsing_context.apply_request_identity(&mut headers);
                    let sender = self.task_sender();
                    let handle = self.network_scope.loader.open_websocket(
                        url,
                        headers,
                        Box::new(move |event| {
                            let event = match event {
                                network::WebSocketEvent::Open => serde_json::json!({"type": "open"}),
                                network::WebSocketEvent::Text(data) => serde_json::json!({"type": "message", "data": data}),
                                network::WebSocketEvent::Binary(data) => serde_json::json!({"type": "message", "data": data}),
                                network::WebSocketEvent::Close { code, reason } => serde_json::json!({"type": "close", "code": code, "reason": reason, "wasClean": code == 1000}),
                                network::WebSocketEvent::Error(message) => serde_json::json!({"type": "error", "message": message}),
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
                PendingWebSocketOperation::Close { id } => {
                    if let Some(socket) = self.websockets.remove(&id) {
                        let _ = socket.close();
                    }
                }
                PendingWebSocketOperation::CancelFetch { id } => {
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
        result: Result<(String, network::ResourceResponse), String>,
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

enum WorkerInstance {
    Starting(Vec<String>),
    Running(WorkerBackend),
}

enum WorkerBackend {
    Local(Rc<RefCell<WorkerRealm>>),
    Coordinated { key: String },
}

impl Drop for Page {
    fn drop(&mut self) {
        self.terminate_workers();
        self.close_websockets();
        self.cancel_fetch_streams();
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

enum ScriptSource {
    Inline(String),
    External(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptMode {
    Blocking,
    Defer,
    Async,
}

struct Script {
    order: usize,
    source: ScriptSource,
    mode: ScriptMode,
}

struct LoadedScript {
    order: usize,
    mode: ScriptMode,
    result: Result<network::ResourceResponse, NetworkError>,
}

#[derive(Clone, Debug)]
pub struct NavigationResponse {
    pub status_code: u16,
    pub reason: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
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

#[derive(Clone, Debug)]
pub struct NavigationRequestInfo {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct PageOptions {
    viewport: Viewport,
    persona: persona::ResolvedPersona,
    subsystems: BrowserSubsystemOptions,
    request_headers: Vec<(String, String)>,
}

impl Default for PageOptions {
    fn default() -> Self {
        let persona = persona::PersonaConfig::default().resolve();
        Self {
            viewport: Viewport {
                width: f64::from(persona.viewport.width),
                height: f64::from(persona.viewport.height),
                device_pixel_ratio: f64::from(persona.viewport.device_scale_factor),
                scroll_x: 0.0,
                scroll_y: 0.0,
            },
            persona,
            subsystems: BrowserSubsystemOptions::default(),
            request_headers: Vec::new(),
        }
    }
}

impl PageOptions {
    pub fn builder() -> PageOptionsBuilder {
        PageOptionsBuilder::default()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn persona(&self) -> &persona::ResolvedPersona {
        &self.persona
    }

    pub fn subsystems(&self) -> &BrowserSubsystemOptions {
        &self.subsystems
    }

    pub fn request_headers(&self) -> &[(String, String)] {
        &self.request_headers
    }

    pub(crate) fn with_persona(mut self, persona: persona::ResolvedPersona) -> Self {
        self.viewport = Viewport {
            width: f64::from(persona.viewport.width),
            height: f64::from(persona.viewport.height),
            device_pixel_ratio: f64::from(persona.viewport.device_scale_factor),
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        self.persona = persona;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct PageOptionsBuilder {
    options: PageOptions,
}

impl PageOptionsBuilder {
    pub fn request_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.options.request_headers = headers;
        self
    }

    pub fn worker_system(mut self, enabled: bool) -> Self {
        self.options.subsystems.worker_system = enabled;
        self
    }

    pub fn streaming_networking(mut self, enabled: bool) -> Self {
        self.options.subsystems.streaming_networking = enabled;
        self
    }

    pub fn persistent_storage(mut self, options: PersistentStorageOptions) -> Self {
        self.options.subsystems.persistent_storage = Some(options);
        self
    }

    pub fn canvas(mut self, enabled: bool) -> Self {
        self.options.subsystems.canvas = enabled;
        self
    }

    pub fn webgl(mut self, enabled: bool) -> Self {
        self.options.subsystems.webgl = enabled;
        self
    }

    pub fn webgpu(mut self, enabled: bool) -> Self {
        self.options.subsystems.webgpu = enabled;
        self
    }

    pub fn webaudio(mut self, enabled: bool) -> Self {
        self.options.subsystems.webaudio = enabled;
        if !enabled {
            self.options.subsystems.webaudio_output = false;
        }
        self
    }

    pub fn webaudio_output(mut self, enabled: bool) -> Self {
        self.options.subsystems.webaudio_output = enabled;
        if enabled {
            self.options.subsystems.webaudio = true;
        }
        self
    }

    pub fn viewport(mut self, width: u32, height: u32) -> Self {
        self.options.viewport.width = f64::from(width);
        self.options.viewport.height = f64::from(height);
        self.options.persona.viewport.width = width;
        self.options.persona.viewport.height = height;
        self.options.persona.screen.width = width;
        self.options.persona.screen.height = height;
        self.options.persona.screen.avail_width = width;
        self.options.persona.screen.avail_height = height;
        self.options.persona.window.outer_width = width;
        self.options.persona.window.outer_height = height;
        self
    }

    pub fn device_pixel_ratio(mut self, device_pixel_ratio: u32) -> Self {
        self.options.viewport.device_pixel_ratio = f64::from(device_pixel_ratio);
        self.options.persona.viewport.device_scale_factor = device_pixel_ratio;
        self.options.persona.screen.device_scale_factor = device_pixel_ratio;
        self
    }

    pub fn build(self) -> PageOptions {
        self.options
    }
}

fn install_persona(
    runtime: &JsRuntime,
    persona: &persona::ResolvedPersona,
    subsystems: &BrowserSubsystemOptions,
) -> Result<ProtectedJsObject, JsException> {
    let persona = serde_json::to_string(persona)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let installer = include_str!("persona.js");
    let features = subsystems.web_features().json();
    runtime
        .eval(&format!("({installer})({persona},{features})"))?
        .to_object()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BrowserSubsystemOptions {
    worker_system: bool,
    streaming_networking: bool,
    persistent_storage: Option<PersistentStorageOptions>,
    canvas: bool,
    webgl: bool,
    webgpu: bool,
    webaudio: bool,
    webaudio_output: bool,
}

impl BrowserSubsystemOptions {
    pub fn worker_system(&self) -> bool {
        self.worker_system
    }

    pub fn streaming_networking(&self) -> bool {
        self.streaming_networking
    }

    pub fn persistent_storage(&self) -> Option<&PersistentStorageOptions> {
        self.persistent_storage.as_ref()
    }

    pub fn canvas(&self) -> bool {
        self.canvas
    }

    pub fn webgl(&self) -> bool {
        self.webgl
    }

    pub fn webgpu(&self) -> bool {
        self.webgpu
    }

    pub fn webaudio(&self) -> bool {
        self.webaudio
    }

    pub fn webaudio_output(&self) -> bool {
        self.webaudio_output
    }

    fn web_features(&self) -> WebFeatureFlags {
        WebFeatureFlags {
            worker_system: self.worker_system,
            streaming_networking: self.streaming_networking,
            persistent_storage: self.persistent_storage.is_some(),
            canvas: self.canvas,
            webgl: self.webgl,
            webgpu: self.webgpu,
            webaudio: self.webaudio,
            webaudio_output: self.webaudio_output,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistentStorageOptions {
    root: PathBuf,
    quota_bytes: u64,
}

impl PersistentStorageOptions {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            quota_bytes: 1024 * 1024 * 1024,
        }
    }

    pub fn quota_bytes(mut self, quota_bytes: u64) -> Self {
        self.quota_bytes = quota_bytes;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn quota(&self) -> u64 {
        self.quota_bytes
    }
}

fn persona_request_headers(persona: &persona::ResolvedPersona) -> Vec<(String, String)> {
    let network = &persona.network;
    [
        ("user-agent", network.user_agent.as_str()),
        ("accept", network.accept_header.as_str()),
        ("accept-language", network.accept_language.as_str()),
        ("accept-encoding", network.accept_encoding.as_str()),
        ("sec-ch-ua", network.sec_ch_ua.as_str()),
        ("sec-ch-ua-mobile", network.sec_ch_ua_mobile.as_str()),
        ("sec-ch-ua-platform", network.sec_ch_ua_platform.as_str()),
        (
            "sec-ch-ua-full-version",
            network.sec_ch_ua_full_version.as_str(),
        ),
        (
            "sec-ch-ua-full-version-list",
            network.sec_ch_ua_full_version_list.as_str(),
        ),
        ("sec-ch-ua-arch", network.sec_ch_ua_arch.as_str()),
        ("sec-ch-ua-bitness", network.sec_ch_ua_bitness.as_str()),
        (
            "sec-ch-ua-platform-version",
            network.sec_ch_ua_platform_version.as_str(),
        ),
        ("sec-ch-ua-model", network.sec_ch_ua_model.as_str()),
    ]
    .into_iter()
    .filter(|(_, value)| !value.is_empty())
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .collect()
}

fn composite_canvas_rasters(
    target: &mut [u8],
    target_width: u32,
    target_height: u32,
    layers: &[(web_bindings::CanvasRaster, [f64; 4])],
    scale: f64,
) {
    for (raster, rect) in layers {
        if raster.width == 0 || raster.height == 0 || rect[2] <= 0.0 || rect[3] <= 0.0 {
            continue;
        }
        let left = (rect[0] * scale).floor() as i64;
        let top = (rect[1] * scale).floor() as i64;
        let width = (rect[2] * scale).ceil().max(1.0) as i64;
        let height = (rect[3] * scale).ceil().max(1.0) as i64;
        for destination_y in top.max(0)..(top + height).min(i64::from(target_height)) {
            let source_y = (((destination_y - top) as u64 * u64::from(raster.height))
                / height as u64)
                .min(u64::from(raster.height - 1)) as usize;
            for destination_x in left.max(0)..(left + width).min(i64::from(target_width)) {
                let source_x = (((destination_x - left) as u64 * u64::from(raster.width))
                    / width as u64)
                    .min(u64::from(raster.width - 1)) as usize;
                let source_index = (source_y * raster.width as usize + source_x) * 4;
                let target_index =
                    (destination_y as usize * target_width as usize + destination_x as usize) * 4;
                blend_pixel(
                    &mut target[target_index..target_index + 4],
                    &raster.pixels[source_index..source_index + 4],
                );
            }
        }
    }
}

fn blend_pixel(destination: &mut [u8], source: &[u8]) {
    let source_alpha = f32::from(source[3]) / 255.0;
    if source_alpha == 0.0 {
        return;
    }
    let destination_alpha = f32::from(destination[3]) / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    for channel in 0..3 {
        let value = (f32::from(source[channel]) * source_alpha
            + f32::from(destination[channel]) * destination_alpha * (1.0 - source_alpha))
            / output_alpha;
        destination[channel] = value.round().clamp(0.0, 255.0) as u8;
    }
    destination[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}
