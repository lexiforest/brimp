//! Realm-scoped JavaScript plugins and a copied-value, asynchronous host bridge.
use brimp_jsc::{
    JsException, JsRuntime, NativeError, NativeValue, PromiseSettlement, ProtectedJsObject,
};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap},
    fmt,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

const MAX_PENDING_CALLS: usize = 1024;
const NOTIFICATION_CAPACITY: usize = 256;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum BridgeValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<BridgeValue>),
    Object(BTreeMap<String, BridgeValue>),
}
impl BridgeValue {
    fn valid(&self) -> bool {
        match self {
            Self::Number(value) => value.is_finite(),
            Self::Array(values) => values.iter().all(Self::valid),
            Self::Object(values) => values.values().all(Self::valid),
            _ => true,
        }
    }
}

/// The adapter schedules the invocation on its managed dispatcher/event loop.
/// It must return promptly; all values in the invocation are owned and Send.
pub type HostHandler = Arc<dyn Fn(HostInvocation) + Send + Sync>;
#[derive(Clone)]
pub struct Plugin {
    pub name: String,
    pub source: String,
    pub handlers: BTreeMap<String, HostHandler>,
}
impl Plugin {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            handlers: BTreeMap::new(),
        }
    }
    pub fn with_handler(mut self, name: impl Into<String>, handler: HostHandler) -> Self {
        self.handlers.insert(name.into(), handler);
        self
    }
}
impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plugin")
            .field("name", &self.name)
            .field("handlers", &self.handlers.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct HostContext {
    pub session_id: u64,
    /// Captured from the trusted browsing context, never from page JavaScript.
    pub url: Option<String>,
    /// None for opaque origins. Do not use a shared "null" storage partition.
    pub origin: Option<String>,
    pub page_id: u64,
    pub realm_id: u64,
    pub plugin_name: String,
    plugin_index: usize,
    cancelled: Arc<AtomicBool>,
    notifications: mpsc::SyncSender<Notification>,
    wake: std::thread::Thread,
}
impl HostContext {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    /// Delivery is ordered and bounded. A full queue returns an error immediately.
    pub fn emit(&self, channel: impl Into<String>, value: BridgeValue) -> Result<(), String> {
        if self.is_cancelled() {
            return Err("plugin realm is closed".into());
        }
        if !value.valid() {
            return Err("host values must contain finite numbers".into());
        }
        self.notifications
            .try_send(Notification {
                plugin: self.plugin_index,
                channel: channel.into(),
                value,
            })
            .map_err(|error| error.to_string())?;
        self.wake.unpark();
        Ok(())
    }
    pub fn enter_callback(&self) -> CallbackGuard {
        enter_page_callback(self.page_id)
    }
}
thread_local! { static CALLBACK_PAGE: Cell<Option<u64>> = const { Cell::new(None) }; }
pub(crate) fn is_callback_page(page: u64) -> bool {
    CALLBACK_PAGE.with(|active| active.get() == Some(page))
}
pub fn enter_page_callback(page_id: u64) -> CallbackGuard {
    let previous = CALLBACK_PAGE.with(|active| active.replace(Some(page_id)));
    CallbackGuard { previous }
}
pub struct CallbackGuard {
    previous: Option<u64>,
}
impl Drop for CallbackGuard {
    fn drop(&mut self) {
        CALLBACK_PAGE.with(|active| active.set(self.previous));
    }
}

pub struct HostInvocation {
    pub context: HostContext,
    pub arguments: Vec<BridgeValue>,
    pub completion: HostCompletion,
}
pub struct HostCompletion {
    wake: std::thread::Thread,
    sender: Option<mpsc::SyncSender<Result<BridgeValue, String>>>,
    cancelled: Arc<AtomicBool>,
}
impl HostCompletion {
    pub fn resolve(mut self, value: BridgeValue) {
        let result = if value.valid() {
            Ok(value)
        } else {
            Err("host values must contain finite numbers".into())
        };
        self.finish(result);
    }
    pub fn reject(mut self, error: impl Into<String>) {
        self.finish(Err(error.into()));
    }
    fn finish(&mut self, result: Result<BridgeValue, String>) {
        if let Some(sender) = self.sender.take()
            && !self.cancelled.load(Ordering::Acquire)
        {
            let _ = sender.try_send(result);
            self.wake.unpark();
        }
    }
}
impl Drop for HostCompletion {
    fn drop(&mut self) {
        self.finish(Err(
            "host handler returned without completing the call".into()
        ));
    }
}
#[derive(serde::Serialize)]
struct Notification {
    plugin: usize,
    channel: String,
    value: BridgeValue,
}
struct Pending {
    operation: String,
    receiver: mpsc::Receiver<Result<BridgeValue, String>>,
    settlement: PromiseSettlement,
}
struct State {
    pending: HashMap<u64, Pending>,
    next: u64,
    cancellation: Arc<AtomicBool>,
}

pub(crate) struct PluginRuntime {
    state: Rc<RefCell<State>>,
    cancelled: Arc<AtomicBool>,
    notifications: mpsc::Receiver<Notification>,
    controller: ProtectedJsObject,
    events: super::events::EventHub,
    last_host_error: RefCell<Option<String>>,
}
impl PluginRuntime {
    pub fn install(
        js: &JsRuntime,
        plugins: &[Plugin],
        page_id: u64,
        realm_id: u64,
        session_id: u64,
        browsing_context: Arc<crate::web_apis::BrowsingContext>,
        events: super::events::EventHub,
    ) -> Result<Self, JsException> {
        let mut names = std::collections::HashSet::new();
        for plugin in plugins {
            if plugin.name.is_empty() || !names.insert(&plugin.name) {
                return Err(JsException::from_message(
                    "plugin names must be nonempty and unique in a session",
                ));
            }
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let state = Rc::new(RefCell::new(State {
            pending: HashMap::new(),
            next: 1,
            cancellation: cancelled.clone(),
        }));
        let (notify, notifications) = mpsc::sync_channel(NOTIFICATION_CAPACITY);
        let callback_state = state.clone();
        let descriptors = plugins.to_vec();
        js.set_global_function("__brimpPluginHost", move |call| {
            let index = call
                .argument(0)
                .ok_or_else(|| NativeError::new("missing plugin"))?
                .to_number()? as usize;
            let name = call
                .argument(1)
                .ok_or_else(|| NativeError::new("missing handler"))?
                .to_string()?;
            let wire = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing arguments"))?
                .to_string()?;
            let callback_cancelled = callback_state.borrow().cancellation.clone();
            if callback_cancelled.load(Ordering::Acquire) {
                return Err(NativeError::new("plugin realm is closed"));
            }
            let plugin = descriptors
                .get(index)
                .ok_or_else(|| NativeError::new("unknown plugin"))?;
            let handler = plugin
                .handlers
                .get(&name)
                .ok_or_else(|| NativeError::new(format!("unregistered host handler: {name}")))?
                .clone();
            let arguments: Vec<BridgeValue> =
                serde_json::from_str(&wire).map_err(NativeError::new)?;
            let (promise, settlement) = call.make_deferred_promise()?.into_parts();
            let (sender, receiver) = mpsc::sync_channel(1);
            {
                let mut state = callback_state.borrow_mut();
                if state.pending.len() >= MAX_PENDING_CALLS {
                    return Err(NativeError::new("too many pending host calls"));
                }
                let id = state.next;
                state.next += 1;
                state.pending.insert(
                    id,
                    Pending {
                        operation: format!("Plugin {} host {name}", plugin.name),
                        receiver,
                        settlement,
                    },
                );
            }
            let url = browsing_context.current_url();
            let origin = url
                .as_deref()
                .and_then(|value| url::Url::parse(value).ok())
                .and_then(|value| match value.origin() {
                    url::Origin::Tuple(..) => Some(value.origin().ascii_serialization()),
                    url::Origin::Opaque(_) => None,
                });
            let invocation = HostInvocation {
                context: HostContext {
                    session_id,
                    url,
                    origin,
                    page_id,
                    realm_id,
                    plugin_name: plugin.name.clone(),
                    plugin_index: index,
                    cancelled: callback_cancelled.clone(),
                    notifications: notify.clone(),
                    wake: std::thread::current(),
                },
                arguments,
                completion: HostCompletion {
                    wake: std::thread::current(),
                    sender: Some(sender),
                    cancelled: callback_cancelled.clone(),
                },
            };
            // A panicking adapter drops its completion, rejecting this call. No
            // foreign-language object or raw JSC value is sent to the adapter.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(invocation)));
            Ok(NativeValue::ProtectedObject(promise))
        })?;
        let config = plugins.iter().map(|plugin| serde_json::json!({"name":plugin.name,"source":plugin.source,"handlers":plugin.handlers.keys().collect::<Vec<_>>()})).collect::<Vec<_>>();
        let source = include_str!("../../js/runtime/plugins.js");
        let controller = js
            .eval(&format!(
                "({source})({})",
                serde_json::to_string(&config).expect("plugin descriptors")
            ))?
            .to_object()?;
        Ok(Self {
            state,
            cancelled,
            notifications,
            controller,
            events,
            last_host_error: RefCell::new(None),
        })
    }
    fn control(
        &self,
        js: &JsRuntime,
        command: serde_json::Value,
    ) -> Result<serde_json::Value, JsException> {
        let result = js
            .call_function_with_string(&self.controller, &command.to_string())?
            .to_string()?;
        serde_json::from_str(&result).map_err(|error| JsException::from_message(error.to_string()))
    }
    pub fn ready(&self, js: &JsRuntime) -> Result<bool, JsException> {
        let state = self.control(js, serde_json::json!({"op":"status"}))?;
        if let Some(error) = state["error"].as_str() {
            let details = self.last_host_error.borrow();
            return Err(JsException::from_message(match details.as_ref() {
                Some(details) => format!("{error}\n{details}"),
                None => error.to_owned(),
            }));
        }
        Ok(state["ready"] == true)
    }
    pub fn pump(&self, js: &JsRuntime) -> Result<bool, JsException> {
        let completions: Vec<_> = {
            let state = self.state.borrow();
            state
                .pending
                .iter()
                .filter_map(|(id, pending)| match pending.receiver.try_recv() {
                    Ok(result) => Some((*id, result)),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some((*id, Err("host handler disconnected".into())))
                    }
                })
                .collect()
        };
        let mut progressed = !completions.is_empty();
        for (id, result) in completions {
            let pending = self.state.borrow_mut().pending.remove(&id);
            if let Some(pending) = pending {
                match result {
                    Ok(value) => pending.settlement.resolve(
                        js,
                        &serde_json::to_string(&value).expect("finite bridge value"),
                    )?,
                    Err(error) => {
                        let detail = format!("{}: {error}", pending.operation);
                        self.events.emit(super::events::PageEvent::Error {
                            message: detail.clone(),
                        });
                        *self.last_host_error.borrow_mut() = Some(detail);
                        pending.settlement.reject(js, "The operation failed.")?;
                    }
                }
            }
        }
        while let Ok(notification) = self.notifications.try_recv() {
            progressed = true;
            self.control(
                js,
                serde_json::json!({"op":"notify","notification":notification}),
            )?;
        }
        Ok(progressed)
    }
    pub fn cancel(&self, js: &JsRuntime) -> Result<(), JsException> {
        if self.cancelled.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let pending: Vec<_> = {
            let mut state = self.state.borrow_mut();
            state.cancellation = Arc::new(AtomicBool::new(false));
            state.pending.drain().map(|(_, value)| value).collect()
        };
        for call in pending {
            call.settlement.reject(js, "plugin realm was closed")?;
        }
        self.control(js, serde_json::json!({"op":"dispose"}))?;
        Ok(())
    }
    pub fn disposed(&self, js: &JsRuntime) -> Result<bool, JsException> {
        Ok(self.control(js, serde_json::json!({"op":"status"}))?["disposed"] == true)
    }
}
impl Drop for PluginRuntime {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        let mut state = self.state.borrow_mut();
        state.cancellation.store(true, Ordering::Release);
        state.pending.clear();
    }
}

