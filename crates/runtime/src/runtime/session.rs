use super::native::{Session, SessionOptions};
use super::{
    AutomationError, Page, PageHandle, PageOptions, page::DocumentNetworkScope,
    page_handle::PageControl,
};
use crate::network::ResourceLoader;
use crate::web_apis::{CookieJar, StoredCookie};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub(super) struct SessionState {
    pages: Vec<Weak<PageControl>>,
    starting: Vec<Weak<AtomicBool>>,
}
#[derive(Clone, Default)]
pub struct SessionCookies {
    cookies: Arc<CookieJar>,
}

impl SessionCookies {
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
impl Session {
    pub(super) fn create(
        mut options: SessionOptions,
        profile: Option<crate::profile::BrowserProfile>,
        loader: Option<Arc<dyn ResourceLoader>>,
    ) -> Result<Self, AutomationError> {
        let network_executor = super::networking::NetworkExecutor::new()?;
        let network_config = if loader.is_none() {
            Some(options.network.clone())
        } else {
            None
        };
        let loader = match loader {
            Some(loader) => loader,
            None => {
                if let Some(profile) = &profile {
                    options.network.impersonation_profile = profile.network_profile().to_owned();
                }
                crate::network::CurlResourceLoader::check_profile(&options.network)
                    .map_err(|e| AutomationError::Transport(e.to_string()))?;
                Arc::new(
                    crate::network::CurlResourceLoader::with_dispatcher(
                        options.network.clone(),
                        network_executor.handle(),
                    )
                    .map_err(|e| AutomationError::Transport(e.to_string()))?,
                )
            }
        };
        let network_config = network_config.map(|_| options.network.clone());
        static NEXT_SESSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(Self {
            id: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
            loader,
            profile,
            closed: AtomicBool::new(false),
            state: Mutex::new(SessionState::default()),
            cookie_store: SessionCookies::default(),
            storage: Arc::new(
                crate::web_apis::storage::Storage::open(options.storage_path.clone())
                    .map_err(AutomationError::InvalidInput)?,
            ),
            network_config,
            options,
            network_executor,
        })
    }
    pub fn with_profile(profile: crate::profile::BrowserProfile) -> Result<Self, AutomationError> {
        Self::with_profile_and_network_config(profile, Default::default())
    }
    pub fn with_profile_and_network_config(
        profile: crate::profile::BrowserProfile,
        network: crate::network::CurlConfig,
    ) -> Result<Self, AutomationError> {
        Self::new(SessionOptions {
            profile: Some(profile),
            network,
            ..Default::default()
        })
    }

    pub fn new_page_with_options(
        &self,
        options: PageOptions,
    ) -> Result<PageHandle, AutomationError> {
        self.new_page_with_loader(options, Arc::clone(&self.loader))
    }

    pub fn new_page_with_proxy(
        &self,
        options: PageOptions,
        proxy: Option<&str>,
    ) -> Result<PageHandle, AutomationError> {
        let Some(proxy) = proxy else {
            return self.new_page_with_options(options);
        };
        let mut config = self.network_config.clone().ok_or_else(|| {
            AutomationError::Unsupported(
                "proxy pages require a session created with CurlConfig".into(),
            )
        })?;
        config.proxy = Some(
            crate::network::Proxy::parse(proxy)
                .map_err(|error| AutomationError::InvalidInput(error.to_string()))?,
        );
        let loader = Arc::new(
            crate::network::CurlResourceLoader::with_dispatcher(
                config.clone(),
                self.network_executor.handle(),
            )
            .map_err(|error| AutomationError::Transport(error.to_string()))?,
        );
        self.new_page_with_transports(
            options,
            loader,
            Some(Arc::new(crate::network::CurlWebSocketConnector(config))),
        )
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
        self.new_page_with_loader(options, loader)
    }
    pub(crate) fn websocket_connector(&self) -> Arc<dyn crate::network::WebSocketConnector> {
        Arc::new(crate::network::CurlWebSocketConnector(
            self.network_config.clone().unwrap_or_default(),
        ))
    }
    pub(crate) fn resource_loader(&self) -> Arc<dyn ResourceLoader> {
        self.loader.clone()
    }

