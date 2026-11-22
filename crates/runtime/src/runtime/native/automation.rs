//! Owner-thread waits and locator actions. Resolution and input are atomic.
use crate::runtime::{AutomationError, Page};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub(super) fn run(
    page: &mut Page,
    command: Value,
    timeout: Duration,
    closed: &AtomicBool,
    interrupt: &AtomicU64,
    epoch: u64,
) -> Result<super::OperationResult, AutomationError> {
    let started = Instant::now();
    let state = page.eval("({})")?.to_object()?;
    let input = command.to_string();
    loop {
        page.check_operation()?;
        if closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        if interrupt.load(Ordering::Acquire) != epoch {
            return Err(AutomationError::Cancellation);
        }
        if !timeout.is_zero() && started.elapsed() >= timeout {
            return Err(AutomationError::Timeout(timeout));
        }
        let result = page
            .call_native(
                include_str!("../../../js/internal/automation.js"),
                &input,
                &state,
            )?
            .to_object()?;
        let status = page
            .call_native(
                "(_, result) => JSON.stringify({done: !!result.done, error: result.error})",
                "",
                &result,
            )?
            .to_string()?;
        let status: Value =
            serde_json::from_str(&status).map_err(|e| AutomationError::Internal(e.to_string()))?;
        if let Some(error) = status["error"].as_str() {
            return Err(AutomationError::InvalidInput(error.into()));
        }
        if status["done"] == true {
            if command["op"] == "input" {
                let target = page
                    .call_native("(_, result) => result.target", "", &result)?
                    .to_object()?;
                page.dispatch_input_object(&command["input"].to_string(), &target)?;
            }
            return Ok(super::OperationResult::Copied(page.bridge_copy(&result)?));
        }
        if !page.run_one_task()? && !page.advance_to_next_timer() {
            page.pump_engine_jobs();
            page.wait_for_work(Some(Duration::from_millis(10)));
        }
    }
}
