use std::io::{Read, Write};

use serde_json::{Value, json};

#[cfg(unix)]
#[test]
fn executable_serves_cdp_over_the_inherited_framed_socket() {
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::process::{Command, Stdio};

    let (mut controller, worker_stream) = UnixStream::pair().unwrap();
    let descriptor = worker_stream.as_raw_fd();
    // SAFETY: fcntl only changes the close-on-exec flag of the valid socket above.
    assert_ne!(unsafe { libc::fcntl(descriptor, libc::F_SETFD, 0) }, -1);
    let child = Command::new(env!("CARGO_BIN_EXE_lite-worker"))
        .arg("--headless")
        .arg("--window-size=640,480")
        .arg(format!("--controller-socket-fd={descriptor}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(worker_stream);

    send(
        &mut controller,
        json!({"id":1,"method":"Browser.getVersion","params":{}}),
    );
    let version = receive(&mut controller);
    assert_eq!(version["id"], 1);
    assert_eq!(version["result"]["product"], "Brimp/0.1.0");

    send(
        &mut controller,
        json!({"id":2,"method":"Target.createTarget","params":{"url":"about:blank"}}),
    );
    let response = receive(&mut controller);
    assert_eq!(response["id"], 2);
    assert!(response["result"]["targetId"].is_string());

    drop(controller);
    let output = child.wait_with_output().unwrap();
    assert!(output.stdout.is_empty());
    assert!(
        !String::from_utf8(output.stderr)
            .unwrap()
            .contains("\"result\"")
    );
}

fn send(stream: &mut impl Write, value: Value) {
    let payload = serde_json::to_vec(&value).unwrap();
    stream
        .write_all(&(payload.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&payload).unwrap();
}

fn receive(stream: &mut impl Read) -> Value {
    let mut header = [0; 4];
    stream.read_exact(&mut header).unwrap();
    let mut payload = vec![0; u32::from_be_bytes(header) as usize];
    stream.read_exact(&mut payload).unwrap();
    serde_json::from_slice(&payload).unwrap()
}