    pub(crate) fn new_page_with_loader(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<PageHandle, AutomationError> {
        self.new_page_with_transports(options, loader, None)
    }
    fn new_page_with_transports(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
        sockets: Option<Arc<dyn crate::network::WebSocketConnector>>,
    ) -> Result<PageHandle, AutomationError> {
        let setup = {
            let mut state = self.state.lock().expect("session state poisoned");
            let mut setup = self.prepare_page(options, loader, &state)?;
            if let Some(sockets) = sockets {
                setup.network_scope.sockets = sockets;
            }
            state.starting.retain(|flag| flag.strong_count() > 0);
            state.starting.push(Arc::downgrade(&setup.options.shutdown));
            setup
        };
        // Plugin installation can invoke host code that accesses the session.
        let page = PageHandle::launch(setup)?;
        let mut state = self.state.lock().expect("session state poisoned");
        if self.is_closed() {
            drop(state);
            page.close();
            return Err(AutomationError::Closed);
        }
        state.starting.retain(|flag| {
            flag.upgrade()
                .is_some_and(|flag| !Arc::ptr_eq(&flag, &page.control.closed))
        });
        state.pages.push(Arc::downgrade(&page.control));
        Ok(page)
    }
    /// Creates a page on the calling thread. Its lifetime is managed by the caller.
    pub fn new_local_page(&self, options: PageOptions) -> Result<Page, AutomationError> {
        let setup = {
            let state = self.state.lock().expect("session state poisoned");
            self.prepare_page(options, Arc::clone(&self.loader), &state)?
        };
        setup.create()
    }

    fn prepare_page(
        &self,
        options: PageOptions,
        loader: Arc<dyn ResourceLoader>,
        _state: &SessionState,
    ) -> Result<PageSetup, AutomationError> {
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        let options = match &self.profile {
            Some(profile) => options.with_profile(profile.clone()),
            None => options,
        };
        let mut options = options;
        options.session_id = self.id;
        options.storage = self.storage.clone();
        let options_sockets: Arc<dyn crate::network::WebSocketConnector> = options
            .network_runtime()
            .map(|network| network as Arc<dyn crate::network::WebSocketConnector>)
            .unwrap_or_else(|| self.websocket_connector());
        Ok(PageSetup {
            options,
            network_scope: DocumentNetworkScope::new(
                loader,
                options_sockets,
                self.network_executor.handle(),
            ),
            cookies: Arc::clone(&self.cookie_store.cookies),
        })
    }

    pub fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let pages = {
            let mut state = self.state.lock().expect("session state poisoned");
            for flag in state.starting.drain(..).filter_map(|flag| flag.upgrade()) {
                flag.store(true, Ordering::Release);
            }
            std::mem::take(&mut state.pages)
        };
        for page in pages.into_iter().filter_map(|page| page.upgrade()) {
            page.close();
        }
        self.storage.close();
    }
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}
pub(super) struct PageSetup {
    options: PageOptions,
    network_scope: DocumentNetworkScope,
    cookies: Arc<CookieJar>,
}

impl PageSetup {
    pub(super) fn shutdown(&self) -> Arc<AtomicBool> {
        self.options.shutdown.clone()
    }
    pub(super) fn create(self) -> Result<Page, AutomationError> {
        let shutdown = self.options.shutdown.clone();
        Page::new(self.options, self.network_scope, self.cookies).map_err(|error| {
            if shutdown.load(Ordering::Acquire) {
                AutomationError::Closed
            } else {
                AutomationError::Internal(error.to_string())
            }
        })
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new(Default::default()).expect("session runtime must initialize")
    }
}
