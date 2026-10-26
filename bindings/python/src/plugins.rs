use brimp_runtime::runtime::plugins::{BridgeValue, HostContext, HostInvocation, Plugin};
use pyo3::prelude::*;
use std::sync::{Arc, Mutex, mpsc};

enum Job {
    Host(Arc<Py<PyAny>>, Box<HostInvocation>),
    Event(Arc<Py<PyAny>>, u64, String),
    Route(Arc<Py<PyAny>>, brimp_runtime::runtime::routes::Route),
}
pub type Descriptor = (String, String, Vec<(String, Py<PyAny>)>);

/// One managed dispatcher per session; never execute Python on a page owner thread.
pub struct Dispatcher(Mutex<Option<mpsc::Sender<Job>>>);
impl Dispatcher {
    pub fn new() -> PyResult<Arc<Self>> {
        let (sender, receiver) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("brimp-python-host".into())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    let (handler, call) = match job {
                        Job::Event(handler, page_id, event) => {
                            let _guard =
                                brimp_runtime::runtime::plugins::enter_page_callback(page_id);
                            Python::attach(|py| {
                                if let Err(error) = handler.call1(py, (event,)) {
                                    error.print(py);
                                }
                            });
                            continue;
                        }
                        Job::Route(handler, route) => {
                            if !route.is_pending() {
                                continue;
                            }
                            let _guard = brimp_runtime::runtime::plugins::enter_page_callback(
                                route.page_id(),
                            );
                            let result = Python::attach(|py| -> PyResult<()> {
                                handler.call1(
                                    py,
                                    (Py::new(
                                        py,
                                        super::network::PyRoute {
                                            inner: route.clone(),
                                        },
                                    )?,),
                                )?;
                                Ok(())
                            });
                            if route.is_pending() {
                                let _ = route.fail(
                                    result.err().map(|error| error.to_string()).unwrap_or_else(
                                        || {
                                            "route handler returned without completing the route"
                                                .into()
                                        },
                                    ),
                                );
                            }
                            continue;
                        }
                        Job::Host(handler, call) => (handler, *call),
                    };
                    if call.context.is_cancelled() {
                        continue;
                    }
                    let _guard = call.context.enter_callback();
                    let result = Python::attach(|py| -> PyResult<BridgeValue> {
                        let arguments = serde_json::to_string(&call.arguments)
                            .map_err(|e| super::RuntimeError::new_err(e.to_string()))?;
                        let context = Py::new(
                            py,
                            PyHostContext {
                                inner: call.context,
                            },
                        )?;
                        let result: String =
                            handler.call1(py, (context, arguments))?.extract(py)?;
                        serde_json::from_str(&result)
                            .map_err(|e| super::InvalidInput::new_err(e.to_string()))
                    });
                    match result {
                        Ok(value) => call.completion.resolve(value),
                        Err(error) => {
                            let details = Python::attach(|py| {
                                let traceback = error
                                    .traceback(py)
                                    .and_then(|traceback| traceback.format().ok())
                                    .unwrap_or_default();
                                format!("{traceback}{error}")
                            });
                            call.completion.reject(details);
                        }
                    }
                }
            })
            .map_err(|e| super::RuntimeError::new_err(e.to_string()))?;
        Ok(Arc::new(Self(Mutex::new(Some(sender)))))
    }
    pub fn plugins(self: &Arc<Self>, descriptors: Vec<Descriptor>) -> Vec<Plugin> {
        descriptors
            .into_iter()
            .map(|(name, source, handlers)| {
                let mut plugin = Plugin::new(name, source);
                for (name, handler) in handlers {
                    let dispatcher = Arc::downgrade(self);
                    let handler = Arc::new(handler);
                    plugin.handlers.insert(
                        name,
                        Arc::new(move |call| {
                            if let Some(dispatcher) = dispatcher.upgrade() {
                                let sender =
                                    dispatcher.0.lock().expect("Python dispatcher poisoned");
                                if let Some(sender) = sender.as_ref() {
                                    if let Err(error) =
                                        sender.send(Job::Host(handler.clone(), Box::new(call)))
                                        && let Job::Host(_, call) = error.0
                                    {
                                        call.completion.reject("Python host dispatcher is closed");
                                    }
                                    return;
                                }
                            }
                            call.completion.reject("Python host dispatcher is closed");
                        }),
                    );
                }
                plugin
            })
            .collect()
    }
    pub fn event_handler(
        self: &Arc<Self>,
        handler: Py<PyAny>,
        page_id: u64,
    ) -> brimp_runtime::runtime::events::EventHandler {
        let dispatcher = Arc::downgrade(self);
        let handler = Arc::new(handler);
        Arc::new(move |event| {
            if let Some(dispatcher) = dispatcher.upgrade()
                && let Some(sender) = dispatcher.0.lock().unwrap().as_ref()
            {
                let _ = sender.send(Job::Event(
                    handler.clone(),
                    page_id,
                    serde_json::to_string(&event).expect("event serialization"),
                ));
            }
        })
    }
    pub fn route_handler(
        self: &Arc<Self>,
        handler: Py<PyAny>,
    ) -> brimp_runtime::runtime::routes::RouteHandler {
        let dispatcher = Arc::downgrade(self);
        let handler = Arc::new(handler);
        Arc::new(move |route| {
            if let Some(dispatcher) = dispatcher.upgrade()
                && let Some(sender) = dispatcher.0.lock().unwrap().as_ref()
            {
                let _ = sender.send(Job::Route(handler.clone(), route));
            }
        })
    }
    pub fn close(&self) {
        self.0.lock().expect("Python dispatcher poisoned").take();
    }
}

#[pyclass(name = "_HostContext")]
pub struct PyHostContext {
    inner: HostContext,
}
#[pymethods]
impl PyHostContext {
    #[getter]
    fn session_id(&self) -> u64 {
        self.inner.session_id
    }
    #[getter]
    fn url(&self) -> Option<String> {
        self.inner.url.clone()
    }
    #[getter]
    fn origin(&self) -> Option<String> {
        self.inner.origin.clone()
    }
    #[getter]
    fn page_id(&self) -> u64 {
        self.inner.page_id
    }
    #[getter]
    fn realm_id(&self) -> u64 {
        self.inner.realm_id
    }
    #[getter]
    fn cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }
    fn bind_reference(&self, token: &str) -> PyResult<super::PyHandle> {
        let token =
            serde_json::from_str(token).map_err(|e| super::InvalidInput::new_err(e.to_string()))?;
        Ok(super::PyHandle {
            inner: brimp_runtime::runtime::native::JSHandle::from_reference(
                self.inner.bridge.clone(),
                token,
            ),
        })
    }
    fn emit(&self, channel: String, value: &str) -> PyResult<()> {
        let value =
            serde_json::from_str(value).map_err(|e| super::InvalidInput::new_err(e.to_string()))?;
        self.inner
            .emit(channel, value)
            .map_err(|e| super::RuntimeError::new_err(e.to_string()))
    }
}
