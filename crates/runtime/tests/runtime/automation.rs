use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use brimp_runtime::network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use brimp_runtime::runtime::{AutomationError, PageOptions, Session};
use http::{HeaderMap, HeaderValue, StatusCode};
use serde_json::json;

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
            metadata: brimp_runtime::network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn reset_replaces_the_document_and_javascript_realm_without_closing_the_page() {
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(WorkflowLoader)).unwrap();
    let page = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    page.navigate("https://example.test/", Duration::from_secs(1))
        .unwrap();
    page.evaluate("globalThis.leaked = 42").unwrap();

    page.reset().unwrap();

    assert_eq!(page.evaluate("location.href").unwrap(), "about:blank");
    assert_eq!(page.evaluate("typeof leaked").unwrap(), "undefined");
    assert_eq!(page.title().unwrap(), "");
    page.navigate("https://example.test/reused", Duration::from_secs(1))
        .unwrap();
    assert_eq!(page.title().unwrap(), "Workflow");
}

#[test]
fn load_html_uses_the_given_url_without_running_page_scripts() {
    let browser = Session::new(Default::default()).unwrap();
    let page = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    page.evaluate("globalThis.beforeLoad = 1").unwrap();
    page.add_preload_script(
        "init",
        "globalThis.fromInit = location.href; globalThis.initReady = document.readyState; \
         globalThis.sawLoad = false; addEventListener('load', () => { globalThis.sawLoad = true; })",
    )
    .unwrap();
    page.load_html(
        r#"<!doctype html><title>Saved</title>
           <base href="../assets/">
           <a id="next" href="next?x=1">next</a>
           <script>globalThis.pageScriptRan = true</script>"#,
        "https://example.test/dir/page.html",
    )
    .unwrap();

    assert_eq!(page.title().unwrap(), "Saved");
    assert_eq!(
        page.evaluate("location.href").unwrap(),
        json!("https://example.test/dir/page.html")
    );
    assert_eq!(
        page.evaluate("document.URL").unwrap(),
        json!("https://example.test/dir/page.html")
    );
    assert_eq!(
        page.evaluate("document.baseURI").unwrap(),
        json!("https://example.test/assets/")
    );
    assert_eq!(
        page.evaluate("document.getElementById('next').href")
            .unwrap(),
        json!("https://example.test/assets/next?x=1")
    );
    assert_eq!(
        page.evaluate("typeof globalThis.pageScriptRan").unwrap(),
        json!("undefined")
    );
    assert_eq!(
        page.evaluate("typeof globalThis.beforeLoad").unwrap(),
        json!("undefined")
    );
    assert_eq!(
        page.evaluate("globalThis.fromInit").unwrap(),
        json!("https://example.test/dir/page.html")
    );
    assert_eq!(page.evaluate("globalThis.sawLoad").unwrap(), json!(true));
    assert_eq!(
        page.evaluate("document.readyState").unwrap(),
        json!("complete")
    );
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
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(HangingLoader)).unwrap();
    let page = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    assert!(matches!(
        page.navigate("https://example.test/", Duration::from_millis(20)),
        Err(AutomationError::Timeout(_))
    ));
    page.close();
}

struct LoopingScriptLoader;

