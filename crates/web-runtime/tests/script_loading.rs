use std::sync::Arc;

use async_trait::async_trait;
use http::{HeaderMap, StatusCode, header::CONTENT_TYPE};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, PageOptions};

struct ScriptLoader;

async fn yield_times(count: usize) {
    for _ in 0..count {
        tokio::task::yield_now().await;
    }
}

#[async_trait]
impl ResourceLoader for ScriptLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://example.test/" => (
                "text/html",
                r#"<!doctype html>
                <script>globalThis.order = ["inline-start"];</script>
                <script src="/defer-first.js" defer></script>
                <script src="/defer-second.js" defer></script>
                <script src="/async-slow.js" async></script>
                <script src="/async-fast.js" async></script>
                <script defer>order.push("inline-attribute")</script>
                <script type="module">order.push("module")</script>
                <script type="application/json">order.push("data")</script>
                <script src="/blocking.js"></script>"#,
            ),
            "https://example.test/defer-first.js" => {
                yield_times(5).await;
                ("text/javascript", "order.push('defer-first')")
            }
            "https://example.test/defer-second.js" => {
                yield_times(1).await;
                ("text/javascript", "order.push('defer-second')")
            }
            "https://example.test/async-slow.js" => {
                yield_times(4).await;
                ("text/javascript", "order.push('async-slow')")
            }
            "https://example.test/async-fast.js" => {
                yield_times(1).await;
                ("text/javascript", "order.push('async-fast')")
            }
            "https://example.test/blocking.js" => ("text/javascript", "order.push('blocking')"),
            other => panic!("unexpected script request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn classic_async_and_defer_scripts_follow_their_execution_ordering_rules() {
    let browser = Browser::with_resource_loader(Arc::new(ScriptLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://example.test/"))
        .unwrap();

    let order = page.eval("order.join(',')").unwrap().to_string().unwrap();
    assert_eq!(
        order,
        "inline-start,inline-attribute,blocking,async-fast,async-slow,defer-first,defer-second"
    );
}

struct ParserLoader;

#[async_trait]
impl ResourceLoader for ParserLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://parser.test/" => (
                "text/html",
                r##"<!doctype html>
                <link rel="stylesheet" href="/style.css">
                <style>body { width: 50%; } #before { height: 100px; }</style>
                <div id="before"></div>
                <script>
                    document.body.setAttribute("data-inline-paused", document.querySelector("#after-inline") === null ? "yes" : "no");
                    document.body.setAttribute("data-style-width", document.querySelector("#before").getBoundingClientRect().width);
                    document.body.setAttribute("data-window-named", before === document.getElementById("before") ? "yes" : "no");
                    document.body.setAttribute("data-sheet-count", document.styleSheets.length);
                    document.body.setAttribute("data-inline-rule-count", document.styleSheets[1].cssRules.length);
                    const inserted = document.createElement("p");
                    inserted.id = "inserted-during-parse";
                    document.body.appendChild(inserted);
                </script>
                <div id="after-inline"></div>
                <script src="/blocking.js"></script>
                <div id="after-external"></div>"##,
            ),
            "https://parser.test/style.css" => ("text/css", "#before { width: 37px }"),
            "https://parser.test/blocking.js" => (
                "text/javascript",
                "document.body.setAttribute('data-external-paused', document.querySelector('#after-external') === null ? 'yes' : 'no')",
            ),
            other => panic!("unexpected parser resource: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn parser_pauses_for_blocking_scripts_and_resumes_the_same_dom() {
    let browser = Browser::with_resource_loader(Arc::new(ParserLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime.block_on(page.goto("https://parser.test/")).unwrap();

    assert_eq!(
        page.eval(
            r##"[
                document.body.getAttribute("data-inline-paused"),
                document.body.getAttribute("data-external-paused"),
                document.body.getAttribute("data-style-width"),
                document.body.getAttribute("data-window-named"),
                document.body.getAttribute("data-sheet-count"),
                document.body.getAttribute("data-inline-rule-count"),
                document.querySelector("#inserted-during-parse") !== null,
                document.querySelector("#after-inline") !== null,
                document.querySelector("#after-external") !== null
            ].join("|")"##,
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "yes|yes|37|yes|2|2|true|true|true"
    );
}

struct ThrowingScriptLoader;

#[async_trait]
impl ResourceLoader for ThrowingScriptLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://errors.test/" => (
                "text/html",
                r#"<!doctype html>
                <script>globalThis.beforeError = true; $R(1, 2);</script>
                <script src="/blocking.js"></script>
                <script src="/async.js" async></script>
                <script src="/defer.js" defer></script>
                <script>globalThis.afterErrors = true;</script>"#,
            ),
            "https://errors.test/blocking.js" => ("text/javascript", "missingBlockingGlobal()"),
            "https://errors.test/async.js" => ("text/javascript", "missingAsyncGlobal()"),
            "https://errors.test/defer.js" => ("text/javascript", "missingDeferredGlobal()"),
            other => panic!("unexpected script request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn uncaught_page_script_errors_do_not_fail_navigation() {
    let browser = Browser::with_resource_loader(Arc::new(ThrowingScriptLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime.block_on(page.goto("https://errors.test/")).unwrap();

    assert_eq!(
        page.eval("Number(beforeError && afterErrors)")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
    assert_eq!(page.load_state(), web_runtime::LoadState::Complete);
    assert!(
        page.eval("$R").is_err(),
        "the missing global remains absent"
    );
}
