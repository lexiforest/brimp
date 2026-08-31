use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken, ExtractionOptions,
    NavigationResponse, PageOptions, PersistentStorageOptions,
};

// Keep Mach-O's LINKEDIT string table eight-byte aligned on current macOS.
#[cfg(target_os = "macos")]
unsafe extern "C" {
    static NSApp: *mut std::ffi::c_void;
}

fn error(error: AutomationError) -> PyErr {
    PyRuntimeError::new_err(format!("brimp {}: {error}", error.code()))
}

#[pyclass(name = "_Response", module = "brimp._brimp")]
struct PyResponse {
    response: NavigationResponse,
}

#[pymethods]
impl PyResponse {
    #[getter]
    fn status_code(&self) -> u16 {
        self.response.status_code
    }

    #[getter]
    fn reason(&self) -> &str {
        &self.response.reason
    }

    #[getter]
    fn url(&self) -> &str {
        &self.response.url
    }

    #[getter]
    fn headers(&self) -> Vec<(String, String)> {
        self.response.headers.clone()
    }

    #[getter]
    fn content<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.response.content)
    }

    #[getter]
    fn html(&self) -> Option<&str> {
        self.response.html.as_deref()
    }

    #[getter]
    fn cookies(&self) -> Vec<(String, String)> {
        self.response.cookies.clone()
    }

    #[getter]
    fn elapsed(&self) -> f64 {
        self.response.elapsed.as_secs_f64()
    }
}

#[pyclass(name = "_Session", module = "brimp._brimp")]
struct PySession {
    browser: Arc<AutomationBrowser>,
    page_options: PageOptions,
}

#[pymethods]
impl PySession {
    #[new]
    #[pyo3(signature = (persona_json = None, ca_bundle = None, enable_worker = false, enable_streaming_networking = false, enable_canvas = false, enable_webgl = false, enable_webgpu = false, enable_webaudio = false, enable_webaudio_output = false, storage_path = None, storage_quota_bytes = None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        persona_json: Option<&str>,
        ca_bundle: Option<String>,
        enable_worker: bool,
        enable_streaming_networking: bool,
        enable_canvas: bool,
        enable_webgl: bool,
        enable_webgpu: bool,
        enable_webaudio: bool,
        enable_webaudio_output: bool,
        storage_path: Option<String>,
        storage_quota_bytes: Option<u64>,
    ) -> PyResult<Self> {
        let persona = persona_json
            .map(persona::PersonaConfig::from_json)
            .transpose()
            .map_err(|persona_error| {
                error(AutomationError::InvalidInput(persona_error.to_string()))
            })?;
        py.detach(move || {
            if storage_quota_bytes == Some(0) {
                return Err(error(AutomationError::InvalidInput(
                    "storage_quota_bytes must be positive".into(),
                )));
            }
            if storage_path.is_none() && storage_quota_bytes.is_some() {
                return Err(error(AutomationError::InvalidInput(
                    "storage_quota_bytes requires storage_path".into(),
                )));
            }
            let persona = persona.unwrap_or_default();
            let config = network::CurlConfig {
                ca_bundle: ca_bundle.map(PathBuf::from),
                ..network::CurlConfig::default()
            };
            let browser = Arc::new(
                AutomationBrowser::with_persona_and_network_config(persona, config)
                    .map_err(error)?,
            );
            let mut page_options = PageOptions::builder()
                .worker_system(enable_worker)
                .streaming_networking(enable_streaming_networking)
                .canvas(enable_canvas)
                .webgl(enable_webgl)
                .webgpu(enable_webgpu)
                .webaudio(enable_webaudio)
                .webaudio_output(enable_webaudio_output);
            if let Some(path) = storage_path {
                let storage = PersistentStorageOptions::new(path)
                    .quota_bytes(storage_quota_bytes.unwrap_or(1024 * 1024 * 1024));
                page_options = page_options.persistent_storage(storage);
            }
            Ok(Self {
                browser,
                page_options: page_options.build(),
            })
        })
    }

    #[pyo3(signature = (proxy = None))]
    fn new_page(&self, py: Python<'_>, proxy: Option<String>) -> PyResult<PyPage> {
        let browser = Arc::clone(&self.browser);
        let page_options = self.page_options.clone();
        py.detach(move || {
            let context = browser.default_context();
            let page = browser
                .new_page_in_context_with_proxy(page_options, &context, proxy.as_deref())
                .map_err(error)?;
            Ok(PyPage { browser, page })
        })
    }

    fn close(&self, py: Python<'_>) {
        let browser = Arc::clone(&self.browser);
        py.detach(move || browser.close());
    }
}

