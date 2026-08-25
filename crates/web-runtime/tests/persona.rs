use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use web_runtime::{AutomationBrowser, PageOptions};

#[derive(Default)]
struct IdentityLoader {
    requests: Mutex<Vec<ResourceRequest>>,
}
#[async_trait]
impl ResourceLoader for IdentityLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let url = request.url.clone();
        self.requests.lock().unwrap().push(request);
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: b"<!doctype html><title>Persona</title>".to_vec(),
            effective_url: url,
        })
    }
}

#[test]
fn request_and_javascript_observe_one_coherent_identity() {
    let loader = Arc::new(IdentityLoader::default());
    let browser = AutomationBrowser::with_resource_loader(loader.clone());
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://identity.test/", Duration::from_secs(1))
        .unwrap();
    let observed = page.evaluate("({ userAgent: navigator.userAgent, platform: navigator.platform, language: navigator.language, languages: navigator.languages, viewport: [window.innerWidth, window.innerHeight, window.devicePixelRatio], screen: [screen.width, screen.height] })").unwrap();
    let request = &loader.requests.lock().unwrap()[0];
    assert_eq!(
        request
            .headers
            .get(http::header::USER_AGENT)
            .unwrap()
            .to_str()
            .unwrap(),
        observed["userAgent"]
    );
    assert_eq!(
        request
            .headers
            .get(http::header::ACCEPT_LANGUAGE)
            .unwrap()
            .to_str()
            .unwrap(),
        observed["language"]
    );
    assert_eq!(observed["platform"], "MacIntel");
    assert_eq!(observed["languages"], serde_json::json!(["en-US", "en"]));
    assert_eq!(observed["viewport"][0], 800);
    assert_eq!(observed["viewport"][1], 600);
    assert_eq!(observed["viewport"][2].as_f64(), Some(1.0));
    assert_eq!(observed["screen"], serde_json::json!([800, 600]));
}
