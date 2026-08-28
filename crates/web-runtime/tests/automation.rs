use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{AutomationBrowser, AutomationError, PageOptions, RemoteArgument};

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
    page.set_viewport(640, 480, 2.0).unwrap();
    page.add_preload_script("test-preload", "globalThis.preloaded = 42")
        .unwrap();
    page.navigate("https://example.test/", Duration::from_secs(1))
        .unwrap();
    let viewport = page.viewport().unwrap();
    assert_eq!((viewport.width, viewport.height), (640.0, 480.0));
    assert_eq!(viewport.device_pixel_ratio, 2.0);
    assert_eq!(page.evaluate("preloaded").unwrap(), 42);
    let remote = page
        .evaluate_remote(
            "({answer: 42, nested: {ok: true}})",
            false,
            Some("test".into()),
            false,
        )
        .unwrap();
    let object_id = remote["objectId"].as_str().unwrap().to_owned();
    let properties = page
        .remote_object_properties(object_id.clone(), true, false)
        .unwrap();
    let answer_property = properties["result"]
        .as_array()
        .unwrap()
        .iter()
        .find(|property| property["name"] == "answer")
        .unwrap();
    assert_eq!(answer_property["value"]["value"], 42);
    let answer = page
        .call_function_remote(
            "function () { return this.answer; }",
            Some(object_id.clone()),
            Vec::<RemoteArgument>::new(),
            true,
            None,
            false,
        )
        .unwrap();
    assert_eq!(answer["value"], 42);
    let promised = page
        .evaluate_remote(
            "new Promise(resolve => setTimeout(() => resolve(42), 5))",
            true,
            None,
            true,
        )
        .unwrap();
    assert_eq!(promised["value"], 42);
    assert_eq!(page.release_remote_object_group("test").unwrap(), 2);
    assert!(!page.release_remote_object(object_id.clone()).unwrap());
    assert!(!page.release_remote_object(object_id).unwrap());
    assert!(page.remove_preload_script("test-preload").unwrap());
    page.navigate("https://example.test/next", Duration::from_secs(1))
        .unwrap();
    assert_eq!(page.evaluate("typeof preloaded").unwrap(), "undefined");
    assert_eq!(page.title().unwrap(), "Workflow");
    assert_eq!(
        page.evaluate("({ answer: 6 * 7, values: [true, null] })")
            .unwrap()["answer"],
        42
    );
    page.evaluate("setTimeout(() => { globalThis.timerFinished = true; }, 10)")
        .unwrap();
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(page.evaluate("timerFinished").unwrap(), true);
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

struct ResponseLoader;

#[async_trait]
impl ResourceLoader for ResponseLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (status, content_type, body) = match request.url.as_str() {
            "https://response.test/missing?q=rust" => (
                StatusCode::NOT_FOUND,
                "text/html; charset=utf-8",
                "<!doctype html><title>Missing</title><main id='value'>raw</main><script>previousRealm = true; document.getElementById('value').textContent = 'rendered'</script>",
            ),
            "https://response.test/next" => (
                StatusCode::OK,
                "text/html",
                "<main id='realm'></main><script>document.getElementById('realm').textContent = typeof previousRealm</script>",
            ),
            "https://response.test/data" => {
                (StatusCode::OK, "application/json", r#"{"answer":42}"#)
            }
            other => panic!("unexpected URL: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static(content_type),
        );
        headers.insert(
            http::header::SET_COOKIE,
            HeaderValue::from_static("session=ready; Path=/"),
        );
        Ok(ResourceResponse {
            status,
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn navigation_returns_raw_and_rendered_responses_without_raising_for_http_status() {
    let browser = AutomationBrowser::with_resource_loader(Arc::new(ResponseLoader));
    let page = browser.new_page(PageOptions::default()).unwrap();

    let missing = page
        .navigate(
            "https://response.test/missing?q=rust",
            Duration::from_secs(1),
        )
        .unwrap();
    assert_eq!(missing.status_code, 404);
    assert_eq!(missing.reason, "Not Found");
    assert!(
        String::from_utf8(missing.content)
            .unwrap()
            .contains(">raw<")
    );
    assert!(missing.html.unwrap().contains(">rendered<"));
    assert_eq!(missing.cookies, vec![("session".into(), "ready".into())]);

    let next = page
        .navigate("https://response.test/next", Duration::from_secs(1))
        .unwrap();
    assert!(next.html.unwrap().contains(">undefined<"));

    let data = page
        .navigate("https://response.test/data", Duration::from_secs(1))
        .unwrap();
    assert_eq!(data.content, br#"{"answer":42}"#);
    assert_eq!(data.html, None);
}
