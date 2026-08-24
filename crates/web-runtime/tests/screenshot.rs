use std::io::Cursor;

use web_runtime::{Browser, PageOptions, ScreenshotOptions};

fn decode(png_bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
    let decoder = png::Decoder::new(Cursor::new(png_bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    rgba.truncate(info.buffer_size());
    (info, rgba)
}

#[test]
fn cpu_screenshot_reflects_post_javascript_style() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(64, 64).build())
        .unwrap();
    page.set_content(
        "<html><head><style>html,body{margin:0}#box{width:32px;height:32px;background:red}</style></head><body><div id='box'></div></body></html>",
    )
    .unwrap();
    page.eval("document.getElementById('box').style.backgroundColor = 'blue'")
        .unwrap();

    let bytes = page.screenshot_png(ScreenshotOptions::new(64, 64)).unwrap();
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let (info, rgba) = decode(&bytes);
    assert_eq!((info.width, info.height), (64, 64));
    assert_eq!(info.color_type, png::ColorType::Rgba);

    let pixel = (10 * info.width as usize + 10) * 4;
    assert_eq!(&rgba[pixel..pixel + 4], &[0, 0, 255, 255]);
}

#[test]
fn full_page_screenshot_extends_beyond_the_viewport() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(64, 64).build())
        .unwrap();
    page.set_content(
        "<html><head><style>html,body{margin:0}#box{height:200px}</style></head><body><div id='box'></div></body></html>",
    )
    .unwrap();
    let mut options = ScreenshotOptions::new(64, 64);
    options.full_page = true;

    let bytes = page.screenshot_png(options).unwrap();
    let (info, _) = decode(&bytes);

    assert_eq!(info.width, 64);
    assert!(info.height >= 200, "full-page height was {}", info.height);
    assert_eq!(
        page.eval("window.innerHeight")
            .unwrap()
            .to_number()
            .unwrap(),
        64.0
    );
}
