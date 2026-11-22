//! Host session construction and managed page creation.

use super::{NativePage, SessionOptions};
use crate::network::{CurlConfig, ResourceLoader};
use crate::runtime::{AutomationError, PageOptions, SessionCookies};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Owns pages and a cookie jar. Cloning a page does not extend the session lifetime.
pub struct Session {
    pub(in crate::runtime) id: u64,
    pub(in crate::runtime) routes: crate::runtime::routes::Routes,
    pub(in crate::runtime) loader: Arc<dyn ResourceLoader>,
    pub(in crate::runtime) profile: Option<crate::profile::BrowserProfile>,
    pub(in crate::runtime) closed: AtomicBool,
    pub(in crate::runtime) state: std::sync::Mutex<crate::runtime::session::SessionState>,
    pub(in crate::runtime) cookie_store: SessionCookies,
    pub(in crate::runtime) storage: Arc<crate::web_apis::storage::Storage>,
    pub(in crate::runtime) network_config: Option<CurlConfig>,
    pub(in crate::runtime) options: SessionOptions,
    pub(in crate::runtime) network_executor: crate::runtime::networking::NetworkExecutor,
}

impl Session {
    pub fn new(options: SessionOptions) -> Result<Arc<Self>, AutomationError> {
        crate::runtime::routes::parse_headers(options.headers.clone())?;
        let mut profile = options.profile.clone().unwrap_or_default();
        options.identity.apply(&mut profile)?;
        Self::create(options, Some(profile), None)
    }

    pub fn with_resource_loader(
        options: SessionOptions,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Arc<Self>, AutomationError> {
        crate::runtime::routes::parse_headers(options.headers.clone())?;
        let mut profile = options.profile.clone().unwrap_or_default();
        options.identity.apply(&mut profile)?;
        Self::create(options, Some(profile), Some(loader))
    }

    pub fn new_page(self: &Arc<Self>) -> Result<NativePage, AutomationError> {
        let events = crate::runtime::events::EventHub::default();
        let network = Arc::new(crate::runtime::networking::NetworkRuntime::new(
            self.resource_loader(),
            events.clone(),
            self.network_executor.handle(),
            self.websocket_connector(),
            self.routes.child(),
        ));
        let page = self.new_page_with_loader(
            PageOptions::builder()
                .operation_control(self.options.deadline, self.options.cancellation.clone())
                .wait_until(self.options.wait_until)
                .clock(self.options.clock.clone())
                .diagnostics(self.options.diagnostics.clone())
                .events(events.clone())
                .network(network.clone())
                .request_headers(self.options.headers.clone())
                .plugins(self.options.plugins.clone())
                .plugin_timeout(self.options.timeout)
                .build(),
            network.clone(),
        )?;
        network.routes.set_page(page.id());
        let mut page = NativePage {
            page,
            network,
            events,
            timeout: self.options.timeout,
            popup_subscription: None,
        };
        let session = Arc::downgrade(self);
        let opener_id = page.id();
        page.popup_subscription = Some(Arc::new(page.events.on(
            "popup_requested",
            Arc::new(move |event| {
                let crate::runtime::events::PageEvent::PopupRequested { id, .. } = event else {
                    return;
                };
                let Some(session) = session.upgrade() else {
                    return;
                };
                std::thread::spawn(move || {
                    let result = (|| {
                        let opener = session.page(opener_id)?;
                        let context = opener.network_context()?;
                        let control = context.popup(id).ok_or(AutomationError::Closed)?;
                        if control.closed() {
                            return Ok(());
                        }
                        let popup = session.new_page()?;
                        control.attach(popup.network_context()?, popup.page.control.closed.clone());
                        let url = control.url();
                        if url != "about:blank"
                            && let Err(error) = popup.request("GET", &url, Default::default())
                        {
                            opener
                                .events
                                .emit(crate::runtime::events::PageEvent::Error {
                                    message: error.to_string(),
                                });
                        }
                        opener
                            .events
                            .emit(crate::runtime::events::PageEvent::Popup {
                                page_id: popup.id(),
                            });
                        Ok::<(), AutomationError>(())
                    })();
                    if let Err(error) = result
                        && let Ok(opener) = session.page(opener_id)
                    {
                        opener
                            .events
                            .emit(crate::runtime::events::PageEvent::Error {
                                message: error.to_string(),
                            });
                    }
                });
            }),
        )?));
        let mut state = self.state.lock().unwrap();
        if self.is_closed() {
            drop(state);
            page.close()?;
            return Err(AutomationError::Closed);
        }
        state.native_pages.push(page.clone());
        Ok(page)
    }

    pub fn pages(&self) -> Vec<NativePage> {
        let (pages, closed) = {
            let mut state = self.state.lock().unwrap();
            let (open, closed): (Vec<_>, Vec<_>) = std::mem::take(&mut state.native_pages)
                .into_iter()
                .partition(|page| !page.is_closed());
            state.native_pages = open;
            (state.native_pages.clone(), closed)
        };
        drop(closed);
        pages
    }

    pub fn page(&self, id: u64) -> Result<NativePage, AutomationError> {
        self.pages()
            .into_iter()
            .find(|page| page.id() == id)
            .ok_or(AutomationError::Closed)
    }

    pub fn route(
        &self,
        pattern: &str,
        stage: crate::runtime::routes::RouteStage,
        handler: crate::runtime::routes::RouteHandler,
    ) -> Result<crate::runtime::routes::RouteRegistration, AutomationError> {
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        self.routes.register(pattern, stage, handler)
    }

    pub fn cookies(&self) -> SessionCookies {
        self.cookie_store.clone()
    }
}
