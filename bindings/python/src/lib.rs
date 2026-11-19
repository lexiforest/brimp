mod network;
mod plugins;
use brimp_runtime::runtime::native::{
    ElementHandle, JSHandle, NativePage, RequestOptions, Session, SessionOptions,
};
use brimp_runtime::runtime::{AutomationError, CancellationToken, NavigationResponse};
use pyo3::{prelude::*, types::PyBytes};
use std::sync::Arc;
use std::time::Duration;

mod errors;
use errors::*;

fn duration(seconds: f64) -> PyResult<Duration> {
    Duration::try_from_secs_f64(seconds)
        .ok()
        .filter(|duration| std::time::Instant::now().checked_add(*duration).is_some())
        .ok_or_else(|| {
            InvalidInput::new_err("timeout must be finite, nonnegative and representable")
        })
}

/// Run one Python wrapper call with cancellation propagated to native commands.
#[pyclass(name = "_Cancellation")]
struct PyCancellation {
    inner: CancellationToken,
}
#[pymethods]
impl PyCancellation {
    #[new]
    fn new() -> Self {
        Self {
            inner: CancellationToken::new(),
        }
    }
    fn cancel(&self) {
        self.inner.cancel();
    }
    fn run(&self, function: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let _scope = self.inner.enter();
        if self.inner.is_cancelled() {
            return Err(RuntimeError::new_err("operation cancelled"));
        }
        Ok(function.call0()?.unbind())
    }
}

#[pyclass(name = "_Session")]
struct PySession {
    inner: Arc<Session>,
    dispatcher: Arc<plugins::Dispatcher>,
}
#[pymethods]
impl PySession {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (headers, timeout=30.0, plugins=Vec::new(), identity="{}", profile="null", clock="{}", diagnostics="null", storage_path=None, proxy_options="{}"))]
    fn new(
        py: Python<'_>,
        headers: Vec<(String, String)>,
        timeout: f64,
        plugins: Vec<plugins::Descriptor>,
        identity: &str,
        profile: &str,
        clock: &str,
        diagnostics: &str,
        storage_path: Option<String>,
        proxy_options: &str,
    ) -> PyResult<Self> {
        let dispatcher = plugins::Dispatcher::new()?;
        let parsed_profile: Option<brimp_runtime::profile::BrowserProfile> =
            serde_json::from_str(profile).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        let mut options = SessionOptions {
            storage_path: storage_path.map(Into::into),
            clock: serde_json::from_str(clock).map_err(|e| InvalidInput::new_err(e.to_string()))?,
            diagnostics: serde_json::from_str(diagnostics)
                .map_err(|e| InvalidInput::new_err(e.to_string()))?,
            identity: serde_json::from_str(identity)
                .map_err(|e| InvalidInput::new_err(e.to_string()))?,
            profile: parsed_profile,
            plugins: dispatcher.plugins(plugins),
            headers,
            timeout: duration(timeout)?,
            ..Default::default()
        };
        // Python timeout formats normalize to one runtime deadline. Avoid an
        // additional fixed 30-second transport deadline cutting it short.
        options.network.request_timeout = Duration::ZERO;
        options.network.connect_timeout = Duration::ZERO;
        options.network.proxy_options = serde_json::from_str(proxy_options)
            .map_err(|e| InvalidInput::new_err(e.to_string()))?;
        options
            .network
            .proxy_options
            .validate()
            .map_err(network_error)?;
        py.detach(|| Session::new(options))
            .map(|inner| Self {
                inner: Arc::new(inner),
                dispatcher,
            })
            .map_err(session_error)
    }
    fn new_page(&self, py: Python<'_>) -> PyResult<PyPage> {
        py.detach(|| self.inner.new_page())
            .map(|inner| PyPage {
                inner,
                dispatcher: self.dispatcher.clone(),
            })
            .map_err(session_error)
    }
    fn close(&self, py: Python<'_>) {
        py.detach(|| self.inner.close());
        self.dispatcher.close();
    }
    fn route(
        &self,
        pattern: &str,
        stage: &str,
        handler: Py<PyAny>,
    ) -> PyResult<network::PyRouteRegistration> {
        let stage = serde_json::from_value(serde_json::Value::String(stage.into()))
            .map_err(|e| InvalidInput::new_err(e.to_string()))?;
        self.inner
            .route(pattern, stage, self.dispatcher.route_handler(handler))
            .map(|inner| network::PyRouteRegistration { inner })
            .map_err(session_error)
    }
    fn set_cookie(&self, url: &str, name: &str, value: &str) -> PyResult<()> {
        self.inner
            .cookies()
            .set_cookie(url, name, value)
            .map_err(session_error)
    }
    fn cookies_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.cookies().cookies())
            .map_err(|e| RuntimeError::new_err(e.to_string()))
    }
    fn clear_cookies(&self) {
        self.inner.cookies().clear_cookies();
    }
}

