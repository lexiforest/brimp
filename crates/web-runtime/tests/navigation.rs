use std::sync::Arc;

use async_trait::async_trait;
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, LoadState, PageOptions};

struct StaticLoader;

#[async_trait]
impl ResourceLoader for StaticLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body: b"<!doctype html><html><head><title>Loaded</title></head><body id='ready'></body></html>".to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn goto_fetches_and_installs_the_main_html_document() {
    let browser = Browser::with_resource_loader(Arc::new(StaticLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://example.test/page"))
        .unwrap();
    runtime.block_on(page.wait_for_load()).unwrap();

    assert_eq!(page.load_state(), LoadState::Complete);
    assert_eq!(page.url(), Some("https://example.test/page"));
    assert!(page.document().get_element_by_id("ready").is_some());
    assert_eq!(
        page.eval("document.title").unwrap().to_string().unwrap(),
        "Loaded"
    );
}

struct SubresourceLoader;

#[async_trait]
impl ResourceLoader for SubresourceLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://example.test/app/" => (
                "text/html",
                r#"<!doctype html>
                <link rel="stylesheet" href="assets/site.css">
                <div id="box"></div>
                <script>globalThis.scriptOrder = ["inline"]; document.cookie = "client=yes; Path=/";</script>
                <script src="assets/app.js"></script>"#,
            ),
            "https://example.test/app/assets/site.css" => {
                let cookies = request.headers.get(COOKIE).unwrap().to_str().unwrap();
                assert!(cookies.contains("session=abc"));
                assert!(cookies.contains("secret=hidden"));
                ("text/css", "#box { width: 123px; }")
            }
            "https://example.test/app/assets/app.js" => (
                {
                    let cookies = request.headers.get(COOKIE).unwrap().to_str().unwrap();
                    assert!(cookies.contains("session=abc"));
                    assert!(cookies.contains("client=yes"));
                    "text/javascript"
                },
                "scriptOrder.push('external'); document.querySelector('#box').setAttribute('data-script', 'done');",
            ),
            other => panic!("unexpected resource request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        if request.url == "https://example.test/app/" {
            headers.insert(SET_COOKIE, HeaderValue::from_static("session=abc; Path=/"));
            headers.append(
                SET_COOKIE,
                HeaderValue::from_static("secret=hidden; Path=/; HttpOnly"),
            );
        }
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn navigation_loads_css_and_classic_scripts_through_the_loader() {
    let browser = Browser::with_resource_loader(Arc::new(SubresourceLoader));
    let mut page = browser
        .new_page(PageOptions::builder().viewport(640, 480).build())
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://example.test/app/"))
        .unwrap();

    assert_eq!(
        page.eval("scriptOrder.join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "inline,external"
    );
    let document = page.document();
    let box_id = document.get_element_by_id("box").unwrap();
    assert_eq!(
        document
            .node(box_id)
            .unwrap()
            .attr(blitz_dom::LocalName::from("data-script")),
        Some("done")
    );
    drop(document);
    assert_eq!(
        page.eval("document.querySelector('#box').getBoundingClientRect().width")
            .unwrap()
            .to_number()
            .unwrap(),
        123.0
    );
    let cookies = page.eval("document.cookie").unwrap().to_string().unwrap();
    assert!(cookies.contains("session=abc"));
    assert!(cookies.contains("client=yes"));
    assert!(!cookies.contains("secret=hidden"));
}

#[test]
fn navigation_updates_location_and_exposes_a_navigator_subset() {
    let browser = Browser::with_resource_loader(Arc::new(StaticLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime
        .block_on(page.goto("https://example.test:8443/path?q=rust#result"))
        .unwrap();

    assert_eq!(
        page.eval(
            "[location.protocol, location.host, location.pathname, location.search, location.hash].join('|')"
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "https:|example.test:8443|/path|?q=rust|#result"
    );
    assert_eq!(
        page.eval("`${navigator.userAgent}|${navigator.platform}|${navigator.language}`")
            .unwrap()
            .to_string()
            .unwrap(),
        "Brimp/0.1|MacIntel|en-US"
    );
}
