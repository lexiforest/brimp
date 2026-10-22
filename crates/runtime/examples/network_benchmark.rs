//! Loopback HTTP/1.1 comparison of the worker resource adapter and direct brimp-net calls.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use brimp_net::{Client, Config};
use brimp_runtime::network::{CurlConfig, CurlResourceLoader, ResourceLoader, ResourceRequest};
use bytes::Bytes;
use http::Request;
use http_body_util::{BodyExt, Empty};
use serde_json::json;

fn serve(mut stream: TcpStream) {
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    loop {
        let mut header = Vec::new();
        let mut byte = [0];
        while !header.ends_with(b"\r\n\r\n") {
            if stream.read_exact(&mut byte).is_err() {
                return;
            }
            header.push(byte[0]);
        }
        let header = String::from_utf8(header).unwrap();
        let path = header.split_whitespace().nth(1).unwrap();
        let mut args = path.trim_start_matches('/').split('/');
        let size: usize = args.next().unwrap().parse().unwrap();
        let delay: u64 = args.next().unwrap().parse().unwrap();
        thread::sleep(Duration::from_millis(delay));
        let body = vec![b'x'; size];
        if write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {size}\r\n\r\n").is_err()
            || stream.write_all(&body).is_err()
        {
            return;
        }
    }
}

#[derive(Clone)]
enum Transport {
    Worker(CurlResourceLoader),
    Net(Client),
}
impl Transport {
    async fn batch(&self, url: &str, concurrency: usize, size: usize) -> f64 {
        let start = Instant::now();
        let mut pending = Vec::new();
        for _ in 0..concurrency {
            let transport = self.clone();
            let url = url.to_owned();
            pending.push(tokio::spawn(async move {
                let body = match transport {
                    Self::Worker(loader) => {
                        Bytes::from(loader.fetch(ResourceRequest::get(url)).await.unwrap().body)
                    }
                    Self::Net(client) => client
                        .send(Request::get(url).body(Empty::<Bytes>::new()).unwrap())
                        .await
                        .unwrap()
                        .into_body()
                        .collect()
                        .await
                        .unwrap()
                        .to_bytes(),
                };
                assert_eq!(body.len(), size);
                assert!(body.iter().all(|b| *b == b'x'));
            }));
        }
        for task in pending {
            task.await.unwrap();
        }
        start.elapsed().as_secs_f64() * 1000.0
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = Arc::clone(&stop);
    let server = thread::spawn(move || {
        let mut handlers = Vec::new();
        while !stopped.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((stream, _)) => handlers.push(thread::spawn(move || serve(stream))),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1))
                }
                Err(error) => panic!("{error}"),
            }
        }
        for handler in handlers {
            handler.join().unwrap();
        }
    });
    let worker = Transport::Worker(
        CurlResourceLoader::new(CurlConfig {
            impersonation_profile: "chrome136".into(),
            prefer_http3: false,
            ..CurlConfig::default()
        })
        .unwrap(),
    );
    let net = Transport::Net(Client::new(Config {
        impersonation_target: "chrome136".into(),
        ..Config::default()
    }));
    let mut results = Vec::new();
    for (label, concurrency, size, delay) in [
        ("small_serial", 1, 1024, 0),
        ("small_concurrent", 16, 1024, 0),
        ("delayed_concurrent", 16, 1024, 30),
        ("bulk_concurrent", 8, 2 * 1024 * 1024, 0),
    ] {
        let url = format!("http://{address}/{size}/{delay}");
        worker.batch(&url, concurrency, size).await;
        net.batch(&url, concurrency, size).await;
        let mut worker_ms = Vec::new();
        let mut net_ms = Vec::new();
        for round in 0..7 {
            if round % 2 == 0 {
                worker_ms.push(worker.batch(&url, concurrency, size).await);
                net_ms.push(net.batch(&url, concurrency, size).await);
            } else {
                net_ms.push(net.batch(&url, concurrency, size).await);
                worker_ms.push(worker.batch(&url, concurrency, size).await);
            }
        }
        let median = |samples: &[f64]| {
            let mut sorted = samples.to_vec();
            sorted.sort_by(f64::total_cmp);
            sorted[sorted.len() / 2]
        };
        results.push(json!({"workload":label, "concurrency":concurrency, "response_bytes":size,
            "server_delay_ms":delay, "worker_median_ms":median(&worker_ms), "brimp_net_median_ms":median(&net_ms),
            "worker_samples_ms":worker_ms, "brimp_net_samples_ms":net_ms}));
    }
    drop(worker);
    drop(net);
    stop.store(true, Ordering::Release);
    server.join().unwrap();
    println!("{}", serde_json::to_string_pretty(&json!({
        "os":std::env::consts::OS, "arch":std::env::consts::ARCH,
        "debug_assertions":cfg!(debug_assertions), "profile":"chrome136",
        "protocol":"loopback HTTP/1.1 keep-alive", "samples_per_transport":7,
        "measurement":"batch completion including body collection and byte validation; excludes client startup and warmup",
        "results":results
    })).unwrap());
}
