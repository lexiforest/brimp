use web_runtime::{Browser, PageOptions};

#[test]
fn global_event_methods_have_the_window_receiver() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body></body></html>").unwrap();

    assert_eq!(
        page.eval(
            r#"
            let received = false;
            addEventListener("probe", () => { received = true; });
            dispatchEvent(new Event("probe"));
            Number(received);
            "#,
        )
        .unwrap()
        .to_number()
        .unwrap(),
        1.0
    );
}

#[test]
fn events_capture_target_and_bubble_over_stable_wrappers() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content(
        "<html><body><div id='parent'><button id='target'>Go</button></div></body></html>",
    )
    .unwrap();

    let order = page
        .eval(
            r##"
            const order = [];
            const parent = document.querySelector("#parent");
            const target = document.querySelector("#target");
            window.addEventListener("click", event => order.push(`window:${event.eventPhase}`), true);
            parent.addEventListener("click", event => order.push(`parent-capture:${event.eventPhase}`), true);
            target.addEventListener("click", event => order.push(`target-capture:${event.eventPhase}`), true);
            target.addEventListener("click", event => order.push(`target:${event.eventPhase}`));
            parent.addEventListener("click", event => order.push(`parent:${event.eventPhase}`));
            target.click();
            order.join(",");
            "##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        order,
        "window:1,parent-capture:1,target-capture:2,target:2,parent:3"
    );
}

#[test]
fn listeners_can_be_removed_and_cancel_dispatch() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body><button id='target'></button></body></html>")
        .unwrap();

    assert_eq!(
        page.eval(
            r##"
            const target = document.querySelector("#target");
            let removedRan = false;
            const removed = () => { removedRan = true; };
            target.addEventListener("submit", removed);
            target.removeEventListener("submit", removed);
            target.addEventListener("submit", event => event.preventDefault(), { once: true });
            const first = target.dispatchEvent(new Event("submit", { cancelable: true }));
            const second = target.dispatchEvent(new Event("submit", { cancelable: true }));
            `${removedRan},${first},${second}`;
            "##,
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "false,false,true"
    );
}

#[test]
fn creates_legacy_events_and_abort_signals() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body></body></html>").unwrap();

    let result = page
        .eval(
            r#"
            const event = document.createEvent("Events");
            event.initEvent("ready", true, true);
            const controller = new AbortController();
            const dependent = AbortSignal.any([controller.signal]);
            let calls = 0;
            dependent.addEventListener("abort", e => {
                if (e.isTrusted && e.target === dependent) calls++;
            });
            controller.abort("done");
            [
                event.type,
                event.bubbles,
                event.cancelable,
                dependent.aborted,
                dependent.reason,
                calls,
                AbortSignal.abort().reason instanceof DOMException,
                DOMException.ABORT_ERR,
            ].join(",");
            "#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "ready,true,true,true,done,1,true,20");
}
