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
        let mut headers = HeaderList::new();
        headers.append("content-type", HeaderValue::from_static("text/html"));
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body: b"<!doctype html><title>CDP</title><main>Hello CDP</main>".to_vec(),
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
    command(&mut socket, 20, "Target.setAutoAttach", json!({"autoAttach": true, "waitForDebuggerOnStart": true, "flatten": true, "filter": [{"type": "page", "exclude": true}, {}]}), None).await;
    let created = command(
        &mut socket,
        3,
        "Target.createTarget",
        json!({"url": "about:blank"}),
        None,
    )
    .await;
    let target_id = created["result"]["targetId"].as_str().unwrap().to_owned();
    let created_event = event(&mut socket).await;
    assert_eq!(created_event["method"], "Target.targetCreated");
    assert_eq!(created_event["params"]["targetInfo"]["targetId"], target_id);
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
        8,
        "Page.navigate",
        json!({"url": "https://fixture.test/"}),
        Some(&session),
    )
    .await;
    for expected in ["init", "DOMContentLoaded", "load"] {
        let lifecycle = event(&mut socket).await;
        assert_eq!(lifecycle["method"], "Page.lifecycleEvent");
        assert_eq!(lifecycle["params"]["name"], expected);
    }
    assert_eq!(event(&mut socket).await["method"], "Page.loadEventFired");

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

    let unknown = command(&mut socket, 11, "Log.enable", json!({}), Some(&session)).await;
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
    let auto_target = command(
        &mut socket,
        25,
        "Target.createTarget",
        json!({"url": "about:blank"}),
        None,
    )
    .await["result"]["targetId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(event(&mut socket).await["method"], "Target.targetCreated");
    let auto_attached = event(&mut socket).await;
    assert_eq!(auto_attached["method"], "Target.attachedToTarget");
    assert_eq!(auto_attached["params"]["waitingForDebugger"], true);
    let auto_session = auto_attached["params"]["sessionId"]
        .as_str()
        .unwrap()
        .to_owned();
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
