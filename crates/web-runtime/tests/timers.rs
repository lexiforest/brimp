use web_runtime::{Browser, PageOptions};

#[test]
fn zero_delay_timers_run_on_the_page_owner_thread() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.set_content("<html><body></body></html>").unwrap();

    page.eval(
        r#"
        globalThis.timerResult = "waiting";
        setTimeout(() => {
            document.body.setAttribute("data-timer", "done");
            globalThis.timerResult = "done";
        }, 0);
        "#,
    )
    .unwrap();
    assert_eq!(
        page.eval("timerResult").unwrap().to_string().unwrap(),
        "waiting"
    );

    page.run_pending_tasks().unwrap();

    assert_eq!(
        page.eval("timerResult").unwrap().to_string().unwrap(),
        "done"
    );
    let document = page.document();
    let body = document.query_selector("body").unwrap().unwrap();
    assert_eq!(
        document
            .node(body)
            .unwrap()
            .attr(blitz_dom::LocalName::from("data-timer")),
        Some("done")
    );
}

#[test]
fn clear_timeout_prevents_the_callback() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.eval(
        r#"
        globalThis.timerRan = false;
        const id = setTimeout(() => { globalThis.timerRan = true; }, 0);
        clearTimeout(id);
        "#,
    )
    .unwrap();

    page.run_pending_tasks().unwrap();

    assert_eq!(page.eval("timerRan").unwrap().to_string().unwrap(), "false");
}

#[test]
fn microtasks_run_at_the_end_of_the_javascript_checkpoint() {
    let browser = Browser::new().unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();

    let result = page
        .eval(
            r#"
            globalThis.order = ["script"];
            queueMicrotask(() => order.push("microtask"));
            order.push("script-end");
            "evaluation-result";
            "#,
        )
        .unwrap();

    assert_eq!(result.to_string().unwrap(), "evaluation-result");
    drop(result);
    assert_eq!(
        page.eval("order.join(',')").unwrap().to_string().unwrap(),
        "script,script-end,microtask"
    );
}