#[pyclass(name = "_Response")]
struct PyResponse {
    inner: NavigationResponse,
}
#[pymethods]
impl PyResponse {
    #[getter]
    fn content<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.content)
    }
    fn metadata_json(&self) -> PyResult<String> {
        let mut data =
            serde_json::to_value(&self.inner).map_err(|e| RuntimeError::new_err(e.to_string()))?;
        data.as_object_mut().unwrap().remove("content");
        Ok(data.to_string())
    }
}

#[pyclass(name = "_Page")]
struct PyPage {
    inner: NativePage,
    dispatcher: Arc<plugins::Dispatcher>,
}
#[pymethods]
impl PyPage {
    fn advance_time(
        &self,
        py: Python<'_>,
        milliseconds: f64,
        max_tasks: usize,
    ) -> PyResult<String> {
        py.detach(|| self.inner.advance_time(milliseconds, max_tasks))
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn run_pending_tasks(&self, py: Python<'_>, max_tasks: usize) -> PyResult<String> {
        py.detach(|| self.inner.run_pending_tasks(max_tasks))
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn profile_contract(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.profile_contract())
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn profile_report(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.profile_report())
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn take_diagnostics(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.take_diagnostics())
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn peek_diagnostics(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.peek_diagnostics())
            .map(|v| v.to_string())
            .map_err(error)
    }
    fn summarize_diagnostics(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.summarize_diagnostics())
            .map(|v| v.to_string())
            .map_err(error)
    }

    fn set_identity(&self, py: Python<'_>, identity: &str) -> PyResult<()> {
        let identity =
            serde_json::from_str(identity).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        py.detach(|| self.inner.set_identity(identity))
            .map_err(error)
    }

