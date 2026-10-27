//! Host session construction and managed page creation.

use super::options::validate_timeout;
use super::{NativePage, SessionOptions};
use crate::network::{CurlConfig, ResourceLoader};
use crate::runtime::{AutomationError, PageOptions, SessionCookies};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Owns pages and a cookie jar. Cloning a page does not extend the session lifetime.
pub struct Session {
    pub(in crate::runtime) id: u64,
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
    pub fn new(options: SessionOptions) -> Result<Self, AutomationError> {
        validate_timeout(options.timeout)?;
        let mut profile = options.profile.clone().unwrap_or_default();
        options.identity.apply(&mut profile)?;
        Self::create(options, Some(profile), None)
    }

    pub fn with_resource_loader(
        options: SessionOptions,
        loader: Arc<dyn ResourceLoader>,
    ) -> Result<Self, AutomationError> {
        validate_timeout(options.timeout)?;
        let mut profile = options.profile.clone().unwrap_or_default();
        options.identity.apply(&mut profile)?;
        Self::create(options, Some(profile), Some(loader))
    }

    pub fn new_page(&self) -> Result<NativePage, AutomationError> {
        let events = crate::runtime::events::EventHub::default();
        let network = Arc::new(crate::runtime::networking::NetworkRuntime::new(
            self.resource_loader(),
            events.clone(),
            self.network_executor.handle(),
            self.websocket_connector(),
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
        Ok(NativePage {
            page,
            network,
            events,
            timeout: self.options.timeout,
        })
    }

    pub fn cookies(&self) -> SessionCookies {
        self.cookie_store.clone()
    }
}
