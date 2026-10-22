use async_trait::async_trait;
use brimp_runtime::{
    network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse},
    runtime::{
        events::PageEvent,
        native::{RequestOptions, Session, SessionOptions},
        networking::NetworkOptions,
        routes::{RouteOverrides, RouteStage},
    },
};
use serde_json::json;
use std::sync::{Arc, Mutex, mpsc};
#[derive(Default)]
struct Fixture {
    requests: Mutex<Vec<ResourceRequest>>,
}
#[async_trait]
impl ResourceLoader for Fixture {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        self.requests.lock().unwrap().push(request.clone());
        let (status, body) = if request.url.ends_with("/auth") && request.auth.is_empty() {
            (401, b"challenge".to_vec())
        } else if request.url.ends_with("/redirect") {
            (302, Vec::new())
        } else {
            (200,b"<html><body><p>network</p><script>console.warn('page message'); document.querySelector('p').setAttribute('data-test','yes')</script></body></html>".to_vec())
        };
        let mut headers = brimp_runtime::network::HeaderList::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/html"),
        );
        headers.insert(
            http::header::CACHE_CONTROL,
            http::HeaderValue::from_static("max-age=60"),
        );
        if status == 401 {
            headers.insert(
                http::header::WWW_AUTHENTICATE,
                http::HeaderValue::from_static("Basic realm=\"fixture\""),
            );
        }
        if status == 302 {
            headers.insert(
                http::header::LOCATION,
                http::HeaderValue::from_static("/final"),
            );
        }
        Ok(ResourceResponse {
            status: http::StatusCode::from_u16(status).unwrap(),
            headers,
            body,
            effective_url: request.url,
            metadata: Default::default(),
        })
    }
}
#[test]
fn typed_events_routes_auth_cache_headers_and_retained_bodies() {
    let fixture = Arc::new(Fixture::default());
    let session =
        Session::with_resource_loader(SessionOptions::default(), fixture.clone()).unwrap();
    let page = session.new_page().unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut subscriptions = Vec::new();
    for name in [
        "request",
        "response",
        "request_finished",
        "console",
        "mutation",
        "lifecycle",
    ] {
        let events = events.clone();
        subscriptions.push(
            page.on(
                name,
                Arc::new(move |event| events.lock().unwrap().push(event)),
            )
            .unwrap(),
        );
    }
    let route = page
        .route(
            "https://example.test/*",
            RouteStage::Request,
            Arc::new(|route| {
                route
                    .continue_request(RouteOverrides {
                        headers: Some(vec![("x-route".into(), "yes".into())]),
                        ..Default::default()
                    })
                    .unwrap();
                assert!(route.fail("twice").is_err());
            }),
        )
        .unwrap();
    page.request(
        "GET",
        "https://example.test/page",
        RequestOptions::default(),
    )
    .unwrap();
    assert_eq!(
        fixture.requests.lock().unwrap()[0]
            .headers
            .get("x-route")
            .unwrap(),
        "yes"
    );
    let records = events.lock().unwrap().clone();
    assert!(matches!(records[0], PageEvent::Lifecycle { .. }));
    assert!(records.iter().any(|event|matches!(event,PageEvent::Console{level,message} if level=="warn"&&message=="page message")));
    assert!(records.iter().any(|event|matches!(event,PageEvent::Mutation{record} if record.attribute.as_deref()==Some("data-test"))));
    let id = records
        .iter()
        .find_map(|event| {
            if let PageEvent::RequestFinished { request_id, .. } = event {
                Some(*request_id)
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        String::from_utf8(page.response_body(id).unwrap())
            .unwrap()
            .contains("network")
    );
    route.remove();
    let auth = page
        .route(
            "*/auth",
            RouteStage::Auth,
            Arc::new(|route| {
                assert!(
                    route
                        .info()
                        .auth_challenge
                        .as_ref()
                        .unwrap()
                        .contains("fixture")
                );
                route.authenticate("user".into(), "secret".into()).unwrap();
            }),
        )
        .unwrap();
    page.request(
        "GET",
        "https://example.test/auth",
        RequestOptions::default(),
    )
    .unwrap();
    assert_eq!(
        fixture.requests.lock().unwrap().last().unwrap().auth[0].username,
        "user"
    );
    auth.remove();
    page.set_extra_headers(vec![("x-extra".into(), "yes".into())])
        .unwrap();
    page.request(
        "GET",
        "https://example.test/cached",
        RequestOptions::default(),
    )
    .unwrap();
    let count = fixture.requests.lock().unwrap().len();
    page.request(
        "GET",
        "https://example.test/cached",
        RequestOptions::default(),
    )
    .unwrap();
    assert_eq!(fixture.requests.lock().unwrap().len(), count);
    page.clear_cache().unwrap();
    page.request(
        "GET",
        "https://example.test/cached",
        RequestOptions::default(),
    )
    .unwrap();
    assert_eq!(fixture.requests.lock().unwrap().len(), count + 1);
    assert_eq!(
        fixture
            .requests
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .headers
            .get("x-extra")
            .unwrap(),
        "yes"
    );
    page.configure_network(NetworkOptions {
        offline: true,
        ..Default::default()
    })
    .unwrap();
    assert!(
        page.request(
            "GET",
            "https://example.test/offline",
            RequestOptions::default()
        )
        .is_err()
    );
    assert_eq!(page.evaluate("navigator.onLine").unwrap(), json!(false));
}
#[test]
fn response_routes_fulfillment_drop_and_cancellation() {
    let fixture = Arc::new(Fixture::default());
    let session = Session::with_resource_loader(SessionOptions::default(), fixture).unwrap();
    let page = session.new_page().unwrap();
    let response = page
        .route(
            "*/original",
            RouteStage::Response,
            Arc::new(|route| {
                assert!(!route.body().unwrap().is_empty());
                route
                    .continue_request(RouteOverrides {
                        body: Some(b"changed".to_vec()),
                        status: Some(201),
                        ..Default::default()
                    })
                    .unwrap();
            }),
        )
        .unwrap();
    let result = page
        .request(
            "GET",
            "https://example.test/original",
            RequestOptions::default(),
        )
        .unwrap();
    assert_eq!(result.status_code, 201);
    assert_eq!(result.content, b"changed");
    response.remove();
    let fulfill = page
        .route(
            "*/mock",
            RouteStage::Request,
            Arc::new(|route| {
                route
                    .fulfill(
                        200,
                        vec![("content-type".into(), "text/plain".into())],
                        vec![0, 255, 1],
                    )
                    .unwrap();
            }),
        )
        .unwrap();
    assert_eq!(
        page.request(
            "GET",
            "https://example.test/mock",
            RequestOptions::default()
        )
        .unwrap()
        .content,
        vec![0, 255, 1]
    );
    fulfill.remove();
    let unresolved = page
        .route("*/unresolved", RouteStage::Request, Arc::new(|_| {}))
        .unwrap();
    assert!(
        page.request(
            "GET",
            "https://example.test/unresolved",
            RequestOptions::default()
        )
        .is_err()
    );
    unresolved.remove();
    let (tx, rx) = mpsc::sync_channel(1);
    let pending = page
        .route(
            "*/pending",
            RouteStage::Request,
            Arc::new(move |route| {
                tx.send(route).unwrap();
            }),
        )
        .unwrap();
    let other = page.clone();
    let navigation = std::thread::spawn(move || {
        other.request(
            "GET",
            "https://example.test/pending",
            RequestOptions::default(),
        )
    });
    let route = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
    page.close().unwrap();
    assert!(navigation.join().unwrap().is_err());
    assert!(!route.is_pending());
    assert!(route.continue_request(Default::default()).is_err());
    drop(pending);
}

#[test]
fn websocket_and_event_source_deliver_dom_values_and_typed_records() {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        time::Duration,
    };
    let websocket = TcpListener::bind("127.0.0.1:0").unwrap();
    let ws_address = websocket.local_addr().unwrap();
    let ws_server = std::thread::spawn(move || {
        let (stream, _) = websocket.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        let message = socket.read().unwrap();
        socket.send(message).unwrap();
        socket.close(None).unwrap();
    });
    let sse = TcpListener::bind("127.0.0.1:0").unwrap();
    let sse_address = sse.local_addr().unwrap();
    let sse_server = std::thread::spawn(move || {
        let (stream, _) = sse.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = BufReader::new(stream);
        loop {
            let mut line = String::new();
            assert!(request.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" {
                break;
            }
        }
        let mut stream = request.into_inner();
        let body = "id: first\nevent: ping\ndata: hello\ndata: world\n\n";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    });
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    let (ws_tx, ws_rx) = mpsc::channel();
    let (sse_tx, sse_rx) = mpsc::channel();
    let ws_subscription = page
        .on(
            "websocket",
            Arc::new(move |event| {
                ws_tx.send(event).unwrap();
            }),
        )
        .unwrap();
    let sse_subscription = page
        .on(
            "eventsource",
            Arc::new(move |event| {
                sse_tx.send(event).unwrap();
            }),
        )
        .unwrap();
    assert_eq!(page.evaluate(&format!(r#"
        new Promise((resolve, reject) => {{
          const socket = new WebSocket('ws://{ws_address}/echo');
          socket.binaryType = 'arraybuffer';
          socket.onopen = () => socket.send(new Uint8Array([0, 1, 255]));
          socket.onerror = () => reject(new Error('websocket failed'));
          socket.onmessage = event => {{ resolve(Array.from(new Uint8Array(event.data))); socket.close(); }};
        }})
    "#)).unwrap(), json!([0, 1, 255]));
    let mut kinds = Vec::new();
    loop {
        let PageEvent::WebSocket { kind, data, .. } =
            ws_rx.recv_timeout(Duration::from_secs(3)).unwrap()
        else {
            unreachable!()
        };
        kinds.push(kind.clone());
        if kind == "message" {
            assert_eq!(data, vec![0, 1, 255]);
            break;
        }
    }
    assert!(
        kinds.iter().position(|kind| kind == "open").unwrap()
            < kinds.iter().position(|kind| kind == "message").unwrap()
    );
    assert_eq!(
        page.evaluate(&format!(
            r#"
        new Promise((resolve, reject) => {{
          const events = new EventSource('http://{sse_address}/events');
          events.addEventListener('ping', event => {{
            events.close(); resolve([event.lastEventId, event.data]);
          }});
          events.onerror = () => {{ events.close(); reject(new Error('event source failed')); }};
        }})
    "#
        ))
        .unwrap(),
        json!(["first", "hello\nworld"])
    );
    let PageEvent::EventSource {
        event_name,
        event_id,
        data,
        ..
    } = sse_rx.recv_timeout(Duration::from_secs(3)).unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        (event_name, event_id, data),
        ("ping".into(), "first".into(), "hello\nworld".into())
    );
    drop((ws_subscription, sse_subscription));
    session.close();
    ws_server.join().unwrap();
    sse_server.join().unwrap();
}
