use std::io::Cursor;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode, header::CONTENT_TYPE};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, PageOptions, ScreenshotOptions};

struct ImageLoader {
    red: Vec<u8>,
    blue: Vec<u8>,
    requests: Mutex<Vec<String>>,
}

#[async_trait]
impl ResourceLoader for ImageLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let (content_type, body) = match request.url.as_str() {
            "https://example.test/" => (
                "text/html",
                b"<!doctype html><style>html,body{margin:0}img{display:block;width:2px;height:2px}#background{width:2px;height:2px;background-image:url('/blue.png')}</style><img src='/red.png'><div id='background'></div>".to_vec(),
            ),
            "https://example.test/red.png" => ("image/png", self.red.clone()),
            "https://example.test/blue.png" => ("image/png", self.blue.clone()),
            other => panic!("unexpected resource request: {other}"),
        };
        self.requests.lock().unwrap().push(request.url.clone());
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body,
            effective_url: request.url,
        })
    }
}

fn solid_png(rgba: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 2, 2);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&rgba.repeat(4)).unwrap();
    drop(writer);
    bytes
}

fn decode(bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
    let mut reader = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    rgba.truncate(info.buffer_size());
    (info, rgba)
}

#[test]
fn navigation_loads_and_paints_element_and_css_images_through_resource_loader() {
    let loader = Arc::new(ImageLoader {
        red: solid_png([255, 0, 0, 255]),
        blue: solid_png([0, 0, 255, 255]),
        requests: Mutex::new(Vec::new()),
    });
    let browser = Browser::with_resource_loader(loader.clone());
    let mut page = browser
        .new_page(PageOptions::builder().viewport(4, 4).build())
        .unwrap();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(page.goto("https://example.test/"))
        .unwrap();
    assert!(
        page.run_until_idle_for(std::time::Duration::from_secs(1))
            .unwrap()
    );

    let screenshot = page.screenshot_png(ScreenshotOptions::new(4, 4)).unwrap();
    let (info, rgba) = decode(&screenshot);
    let pixel = |x: usize, y: usize| {
        let start = (y * info.width as usize + x) * 4;
        &rgba[start..start + 4]
    };
    assert_eq!(pixel(0, 0), &[255, 0, 0, 255]);
    assert_eq!(pixel(0, 2), &[0, 0, 255, 255]);

    let requests = loader.requests.lock().unwrap();
    assert!(requests.iter().any(|url| url.ends_with("/red.png")));
    assert!(requests.iter().any(|url| url.ends_with("/blue.png")));
}
