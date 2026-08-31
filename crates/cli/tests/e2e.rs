use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_brimp"))
}

#[test]
fn help_dispatches_to_each_public_command() {
    for (command, expected) in [
        ("get", "usage: brimp get"),
        ("crawl", "usage: brimp crawl"),
        ("cdp", "usage: brimp cdp"),
        ("doctor", "usage: brimp doctor"),
    ] {
        let output = binary().args(["help", command]).output().unwrap();
        assert!(output.status.success(), "{command}");
        assert!(
            String::from_utf8_lossy(&output.stdout).starts_with(expected),
            "{command}"
        );
        assert!(output.stderr.is_empty(), "{command}");
    }
}

#[test]
fn parser_rejects_conflicts_duplicates_and_ambiguous_stdout_before_launch() {
    for arguments in [
        vec![
            "get",
            "http://unused.test/",
            "--eval",
            "1",
            "--output",
            "x.json",
        ],
        vec!["get", "http://unused.test/", "--output", "-"],
        vec![
            "get",
            "http://unused.test/",
            "--timeout",
            "1s",
            "--timeout",
            "2s",
        ],
        vec!["crawl", "http://unused.test/", "--unknown"],
    ] {
        let output = binary().args(arguments).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn cdp_serves_from_the_brimp_subcommand() {
    let mut child = binary()
        .args(["cdp", "--bind", "0.0.0.0:0", "--allow-non-loopback"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut endpoint = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut endpoint)
        .unwrap();
    assert!(endpoint.starts_with("ws://0.0.0.0:"));
    #[cfg(unix)]
    unsafe {
        unsafe extern "C" {
            fn kill(process: i32, signal: i32) -> i32;
        }
        assert_eq!(kill(child.id() as i32, 2), 0);
    }
    #[cfg(not(unix))]
    child.kill().unwrap();
    let status = child.wait().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(2));
    }
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stderr.starts_with("WARNING: Brimp CDP is binding to non-loopback address"));
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
fn get_evaluation_keeps_structured_data_on_stdout() {
    let (url, worker) = server(b"<!doctype html><title>CLI</title>");
    let output = binary()
        .args([
            "get",
            &url,
            "--eval",
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
fn get_inserts_cookie_option_into_the_browser_jar() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (observed, request) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0; 4096];
        let read = stream.read(&mut bytes).unwrap();
        observed
            .send(String::from_utf8_lossy(&bytes[..read]).into_owned())
            .unwrap();
        let body = b"<!doctype html><title>Cookie</title>";
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
    });
    let output = binary()
        .args([
            "get",
            &format!("http://{address}/"),
            "--cookie",
            "agent=ready",
            "--eval",
            "document.cookie",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<String>(&output.stdout).unwrap(),
        "agent=ready"
    );
    let request = request.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(request.to_ascii_lowercase().contains("cookie: agent=ready"));
}

#[test]
fn get_page_options_enable_opt_in_subsystems() {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let storage = std::env::temp_dir().join(format!("brimp-cli-storage-{unique}"));
    let (url, worker) = server(b"<!doctype html><title>Options</title>");
    let output = binary()
        .args([
            "get",
            &url,
            "--enable-worker",
            "--enable-streaming-networking",
            "--storage-path",
            storage.to_str().unwrap(),
            "--eval",
            "localStorage.enabled = 'yes'; [typeof Worker, typeof WebSocket, typeof indexedDB, typeof navigator.storage]",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!(["function", "function", "object", "object"])
    );
    if storage.exists() {
        std::fs::remove_dir_all(storage).unwrap();
    }
}

#[test]
fn get_loads_the_versioned_persona_schema() {
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
            "get",
            &url,
            "--persona",
            path.to_str().unwrap(),
            "--eval",
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
fn get_writes_screenshot_binary_and_protects_existing_output() {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brimp-cli-{unique}.png"));
    let (url, worker) = server(b"<!doctype html><body style='background:red'>shot</body>");
    let output = binary()
        .args(["get", &url, "--output", path.to_str().unwrap()])
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
            "get",
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
fn get_preserves_binary_raw_output_through_the_configured_proxy() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (observed, request) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0; 4096];
        let read = stream.read(&mut bytes).unwrap();
        observed
            .send(String::from_utf8_lossy(&bytes[..read]).into_owned())
            .unwrap();
        let body = b"\0brimp\xffraw";
        let head = format!(
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
    });
    let output = binary()
        .args([
            "get",
            "http://example.invalid/archive",
            "--proxy",
            &format!("http://{address}"),
            "--format",
            "raw",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"\0brimp\xffraw");
    assert!(
        request
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .starts_with("GET http://example.invalid/archive ")
    );
}

#[test]
fn get_extracts_javascript_rendered_markdown_and_protects_existing_output() {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("brimp-cli-{unique}.md"));
    let body = br#"<!doctype html>
        <title>Live article</title>
        <article id="story"><h1>Live article</h1><p>Before</p></article>
        <script>document.querySelector('p').textContent = 'Rendered by JavaScript';</script>"#;
    let (url, worker) = server(body);
    let output = binary()
        .args([
            "get",
            &url,
            "--content",
            "#story",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let markdown = std::fs::read_to_string(&path).unwrap();
    assert!(markdown.contains("Rendered by JavaScript"), "{markdown}");

    let output = binary()
        .args([
            "get",
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
fn get_serializes_prepared_html_and_structured_extraction_json() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 4096];
            let _ = stream.read(&mut bytes);
            let body =
                b"<!doctype html><title>Structured</title><article><p>Original</p></article>";
            let head = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body).unwrap();
        }
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script = std::env::temp_dir().join(format!("brimp-get-{unique}.js"));
    fs::write(
        &script,
        "document.querySelector('p').textContent = 'Prepared'",
    )
    .unwrap();
    let url = format!("http://{address}/");
    let html = binary()
        .args([
            "get",
            &url,
            "--wait",
            "networkidle",
            "--network-idle",
            "10ms",
            "--wait-selector",
            "article",
            "--script",
            script.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(html.status.success());
    assert!(String::from_utf8_lossy(&html.stdout).contains("Prepared"));

    let json = binary()
        .args([
            "get",
            &url,
            "--format",
            "json",
            "--content",
            "article",
            "--extract-debug",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(json.status.success());
    let document: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(document["title"], "Structured");
    assert!(
        document["contentMarkdown"]
            .as_str()
            .unwrap()
            .contains("Original")
    );
    assert!(document["debug"].is_object());
    assert!(json.stderr.is_empty());
    fs::remove_file(script).unwrap();
}

#[test]
fn crawl_is_bounded_deterministic_and_writes_a_terminal_manifest() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 4096];
            let read = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..read]);
            let path = request.split_whitespace().nth(1).unwrap();
            let (content_type, body) = match path {
                "/robots.txt" => ("text/plain", "User-agent: *\nAllow: /\n".to_owned()),
                "/" => (
                    "text/html",
                    "<!doctype html><title>Root</title><article><h1>Root</h1><p>Root page</p><a href='/b'>B</a><a href='/a'>A</a></article>".to_owned(),
                ),
                "/a" => ("text/html", "<!doctype html><title>A</title><article><h1>A</h1><p>Page A</p></article>".to_owned()),
                "/b" => ("text/html", "<!doctype html><title>B</title><article><h1>B</h1><p>Page B</p></article>".to_owned()),
                _ => unreachable!("unexpected crawl path {path}"),
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body.as_bytes()).unwrap();
        }
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = std::env::temp_dir().join(format!("brimp-crawl-{unique}"));
    let script = std::env::temp_dir().join(format!("brimp-crawl-{unique}.js"));
    fs::write(
        &script,
        "document.querySelector('article').append(' Prepared')",
    )
    .unwrap();
    let output = binary()
        .args([
            "crawl",
            &format!("http://{address}/"),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--depth",
            "1",
            "--max-pages",
            "3",
            "--workers",
            "2",
            "--content",
            "article",
            "--wait-selector",
            "article",
            "--script",
            script.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let root_markdown = fs::read_to_string(output_dir.join("index.md")).unwrap();
    assert!(root_markdown.contains("Root page"), "{root_markdown}");
    assert!(root_markdown.contains("Prepared"), "{root_markdown}");
    assert!(
        fs::read_to_string(output_dir.join("a.md"))
            .unwrap()
            .contains("Page A")
    );
    assert!(
        fs::read_to_string(output_dir.join("b.md"))
            .unwrap()
            .contains("Page B")
    );
    let records = fs::read_to_string(output_dir.join("manifest.jsonl")).unwrap();
    let records = records
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["depth"], 0);
    assert!(records.iter().all(|record| record["ok"] == true));
    fs::remove_dir_all(output_dir).unwrap();
    fs::remove_file(script).unwrap();
}

#[test]
fn crawl_obeys_robots_scope_and_fragment_deduplication() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 4096];
            let read = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..read]);
            let path = request.split_whitespace().nth(1).unwrap();
            let (content_type, body) = match path {
                "/robots.txt" => ("text/plain", "User-agent: *\nDisallow: /private\n"),
                "/" => (
                    "text/html",
                    "<!doctype html><article><p>Root</p><a href='/public#one'>One</a><a href='/public#two'>Two</a><a href='/private'>Private</a><a href='/excluded'>Excluded</a><a href='https://outside.invalid/page'>Outside</a></article>",
                ),
                "/public" => ("text/html", "<!doctype html><article>Public</article>"),
                _ => panic!("unexpected crawl request {path}"),
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body.as_bytes()).unwrap();
        }
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = std::env::temp_dir().join(format!("brimp-crawl-policy-{unique}"));
    let output = binary()
        .args([
            "crawl",
            &format!("http://{address}/"),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--depth",
            "1",
            "--max-pages",
            "10",
            "--delay",
            "10ms",
            "--include",
            "/**",
            "--exclude",
            "/excluded",
            "--content",
            "article",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = fs::read_to_string(output_dir.join("manifest.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 3);
    assert_eq!(
        records
            .iter()
            .filter(|record| record["url"].as_str().unwrap().ends_with("/public"))
            .count(),
        1
    );
    let private = records
        .iter()
        .find(|record| record["url"].as_str().unwrap().ends_with("/private"))
        .unwrap();
    assert_eq!(private["skipped"], "robots");
    assert!(
        !records
            .iter()
            .any(|record| record["url"].as_str().unwrap().contains("outside.invalid"))
    );
    assert!(
        !records
            .iter()
            .any(|record| record["url"].as_str().unwrap().ends_with("/excluded"))
    );
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn crawl_records_script_failures_and_requires_allow_errors_for_success() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 4096];
            let read = stream.read(&mut bytes).unwrap();
            let request = String::from_utf8_lossy(&bytes[..read]);
            let path = request.split_whitespace().nth(1).unwrap();
            let body = if path == "/robots.txt" {
                "User-agent: *\nAllow: /\n"
            } else {
                "<!doctype html><article>Page</article>"
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(body.as_bytes()).unwrap();
        }
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script = std::env::temp_dir().join(format!("brimp-crawl-failure-{unique}.js"));
    fs::write(&script, "throw new Error('preparation failed')").unwrap();
    let url = format!("http://{address}/");
    for (suffix, allow_errors) in [("strict", false), ("allowed", true)] {
        let output_dir =
            std::env::temp_dir().join(format!("brimp-crawl-failure-{unique}-{suffix}"));
        let mut arguments = vec![
            "crawl",
            &url,
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--script",
            script.to_str().unwrap(),
        ];
        if allow_errors {
            arguments.push("--allow-errors");
        }
        let output = binary().args(arguments).output().unwrap();
        assert_eq!(output.status.success(), allow_errors);
        let manifest = fs::read_to_string(output_dir.join("manifest.jsonl")).unwrap();
        let record: serde_json::Value = serde_json::from_str(manifest.trim()).unwrap();
        assert_eq!(record["ok"], false);
        assert!(
            record["error"]
                .as_str()
                .unwrap()
                .contains("preparation failed")
        );
        fs::remove_dir_all(output_dir).unwrap();
    }
    worker.join().unwrap();
    fs::remove_file(script).unwrap();
}

#[cfg(unix)]
#[test]
fn interrupt_cancels_crawl_and_finishes_its_manifest() {
    unsafe extern "C" {
        fn kill(process: i32, signal: i32) -> i32;
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (page_started, waiting) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let (mut robots, _) = listener.accept().unwrap();
        let mut bytes = [0; 4096];
        let _ = robots.read(&mut bytes);
        let body = b"User-agent: *\nAllow: /\n";
        let head = format!(
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        robots.write_all(head.as_bytes()).unwrap();
        robots.write_all(body).unwrap();

        let (mut page, _) = listener.accept().unwrap();
        let _ = page_started.send(());
        let _ = page.read(&mut bytes);
        page.set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        assert_eq!(page.read(&mut bytes).unwrap_or(0), 0);
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = std::env::temp_dir().join(format!("brimp-crawl-cancel-{unique}"));
    let mut child = binary()
        .args([
            "crawl",
            &format!("http://{address}/"),
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    unsafe {
        assert_eq!(kill(child.id() as i32, 2), 0);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(15));
    worker.join().unwrap();
    let records = fs::read_to_string(output_dir.join("manifest.jsonl")).unwrap();
    let records = records.lines().collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let record: serde_json::Value = serde_json::from_str(records[0]).unwrap();
    assert_eq!(record["ok"], false);
    assert!(record["error"].as_str().unwrap().contains("cancel"));
    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn timeout_uses_stable_exit_category() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut bytes = [0; 1024];
        let _ = stream.read(&mut bytes);
        std::thread::sleep(Duration::from_secs(1));
    });
    let output = binary()
        .args([
            "get",
            &format!("http://{address}/"),
            "--eval",
            "1",
            "--timeout",
            "500ms",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert_eq!(output.status.code(), Some(14));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("timed out"));
}

#[test]
fn crawl_timeout_uses_stable_exit_category_and_terminal_manifest() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = [0; 1024];
        let _ = stream.read(&mut bytes);
        std::thread::sleep(Duration::from_secs(1));
    });
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = std::env::temp_dir().join(format!("brimp-crawl-timeout-{unique}"));
    let output = binary()
        .args([
            "crawl",
            &format!("http://{address}/"),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--ignore-robots",
            "--timeout",
            "500ms",
        ])
        .output()
        .unwrap();
    worker.join().unwrap();
    assert_eq!(output.status.code(), Some(14));
    let manifest = fs::read_to_string(output_dir.join("manifest.jsonl")).unwrap();
    let record: serde_json::Value = serde_json::from_str(manifest.trim()).unwrap();
    assert_eq!(record["ok"], false);
    assert!(record["error"].as_str().unwrap().contains("timed out"));
    fs::remove_dir_all(output_dir).unwrap();
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
        .args(["get", &format!("http://{address}/"), "--eval", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    unsafe {
        assert_eq!(kill(child.id() as i32, 2), 0);
    }
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(15));
    worker.join().unwrap();
}
