use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use web_runtime::{
    AutomationBrowser, AutomationError, AutomationPage, CancellationToken, PageOptions,
};

// Keep Mach-O's LINKEDIT string table eight-byte aligned on current macOS.
#[cfg(target_os = "macos")]
unsafe extern "C" {
    static NSApp: *mut std::ffi::c_void;
}

fn error(error: AutomationError) -> PyErr {
    PyRuntimeError::new_err(format!("brimp {}: {error}", error.code()))
}

#[pyclass(name = "Browser", module = "brimp._brimp")]
struct PyBrowser {
    inner: Arc<AutomationBrowser>,
}

#[pyclass(name = "Page", module = "brimp._brimp")]
struct PyPage {
    inner: AutomationPage,
}

#[pyclass(name = "CancellationToken", module = "brimp._brimp")]
struct PyCancellationToken {
    inner: CancellationToken,
}

#[pymethods]
impl PyCancellationToken {
    #[new]
    fn new() -> Self {
        Self {
            inner: CancellationToken::new(),
        }
    }
    fn cancel(&self) {
        self.inner.cancel();
    }
}

#[pymethods]
impl PyBrowser {
    #[staticmethod]
    #[pyo3(signature = (persona_json = None))]
    fn launch(py: Python<'_>, persona_json: Option<&str>) -> PyResult<Self> {
        let persona = persona_json
            .map(persona::Persona::from_json)
            .transpose()
            .map_err(|persona_error| {
                error(AutomationError::InvalidInput(persona_error.to_string()))
            })?;
        py.detach(move || {
            let browser = match persona {
                Some(persona) => AutomationBrowser::with_persona(persona),
                None => AutomationBrowser::new(),
            }
            .map_err(error)?;
            Ok(Self {
                inner: Arc::new(browser),
            })
        })
    }
    fn new_page(&self, py: Python<'_>) -> PyResult<PyPage> {
        let browser = Arc::clone(&self.inner);
        py.detach(move || {
            browser
                .new_page(PageOptions::default())
                .map(|inner| PyPage { inner })
                .map_err(error)
        })
    }
    fn close(&self, py: Python<'_>) {
        let browser = Arc::clone(&self.inner);
        py.detach(move || browser.close());
    }
}

#[pymethods]
impl PyPage {
    fn goto(
        &self,
        py: Python<'_>,
        url: String,
        timeout_ms: u64,
        token: &PyCancellationToken,
    ) -> PyResult<()> {
        let page = self.inner.clone();
        let token = token.inner.clone();
        py.detach(move || {
            page.navigate_cancellable(url, Duration::from_millis(timeout_ms), token)
                .map_err(error)
        })
    }
    fn evaluate(&self, py: Python<'_>, expression: String) -> PyResult<String> {
        let page = self.inner.clone();
        py.detach(move || {
            page.evaluate(expression)
                .and_then(|value| {
                    serde_json::to_string(&value)
                        .map_err(|error| AutomationError::Internal(error.to_string()))
                })
                .map_err(error)
        })
    }
    fn title(&self, py: Python<'_>) -> PyResult<String> {
        let page = self.inner.clone();
        py.detach(move || page.title().map_err(error))
    }
    fn text_content(&self, py: Python<'_>) -> PyResult<String> {
        let page = self.inner.clone();
        py.detach(move || page.text_content().map_err(error))
    }
    fn screenshot<'py>(&self, py: Python<'py>, full_page: bool) -> PyResult<Bound<'py, PyBytes>> {
        let page = self.inner.clone();
        let bytes = py.detach(move || page.screenshot(full_page).map_err(error))?;
        Ok(PyBytes::new(py, &bytes))
    }
    fn close(&self, py: Python<'_>) {
        let page = self.inner.clone();
        py.detach(move || page.close());
    }
}

#[pymodule]
fn _brimp(module: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(target_os = "macos")]
    let _alignment = unsafe { std::ptr::read_volatile(&raw const NSApp) };
    module.add_class::<PyBrowser>()?;
    module.add_class::<PyPage>()?;
    module.add_class::<PyCancellationToken>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
