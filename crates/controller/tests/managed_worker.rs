#![cfg(target_os = "macos")]
use brimp_controller::{
    AutomationError, CancellationToken,
    browser::{Browser, WorkerOptions, WorkerProtocol},
};
use std::time::Duration;

#[test]
fn browser_owns_websocket_process_even_when_page_handles_survive() {
    let directory = tempfile::tempdir().unwrap();
    let record = directory.path().join("worker.json");
    let options = WorkerOptions {
        path: "/usr/bin/python3".into(),
        protocol: WorkerProtocol::Cdp,
        arguments: vec![
            format!("{}/tests/fake_cdp_worker.py", env!("CARGO_MANIFEST_DIR")),
            format!("--record={}", record.display()),
            "--remote-debugging-port={port}".into(),
        ],
    };
    let browser = Browser::launch(
        &options,
        Default::default(),
        Duration::from_secs(10),
        CancellationToken::new(),
        false,
    )
    .unwrap();
    let page = browser.new_page().unwrap();
    assert_eq!(page.evaluate("document.title").unwrap(), "Managed");
    let worker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record).unwrap()).unwrap();
    let pid = worker["pid"].as_i64().unwrap() as i32;
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0);
    browser.close();
    browser.close();
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    assert!(matches!(page.evaluate("1"), Err(AutomationError::Closed)));
    drop(browser);
    assert!(!std::path::Path::new(worker["profile"].as_str().unwrap()).exists());
}
