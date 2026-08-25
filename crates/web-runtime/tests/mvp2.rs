use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, LoadState, PageOptions, ScreenshotOptions};

struct MvpLoader;

#[async_trait]
impl ResourceLoader for MvpLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        if request.url != "https://example.test/" {
            assert_eq!(request.headers.get(COOKIE).unwrap(), "session=mvp2");
        }
        let (content_type, body) = match request.url.as_str() {
            "https://example.test/" => (
                "text/html",
                br#"<!doctype html><html><head><title>MVP 2</title><link rel="stylesheet" href="/style.css"></head><body><div id="box"></div><script src="/app.js"></script></body></html>"#.to_vec(),
            ),
            "https://example.test/style.css" => (
                "text/css",
                b"html,body{margin:0}#box{width:4px;height:4px;background:red}".to_vec(),
            ),
            "https://example.test/app.js" => (
                "text/javascript",
                br##"
                const box = document.querySelector("#box");
                box.addEventListener("ready", () => box.setAttribute("data-event", "yes"));
                box.dispatchEvent(new Event("ready"));
                setTimeout(() => box.setAttribute("data-timer", "yes"), 0);
                fetch("/data.json").then(response => response.json()).then(data => {
                    box.setAttribute("data-fetch", data.value);
                    box.style.backgroundColor = "blue";
                });
                "##.to_vec(),
            ),
            "https://example.test/data.json" => {
                ("application/json", br#"{"value":"yes"}"#.to_vec())
            }
            other => panic!("unexpected MVP 2 request: {other}"),
        };
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        if request.url == "https://example.test/" {
            headers.insert(SET_COOKIE, HeaderValue::from_static("session=mvp2; Path=/"));
        }
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body,
            effective_url: request.url,
        })
    }
}

#[test]
fn mvp2_navigates_runs_web_apis_and_screenshots_the_result() {
    let browser = Browser::with_resource_loader(Arc::new(MvpLoader));
    let mut page = browser
        .new_page(PageOptions::builder().viewport(8, 8).build())
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    runtime
        .block_on(page.goto("https://example.test/"))
        .unwrap();
    runtime.block_on(page.wait_for_load()).unwrap();
    assert!(page.run_until_idle_for(Duration::from_secs(1)).unwrap());

    assert_eq!(page.load_state(), LoadState::Complete);
    assert_eq!(
        page.eval(
            r##"[
                document.title,
                location.origin,
                navigator.language,
                document.cookie,
                document.querySelector("#box").getAttribute("data-event"),
                document.querySelector("#box").getAttribute("data-timer"),
                document.querySelector("#box").getAttribute("data-fetch")
            ].join("|")"##,
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "MVP 2|https://example.test|en-US|session=mvp2|yes|yes|yes"
    );

    let png = page.screenshot_png(ScreenshotOptions::new(8, 8)).unwrap();
    let mut reader = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    rgba.truncate(info.buffer_size());
    assert_eq!(&rgba[..4], &[0, 0, 255, 255]);
}