#[async_trait]
impl ResourceLoader for LoopingScriptLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        let body = if request.url.ends_with("/loop") {
            "<!doctype html><script>while (true) {}</script>"
        } else {
            "<!doctype html><title>Recovered</title>"
        };
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
            metadata: brimp_runtime::network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn navigation_timeout_interrupts_non_yielding_javascript() {
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(LoopingScriptLoader)).unwrap();
    let page = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    let started = std::time::Instant::now();
    let result = page.navigate("https://loop.test/loop", Duration::from_millis(50));
    assert!(
        matches!(result, Err(AutomationError::Timeout(_))),
        "unexpected navigation result: {result:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
    page.reset().unwrap();
    page.navigate("https://loop.test/recovered", Duration::from_secs(1))
        .unwrap();
    assert_eq!(page.title().unwrap(), "Recovered");
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
            metadata: brimp_runtime::network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn navigation_returns_raw_and_rendered_responses_without_raising_for_http_status() {
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(ResponseLoader)).unwrap();
    let page = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();

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

#[derive(Default)]
struct CookieLoader {
    observed: std::sync::Mutex<Vec<(String, Option<String>)>>,
}

#[async_trait]
impl ResourceLoader for CookieLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        self.observed.lock().unwrap().push((
            request.url.to_string(),
            request
                .headers
                .get(http::header::COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
        ));
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        if request.url.ends_with("/set") {
            headers.insert(
                http::header::SET_COOKIE,
                HeaderValue::from_static("shared=yes; Path=/"),
            );
        }
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: b"<!doctype html><title>Cookies</title>".to_vec(),
            effective_url: request.url,
            metadata: brimp_runtime::network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn session_closure_closes_pages_and_new_session_isolates_cookies() {
    let loader = Arc::new(CookieLoader::default());
    let browser = Session::with_resource_loader(Default::default(), loader.clone()).unwrap();
    let first = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    first
        .navigate("https://cookies.test/set", Duration::from_secs(1))
        .unwrap();
    let second = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    second
        .navigate("https://cookies.test/shared", Duration::from_secs(1))
        .unwrap();

    browser.close();
    let browser = Session::with_resource_loader(Default::default(), loader.clone()).unwrap();
    assert!(matches!(first.evaluate("1"), Err(AutomationError::Closed)));
    assert!(matches!(second.evaluate("1"), Err(AutomationError::Closed)));
    let isolated = browser
        .new_page_with_options(PageOptions::default())
        .unwrap();
    isolated
        .navigate("https://cookies.test/isolated", Duration::from_secs(1))
        .unwrap();

    let observed = loader.observed.lock().unwrap();
    assert_eq!(observed[0].1, None);
    assert_eq!(observed[1].1.as_deref(), Some("shared=yes"));
    assert_eq!(observed[2].1, None);
}

fn proxy_fixture(
    marker: &'static str,
) -> (
    String,
    std::sync::mpsc::Receiver<Vec<String>>,
    std::thread::JoinHandle<()>,
) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let route = marker.chars().last().unwrap().to_ascii_lowercase();
        let mut observed = Vec::new();
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 8192];
            let read = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..read]).into_owned();
            let first_line = request.lines().next().unwrap_or_default().to_owned();
            observed.push(request);
            if first_line.contains(&format!("/{route} ")) {
                write!(
                    stream,
                    "HTTP/1.1 302 Found\r\nLocation: http://agent.invalid/{route}-final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .unwrap();
            } else if first_line.contains(&format!("/{route}-final ")) {
                let body = format!(
                    "<!doctype html><title>{marker}</title><script src='/{route}.js'></script>"
                );
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            } else {
                let body = "globalThis.proxyScript = true";
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        }
        sender.send(observed).unwrap();
    });
    (format!("http://{address}"), receiver, worker)
}

#[test]
fn concurrent_pages_pin_distinct_proxies_and_share_context_cookies() {
    let (proxy_a, observed_a, worker_a) = proxy_fixture("Proxy A");
    let (proxy_b, observed_b, worker_b) = proxy_fixture("Proxy B");
    let browser = Session::with_profile_and_network_config(
        brimp_runtime::profile::BrowserProfile::default(),
        brimp_runtime::network::CurlConfig::default(),
    )
    .unwrap();
    let context = browser.cookies();
    context
        .set_cookie("http://agent.invalid/", "shared", "yes")
        .unwrap();
    let page_a = browser
        .new_page_with_proxy(PageOptions::default(), Some(&proxy_a))
        .unwrap();
    let page_b = browser
        .new_page_with_proxy(PageOptions::default(), Some(&proxy_b))
        .unwrap();

    let navigation_a = std::thread::spawn(move || {
        page_a.navigate("http://agent.invalid/a", Duration::from_secs(5))
    });
    let navigation_b = std::thread::spawn(move || {
        page_b.navigate("http://agent.invalid/b", Duration::from_secs(5))
    });
    let response_a = navigation_a.join().unwrap().unwrap();
    let response_b = navigation_b.join().unwrap().unwrap();
    worker_a.join().unwrap();
    worker_b.join().unwrap();

    assert!(response_a.html.unwrap().contains("Proxy A"));
    assert!(response_b.html.unwrap().contains("Proxy B"));
    let requests_a = observed_a.recv().unwrap();
    let requests_b = observed_b.recv().unwrap();
    for (route, requests) in [('a', requests_a), ('b', requests_b)] {
        let requests = requests
            .into_iter()
            .map(|request| request.to_ascii_lowercase())
            .collect::<Vec<_>>();
        assert!(requests[0].starts_with(&format!("get http://agent.invalid/{route} ")));
        assert!(requests[1].starts_with(&format!("get http://agent.invalid/{route}-final ")));
        assert!(requests[2].starts_with(&format!("get http://agent.invalid/{route}.js ")));
        assert!(
            requests
                .iter()
                .all(|request| request.contains("cookie: shared=yes"))
        );
    }
}

