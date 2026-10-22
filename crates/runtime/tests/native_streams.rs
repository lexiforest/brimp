use async_trait::async_trait;
use brimp_runtime::{
    network::*,
    runtime::{AutomationError, native::*},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
struct Fixture;
#[async_trait]
impl ResourceLoader for Fixture {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderList::new();
        let (status, body) = if request.url.ends_with("/redirect") {
            headers.insert(
                "location".parse::<http::HeaderName>().unwrap(),
                "/body".parse().unwrap(),
            );
            headers.insert(
                "set-cookie".parse::<http::HeaderName>().unwrap(),
                "redirect=yes; Path=/".parse().unwrap(),
            );
            (http::StatusCode::FOUND, vec![])
        } else if request.url.ends_with("/body") {
            assert!(
                request.headers["cookie"]
                    .to_str()
                    .unwrap()
                    .contains("redirect=yes")
            );
            (
                http::StatusCode::OK,
                (0..1024 * 1024).map(|i| (i % 251) as u8).collect(),
            )
        } else {
            (http::StatusCode::OK, b"<main>OK</main>".to_vec())
        };
        Ok(ResourceResponse {
            status,
            headers,
            body,
            effective_url: request.url,
            metadata: Default::default(),
        })
    }
}
#[test]
fn stream_is_bounded_resumable_and_cancelled_by_navigation() {
    let session =
        Session::with_resource_loader(SessionOptions::default(), Arc::new(Fixture)).unwrap();
    let page = session.new_page().unwrap();
    let stream = page
        .stream("GET", "https://stream.test/redirect", Default::default())
        .unwrap();
    assert_eq!(stream.metadata.status, 200);
    assert!(stream.metadata.url.ends_with("/body"));
    std::thread::sleep(Duration::from_millis(30)); // Fill the bounded queue and force replay-on-pause.
    assert_eq!(page.evaluate("6*7").unwrap(), 42);
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.body.next_chunk().unwrap() {
        assert!(chunk.len() <= 16 * 1024);
        bytes.extend(chunk);
    }
    assert_eq!(
        bytes,
        (0..1024 * 1024)
            .map(|i| (i % 251) as u8)
            .collect::<Vec<_>>()
    );
    let pending = page
        .stream("GET", "https://stream.test/body", Default::default())
        .unwrap();
    page.request("GET", "https://stream.test/page", Default::default())
        .unwrap();
    assert!(matches!(
        pending.body.next_chunk(),
        Err(AutomationError::Cancellation)
    ));
    let pending = page
        .stream("GET", "https://stream.test/body", Default::default())
        .unwrap();
    session.close();
    assert!(matches!(
        pending.body.next_chunk(),
        Err(AutomationError::Cancellation)
    ));
}
#[test]
fn buffered_route_streams_replay_chunks() {
    let session =
        Session::with_resource_loader(SessionOptions::default(), Arc::new(Fixture)).unwrap();
    let page = session.new_page().unwrap();
    let route = page
        .route(
            "*",
            brimp_runtime::runtime::routes::RouteStage::Request,
            Arc::new(|route| {
                route.fulfill(201, vec![], vec![37; 700_000]).unwrap();
            }),
        )
        .unwrap();
    let response = page
        .stream("GET", "https://stream.test/", Default::default())
        .unwrap();
    std::thread::sleep(Duration::from_millis(30));
    let mut bytes = Vec::new();
    while let Some(chunk) = response.body.next_chunk().unwrap() {
        bytes.extend(chunk);
    }
    assert_eq!(response.metadata.status, 201);
    assert_eq!(bytes, vec![37; 700_000]);
    drop(route);
    session.close();
}

#[test]
fn realm_fetch_streams_follow_redirects_and_store_intermediate_cookies() {
    let session =
        Session::with_resource_loader(SessionOptions::default(), Arc::new(Fixture)).unwrap();
    let page = session.new_page().unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let subscriptions: Vec<_> = ["request", "response", "request_finished", "request_failed"]
        .into_iter()
        .map(|name| {
            let events = events.clone();
            page.on(
                name,
                Arc::new(move |event| events.lock().unwrap().push(event)),
            )
            .unwrap()
        })
        .collect();
    assert_eq!(
        page.evaluate(
            "fetch('https://stream.test/redirect').then(r=>r.arrayBuffer()).then(b=>b.byteLength)"
        )
        .unwrap(),
        1024 * 1024
    );
    // Redirect hops finish normally; stopping their transport must not invent a failure.
    let response = page
        .stream("GET", "https://stream.test/redirect", Default::default())
        .unwrap();
    while response.body.next_chunk().unwrap().is_some() {}
    use brimp_runtime::runtime::events::PageEvent;
    let events = events.lock().unwrap();
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, PageEvent::RequestFailed { .. }))
    );
    let ids: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PageEvent::Request { request } => Some(request.id),
            _ => None,
        })
        .collect();
    assert_eq!(ids.len(), 4);
    for id in ids {
        let response = events
            .iter()
            .position(|event| {
                matches!(event,
            PageEvent::Response { response } if response.request_id == id)
            })
            .unwrap();
        let finished = events
            .iter()
            .position(|event| {
                matches!(event,
            PageEvent::RequestFinished { request_id, .. } if *request_id == id)
            })
            .unwrap();
        assert!(response < finished);
    }
    drop(events);
    drop(subscriptions);
    session.close();
}

struct CountedFixture(Arc<std::sync::atomic::AtomicUsize>);
#[async_trait]
impl ResourceLoader for CountedFixture {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        Fixture.fetch(request).await
    }
    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let count = self.0.clone();
        Fixture.fetch_stream_callback(
            request,
            Box::new(move |event, control| {
                let bytes = match &event {
                    ResourceStreamEvent::Chunk(bytes) => bytes.len(),
                    _ => 0,
                };
                let directive = callback(event, control);
                if directive == ResourceStreamDirective::Continue {
                    count.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);
                }
                directive
            }),
        )
    }
}
#[test]
fn unread_fetch_applies_backpressure_until_a_byob_reader_consumes() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let count = Arc::new(AtomicUsize::new(0));
    let session = Session::with_resource_loader(
        SessionOptions::default(),
        Arc::new(CountedFixture(count.clone())),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    page.evaluate(
        "fetch('https://stream.test/redirect').then(r=>{globalThis.response=r;return null})",
    )
    .unwrap();
    page.evaluate("new Promise(resolve=>setTimeout(()=>resolve(null),30))")
        .unwrap();
    assert!(count.load(Ordering::SeqCst) > 0);
    assert!(
        count.load(Ordering::SeqCst) <= 256 * 1024,
        "unread bytes escaped backpressure"
    );
    assert_eq!(page.evaluate("(async()=>{const reader=response.body.getReader({mode:'byob'});let size=0;for(;;){const {value,done}=await reader.read(new Uint8Array(8192));if(value)size+=value.byteLength;if(done)return size;}})()").unwrap(),1024*1024);
    assert_eq!(count.load(Ordering::SeqCst), 1024 * 1024);
}
