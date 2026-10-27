//! Page configuration and its builder.

use super::NavigationWait;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct PageOptions {
    pub(crate) session_id: u64,
    pub(crate) storage: Arc<crate::web_apis::storage::Storage>,
    pub(in crate::runtime) operation_deadline: Option<Instant>,
    pub(in crate::runtime) cancellation: crate::runtime::CancellationToken,
    pub(super) wait_until: NavigationWait,
    pub(super) clock: crate::runtime::clock::ClockOptions,
    pub(super) diagnostics: Option<crate::runtime::diagnostics::DiagnosticsOptions>,
    pub(super) events: crate::runtime::events::EventHub,
    pub(super) network: Option<Arc<crate::runtime::networking::NetworkRuntime>>,
    pub(super) plugins: Vec<crate::runtime::plugins::Plugin>,
    pub(super) plugin_timeout: Duration,
    pub(crate) shutdown: Arc<AtomicBool>,
    pub(super) profile: crate::profile::BrowserProfile,
    pub(super) request_headers: Vec<(String, String)>,
}

impl Default for PageOptions {
    fn default() -> Self {
        let profile = crate::profile::BrowserProfile::default();
        Self {
            session_id: 0,
            storage: Arc::default(),
            operation_deadline: None,
            cancellation: Default::default(),
            wait_until: Default::default(),
            clock: Default::default(),
            diagnostics: None,
            events: crate::runtime::events::EventHub::default(),
            network: None,
            plugins: crate::runtime::plugins::bundled::browser(),
            plugin_timeout: Duration::from_secs(30),
            shutdown: Arc::new(AtomicBool::new(false)),
            profile,
            request_headers: Vec::new(),
        }
    }
}

impl PageOptions {
    pub(crate) fn network_runtime(
        &self,
    ) -> Option<Arc<crate::runtime::networking::NetworkRuntime>> {
        self.network.clone()
    }
    pub fn builder() -> PageOptionsBuilder {
        PageOptionsBuilder::default()
    }

    pub fn profile(&self) -> &crate::profile::BrowserProfile {
        &self.profile
    }

    pub fn request_headers(&self) -> &[(String, String)] {
        &self.request_headers
    }

    pub(crate) fn with_profile(mut self, profile: crate::profile::BrowserProfile) -> Self {
        self.profile = profile;
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
        cancellation: crate::runtime::CancellationToken,
    ) -> Self {
        self.options.operation_deadline = deadline;
        self.options.cancellation = cancellation;
        self
    }
    pub fn wait_until(mut self, wait_until: NavigationWait) -> Self {
        self.options.wait_until = wait_until;
        self
    }
    pub fn clock(mut self, options: crate::runtime::clock::ClockOptions) -> Self {
        self.options.clock = options;
        self
    }
    pub fn diagnostics(
        mut self,
        options: Option<crate::runtime::diagnostics::DiagnosticsOptions>,
    ) -> Self {
        self.options.diagnostics = options;
        self
    }

    pub(crate) fn events(mut self, events: crate::runtime::events::EventHub) -> Self {
        self.options.events = events;
        self
    }
    pub(crate) fn network(
        mut self,
        network: Arc<crate::runtime::networking::NetworkRuntime>,
    ) -> Self {
        self.options.network = Some(network);
        self
    }
    pub fn plugins(mut self, plugins: Vec<crate::runtime::plugins::Plugin>) -> Self {
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