#[test]
fn navigator_locks_queue_query_abort_and_release_real_operations() {
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(WorkflowLoader)).unwrap();
    let page = browser
        .new_page_with_options(
            PageOptions::builder()
                .plugins(vec![brimp_runtime::runtime::plugins::bundled::environment(
                    serde_json::json!({}),
                )])
                .build(),
        )
        .unwrap();
    page.navigate("https://locks.test/", Duration::from_secs(1))
        .unwrap();

    let result = page
        .evaluate(
            r#"(async () => {
                const shape = {
                    manager: navigator.locks instanceof LockManager,
                    sameManager: navigator.locks === navigator.locks,
                    webdriver: navigator.webdriver,
                    webdriverOwn: Object.hasOwn(navigator, "webdriver"),
                    native: LockManager.prototype.request.toString(),
                    lengths: [UserActivation.length, Lock.length, LockManager.length,
                        LockManager.prototype.request.length],
                    illegal: [Lock, LockManager, UserActivation].every(constructor => {
                        try { new constructor(); return false; }
                        catch (error) { return error instanceof TypeError && error.message === "Illegal constructor"; }
                    }),
                };

                const order = [];
                let releaseFirst;
                const first = navigator.locks.request("resource", lock => {
                    order.push(`${lock.name}:${lock.mode}:first`);
                    return new Promise(resolve => { releaseFirst = resolve; });
                });
                await Promise.resolve();
                const second = navigator.locks.request("resource", lock => {
                    order.push(`${lock.name}:${lock.mode}:second`);
                    return "second-result";
                });
                const unavailable = await navigator.locks.request(
                    "resource",
                    { ifAvailable: true },
                    lock => lock === null,
                );
                const queued = await navigator.locks.query();
                releaseFirst("first-result");
                const exclusiveResults = await Promise.all([first, second]);

                let releaseShared;
                const sharedGate = new Promise(resolve => { releaseShared = resolve; });
                const sharedA = navigator.locks.request("shared", { mode: "shared" }, lock => {
                    order.push(`${lock.name}:${lock.mode}:a`);
                    return sharedGate;
                });
                const sharedB = navigator.locks.request("shared", { mode: "shared" }, lock => {
                    order.push(`${lock.name}:${lock.mode}:b`);
                    return sharedGate;
                });
                await Promise.resolve();
                await Promise.resolve();
                const sharedSnapshot = await navigator.locks.query();
                const sharedAvailable = await navigator.locks.request(
                    "shared",
                    { mode: "shared", ifAvailable: true },
                    lock => lock?.mode,
                );
                releaseShared("shared-result");
                await Promise.all([sharedA, sharedB]);

                let releaseBlocker, blockerStarted;
                const started = new Promise(resolve => { blockerStarted = resolve; });
                const blocker = navigator.locks.request("abortable", () =>
                    new Promise(resolve => { releaseBlocker = resolve; blockerStarted(); }));
                await started;
                const controller = new AbortController();
                const aborted = navigator.locks.request(
                    "abortable",
                    { signal: controller.signal },
                    () => "not-run",
                ).catch(error => error.name);
                controller.abort();
                releaseBlocker();
                await blocker;

                return {
                    shape,
                    order,
                    unavailable,
                    exclusiveResults,
                    queued: {
                        held: queued.held.map(lock => `${lock.name}:${lock.mode}`),
                        pending: queued.pending.map(lock => `${lock.name}:${lock.mode}`),
                    },
                    sharedHeld: sharedSnapshot.held.filter(lock => lock.name === "shared").length,
                    sharedAvailable,
                    aborted: await aborted,
                    final: await navigator.locks.query(),
                };
            })()"#,
        )
        .unwrap();
    let value = &result;
    assert_eq!(value["shape"]["manager"], true);
    assert_eq!(value["shape"]["sameManager"], true);
    assert_eq!(value["shape"]["webdriver"], false);
    assert_eq!(value["shape"]["webdriverOwn"], false);
    assert_eq!(value["shape"]["illegal"], true);
    assert_eq!(value["shape"]["lengths"], serde_json::json!([0, 0, 0, 2]));
    assert_eq!(value["unavailable"], true);
    assert_eq!(
        value["exclusiveResults"],
        serde_json::json!(["first-result", "second-result"])
    );
    assert_eq!(
        value["queued"],
        serde_json::json!({
            "held": ["resource:exclusive"],
            "pending": ["resource:exclusive"],
        })
    );
    assert_eq!(value["sharedHeld"], 2);
    assert_eq!(value["sharedAvailable"], "shared");
    assert_eq!(value["aborted"], "AbortError");
    assert_eq!(
        value["final"],
        serde_json::json!({"held": [], "pending": []})
    );
}

#[test]
fn local_and_managed_pages_share_setup_and_have_explicit_lifetimes() {
    let browser =
        Session::with_resource_loader(Default::default(), Arc::new(WorkflowLoader)).unwrap();
    let options = PageOptions::default();
    let mut local = browser.new_local_page(options.clone()).unwrap();
    let managed = browser.new_page_with_options(options.clone()).unwrap();
    browser
        .cookies()
        .set_cookie("https://example.test/", "shared", "yes")
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime
        .block_on(local.goto("https://example.test/"))
        .unwrap();
    managed
        .navigate("https://example.test/", Duration::from_secs(1))
        .unwrap();
    assert!(
        local
            .eval("document.cookie")
            .unwrap()
            .to_string()
            .unwrap()
            .contains("shared=yes")
    );
    assert!(
        managed
            .evaluate("document.cookie")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("shared=yes")
    );
    browser.close();
    assert!(matches!(
        managed.evaluate("1"),
        Err(AutomationError::Closed)
    ));
    assert!(matches!(
        browser.new_page_with_options(options.clone()),
        Err(AutomationError::Closed)
    ));
    assert!(matches!(
        browser.new_local_page(options),
        Err(AutomationError::Closed)
    ));
    assert_eq!(local.eval("6 * 7").unwrap().to_number().unwrap(), 42.0);
}
