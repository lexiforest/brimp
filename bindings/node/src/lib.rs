use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken as CoreCancellationToken,
    ExtractionOptions, NavigationResponse, PageOptions, PersistentStorageOptions,
};

#[napi]
pub fn native_ready() -> bool {
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

#[napi(object)]
pub struct NativeSessionOptions {
    pub persona_json: Option<String>,
    pub ca_bundle: Option<String>,
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

fn page_options(options: &NativeSessionOptions) -> Result<PageOptions> {
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
    if let Some(path) = options.storage_path.as_ref() {
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

fn pairs(values: Vec<(String, String)>) -> Vec<Vec<String>> {
    values
        .into_iter()
        .map(|(name, value)| vec![name, value])
        .collect()
}

#[napi(object)]
pub struct NativeResponse {
    pub status_code: u32,
    pub reason: String,
    pub url: String,
    pub headers: Vec<Vec<String>>,
    pub content: Buffer,
    pub html: Option<String>,
    pub cookies: Vec<Vec<String>>,
    pub elapsed: f64,
    pub http_version: Option<String>,
    pub downloaded_bytes: f64,
    pub uploaded_bytes: f64,
    pub header_bytes: f64,
    pub request: NativeRequest,
    pub history: Vec<NativeRedirect>,
}

#[napi(object)]
pub struct NativeRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<Vec<String>>,
    pub body: Option<Buffer>,
}

#[napi(object)]
pub struct NativeRedirect {
    pub status_code: u32,
    pub reason: String,
    pub url: String,
    pub headers: Vec<Vec<String>>,
    pub request: NativeRequest,
}

impl From<NavigationResponse> for NativeResponse {
    fn from(response: NavigationResponse) -> Self {
        Self {
            status_code: u32::from(response.status_code),
            reason: response.reason,
            url: response.url,
            headers: pairs(response.headers),
            content: Buffer::from(response.content),
            html: response.html,
            cookies: pairs(response.cookies),
            elapsed: response.elapsed.as_secs_f64(),
            http_version: response.http_version,
            downloaded_bytes: response.downloaded_bytes as f64,
            uploaded_bytes: response.uploaded_bytes as f64,
            header_bytes: response.header_bytes as f64,
            request: NativeRequest {
                method: response.request.method,
                url: response.request.url,
                headers: pairs(response.request.headers),
                body: response.request.body.map(Buffer::from),
            },
            history: response
                .history
                .into_iter()
                .map(|entry| NativeRedirect {
                    status_code: u32::from(entry.status_code),
                    reason: entry.reason,
                    url: entry.url,
                    headers: pairs(entry.headers),
                    request: NativeRequest {
                        method: entry.request.method,
                        url: entry.request.url,
                        headers: pairs(entry.request.headers),
                        body: entry.request.body.map(Buffer::from),
                    },
                })
                .collect(),
        }
    }
}

#[napi(js_name = "NativeSession")]
pub struct Session {
    browser: Arc<AutomationBrowser>,
    page_options: PageOptions,
}

#[napi]
impl Session {
    #[napi(factory)]
    pub async fn create(options: NativeSessionOptions) -> Result<Self> {
        let persona = options
            .persona_json
            .as_deref()
            .map(persona::PersonaConfig::from_json)
            .transpose()
            .map_err(|persona_error| {
                error(AutomationError::InvalidInput(persona_error.to_string()))
            })?
            .unwrap_or_default();
        let page_options = page_options(&options)?;
        let config = network::CurlConfig {
            ca_bundle: options.ca_bundle.map(PathBuf::from),
            ..network::CurlConfig::default()
        };
        blocking(move || {
            let browser = Arc::new(AutomationBrowser::with_persona_and_network_config(
                persona, config,
            )?);
            Ok(Self {
                browser,
                page_options,
            })
        })
        .await
    }

    #[napi]
    pub async fn new_page(&self, proxy: Option<String>) -> Result<NativePage> {
        let browser = Arc::clone(&self.browser);
        let page_options = self.page_options.clone();
        blocking(move || {
            let context = browser.default_context();
            let page =
                browser.new_page_in_context_with_proxy(page_options, &context, proxy.as_deref())?;
            Ok(NativePage { browser, page })
        })
        .await
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        let browser = Arc::clone(&self.browser);
        blocking(move || {
            browser.close();
            Ok(())
        })
        .await
    }
}

#[napi(js_name = "NativePage")]
pub struct NativePage {
    browser: Arc<AutomationBrowser>,
    page: AutomationPage,
}

#[napi]
impl NativePage {
    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub async fn request(
        &self,
        method: String,
        url: String,
        timeout_ms: u32,
        token: &CancellationToken,
        headers: Vec<Vec<String>>,
        cookies: Vec<Vec<String>>,
        body: Option<Buffer>,
        allow_redirects: bool,
        max_redirects: u32,
    ) -> Result<NativeResponse> {
        let mut request_headers = Vec::with_capacity(headers.len());
        for entry in headers {
            if entry.len() != 2 {
                return Err(error(AutomationError::InvalidInput(
                    "headers must contain name/value pairs".into(),
                )));
            }
            request_headers.push((entry[0].clone(), entry[1].clone()));
        }
        let mut cookie_pairs = Vec::with_capacity(cookies.len());
        for entry in cookies {
            if entry.len() != 2 {
                return Err(error(AutomationError::InvalidInput(
                    "cookies must contain name/value pairs".into(),
                )));
            }
            cookie_pairs.push((entry[0].clone(), entry[1].clone()));
        }
        let page = self.page.clone();
        let context = self.browser.default_context();
        let token = token.inner.clone();
        blocking(move || {
            for (name, value) in cookie_pairs {
                context.set_cookie(&url, &name, &value)?;
            }
            page.navigate_request(
                method,
                url,
                Duration::from_millis(u64::from(timeout_ms)),
                token,
                request_headers,
                body.map(|body| body.to_vec()),
                allow_redirects,
                max_redirects as usize,
            )
            .map(NativeResponse::from)
        })
        .await
    }

    #[napi]
    pub async fn html(&self) -> Result<String> {
        let page = self.page.clone();
        blocking(move || page.html()).await
    }

    #[napi]
    pub async fn evaluate(&self, expression: String) -> Result<String> {
        let page = self.page.clone();
        blocking(move || {
            page.evaluate(expression).and_then(|value| {
                serde_json::to_string(&value)
                    .map_err(|error| AutomationError::Internal(error.to_string()))
            })
        })
        .await
    }

    #[napi]
    pub async fn screenshot(&self, full_page: bool) -> Result<Buffer> {
        let page = self.page.clone();
        blocking(move || page.screenshot(full_page))
            .await
            .map(Buffer::from)
    }

    #[napi]
    pub async fn extract(&self, options_json: String) -> Result<String> {
        let options =
            serde_json::from_str::<ExtractionOptions>(&options_json).map_err(|json_error| {
                error(AutomationError::InvalidInput(format!(
                    "invalid extraction options: {json_error}"
                )))
            })?;
        let page = self.page.clone();
        blocking(move || {
            page.extract(options).and_then(|document| {
                serde_json::to_string(&document)
                    .map_err(|error| AutomationError::Internal(error.to_string()))
            })
        })
        .await
    }

    #[napi]
    pub async fn click(&self, selector: String) -> Result<()> {
        let page = self.page.clone();
        blocking(move || page.click(selector)).await
    }

    #[napi]
    pub async fn hover(&self, selector: String) -> Result<()> {
        let page = self.page.clone();
        blocking(move || page.hover(selector)).await
    }

    #[napi]
    pub async fn type_text(&self, selector: String, text: String) -> Result<()> {
        let page = self.page.clone();
        blocking(move || page.type_text(selector, text)).await
    }

    #[napi]
    pub async fn tap(&self, selector: String) -> Result<()> {
        let page = self.page.clone();
        blocking(move || page.tap(selector)).await
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        let page = self.page.clone();
        blocking(move || {
            page.close();
            Ok(())
        })
        .await
    }
}
