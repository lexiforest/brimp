//! Shared copied values and realm-bound reference operations for both host bindings.
use super::AutomationError;
use brimp_jsc::{JsException, JsRuntime, ProtectedJsObject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum BridgeValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    SpecialNumber(String),
    BigInt(String),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<BridgeValue>),
    Object(std::collections::BTreeMap<String, BridgeValue>),
    Reference(Reference),
    HostFunction(String),
}
impl BridgeValue {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Number(n) => n.is_finite(),
            Self::SpecialNumber(n) => matches!(n.as_str(), "NaN" | "Infinity" | "-Infinity" | "-0"),
            Self::BigInt(n) => {
                let digits = n.strip_prefix('-').unwrap_or(n);
                !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
            }
            Self::Array(v) => v.iter().all(Self::valid),
            Self::Object(v) => v.values().all(Self::valid),
            _ => true,
        }
    }
    fn references(&self, into: &mut Vec<Reference>) {
        match self {
            Self::Reference(token) => into.push(*token),
            Self::Array(values) => {
                for value in values {
                    value.references(into);
                }
            }
            Self::Object(values) => {
                for value in values.values() {
                    value.references(into);
                }
            }
            _ => {}
        }
    }
    pub fn from_json(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(v) => Self::Boolean(v),
            Value::Number(v) => Self::Number(v.as_f64().expect("JSON number")),
            Value::String(v) => Self::String(v),
            Value::Array(v) => Self::Array(v.into_iter().map(Self::from_json).collect()),
            Value::Object(v) => Self::Object(
                v.into_iter()
                    .map(|(k, v)| (k, Self::from_json(v)))
                    .collect(),
            ),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub realm: u64,
    pub id: u64,
}

