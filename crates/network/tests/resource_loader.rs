use std::io::{Read, Write};
use std::net::TcpListener;

use network::{CurlResourceLoader, ResourceLoader, ResourceRequest};

#[test]
fn curl_loader_fetches_a_resource_off_the_caller_thread() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 1024];
        let count = stream.read(&mut request).unwrap();
        assert!(String::from_utf8_lossy(&request[..count]).starts_with("GET /page HTTP/1.1"));
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello")
            .unwrap();
    });

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let response = runtime
        .block_on(
            CurlResourceLoader::default()
                .fetch(ResourceRequest::get(format!("http://{address}/page"))),
        )
        .unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"hello");
    assert_eq!(response.effective_url, format!("http://{address}/page"));
    server.join().unwrap();
}
