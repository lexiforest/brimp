use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;

use jsc::{JsRuntime, NativeError, NativeValue};
use network::ResourceRequest;

const WORKER_BOOTSTRAP: &str = include_str!("worker_scope.js");

pub(crate) struct WorkerRealm {
    // Outputs drop before the runtime callback holding a clone of this queue.
    outputs: Rc<RefCell<Vec<String>>>,
    runtime: JsRuntime,
}

impl WorkerRealm {
    pub(crate) fn new(source: String, kind: &str) -> Result<Self, String> {
        let runtime = JsRuntime::new().map_err(|error| error.to_string())?;
        let outputs = Rc::new(RefCell::new(Vec::<String>::new()));
        let callback_outputs = Rc::clone(&outputs);
        runtime
            .set_global_function("__brimpWorkerThread", move |call| {
                let operation = call
                    .argument(0)
                    .ok_or_else(|| NativeError::new("missing worker operation"))?
                    .to_string()?;
                match operation.as_str() {
                    "post" => {
                        let message = call
                            .argument(1)
                            .ok_or_else(|| NativeError::new("missing worker message"))?
                            .to_string()?;
                        let data: serde_json::Value =
                            serde_json::from_str(&message).map_err(NativeError::new)?;
                        callback_outputs
                            .borrow_mut()
                            .push(serde_json::json!({"type": "message", "data": data}).to_string());
                    }
                    "close" => {}
                    "fetchResponse" => {
                        let response = call
                            .argument(1)
                            .ok_or_else(|| NativeError::new("missing service worker response"))?
                            .to_string()?;
                        let response: serde_json::Value =
                            serde_json::from_str(&response).map_err(NativeError::new)?;
                        callback_outputs.borrow_mut().push(
                            serde_json::json!({"type": "fetchResponse", "response": response})
                                .to_string(),
                        );
                    }
                    _ => return Err(NativeError::new("unknown worker operation")),
                }
                Ok(NativeValue::Undefined)
            })
            .map_err(|error| error.to_string())?;
        runtime
            .eval(WORKER_BOOTSTRAP)
            .map_err(|error| error.to_string())?;
        runtime
            .eval(&format!(
                "__brimpConfigureWorkerScope({})",
                serde_json::json!(kind)
            ))
            .and_then(|_| runtime.eval(&source))
            .map_err(|error| error.to_string())?;
        if kind == "shared" {
            runtime
                .eval("__brimpConnectShared()")
                .map_err(|error| error.to_string())?;
        } else if kind == "service" {
            runtime
                .eval("__brimpDispatchLifecycle('install'); __brimpDispatchLifecycle('activate')")
                .map_err(|error| error.to_string())?;
        }
        Ok(Self { outputs, runtime })
    }

    pub(crate) fn post_message(&mut self, message: &str) -> Vec<String> {
        let script = format!(
            "__brimpDispatchWorkerMessage({})",
            serde_json::json!(message)
        );
        if let Err(error) = self.runtime.eval(&script) {
            self.outputs.borrow_mut().push(
                serde_json::json!({"type": "error", "message": error.to_string()}).to_string(),
            );
        }
        self.take_outputs()
    }

    fn connect_shared(&mut self) -> Result<Vec<String>, String> {
        self.runtime
            .eval("__brimpConnectShared()")
            .map_err(|error| error.to_string())?;
        Ok(self.take_outputs())
    }

    pub(crate) fn take_outputs(&mut self) -> Vec<String> {
        self.outputs.borrow_mut().drain(..).collect()
    }

    pub(crate) fn dispatch_fetch(
        &mut self,
        request: &ResourceRequest,
    ) -> Option<ServiceWorkerResponse> {
        let headers = request
            .headers
            .iter()
            .filter_map(|(name, value)| value.to_str().ok().map(|value| [name.as_str(), value]))
            .collect::<Vec<_>>();
        let record = serde_json::json!({
            "url": request.url,
            "method": request.method.as_str(),
            "headers": headers,
            "body": request.body.as_ref().map(|body| String::from_utf8_lossy(body)),
        });
        self.runtime
            .eval(&format!(
                "__brimpDispatchFetch({})",
                serde_json::json!(record.to_string())
            ))
            .ok()?;
        self.take_outputs().into_iter().find_map(|output| {
            let envelope: serde_json::Value = serde_json::from_str(&output).ok()?;
            (envelope.get("type")?.as_str()? == "fetchResponse")
                .then(|| serde_json::from_value(envelope["response"].clone()).ok())
                .flatten()
        })
    }
}

