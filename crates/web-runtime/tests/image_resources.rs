use std::io::Cursor;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use http::{
    HeaderMap, HeaderValue, StatusCode,
    header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, SET_COOKIE,
    },
};
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
            "https://example.test/cross" => (
                "text/html",
                b"<!doctype html><img src='https://other.test/red.png'>".to_vec(),
            ),
            "https://example.test/cors" => (
                "text/html",
                br#"<!doctype html>
                    <img id='anonymous' crossorigin src='https://other.test/wildcard.png'>
                    <img id='credentials' crossorigin='use-credentials' src='https://other.test/credentials.png'>
                    <img id='credentials-wildcard' crossorigin='use-credentials' src='https://other.test/wildcard.png'>
                    <img id='no-mode' src='https://other.test/wildcard.png'>
                    <img id='mismatch' crossorigin='anonymous' src='https://other.test/mismatch.png'>"#
                    .to_vec(),
            ),
            "https://example.test/red.png" => ("image/png", self.red.clone()),
            "https://other.test/red.png" => ("image/png", self.red.clone()),
            "https://other.test/wildcard.png" => ("image/png", self.red.clone()),
            "https://other.test/credentials.png" => ("image/png", self.red.clone()),
            "https://other.test/mismatch.png" => ("image/png", self.red.clone()),
            "https://example.test/blue.png" => ("image/png", self.blue.clone()),
            other => panic!("unexpected resource request: {other}"),
        };
        self.requests.lock().unwrap().push(request.url.clone());
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        match request.url.as_str() {
            "https://example.test/cors" => {
                headers.insert(SET_COOKIE, HeaderValue::from_static("page=credentialed"));
            }
            "https://other.test/wildcard.png" => {
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            }
            "https://other.test/credentials.png" => {
                headers.insert(
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_static("https://example.test"),
                );
                headers.insert(
                    ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    HeaderValue::from_static("true"),
                );
            }
            "https://other.test/mismatch.png" => {
                headers.insert(
                    ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_static("https://wrong.test"),
                );
            }
            _ => {}
        }
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body,
            effective_url: request.url,
            metadata: network::ResponseMetadata::default(),
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

