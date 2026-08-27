use web_runtime::{Browser, PageOptions};

#[test]
fn exposes_and_updates_the_blitz_viewport() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(
            PageOptions::builder()
                .viewport(1280, 720)
                .device_pixel_ratio(2)
                .build(),
        )
        .unwrap();
    page.set_content(
        "<html><head><style>#box { width: 50vw; }</style></head><body><div id='box'></div></body></html>",
    )
    .unwrap();

    assert_eq!(
        page.eval("window.innerWidth + window.innerHeight + window.devicePixelRatio")
            .unwrap()
            .to_number()
            .unwrap(),
        2002.0
    );
    assert_eq!(
        page.eval("document.getElementById('box').offsetWidth")
            .unwrap()
            .to_number()
            .unwrap(),
        640.0
    );

    page.set_viewport(800, 600, 1.5);

    assert_eq!(
        page.eval("window.innerWidth + window.innerHeight + window.devicePixelRatio")
            .unwrap()
            .to_number()
            .unwrap(),
        1401.5
    );
    assert_eq!(
        page.eval("document.getElementById('box').offsetWidth")
            .unwrap()
            .to_number()
            .unwrap(),
        400.0
    );
}
