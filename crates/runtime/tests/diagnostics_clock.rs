use brimp_runtime::runtime::{
    clock::{ClockOptions, TimeMode},
    diagnostics::{DiagnosticsOptions, TraceMode},
    native::{Session, SessionOptions},
};
use serde_json::json;
use std::time::{Duration, Instant};

fn logical(diagnostics: Option<DiagnosticsOptions>) -> Session {
    Session::new(SessionOptions {
        clock: ClockOptions {
            mode: TimeMode::Logical,
            epoch_ms: Some(1000),
        },
        diagnostics,
        timeout: Duration::from_secs(2),
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn logical_clock_orders_timers_and_microtasks_without_sleeping() {
    let session = logical(None);
    let page = session.new_page().unwrap();
    page.evaluate("globalThis.log=[]; setTimeout(()=>{log.push(['a',performance.now(),Date.now(),+new Date()]); Promise.resolve().then(()=>log.push('micro')); setTimeout(()=>log.push('nested'),5)},10); setTimeout(()=>log.push('b'),20); null").unwrap();
    assert_eq!(page.evaluate("performance.now()").unwrap(), json!(0));
    page.advance_time(20.0, 100).unwrap();
    assert_eq!(
        page.evaluate("log").unwrap(),
        json!([["a", 10, 1010, 1010], "micro", "nested", "b"])
    );
    assert_eq!(
        page.evaluate("[performance.now(),performance.timeOrigin,Date.now()]")
            .unwrap(),
        json!([20, 1000, 1020])
    );
    let start = Instant::now();
    assert_eq!(
        page.evaluate("new Promise(resolve=>setTimeout(()=>resolve(performance.now()),5000))")
            .unwrap(),
        json!(5020)
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(page.advance_time(-1.0, 100).is_err());
    assert!(page.advance_time(f64::INFINITY, 100).is_err());
    let other = session.new_page().unwrap();
    assert_eq!(other.evaluate("performance.now()").unwrap(), json!(0));
}
#[test]
fn logical_intervals_are_bounded_and_drain_does_not_advance() {
    let session = logical(None);
    let page = session.new_page().unwrap();
    page.evaluate("globalThis.n=0; globalThis.timer=setInterval(()=>n++,10); null")
        .unwrap();
    page.run_pending_tasks(100).unwrap();
    assert_eq!(page.evaluate("n").unwrap(), json!(0));
    page.advance_time(30.0, 100).unwrap();
    assert_eq!(page.evaluate("n").unwrap(), json!(3));
    assert!(
        page.advance_time(10000.0, 5)
            .unwrap_err()
            .to_string()
            .contains("budget")
    );
    page.evaluate("clearInterval(timer); null").unwrap();
    assert!(page.advance_time(10.0, 10).is_ok());
}
#[test]
fn diagnostic_records_filter_bound_and_do_not_read_getters() {
    let path = std::env::temp_dir().join(format!("brimp-trace-{}.jsonl", std::process::id()));
    let session = logical(Some(DiagnosticsOptions {
        path: Some(path.to_string_lossy().into()),
        include: vec!["Document.prototype".into(), "Reflect".into()],
        capacity: 3,
        stack: true,
        ..Default::default()
    }));
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("globalThis.getterCalls=0; const x={get trap(){getterCalls++;throw Error('no')}}; Reflect.ownKeys(x); for(let i=0;i<8;i++) document.querySelector('missing'); getterCalls").unwrap(),json!(0));
    let batch = page.take_diagnostics().unwrap();
    assert_eq!(batch["records"].as_array().unwrap().len(), 3);
    assert!(batch["dropped"].as_u64().unwrap() >= 6);
    assert!(batch["records"].as_array().unwrap().iter().all(|r| {
        r["path"]
            .as_str()
            .unwrap()
            .starts_with("Document.prototype")
            && r["stack"].is_string()
    }));
    assert!(
        page.take_diagnostics().unwrap()["records"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    session.close();
    let lines = std::fs::read_to_string(&path).unwrap();
    assert!(lines.lines().count() >= 9);
    for line in lines.lines() {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
    std::fs::remove_file(path).unwrap();
}
#[test]
fn real_time_promises_wake_owner_and_clock_is_opt_in() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    assert!(page.advance_time(1.0, 100).is_err());
    assert_eq!(
        page.evaluate("new Promise(resolve=>setTimeout(()=>resolve(42),15))")
            .unwrap(),
        json!(42)
    );
    assert!(
        page.take_diagnostics().unwrap()["records"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn diagnostic_serialization_does_not_invoke_inherited_to_json() {
    let session = logical(Some(DiagnosticsOptions::default()));
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("globalThis.serialized=0; Object.defineProperty(Object.prototype,'toJSON',{configurable:true,value(){serialized++;throw Error('unexpected serialization')}}); document.querySelector('missing'); delete Object.prototype.toJSON; serialized").unwrap(),json!(0));
    let batch = page.take_diagnostics().unwrap();
    assert!(
        batch["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["details"]["property"] == "toJSON")
    );
}

#[test]
fn navigation_resets_clock_and_tracing_tracks_realms() {
    use async_trait::async_trait;
    use brimp_runtime::network::*;
    struct Fixture;
    #[async_trait]
    impl ResourceLoader for Fixture {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            Ok(ResourceResponse {
                status: http::StatusCode::OK,
                headers: Default::default(),
                body:
                    b"<script>globalThis.atStart=Date.now();document.querySelector('body')</script>"
                        .to_vec(),
                effective_url: request.url,
                metadata: Default::default(),
            })
        }
    }
    let session = Session::with_resource_loader(
        SessionOptions {
            clock: ClockOptions {
                mode: TimeMode::Logical,
                epoch_ms: Some(2000),
            },
            diagnostics: Some(DiagnosticsOptions::default()),
            ..Default::default()
        },
        std::sync::Arc::new(Fixture),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    page.evaluate("document.querySelector('body');null")
        .unwrap();
    let old = page.take_diagnostics().unwrap()["records"][0]["realmId"].clone();
    page.advance_time(100.0, 100).unwrap();
    page.request("GET", "https://fixture.invalid/", Default::default())
        .unwrap();
    assert_eq!(
        page.evaluate("[atStart,performance.now()]").unwrap(),
        json!([2000, 0])
    );
    let batch = page.take_diagnostics().unwrap();
    assert!(
        batch["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["realmId"] != old)
    );
}

#[test]
fn diagnostic_summary_separates_layers_and_reflection_dependencies() {
    let session = logical(Some(DiagnosticsOptions {
        include: vec![
            "Document.prototype.querySelector".into(),
            "native.querySelector".into(),
            "Reflect.has".into(),
        ],
        ..Default::default()
    }));
    let page = session.new_page().unwrap();
    page.take_diagnostics().unwrap();
    page.evaluate(
        r#"
        document.querySelector('body'); document.querySelector('missing');
        try { document.querySelector('['); } catch (_) {}
        Reflect.has(navigator, 'userAgent'); Reflect.has(document, 'body');
        globalThis.unobserved = {value: 1}; unobserved.value; unobserved.value = 2;
        void window.missing; void ('missing' in window); delete window.missing;
        null;
    "#,
    )
    .unwrap();
    let before = page.peek_diagnostics().unwrap();
    let summary = page.summarize_diagnostics().unwrap();
    assert_eq!(before, page.peek_diagnostics().unwrap());
    assert_eq!(summary["retained"], 8);
    assert_eq!(summary["dropped"], 0);
    assert_eq!(summary["coverage"], before["coverage"]);
    assert_eq!(summary["coverage"]["enabled"], true);
    assert!(
        !summary["coverage"]["limitations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let dependencies = summary["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 4);
    for layer in ["api", "native"] {
        let entry = dependencies
            .iter()
            .find(|entry| entry["layer"] == layer)
            .unwrap();
        assert_eq!(entry["count"], 3);
        assert_eq!(entry["throws"], 1);
        assert!(entry["target"].is_null());
    }
    for (target, property) in [("navigator", "userAgent"), ("document", "body")] {
        assert!(
            dependencies
                .iter()
                .any(|entry| entry["layer"] == "reflection"
                    && entry["path"] == "Reflect.has"
                    && entry["target"] == target
                    && entry["property"] == property
                    && entry["count"] == 1
                    && entry["throws"] == 0)
        );
    }
    assert_eq!(before, page.take_diagnostics().unwrap());
    assert_eq!(page.summarize_diagnostics().unwrap()["retained"], 0);
}

#[test]
fn diagnostic_inspection_preserves_loss_and_distinguishes_disabled_from_empty() {
    let session = logical(Some(DiagnosticsOptions {
        capacity: 3,
        include: vec!["Document.prototype.querySelector".into()],
        ..Default::default()
    }));
    let page = session.new_page().unwrap();
    page.take_diagnostics().unwrap();
    page.evaluate("for(let i=0;i<5;i++) document.querySelector('body'); null")
        .unwrap();
    let batch = page.peek_diagnostics().unwrap();
    let summary = page.summarize_diagnostics().unwrap();
    assert_eq!(summary["dropped"], 2);
    assert_eq!(summary["retained"], 3);
    assert_eq!(summary["dependencies"][0]["count"], 3);
    assert_eq!(
        summary["coverage"]["include"],
        json!(["Document.prototype.querySelector"])
    );
    assert_eq!(batch, page.take_diagnostics().unwrap());
    assert_eq!(page.peek_diagnostics().unwrap()["dropped"], 0);
    assert_eq!(
        page.summarize_diagnostics().unwrap()["coverage"]["enabled"],
        true
    );

    let disabled = logical(None);
    let page = disabled.new_page().unwrap();
    assert_eq!(
        page.peek_diagnostics().unwrap(),
        page.take_diagnostics().unwrap()
    );
    let summary = page.summarize_diagnostics().unwrap();
    assert_eq!(summary["coverage"]["enabled"], false);
    assert_eq!(summary["dependencies"], json!([]));
    assert_eq!(summary["unresolved"]["reads"], json!([]));
    assert_eq!(summary["unresolved"]["operations"], json!([]));
    assert_eq!(summary["retained"], 0);
}

#[test]
fn recursive_tracing_reinstalls_on_navigation_and_keeps_page_sequences() {
    use async_trait::async_trait;
    use brimp_runtime::network::*;
    struct Fixture;
    #[async_trait]
    impl ResourceLoader for Fixture {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            Ok(ResourceResponse {
                status: http::StatusCode::OK,
                headers: Default::default(),
                body: b"<body><script>window.fixture={nested:{}};void window.fixture.nested.missing;document.body.dataset.traced='yes'</script>".to_vec(),
                effective_url: request.url,
                metadata: Default::default(),
            })
        }
    }
    let session = Session::with_resource_loader(
        SessionOptions {
            diagnostics: Some(DiagnosticsOptions {
                mode: TraceMode::Recursive,
                ..Default::default()
            }),
            ..Default::default()
        },
        std::sync::Arc::new(Fixture),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    page.evaluate("void window.beforeNavigation;null").unwrap();
    let before = page.peek_diagnostics().unwrap();
    let last = before["records"].as_array().unwrap().last().unwrap();
    page.request("GET", "https://fixture.invalid/", Default::default())
        .unwrap();
    assert_eq!(
        page.evaluate("document.body.dataset.traced").unwrap(),
        json!("yes")
    );
    let batch = page.peek_diagnostics().unwrap();
    assert_eq!(batch["coverage"]["mode"], "recursive");
    let missing = batch["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == "window.fixture.nested.missing")
        .unwrap();
    assert_ne!(missing["realmId"], last["realmId"]);
    assert!(missing["sequence"].as_u64().unwrap() > last["sequence"].as_u64().unwrap());
    assert_eq!(missing["details"]["present"], json!(false));

    page.evaluate("void navigator.missing; null").unwrap();
    let summary = page.summarize_diagnostics().unwrap();
    let read = |path: &str| {
        summary["unresolved"]["reads"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["path"] == path)
            .unwrap()
    };
    let page_read = read("window.fixture.nested.missing");
    assert_eq!(page_read["present"], json!(false));
    assert_eq!(page_read["owner"], json!("unknown"));
    let navigator_read = read("navigator.missing");
    assert_eq!(navigator_read["present"], json!(false));
    assert_eq!(navigator_read["owner"], json!("profile"));
}
