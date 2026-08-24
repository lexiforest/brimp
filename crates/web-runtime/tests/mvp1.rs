use web_runtime::{Browser, PageOptions, ScreenshotOptions};

#[test]
fn mvp1_runs_html_javascript_layout_and_cpu_paint_end_to_end() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(480, 240).build())
        .unwrap();
    page.set_content(
        r##"
        <html>
        <head>
        <style>
        html, body { margin: 0; }
        #box {
            width: 200px;
            height: 40px;
            padding: 20px;
            background: #eee;
        }
        </style>
        </head>
        <body><div id="box">Hello</div></body>
        </html>
        "##,
    )
    .unwrap();

    page.eval(
        r##"
        (() => {
            const box = document.querySelector("#box");
            if (box !== document.querySelector("#box")) throw new Error("unstable wrapper");
            box.setAttribute("data-test", "yes");
            box.style.width = "300px";
            if (box.getBoundingClientRect().width !== 340) throw new Error("wrong layout");
            console.log(box.getBoundingClientRect().width);
        })()
        "##,
    )
    .unwrap();

    let document = page.document();
    let box_id = document.get_element_by_id("box").unwrap();
    let box_node = document.node(box_id).unwrap();
    assert_eq!(
        box_node.attr(blitz_dom::LocalName::from("data-test")),
        Some("yes")
    );
    assert_eq!(document.bounding_rect(box_id).unwrap()[2], 340.0);
    drop(document);

    let png = page
        .screenshot_png(ScreenshotOptions::new(480, 240))
        .unwrap();
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}
