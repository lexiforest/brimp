use web_runtime::{Browser, PageOptions};

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
