#[cfg(unix)]
use brimp_controller::AutomationError;
use brimp_controller::transport::{Transport, WebSocketTransport};
use brimp_controller::{CancellationToken, connection::Connection};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tungstenite::Message;

const TIMEOUT: Duration = Duration::from_secs(3);

fn receive(transport: &dyn Transport) -> Vec<u8> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(message) = transport.receive().unwrap() {
            return message;
        }
        assert!(Instant::now() < deadline, "message timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(unix)]
#[test]
fn framed_stream_retains_fragments_and_separates_messages() {
    use brimp_controller::transport::SocketPair;
    let (stream, mut peer) = std::os::unix::net::UnixStream::pair().unwrap();
    let transport = SocketPair::from_stream(stream).unwrap();
    peer.set_read_timeout(Some(TIMEOUT)).unwrap();
    peer.write_all(&[0, 0]).unwrap();
    assert!(transport.receive().unwrap().is_none());
    peer.write_all(&[0, 3, b'a']).unwrap();
    assert!(transport.receive().unwrap().is_none());
    peer.write_all(&[b'b', b'c', 0, 0, 0, 1, b'd']).unwrap();
    assert_eq!(receive(&transport), b"abc");
    assert_eq!(receive(&transport), b"d");
    transport.send(b"reply").unwrap();
    let mut bytes = [0; 9];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"\0\0\0\x05reply");
    transport.close();
    transport.close();
    assert!(transport.send(b"x").is_err());
    assert_eq!(peer.read(&mut bytes).unwrap(), 0);
}

#[cfg(unix)]
#[test]
fn framed_stream_rejects_oversized_and_truncated_messages() {
    use brimp_controller::transport::SocketPair;
    for bytes in [vec![4, 0, 0, 1], vec![0, 0, 0, 2, b'a']] {
        let (stream, mut peer) = std::os::unix::net::UnixStream::pair().unwrap();
        let transport = SocketPair::from_stream(stream).unwrap();
        peer.write_all(&bytes).unwrap();
        drop(peer);
        assert!(transport.receive().is_err());
    }
}

#[test]
fn websocket_correlates_out_of_order_responses_and_session_events() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!(
        "ws://{}/devtools/browser/test",
        listener.local_addr().unwrap()
    );
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(TIMEOUT)).unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        let mut requests = Vec::new();
        for _ in 0..2 {
            requests.push(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
            );
        }
        socket.send(Message::Ping(vec![1, 2].into())).unwrap();
        socket
            .send(Message::Text(
                json!({"method":"Page.loadEventFired", "sessionId":"page", "params":{}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        for request in requests.into_iter().rev() {
            socket
                .send(Message::Text(
                    json!({"id":request["id"],"result":{"method":request["method"]}})
                        .to_string()
                        .into(),
                ))
                .unwrap();
        }
        // Wait for connection close, driving the ping/close handshake.
        while let Ok(message) = socket.read() {
            if message.is_close() {
                break;
            }
        }
    });
    let connection = Arc::new(Connection::new(
        Arc::new(
            WebSocketTransport::connect(&endpoint, TIMEOUT, &CancellationToken::new()).unwrap(),
        ),
        TIMEOUT,
        CancellationToken::new(),
    ));
    let other = connection.clone();
    let command =
        std::thread::spawn(move || other.command("first", json!({}), Some("page")).unwrap());
    assert_eq!(
        connection
            .command("second", json!({}), Some("page"))
            .unwrap()["method"],
        "second"
    );
    assert_eq!(command.join().unwrap()["method"], "first");
    assert_eq!(
        connection
            .event("page", |event| event["method"] == "Page.loadEventFired")
            .unwrap()["sessionId"],
        "page"
    );
    connection.close();
    server.join().unwrap();
}

#[cfg(unix)]
#[test]
fn cancellation_and_timeout_interrupt_an_unresponsive_peer() {
    use brimp_controller::transport::SocketPair;
    for cancel in [true, false] {
        let (stream, _peer) = std::os::unix::net::UnixStream::pair().unwrap();
        let token = CancellationToken::new();
        let connection = Connection::new(
            Arc::new(SocketPair::from_stream(stream).unwrap()),
            Duration::from_millis(50),
            token.clone(),
        );
        let canceller = std::thread::spawn(move || {
            if cancel {
                std::thread::sleep(Duration::from_millis(10));
                token.cancel();
            }
        });
        let error = connection.command("never", json!({}), None).unwrap_err();
        if cancel {
            assert!(matches!(error, AutomationError::Cancellation));
        } else {
            assert!(matches!(error, AutomationError::Timeout(_)));
        }
        canceller.join().unwrap();
    }
}

#[cfg(windows)]
#[test]
fn named_pipe_retains_fragments_and_closes_without_blocking() {
    use brimp_controller::transport::NamedPipe;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, PIPE_TYPE_BYTE, PIPE_WAIT,
    };

    let name = format!(r"\\.\pipe\brimp-transport-test-{}", std::process::id());
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe {
        CreateNamedPipeW(
            wide.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_WAIT,
            1,
            4096,
            4096,
            0,
            std::ptr::null(),
        )
    };
    assert_ne!(handle, INVALID_HANDLE_VALUE);
    let mut peer = unsafe { std::fs::File::from_raw_handle(handle) };
    let transport = NamedPipe::from_file(
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&name)
            .unwrap(),
    )
    .unwrap();
    if unsafe { ConnectNamedPipe(peer.as_raw_handle(), std::ptr::null_mut()) } == 0 {
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(ERROR_PIPE_CONNECTED as i32)
        );
    }
    assert!(transport.receive().unwrap().is_none());
    peer.write_all(&[0, 0, 0, 3, b'a']).unwrap();
    assert!(transport.receive().unwrap().is_none());
    peer.write_all(b"bc").unwrap();
    assert_eq!(receive(&transport), b"abc");
    transport.send(b"reply").unwrap();
    let mut response = [0; 9];
    peer.read_exact(&mut response).unwrap();
    assert_eq!(&response, b"\0\0\0\x05reply");
    drop(peer);
    assert!(transport.receive().is_err());
    transport.close();
    transport.close();
    assert!(transport.send(b"x").is_err());
}

#[cfg(unix)]
#[test]
fn framed_stream_flushes_large_messages_under_backpressure() {
    use brimp_controller::transport::SocketPair;
    let (stream, mut peer) = std::os::unix::net::UnixStream::pair().unwrap();
    let transport = SocketPair::from_stream(stream).unwrap();
    let payload = vec![b'x'; 2 * 1024 * 1024];
    transport.send(&payload).unwrap();
    let server = std::thread::spawn(move || {
        peer.set_read_timeout(Some(TIMEOUT)).unwrap();
        let mut header = [0; 4];
        peer.read_exact(&mut header).unwrap();
        let mut bytes = vec![0; u32::from_be_bytes(header) as usize];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, payload);
        peer.write_all(b"\0\0\0\x02ok").unwrap();
    });
    assert_eq!(receive(&transport), b"ok");
    server.join().unwrap();
}