    fn on(&self, event: &str, handler: Py<PyAny>) -> PyResult<network::PySubscription> {
        self.inner
            .on(
                event,
                self.dispatcher.event_handler(handler, self.inner.id()),
            )
            .map(|inner| network::PySubscription { inner })
            .map_err(error)
    }
    fn route(
        &self,
        pattern: &str,
        stage: &str,
        handler: Py<PyAny>,
    ) -> PyResult<network::PyRouteRegistration> {
        let stage = serde_json::from_value(serde_json::Value::String(stage.into()))
            .map_err(|e| InvalidInput::new_err(e.to_string()))?;
        self.inner
            .route(pattern, stage, self.dispatcher.route_handler(handler))
            .map(|inner| network::PyRouteRegistration { inner })
            .map_err(error)
    }
    #[allow(clippy::too_many_arguments)] // Named Python API options.
    fn configure_network(
        &self,
        py: Python<'_>,
        offline: bool,
        latency: f64,
        bypass_cache: bool,
        max_retained_bytes: usize,
        max_retained_body_bytes: usize,
        buffered_download_bytes_per_second: Option<usize>,
    ) -> PyResult<()> {
        let latency = Duration::try_from_secs_f64(latency)
            .map_err(|e| InvalidInput::new_err(e.to_string()))?;
        let options = brimp_runtime::runtime::networking::NetworkOptions {
            offline,
            latency,
            bypass_cache,
            max_retained_bytes,
            max_retained_body_bytes,
            buffered_download_bytes_per_second,
        };
        py.detach(|| self.inner.configure_network(options))
            .map_err(error)
    }
    fn set_extra_headers(&self, headers: Vec<(String, String)>) -> PyResult<()> {
        self.inner.set_extra_headers(headers).map_err(error)
    }
    fn clear_cache(&self) -> PyResult<()> {
        self.inner.clear_cache().map_err(error)
    }
    fn response_body<'py>(
        &self,
        py: Python<'py>,
        request_id: u64,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.inner.response_body(request_id).map_err(error)?;
        Ok(PyBytes::new(py, &bytes))
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (method, url, headers, body=None, timeout=None, allow_redirects=true, max_redirects=20, proxy_options="{}"))]
    fn request(
        &self,
        py: Python<'_>,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
        timeout: Option<f64>,
        allow_redirects: bool,
        max_redirects: usize,
        proxy_options: &str,
    ) -> PyResult<PyResponse> {
        let options = RequestOptions {
            headers,
            body,
            timeout: timeout.map(duration).transpose()?,
            allow_redirects,
            max_redirects,
            proxy_options: serde_json::from_str(proxy_options)
                .map_err(|e| InvalidInput::new_err(e.to_string()))?,
            cancellation: CancellationToken::current(),
        };
        py.detach(|| self.inner.request(method, url, options))
            .map(|inner| PyResponse { inner })
            .map_err(error)
    }
    #[pyo3(signature=(method,url,headers,body,timeout,allow_redirects,max_redirects,proxy_options="{}"))]
    #[allow(clippy::too_many_arguments)]
    fn stream(
        &self,
        py: Python<'_>,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
        timeout: Option<f64>,
        allow_redirects: bool,
        max_redirects: usize,
        proxy_options: &str,
    ) -> PyResult<PyStream> {
        let options = RequestOptions {
            headers,
            body,
            timeout: timeout.map(duration).transpose()?,
            allow_redirects,
            max_redirects,
            proxy_options: serde_json::from_str(proxy_options)
                .map_err(|e| InvalidInput::new_err(e.to_string()))?,
            cancellation: CancellationToken::current(),
        };
        py.detach(|| self.inner.stream(method, url, options))
            .map(|inner| PyStream {
                inner: Arc::new(inner),
            })
            .map_err(error)
    }
    fn evaluate(&self, py: Python<'_>, source: &str) -> PyResult<String> {
        py.detach(|| self.inner.evaluate_handle(source)?.value())
            .and_then(|v| {
                serde_json::to_string(&v).map_err(|e| AutomationError::Internal(e.to_string()))
            })
            .map_err(error)
    }
    fn evaluate_handle(&self, py: Python<'_>, source: &str) -> PyResult<PyHandle> {
        py.detach(|| self.inner.evaluate_handle(source))
            .map(|inner| PyHandle { inner })
            .map_err(error)
    }
    fn query_selector(&self, py: Python<'_>, selector: &str) -> PyResult<Option<PyHandle>> {
        py.detach(|| self.inner.query_selector(selector))
            .map(|v| v.map(|e| PyHandle { inner: e.0 }))
            .map_err(error)
    }
    fn query_selector_all(&self, py: Python<'_>, selector: &str) -> PyResult<Vec<PyHandle>> {
        py.detach(|| self.inner.query_selector_all(selector))
            .map(|v| v.into_iter().map(|e| PyHandle { inner: e.0 }).collect())
            .map_err(error)
    }
    fn add_init_script(&self, py: Python<'_>, source: &str) -> PyResult<()> {
        py.detach(|| self.inner.add_init_script(source))
            .map_err(error)
    }
    fn click(&self, py: Python<'_>, selector: &str) -> PyResult<()> {
        py.detach(|| self.inner.click(selector)).map_err(error)
    }
    fn hover(&self, py: Python<'_>, selector: &str) -> PyResult<()> {
        py.detach(|| self.inner.hover(selector)).map_err(error)
    }
    fn tap(&self, py: Python<'_>, selector: &str) -> PyResult<()> {
        py.detach(|| self.inner.tap(selector)).map_err(error)
    }
    fn focus(&self, py: Python<'_>, selector: &str) -> PyResult<()> {
        py.detach(|| self.inner.focus(selector)).map_err(error)
    }
    fn type_text(&self, py: Python<'_>, selector: &str, text: &str) -> PyResult<()> {
        py.detach(|| self.inner.type_text(selector, text))
            .map_err(error)
    }
    fn fill(&self, py: Python<'_>, selector: &str, text: &str) -> PyResult<()> {
        py.detach(|| self.inner.fill(selector, text)).map_err(error)
    }
    #[pyo3(signature = (selector, key, modifiers=0))]
    fn press(&self, py: Python<'_>, selector: &str, key: &str, modifiers: u8) -> PyResult<()> {
        py.detach(|| self.inner.press(selector, key, modifiers))
            .map_err(error)
    }
    fn drag_to(&self, py: Python<'_>, source: &str, destination: &str) -> PyResult<()> {
        py.detach(|| self.inner.drag_to(source, destination))
            .map_err(error)
    }
    fn mouse_move(&self, py: Python<'_>, x: f64, y: f64, steps: u32) -> PyResult<()> {
        py.detach(|| self.inner.mouse_move(x, y, steps))
            .map_err(error)
    }
    fn mouse_down(
        &self,
        py: Python<'_>,
        x: f64,
        y: f64,
        button: &str,
        click_count: u32,
        modifiers: u8,
    ) -> PyResult<()> {
        py.detach(|| self.inner.mouse_down(x, y, button, click_count, modifiers))
            .map_err(error)
    }
    fn mouse_up(
        &self,
        py: Python<'_>,
        x: f64,
        y: f64,
        button: &str,
        click_count: u32,
        modifiers: u8,
    ) -> PyResult<()> {
        py.detach(|| self.inner.mouse_up(x, y, button, click_count, modifiers))
            .map_err(error)
    }
    fn mouse_click(
        &self,
        py: Python<'_>,
        x: f64,
        y: f64,
        button: &str,
        click_count: u32,
        modifiers: u8,
    ) -> PyResult<()> {
        py.detach(|| self.inner.mouse_click(x, y, button, click_count, modifiers))
            .map_err(error)
    }
    fn mouse_wheel(
        &self,
        py: Python<'_>,
        x: f64,
        y: f64,
        delta_x: f64,
        delta_y: f64,
    ) -> PyResult<()> {
        py.detach(|| self.inner.mouse_wheel(x, y, delta_x, delta_y))
            .map_err(error)
    }
    fn set_input_files(&self, py: Python<'_>, selector: &str, files: &str) -> PyResult<()> {
        let files =
            serde_json::from_str(files).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        py.detach(|| self.inner.set_input_files(selector, files))
            .map_err(error)
    }
    fn extract(&self, py: Python<'_>, options: &str) -> PyResult<String> {
        let options =
            serde_json::from_str(options).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        let result = py.detach(|| self.inner.extract(options)).map_err(error)?;
        serde_json::to_string(&result).map_err(|e| RuntimeError::new_err(e.to_string()))
    }
    fn html(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.html()).map_err(error)
    }
    #[pyo3(signature = (html, url="about:blank"))]
    fn load_html(&self, py: Python<'_>, html: &str, url: &str) -> PyResult<()> {
        py.detach(|| self.inner.load_html(html, url)).map_err(error)
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.inner.close()).map_err(error)
    }
}

