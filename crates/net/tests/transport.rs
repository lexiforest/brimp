use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use brimp_net::{Client, Config, RedirectPolicy};
use bytes::Bytes;
use http::Request;
use http_body_util::{BodyExt, Empty, Full};

const DEADLINE: Duration = Duration::from_secs(5);

fn client() -> Client {
    Client::new(Config {
        request_timeout: DEADLINE,
        ..Config::default()
    })
}

fn request(url: &str) -> Request<Empty<Bytes>> {
    Request::builder().uri(url).body(Empty::new()).unwrap()
}

fn head(stream: &mut TcpStream) -> String {
    stream.set_nonblocking(false).unwrap();
    stream.set_read_timeout(Some(DEADLINE)).unwrap();
    stream.set_write_timeout(Some(DEADLINE)).unwrap();
    let mut bytes = Vec::new();
    let mut byte = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes).unwrap()
}

fn reply(stream: &mut TcpStream, body: &[u8]) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
}

fn server(
    n: usize,
    handler: impl Fn(TcpStream) + Send + Sync + 'static,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handler = Arc::new(handler);
    let task = thread::spawn(move || {
        let deadline = Instant::now() + DEADLINE;
        let mut tasks = Vec::new();
        while tasks.len() < n {
            match listener.accept() {
                Ok((stream, _)) => {
                    let handler = Arc::clone(&handler);
                    tasks.push(thread::spawn(move || handler(stream)));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "missing request to fixture");
                    thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("{error}"),
            }
        }
        for task in tasks {
            task.join().unwrap();
        }
    });
    (url, task)
}

#[tokio::test]
async fn same_host_requests_start_concurrently() {
    let (seen_tx, seen_rx) = mpsc::channel();
    let (release_tx, release_rx) = tokio::sync::watch::channel(false);
    let (url, task) = server(4, move |mut stream| {
        head(&mut stream);
        seen_tx.send(()).unwrap();
        let started = Instant::now();
        while !*release_rx.borrow() {
            assert!(
                started.elapsed() < DEADLINE,
                "same-host requests serialized"
            );
            thread::sleep(Duration::from_millis(1));
        }
        reply(&mut stream, b"ok");
    });
    let client = client();
    let mut pending = Vec::new();
    for _ in 0..4 {
        let client = client.clone();
        let url = url.clone();
        pending.push(tokio::spawn(async move {
            client
                .send(request(&url))
                .await
                .unwrap()
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
        }));
    }
    tokio::task::spawn_blocking(move || {
        for _ in 0..4 {
            seen_rx.recv_timeout(DEADLINE).unwrap();
        }
    })
    .await
    .unwrap();
    release_tx.send(true).unwrap();
    for task in pending {
        assert_eq!(task.await.unwrap(), "ok");
    }
    task.join().unwrap();
}

#[tokio::test]
async fn slow_consumer_does_not_block_other_requests_and_resume_preserves_bytes() {
    const SIZE: usize = 8 * 1024 * 1024;
    let (url, task) = server(2, |mut stream| {
        let request = head(&mut stream);
        if request.starts_with("GET /slow ") {
            let payload: Vec<u8> = (0..SIZE).map(|i| (i % 251) as u8).collect();
            reply(&mut stream, &payload);
        } else {
            reply(&mut stream, b"fast");
        }
    });
    let client = client();
    let slow = client.send(request(&format!("{url}/slow"))).await.unwrap();
    // Let the small bounded channel fill while retaining, but not reading, the body.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let fast = tokio::time::timeout(Duration::from_secs(2), async {
        client
            .send(request(&format!("{url}/fast")))
            .await
            .unwrap()
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
    })
    .await
    .expect("slow reader blocked the curl coordinator");
    assert_eq!(fast, "fast");
    let bytes = tokio::time::timeout(DEADLINE, slow.into_body().collect())
        .await
        .unwrap()
        .unwrap()
        .to_bytes();
    assert_eq!(bytes.len(), SIZE);
    assert!(bytes.iter().enumerate().all(|(i, b)| *b == (i % 251) as u8));
    task.join().unwrap();
}