#[pyclass(name = "_Page", module = "brimp._brimp")]
struct PyPage {
    browser: Arc<AutomationBrowser>,
    page: AutomationPage,
}

#[pymethods]
impl PyPage {
    #[pyo3(signature = (url, timeout_ms, headers = None, cookies = None))]
    fn get(
        &self,
        py: Python<'_>,
        url: String,
        timeout_ms: u64,
        headers: Option<Vec<(String, String)>>,
        cookies: Option<Vec<(String, String)>>,
    ) -> PyResult<PyResponse> {
        let page = self.page.clone();
        let context = self.browser.default_context();
        py.detach(move || {
            for (name, value) in cookies.unwrap_or_default() {
                context.set_cookie(&url, &name, &value).map_err(error)?;
            }
            page.navigate_with_headers(
                url,
                Duration::from_millis(timeout_ms),
                CancellationToken::new(),
                headers.unwrap_or_default(),
            )
            .map(|response| PyResponse { response })
            .map_err(error)
        })
    }

    fn evaluate(&self, py: Python<'_>, expression: String) -> PyResult<String> {
        let page = self.page.clone();
        py.detach(move || {
            page.evaluate(expression)
                .and_then(|value| {
                    serde_json::to_string(&value)
                        .map_err(|error| AutomationError::Internal(error.to_string()))
                })
                .map_err(error)
        })
    }

    fn screenshot<'py>(&self, py: Python<'py>, full_page: bool) -> PyResult<Bound<'py, PyBytes>> {
        let page = self.page.clone();
        let bytes = py.detach(move || page.screenshot(full_page).map_err(error))?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn extract(&self, py: Python<'_>, options_json: String) -> PyResult<String> {
        let options =
            serde_json::from_str::<ExtractionOptions>(&options_json).map_err(|json_error| {
                error(AutomationError::InvalidInput(format!(
                    "invalid extraction options: {json_error}"
                )))
            })?;
        let page = self.page.clone();
        py.detach(move || {
            page.extract(options)
                .and_then(|document| {
                    serde_json::to_string(&document)
                        .map_err(|error| AutomationError::Internal(error.to_string()))
                })
                .map_err(error)
        })
    }

    fn click(&self, py: Python<'_>, selector: String) -> PyResult<()> {
        let page = self.page.clone();
        py.detach(move || page.click(selector).map_err(error))
    }

    fn hover(&self, py: Python<'_>, selector: String) -> PyResult<()> {
        let page = self.page.clone();
        py.detach(move || page.hover(selector).map_err(error))
    }

    fn type_text(&self, py: Python<'_>, selector: String, text: String) -> PyResult<()> {
        let page = self.page.clone();
        py.detach(move || page.type_text(selector, text).map_err(error))
    }

    fn tap(&self, py: Python<'_>, selector: String) -> PyResult<()> {
        let page = self.page.clone();
        py.detach(move || page.tap(selector).map_err(error))
    }

    fn close(&self, py: Python<'_>) {
        let page = self.page.clone();
        py.detach(move || page.close());
    }
}

#[pymodule]
fn _brimp(module: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(target_os = "macos")]
    let _alignment = unsafe { std::ptr::read_volatile(&raw const NSApp) };
    module.add_class::<PySession>()?;
    module.add_class::<PyPage>()?;
    module.add_class::<PyResponse>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
