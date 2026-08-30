use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{AutomationBrowser, AutomationError, PageOptions, RemoteArgument, TouchPoint};

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

struct InputLoader;

#[async_trait]
impl ResourceLoader for InputLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let body = br#"<!doctype html>
            <style>
              #click, #tap, #raw, #cancel-touch { display:block; width:120px; height:40px; }
              input { display:block; width:120px; height:30px; }
            </style>
            <button id='click'>Click</button>
            <input id='name'>
            <input id='check' type='checkbox'>
            <input id='cancel-check' type='checkbox'>
            <div id='tap'>Tap</div>
            <button id='raw'>Raw</button>
            <div id='cancel-touch'>Cancel touch</div>
            <script>
              globalThis.inputEvents = [];
              document.getElementById('cancel-check').addEventListener('click', event => event.preventDefault());
              for (const id of ['click', 'name', 'check', 'cancel-check', 'tap', 'raw', 'cancel-touch']) {
                const target = document.getElementById(id);
                for (const type of ['pointerover', 'pointerenter', 'pointermove', 'pointerdown',
                  'mousedown', 'focus', 'focusin', 'keydown', 'keypress', 'input', 'change',
                  'keyup', 'touchstart', 'touchmove', 'touchend', 'touchcancel', 'pointercancel',
                  'pointerup', 'mouseup', 'click']) {
                  target.addEventListener(type, event => inputEvents.push({
                    id, type, trusted: event.isTrusted,
                    ownTrusted: Object.hasOwn(event, 'isTrusted'),
                    pointerType: event.pointerType || '',
                    button: event.button ?? null, buttons: event.buttons ?? null,
                    shift: event.shiftKey, ctrl: event.ctrlKey,
                    value: 'value' in target ? target.value : null,
                  }));
                }
              }
            </script>"#;
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: body.to_vec(),
            effective_url: request.url,
        })
    }
}