#[pyclass(name = "_Handle")]
struct PyHandle {
    inner: JSHandle,
}
#[pymethods]
impl PyHandle {
    fn focus(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| ElementHandle(self.inner.clone()).focus())
            .map_err(error)
    }
    fn click(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| ElementHandle(self.inner.clone()).click())
            .map_err(error)
    }
    fn set_input_files(&self, py: Python<'_>, files: &str) -> PyResult<()> {
        let files =
            serde_json::from_str(files).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        py.detach(|| ElementHandle(self.inner.clone()).set_input_files(files))
            .map_err(error)
    }

    fn command(&self, py: Python<'_>, command: &str) -> PyResult<String> {
        let command =
            serde_json::from_str(command).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        py.detach(|| self.inner.bridge_command(command))
            .and_then(|value| {
                serde_json::to_string(&value).map_err(|e| AutomationError::Internal(e.to_string()))
            })
            .map_err(error)
    }
    fn reference(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.reference().map_err(error)?)
            .map_err(|e| InvalidInput::new_err(e.to_string()))
    }
    fn bind_reference(&self, token: &str) -> PyResult<PyHandle> {
        let token =
            serde_json::from_str(token).map_err(|e| InvalidInput::new_err(e.to_string()))?;
        Ok(PyHandle {
            inner: self.inner.peer_handle(token),
        })
    }
    fn query_selector(&self, py: Python<'_>, selector: &str) -> PyResult<Option<PyHandle>> {
        py.detach(|| ElementHandle(self.inner.clone()).query_selector(selector))
            .map(|v| v.map(|e| PyHandle { inner: e.0 }))
            .map_err(error)
    }
    fn query_selector_all(&self, py: Python<'_>, selector: &str) -> PyResult<Vec<PyHandle>> {
        py.detach(|| ElementHandle(self.inner.clone()).query_selector_all(selector))
            .map(|v| v.into_iter().map(|e| PyHandle { inner: e.0 }).collect())
            .map_err(error)
    }
    fn dispose(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.inner.dispose()).map_err(error)
    }
}

#[pyclass(name = "_Stream")]
struct PyStream {
    inner: Arc<brimp_runtime::runtime::native::streaming::StreamingResponse>,
}
#[pymethods]
impl PyStream {
    fn metadata_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.metadata)
            .map_err(|e| RuntimeError::new_err(e.to_string()))
    }
    fn next_chunk<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyBytes>>> {
        py.detach(|| self.inner.body.next_chunk())
            .map(|value| value.map(|bytes| PyBytes::new(py, &bytes)))
            .map_err(error)
    }
    fn cancel(&self) {
        self.inner.body.cancel();
    }
}

#[pymodule]
fn _brimp(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<network::PyRoute>()?;
    m.add_class::<network::PySubscription>()?;
    m.add_class::<network::PyRouteRegistration>()?;
    m.add_class::<PyCancellation>()?;
    m.add_class::<PyStream>()?;
    m.add_class::<PySession>()?;
    m.add_class::<PyPage>()?;
    m.add_class::<PyResponse>()?;
    m.add_class::<PyHandle>()?;
    m.add_class::<plugins::PyHostContext>()?;
    Ok(())
}
