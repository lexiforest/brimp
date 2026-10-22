use brimp_runtime::runtime::{
    events::{EventHandler, Subscription},
    routes::{Route, RouteHandler, RouteRegistration},
};
use napi::{
    JsFunction,
    bindgen_prelude::*,
    threadsafe_function::{
        ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
    },
};
use napi_derive::napi;
use serde_json::Value;
use std::sync::Arc;
pub fn event_handler(env: Env, callback: JsFunction) -> Result<EventHandler> {
    let mut dispatcher: ThreadsafeFunction<String, ErrorStrategy::Fatal> = callback
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<String>| Ok(vec![ctx.value]))?;
    dispatcher.unref(&env)?;
    Ok(Arc::new(move |event| {
        dispatcher.call(
            serde_json::to_string(&event).expect("event serialization"),
            ThreadsafeFunctionCallMode::NonBlocking,
        );
    }))
}
pub fn route_handler(env: Env, callback: JsFunction) -> Result<RouteHandler> {
    let mut dispatcher: ThreadsafeFunction<Route, ErrorStrategy::Fatal> = callback
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<Route>| {
            Ok(vec![NativeRoute { inner: ctx.value }])
        })?;
    dispatcher.unref(&env)?;
    Ok(Arc::new(move |route| {
        dispatcher.call(route, ThreadsafeFunctionCallMode::NonBlocking);
    }))
}
#[napi]
pub struct NativeRoute {
    inner: Route,
}
#[napi]
impl NativeRoute {
    #[napi(getter)]
    pub fn info(&self) -> Result<Value> {
        serde_json::to_value(self.inner.info()).map_err(|e| Error::from_reason(e.to_string()))
    }
    #[napi(getter)]
    pub fn pending(&self) -> bool {
        self.inner.is_pending()
    }
    #[napi]
    pub fn body(&self) -> Option<Buffer> {
        self.inner.body().map(Into::into)
    }
    #[napi]
    pub fn continue_request(&self, overrides: Value) -> Result<()> {
        let overrides =
            serde_json::from_value(overrides).map_err(|e| Error::from_reason(e.to_string()))?;
        self.inner.continue_request(overrides).map_err(super::error)
    }
    #[napi]
    pub fn fulfill(&self, status: u16, headers: Vec<Vec<String>>, body: Buffer) -> Result<()> {
        self.inner
            .fulfill(status, super::pairs(headers)?, body.to_vec())
            .map_err(super::error)
    }
    #[napi]
    pub fn fail(&self, message: String) -> Result<()> {
        self.inner.fail(message).map_err(super::error)
    }
    #[napi]
    pub fn authenticate(&self, username: String, password: String) -> Result<()> {
        self.inner
            .authenticate(username, password)
            .map_err(super::error)
    }
}
#[napi]
pub struct NativeSubscription {
    inner: Subscription,
}
impl NativeSubscription {
    pub(crate) fn new(inner: Subscription) -> Self {
        Self { inner }
    }
}
#[napi]
impl NativeSubscription {
    #[napi]
    pub fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }
}
#[napi]
pub struct NativeRouteRegistration {
    inner: RouteRegistration,
}
impl NativeRouteRegistration {
    pub(crate) fn new(inner: RouteRegistration) -> Self {
        Self { inner }
    }
}
#[napi]
impl NativeRouteRegistration {
    #[napi]
    pub fn remove(&self) {
        self.inner.remove();
    }
}
