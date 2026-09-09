use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

use super::{
    AutomationError, Page, PageHandle, PageOptions, page::DocumentNetworkScope,
    page_handle::PageControl, worker::WorkerCoordinator,
};
use crate::network::ResourceLoader;
use crate::web_apis::{CookieJar, StoredCookie};

pub struct Browser {
    loader: Arc<dyn ResourceLoader>,
    persona: Option<crate::persona::PersonaConfig>,
    pages: Mutex<Vec<Weak<PageControl>>>,
    closed: AtomicBool,
    workers: WorkerCoordinator,
    default_context: BrowserContext,
    network_config: Option<crate::network::CurlConfig>,
}

#[derive(Clone, Default)]
pub struct BrowserContext {
    cookies: Arc<CookieJar>,
}

impl BrowserContext {
    pub fn set_cookie(&self, url: &str, name: &str, value: &str) -> Result<(), AutomationError> {
        self.cookies
            .set(url, name, value)
            .map_err(AutomationError::InvalidInput)
    }

    pub fn store_cookie(&self, url: &str, header: &str) -> Result<(), AutomationError> {
        self.cookies
            .store(url, header)
            .map_err(AutomationError::InvalidInput)
    }

    pub fn cookies(&self) -> Vec<StoredCookie> {
        self.cookies.all()
    }

    pub fn cookies_for_url(&self, url: &str) -> Vec<StoredCookie> {
        self.cookies.matching(url)
    }

    pub fn delete_cookies(
        &self,
        name: &str,
        url: Option<&str>,
        domain: Option<&str>,
        path: Option<&str>,
    ) {
        self.cookies.delete(name, url, domain, path);
    }

    pub fn clear_cookies(&self) {
        self.cookies.clear();
    }
}
impl Browser {
    pub fn new() -> Result<Self, AutomationError> {
        let mut browser = Self::with_persona(crate::persona::PersonaConfig::default())?;
        // PageOptions supplies defaults; only an explicitly configured browser persona overrides it.
        browser.persona = None;
        Ok(browser)
    }
    pub fn with_resource_loader(loader: Arc<dyn ResourceLoader>) -> Self {
        Self {
            loader,
            persona: None,
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().expect("shared worker coordinator must start"),
            default_context: BrowserContext::default(),
            network_config: None,
        }
    }
    pub fn with_persona_and_resource_loader(
        persona: crate::persona::PersonaConfig,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, AutomationError> {
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        Ok(Self {
            loader,
            persona: Some(persona),
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().map_err(AutomationError::Internal)?,
            default_context: BrowserContext::default(),
            network_config: None,
        })
    }
    pub fn with_persona(persona: crate::persona::PersonaConfig) -> Result<Self, AutomationError> {
        Self::with_persona_and_network_config(persona, crate::network::CurlConfig::default())
    }
    pub fn with_persona_and_network_config(
        persona: crate::persona::PersonaConfig,
        mut config: crate::network::CurlConfig,
    ) -> Result<Self, AutomationError> {
        persona
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))?;
        config.impersonation_profile = persona.network_profile.clone();
        config.default_headers = false;
        crate::network::CurlResourceLoader::check_profile(&config)
            .map_err(|error| AutomationError::Transport(error.to_string()))?;
        let loader = Arc::new(
            crate::network::CurlResourceLoader::new(config.clone())
                .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        Ok(Self {
            loader,
            persona: Some(persona),
            pages: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
            workers: WorkerCoordinator::new().map_err(AutomationError::Internal)?,
            default_context: BrowserContext::default(),
            network_config: Some(config),
        })
    }
    pub fn new_page(&self, options: PageOptions) -> Result<PageHandle, AutomationError> {
        self.new_page_in_context(options, &self.default_context)
    }
    pub fn default_context(&self) -> BrowserContext {
        self.default_context.clone()
    }
    pub fn create_context(&self) -> BrowserContext {
        BrowserContext::default()
    }
    pub fn new_page_in_context(
        &self,
        options: PageOptions,
        context: &BrowserContext,
    ) -> Result<PageHandle, AutomationError> {
        self.new_page_with_loader(options, Arc::clone(&self.loader), context)
    }
    pub fn new_page_in_context_with_proxy(
        &self,
        options: PageOptions,
        context: &BrowserContext,
        proxy: Option<&str>,
    ) -> Result<PageHandle, AutomationError> {
        let Some(proxy) = proxy else {
            return self.new_page_in_context(options, context);
        };
        let mut config = self.network_config.clone().ok_or_else(|| {
            AutomationError::Unsupported(
                "proxy pages require a browser created with CurlConfig".into(),
            )
        })?;
        config.proxy = Some(
            crate::network::Proxy::parse(proxy)
                .map_err(|error| AutomationError::InvalidInput(error.to_string()))?,
        );
        let loader = Arc::new(
            crate::network::CurlResourceLoader::new(config)
                .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        self.new_page_with_loader(options, loader, context)
    }
    pub fn new_page_with_request_interceptor(
        &self,
        options: PageOptions,
        interceptor: Arc<dyn crate::network::ResourceInterceptor>,
    ) -> Result<PageHandle, AutomationError> {
        let loader = Arc::new(crate::network::InterceptingResourceLoader::new(
            Arc::clone(&self.loader),
            interceptor,
        ));
        self.new_page_with_loader(options, loader, &self.default_context)
    }
    pub fn new_page_in_context_with_request_interceptor(
        &self,
        options: PageOptions,
        context: &BrowserContext,
        interceptor: Arc<dyn crate::network::ResourceInterceptor>,
    ) -> Result<PageHandle, AutomationError> {
        let loader = Arc::new(crate::network::InterceptingResourceLoader::new(
            Arc::clone(&self.loader),
            interceptor,
        ));
        self.new_page_with_loader(options, loader, context)
    }
    fn new_page_with_loader(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
        context: &BrowserContext,
    ) -> Result<PageHandle, AutomationError> {
        let setup = self.prepare_page(options, loader, context)?;
        let page = PageHandle::launch(setup)?;
        self.pages
            .lock()
            .expect("automation page list poisoned")
            .push(Arc::downgrade(&page.control));
        Ok(page)
    }
    /// Creates a page on the calling thread. Its lifetime is managed by the caller.
    pub fn new_local_page(&self, options: PageOptions) -> Result<Page, AutomationError> {
        self.prepare_page(options, Arc::clone(&self.loader), &self.default_context)?
            .create()
    }

    fn prepare_page(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
        context: &BrowserContext,
    ) -> Result<PageSetup, AutomationError> {
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        let options = match &self.persona {
            Some(persona) => options.with_persona(persona.clone()),
            None => options,
        };
        Ok(PageSetup {
            options,
            network_scope: DocumentNetworkScope::new(loader),
            workers: self.workers.clone(),
            cookies: Arc::clone(&context.cookies),
        })
    }

    pub fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let pages = std::mem::take(&mut *self.pages.lock().expect("automation page list poisoned"));
        for page in pages.into_iter().filter_map(|page| page.upgrade()) {
            page.close();
        }
    }
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        self.close();
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new().expect("browser runtime must initialize")
    }
}

pub(super) struct PageSetup {
    options: PageOptions,
    network_scope: DocumentNetworkScope,
    workers: WorkerCoordinator,
    cookies: Arc<CookieJar>,
}

impl PageSetup {
    pub(super) fn create(self) -> Result<Page, AutomationError> {
        Page::new(self.options, self.network_scope, self.workers, self.cookies)
            .map_err(|error| AutomationError::Internal(error.to_string()))
    }
}
