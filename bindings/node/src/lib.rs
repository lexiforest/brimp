mod network;
mod plugins;
use brimp_runtime::runtime::native::{
    ElementHandle, JSHandle, NativePage, RequestOptions, Session, SessionOptions,
};
use brimp_runtime::runtime::{AutomationError, CancellationToken};
use napi::{Error, bindgen_prelude::*};
use napi_derive::napi;
use serde_json::Value;
use std::{sync::Arc, time::Duration};

fn error(e: AutomationError) -> Error {
    Error::new(Status::GenericFailure, format!("{}: {e}", e.code()))
}
async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> std::result::Result<T, AutomationError> + Send + 'static,
) -> Result<T> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?
        .map_err(error)
}
fn duration(ms: f64) -> Result<Duration> {
    Duration::try_from_secs_f64(ms / 1000.0)
        .ok()
        .filter(|v| !v.is_zero())
        .ok_or_else(|| Error::from_reason("timeout must be finite and positive"))
}
fn pairs(headers: Vec<Vec<String>>) -> Result<Vec<(String, String)>> {
    headers
        .into_iter()
        .map(|pair| {
            if pair.len() != 2 {
                return Err(Error::from_reason("headers require [name, value] pairs"));
            }
            Ok((pair[0].clone(), pair[1].clone()))
        })
        .collect()
}

#[napi]
#[derive(Default)]
pub struct NativeCancellation {
    inner: CancellationToken,
}
#[napi]
impl NativeCancellation {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CancellationToken::new(),
        }
    }
    #[napi]
    pub fn cancel(&self) {
        self.inner.cancel();
    }
}

#[napi]
pub struct NativeSession {
    inner: Arc<Session>,
}
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn create_native_session(
    env: Env,
    headers: Vec<Vec<String>>,
    timeout: f64,
    proxy: Option<String>,
    plugins: Vec<plugins::PluginDescriptor>,
    callback: napi::JsFunction,
    identity: Value,
    profile: Value,
    clock: Value,
    diagnostics: Value,
    storage_path: Option<String>,
) -> Result<AsyncTask<plugins::CreateSession>> {
    let mut options = SessionOptions {
        storage_path: storage_path.map(Into::into),
        clock: serde_json::from_value(clock).map_err(|e| Error::from_reason(e.to_string()))?,
        diagnostics: serde_json::from_value(diagnostics)
            .map_err(|e| Error::from_reason(e.to_string()))?,
        identity: serde_json::from_value(identity)
            .map_err(|e| Error::from_reason(e.to_string()))?,
        profile: if profile.is_null() {
            None
        } else {
            Some(serde_json::from_value(profile).map_err(|e| Error::from_reason(e.to_string()))?)
        },
        plugins: plugins::configure(env, plugins, callback)?,
        headers: pairs(headers)?,
        timeout: duration(timeout)?,
        ..Default::default()
    };
    options.network.proxy = proxy
        .map(brimp_runtime::network::Proxy::parse)
        .transpose()
        .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(AsyncTask::new(plugins::CreateSession(options)))
}
#[napi]
impl NativeSession {
    #[napi]
    pub async fn new_page(&self) -> Result<NativePageBinding> {
        let session = self.inner.clone();
        blocking(move || session.new_page().map(|inner| NativePageBinding { inner })).await
    }
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let session = self.inner.clone();
        blocking(move || {
            session.close();
            Ok(())
        })
        .await
    }
    #[napi]
    pub fn set_cookie(&self, url: String, name: String, value: String) -> Result<()> {
        self.inner
            .cookies()
            .set_cookie(&url, &name, &value)
            .map_err(error)
    }
    #[napi]
    pub fn cookies(&self) -> Result<Value> {
        serde_json::to_value(self.inner.cookies().cookies())
            .map_err(|e| Error::from_reason(e.to_string()))
    }
    #[napi]
    pub fn clear_cookies(&self) {
        self.inner.cookies().clear_cookies();
    }
}

