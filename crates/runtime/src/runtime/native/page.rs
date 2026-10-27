//! Thread-safe host operations on a managed page.

use super::options::validate_timeout;
use super::{
    ElementHandle, FileUpload, HandleToken, IdentityOptions, JSHandle, Operation, OperationResult,
    RequestOptions,
};
use crate::runtime::diagnostics::DiagnosticsView;
use crate::runtime::{AutomationError, CancellationToken, NavigationResponse, PageHandle};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct NativePage {
    pub(super) page: PageHandle,
    pub(super) network: Arc<crate::runtime::networking::NetworkRuntime>,
    pub(super) events: crate::runtime::events::EventHub,
    pub(super) timeout: Duration,
}

impl NativePage {
    pub fn wait_for_selector(
        &self,
        selector: &str,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), AutomationError> {
        self.page.wait_for_selector(selector, timeout, cancellation)
    }
    pub fn wait_for_network_idle(
        &self,
        quiet: Duration,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<(), AutomationError> {
        self.page
            .wait_for_network_idle(quiet, timeout, cancellation)
    }

    pub fn advance_time(
        &self,
        milliseconds: f64,
        max_tasks: usize,
    ) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::AdvanceTime {
            milliseconds,
            max_tasks,
        })?
        else {
            unreachable!()
        };
        Ok(value)
    }
    pub fn run_pending_tasks(&self, max_tasks: usize) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::DrainTasks { max_tasks })?
        else {
            unreachable!()
        };
        Ok(value)
    }
    /// Describe profile values and capture recipes from the installed interfaces.
    pub fn profile_contract(&self) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::ProfileContract)? else {
            unreachable!()
        };
        Ok(value)
    }
    /// Report defaults used and the supplied browser capture metadata.
    pub fn profile_report(&self) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::ProfileReport)? else {
            unreachable!()
        };
        Ok(value)
    }
    pub fn take_diagnostics(&self) -> Result<Value, AutomationError> {
        self.diagnostics(DiagnosticsView::Take)
    }
    /// Inspect retained diagnostic records without consuming them or the loss count.
    pub fn peek_diagnostics(&self) -> Result<Value, AutomationError> {
        self.diagnostics(DiagnosticsView::Peek)
    }
    /// Group retained observations by layer, path, operation and reflection target.
    /// This does not consume records and does not imply complete dependency coverage.
    pub fn summarize_diagnostics(&self) -> Result<Value, AutomationError> {
        self.diagnostics(DiagnosticsView::Summary)
    }
    fn diagnostics(&self, view: DiagnosticsView) -> Result<Value, AutomationError> {
        let OperationResult::Value(value) = self.operation(Operation::Diagnostics(view))? else {
            unreachable!()
        };
        Ok(value)
    }

    pub fn stream(
        &self,
        method: &str,
        url: &str,
        options: RequestOptions,
    ) -> Result<crate::runtime::native::streaming::StreamingResponse, AutomationError> {
        let method = http::Method::from_bytes(method.as_bytes())
            .map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        let url = url::Url::parse(url).map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        if !["http", "https"].contains(&url.scheme()) {
            return Err(AutomationError::InvalidInput(
                "streams require HTTP(S)".into(),
            ));
        }
        let mut request = crate::network::ResourceRequest::new(method, url.to_string());
        request.headers = crate::runtime::routes::parse_headers(options.headers.clone())?;
        request.body = options.body.clone();
        // Capture before obtaining the context: navigation at either boundary must
        // invalidate this stream, including a gap between redirect hops.
        let generation = self.network.stream_generation();
        let OperationResult::NetworkContext(context) = self.operation(Operation::NetworkContext)?
        else {
            unreachable!()
        };
        let timeout = options.timeout.unwrap_or(self.timeout);
        validate_timeout(timeout)?;
        crate::runtime::native::streaming::open(
            &self.network,
            context,
            self.id(),
            generation,
            request,
            options,
            timeout,
        )
    }
    pub fn on(
        &self,
        event: &str,
        handler: crate::runtime::events::EventHandler,
    ) -> Result<crate::runtime::events::Subscription, AutomationError> {
        self.page.ensure_access()?;
        let page_id = self.id();
        self.events.on(
            event,
            Arc::new(move |event| {
                let _guard = crate::runtime::plugins::enter_page_callback(page_id);
                handler(event);
            }),
        )
    }
    pub fn route(
        &self,
        pattern: &str,
        stage: crate::runtime::routes::RouteStage,
        handler: crate::runtime::routes::RouteHandler,
    ) -> Result<crate::runtime::routes::RouteRegistration, AutomationError> {
        self.page.ensure_access()?;
        self.network.routes.register(pattern, stage, handler)
    }
    pub fn configure_network(
        &self,
        options: crate::runtime::networking::NetworkOptions,
    ) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        let offline = options.offline;
        self.network.configure(options)?;
        self.page.set_network_offline(offline)
    }
    pub fn set_identity(&self, identity: IdentityOptions) -> Result<(), AutomationError> {
        self.page.set_identity(identity)
    }
    pub fn set_extra_headers(&self, headers: Vec<(String, String)>) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        self.network.extra_headers(headers)
    }
    pub fn clear_cache(&self) -> Result<(), AutomationError> {
        self.page.ensure_access()?;
        self.network.clear_cache();
        Ok(())
    }
    pub fn response_body(&self, request_id: u64) -> Result<Vec<u8>, AutomationError> {
        self.page.ensure_access()?;
        self.network.body(request_id)
    }
    pub fn id(&self) -> u64 {
        self.page.id()
    }
    pub fn check_callback(&self) -> Result<(), AutomationError> {
        if crate::runtime::plugins::is_callback_page(self.id()) {
            return Err(AutomationError::Reentry);
        }
        Ok(())
    }
    pub fn request(
        &self,
        method: &str,
        url: &str,
        options: RequestOptions,
    ) -> Result<NavigationResponse, AutomationError> {
        self.page.navigate_request(
            method,
            url,
            options.timeout.unwrap_or(self.timeout),
            options.cancellation,
            options.headers,
            options.body,
            options.allow_redirects,
            options.max_redirects,
        )
    }

    pub fn evaluate(&self, source: &str) -> Result<Value, AutomationError> {
        match self.operation(Operation::Evaluate {
            source: source.into(),
            by_value: true,
        })? {
            OperationResult::Value(value) => Ok(value),
            _ => unreachable!(),
        }
    }

    pub fn evaluate_handle(&self, source: &str) -> Result<JSHandle, AutomationError> {
        match self.operation(Operation::Evaluate {
            source: source.into(),
            by_value: false,
        })? {
            OperationResult::Handle(token) => Ok(self.handle(token)),
            _ => unreachable!(),
        }
    }

    pub fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>, AutomationError> {
        Ok(self.query(None, selector, false)?.into_iter().next())
    }

    pub fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        self.query(None, selector, true)
    }

    pub(super) fn query(
        &self,
        root: Option<HandleToken>,
        selector: &str,
        all: bool,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        match self.operation(Operation::Query {
            root,
            selector: selector.into(),
            all,
        })? {
            OperationResult::Handles(tokens) => Ok(tokens
                .into_iter()
                .map(|token| ElementHandle(self.handle(token)))
                .collect()),
            _ => unreachable!(),
        }
    }

    fn handle(&self, mut handle: JSHandle) -> JSHandle {
        handle.1 = Some(self.clone());
        handle
    }
    pub(super) fn operation(
        &self,
        operation: Operation,
    ) -> Result<OperationResult, AutomationError> {
        self.page.native_operation(operation, self.timeout)
    }

    pub fn add_init_script(&self, source: &str) -> Result<(), AutomationError> {
        self.page.add_preload_script(source, source)
    }
    fn input(&self, command: Value) -> Result<(), AutomationError> {
        self.operation(Operation::Input {
            target: None,
            command,
        })?;
        Ok(())
    }
    pub fn click(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"click","selector":selector}))
    }
    pub fn hover(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"hover","selector":selector}))
    }
    pub fn tap(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"tap","selector":selector}))
    }
    pub fn focus(&self, selector: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"focus","selector":selector}))
    }
    pub fn type_text(&self, selector: &str, text: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"type","selector":selector,"text":text}))
    }
    pub fn fill(&self, selector: &str, text: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"fill","selector":selector,"text":text}))
    }
    pub fn press(&self, selector: &str, key: &str, modifiers: u8) -> Result<(), AutomationError> {
        self.input(json!({"action":"press","selector":selector,"key":key,"modifiers":modifiers}))
    }
    pub fn drag_to(&self, source: &str, destination: &str) -> Result<(), AutomationError> {
        self.input(json!({"action":"dragTo","selector":source,"destination":destination}))
    }
    pub fn set_input_files(
        &self,
        selector: &str,
        files: Vec<FileUpload>,
    ) -> Result<(), AutomationError> {
        self.input(json!({"action":"files","selector":selector,"files":files}))
    }
    pub fn extract(
        &self,
        options: crate::runtime::ExtractionOptions,
    ) -> Result<crate::runtime::ExtractedDocument, AutomationError> {
        self.page.extract(options)
    }
    pub fn html(&self) -> Result<String, AutomationError> {
        self.page.html()
    }
    pub fn close(&self) -> Result<(), AutomationError> {
        self.check_callback()?;
        self.page.close();
        Ok(())
    }
    pub fn is_closed(&self) -> bool {
        self.page.is_closed()
    }
}

#[cfg(test)]
mod stream_lifecycle_tests {
    use super::*;
    use crate::runtime::native::{Session, SessionOptions};

    #[test]
    fn navigation_or_close_between_context_capture_and_stream_start_cancels_it() {
        let session = Session::new(SessionOptions::default()).unwrap();
        let page = session.new_page().unwrap();
        let generation = page.network.stream_generation();
        let OperationResult::NetworkContext(context) =
            page.operation(Operation::NetworkContext).unwrap()
        else {
            unreachable!()
        };
        page.request("GET", "data:text/html,new", RequestOptions::default())
            .unwrap();
        let start = || {
            crate::runtime::native::streaming::open(
                &page.network,
                context.clone(),
                page.id(),
                generation,
                crate::network::ResourceRequest::get("https://unused.test/"),
                RequestOptions::default(),
                Duration::from_secs(1),
            )
        };
        assert!(matches!(start(), Err(AutomationError::Cancellation)));
        page.close().unwrap();
        assert!(matches!(start(), Err(AutomationError::Closed)));
    }
}
