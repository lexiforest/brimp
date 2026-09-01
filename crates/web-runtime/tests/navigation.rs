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
            headers: headers.into(),
            body: b"<!doctype html><html><head><title>Loaded</title></head><body id='ready'></body></html>".to_vec(),
            effective_url: request.url,
            metadata: network::ResponseMetadata::default(),
        })
    }
}

struct LoadEventLoader;

#[async_trait]
impl ResourceLoader for LoadEventLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: br#"<!doctype html>
                <meta name="first">
                <meta name="second">
                <script>
                    globalThis.events = [];
                    document.addEventListener("DOMContentLoaded", () => events.push("dom"));
                    window.addEventListener("load", () => {
                        events.push("load");
                        setTimeout(() => events.push("timer"), 0);
                    });
                </script>"#
                .to_vec(),
            effective_url: request.url,
            metadata: network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn navigation_dispatches_load_events_and_runs_immediate_tasks() {
    let browser = Browser::with_resource_loader(Arc::new(LoadEventLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://example.test/"))
        .unwrap();

    assert_eq!(
        page.eval("events.join(',')").unwrap().to_string().unwrap(),
        "dom,load,timer"
    );
    assert_eq!(
        page.eval("document.getElementsByTagName('meta').length")
            .unwrap()
            .to_number()
            .unwrap(),
        2.0
    );
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
    assert_eq!(page.url().as_deref(), Some("https://example.test/page"));
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
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
            metadata: network::ResponseMetadata::default(),
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
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36|MacIntel|en-US"
    );
}

#[test]
fn history_api_updates_same_document_entries_and_dispatches_traversal_events() {
    let browser = Browser::with_resource_loader(Arc::new(StaticLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime
        .block_on(page.goto("https://example.test/app?start#initial"))
        .unwrap();

    let initialized = page
        .eval(
            r#"Number(Boolean((() => {
                globalThis.historyEvents = [];
                window.addEventListener("popstate", event => historyEvents.push([
                    event.type, event.state.step, event.isTrusted, location.href,
                ].join("|")));
                window.addEventListener("hashchange", event => historyEvents.push([
                    event.type, event.oldURL, event.newURL, event.isTrusted,
                ].join("|")));
                history.scrollRestoration = "manual";
                history.replaceState({ step: 0 }, "ignored", "/app?base#zero");
                const first = { step: 1 };
                history.pushState(first, "ignored", "?first#one");
                first.step = 99;
                history.pushState({ step: 2 }, "ignored", "?second#two");
                history.back();
                return history.length === 3 && history.state.step === 2 &&
                    history.scrollRestoration === "manual" &&
                    Object.prototype.toString.call(history) === "[object History]" &&
                    window.history === history && Object.keys(history).length === 0 &&
                    History.prototype.pushState.length === 2 &&
                    Object.getOwnPropertyDescriptor(History.prototype, "state").enumerable &&
                    Function.prototype.toString.call(History.prototype.back) ===
                        "function back() { [native code] }";
            })()))"#,
        )
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(initialized, 1.0);
    page.run_pending_tasks().unwrap();

    assert_eq!(
        page.eval("`${history.state.step}|${location.pathname}${location.search}${location.hash}|${document.URL}`")
            .unwrap()
            .to_string()
            .unwrap(),
        "1|/app?first#one|https://example.test/app?first#one"
    );
    assert_eq!(
        page.url().as_deref(),
        Some("https://example.test/app?first#one")
    );
    assert_eq!(
        page.eval("historyEvents.join('\\n')")
            .unwrap()
            .to_string()
            .unwrap(),
        "popstate|1|true|https://example.test/app?first#one\n\
         hashchange|https://example.test/app?second#two|https://example.test/app?first#one|true"
    );

    page.eval("history.forward()").unwrap();
    page.run_pending_tasks().unwrap();
    assert_eq!(
        page.eval("history.state.step")
            .unwrap()
            .to_number()
            .unwrap(),
        2.0
    );
    assert!(
        page.eval(
            r#"Number(Boolean((() => {
                try { history.pushState({}, "", "https://other.test/"); return false; }
                catch (error) { if (!(error instanceof DOMException) || error.name !== "SecurityError") return false; }
                try { history.pushState(() => {}, ""); return false; }
                catch (error) { if (!(error instanceof DOMException) || error.name !== "DataCloneError") return false; }
                try { History.prototype.back.call({}); return false; }
                catch (error) { return error instanceof TypeError; }
            })()))"#,
        )
        .unwrap()
        .to_number()
        .unwrap()
            == 1.0
    );
}
