use brimp_runtime::runtime::{
    events::Subscription,
    routes::{Route, RouteRegistration},
};
use pyo3::{prelude::*, types::PyBytes};
#[pyclass(name = "_Route")]
pub struct PyRoute {
    pub inner: Route,
}
#[pymethods]
impl PyRoute {
    fn info_json(&self) -> PyResult<String> {
        serde_json::to_string(self.inner.info())
            .map_err(|e| super::RuntimeError::new_err(e.to_string()))
    }
    #[getter]
    fn pending(&self) -> bool {
        self.inner.is_pending()
    }
    fn body<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.body().map(|body| PyBytes::new(py, &body))
    }
    fn continue_request(&self, overrides: &str) -> PyResult<()> {
        let overrides = serde_json::from_str(overrides)
            .map_err(|e| super::InvalidInput::new_err(e.to_string()))?;
        self.inner.continue_request(overrides).map_err(super::error)
    }
    fn fulfill(&self, status: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> PyResult<()> {
        self.inner
            .fulfill(status, headers, body)
            .map_err(super::error)
    }
    fn fail(&self, message: String) -> PyResult<()> {
        self.inner.fail(message).map_err(super::error)
    }
    fn authenticate(&self, username: String, password: String) -> PyResult<()> {
        self.inner
            .authenticate(username, password)
            .map_err(super::error)
    }
}
#[pyclass(name = "_Subscription")]
pub struct PySubscription {
    pub inner: Subscription,
}
#[pymethods]
impl PySubscription {
    fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }
}
#[pyclass(name = "_RouteRegistration")]
pub struct PyRouteRegistration {
    pub inner: RouteRegistration,
}
#[pymethods]
impl PyRouteRegistration {
    fn remove(&self) {
        self.inner.remove();
    }
}
