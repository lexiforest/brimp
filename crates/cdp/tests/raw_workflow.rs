use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use http::{HeaderValue, StatusCode};
use network::{HeaderList, NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use web_runtime::AutomationBrowser;

use brimp_cdp::{ServerConfig, ServerError, start_with_browser};

struct FixtureLoader;

#[async_trait]
impl ResourceLoader for FixtureLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        assert_eq!(
            request.headers.get("x-brimp-test"),
            Some(&HeaderValue::from_static("ready"))
        );
        assert_eq!(
            request.headers.get(http::header::USER_AGENT),
            Some(&HeaderValue::from_static("BrimpTest/1.0"))
        );
        assert_eq!(
            request.headers.get(http::header::ACCEPT_LANGUAGE),
            Some(&HeaderValue::from_static("zh-TW, en;q=0.8"))
        );
        let mut headers = HeaderList::new();
        headers.append("content-type", HeaderValue::from_static("text/html"));
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body: b"<!doctype html><title>CDP</title><main style='height:1200px'>Hello CDP</main>"
                .to_vec(),
            effective_url: request.url,
        })
    }
}

fn browser() -> Arc<AutomationBrowser> {
    Arc::new(AutomationBrowser::with_resource_loader(Arc::new(
        FixtureLoader,
    )))
}

