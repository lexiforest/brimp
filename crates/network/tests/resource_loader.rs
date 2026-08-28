use std::future::Future;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

use http::{HeaderValue, Method};
use network::{
    CurlConfig, CurlResourceLoader, NetworkError, Proxy, ResourceLoader, ResourceRequest,
    ResourceStreamDirective, ResourceStreamEvent,
};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

#[test]
fn streaming_callback_delivers_chunks_before_completion() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (continue_sender, continue_receiver) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = request_head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nfirst\r\n")
            .unwrap();
        stream.flush().unwrap();
        continue_receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        stream.write_all(b"6\r\nsecond\r\n0\r\n\r\n").unwrap();
    });
    let (event_sender, event_receiver) = mpsc::channel();
    let loader = CurlResourceLoader::default();
    let _stream = loader
        .fetch_stream_callback(
            ResourceRequest::get(format!("http://{address}/stream")),
            Box::new(move |event, _| {
                let _ = event_sender.send(event);
                ResourceStreamDirective::Continue
            }),
        )
        .unwrap();
    let first = event_receiver.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(
        first,
        ResourceStreamEvent::Headers { status, .. } if status == 200
    ));
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Chunk(bytes) if bytes == b"first"
    ));
    continue_sender.send(()).unwrap();
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Chunk(bytes) if bytes == b"second"
    ));
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Complete
    ));
    server.join().unwrap();
}

#[test]
fn streaming_handle_pauses_and_resumes_curl_without_losing_the_chunk() {
    let (url, server) = one_response(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
    let (event_sender, event_receiver) = mpsc::channel();
    let first_chunk = Arc::new(AtomicBool::new(true));
    let callback_first_chunk = Arc::clone(&first_chunk);
    let loader = CurlResourceLoader::default();
    let stream = loader
        .fetch_stream_callback(
            ResourceRequest::get(url),
            Box::new(move |event, _| match event {
                ResourceStreamEvent::Chunk(_)
                    if callback_first_chunk.swap(false, Ordering::AcqRel) =>
                {
                    ResourceStreamDirective::Pause
                }
                event => {
                    let _ = event_sender.send(event);
                    ResourceStreamDirective::Continue
                }
            }),
        )
        .unwrap();
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Headers { .. }
    ));
    assert!(
        event_receiver
            .recv_timeout(Duration::from_millis(100))
            .is_err()
    );
    stream.resume();
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Chunk(bytes) if bytes == b"hello"
    ));
    assert!(matches!(
        event_receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        ResourceStreamEvent::Complete
    ));
    server.join().unwrap();
}

#[test]
fn cancelling_a_stream_removes_the_curl_transfer() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (closed_sender, closed_receiver) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = request_head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000000\r\n\r\n")
            .unwrap();
        stream.flush().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut byte = [0];
        closed_sender
            .send(stream.read(&mut byte).unwrap_or(0))
            .unwrap();
    });
    let (headers_sender, headers_receiver) = mpsc::channel();
    let loader = CurlResourceLoader::default();
    let stream = loader
        .fetch_stream_callback(
            ResourceRequest::get(format!("http://{address}/cancel")),
            Box::new(move |event, _| {
                if matches!(event, ResourceStreamEvent::Headers { .. }) {
                    let _ = headers_sender.send(());
                }
                ResourceStreamDirective::Continue
            }),
        )
        .unwrap();
    headers_receiver
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    stream.cancel();
    assert_eq!(
        closed_receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap(),
        0
    );
    server.join().unwrap();
}
fn request_head(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 1024];
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        let count = stream.read(&mut chunk).unwrap();
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    bytes
}
fn one_response(response: &'static [u8]) -> (String, std::thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = request_head(&mut stream);
        stream.write_all(response).unwrap();
        request
    });
    (format!("http://{address}/resource"), server)
}

#[test]
fn supports_methods_bodies_duplicate_headers_and_compression() {
    let gzip = b"HTTP/1.1 201 Created\r\nContent-Encoding: gzip\r\nContent-Length: 25\r\nX-Multi: one\r\nX-Multi: two\r\n\r\n\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03\xcbH\xcd\xc9\xc9\x07\x00\x86\xa6\x10\x36\x05\x00\x00\x00";
    let (url, server) = one_response(gzip);
    let mut request = ResourceRequest::new(Method::PATCH, url);
    request
        .headers
        .append("x-test", HeaderValue::from_static("yes"));
    request.body = Some(b"hello".to_vec());
    let response = runtime()
        .block_on(CurlResourceLoader::default().fetch(request))
        .unwrap();
    let sent = String::from_utf8_lossy(&server.join().unwrap()).to_ascii_lowercase();
    assert!(sent.starts_with("patch /resource http/1.1"));
    assert!(sent.contains("x-test: yes"));
    assert_eq!(response.status, 201);
    assert_eq!(response.body, b"hello");
    assert_eq!(response.headers.get_all("x-multi").count(), 2);
    assert!(!response.headers.contains_key("content-encoding"));
    assert!(!response.headers.contains_key("content-length"));
}

