//! Realm-scoped JavaScript plugins using the shared value/reference host bridge.
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

use std::time::Duration;

const MAX_PENDING_CALLS: usize = 1024;
const NOTIFICATION_CAPACITY: usize = 256;

pub use super::bridge::BridgeValue;

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
    pub bridge: super::bridge::BridgeEndpoint,
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
            return Err("invalid bridge value".into());
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
pub(crate) fn is_any_callback() -> bool {
    CALLBACK_PAGE.with(|active| active.get().is_some())
}
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
            Err("invalid bridge value".into())
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
    #[allow(clippy::too_many_arguments)]
    pub fn install(
        js: &JsRuntime,
        plugins: &[Plugin],
        page_id: u64,
        realm_id: u64,
        session_id: u64,
        browsing_context: Arc<crate::web_apis::BrowsingContext>,
        events: super::events::EventHub,
        bridge: &super::bridge::BridgeRuntime,
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
        let callback_descriptors = descriptors.clone();
        let callback_browsing = browsing_context.clone();
        let endpoint = bridge.endpoint();
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
            let plugin = callback_descriptors
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
            let url = callback_browsing.current_url();
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
                    bridge: endpoint.clone(),
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
        if plugins.iter().any(|plugin| plugin.name == "canvas") {
            js.set_global_function("__brimpCanvas2D", move |call| {
                let operation = call
                    .argument(0)
                    .ok_or_else(|| NativeError::new("missing canvas operation"))?
                    .to_string()?;
                let id = call
                    .argument(1)
                    .ok_or_else(|| NativeError::new("missing canvas id"))?
                    .to_number()? as u64;
                let wire = call
                    .argument(2)
                    .ok_or_else(|| NativeError::new("missing canvas arguments"))?
                    .to_string()?;
                let values: Vec<serde_json::Value> =
                    serde_json::from_str(&wire).map_err(NativeError::new)?;
                let mut arguments: Vec<BridgeValue> =
                    values.into_iter().map(BridgeValue::from_json).collect();
                arguments.insert(0, BridgeValue::String(operation));
                arguments.insert(1, BridgeValue::Number(id as f64));
                let value =
                    crate::runtime::canvas::dispatch(&arguments).map_err(NativeError::new)?;
                Ok(match value {
                    BridgeValue::Undefined => NativeValue::Undefined,
                    BridgeValue::Null => NativeValue::Null,
                    BridgeValue::Boolean(value) => NativeValue::Boolean(value),
                    BridgeValue::Number(value) => NativeValue::Number(value),
                    BridgeValue::String(value) => NativeValue::String(value),
                    BridgeValue::Binary(value) => NativeValue::Bytes(value),
                    _ => return Err(NativeError::new("unsupported canvas result")),
                })
            })?;
        }
        let sync_state = state.clone();
        let sync_descriptors = descriptors.clone();
        let sync_context = browsing_context.clone();
        let sync_endpoint = bridge.endpoint();
        js.set_global_function("__brimpPluginHostSync", move |call| {
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
            let cancelled = sync_state.borrow().cancellation.clone();
            if cancelled.load(Ordering::Acquire) {
                return Err(NativeError::new("plugin realm is closed"));
            }
            let plugin = sync_descriptors
                .get(index)
                .ok_or_else(|| NativeError::new("unknown plugin"))?;
            let handler = plugin
                .handlers
                .get(&name)
                .ok_or_else(|| NativeError::new(format!("unregistered host handler: {name}")))?
                .clone();
            let arguments: Vec<BridgeValue> =
                serde_json::from_str(&wire).map_err(NativeError::new)?;
            let url = sync_context.current_url();
            let origin = url
                .as_deref()
                .and_then(|value| url::Url::parse(value).ok())
                .and_then(|value| match value.origin() {
                    url::Origin::Tuple(..) => Some(value.origin().ascii_serialization()),
                    url::Origin::Opaque(_) => None,
                });
            let (sender, receiver) = mpsc::sync_channel(1);
            let (notify, _notifications) = mpsc::sync_channel(1);
            let invocation = HostInvocation {
                context: HostContext {
                    session_id,
                    url,
                    origin,
                    page_id,
                    realm_id,
                    plugin_name: plugin.name.clone(),
                    plugin_index: index,
                    cancelled: cancelled.clone(),
                    notifications: notify,
                    wake: std::thread::current(),
                    bridge: sync_endpoint.clone(),
                },
                arguments,
                completion: HostCompletion {
                    wake: std::thread::current(),
                    sender: Some(sender),
                    cancelled,
                },
            };
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(invocation)));
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(Ok(value)) => Ok(NativeValue::String(
                    serde_json::to_string(&value)
                        .map_err(|error| NativeError::new(error.to_string()))?,
                )),
                Ok(Err(error)) => Err(NativeError::new(error)),
                Err(_) => Err(NativeError::new(
                    "synchronous host handler did not complete",
                )),
            }
        })?;
        let config = plugins.iter().map(|plugin| serde_json::json!({"name":plugin.name,"source":plugin.source,"handlers":plugin.handlers.keys().collect::<Vec<_>>()})).collect::<Vec<_>>();
        let source = include_str!("../../js/internal/plugins.js");
        let installer = js
            .eval(&format!(
                "(config, bridge) => ({source})(JSON.parse(config), bridge.codec)"
            ))?
            .to_object()?;
        let controller = js
            .call_function_with_string_and_object(
                &installer,
                &serde_json::to_string(&config).expect("plugin descriptors"),
                &bridge.controller,
            )?
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
    use crate::runtime::canvas;
    /// Default browser subsystems; extraction and host integrations stay explicit.
    pub fn browser() -> Vec<Plugin> {
        vec![
            environment(serde_json::json!({})),
            permissions(serde_json::json!({})),
            devices(serde_json::json!({})),
            media(serde_json::json!({})),
            workers(serde_json::json!({})),
            trusted_types(serde_json::json!({})),
            canvas(serde_json::json!({})),
            graphics(serde_json::json!({})),
            audio(serde_json::json!({})),
            rtc(),
        ]
    }
    pub fn rtc() -> Plugin {
        Plugin::new("rtc", include_str!("../../js/plugins/rtc.js"))
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
    pub fn canvas(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "canvas",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/canvas.js"),
                options
            ),
        )
        .with_handler("canvas2d", canvas::handler())
    }
    pub fn environment(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "environment",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/environment.js"),
                options
            ),
        )
    }
    pub fn permissions(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "permissions",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/permissions.js"),
                options
            ),
        )
    }
    pub fn devices(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "devices",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/devices.js"),
                options
            ),
        )
    }
    pub fn media(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "media",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/media.js"),
                options
            ),
        )
    }
    pub fn workers(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "workers",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/workers.js"),
                options
            ),
        )
    }
    pub fn trusted_types(options: serde_json::Value) -> Plugin {
        Plugin::new(
            "trusted-types",
            format!(
                "context => ({}) (context, {})",
                include_str!("../../js/plugins/trusted-types.js"),
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
