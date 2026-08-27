use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken, NavigationResponse,
    PageOptions,
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
    page: AutomationPage,
}

#[pymethods]
impl PySession {
    #[new]
    #[pyo3(signature = (persona_json = None, ca_bundle = None))]
    fn new(
        py: Python<'_>,
        persona_json: Option<&str>,
        ca_bundle: Option<String>,
    ) -> PyResult<Self> {
        let persona = persona_json
            .map(persona::PersonaConfig::from_json)
            .transpose()
            .map_err(|persona_error| {
                error(AutomationError::InvalidInput(persona_error.to_string()))
            })?;
        py.detach(move || {
            let persona = persona.unwrap_or_default();
            let config = network::CurlConfig {
                ca_bundle: ca_bundle.map(PathBuf::from),
                ..network::CurlConfig::default()
            };
            let browser = Arc::new(
                AutomationBrowser::with_persona_and_network_config(persona, config)
                    .map_err(error)?,
            );
            let page = browser.new_page(PageOptions::default()).map_err(error)?;
            Ok(Self { browser, page })
        })
    }

    #[pyo3(signature = (url, timeout_ms, headers = None))]
    fn get(
        &self,
        py: Python<'_>,
        url: String,
        timeout_ms: u64,
        headers: Option<Vec<(String, String)>>,
    ) -> PyResult<PyResponse> {
        let page = self.page.clone();
        py.detach(move || {
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

    fn close(&self, py: Python<'_>) {
        let browser = Arc::clone(&self.browser);
        py.detach(move || browser.close());
    }
}

#[pymodule]
fn _brimp(module: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(target_os = "macos")]
    let _alignment = unsafe { std::ptr::read_volatile(&raw const NSApp) };
    module.add_class::<PySession>()?;
    module.add_class::<PyResponse>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