#[test]
fn canvas_draws_and_patterns_loaded_image_elements_from_the_shared_decoder() {
    let loader = Arc::new(ImageLoader {
        red: solid_png([255, 0, 0, 255]),
        blue: solid_png([0, 0, 255, 255]),
        requests: Mutex::new(Vec::new()),
    });
    let browser = Browser::with_resource_loader(loader);
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
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

    page.eval(
        r#"
        const loadedImage = document.querySelector("img");
        const imageCanvas = document.createElement("canvas");
        imageCanvas.width = 4;
        imageCanvas.height = 2;
        const imageContext = imageCanvas.getContext("2d");
        imageContext.drawImage(loadedImage, 0, 0);
        const imagePattern = imageContext.createPattern(loadedImage, "repeat");
        imageContext.fillStyle = imagePattern;
        imageContext.fillRect(2, 0, 2, 2);
        const constructedImage = new Image(3, 4);
        globalThis.loadedImageResult = JSON.stringify({
            complete: loadedImage.complete,
            natural: [loadedImage.naturalWidth, loadedImage.naturalHeight],
            currentSrc: loadedImage.currentSrc,
            pixels: [...imageContext.getImageData(0, 0, 4, 2).data],
            pattern: imagePattern instanceof CanvasPattern,
            decodePromise: loadedImage.decode() instanceof Promise,
            constructed: constructedImage instanceof HTMLImageElement,
            dimensions: [constructedImage.width, constructedImage.height],
            native: imageContext.drawImage.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("loadedImageResult").unwrap().to_string().unwrap(),
        r#"{"complete":true,"natural":[2,2],"currentSrc":"https://example.test/red.png","pixels":[255,0,0,255,255,0,0,255,255,0,0,255,255,0,0,255,255,0,0,255,255,0,0,255,255,0,0,255,255,0,0,255],"pattern":true,"decodePromise":true,"constructed":true,"dimensions":[3,4],"native":"function drawImage() { [native code] }"}"#,
    );
}

#[test]
fn cross_origin_images_taint_canvas_readback_and_patterns_until_reset() {
    let loader = Arc::new(ImageLoader {
        red: solid_png([255, 0, 0, 255]),
        blue: solid_png([0, 0, 255, 255]),
        requests: Mutex::new(Vec::new()),
    });
    let browser = Browser::with_resource_loader(loader);
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).webgpu(true).build())
        .unwrap();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(page.goto("https://example.test/cross"))
        .unwrap();
    assert!(
        page.run_until_idle_for(std::time::Duration::from_secs(1))
            .unwrap()
    );

    page.eval(
        r#"
        globalThis.taintResult = "pending";
        const crossImage = document.querySelector("img");
        const taintedCanvas = document.createElement("canvas");
        taintedCanvas.width = 2;
        taintedCanvas.height = 2;
        const taintedContext = taintedCanvas.getContext("2d");
        taintedContext.drawImage(crossImage, 0, 0);
        const securityName = operation => {
            try { operation(); return "none"; }
            catch (error) { return error.name; }
        };
        const directRead = securityName(() => taintedContext.getImageData(0, 0, 1, 1));
        const directExport = securityName(() => taintedCanvas.toDataURL());
        const copiedCanvas = document.createElement("canvas");
        copiedCanvas.width = 2;
        copiedCanvas.height = 2;
        const copiedContext = copiedCanvas.getContext("2d");
        copiedContext.drawImage(taintedCanvas, 0, 0);
        const copiedRead = securityName(() => copiedContext.getImageData(0, 0, 1, 1));

        const patternedCanvas = document.createElement("canvas");
        patternedCanvas.width = 2;
        patternedCanvas.height = 2;
        const patternedContext = patternedCanvas.getContext("2d");
        patternedContext.fillStyle = patternedContext.createPattern(crossImage, "repeat");
        patternedContext.fillRect(0, 0, 2, 2);
        const patternRead = securityName(() => patternedContext.getImageData(0, 0, 1, 1));

        createImageBitmap(taintedCanvas).then(async bitmap => {
            const bitmapCanvas = document.createElement("canvas");
            bitmapCanvas.width = 2;
            bitmapCanvas.height = 2;
            const bitmapContext = bitmapCanvas.getContext("2d");
            bitmapContext.drawImage(bitmap, 0, 0);
            const bitmapRead = securityName(() => bitmapContext.getImageData(0, 0, 1, 1));
            const adapter = await navigator.gpu.requestAdapter();
            let gpuCanvas = "no-adapter";
            let gpuBitmap = "no-adapter";
            let gpuImage = "no-adapter";
            if (adapter) {
                const device = await adapter.requestDevice();
                const texture = device.createTexture({
                    size: [2, 2],
                    format: "rgba8unorm",
                    usage: GPUTextureUsage.COPY_DST,
                });
                const copy = source => device.queue.copyExternalImageToTexture(
                    { source }, { texture }, [2, 2],
                );
                gpuCanvas = securityName(() => copy(taintedCanvas));
                gpuBitmap = securityName(() => copy(bitmap));
                gpuImage = securityName(() => copy(crossImage));
            }
            bitmap.close();
            taintedCanvas.width = 2;
            const resetRead = securityName(() => taintedContext.getImageData(0, 0, 1, 1));
            globalThis.taintResult = JSON.stringify({
                directRead, directExport, copiedRead, patternRead, bitmapRead,
                gpuCanvas, gpuBitmap, gpuImage, resetRead,
            });
        }).catch(error => globalThis.taintResult = `error:${error}`);
        "#,
    )
    .unwrap();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while page.eval("taintResult").unwrap().to_string().unwrap() == "pending"
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let result = page.eval("taintResult").unwrap().to_string().unwrap();
    assert!(
        !result.starts_with("error:"),
        "unexpected taint test error: {result}"
    );
    assert_ne!(
        result, "pending",
        "taint test did not settle before its deadline"
    );
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for field in [
        "directRead",
        "directExport",
        "copiedRead",
        "patternRead",
        "bitmapRead",
    ] {
        assert_eq!(result[field], "SecurityError");
    }
    assert_eq!(result["resetRead"], "none");
    let gpu_result = result["gpuCanvas"].as_str().unwrap();
    assert!(gpu_result == "no-adapter" || gpu_result == "SecurityError");
    assert_eq!(result["gpuBitmap"], gpu_result);
    assert_eq!(result["gpuImage"], gpu_result);
}

#[test]
fn cors_approved_cross_origin_images_remain_canvas_origin_clean() {
    let loader = Arc::new(ImageLoader {
        red: solid_png([255, 0, 0, 255]),
        blue: solid_png([0, 0, 255, 255]),
        requests: Mutex::new(Vec::new()),
    });
    let browser = Browser::with_resource_loader(loader);
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).webgl(true).build())
        .unwrap();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(page.goto("https://example.test/cors"))
        .unwrap();
    assert!(
        page.run_until_idle_for(std::time::Duration::from_secs(1))
            .unwrap()
    );

    let result = page
        .eval(
            r#"(() => {
                const read = id => {
                    const canvas = document.createElement("canvas");
                    canvas.width = 2;
                    canvas.height = 2;
                    const context = canvas.getContext("2d");
                    context.drawImage(document.getElementById(id), 0, 0);
                    try { return [...context.getImageData(0, 0, 1, 1).data]; }
                    catch (error) { return error.name; }
                };
                const image = document.getElementById("anonymous");
                const gl = document.createElement("canvas").getContext("webgl");
                let webgl = "unavailable";
                if (gl) {
                    const texture = gl.createTexture();
                    gl.bindTexture(gl.TEXTURE_2D, texture);
                    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
                    webgl = gl.getError() === gl.NO_ERROR;
                }
                return JSON.stringify({
                    anonymous: read("anonymous"),
                    credentials: read("credentials"),
                    credentialsWildcard: read("credentials-wildcard"),
                    noMode: read("no-mode"),
                    mismatch: read("mismatch"),
                    webgl,
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["anonymous"], serde_json::json!([255, 0, 0, 255]));
    assert_eq!(result["credentials"], serde_json::json!([255, 0, 0, 255]));
    assert_eq!(result["credentialsWildcard"], "SecurityError");
    assert_eq!(result["noMode"], "SecurityError");
    assert_eq!(result["mismatch"], "SecurityError");
    assert!(result["webgl"] == true || result["webgl"] == "unavailable");
}