#[tokio::test]
async fn discovery_and_raw_websocket_workflow() {
    let server = start_with_browser(ServerConfig::default(), browser())
        .await
        .unwrap();
    let response = http_get(server.local_addr(), "/json/version").await;
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains(&server.browser_websocket_url()));
    assert!(
        http_get(server.local_addr(), "/json/list")
            .await
            .ends_with("[]")
    );
    assert!(
        http_get(server.local_addr(), "/missing")
            .await
            .starts_with("HTTP/1.1 404 Not Found")
    );

    let (mut socket, _) = tokio_tungstenite::connect_async(server.browser_websocket_url())
        .await
        .unwrap();

    assert_eq!(
        command(&mut socket, 1, "Browser.getVersion", json!({}), None).await["result"]["product"],
        "Brimp/0.1.0"
    );
    command(
        &mut socket,
        2,
        "Target.setDiscoverTargets",
        json!({"discover": true}),
        None,
    )
    .await;
    command(&mut socket, 20, "Target.setAutoAttach", json!({"autoAttach": false, "waitForDebuggerOnStart": true, "flatten": true, "filter": [{"type": "page", "exclude": true}, {}]}), None).await;
    socket
        .send(Message::Text(
            json!({"id": 3, "method": "Target.createTarget", "params": {"url": "about:blank"}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    let created_event = event(&mut socket).await;
    assert_eq!(created_event["method"], "Target.targetCreated");
    let target_id = created_event["params"]["targetInfo"]["targetId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(created_event["params"]["targetInfo"]["targetId"], target_id);
    assert_eq!(event(&mut socket).await["result"]["targetId"], target_id);
    assert_eq!(
        command(&mut socket, 21, "Target.getTargets", json!({}), None).await["result"]["targetInfos"]
            [0]["targetId"],
        target_id
    );
    assert_eq!(
        command(
            &mut socket,
            22,
            "Target.getTargetInfo",
            json!({"targetId": target_id}),
            None
        )
        .await["result"]["targetInfo"]["type"],
        "page"
    );

    socket.send(Message::Text(json!({"id": 4, "method": "Target.attachToTarget", "params": {"targetId": target_id, "flatten": true}}).to_string().into())).await.unwrap();
    assert_eq!(
        event(&mut socket).await["method"],
        "Target.attachedToTarget"
    );
    let attached = event(&mut socket).await;
    assert_eq!(attached["id"], 4);
    let session = attached["result"]["sessionId"].as_str().unwrap().to_owned();
    command(&mut socket, 5, "Page.enable", json!({}), Some(&session)).await;
    command(
        &mut socket,
        6,
        "Page.setLifecycleEventsEnabled",
        json!({"enabled": true}),
        Some(&session),
    )
    .await;
    command(&mut socket, 7, "Runtime.enable", json!({}), Some(&session)).await;
    assert_eq!(
        event(&mut socket).await["method"],
        "Runtime.executionContextCreated"
    );
    command(
        &mut socket,
        43,
        "Page.createIsolatedWorld",
        json!({"frameId": target_id, "worldName": "test-world"}),
        Some(&session),
    )
    .await;
    let isolated_context = event(&mut socket).await;
    assert_eq!(
        isolated_context["method"],
        "Runtime.executionContextCreated"
    );
    assert_eq!(isolated_context["params"]["context"]["name"], "test-world");
    assert_eq!(
        isolated_context["params"]["context"]["auxData"]["isDefault"],
        false
    );
    let frame_tree = command(
        &mut socket,
        29,
        "Page.getFrameTree",
        json!({}),
        Some(&session),
    )
    .await;
    assert_eq!(frame_tree["result"]["frameTree"]["frame"]["id"], target_id);
    assert_eq!(
        frame_tree["result"]["frameTree"]["frame"]["url"],
        "about:blank"
    );
    command(&mut socket, 30, "Network.enable", json!({}), Some(&session)).await;
    command(
        &mut socket,
        59,
        "Emulation.setUserAgentOverride",
        json!({"userAgent": "BrimpTest/1.0", "acceptLanguage": "zh-TW, en;q=0.8", "platform": "TestOS"}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        49,
        "Network.setExtraHTTPHeaders",
        json!({"headers": {"x-brimp-test": "ready"}}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        31,
        "Emulation.setDeviceMetricsOverride",
        json!({"width": 640, "height": 480, "deviceScaleFactor": 2, "mobile": false}),
        Some(&session),
    )
    .await;
    let metrics = command(
        &mut socket,
        32,
        "Page.getLayoutMetrics",
        json!({}),
        Some(&session),
    )
    .await;
    assert_eq!(metrics["result"]["cssLayoutViewport"]["clientWidth"], 640);
    assert_eq!(metrics["result"]["cssLayoutViewport"]["clientHeight"], 480);

    let navigation = command(
        &mut socket,
        8,
        "Page.navigate",
        json!({"url": "https://fixture.test/"}),
        Some(&session),
    )
    .await;
    for expected in [
        "Network.requestWillBeSent",
        "Network.responseReceived",
        "Network.loadingFinished",
        "Runtime.executionContextsCleared",
        "Page.frameNavigated",
        "Runtime.executionContextCreated",
        "Runtime.executionContextCreated",
    ] {
        let notification = event(&mut socket).await;
        assert_eq!(notification["method"], expected);
        if expected == "Network.requestWillBeSent" {
            assert_eq!(
                notification["params"]["request"]["headers"]["x-brimp-test"],
                "ready"
            );
            assert_eq!(
                notification["params"]["request"]["headers"]["User-Agent"],
                "BrimpTest/1.0"
            );
        }
    }
    for expected in ["init", "DOMContentLoaded", "load"] {
        let lifecycle = event(&mut socket).await;
        assert_eq!(lifecycle["method"], "Page.lifecycleEvent");
        assert_eq!(lifecycle["params"]["name"], expected);
    }
    assert_eq!(
        event(&mut socket).await["method"],
        "Page.domContentEventFired"
    );
    assert_eq!(event(&mut socket).await["method"], "Page.loadEventFired");
    let response_body = command(
        &mut socket,
        50,
        "Network.getResponseBody",
        json!({"requestId": navigation["result"]["loaderId"]}),
        Some(&session),
    )
    .await;
    assert_eq!(response_body["result"]["base64Encoded"], false);
    assert!(
        response_body["result"]["body"]
            .as_str()
            .unwrap()
            .contains("Hello CDP")
    );
    let identity = command(
        &mut socket,
        60,
        "Runtime.evaluate",
        json!({"expression": "[navigator.userAgent, navigator.platform, navigator.language, navigator.languages, Object.hasOwn(navigator, 'userAgent'), Function.prototype.toString.call(Object.getOwnPropertyDescriptor(Navigator.prototype, 'userAgent').get)]", "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert_eq!(
        identity["result"]["result"]["value"],
        json!([
            "BrimpTest/1.0",
            "TestOS",
            "zh-TW",
            ["zh-TW", "en"],
            false,
            "function get userAgent() { [native code] }"
        ])
    );
    let document = command(
        &mut socket,
        61,
        "DOM.getDocument",
        json!({}),
        Some(&session),
    )
    .await;
    let document_node_id = document["result"]["root"]["nodeId"].as_u64().unwrap();
    let main_node_id = command(
        &mut socket,
        62,
        "DOM.querySelector",
        json!({"nodeId": document_node_id, "selector": "main"}),
        Some(&session),
    )
    .await["result"]["nodeId"]
        .as_u64()
        .unwrap();
    assert_ne!(main_node_id, 0);
    let all = command(
        &mut socket,
        63,
        "DOM.querySelectorAll",
        json!({"nodeId": document_node_id, "selector": "main, title"}),
        Some(&session),
    )
    .await;
    assert_eq!(all["result"]["nodeIds"].as_array().unwrap().len(), 2);
    let attributes = command(
        &mut socket,
        64,
        "DOM.getAttributes",
        json!({"nodeId": main_node_id}),
        Some(&session),
    )
    .await;
    assert_eq!(attributes["result"]["attributes"][0], "style");
    assert!(
        attributes["result"]["attributes"][1]
            .as_str()
            .unwrap()
            .contains("height")
    );

    let evaluated = command(
        &mut socket,
        9,
        "Runtime.evaluate",
        json!({"expression": "({answer: 6 * 7})", "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert_eq!(evaluated["result"]["result"]["value"]["answer"], 42);
    let called = command(&mut socket, 23, "Runtime.callFunctionOn", json!({"functionDeclaration": "(left, right) => ({sum: left + right})", "arguments": [{"value": 20}, {"value": 22}], "returnByValue": true}), Some(&session)).await;
    assert_eq!(called["result"]["result"]["value"]["sum"], 42);
    let handle = command(
        &mut socket,
        34,
        "Runtime.evaluate",
        json!({"expression": "({answer: 42, nested: {ok: true}})", "returnByValue": false, "objectGroup": "raw-test"}),
        Some(&session),
    )
    .await["result"]["result"]["objectId"]
        .as_str()
        .unwrap()
        .to_owned();
    let called_on_handle = command(
        &mut socket,
        35,
        "Runtime.callFunctionOn",
        json!({"functionDeclaration": "function () { return this.answer; }", "objectId": handle.clone(), "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert_eq!(called_on_handle["result"]["result"]["value"], 42);
    let node_handle = command(
        &mut socket,
        44,
        "Runtime.evaluate",
        json!({"expression": "document.querySelector('main')", "objectGroup": "node-test"}),
        Some(&session),
    )
    .await["result"]["result"]["objectId"]
        .as_str()
        .unwrap()
        .to_owned();
    let described = command(
        &mut socket,
        45,
        "DOM.describeNode",
        json!({"objectId": node_handle}),
        Some(&session),
    )
    .await;
    let backend_node_id = described["result"]["node"]["backendNodeId"]
        .as_u64()
        .unwrap();
    assert_eq!(described["result"]["node"]["nodeName"], "MAIN");
    let quads = command(
        &mut socket,
        51,
        "DOM.getContentQuads",
        json!({"backendNodeId": backend_node_id}),
        Some(&session),
    )
    .await;
    assert_eq!(quads["result"]["quads"][0].as_array().unwrap().len(), 8);
    let box_model = command(
        &mut socket,
        52,
        "DOM.getBoxModel",
        json!({"backendNodeId": backend_node_id}),
        Some(&session),
    )
    .await;
    assert!(box_model["result"]["model"]["width"].as_f64().unwrap() > 0.0);
    command(
        &mut socket,
        53,
        "DOM.scrollIntoViewIfNeeded",
        json!({"backendNodeId": backend_node_id}),
        Some(&session),
    )
    .await;
    let resolved = command(
        &mut socket,
        46,
        "DOM.resolveNode",
        json!({"backendNodeId": backend_node_id, "objectGroup": "node-test"}),
        Some(&session),
    )
    .await["result"]["object"]["objectId"]
        .as_str()
        .unwrap()
        .to_owned();
    let node_text = command(
        &mut socket,
        47,
        "Runtime.callFunctionOn",
        json!({"functionDeclaration": "function () { return this.textContent; }", "objectId": resolved, "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert_eq!(node_text["result"]["result"]["value"], "Hello CDP");
    let input_handle = command(
        &mut socket,
        54,
        "Runtime.evaluate",
        json!({"expression": r#"(() => {
            globalThis.inputEvents = [];
            const input = document.createElement('input');
            input.id = 'input';
            input.style.cssText = 'position:fixed;left:20px;top:20px;width:120px;height:30px';
            const button = document.createElement('button');
            button.id = 'button';
            button.textContent = 'click';
            button.style.cssText = 'position:fixed;left:20px;top:70px;width:120px;height:30px';
            const tap = document.createElement('button');
            tap.id = 'tap';
            tap.textContent = 'tap';
            tap.style.cssText = 'position:fixed;left:20px;top:120px;width:120px;height:30px';
            document.body.append(input, button, tap);
            for (const target of [input, button, tap]) {
                for (const type of ['input', 'keydown', 'keyup', 'pointerdown', 'mousedown',
                    'touchstart', 'touchend', 'pointerup', 'mouseup', 'click']) {
                    target.addEventListener(type, event => inputEvents.push({
                        id: target.id, type, trusted: event.isTrusted,
                        pointerType: event.pointerType || ''
                    }));
                }
            }
            return input;
        })()"#}),
        Some(&session),
    )
    .await["result"]["result"]["objectId"]
        .as_str()
        .unwrap()
        .to_owned();
    let input_node = command(
        &mut socket,
        55,
        "DOM.describeNode",
        json!({"objectId": input_handle.clone()}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        56,
        "DOM.focus",
        json!({"backendNodeId": input_node["result"]["node"]["backendNodeId"]}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        57,
        "Input.insertText",
        json!({"text": "typed"}),
        Some(&session),
    )
    .await;
    let input_value = command(
        &mut socket,
        58,
        "Runtime.callFunctionOn",
        json!({"functionDeclaration": "function () { return this.value; }", "objectId": input_handle, "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert_eq!(input_value["result"]["result"]["value"], "typed");
    command(
        &mut socket,
        90,
        "Input.dispatchKeyEvent",
        json!({"type": "keyDown", "key": "x", "text": "x"}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        91,
        "Input.dispatchKeyEvent",
        json!({"type": "keyUp", "key": "x"}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        92,
        "Input.dispatchMouseEvent",
        json!({"type": "mousePressed", "x": 40, "y": 80, "button": "left", "buttons": 1, "clickCount": 1}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        93,
        "Input.dispatchMouseEvent",
        json!({"type": "mouseReleased", "x": 40, "y": 80, "button": "left", "buttons": 0, "clickCount": 1}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        94,
        "Emulation.setTouchEmulationEnabled",
        json!({"enabled": true, "maxTouchPoints": 1}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        95,
        "Input.dispatchTouchEvent",
        json!({"type": "touchStart", "touchPoints": [{"id": 0, "x": 40, "y": 130}]}),
        Some(&session),
    )
    .await;
    command(
        &mut socket,
        96,
        "Input.dispatchTouchEvent",
        json!({"type": "touchEnd", "touchPoints": []}),
        Some(&session),
    )
    .await;
    let input_audit = command(
        &mut socket,
        97,
        "Runtime.evaluate",
        json!({"expression": "({value: document.querySelector('#input').value, events: inputEvents})", "returnByValue": true}),
        Some(&session),
    )
    .await;
    let audit = &input_audit["result"]["result"]["value"];
    assert_eq!(audit["value"], "typedx");
    let input_events = audit["events"].as_array().unwrap();
    assert!(input_events.iter().all(|event| event["trusted"] == true));
    assert!(
        input_events
            .iter()
            .any(|event| event["id"] == "button" && event["type"] == "click")
    );
    assert!(
        input_events
            .iter()
            .any(|event| event["id"] == "tap" && event["type"] == "touchstart")
    );
    assert!(input_events.iter().any(|event| event["id"] == "tap"
        && event["type"] == "click"
        && event["pointerType"] == "touch"));
    let properties = command(
        &mut socket,
        36,
        "Runtime.getProperties",
        json!({"objectId": handle, "ownProperties": true}),
        Some(&session),
    )
    .await;
    let descriptors = properties["result"]["result"].as_array().unwrap();
    assert_eq!(
        descriptors
            .iter()
            .find(|property| property["name"] == "answer")
            .unwrap()["value"]["value"],
        42
    );
    assert!(
        descriptors
            .iter()
            .find(|property| property["name"] == "nested")
            .unwrap()["value"]["objectId"]
            .is_string()
    );
    command(
        &mut socket,
        37,
        "Runtime.releaseObjectGroup",
        json!({"objectGroup": "raw-test"}),
        Some(&session),
    )
    .await;
    let released = command(
        &mut socket,
        38,
        "Runtime.callFunctionOn",
        json!({"functionDeclaration": "function () { return this.answer; }", "objectId": handle, "returnByValue": true}),
        Some(&session),
    )
    .await;
    assert!(
        released["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown remote object")
    );
    let history = command(
        &mut socket,
        39,
        "Page.getNavigationHistory",
        json!({}),
        Some(&session),
    )
    .await;
    assert_eq!(history["result"]["currentIndex"], 1);
    assert_eq!(history["result"]["entries"].as_array().unwrap().len(), 2);
    let blank_entry = history["result"]["entries"][0]["id"].as_u64().unwrap();
    command(
        &mut socket,
        40,
        "Page.navigateToHistoryEntry",
        json!({"entryId": blank_entry}),
        Some(&session),
    )
    .await;
    for _ in 0..12 {
        event(&mut socket).await;
    }
    let history = command(
        &mut socket,
        41,
        "Page.getNavigationHistory",
        json!({}),
        Some(&session),
    )
    .await;
    assert_eq!(history["result"]["currentIndex"], 0);
    assert_eq!(history["result"]["entries"].as_array().unwrap().len(), 2);
    let screenshot = command(
        &mut socket,
        10,
        "Page.captureScreenshot",
        json!({"format": "png"}),
        Some(&session),
    )
    .await;
    let png = base64::engine::general_purpose::STANDARD
        .decode(screenshot["result"]["data"].as_str().unwrap())
        .unwrap();
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 1280);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 960);
    let full_page = command(
        &mut socket,
        42,
        "Page.captureScreenshot",
        json!({"format": "png", "captureBeyondViewport": true}),
        Some(&session),
    )
    .await;
    let full_page_png = base64::engine::general_purpose::STANDARD
        .decode(full_page["result"]["data"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        u32::from_be_bytes(full_page_png[16..20].try_into().unwrap()),
        1280
    );
    assert!(u32::from_be_bytes(full_page_png[20..24].try_into().unwrap()) > 960);

    assert!(
        command(&mut socket, 11, "Log.enable", json!({}), Some(&session)).await["result"]
            .is_object()
    );
    let unknown = command(
        &mut socket,
        33,
        "DefinitelyMissing.enable",
        json!({}),
        Some(&session),
    )
    .await;
    assert_eq!(unknown["error"]["code"], -32601);
    command(
        &mut socket,
        12,
        "Target.closeTarget",
        json!({"targetId": target_id}),
        None,
    )
    .await;
    assert_eq!(
        event(&mut socket).await["method"],
        "Target.detachedFromTarget"
    );
    assert_eq!(event(&mut socket).await["method"], "Target.targetDestroyed");

    command(&mut socket, 24, "Target.setAutoAttach", json!({"autoAttach": true, "waitForDebuggerOnStart": true, "flatten": true, "filter": [{}]}), None).await;
    socket
        .send(Message::Text(
            json!({"id": 25, "method": "Target.createTarget", "params": {"url": "about:blank"}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    let auto_created = event(&mut socket).await;
    assert_eq!(auto_created["method"], "Target.targetCreated");
    let auto_target = auto_created["params"]["targetInfo"]["targetId"]
        .as_str()
        .unwrap()
        .to_owned();
    let auto_attached = event(&mut socket).await;
    assert_eq!(auto_attached["method"], "Target.attachedToTarget");
    assert_eq!(auto_attached["params"]["waitingForDebugger"], true);
    let auto_session = auto_attached["params"]["sessionId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(event(&mut socket).await["result"]["targetId"], auto_target);
    command(
        &mut socket,
        26,
        "Runtime.runIfWaitingForDebugger",
        json!({}),
        Some(&auto_session),
    )
    .await;
    command(
        &mut socket,
        27,
        "Target.detachFromTarget",
        json!({"sessionId": auto_session}),
        None,
    )
    .await;
    assert_eq!(
        event(&mut socket).await["method"],
        "Target.detachedFromTarget"
    );
    command(
        &mut socket,
        28,
        "Target.closeTarget",
        json!({"targetId": auto_target}),
        None,
    )
    .await;
    assert_eq!(event(&mut socket).await["method"], "Target.targetDestroyed");
    socket.close(None).await.unwrap();
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn malformed_and_oversized_messages_are_rejected() {
    let server = start_with_browser(ServerConfig::default(), browser())
        .await
        .unwrap();
    let (mut socket, _) = tokio_tungstenite::connect_async(server.browser_websocket_url())
        .await
        .unwrap();
    socket.send(Message::Text("{".into())).await.unwrap();
    let parse_error = event(&mut socket).await;
    assert_eq!(parse_error["id"], 0);
    assert_eq!(parse_error["error"]["code"], -32700);
    let rejected = match socket
        .send(Message::Text("x".repeat(1024 * 1024 + 1).into()))
        .await
    {
        Err(_) => true,
        Ok(()) => socket.next().await.is_some_and(|message| message.is_err()),
    };
    assert!(rejected);
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn disconnect_discards_the_connection_target_registry() {
    let server = start_with_browser(ServerConfig::default(), browser())
        .await
        .unwrap();
    let (mut first, _) = tokio_tungstenite::connect_async(server.browser_websocket_url())
        .await
        .unwrap();
    command(
        &mut first,
        1,
        "Target.createTarget",
        json!({"url": "about:blank"}),
        None,
    )
    .await;
    first.close(None).await.unwrap();
    let (mut second, _) = tokio_tungstenite::connect_async(server.browser_websocket_url())
        .await
        .unwrap();
    assert_eq!(
        command(&mut second, 2, "Target.getTargets", json!({}), None).await["result"]["targetInfos"],
        json!([])
    );
    second.close(None).await.unwrap();
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn non_loopback_bind_requires_explicit_permission() {
    let result = start_with_browser(
        ServerConfig {
            bind: "0.0.0.0:0".parse().unwrap(),
            allow_non_loopback: false,
            ..ServerConfig::default()
        },
        browser(),
    )
    .await;
    assert!(matches!(result, Err(ServerError::NonLoopback(_))));
}

async fn command(
    socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    id: u64,
    method: &str,
    params: Value,
    session: Option<&str>,
) -> Value {
    let mut request = json!({"id": id, "method": method, "params": params});
    if let Some(session) = session {
        request["sessionId"] = json!(session);
    }
    socket
        .send(Message::Text(request.to_string().into()))
        .await
        .unwrap();
    let value = event(socket).await;
    assert_eq!(value["id"], id);
    value
}

async fn event(
    socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
) -> Value {
    let message = socket.next().await.unwrap().unwrap();
    serde_json::from_str(message.to_text().unwrap()).unwrap()
}

async fn http_get(address: std::net::SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8(response).unwrap()
}
