use async_trait::async_trait;
use brimp_runtime::{
    network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse, ResponseMetadata},
    runtime::{
        AutomationError,
        native::{RequestOptions, Session, SessionOptions},
    },
};
use http::{HeaderMap, HeaderValue, StatusCode};
use serde_json::json;
use std::{sync::Arc, time::Duration};

struct Fixture;
#[async_trait]
impl ResourceLoader for Fixture {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("text/html"));
        let body = if request.url.ends_with("/value") {
            b"42".to_vec()
        } else {
            br#"<!doctype html><title>Native</title><main id="main"><b>old</b></main>
            <script>globalThis.order = [document.querySelector('#later') === null]; document.querySelector('b').textContent = 'script';</script>
            <p id="later">later</p><script>order.push(!!document.querySelector('#later'));</script>"#.to_vec()
        };
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body,
            effective_url: request.url,
            metadata: ResponseMetadata::default(),
        })
    }
}
fn session() -> Session {
    Session::with_resource_loader(
        SessionOptions {
            timeout: Duration::from_secs(2),
            ..Default::default()
        },
        Arc::new(Fixture),
    )
    .unwrap()
}
#[test]
fn request_script_dom_handles_and_realm_lifetime() {
    let session = session();
    let page = session.new_page().unwrap();
    page.add_init_script("globalThis.beforeDocument = 42")
        .unwrap();
    let response = page
        .request("GET", "https://test.invalid/", RequestOptions::default())
        .unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(page.evaluate("beforeDocument").unwrap(), 42);
    assert_eq!(page.evaluate("order").unwrap(), json!([true, true]));
    let main = page.query_selector("main").unwrap().unwrap();
    let node = main.query_selector("b").unwrap().unwrap();
    assert_eq!(node.text_content().unwrap(), "script");
    node.set_attribute("data-note", "quote '\" \\ unicode λ")
        .unwrap();
    assert_eq!(
        node.get_attribute("data-note").unwrap().unwrap(),
        "quote '\" \\ unicode λ"
    );
    assert_eq!(page.query_selector_all("main b").unwrap().len(), 1);
    node.remove().unwrap();
    assert!(page.query_selector("b").unwrap().is_none());
    node.set_text_content("detached").unwrap();
    assert_eq!(node.text_content().unwrap(), "detached");
    let object = page.evaluate_handle("({a: 1, nested: {b: 2}})").unwrap();
    assert_eq!(object.property_names().unwrap(), vec!["a", "nested"]);
    assert_eq!(
        object
            .call("(value, increment) => value.a + increment", vec![json!(41)])
            .unwrap()
            .json_value()
            .unwrap(),
        42
    );
    let clone = object.clone();
    object.dispose().unwrap();
    assert!(matches!(
        clone.json_value(),
        Err(AutomationError::StaleHandle)
    ));
    page.request(
        "GET",
        "https://test.invalid/next",
        RequestOptions::default(),
    )
    .unwrap();
    assert!(matches!(
        node.text_content(),
        Err(AutomationError::StaleHandle)
    ));
    session.close();
    assert!(matches!(page.evaluate("42"), Err(AutomationError::Closed)));
}
#[test]
fn evaluation_pumps_timers_and_network() {
    let session = session();
    let page = session.new_page().unwrap();
    page.request("GET", "https://test.invalid/", RequestOptions::default())
        .unwrap();
    assert_eq!(
        page.evaluate("new Promise(resolve => setTimeout(() => resolve(42), 5))")
            .unwrap(),
        42
    );
    assert_eq!(
        page.evaluate("fetch('/value').then(r => r.text()).then(Number)")
            .unwrap(),
        42
    );
    assert!(
        matches!(page.evaluate("Promise.reject(new Error('boom'))"), Err(AutomationError::JavaScript(message)) if message.contains("boom"))
    );
    assert_eq!(page.evaluate("21 * 2").unwrap(), 42);
}
#[test]
fn concurrent_pages_and_session_isolation() {
    let first = session();
    let second = session();
    first
        .cookies()
        .set_cookie("https://test.invalid/", "secret", "one")
        .unwrap();
    let a = first.new_page().unwrap();
    let b = first.new_page().unwrap();
    let c = second.new_page().unwrap();
    std::thread::scope(|scope| {
        let handles = [&a, &b, &c].map(|page| {
            scope.spawn(move || {
                page.request("GET", "https://test.invalid/", RequestOptions::default())
                    .unwrap()
            })
        });
        for handle in handles {
            handle.join().unwrap();
        }
    });
    assert_eq!(a.evaluate("document.cookie").unwrap(), "secret=one");
    assert_eq!(b.evaluate("document.cookie").unwrap(), "secret=one");
    assert_eq!(c.evaluate("document.cookie").unwrap(), "");
    a.evaluate("globalThis.onlyA = 1").unwrap();
    assert_eq!(b.evaluate("typeof onlyA").unwrap(), "undefined");
    first.close();
    assert_eq!(c.evaluate("42").unwrap(), 42);
}

#[test]
fn close_interrupts_pending_evaluation() {
    let session = session();
    let page = session.new_page().unwrap();
    let pending_page = page.clone();
    let pending = std::thread::spawn(move || pending_page.evaluate("new Promise(() => {})"));
    std::thread::sleep(Duration::from_millis(20));
    let started = std::time::Instant::now();
    page.close().unwrap();
    assert!(matches!(
        pending.join().unwrap(),
        Err(AutomationError::Closed)
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}

struct PendingLoader;
#[async_trait]
impl ResourceLoader for PendingLoader {
    async fn fetch(&self, _: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        std::future::pending().await
    }
}

#[test]
fn close_cancels_pending_navigation() {
    let session =
        Session::with_resource_loader(SessionOptions::default(), Arc::new(PendingLoader)).unwrap();
    let page = session.new_page().unwrap();
    let pending_page = page.clone();
    let pending = std::thread::spawn(move || {
        pending_page.request("GET", "https://test.invalid/", RequestOptions::default())
    });
    std::thread::sleep(Duration::from_millis(20));
    let started = std::time::Instant::now();
    page.close().unwrap();
    assert!(matches!(
        pending.join().unwrap(),
        Err(AutomationError::Closed)
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn evaluation_timeout_does_not_poison_the_page() {
    let session = Session::with_resource_loader(
        SessionOptions {
            timeout: Duration::from_millis(30),
            ..Default::default()
        },
        Arc::new(Fixture),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    assert!(matches!(
        page.evaluate("new Promise(() => {})"),
        Err(AutomationError::Timeout(_))
    ));
    assert_eq!(page.evaluate("42").unwrap(), 42);
}
