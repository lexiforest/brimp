use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    path::Path,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use blitz_traits::net::NetProvider;
use browser_dom::{BrowserDocument, HtmlParserSession, NodeId, ParseProgress};
use jsc::{JsException, JsRuntime, JsValue};
use network::{NetworkError, ResourceLoader, ResourceRequest};
use screenshot::{ScreenshotError, ScreenshotOptions};
use web_bindings::{
    BindingRuntime, BrowsingContext, FetchQueue, PendingFetch, TimerQueue, WrapperCache,
};

use crate::{TaskQueue, TaskSender, Viewport, blitz_net::BlitzResourceProvider};

pub struct Page {
    // Bindings must drop before `js`, because their objects are protected in that context.
    bindings: BindingRuntime,
    timers: Rc<RefCell<TimerQueue>>,
    browsing_context: Arc<BrowsingContext>,
    blitz_network: Arc<BlitzResourceProvider>,
    fetches: Rc<RefCell<FetchQueue>>,
    js: JsRuntime,
    document: Rc<RefCell<BrowserDocument>>,
    tasks: TaskQueue,
    viewport: Viewport,
    loader: Arc<dyn ResourceLoader>,
    load_state: LoadState,
    url: Option<String>,
    async_error: Option<JsException>,
}

impl Page {
    pub(crate) fn new(
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, JsException> {
        let js = JsRuntime::new()?;
        let browsing_context = Arc::new(BrowsingContext::default());
        browsing_context
            .set_request_identity(&options.persona.user_agent, &options.persona.locale)
            .map_err(JsException::from_message)?;
        let blitz_network = Arc::new(BlitzResourceProvider::new(
            Arc::clone(&loader),
            Arc::clone(&browsing_context),
        ));
        let net_provider: Arc<dyn NetProvider> = blitz_network.clone();
        let initial_document = BrowserDocument::empty_at_with_net(None, Some(net_provider));
        let document = Rc::new(RefCell::new(initial_document));
        let timers = Rc::new(RefCell::new(TimerQueue::default()));
        let fetches = Rc::new(RefCell::new(FetchQueue::default()));
        let bindings = BindingRuntime::install(
            &js,
            Rc::clone(&document),
            Rc::clone(&timers),
            Arc::clone(&browsing_context),
            Rc::clone(&fetches),
        )?;
        install_persona(&js, &options.persona)?;
        Ok(Self {
            bindings,
            timers,
            browsing_context,
            blitz_network,
            fetches,
            js,
            document,
            tasks: TaskQueue::default(),
            viewport: options.viewport,
            loader,
            load_state: LoadState::Idle,
            url: None,
            async_error: None,
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
        self.bindings.reset_document(&self.js)
    }

    fn reset_document_at(&mut self, base_url: &str) -> Result<(), JsException> {
        let net_provider: Arc<dyn NetProvider> = self.blitz_network.clone();
        let document = BrowserDocument::empty_at_with_net(Some(base_url), Some(net_provider));
        *self.document.borrow_mut() = document;
        self.bindings.reset_document(&self.js)
    }

    pub async fn goto(&mut self, url: &str) -> Result<(), NavigationError> {
        self.load_state = LoadState::Loading;
        let response = match self.fetch_success(url).await {
            Ok(response) => response,
            Err(error) => {
                self.load_state = LoadState::Failed;
                return Err(error);
            }
        };
        if let Some(content_type) = response.headers.get(http::header::CONTENT_TYPE) {
            let content_type = content_type
                .to_str()
                .map_err(|_| NavigationError::UnsupportedContentType("non-ASCII".to_string()))?;
            if !content_type
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/html"))
            {
                self.load_state = LoadState::Failed;
                return Err(NavigationError::UnsupportedContentType(
                    content_type.to_string(),
                ));
            }
        }
        let effective_url = response.effective_url;
        self.browsing_context.set_url(&effective_url);
        self.url = Some(effective_url.clone());
        let html = String::from_utf8(response.body)?;
        if let Err(error) = self.reset_document_at(&effective_url) {
            self.load_state = LoadState::Failed;
            return Err(error.into());
        }
        if let Err(error) = self.parse_navigation_document(&html, &effective_url).await {
            self.load_state = LoadState::Failed;
            return Err(error);
        }
        self.process_blitz_resources().await;
        self.load_state = LoadState::Complete;
        Ok(())
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

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
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
                    self.eval(&source)?;
                }
                (ScriptMode::Blocking, ScriptSource::External(src)) => {
                    self.process_blitz_resources().await;
                    let script_url = base_url.join(&src)?;
                    let source =
                        String::from_utf8(self.fetch_success(script_url.as_str()).await?.body)?;
                    self.eval(&source)?;
                }
                (mode, ScriptSource::External(src)) => {
                    if mode == ScriptMode::Defer {
                        deferred_order.push(script.order);
                    }
                    let script_url = base_url.join(&src)?.to_string();
                    let request = self.resource_request(&script_url)?;
                    let loader = Arc::clone(&self.loader);
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
                self.eval(&source)?;
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
            self.eval(&source)?;
            *next += 1;
        }
        Ok(())
    }

    async fn fetch_success(&self, url: &str) -> Result<network::ResourceResponse, NavigationError> {
        let request = self.resource_request(url)?;
        let response =
            crate::request::fetch(self.loader.as_ref(), &self.browsing_context, request).await?;
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
        self.document
            .borrow_mut()
            .set_viewport(width, height, device_pixel_ratio as f32);
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
        let result = screenshot::render_png(
            self.document.borrow_mut().blitz_mut(),
            options,
            original.device_pixel_ratio,
        );
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
        let timer_callback = self.timers.borrow_mut().pop_due();
        if let Some(callback) = timer_callback {
            self.js.call_function(&callback)?;
            self.perform_microtask_checkpoint()?;
            self.start_pending_fetches()?;
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
            let request = match self.prepare_fetch_request(pending) {
                Ok(request) => request,
                Err(error) => {
                    self.reject_fetch(id, &error)?;
                    continue;
                }
            };
            let requested_url = request.url.clone();
            let loader = Arc::clone(&self.loader);
            let browsing_context = Arc::clone(&self.browsing_context);
            let sender = self.task_sender();
            let result = crate::request::fetch_callback(
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

    fn prepare_fetch_request(&self, pending: PendingFetch) -> Result<ResourceRequest, String> {
        let url = match url::Url::parse(&pending.url) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => self
                .url
                .as_deref()
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
        request.body = pending.body.map(String::into_bytes);
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
                    "body": String::from_utf8_lossy(&response.body),
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

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error("navigation returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("navigation returned unsupported content type `{0}`")]
    UnsupportedContentType(String),
    #[error("HTML response is not UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("invalid resource URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageOptions {
    viewport: Viewport,
    persona: persona::Persona,
}

impl PageOptions {
    pub fn builder() -> PageOptionsBuilder {
        PageOptionsBuilder::default()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn persona(&self) -> &persona::Persona {
        &self.persona
    }

    pub(crate) fn with_persona(mut self, persona: persona::Persona) -> Self {
        self.viewport = Viewport {
            width: f64::from(persona.viewport.width),
            height: f64::from(persona.viewport.height),
            device_pixel_ratio: persona.viewport.device_pixel_ratio,
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
    pub fn viewport(mut self, width: u32, height: u32) -> Self {
        self.options.viewport.width = f64::from(width);
        self.options.viewport.height = f64::from(height);
        self.options.persona.viewport.width = width;
        self.options.persona.viewport.height = height;
        self
    }

    pub fn device_pixel_ratio(mut self, device_pixel_ratio: f64) -> Self {
        self.options.viewport.device_pixel_ratio = device_pixel_ratio;
        self.options.persona.viewport.device_pixel_ratio = device_pixel_ratio;
        self
    }

    pub fn build(self) -> PageOptions {
        self.options
    }
}

fn install_persona(runtime: &JsRuntime, persona: &persona::Persona) -> Result<(), JsException> {
    let user_agent = serde_json::to_string(&persona.user_agent)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let platform = serde_json::to_string(&persona.platform)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let locale = serde_json::to_string(&persona.locale)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let languages = serde_json::to_string(&persona.languages)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    runtime.eval(&format!(
        r#"
        Object.defineProperties(navigator, {{
            userAgent: {{ value: {user_agent}, enumerable: true }},
            platform: {{ value: {platform}, enumerable: true }},
            language: {{ value: {locale}, enumerable: true }},
            languages: {{ value: Object.freeze({languages}), enumerable: true }},
        }});
        globalThis.screen = Object.freeze({{
            width: {}, height: {}, availWidth: {}, availHeight: {}, colorDepth: 24, pixelDepth: 24,
        }});
        window.screen = globalThis.screen;
    "#,
        persona.viewport.width,
        persona.viewport.height,
        persona.viewport.width,
        persona.viewport.height
    ))?;
    Ok(())
}
