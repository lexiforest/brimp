use std::sync::{Arc, Mutex};

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
                <script>globalThis.order = ["inline-start"]; globalThis.currentScripts = [document.currentScript === document.scripts[0]];</script>
                <script src="/defer-first.js" defer></script>
                <script src="/defer-second.js" defer></script>
                <script src="/async-slow.js" async></script>
                <script src="/async-fast.js" async></script>
                <script defer>order.push("inline-attribute"); currentScripts.push(document.currentScript === document.scripts[5])</script>
                <script type="module">order.push("module"); currentScripts.push(document.currentScript === null)</script>
                <script type="application/json">order.push("data")</script>
                <script src="/blocking.js"></script>
                <iframe id="child-frame" name="namedFrame"></iframe>"#,
            ),
            "https://example.test/defer-first.js" => {
                yield_times(5).await;
                (
                    "text/javascript",
                    "order.push('defer-first'); currentScripts.push(document.currentScript.src.endsWith('/defer-first.js'))",
                )
            }
            "https://example.test/defer-second.js" => {
                yield_times(1).await;
                (
                    "text/javascript",
                    "order.push('defer-second'); currentScripts.push(document.currentScript.src.endsWith('/defer-second.js'))",
                )
            }
            "https://example.test/async-slow.js" => {
                yield_times(4).await;
                (
                    "text/javascript",
                    "order.push('async-slow'); currentScripts.push(document.currentScript.src.endsWith('/async-slow.js'))",
                )
            }
            "https://example.test/async-fast.js" => {
                yield_times(1).await;
                (
                    "text/javascript",
                    "order.push('async-fast'); currentScripts.push(document.currentScript.src.endsWith('/async-fast.js'))",
                )
            }
            "https://example.test/blocking.js" => (
                "text/javascript",
                "order.push('blocking'); currentScripts.push(document.currentScript.src.endsWith('/blocking.js'))",
            ),
            other => panic!("unexpected script request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
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
fn classic_and_module_scripts_follow_their_execution_ordering_rules() {
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
        "inline-start,inline-attribute,blocking,async-fast,async-slow,defer-first,defer-second,module"
    );
    assert_eq!(
        page.eval("currentScripts.every(Boolean) && document.currentScript === null")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
    assert_eq!(
        page.eval("Number(window.frames === window && window.length === 1 && window[0] === document.querySelector('iframe').contentWindow && namedFrame === window[0])")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
    assert_eq!(
        page.eval("document.querySelector('iframe').remove(); Number(window.length === 0 && window[0] === undefined)")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
    assert_eq!(
        page.eval("document.domain = 'example.test'; Number(document.domain === 'example.test')")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
    assert!(page.eval("document.domain = 'invalid.test'").is_err());
}

struct ModuleLoader;

#[async_trait]
impl ResourceLoader for ModuleLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://modules.test/" => (
                "text/html",
                r#"<!doctype html>
                <script type="module" src="/main.js"></script>
                <main id="content">module host</main>"#,
            ),
            "https://modules.test/main.js" => (
                "text/javascript",
                r#"import { value } from "./dependency.js";
                window.moduleResult = `${value}|${import.meta.url}|${document.body.id}`;
                import("./lazy.js").then(module => window.lazyResult = module.default);
                window.dynamicEvents = [];
                const script = document.createElement("script");
                script.type = "module";
                script.src = "./inserted.js";
                script.onload = () => dynamicEvents.push("load");
                document.body.appendChild(script);
                window.dynamicClassic = [];
                const inlineClassic = document.createElement("script");
                inlineClassic.id = "inline-classic";
                inlineClassic.textContent = "dynamicClassic.push(document.currentScript === document.getElementById('inline-classic'))";
                document.body.appendChild(inlineClassic);
                const externalClassic = document.createElement("script");
                externalClassic.id = "external-classic";
                externalClassic.src = "./inserted-classic.js";
                document.body.appendChild(externalClassic);"#,
            ),
            "https://modules.test/dependency.js" => {
                ("text/javascript", "export const value = 41 + 1;")
            }
            "https://modules.test/lazy.js" => ("text/javascript", "export default 'loaded';"),
            "https://modules.test/inserted.js" => (
                "text/javascript",
                "window.dynamicModuleResult = 'inserted';",
            ),
            "https://modules.test/inserted-classic.js" => (
                "text/javascript",
                "dynamicClassic.push(document.currentScript === document.getElementById('external-classic'))",
            ),
            other => panic!("unexpected module request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
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
fn module_scripts_load_their_graph_and_support_dynamic_imports() {
    let browser = Browser::with_resource_loader(Arc::new(ModuleLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://modules.test/"))
        .unwrap();

    assert_eq!(
        page.eval("moduleResult").unwrap().to_string().unwrap(),
        "42|https://modules.test/main.js|"
    );
    assert_eq!(
        page.eval("lazyResult").unwrap().to_string().unwrap(),
        "loaded"
    );
    assert_eq!(
        page.eval("`${dynamicModuleResult}|${dynamicEvents.join(',')}`")
            .unwrap()
            .to_string()
            .unwrap(),
        "inserted|load"
    );
    assert_eq!(
        page.eval("Number(dynamicClassic.length === 2 && dynamicClassic.every(Boolean) && document.currentScript === null)")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
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
            metadata: network::ResponseMetadata::default(),
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
        if request.url.as_str() == "https://errors.test/network-error.js" {
            return Err(NetworkError::Transport("deliberate script failure".into()));
        }
        let (content_type, body) = match request.url.as_str() {
            "https://errors.test/" => (
                "text/html",
                r#"<!doctype html>
                <script>globalThis.beforeError = true; $R(1, 2);</script>
                <script src="/blocking.js"></script>
                <script src="http://[invalid"></script>
                <script src="/network-error.js"></script>
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
            metadata: network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn uncaught_page_script_errors_do_not_fail_navigation() {
    let browser = Browser::with_resource_loader(Arc::new(ThrowingScriptLoader));
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&errors);
    page.set_console_callback(move |message| {
        captured.lock().unwrap().push(message.to_owned());
    })
    .unwrap();
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
    let errors = errors.lock().unwrap();
    assert!(errors.iter().any(|error| error.contains("$R")));
    assert!(
        errors
            .iter()
            .any(|error| error.contains("missingBlockingGlobal"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("missingAsyncGlobal"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("missingDeferredGlobal"))
    );
    assert!(errors.iter().any(|error| error.contains("invalid")));
    assert!(
        errors
            .iter()
            .any(|error| error.contains("deliberate script failure"))
    );
}
