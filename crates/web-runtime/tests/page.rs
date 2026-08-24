use web_runtime::{Browser, PageOptions};

#[test]
fn page_owns_javascript_document_and_viewport() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(1280, 720).build())
        .unwrap();

    page.set_content(r#"<html><body><div id="hello">Hello</div></body></html>"#)
        .unwrap();

    assert_eq!(page.eval("1 + 2").unwrap().to_number().unwrap(), 3.0);
    assert!(page.document().get_element_by_id("hello").is_some());
    assert_eq!(page.viewport().width, 1280.0);
    assert_eq!(page.viewport().height, 720.0);
}

#[test]
fn page_tasks_run_on_the_page_and_can_mutate_it() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    page.tasks().push(|page| {
        page.set_content(r#"<html><body id="ready"></body></html>"#)
            .unwrap();
    });

    page.run_pending_tasks().unwrap();

    assert!(page.tasks().is_empty());
    assert!(page.document().get_element_by_id("ready").is_some());
}

#[test]
fn worker_tasks_are_handed_back_to_the_page_owner_thread() {
    let browser = Browser::new().unwrap();
    let mut page = browser.new_page(PageOptions::default()).unwrap();
    let sender = page.task_sender();

    std::thread::spawn(move || {
        sender
            .post(|page| {
                page.set_content(r#"<html><body id="from-worker"></body></html>"#)
                    .unwrap();
            })
            .unwrap();
    })
    .join()
    .unwrap();

    page.run_pending_tasks().unwrap();

    assert!(page.document().get_element_by_id("from-worker").is_some());
}
