//! Typed command dispatch and promise completion on the page owner thread.

use super::JSHandle;
use crate::runtime::bridge::Reference;
use crate::runtime::diagnostics::DiagnosticsView;
use crate::runtime::{AutomationError, Page};
use brimp_jsc::ProtectedJsObject;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub(crate) type HandleToken = Reference;

pub(crate) enum Operation {
    AdvanceTime {
        milliseconds: f64,
        max_tasks: usize,
    },
    DrainTasks {
        max_tasks: usize,
    },
    ProfileContract,
    ProfileReport,
    ShapeManifest,
    Diagnostics(DiagnosticsView),
    NetworkContext,
    Input {
        target: Option<HandleToken>,
        command: Value,
    },
    Evaluate {
        source: String,
        by_value: bool,
    },
    Query {
        root: Option<HandleToken>,
        selector: String,
        all: bool,
    },
}

pub(crate) enum OperationResult {
    NetworkContext(Arc<crate::web_apis::BrowsingContext>),
    Value(Value),
    Handle(JSHandle),
    Handles(Vec<JSHandle>),
}

/// Commands execute on the owner thread; the shared bridge owns all GC roots.
fn retain_handle(page: &Page, value: &ProtectedJsObject) -> Result<JSHandle, AutomationError> {
    Ok(JSHandle::from_reference(
        page.bridge.endpoint(),
        page.bridge_retain(value)?,
    ))
}
pub(crate) fn execute_operation(
    page: &mut Page,
    operation: Operation,
    timeout: Duration,
    closed: &AtomicBool,
    interrupt: &AtomicU64,
    epoch: u64,
) -> Result<OperationResult, AutomationError> {
    if interrupt.load(Ordering::Acquire) != epoch {
        return Err(AutomationError::Cancellation);
    }
    let result = match operation {
        Operation::ProfileContract => OperationResult::Value(page.profile_contract()?),
        Operation::ProfileReport => OperationResult::Value(page.profile_report()?),
        Operation::ShapeManifest => OperationResult::Value(page.shape_manifest()),
        Operation::Diagnostics(view) => OperationResult::Value(page.diagnostics(view)),
        Operation::AdvanceTime {
            milliseconds,
            max_tasks,
        } => OperationResult::Value(page.advance_time(milliseconds, max_tasks)?),
        Operation::DrainTasks { max_tasks } => OperationResult::Value(page.drain_tasks(max_tasks)?),
        Operation::NetworkContext => OperationResult::NetworkContext(page.network_context()),
        Operation::Input { target, command } => {
            let command = command.to_string();
            match target {
                Some(token) => {
                    let target = page
                        .call_native("(_, box) => box.value", "", &page.bridge_get(&token)?)?
                        .to_object()?;
                    page.dispatch_input_object(&command, &target)?;
                }
                None => {
                    page.dispatch_input(&command)?;
                }
            }
            OperationResult::Value(Value::Null)
        }
        Operation::Evaluate { source, by_value } => {
            let source = serde_json::to_string(&source).expect("string serialization");
            let pending = page.eval(&format!(
                    "(() => {{ const box = {{ done: false }}; Promise.resolve((0, eval)({source})).then(value => {{ box.value = value; box.done = true; }}, error => {{ box.error = String(error); box.failed = true; box.done = true; }}); return box; }})()"
                ))?.to_object()?;
            wait(page, &pending, timeout, closed, interrupt, epoch)?;
            if by_value {
                OperationResult::Value(json_value(page, &pending)?)
            } else {
                OperationResult::Handle(retain_handle(page, &pending)?)
            }
        }
        Operation::Query {
            root,
            selector,
            all,
        } => {
            let document;
            let root = match root {
                Some(token) => {
                    document = page.bridge_get(&token)?;
                    &document
                }
                None => {
                    document = page.eval("({value: document})")?.to_object()?;
                    &document
                }
            };
            let nodes = page.call_native(
                    "(input, root) => { const { selector, all } = JSON.parse(input); return { value: all ? Array.from(root.value.querySelectorAll(selector)) : [root.value.querySelector(selector)].filter(Boolean) }; }",
                    &json!({ "selector": selector, "all": all }).to_string(), root,
                )?.to_object()?;
            let len = page
                .call_native("(_, nodes) => nodes.value.length", "", &nodes)?
                .to_number()? as usize;
            let mut tokens = Vec::with_capacity(len);
            for index in 0..len {
                let node = page
                    .call_native(
                        "(index, nodes) => ({value: nodes.value[Number(index)]})",
                        &index.to_string(),
                        &nodes,
                    )?
                    .to_object()?;
                tokens.push(retain_handle(page, &node)?);
            }
            OperationResult::Handles(tokens)
        }
    };
    Ok(result)
}

fn json_value(page: &Page, value: &ProtectedJsObject) -> Result<Value, AutomationError> {
    let json = page.call_native(
        "(_, box) => { const json = JSON.stringify(box.value); if (json === undefined) throw new TypeError('Value is not JSON serializable'); return json; }",
        "", value,
    )?.to_string()?;
    serde_json::from_str(&json).map_err(|error| AutomationError::Internal(error.to_string()))
}

fn wait(
    page: &mut Page,
    pending: &ProtectedJsObject,
    timeout: Duration,
    closed: &AtomicBool,
    interrupt: &AtomicU64,
    epoch: u64,
) -> Result<(), AutomationError> {
    let started = Instant::now();
    loop {
        page.check_operation()?;
        if interrupt.load(Ordering::Acquire) != epoch {
            return Err(AutomationError::Cancellation);
        }
        if closed.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        let state = page.call_native(
            "(_, box) => JSON.stringify({done: box.done, failed: box.failed, error: box.error})", "", pending,
        )?.to_string()?;
        let state: Value = serde_json::from_str(&state)
            .map_err(|error| AutomationError::Internal(error.to_string()))?;
        if state["done"] == true {
            if state["failed"] == true {
                return Err(AutomationError::JavaScript(
                    state["error"].as_str().unwrap_or("Promise rejected").into(),
                ));
            }
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(AutomationError::Timeout(timeout));
        }
        if !page.run_one_task()? && !page.advance_to_next_timer() {
            page.wait_for_work(Some(
                timeout
                    .saturating_sub(started.elapsed())
                    .min(Duration::from_millis(10)),
            ));
        }
    }
}

impl From<brimp_jsc::JsException> for AutomationError {
    fn from(error: brimp_jsc::JsException) -> Self {
        Self::JavaScript(error.to_string())
    }
}
