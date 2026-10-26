use brimp_runtime::runtime::{
    native::{Session, SessionOptions},
    plugins::{BridgeValue, HostHandler, Plugin},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

fn threaded(
    operation: impl Fn(brimp_runtime::runtime::plugins::HostInvocation) + Send + Sync + 'static,
) -> HostHandler {
    let operation = Arc::new(operation);
    Arc::new(move |invocation| {
        let operation = operation.clone();
        std::thread::spawn(move || operation(invocation));
    })
}
#[test]
fn ordered_async_installation_copied_binary_notifications_and_disposal() {
    let disposals = Arc::new(AtomicUsize::new(0));
    let count = disposals.clone();
    let plugin = Plugin::new(
        "bridge",
        r#"async ({global,host}) => {
        global.order = ['start'];
        global.echo = value => host.call('echo', value);
        global.messages = [];
        const off = host.subscribe('message', value => messages.push(value));
        global.unsubscribe = off;
        global.fire = () => host.call('fire');
        global.bad = () => host.call('bad');
        global.missing = () => host.call('unknown');
        await new Promise(resolve => setTimeout(resolve, 2));
        global.initial = await host.call('echo', new Uint8Array([0,1,255]));
        order.push('ready');
        return async () => { off(); await host.call('dispose'); };
    }"#,
    )
    .with_handler(
        "echo",
        threaded(|mut invocation| {
            invocation
                .completion
                .resolve(invocation.arguments.remove(0))
        }),
    )
    .with_handler(
        "fire",
        threaded(|invocation| {
            for value in 0..3 {
                invocation
                    .context
                    .emit("message", BridgeValue::Number(value as f64))
                    .unwrap();
            }
            invocation.completion.resolve(BridgeValue::Null);
        }),
    )
    .with_handler(
        "bad",
        threaded(|invocation| invocation.completion.reject("original failure")),
    )
    .with_handler(
        "dispose",
        threaded(move |invocation| {
            count.fetch_add(1, Ordering::SeqCst);
            invocation.completion.resolve(BridgeValue::Null);
        }),
    );
    let next = Plugin::new(
        "second",
        "({global}) => { if (global.order.join(',') !== 'start,ready') throw new Error('wrong order'); order.push('second'); }",
    );
    let session = Session::new(SessionOptions {
        plugins: vec![plugin, next],
        timeout: Duration::from_secs(2),
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(
        page.evaluate("order.join(',')").unwrap(),
        "start,ready,second"
    );
    assert_eq!(
        page.evaluate("Array.from(initial)").unwrap(),
        serde_json::json!([0, 1, 255])
    );
    assert_eq!(page.evaluate("echo({buffer:new Uint8Array([7,8]), nested:[null,true]}).then(value => [Array.from(value.buffer),value.nested])").unwrap(), serde_json::json!([[7,8],[null,true]]));
    page.evaluate("fire()").unwrap();
    assert_eq!(
        page.evaluate("messages").unwrap(),
        serde_json::json!([0, 1, 2])
    );
    assert_eq!(
        page.evaluate("bad().catch(String)").unwrap(),
        "OperationError: The operation failed."
    );
    assert!(
        page.evaluate("missing().catch(String)")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("OperationError")
    );
    page.close().unwrap();
    assert_eq!(disposals.load(Ordering::SeqCst), 1);
}
#[test]
fn installation_failure_names_the_plugin_and_original_error() {
    let session = Session::new(SessionOptions {
        plugins: vec![Plugin::new(
            "broken",
            "async () => { throw new Error('original detail'); }",
        )],
        ..Default::default()
    })
    .unwrap();
    let error = match session.new_page() {
        Ok(_) => panic!("installation should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("broken"), "{error}");
    assert!(error.contains("original detail"), "{error}");
}

struct Fixture;
#[async_trait::async_trait]
impl brimp_runtime::network::ResourceLoader for Fixture {
    async fn fetch(
        &self,
        request: brimp_runtime::network::ResourceRequest,
    ) -> Result<brimp_runtime::network::ResourceResponse, brimp_runtime::network::NetworkError>
    {
        Ok(brimp_runtime::network::ResourceResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            body:
                b"<script>globalThis.scriptSawPlugin = typeof pendingCall === 'function';</script>"
                    .to_vec(),
            effective_url: request.url,
            metadata: Default::default(),
        })
    }
}
#[test]
fn navigation_and_close_cancel_pending_calls_and_discard_late_results() {
    use brimp_runtime::runtime::{AutomationError, native::RequestOptions};
    let (sender, receiver) = std::sync::mpsc::channel();
    let plugin = Plugin::new(
        "pending",
        "({global,host}) => { global.pendingCall = () => host.call('pending'); }",
    )
    .with_handler(
        "pending",
        Arc::new(move |invocation| {
            sender.send(invocation).unwrap();
        }),
    );
    let session = Session::with_resource_loader(
        SessionOptions {
            plugins: vec![plugin],
            timeout: Duration::from_secs(2),
            ..Default::default()
        },
        Arc::new(Fixture),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    let evaluating = page.clone();
    let evaluation = std::thread::spawn(move || evaluating.evaluate("pendingCall()"));
    let pending = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
    page.request("GET", "https://plugin.test/", RequestOptions::default())
        .unwrap();
    assert!(matches!(
        evaluation.join().unwrap(),
        Err(AutomationError::Cancellation)
    ));
    assert!(pending.context.is_cancelled());
    pending
        .completion
        .resolve(BridgeValue::String("late result".into()));
    assert_eq!(page.evaluate("scriptSawPlugin").unwrap(), true);
    let evaluating = page.clone();
    let evaluation = std::thread::spawn(move || evaluating.evaluate("pendingCall()"));
    let pending = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
    page.close().unwrap();
    assert!(matches!(
        evaluation.join().unwrap(),
        Err(AutomationError::Closed)
    ));
    assert!(pending.context.is_cancelled());
    pending.completion.resolve(BridgeValue::Null);
}
#[test]
fn callback_reentry_rejects_without_poisoning_page() {
    use brimp_runtime::runtime::{AutomationError, native::NativePage};
    let target = Arc::new(std::sync::Mutex::new(None::<NativePage>));
    let callback_target = target.clone();
    let plugin = Plugin::new(
        "reentry",
        "({global,host}) => { global.reenter = () => host.call('again'); }",
    )
    .with_handler(
        "again",
        threaded(move |invocation| {
            let _guard = invocation.context.enter_callback();
            let page = callback_target.lock().unwrap().as_ref().unwrap().clone();
            let result = page.evaluate("1 + 1");
            invocation.completion.resolve(BridgeValue::Boolean(matches!(
                result,
                Err(AutomationError::Reentry)
            )));
        }),
    );
    let session = Session::new(SessionOptions {
        plugins: vec![plugin],
        timeout: Duration::from_secs(2),
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    *target.lock().unwrap() = Some(page.clone());
    assert_eq!(page.evaluate("reenter()").unwrap(), true);
    assert_eq!(page.evaluate("1 + 1").unwrap(), 2);
    page.close().unwrap();
}

#[test]
fn closing_session_cancels_pending_page_installation() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let plugin = Plugin::new(
        "pending-install",
        "async ({host}) => { await host.call('wait'); }",
    )
    .with_handler(
        "wait",
        Arc::new(move |call| {
            sender.send(call).unwrap();
        }),
    );
    let session = Arc::new(
        Session::new(SessionOptions {
            plugins: vec![plugin],
            ..Default::default()
        })
        .unwrap(),
    );
    let worker = session.clone();
    let opening = std::thread::spawn(move || worker.new_page());
    // Wait for installation to reach the host call; this is a cancellation
    // contract, not a one-second page-startup performance assertion.
    let call = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    session.close();
    assert!(matches!(
        opening.join().unwrap(),
        Err(brimp_runtime::runtime::AutomationError::Closed)
    ));
    assert!(call.context.is_cancelled());
    call.completion.resolve(BridgeValue::Null);
}
