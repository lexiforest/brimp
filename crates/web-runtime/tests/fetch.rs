use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, PageOptions};

#[derive(Default)]
struct FetchLoader {
    requests: Mutex<Vec<ResourceRequest>>,
}

#[async_trait]
impl ResourceLoader for FetchLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        if request.url.ends_with("/failure") {
            return Err(NetworkError::Transport("offline".to_string()));
        }
        let url = request.url.clone();
        self.requests.lock().unwrap().push(request);
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("x-result", HeaderValue::from_static("yes"));
        Ok(ResourceResponse {
            status: StatusCode::CREATED,
            headers,
            body: br#"{"answer":42}"#.to_vec(),
            effective_url: url,
        })
    }
}

#[test]
fn fetch_resolves_a_response_on_the_page_thread() {
    let loader = Arc::new(FetchLoader::default());
    let browser = Browser::with_resource_loader(loader.clone());
    let mut page = browser.new_page(PageOptions::default()).unwrap();

    page.eval(
        r#"
        globalThis.fetchResult = "waiting";
        fetch("https://example.test/api", {
            method: "POST",
            headers: { "Content-Type": "text/plain", "X-Test": "one" },
            body: "hello",
        }).then(async response => {
            const json = await response.json();
            fetchResult = [response.status, response.ok, response.headers.get("x-result"), json.answer].join("|");
        });
        "#,
    )
    .unwrap();
    assert_eq!(
        page.eval("fetchResult").unwrap().to_string().unwrap(),
        "waiting"
    );

    assert!(page.run_until_idle_for(Duration::from_secs(1)).unwrap());

    assert_eq!(
        page.eval("fetchResult").unwrap().to_string().unwrap(),
        "201|true|yes|42"
    );
    let requests = loader.requests.lock().unwrap();
    let request = &requests[0];
    assert_eq!(request.method, http::Method::POST);
    assert_eq!(request.headers["content-type"], "text/plain");
    assert_eq!(request.headers["x-test"], "one");
    assert_eq!(request.body.as_deref(), Some(&b"hello"[..]));
}

#[test]
fn fetch_rejects_transport_and_invalid_request_failures() {
    let browser = Browser::with_resource_loader(Arc::new(FetchLoader::default()));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.eval(
        r#"
        globalThis.failures = [];
        fetch("https://example.test/failure").catch(error => failures.push(error instanceof TypeError && error.message.includes("offline")));
        fetch("https://example.test/api", { method: "GET", body: "invalid" })
            .catch(error => failures.push(error instanceof TypeError && error.message.includes("cannot have a body")));
        "#,
    )
    .unwrap();

    assert!(page.run_until_idle_for(Duration::from_secs(1)).unwrap());

    assert_eq!(
        page.eval("failures.join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "true,true"
    );
}
