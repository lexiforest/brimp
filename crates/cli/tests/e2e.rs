use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_brimp"))
}
fn server(body: &'static [u8]) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0; 4096];
        let _ = stream.read(&mut bytes);
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
    });
    (format!("http://{address}/"), worker)
}

#[test]
fn doctor_reports_native_dependencies_as_json() {
    let output = binary().arg("doctor").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["javascriptCore"], "ok");
    assert_eq!(value["libcurlImpersonate"], "ok");
    assert!(output.stderr.is_empty());
}

#[test]
fn eval_keeps_structured_data_on_stdout() {
    let (url, worker) = server(b"<!doctype html><title>CLI</title>");
    let output = binary()
        .args([
            "eval",
            &url,
            "--js",
            "({ title: document.title, answer: 42 })",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value, serde_json::json!({"title":"CLI", "answer":42}));
    assert!(output.stderr.is_empty());
}

#[test]
fn eval_loads_the_versioned_persona_schema() {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brimp-persona-{unique}.json"));
    std::fs::write(
        &path,
        r#"{
            "schema_version": 1,
            "transport": { "impersonation_profile": "chrome150" },
            "network": { "user_agent": "CLI Persona/1" },
            "navigator": { "platform": "CLI-OS", "hardware_concurrency": 6 },
            "viewport": { "width": 640, "height": 480, "device_scale_factor": 2 }
        }"#,
    )
    .unwrap();
    let (url, worker) = server(b"<!doctype html><title>Persona</title>");
    let output = binary()
        .args([
            "eval",
            &url,
            "--persona",
            path.to_str().unwrap(),
            "--js",
            "({ userAgent: navigator.userAgent, platform: navigator.platform, concurrency: navigator.hardwareConcurrency, viewport: [innerWidth, innerHeight, devicePixelRatio] })",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "userAgent": "CLI Persona/1",
            "platform": "CLI-OS",
            "concurrency": 6,
            "viewport": [640, 480, 2]
        })
    );
}

#[test]
fn screenshot_writes_binary_and_protects_existing_output() {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brimp-cli-{unique}.png"));
    let (url, worker) = server(b"<!doctype html><body style='background:red'>shot</body>");
    let output = binary()
        .args(["screenshot", &url, "--output", path.to_str().unwrap()])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        std::fs::read(&path)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    let output = binary()
        .args([
            "screenshot",
            "http://unused.test/",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--overwrite"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn timeout_uses_stable_exit_category() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0; 1024];
        let _ = stream.read(&mut bytes);
        std::thread::sleep(Duration::from_millis(200));
    });
    let output = binary()
        .args([
            "eval",
            &format!("http://{address}/"),
            "--js",
            "1",
            "--timeout-ms",
            "20",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert_eq!(output.status.code(), Some(14));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("timed out"));
}

#[cfg(unix)]
#[test]
fn interrupt_cancels_navigation_and_releases_the_request() {
    unsafe extern "C" {
        fn kill(process: i32, signal: i32) -> i32;
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (accepted, waiting) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = accepted.send(());
        let mut bytes = [0; 1024];
        let _ = stream.read(&mut bytes);
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        assert_eq!(stream.read(&mut bytes).unwrap_or(0), 0);
    });
    let mut child = binary()
        .args(["eval", &format!("http://{address}/"), "--js", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    waiting.recv_timeout(Duration::from_secs(2)).unwrap();
    unsafe {
        assert_eq!(kill(child.id() as i32, 2), 0);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(15));
    worker.join().unwrap();
}
