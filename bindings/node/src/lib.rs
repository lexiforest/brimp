use std::sync::Arc;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken as CoreCancellationToken,
    PageOptions, PersistentStorageOptions,
};

#[cfg(target_os = "macos")]
unsafe extern "C" {
    static NSApp: *mut std::ffi::c_void;
}

#[napi]
pub fn native_ready() -> bool {
    #[cfg(target_os = "macos")]
    let _alignment = unsafe { std::ptr::read_volatile(&raw const NSApp) };
    true
}

fn error(error: AutomationError) -> napi::Error {
    napi::Error::from_reason(format!("brimp {}: {error}", error.code()))
}
async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> std::result::Result<T, AutomationError> + Send + 'static,
) -> Result<T> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| napi::Error::from_reason(error.to_string()))?
        .map_err(error)
}

#[napi(js_name = "NativeCancellationToken")]
pub struct CancellationToken {
    inner: CoreCancellationToken,
}
#[napi]
impl CancellationToken {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreCancellationToken::new(),
        }
    }
    #[napi]
    pub fn cancel(&self) {
        self.inner.cancel();
    }
}
impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(js_name = "NativeBrowser")]
pub struct Browser {
    inner: Arc<AutomationBrowser>,
}

#[napi(object)]
pub struct NativePageOptions {
    pub enable_worker: Option<bool>,
    pub enable_streaming_networking: Option<bool>,
    pub enable_canvas: Option<bool>,
    pub enable_webgl: Option<bool>,
    pub enable_webgpu: Option<bool>,
    pub enable_webaudio: Option<bool>,
    pub enable_webaudio_output: Option<bool>,
    pub storage_path: Option<String>,
    pub storage_quota_bytes: Option<f64>,
}

fn page_options(options: Option<NativePageOptions>) -> Result<PageOptions> {
    let options = options.unwrap_or(NativePageOptions {
        enable_worker: None,
        enable_streaming_networking: None,
        enable_canvas: None,
        enable_webgl: None,
        enable_webgpu: None,
        enable_webaudio: None,
        enable_webaudio_output: None,
        storage_path: None,
        storage_quota_bytes: None,
    });
    let mut builder = PageOptions::builder()
        .worker_system(options.enable_worker.unwrap_or(false))
        .streaming_networking(options.enable_streaming_networking.unwrap_or(false))
        .canvas(options.enable_canvas.unwrap_or(false))
        .webgl(options.enable_webgl.unwrap_or(false))
        .webgpu(options.enable_webgpu.unwrap_or(false))
        .webaudio(options.enable_webaudio.unwrap_or(false))
        .webaudio_output(options.enable_webaudio_output.unwrap_or(false));
    if options.storage_path.is_none() && options.storage_quota_bytes.is_some() {
        return Err(error(AutomationError::InvalidInput(
            "storageQuotaBytes requires storagePath".into(),
        )));
    }
    if let Some(path) = options.storage_path {
        let quota = options
            .storage_quota_bytes
            .unwrap_or(1024.0 * 1024.0 * 1024.0);
        if !quota.is_finite() || quota <= 0.0 || quota.fract() != 0.0 || quota > u64::MAX as f64 {
            return Err(error(AutomationError::InvalidInput(
                "storageQuotaBytes must be a positive integer".into(),
            )));
        }
        builder = builder
            .persistent_storage(PersistentStorageOptions::new(path).quota_bytes(quota as u64));
    }
    Ok(builder.build())
}

#[napi]
impl Browser {
    #[napi(factory)]
    pub async fn launch(persona_json: Option<String>) -> Result<Self> {
        let persona = persona_json
            .map(|json| persona::PersonaConfig::from_json(&json))
            .transpose()
            .map_err(|persona_error| {
                error(AutomationError::InvalidInput(persona_error.to_string()))
            })?;
        blocking(move || match persona {
            Some(persona) => AutomationBrowser::with_persona(persona),
            None => AutomationBrowser::new(),
        })
        .await
        .map(|browser| Self {
            inner: Arc::new(browser),
        })
    }
    #[napi]
    pub async fn new_page(&self, options: Option<NativePageOptions>) -> Result<NativePage> {
        let browser = Arc::clone(&self.inner);
        let options = page_options(options)?;
        blocking(move || browser.new_page(options))
            .await
            .map(|inner| NativePage { inner })
    }
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let browser = Arc::clone(&self.inner);
        blocking(move || {
            browser.close();
            Ok(())
        })
        .await
    }
}

#[napi]
pub struct NativePage {
    inner: AutomationPage,
}
#[napi]
impl NativePage {
    #[napi]
    pub async fn goto(
        &self,
        url: String,
        timeout_ms: u32,
        token: &CancellationToken,
    ) -> Result<()> {
        let page = self.inner.clone();
        let token = token.inner.clone();
        blocking(move || {
            page.navigate_cancellable(url, Duration::from_millis(u64::from(timeout_ms)), token)
                .map(|_| ())
        })
        .await
    }
    #[napi]
    pub async fn evaluate(&self, expression: String) -> Result<String> {
        let page = self.inner.clone();
        blocking(move || {
            page.evaluate(expression).and_then(|value| {
                serde_json::to_string(&value)
                    .map_err(|error| AutomationError::Internal(error.to_string()))
            })
        })
        .await
    }
    #[napi]
    pub async fn title(&self) -> Result<String> {
        let page = self.inner.clone();
        blocking(move || page.title()).await
    }
    #[napi]
    pub async fn text_content(&self) -> Result<String> {
        let page = self.inner.clone();
        blocking(move || page.text_content()).await
    }
    #[napi]
    pub async fn screenshot(&self, full_page: bool) -> Result<Buffer> {
        let page = self.inner.clone();
        blocking(move || page.screenshot(full_page))
            .await
            .map(Buffer::from)
    }
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let page = self.inner.clone();
        blocking(move || {
            page.close();
            Ok(())
        })
        .await
    }
}
