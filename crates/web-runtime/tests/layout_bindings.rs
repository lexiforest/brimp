use web_runtime::{Browser, PageOptions};

#[test]
fn style_mutation_restyles_and_relayouts_before_geometry_reads() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(1280, 720).build())
        .unwrap();
    page.set_content(
        r#"
        <html>
        <head>
            <style>
                #box { width: 200px; padding: 20px; border: 5px solid black; }
            </style>
        </head>
        <body><div id="box">Hello</div></body>
        </html>
        "#,
    )
    .unwrap();

    let before = page
        .eval("document.getElementById('box').getBoundingClientRect().width")
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(before, 250.0);

    page.eval("document.getElementById('box').style.width = '300px'")
        .unwrap();

    let result = page
        .eval(
            r#"
            (() => {
                const box = document.getElementById("box");
                const rect = box.getBoundingClientRect();
                if (!(rect instanceof DOMRect)) throw new Error("expected DOMRect");
                if (getComputedStyle(box).width !== "300px") throw new Error("bad computed width");
                if (getComputedStyle(box).padding !== "20px") throw new Error("bad padding");
                if (box.clientWidth !== 340) throw new Error("bad clientWidth: " + box.clientWidth);
                if (box.offsetWidth !== 350) throw new Error("bad offsetWidth: " + box.offsetWidth);
                return rect.width;
            })()
            "#,
        )
        .unwrap()
        .to_number()
        .unwrap();
    assert_eq!(result, 350.0);

    let document = page.document();
    let box_id = document.get_element_by_id("box").unwrap();
    assert_eq!(document.bounding_rect(box_id).unwrap()[2], 350.0);
}

#[test]
fn geometry_uses_the_configured_viewport() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(640, 480).build())
        .unwrap();
    page.set_content(
        "<html><head><style>#box { width: 50%; }</style></head><body><div id='box'></div></body></html>",
    )
    .unwrap();

    let width = page
        .eval("document.getElementById('box').getBoundingClientRect().width")
        .unwrap()
        .to_number()
        .unwrap();

    // The body has the default 8px margins, leaving 624px for its content box.
    assert_eq!(width, 312.0);
}