#[napi(object)]
pub struct NativeResponse {
    pub metadata: Value,
    pub content: Buffer,
}
#[napi]
pub struct NativePageBinding {
    inner: NativePage,
}
#[napi]
impl NativePageBinding {
    #[napi]
    pub async fn advance_time(&self, milliseconds: f64, max_tasks: u32) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.advance_time(milliseconds, max_tasks as usize)).await
    }
    #[napi]
    pub async fn run_pending_tasks(&self, max_tasks: u32) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.run_pending_tasks(max_tasks as usize)).await
    }
    #[napi]
    pub async fn profile_contract(&self) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.profile_contract()).await
    }
    #[napi]
    pub async fn profile_report(&self) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.profile_report()).await
    }
    #[napi]
    pub async fn take_diagnostics(&self) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.take_diagnostics()).await
    }
    #[napi]
    pub async fn peek_diagnostics(&self) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.peek_diagnostics()).await
    }
    #[napi]
    pub async fn summarize_diagnostics(&self) -> Result<Value> {
        let page = self.inner.clone();
        blocking(move || page.summarize_diagnostics()).await
    }

    #[napi]
    pub async fn set_identity(&self, identity: Value) -> Result<()> {
        let identity =
            serde_json::from_value(identity).map_err(|e| Error::from_reason(e.to_string()))?;
        let page = self.inner.clone();
        blocking(move || page.set_identity(identity)).await
    }

    #[napi]
    pub fn on(
        &self,
        env: Env,
        event: String,
        callback: napi::JsFunction,
    ) -> Result<network::NativeSubscription> {
        self.inner
            .on(&event, network::event_handler(env, callback)?)
            .map(network::NativeSubscription::new)
            .map_err(error)
    }
    #[napi]
    pub fn route(
        &self,
        env: Env,
        pattern: String,
        stage: String,
        callback: napi::JsFunction,
    ) -> Result<network::NativeRouteRegistration> {
        let stage = serde_json::from_value(Value::String(stage))
            .map_err(|e| Error::from_reason(e.to_string()))?;
        self.inner
            .route(&pattern, stage, network::route_handler(env, callback)?)
            .map(network::NativeRouteRegistration::new)
            .map_err(error)
    }
    #[napi]
    pub async fn configure_network(
        &self,
        offline: bool,
        latency: f64,
        bypass_cache: bool,
        max_retained_bytes: u32,
        max_retained_body_bytes: u32,
        buffered_download_bytes_per_second: Option<u32>,
    ) -> Result<()> {
        let latency = Duration::try_from_secs_f64(latency / 1000.0)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let options = brimp_runtime::runtime::networking::NetworkOptions {
            offline,
            latency,
            bypass_cache,
            max_retained_bytes: max_retained_bytes as usize,
            max_retained_body_bytes: max_retained_body_bytes as usize,
            buffered_download_bytes_per_second: buffered_download_bytes_per_second
                .map(|n| n as usize),
        };
        let page = self.inner.clone();
        blocking(move || page.configure_network(options)).await
    }
    #[napi]
    pub fn set_extra_headers(&self, headers: Vec<Vec<String>>) -> Result<()> {
        self.inner.set_extra_headers(pairs(headers)?).map_err(error)
    }
    #[napi]
    pub fn clear_cache(&self) -> Result<()> {
        self.inner.clear_cache().map_err(error)
    }
    #[napi]
    pub fn response_body(&self, request_id: i64) -> Result<Buffer> {
        self.inner
            .response_body(request_id as u64)
            .map(Into::into)
            .map_err(error)
    }

    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }
    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub async fn request(
        &self,
        method: String,
        url: String,
        headers: Vec<Vec<String>>,
        body: Option<Buffer>,
        timeout: Option<f64>,
        allow_redirects: bool,
        max_redirects: u32,
        cancellation: &NativeCancellation,
    ) -> Result<NativeResponse> {
        let page = self.inner.clone();
        let options = RequestOptions {
            headers: pairs(headers)?,
            body: body.map(|v| v.to_vec()),
            timeout: timeout.map(duration).transpose()?,
            allow_redirects,
            max_redirects: max_redirects as usize,
            cancellation: cancellation.inner.clone(),
        };
        let response = blocking(move || page.request(&method, &url, options)).await?;
        let mut metadata =
            serde_json::to_value(&response).map_err(|e| Error::from_reason(e.to_string()))?;
        metadata.as_object_mut().unwrap().remove("content");
        Ok(NativeResponse {
            metadata,
            content: response.content.into(),
        })
    }
    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub async fn stream(
        &self,
        method: String,
        url: String,
        headers: Vec<Vec<String>>,
        body: Option<Buffer>,
        timeout: Option<f64>,
        allow_redirects: bool,
        max_redirects: u32,
        cancellation: &NativeCancellation,
    ) -> Result<NativeStream> {
        let options = RequestOptions {
            headers: pairs(headers)?,
            body: body.map(|b| b.to_vec()),
            timeout: timeout.map(duration).transpose()?,
            allow_redirects,
            max_redirects: max_redirects as usize,
            cancellation: cancellation.inner.clone(),
        };
        let page = self.inner.clone();
        blocking(move || {
            page.stream(&method, &url, options)
                .map(|inner| NativeStream {
                    inner: Arc::new(inner),
                })
        })
        .await
    }
    #[napi]
    pub async fn evaluate(&self, source: String) -> Result<String> {
        let page = self.inner.clone();
        blocking(move || {
            serde_json::to_string(&page.evaluate_handle(&source)?.value()?)
                .map_err(|e| AutomationError::Internal(e.to_string()))
        })
        .await
    }
    #[napi]
    pub async fn evaluate_handle(&self, source: String) -> Result<NativeHandle> {
        let page = self.inner.clone();
        blocking(move || {
            page.evaluate_handle(&source)
                .map(|inner| NativeHandle { inner })
        })
        .await
    }
    #[napi]
    pub async fn query_selector(&self, selector: String) -> Result<Option<NativeHandle>> {
        let page = self.inner.clone();
        blocking(move || {
            page.query_selector(&selector)
                .map(|v| v.map(|e| NativeHandle { inner: e.0 }))
        })
        .await
    }
    #[napi]
    pub async fn query_selector_all(&self, selector: String) -> Result<Vec<NativeHandle>> {
        let page = self.inner.clone();
        blocking(move || {
            page.query_selector_all(&selector)
                .map(|v| v.into_iter().map(|e| NativeHandle { inner: e.0 }).collect())
        })
        .await
    }
    #[napi]
    pub async fn add_init_script(&self, source: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.add_init_script(&source)).await
    }
    #[napi]
    pub async fn click(&self, selector: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.click(&selector)).await
    }
    #[napi]
    pub async fn hover(&self, selector: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.hover(&selector)).await
    }
    #[napi]
    pub async fn tap(&self, selector: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.tap(&selector)).await
    }
    #[napi]
    pub async fn focus(&self, selector: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.focus(&selector)).await
    }
    #[napi]
    pub async fn type_text(&self, selector: String, text: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.type_text(&selector, &text)).await
    }
    #[napi]
    pub async fn fill(&self, selector: String, text: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.fill(&selector, &text)).await
    }
    #[napi]
    pub async fn press(&self, selector: String, key: String, modifiers: u8) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.press(&selector, &key, modifiers)).await
    }
    #[napi]
    pub async fn drag_to(&self, source: String, destination: String) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.drag_to(&source, &destination)).await
    }
    #[napi]
    pub async fn set_input_files(&self, selector: String, files: Value) -> Result<()> {
        let files = serde_json::from_value(files).map_err(|e| Error::from_reason(e.to_string()))?;
        let page = self.inner.clone();
        blocking(move || page.set_input_files(&selector, files)).await
    }
    #[napi]
    pub async fn extract(&self, options: Value) -> Result<Value> {
        let options =
            serde_json::from_value(options).map_err(|e| Error::from_reason(e.to_string()))?;
        let page = self.inner.clone();
        let result = blocking(move || page.extract(options)).await?;
        serde_json::to_value(result).map_err(|e| Error::from_reason(e.to_string()))
    }
    #[napi]
    pub async fn html(&self) -> Result<String> {
        let page = self.inner.clone();
        blocking(move || page.html()).await
    }
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || page.close()).await
    }
}

