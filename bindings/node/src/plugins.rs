use brimp_runtime::runtime::{
    native::{Session, SessionOptions},
    plugins::{HostCompletion, HostContext, HostInvocation, Plugin},
};
use napi::{
    JsFunction, Task,
    bindgen_prelude::*,
    threadsafe_function::{
        ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
    },
};
use napi_derive::napi;
use std::sync::{Arc, Mutex};

#[napi(object)]
pub struct PluginDescriptor {
    pub name: String,
    pub source: String,
    pub handlers: Vec<String>,
}

pub fn configure(
    env: Env,
    descriptors: Vec<PluginDescriptor>,
    callback: JsFunction,
) -> Result<Vec<Plugin>> {
    let mut dispatcher: ThreadsafeFunction<(String, HostInvocation), ErrorStrategy::Fatal> =
        callback.create_threadsafe_function(
            0,
            |ctx: ThreadSafeCallContext<(String, HostInvocation)>| {
                let (name, call) = ctx.value;
                let arguments = serde_json::to_string(&call.arguments)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                Ok(vec![NativeHostCall {
                    context: call.context,
                    completion: Mutex::new(Some(call.completion)),
                    name,
                    arguments,
                }])
            },
        )?;
    dispatcher.unref(&env)?;
    Ok(descriptors
        .into_iter()
        .map(|descriptor| {
            let mut plugin = Plugin::new(descriptor.name, descriptor.source);
            for name in descriptor.handlers {
                let dispatcher = dispatcher.clone();
                let handler_name = name.clone();
                plugin.handlers.insert(
                    name,
                    Arc::new(move |call| {
                        dispatcher.call(
                            (handler_name.clone(), call),
                            ThreadsafeFunctionCallMode::NonBlocking,
                        );
                    }),
                );
            }
            plugin
        })
        .collect())
}

#[napi]
pub struct NativeHostCall {
    context: HostContext,
    completion: Mutex<Option<HostCompletion>>,
    name: String,
    arguments: String,
}
#[napi]
impl NativeHostCall {
    #[napi(getter)]
    pub fn session_id(&self) -> String {
        self.context.session_id.to_string()
    }
    #[napi(getter)]
    pub fn url(&self) -> Option<String> {
        self.context.url.clone()
    }
    #[napi(getter)]
    pub fn origin(&self) -> Option<String> {
        self.context.origin.clone()
    }
    #[napi(getter)]
    pub fn page_id(&self) -> String {
        self.context.page_id.to_string()
    }
    #[napi(getter)]
    pub fn realm_id(&self) -> String {
        self.context.realm_id.to_string()
    }
    #[napi(getter)]
    pub fn plugin_name(&self) -> String {
        self.context.plugin_name.clone()
    }
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[napi(getter)]
    pub fn arguments(&self) -> String {
        self.arguments.clone()
    }
    #[napi(getter)]
    pub fn cancelled(&self) -> bool {
        self.context.is_cancelled()
    }
    #[napi]
    pub fn resolve(&self, value: String) -> Result<()> {
        let value = serde_json::from_str(&value).map_err(|e| Error::from_reason(e.to_string()))?;
        self.take()?.resolve(value);
        Ok(())
    }
    #[napi]
    pub fn reject(&self, message: String) -> Result<()> {
        self.take()?.reject(message);
        Ok(())
    }
    #[napi]
    pub fn bind_reference(&self, token: String) -> Result<super::NativeHandle> {
        let token = serde_json::from_str(&token).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(super::NativeHandle {
            inner: brimp_runtime::runtime::native::JSHandle::from_reference(
                self.context.bridge.clone(),
                token,
            ),
        })
    }
    #[napi]
    pub fn emit(&self, channel: String, value: String) -> Result<()> {
        let value = serde_json::from_str(&value).map_err(|e| Error::from_reason(e.to_string()))?;
        self.context
            .emit(channel, value)
            .map_err(Error::from_reason)
    }
}
impl NativeHostCall {
    fn take(&self) -> Result<HostCompletion> {
        self.completion
            .lock()
            .expect("host completion poisoned")
            .take()
            .ok_or_else(|| Error::from_reason("host call already completed"))
    }
}

pub struct CreateSession(pub SessionOptions);
impl Task for CreateSession {
    type Output = Session;
    type JsValue = super::NativeSession;
    fn compute(&mut self) -> Result<Session> {
        Session::new(self.0.clone()).map_err(super::error)
    }
    fn resolve(&mut self, _env: Env, session: Session) -> Result<Self::JsValue> {
        Ok(super::NativeSession {
            inner: Arc::new(session),
        })
    }
}