// A result may race the caller's timeout. Keep new reference roots until the
// caller acknowledges receipt; reclaim them if ownership was never transferred.
const WAITING: u8 = 0;
const RECEIVED: u8 = 1;
const ABANDONED: u8 = 2;
struct Request {
    command: Value,
    response: mpsc::SyncSender<Result<BridgeValue, AutomationError>>,
    delivery: Arc<AtomicU8>,
    deadline: Instant,
}
#[derive(Clone)]
pub struct BridgeEndpoint {
    sender: mpsc::Sender<Request>,
    alive: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    wake: std::thread::Thread,
    realm: u64,
    timeout: Duration,
}
impl BridgeEndpoint {
    pub fn request(&self, mut command: Value) -> Result<BridgeValue, AutomationError> {
        self.check()?;
        if command["token"]["realm"] != self.realm {
            return Err(AutomationError::StaleHandle);
        }
        command["callback"] = json!(
            command["callback"].as_bool().unwrap_or(false) || super::plugins::is_any_callback()
        );
        let (response, receiver) = mpsc::sync_channel(1);
        let delivery = Arc::new(AtomicU8::new(WAITING));
        self.sender
            .send(Request {
                command,
                response,
                delivery: delivery.clone(),
                deadline: Instant::now() + self.timeout,
            })
            .map_err(|_| AutomationError::StaleHandle)?;
        self.wake.unpark();
        let result = receiver
            .recv_timeout(self.timeout)
            .map_err(|_| AutomationError::Timeout(self.timeout))
            .and_then(|v| v);
        delivery.store(
            if result.is_ok() { RECEIVED } else { ABANDONED },
            Ordering::Release,
        );
        self.wake.unpark();
        self.check()?;
        result
    }
    pub fn check(&self) -> Result<(), AutomationError> {
        if self.shutdown.load(Ordering::Acquire) {
            return Err(AutomationError::Closed);
        }
        if !self.alive.load(Ordering::Acquire) {
            return Err(AutomationError::StaleHandle);
        }
        Ok(())
    }
    pub fn release(&self, token: &Reference) {
        if self.check().is_err() {
            return;
        }
        let (response, _) = mpsc::sync_channel(1);
        let _ = self.sender.send(Request {
            command: json!({"op":"dispose","token":token}),
            response,
            delivery: Arc::new(AtomicU8::new(RECEIVED)),
            deadline: Instant::now() + self.timeout,
        });
        self.wake.unpark();
    }
}
struct Transfer {
    delivery: Arc<AtomicU8>,
    references: Vec<Reference>,
}
struct Pending {
    request: Request,
    result: ProtectedJsObject,
}
pub(crate) struct BridgeRuntime {
    pub controller: ProtectedJsObject,
    endpoint: BridgeEndpoint,
    receiver: mpsc::Receiver<Request>,
    pending: Vec<Pending>,
    transfers: Vec<Transfer>,
}
impl BridgeRuntime {
    pub fn new(
        js: &JsRuntime,
        realm: u64,
        shutdown: Arc<AtomicBool>,
        timeout: Duration,
    ) -> Result<Self, JsException> {
        let controller = js
            .eval(&format!(
                "({})({:?})",
                include_str!("../../js/runtime/bridge.js"),
                realm
            ))?
            .to_object()?;
        let (sender, receiver) = mpsc::channel();
        Ok(Self {
            controller,
            receiver,
            pending: Vec::new(),
            transfers: Vec::new(),
            endpoint: BridgeEndpoint {
                sender,
                alive: Arc::new(AtomicBool::new(true)),
                shutdown,
                wake: std::thread::current(),
                realm,
                timeout,
            },
        })
    }
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub fn endpoint(&self) -> BridgeEndpoint {
        self.endpoint.clone()
    }
    pub fn retain(
        &self,
        js: &JsRuntime,
        value: &ProtectedJsObject,
    ) -> Result<Reference, AutomationError> {
        let wire = js
            .call_function_with_string_and_object(&self.controller, r#"{"op":"retain"}"#, value)?
            .to_string()?;
        serde_json::from_str(&wire).map_err(|e| AutomationError::Internal(e.to_string()))
    }
    pub fn get(
        &self,
        js: &JsRuntime,
        token: &Reference,
    ) -> Result<ProtectedJsObject, AutomationError> {
        if token.realm != self.endpoint.realm {
            return Err(AutomationError::StaleHandle);
        }
        js.call_function_with_string(
            &self.controller,
            &json!({"op":"get","token":token}).to_string(),
        )
        .and_then(|v| v.to_object())
        .map_err(bridge_error)
    }
    pub fn pump(&mut self, js: &JsRuntime) -> Result<bool, JsException> {
        let mut progress = false;
        let mut transfer_index = 0;
        while transfer_index < self.transfers.len() {
            match self.transfers[transfer_index]
                .delivery
                .load(Ordering::Acquire)
            {
                WAITING => transfer_index += 1,
                state => {
                    let transfer = self.transfers.swap_remove(transfer_index);
                    if state == ABANDONED {
                        for token in transfer.references {
                            js.call_function_with_string(
                                &self.controller,
                                &json!({"op":"dispose","token":token}).to_string(),
                            )?;
                        }
                    }
                }
            }
        }
        for _ in 0..64 {
            let Ok(request) = self.receiver.try_recv() else {
                break;
            };
            if request.delivery.load(Ordering::Acquire) == ABANDONED {
                continue;
            }
            progress = true;
            let result = js
                .call_function_with_string(&self.controller, &request.command.to_string())
                .and_then(|v| v.to_object());
            match result {
                Ok(result) => self.pending.push(Pending { request, result }),
                Err(e) => {
                    let _ = request.response.send(Err(bridge_error(e)));
                }
            }
        }
        if self.pending.is_empty() {
            return Ok(progress);
        }
        let poll = js.eval("(cancel, box) => { if (cancel === 'true') box.cancel?.(); return JSON.stringify(box); }")?.to_object()?;
        let mut index = 0;
        while index < self.pending.len() {
            let pending = &self.pending[index];
            let cancelled = pending.request.delivery.load(Ordering::Acquire) == ABANDONED
                || Instant::now() >= pending.request.deadline;
            let state = js
                .call_function_with_string_and_object(
                    &poll,
                    if cancelled { "true" } else { "false" },
                    &pending.result,
                )?
                .to_string()?;
            let state: Value = serde_json::from_str(&state)
                .map_err(|e| JsException::from_message(e.to_string()))?;
            let result = if cancelled {
                Some(Err(AutomationError::Timeout(self.endpoint.timeout)))
            } else if state["done"] == true {
                Some(if let Some(error) = state["error"].as_str() {
                    Err(message_error(error))
                } else {
                    serde_json::from_value::<BridgeValue>(state["wire"].clone())
                        .map_err(|e| AutomationError::Internal(e.to_string()))
                })
            } else {
                None
            };
            if let Some(result) = result {
                let pending = self.pending.swap_remove(index);
                if let Ok(value) = &result {
                    let mut references = Vec::new();
                    value.references(&mut references);
                    if !references.is_empty() {
                        self.transfers.push(Transfer {
                            delivery: pending.request.delivery.clone(),
                            references,
                        });
                    }
                }
                if pending.request.response.send(result).is_err() {
                    pending.request.delivery.store(ABANDONED, Ordering::Release);
                }
                progress = true;
            } else {
                index += 1;
            }
        }
        Ok(progress)
    }
}
fn bridge_error(error: JsException) -> AutomationError {
    message_error(&error.to_string())
}
fn message_error(error: &str) -> AutomationError {
    if error.contains("stale_handle:") {
        AutomationError::StaleHandle
    } else if error.contains("reentry:") {
        AutomationError::Reentry
    } else {
        AutomationError::JavaScript(error.into())
    }
}
impl Drop for BridgeRuntime {
    fn drop(&mut self) {
        self.endpoint.alive.store(false, Ordering::Release);
        for pending in self.pending.drain(..) {
            let _ = pending
                .request
                .response
                .send(Err(AutomationError::StaleHandle));
        }
        while let Ok(request) = self.receiver.try_recv() {
            let _ = request.response.send(Err(AutomationError::StaleHandle));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_out_callback_operation_does_not_block_subsequent_host_calls() {
        let js = JsRuntime::new().unwrap();
        let mut bridge = BridgeRuntime::new(
            &js,
            1,
            Arc::new(AtomicBool::new(false)),
            Duration::from_millis(30),
        )
        .unwrap();
        let value = js
            .eval("({value: () => new Promise(() => {})})")
            .unwrap()
            .to_object()
            .unwrap();
        let token = bridge.retain(&js, &value).unwrap();
        let endpoint = bridge.endpoint();
        let caller = std::thread::spawn(move || {
            endpoint.request(json!({"op":"invoke","token":token,"callback":true,"handle":true}))
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !caller.is_finished() {
            bridge.pump(&js).unwrap();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(matches!(
            caller.join().unwrap(),
            Err(AutomationError::Timeout(_))
        ));
        bridge.pump(&js).unwrap();
        let check = js
            .eval("(_, bridge) => { bridge.codec.checkCall(); return true; }")
            .unwrap()
            .to_object()
            .unwrap();
        assert_eq!(
            js.call_function_with_string_and_object(&check, "", &bridge.controller)
                .unwrap()
                .to_string()
                .unwrap(),
            "true"
        );
    }

    #[test]
    fn discarded_reference_results_are_released() {
        let js = JsRuntime::new().unwrap();
        let mut bridge = BridgeRuntime::new(
            &js,
            7,
            Arc::new(AtomicBool::new(false)),
            Duration::from_secs(1),
        )
        .unwrap();
        let value = js
            .eval("({value: () => ({answer:42})})")
            .unwrap()
            .to_object()
            .unwrap();
        let token = bridge.retain(&js, &value).unwrap();
        let delivery = Arc::new(AtomicU8::new(WAITING));
        let (response, receiver) = mpsc::sync_channel(1);
        bridge
            .endpoint
            .sender
            .send(Request {
                command: json!({"op":"invoke","token":token,"handle":true}),
                response,
                delivery: delivery.clone(),
                deadline: Instant::now() + Duration::from_secs(1),
            })
            .unwrap();
        bridge.pump(&js).unwrap();
        let BridgeValue::Reference(result) = receiver.recv().unwrap().unwrap() else {
            panic!("reference expected")
        };
        // Model the timeout/receipt race: the host never takes ownership.
        delivery.store(ABANDONED, Ordering::Release);
        bridge.pump(&js).unwrap();
        assert!(matches!(
            bridge.get(&js, &result),
            Err(AutomationError::StaleHandle)
        ));
        assert!(bridge.get(&js, &token).is_ok());
    }
}