pub(crate) struct WorkerCoordinator {
    inner: Arc<CoordinatorInner>,
}

struct CoordinatorInner {
    commands: mpsc::Sender<CoordinatorCommand>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

enum CoordinatorCommand {
    Connect {
        key: String,
        source: String,
        kind: String,
        response: mpsc::SyncSender<Result<Vec<String>, String>>,
    },
    Post {
        key: String,
        message: String,
        response: mpsc::SyncSender<Result<Vec<String>, String>>,
    },
    Fetch {
        key: String,
        request: ResourceRequest,
        response: mpsc::SyncSender<Option<ServiceWorkerResponse>>,
    },
    Remove {
        key: String,
    },
    Shutdown,
}

impl WorkerCoordinator {
    pub(crate) fn new() -> Result<Self, String> {
        let (commands, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("brimp-shared-worker".into())
            .spawn(move || run_coordinator(receiver))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            inner: Arc::new(CoordinatorInner {
                commands,
                worker: Mutex::new(Some(worker)),
            }),
        })
    }

    pub(crate) fn connect(
        &self,
        key: String,
        source: String,
        kind: String,
    ) -> Result<Vec<String>, String> {
        let (response, receiver) = mpsc::sync_channel(1);
        self.inner
            .commands
            .send(CoordinatorCommand::Connect {
                key,
                source,
                kind,
                response,
            })
            .map_err(|_| "worker coordinator is closed".to_string())?;
        receiver
            .recv()
            .map_err(|_| "worker coordinator exited".to_string())?
    }

    pub(crate) fn post(&self, key: String, message: String) -> Result<Vec<String>, String> {
        let (response, receiver) = mpsc::sync_channel(1);
        self.inner
            .commands
            .send(CoordinatorCommand::Post {
                key,
                message,
                response,
            })
            .map_err(|_| "worker coordinator is closed".to_string())?;
        receiver
            .recv()
            .map_err(|_| "worker coordinator exited".to_string())?
    }

    pub(crate) fn dispatch_fetch(
        &self,
        key: String,
        request: ResourceRequest,
    ) -> Option<ServiceWorkerResponse> {
        let (response, receiver) = mpsc::sync_channel(1);
        self.inner
            .commands
            .send(CoordinatorCommand::Fetch {
                key,
                request,
                response,
            })
            .ok()?;
        receiver.recv().ok().flatten()
    }

    pub(crate) fn remove(&self, key: String) {
        let _ = self.inner.commands.send(CoordinatorCommand::Remove { key });
    }
}

impl Clone for WorkerCoordinator {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Drop for CoordinatorInner {
    fn drop(&mut self) {
        let _ = self.commands.send(CoordinatorCommand::Shutdown);
        if let Some(worker) = self.worker.get_mut().expect("worker lock poisoned").take()
            && worker.thread().id() != std::thread::current().id()
        {
            let _ = worker.join();
        }
    }
}

fn run_coordinator(receiver: mpsc::Receiver<CoordinatorCommand>) {
    let mut realms = HashMap::<String, WorkerRealm>::new();
    while let Ok(command) = receiver.recv() {
        match command {
            CoordinatorCommand::Connect {
                key,
                source,
                kind,
                response,
            } => {
                let result = if let Some(realm) = realms.get_mut(&key) {
                    if kind == "shared" {
                        realm.connect_shared()
                    } else {
                        Ok(realm.take_outputs())
                    }
                } else {
                    WorkerRealm::new(source, &kind).map(|mut realm| {
                        let outputs = realm.take_outputs();
                        realms.insert(key, realm);
                        outputs
                    })
                };
                let _ = response.send(result);
            }
            CoordinatorCommand::Post {
                key,
                message,
                response,
            } => {
                let result = realms
                    .get_mut(&key)
                    .map(|realm| realm.post_message(&message))
                    .ok_or_else(|| "shared worker realm is unavailable".to_string());
                let _ = response.send(result);
            }
            CoordinatorCommand::Fetch {
                key,
                request,
                response,
            } => {
                let result = realms
                    .get_mut(&key)
                    .and_then(|realm| realm.dispatch_fetch(&request));
                let _ = response.send(result);
            }
            CoordinatorCommand::Remove { key } => {
                realms.remove(&key);
            }
            CoordinatorCommand::Shutdown => break,
        }
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ServiceWorkerResponse {
    pub(crate) status: u16,
    #[serde(rename = "statusText")]
    pub(crate) status_text: String,
    pub(crate) headers: Vec<[String; 2]>,
    pub(crate) body: String,
}
