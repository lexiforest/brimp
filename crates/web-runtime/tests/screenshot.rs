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
fn canvas_raster_is_composited_into_document_screenshot() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(16, 16).canvas(true).build())
        .unwrap();
    page.set_content(
        "<style>html,body{margin:0;background:white}canvas{display:block;width:8px;height:8px}</style><canvas width='2' height='2'></canvas>",
    )
    .unwrap();
    page.eval(
        "const context = document.querySelector('canvas').getContext('2d'); context.fillStyle = 'rgba(255,0,0,.5)'; context.fillRect(0,0,2,2)",
    )
    .unwrap();

    let bytes = page.screenshot_png(ScreenshotOptions::new(16, 16)).unwrap();
    let (info, rgba) = decode(&bytes);
    let inside = (4 * info.width as usize + 4) * 4;
    let outside = (12 * info.width as usize + 12) * 4;
    assert_eq!(&rgba[inside..inside + 4], &[255, 127, 127, 255]);
    assert_eq!(&rgba[outside..outside + 4], &[255, 255, 255, 255]);
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

#[test]
fn bundled_cjk_and_emoji_fonts_paint_visible_glyphs() {
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().viewport(160, 64).build())
        .unwrap();
    page.set_content(
        "<html><head><style>html,body{margin:0;background:white;color:black;font-size:32px;line-height:64px}span{display:inline-block;width:80px;height:64px}</style></head><body><span>中文</span><span>😀</span></body></html>",
    )
    .unwrap();

    let bytes = page
        .screenshot_png(ScreenshotOptions::new(160, 64))
        .unwrap();
    let (info, rgba) = decode(&bytes);
    let non_white_pixels = |start_x: usize, end_x: usize| {
        (0..info.height as usize)
            .flat_map(|y| (start_x..end_x).map(move |x| (y * info.width as usize + x) * 4))
            .filter(|offset| rgba[*offset..*offset + 3] != [255, 255, 255])
            .count()
    };

    assert!(non_white_pixels(0, 80) > 50, "CJK glyphs were not painted");
    assert!(
        non_white_pixels(80, 160) > 50,
        "emoji glyph was not painted"
    );
}