/// Explicit bundled installers. Calling these functions does not enable a plugin globally.
pub mod bundled {
    use super::Plugin;
    pub fn geometry() -> Plugin {
        Plugin::new("geometry", include_str!("../../js/plugins/geometry.js"))
    }
    pub fn css_typed_om() -> Plugin {
        Plugin::new(
            "css-typed-om",
            include_str!("../../js/plugins/css-typed-om.js"),
        )
    }
    pub fn scheduling() -> Plugin {
        Plugin::new("scheduling", include_str!("../../js/plugins/scheduling.js"))
    }
    pub fn graphics(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "graphics",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/graphics.js"),
                options
            ),
        )
    }
    pub fn audio(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "audio",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/audio.js"),
                options
            ),
        )
    }
    pub fn storage() -> Plugin {
        Plugin::new("storage", include_str!("../../js/plugins/storage.js"))
    }
    pub fn canvas(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "canvas",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/canvas.js"),
                options
            ),
        )
    }
    pub fn services(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "services",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/services.js"),
                options
            ),
        )
    }
    pub fn extraction() -> Plugin {
        Plugin::new(
            "defuddle",
            format!(
                "context => {{ {}; return ({}) (context); }}",
                include_str!("../../../../assets/defuddle/0.19.3/index.full.js"),
                include_str!("../../js/plugins/extraction.js")
            ),
        )
    }
}