#[test]
fn trusted_input_state_machine_dispatches_human_event_sequences_and_defaults() {
    let browser = AutomationBrowser::with_resource_loader(Arc::new(InputLoader));
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://input.test/", Duration::from_secs(1))
        .unwrap();

    page.hover("#click").unwrap();
    page.click("#click").unwrap();
    page.type_text("#name", "Brimp").unwrap();
    page.click("#check").unwrap();
    page.click("#cancel-check").unwrap();
    page.tap("#tap").unwrap();
    page.click("#name").unwrap();
    page.dispatch_key_event("keyDown", "Z", "KeyZ", "Z", false, 8)
        .unwrap();
    page.dispatch_key_event("keyUp", "Z", "KeyZ", "", false, 8)
        .unwrap();
    page.insert_text("!").unwrap();

    let raw_point = page
        .evaluate("(() => { const r = document.getElementById('raw').getBoundingClientRect(); return {x:r.left+r.width/2,y:r.top+r.height/2}; })()")
        .unwrap();
    let raw_x = raw_point["x"].as_f64().unwrap();
    let raw_y = raw_point["y"].as_f64().unwrap();
    page.dispatch_mouse_event("mouseMoved", raw_x, raw_y, 0, 0, 1, 8)
        .unwrap();
    page.dispatch_mouse_event("mousePressed", raw_x, raw_y, 0, 1, 1, 8)
        .unwrap();
    page.dispatch_mouse_event("mouseReleased", raw_x, raw_y, 0, 0, 1, 8)
        .unwrap();

    let cancel_point = page
        .evaluate("(() => { const r = document.getElementById('cancel-touch').getBoundingClientRect(); return {x:r.left+r.width/2,y:r.top+r.height/2}; })()")
        .unwrap();
    let cancel_x = cancel_point["x"].as_f64().unwrap();
    let cancel_y = cancel_point["y"].as_f64().unwrap();
    let touch = |x, y| TouchPoint {
        id: 7,
        x,
        y,
        radius_x: 2.0,
        radius_y: 3.0,
        rotation_angle: 0.0,
        force: 0.7,
        tangential_pressure: 0.0,
    };
    page.dispatch_touch_event("touchStart", vec![touch(cancel_x, cancel_y)], 2)
        .unwrap();
    page.dispatch_touch_event("touchMove", vec![touch(cancel_x + 20.0, cancel_y)], 2)
        .unwrap();
    page.dispatch_touch_event("touchCancel", vec![], 2).unwrap();

    let result = page
        .evaluate(
            r#"({
                allTrusted: inputEvents.every(event => event.trusted),
                noForgedOwnTrusted: inputEvents.every(event => !event.ownTrusted),
                scriptEventTrusted: new Event('script').isTrusted,
                trustedDescriptor: Object.getOwnPropertyDescriptor(Event.prototype, 'isTrusted'),
                click: inputEvents.filter(event => event.id === 'click').map(event => event.type),
                typed: document.getElementById('name').value,
                typeEvents: inputEvents.filter(event => event.id === 'name').map(event => event.type),
                checked: document.getElementById('check').checked,
                canceledChecked: document.getElementById('cancel-check').checked,
                checkEvents: inputEvents.filter(event => event.id === 'check').map(event => event.type),
                tapEvents: inputEvents.filter(event => event.id === 'tap').map(event => `${event.type}:${event.pointerType}`),
                rawEvents: inputEvents.filter(event => event.id === 'raw'),
                cancelTouchEvents: inputEvents.filter(event => event.id === 'cancel-touch'),
            })"#,
        )
        .unwrap();
    assert_eq!(result["allTrusted"], true);
    assert_eq!(result["noForgedOwnTrusted"], true);
    assert_eq!(result["scriptEventTrusted"], false);
    assert_eq!(result["trustedDescriptor"]["enumerable"], true);
    assert_eq!(result["trustedDescriptor"]["configurable"], false);
    assert_eq!(result["typed"], "BrimpZ!", "{result}");
    assert_eq!(result["checked"], true);
    assert_eq!(result["canceledChecked"], false);
    let click = result["click"].as_array().unwrap();
    assert!(click.iter().any(|event| event == "pointerdown"));
    assert!(click.iter().filter(|event| *event == "pointermove").count() >= 2);
    assert!(click.iter().any(|event| event == "mousedown"));
    assert!(click.iter().any(|event| event == "focus"));
    assert!(click.iter().any(|event| event == "click"));
    let typed = result["typeEvents"].as_array().unwrap();
    assert_eq!(typed.iter().filter(|event| *event == "keydown").count(), 6);
    assert_eq!(typed.iter().filter(|event| *event == "input").count(), 7);
    assert_eq!(typed.iter().filter(|event| *event == "keyup").count(), 6);
    let checked = result["checkEvents"].as_array().unwrap();
    assert!(checked.iter().any(|event| event == "input"));
    assert!(checked.iter().any(|event| event == "change"));
    let tapped = result["tapEvents"].as_array().unwrap();
    assert!(tapped.iter().any(|event| event == "touchstart:"));
    assert!(tapped.iter().any(|event| event == "touchend:"));
    assert!(tapped.iter().any(|event| event == "pointerdown:touch"));
    assert!(tapped.iter().any(|event| event == "click:touch"));
    let raw = result["rawEvents"].as_array().unwrap();
    assert!(raw.iter().all(|event| event["trusted"] == true));
    assert!(raw.iter().any(|event| event["type"] == "mousedown"
        && event["buttons"] == 1
        && event["shift"] == true));
    assert!(
        raw.iter()
            .any(|event| event["type"] == "click" && event["buttons"] == 0)
    );
    let canceled_touch = result["cancelTouchEvents"].as_array().unwrap();
    assert!(
        canceled_touch
            .iter()
            .any(|event| event["type"] == "touchmove" && event["ctrl"] == true)
    );
    assert!(
        canceled_touch
            .iter()
            .any(|event| event["type"] == "touchcancel")
    );
    assert!(
        canceled_touch
            .iter()
            .any(|event| event["type"] == "pointercancel")
    );
    assert!(!canceled_touch.iter().any(|event| event["type"] == "click"));

    assert!(matches!(
        page.click("#missing"),
        Err(AutomationError::InvalidInput(_))
    ));
    assert!(matches!(
        page.insert_text("not editable"),
        Err(AutomationError::InvalidInput(_))
    ));
}