#[test]
fn head_returns_no_body() {
    let (url, server) = one_response(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n");
    let response = runtime()
        .block_on(CurlResourceLoader::default().fetch(ResourceRequest::new(Method::HEAD, url)))
        .unwrap();
    assert!(
        String::from_utf8_lossy(&server.join().unwrap()).starts_with("HEAD /resource HTTP/1.1")
    );
    assert!(response.body.is_empty());
}

#[test]
fn concurrent_requests_share_one_executor_without_request_threads() {
    const COUNT: usize = 24;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut workers = Vec::new();
        for _ in 0..COUNT {
            let (mut stream, _) = listener.accept().unwrap();
            workers.push(std::thread::spawn(move || {
                let _ = request_head(&mut stream);
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                    .unwrap();
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
    });
    let loader = CurlResourceLoader::default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let responses = runtime.block_on(async {
        let mut tasks = tokio::task::JoinSet::new();
        for index in 0..COUNT {
            let loader = loader.clone();
            tasks.spawn(async move {
                loader
                    .fetch(ResourceRequest::get(format!("http://{address}/{index}")))
                    .await
            });
        }
        let mut responses = Vec::new();
        while let Some(result) = tasks.join_next().await {
            responses.push(result.unwrap().unwrap());
        }
        responses
    });
    assert_eq!(responses.len(), COUNT);
    assert!(responses.iter().all(|response| response.body == b"ok"));
    server.join().unwrap();
}

#[test]
fn sequential_requests_reuse_a_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        for _ in 0..2 {
            assert!(!request_head(&mut stream).is_empty());
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok",
                )
                .unwrap();
        }
        listener.set_nonblocking(true).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            listener.accept().is_err(),
            "a second TCP connection was opened"
        );
    });
    let loader = CurlResourceLoader::default();
    let rt = runtime();
    for path in ["one", "two"] {
        rt.block_on(loader.fetch(ResourceRequest::get(format!("http://{address}/{path}"))))
            .unwrap();
    }
    server.join().unwrap();
}

#[test]
fn enforces_timeout_and_response_limit() {
    let (url, server) = one_response(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
    let loader = CurlResourceLoader::new(CurlConfig {
        max_response_bytes: 4,
        ..CurlConfig::default()
    })
    .unwrap();
    assert!(matches!(
        runtime().block_on(loader.fetch(ResourceRequest::get(url))),
        Err(NetworkError::ResponseTooLarge { limit: 4 })
    ));
    server.join().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = request_head(&mut stream);
        std::thread::sleep(Duration::from_millis(200));
    });
    let loader = CurlResourceLoader::new(CurlConfig {
        request_timeout: Duration::from_millis(30),
        ..CurlConfig::default()
    })
    .unwrap();
    let error = runtime()
        .block_on(loader.fetch(ResourceRequest::get(format!("http://{address}/slow"))))
        .unwrap_err();
    assert!(matches!(error, NetworkError::Transport(_)));
    server.join().unwrap();
}

#[test]
fn dropping_request_and_final_loader_cancels_and_joins_worker() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (accepted_sender, accepted_receiver) = std::sync::mpsc::sync_channel(1);
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        accepted_sender.send(()).unwrap();
        let _ = request_head(&mut stream);
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte).unwrap_or(0), 0);
    });
    let loader = Arc::new(CurlResourceLoader::default());
    let cloned = Arc::clone(&loader);
    let mut future = Box::pin(async move {
        cloned
            .fetch(ResourceRequest::get(format!("http://{address}/hang")))
            .await
    });
    struct Noop;
    impl Wake for Noop {
        fn wake(self: Arc<Self>) {}
    }
    let waker = Waker::from(Arc::new(Noop));
    assert!(matches!(
        Pin::new(&mut future).poll(&mut Context::from_waker(&waker)),
        Poll::Pending
    ));
    accepted_receiver
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    drop(future);
    let started = Instant::now();
    drop(loader);
    assert!(started.elapsed() < Duration::from_secs(1));
    server.join().unwrap();
}

#[test]
fn validates_proxy_urls() {
    assert!(Proxy::parse("http://proxy.test:8080").is_ok());
    assert!(Proxy::parse("socks5h://proxy.test:1080").is_ok());
    assert!(Proxy::parse("https://proxy.test").is_err());
}

#[test]
fn profile_diagnostics_reject_missing_profile_before_navigation() {
    let config = CurlConfig {
        impersonation_profile: "brimp-profile-that-does-not-exist".into(),
        ..CurlConfig::default()
    };
    let error = CurlResourceLoader::check_profile(&config).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("brimp-profile-that-does-not-exist")
    );
}
