//! Lite-specific configuration and diagnostics on the same CDP connection.
use super::*;
use brimp_worker_api::WorkerConfig;

impl ConnectionState {
    pub(super) fn configure_worker(&mut self, request: &Request) -> Result<Value, DispatchError> {
        if !self.pages.is_empty() || !self.browser_contexts.is_empty() || !self.sessions.is_empty()
        {
            return Err(DispatchError::invalid_request(
                "configure the worker before creating targets or contexts",
            ));
        }
        let config: WorkerConfig = serde_json::from_value(request.params.clone())
            .map_err(|e| DispatchError::invalid_params(e.to_string()))?;
        if config.storage_quota == Some(0)
            || (config.storage_quota.is_some() && config.storage_path.is_none())
        {
            return Err(DispatchError::invalid_params(
                "storage quota requires a path and must be positive",
            ));
        }
        let network = brimp_network::CurlConfig {
            proxy: config
                .proxy
                .map(brimp_network::Proxy::parse)
                .transpose()
                .map_err(|e| AutomationError::InvalidInput(e.to_string()))?,
            ca_bundle: config.ca_bundle,
            ..Default::default()
        };
        let persona = config.persona.unwrap_or_default();
        let viewport = persona.resolve().viewport;
        let browser = AutomationBrowser::with_persona_and_network_config(persona, network)?;
        let mut options = PageOptions::builder()
            .viewport(viewport.width, viewport.height)
            .device_pixel_ratio(viewport.device_scale_factor)
            .request_headers(config.headers)
            .worker_system(config.worker)
            .streaming_networking(config.streaming_networking)
            .canvas(config.canvas);
        if let Some(path) = config.storage_path {
            options = options.persistent_storage(
                brimp_runtime::PersistentStorageOptions::new(path)
                    .quota_bytes(config.storage_quota.unwrap_or(1_073_741_824)),
            );
        }
        if config.navigation_timeout_ms == Some(0) {
            return Err(DispatchError::invalid_params(
                "navigation_timeout_ms must be positive",
            ));
        }
        self.navigation_timeout =
            Duration::from_millis(config.navigation_timeout_ms.unwrap_or(30_000));
        self.browser.close();
        self.browser = Arc::new(browser);
        self.page_options = options.build();
        Ok(json!({}))
    }
    pub(super) fn worker_doctor(&self) -> Result<Value, DispatchError> {
        let page = self.browser.new_page(self.page_options.clone())?;
        page.evaluate("1 + 1")?;
        page.close();
        // Browser construction validates the persona's native transport profile.
        Ok(json!({"javascriptCore":"ok","libcurlImpersonate":"ok","worker":"lite-worker"}))
    }
    pub(super) async fn worker_wait_for_network_idle(
        &self,
        request: &Request,
    ) -> Result<Value, DispatchError> {
        let quiet = request.params["quietMs"]
            .as_u64()
            .ok_or_else(|| DispatchError::invalid_params("quietMs is required"))?;
        let timeout = request.params["timeoutMs"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or_else(|| DispatchError::invalid_params("timeoutMs must be positive"))?;
        let page = self.page_for_session(self.session(request)?)?.clone();
        tokio::task::spawn_blocking(move || {
            page.wait_for_network_idle(
                Duration::from_millis(quiet),
                Duration::from_millis(timeout),
                CancellationToken::new(),
            )
        })
        .await
        .map_err(internal_join)??;
        Ok(json!({}))
    }
}