#[napi]
pub struct NativeHandle {
    inner: JSHandle,
}
#[napi]
impl NativeHandle {
    #[napi]
    pub async fn focus(&self) -> Result<()> {
        let handle = self.inner.clone();
        blocking(move || ElementHandle(handle).focus()).await
    }
    #[napi]
    pub async fn click(&self) -> Result<()> {
        let handle = self.inner.clone();
        blocking(move || ElementHandle(handle).click()).await
    }
    #[napi]
    pub async fn set_input_files(&self, files: Value) -> Result<()> {
        let files = serde_json::from_value(files).map_err(|e| Error::from_reason(e.to_string()))?;
        let handle = self.inner.clone();
        blocking(move || ElementHandle(handle).set_input_files(files)).await
    }

    #[napi]
    pub async fn command(&self, command: String) -> Result<String> {
        let command =
            serde_json::from_str(&command).map_err(|e| Error::from_reason(e.to_string()))?;
        let handle = self.inner.clone();
        blocking(move || {
            serde_json::to_string(&handle.bridge_command(command)?)
                .map_err(|e| AutomationError::Internal(e.to_string()))
        })
        .await
    }
    #[napi]
    pub fn reference(&self) -> Result<String> {
        serde_json::to_string(&self.inner.reference().map_err(error)?)
            .map_err(|e| Error::from_reason(e.to_string()))
    }
    #[napi]
    pub fn bind_reference(&self, token: String) -> Result<NativeHandle> {
        let token = serde_json::from_str(&token).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(NativeHandle {
            inner: self.inner.peer_handle(token),
        })
    }
    #[napi]
    pub async fn query_selector(&self, selector: String) -> Result<Option<NativeHandle>> {
        let handle = self.inner.clone();
        blocking(move || {
            ElementHandle(handle)
                .query_selector(&selector)
                .map(|v| v.map(|e| NativeHandle { inner: e.0 }))
        })
        .await
    }
    #[napi]
    pub async fn query_selector_all(&self, selector: String) -> Result<Vec<NativeHandle>> {
        let handle = self.inner.clone();
        blocking(move || {
            ElementHandle(handle)
                .query_selector_all(&selector)
                .map(|v| v.into_iter().map(|e| NativeHandle { inner: e.0 }).collect())
        })
        .await
    }
    #[napi]
    pub async fn dispose(&self) -> Result<()> {
        let handle = self.inner.clone();
        blocking(move || handle.dispose()).await
    }
}

#[napi]
pub struct NativeStream {
    inner: Arc<brimp_runtime::runtime::native::streaming::StreamingResponse>,
}
#[napi]
impl NativeStream {
    #[napi(getter)]
    pub fn metadata(&self) -> Result<Value> {
        serde_json::to_value(&self.inner.metadata).map_err(|e| Error::from_reason(e.to_string()))
    }
    #[napi]
    pub async fn next_chunk(&self) -> Result<Option<Buffer>> {
        let stream = self.inner.clone();
        blocking(move || stream.body.next_chunk().map(|value| value.map(Into::into))).await
    }
    #[napi]
    pub fn cancel(&self) {
        self.inner.body.cancel();
    }
}
