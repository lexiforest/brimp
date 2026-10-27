//! Page plugin installation and disposal.

use super::Page;
use brimp_jsc::JsException;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl Page {
    pub(super) fn initialize_plugins(&mut self) -> Result<(), JsException> {
        if self.plugins.is_empty() {
            return Ok(());
        }
        self.js.set_execution_time_limit(self.plugin_timeout);
        let result = (|| {
            self.plugin_runtime = Some(crate::runtime::plugins::PluginRuntime::install(
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

    pub(super) fn close_plugins(&mut self) {
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
}