#[tokio::test]
async fn first_small_chunk_is_available_before_response_finishes() {
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = std::sync::Mutex::new(release_rx);
    let (url, task) = server(1, move |mut stream| {
        head(&mut stream);
        stream.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\none\r\n").unwrap();
        release_rx.lock().unwrap().recv_timeout(DEADLINE).unwrap();
        stream.write_all(b"3\r\ntwo\r\n0\r\n\r\n").unwrap();
    });
    let client = client();
    let mut body = client.send(request(&url)).await.unwrap().into_body();
    let frame = tokio::time::timeout(Duration::from_secs(2), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(frame.into_data().unwrap(), "one");
    release_tx.send(()).unwrap();
    assert_eq!(body.collect().await.unwrap().to_bytes(), "two");
    task.join().unwrap();
}

#[tokio::test]
async fn dropping_body_cancels_stalled_transfer() {
    let (closed_tx, closed_rx) = mpsc::channel();
    let (url, task) = server(1, move |mut stream| {
        head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 999999\r\n\r\na")
            .unwrap();
        let mut byte = [0];
        let result = stream.read(&mut byte);
        closed_tx
            .send(
                matches!(result, Ok(0))
                    || result.is_err_and(|e| e.kind() == std::io::ErrorKind::ConnectionReset),
            )
            .unwrap();
    });
    let client = client();
    let response = client.send(request(&url)).await.unwrap();
    drop(response);
    assert!(closed_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    task.join().unwrap();
}

#[tokio::test]
async fn cancelling_before_headers_closes_connection() {
    let (seen_tx, seen_rx) = tokio::sync::oneshot::channel();
    let seen_tx = std::sync::Mutex::new(Some(seen_tx));
    let (closed_tx, closed_rx) = mpsc::channel();
    let (url, task) = server(1, move |mut stream| {
        head(&mut stream);
        seen_tx.lock().unwrap().take().unwrap().send(()).unwrap();
        let mut byte = [0];
        closed_tx
            .send(stream.read(&mut byte).unwrap() == 0)
            .unwrap();
    });
    let client = client();
    let other = client.clone();
    let pending = tokio::spawn(async move { other.send(request(&url)).await });
    seen_rx.await.unwrap();
    pending.abort();
    assert!(matches!(pending.await, Err(error) if error.is_cancelled()));
    assert!(closed_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    task.join().unwrap();
}

#[tokio::test]
async fn client_shutdown_reports_error_to_pending_body() {
    let (url, task) = server(1, |mut stream| {
        head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 999999\r\n\r\na")
            .unwrap();
        let mut byte = [0];
        let _ = stream.read(&mut byte);
    });
    let client = client();
    let body = client.send(request(&url)).await.unwrap().into_body();
    drop(client);
    let result = tokio::time::timeout(Duration::from_secs(2), body.collect())
        .await
        .unwrap();
    assert!(result.is_err());
    task.join().unwrap();
}

#[tokio::test]
async fn truncated_body_reports_error_after_buffered_data() {
    let (url, task) = server(1, |mut stream| {
        head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 99\r\nConnection: close\r\n\r\nshort")
            .unwrap();
    });
    let client = client();
    let mut body = client.send(request(&url)).await.unwrap().into_body();
    assert_eq!(
        body.frame().await.unwrap().unwrap().into_data().unwrap(),
        "short"
    );
    assert!(body.frame().await.unwrap().is_err());
    assert!(body.frame().await.is_none());
    task.join().unwrap();
}

#[tokio::test]
async fn follows_redirects_and_returns_final_headers() {
    let (url, task) = server(2, |mut stream| {
        let request = head(&mut stream);
        if request.starts_with("GET /start ") {
            stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /end\r\nX-Hop: first\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        } else {
            stream.write_all(b"HTTP/1.1 200 OK\r\nX-Hop: last\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok").unwrap();
        }
    });
    let client = Client::new(Config {
        redirect_policy: RedirectPolicy::Follow,
        ..Config::default()
    });
    let response = client.send(request(&format!("{url}/start"))).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-hop"], "last");
    assert_eq!(response.headers().get_all("set-cookie").iter().count(), 2);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "ok"
    );
    task.join().unwrap();
}

#[tokio::test]
async fn sends_post_body_and_handles_empty_response() {
    let (url, task) = server(1, |mut stream| {
        let request = head(&mut stream);
        assert!(request.starts_with("POST / "));
        let mut body = [0; 5];
        stream.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"hello");
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
            .unwrap();
    });
    let client = client();
    let response = client
        .send(
            Request::post(url)
                .body(Full::new(Bytes::from_static(b"hello")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    assert!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    task.join().unwrap();
}

#[test]
fn blocking_collection_remains_available() {
    let (url, task) = server(1, |mut stream| {
        head(&mut stream);
        reply(&mut stream, b"collected");
    });
    let result = client()
        .send_collect(Request::get(&url).body(None).unwrap())
        .unwrap();
    assert_eq!(result.body, b"collected");
    assert_eq!(result.effective_url, format!("{url}/"));
    task.join().unwrap();
}

#[tokio::test]
async fn catalog_resolution_is_shared_by_clones() {
    // Change process environment only in a dedicated child; the parallel test
    // suite must never race with a process-wide set_var call.
    if std::env::var_os("BRIMP_NET_CATALOG_TEST_CHILD").is_none() {
        let directory =
            std::env::temp_dir().join(format!("brimp-net-catalog-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "catalog_resolution_is_shared_by_clones",
                "--nocapture",
            ])
            .env("BRIMP_NET_CATALOG_TEST_CHILD", "1")
            .env("IMPERSONATE_CONFIG_DIR", &directory)
            .status()
            .unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        assert!(status.success());
        return;
    }
    let (url, task) = server(2, |mut stream| {
        head(&mut stream);
        reply(&mut stream, b"ok");
    });
    let client = Client::new(Config {
        impersonation_target: "chrome136".into(),
        ..Config::default()
    });
    client
        .send(request(&url))
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap();
    let path = std::path::PathBuf::from(std::env::var_os("IMPERSONATE_CONFIG_DIR").unwrap())
        .join("fingerprints.json");
    std::fs::write(path, b"invalid catalog").unwrap();
    // Existing clones retain their resolution. New clients see the changed file.
    client
        .clone()
        .send(request(&url))
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap();
    assert!(
        Client::new(Config::default())
            .send(request(&url))
            .await
            .is_err()
    );
    task.join().unwrap();
}

#[tokio::test]
async fn dropping_a_paused_body_stops_the_download() {
    let (stopped_tx, stopped_rx) = mpsc::channel();
    let (url, task) = server(1, move |mut stream| {
        head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 33554432\r\n\r\n")
            .unwrap();
        let chunk = [42; 64 * 1024];
        for _ in 0..512 {
            if stream.write_all(&chunk).is_err() {
                stopped_tx.send(()).unwrap();
                return;
            }
        }
        panic!("unread response downloaded in full instead of pausing");
    });
    let client = client();
    let body = client.send(request(&url)).await.unwrap().into_body();
    tokio::time::sleep(Duration::from_millis(100)).await;
    drop(body);
    stopped_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    task.join().unwrap();
}

#[tokio::test]
async fn error_survives_a_full_body_channel() {
    const SIZE: usize = 1024 * 1024;
    let (url, task) = server(1, |mut stream| {
        head(&mut stream);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            SIZE + 1
        )
        .unwrap();
        stream.write_all(&vec![42; SIZE]).unwrap();
    });
    let client = client();
    let mut body = client.send(request(&url)).await.unwrap().into_body();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut received = 0;
    let mut failed = false;
    while let Some(frame) = body.frame().await {
        match frame {
            Ok(frame) => received += frame.into_data().unwrap().len(),
            Err(_) => {
                failed = true;
                break;
            }
        }
    }
    assert_eq!(received, SIZE);
    assert!(failed);
    assert!(body.frame().await.is_none());
    task.join().unwrap();
}

#[tokio::test]
async fn repeated_request_headers_are_preserved() {
    let (url, task) = server(1, |mut stream| {
        let request = head(&mut stream).to_ascii_lowercase();
        assert!(request.contains("x-repeat: first\r\n"));
        assert!(request.contains("x-repeat: second\r\n"));
        reply(&mut stream, b"ok");
    });
    let client = client();
    let request = Request::get(url)
        .header("x-repeat", "first")
        .header("x-repeat", "second")
        .body(Empty::<Bytes>::new())
        .unwrap();
    client
        .send(request)
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap();
    task.join().unwrap();
}
