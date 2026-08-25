use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{AutomationBrowser, AutomationError, PageOptions};

struct WorkflowLoader;
#[async_trait]
impl ResourceLoader for WorkflowLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: b"<!doctype html><title>Workflow</title><main>Hello</main>".to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn shared_automation_workflow_navigates_evaluates_screenshots_and_closes() {
    let browser = AutomationBrowser::with_resource_loader(Arc::new(WorkflowLoader));
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://example.test/", Duration::from_secs(1))
        .unwrap();
    assert_eq!(page.title().unwrap(), "Workflow");
    assert_eq!(
        page.evaluate("({ answer: 6 * 7, values: [true, null] })")
            .unwrap()["answer"],
        42
    );
    assert!(page.text_content().unwrap().contains("Hello"));
    assert!(
        page.screenshot(false)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert!(matches!(
        page.evaluate("undefined"),
        Err(AutomationError::Unsupported(_))
    ));
    assert!(matches!(
        page.evaluate("throw new Error('boom')"),
        Err(AutomationError::JavaScript(_))
    ));
    page.close();
    page.close();
    assert!(matches!(page.title(), Err(AutomationError::Closed)));
    browser.close();
    browser.close();
    assert!(browser.is_closed());
}

struct HangingLoader;
#[async_trait]
impl ResourceLoader for HangingLoader {
    async fn fetch(&self, _request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        std::future::pending().await
    }
}

#[test]
fn navigation_timeout_is_deterministic_and_close_remains_safe() {
    let browser = AutomationBrowser::with_resource_loader(Arc::new(HangingLoader));
    let page = browser.new_page(PageOptions::default()).unwrap();
    assert!(matches!(
        page.navigate("https://example.test/", Duration::from_millis(20)),
        Err(AutomationError::Timeout(_))
    ));
    page.close();
}
