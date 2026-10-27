//! Page task execution, timers and operation deadlines.

use super::Page;
use crate::runtime::{TaskQueue, TaskSender};
use brimp_jsc::JsException;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl Page {
    pub(crate) fn check_operation(&self) -> Result<(), crate::runtime::AutomationError> {
        if self.cancellation.is_cancelled() {
            return Err(crate::runtime::AutomationError::Cancellation);
        }
        if self.shutdown.load(Ordering::Acquire) {
            return Err(crate::runtime::AutomationError::Closed);
        }
        if self
            .operation_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(crate::runtime::AutomationError::Timeout(Duration::ZERO));
        }
        Ok(())
    }

    pub(crate) fn set_script_execution_deadline(&mut self, timeout: Duration) {
        self.script_execution_deadline = Instant::now().checked_add(timeout);
        if let Some(deadline) = self.script_execution_deadline {
            self.js
                .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        }
    }

    pub(crate) fn clear_script_execution_deadline(&mut self) {
        self.js.clear_execution_time_limit();
        self.script_execution_deadline = None;
    }

    pub fn tasks(&mut self) -> &mut TaskQueue {
        &mut self.tasks
    }

    pub fn task_sender(&self) -> TaskSender {
        self.tasks.sender()
    }

    pub(crate) fn wait_for_work(&self, maximum: Option<Duration>) {
        // Engine worker completions target the platform loop, not Rust's park
        // token. Bound the wait so engine jobs also progress without host traffic.
        let maximum = Some(
            maximum
                .unwrap_or(Duration::from_millis(10))
                .min(Duration::from_millis(10)),
        );
        let timer = self.timers.borrow().wait_duration();
        match (timer, maximum) {
            (Some(a), Some(b)) => std::thread::park_timeout(a.min(b)),
            (Some(a), None) | (None, Some(a)) => std::thread::park_timeout(a),
            (None, None) => std::thread::park(),
        }
    }

    pub(crate) fn advance_bridge_timer(&mut self) -> bool {
        self.bridge.has_pending() && self.advance_to_next_timer()
    }

    pub(crate) fn advance_to_next_timer(&mut self) -> bool {
        let mut timers = self.timers.borrow_mut();
        if !timers.clock.logical() {
            return false;
        }
        if let Some(next) = timers.next_deadline() {
            timers.clock.advance_to(next);
            true
        } else {
            false
        }
    }

    pub(crate) fn drain_tasks(
        &mut self,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        self.drive_clock(None, max_tasks)
    }

    pub(crate) fn advance_time(
        &mut self,
        milliseconds: f64,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        if !self.timers.borrow().clock.logical() {
            return Err(JsException::from_message(
                "advance_time requires logical time",
            ));
        }
        let delta = Duration::try_from_secs_f64(milliseconds / 1000.0).map_err(|_| {
            JsException::from_message("time advance must be finite and nonnegative")
        })?;
        let target = self
            .timers
            .borrow()
            .clock
            .now()
            .checked_add(delta)
            .ok_or_else(|| JsException::from_message("clock overflow"))?;
        self.drive_clock(Some(target), max_tasks)
    }

    fn drive_clock(
        &mut self,
        target: Option<Duration>,
        max_tasks: usize,
    ) -> Result<serde_json::Value, JsException> {
        if max_tasks == 0 || max_tasks > 1_000_000 {
            return Err(JsException::from_message("max_tasks must be 1..1000000"));
        }
        let mut tasks = 0;
        loop {
            if self.shutdown.load(Ordering::Acquire) {
                return Err(JsException::from_message("page closed"));
            }
            if self
                .script_execution_deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return Err(JsException::from_message("clock operation timed out"));
            }
            if tasks == max_tasks {
                return Err(JsException::from_message(
                    "task budget exhausted; clock stopped at last processed task",
                ));
            }
            if self.run_one_task()? {
                tasks += 1;
                continue;
            }
            let Some(target) = target else {
                break;
            };
            let next = self.timers.borrow().next_deadline();
            if let Some(next) = next.filter(|next| *next <= target) {
                self.timers.borrow_mut().clock.advance_to(next);
            } else {
                self.timers.borrow_mut().clock.advance_to(target);
                break;
            }
        }
        Ok(
            serde_json::json!({"now": self.timers.borrow().clock.now().as_secs_f64() * 1000.0, "tasks": tasks}),
        )
    }

    pub fn run_one_task(&mut self) -> Result<bool, JsException> {
        let deadline = self
            .script_execution_deadline
            .unwrap_or_else(|| Instant::now() + self.plugin_timeout);
        self.js
            .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        self.js.pump_jobs();
        let bridge_progress = self.bridge.pump(&self.js);
        if let Some(deadline) = self.script_execution_deadline {
            self.js
                .set_execution_time_limit(deadline.saturating_duration_since(Instant::now()));
        } else {
            self.js.clear_execution_time_limit();
        }
        let bridge_progress = bridge_progress?;
        let storage_progress = self.bindings.pump_storage(&self.js)?;
        let plugin_progress = self
            .plugin_runtime
            .as_ref()
            .map(|runtime| runtime.pump(&self.js))
            .transpose()?
            .unwrap_or(false);
        self.start_pending_fetches()?;
        self.start_pending_websockets()?;
        let timer_callback = self.timers.borrow_mut().pop_due();
        if let Some(callback) = timer_callback {
            self.js.call_function(&callback)?;
            self.start_pending_fetches()?;
            self.start_pending_websockets()?;
            return Ok(true);
        }
        let Some(task) = self.tasks.pop() else {
            return Ok(plugin_progress || bridge_progress || storage_progress);
        };
        task(self);
        if let Some(error) = self.async_error.take() {
            return Err(error);
        }
        self.start_pending_fetches()?;
        self.start_pending_websockets()?;
        Ok(true)
    }

    pub fn run_pending_tasks(&mut self) -> Result<(), JsException> {
        for _ in 0..1000 {
            if self.shutdown.load(Ordering::Acquire) {
                break;
            }
            if !self.run_one_task()? {
                break;
            }
        }
        Ok(())
    }

    pub fn run_until_idle_for(&mut self, timeout: Duration) -> Result<bool, JsException> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut progressed = false;
            while self.run_one_task()? {
                progressed = true;
            }
            if self.fetches.borrow().is_empty()
                && self.fetch_streams.borrow().is_empty()
                && self.tasks.is_empty()
            {
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
}
