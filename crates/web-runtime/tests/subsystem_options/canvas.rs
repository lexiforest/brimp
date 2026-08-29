use std::sync::Arc;

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn canvas_2d_option_enables_a_real_skia_backing_store() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas id='canvas' width='4' height='3'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r##"(() => {
                const canvas = document.getElementById("canvas");
                const context = canvas.getContext("2d");
                const stable = context === canvas.getContext("2d");
                context.fillStyle = "#f60";
                context.globalAlpha = 0.5;
                context.fillRect(1, 1, 2, 1);
                const drawn = [...context.getImageData(1, 1, 1, 1).data];
                const image = new ImageData(new Uint8ClampedArray([0, 255, 0, 255]), 1, 1);
                context.putImageData(image, 0, 0);
                const written = [...context.getImageData(0, 0, 1, 1).data];
                const png = canvas.toDataURL();
                canvas.width = 4;
                const reset = [...context.getImageData(0, 0, 1, 1).data];
                return JSON.stringify({
                    stable,
                    drawn,
                    written,
                    reset,
                    png: png.startsWith("data:image/png;base64,iVBORw0KGgo"),
                    otherContext: canvas.getContext("webgl"),
                    native: Function.prototype.toString.call(CanvasRenderingContext2D.prototype.fillRect),
                });
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r##"{"stable":true,"drawn":[255,102,0,128],"written":[0,255,0,255],"reset":[0,0,0,0],"png":true,"otherContext":null,"native":"function fillRect() { [native code] }"}"##,
    );
}

#[test]
fn canvas_2d_context_settings_and_reset_control_the_skia_bitmap() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"
            (() => {
                const canvas = document.createElement("canvas");
                canvas.width = 2;
                canvas.height = 1;
                const context = canvas.getContext("2d", {
                    alpha: false,
                    desynchronized: true,
                    colorSpace: "srgb",
                    colorType: "unorm8",
                    willReadFrequently: true,
                });
                const pixel = x => [...context.getImageData(x, 0, 1, 1).data];
                const initial = pixel(0);
                context.fillStyle = "rgba(255, 255, 255, 0.5)";
                context.fillRect(0, 0, 1, 1);
                const blended = pixel(0);
                context.save();
                context.translate(10, 20);
                context.lineWidth = 7;
                context.fillStyle = "red";
                context.beginPath();
                context.rect(0, 0, 2, 1);
                context.reset();
                context.restore();
                context.clearRect(1, 0, 1, 1);
                const attributes = context.getContextAttributes();
                return JSON.stringify({
                    same: canvas.getContext("2d", { alpha: true }) === context,
                    initial,
                    blended,
                    reset: pixel(0),
                    cleared: pixel(1),
                    defaults: [context.fillStyle, context.lineWidth, ...Object.values(context.getTransform()).slice(0, 6)],
                    attributes,
                    freshAttributes: attributes !== context.getContextAttributes(),
                    lost: context.isContextLost(),
                    native: [context.reset.toString(), context.isContextLost.toString(), context.getContextAttributes.toString()],
                });
            })()
            "#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        result,
        r##"{"same":true,"initial":[0,0,0,255],"blended":[128,128,128,255],"reset":[0,0,0,255],"cleared":[0,0,0,255],"defaults":["#000000",1,1,0,0,1,0,0],"attributes":{"alpha":false,"desynchronized":true,"colorSpace":"srgb","colorType":"unorm8","willReadFrequently":true},"freshAttributes":true,"lost":false,"native":["function reset() { [native code] }","function isContextLost() { [native code] }","function getContextAttributes() { [native code] }"]}"##,
    );
}

#[test]
fn canvas_2d_wide_gamut_float16_backing_and_image_data_are_real() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"
            (() => {
                const canvas = document.createElement("canvas");
                canvas.width = 2;
                canvas.height = 1;
                const context = canvas.getContext("2d", {
                    colorSpace: "display-p3",
                    colorType: "float16",
                });
                const source = new ImageData(
                    new Float16Array([1.25, 0.25, 0.5, 0.5, 0.1, 0.2, 0.3, 1]),
                    2,
                    1,
                    { colorSpace: "display-p3", pixelFormat: "rgba-float16" },
                );
                context.putImageData(source, 0, 0);
                const wide = context.getImageData(0, 0, 2, 1, {
                    colorSpace: "display-p3",
                    pixelFormat: "rgba-float16",
                });
                const srgb = context.getImageData(0, 0, 1, 1, { colorSpace: "srgb" });
                context.putImageData(new ImageData(
                    new Uint8ClampedArray([255, 128, 0, 255]),
                    1,
                    1,
                    { colorSpace: "display-p3" },
                ), 0, 0);
                const converted = [...context.getImageData(0, 0, 1, 1, { colorSpace: "srgb" }).data];
                const allocated = context.createImageData(1, 1, { pixelFormat: "rgba-float16" });
                const copied = context.createImageData(source);
                let mismatch;
                try {
                    new ImageData(new Uint8ClampedArray(4), 1, 1, { pixelFormat: "rgba-float16" });
                } catch (error) {
                    mismatch = error.name;
                }
                canvas.width = 2;
                context.putImageData(source, 0, 0, 1, 0, 1, 1);
                const afterResize = context.getImageData(1, 0, 1, 1, {
                    colorSpace: "display-p3",
                    pixelFormat: "rgba-float16",
                });
                return JSON.stringify({
                    float16Global: typeof Float16Array,
                    attributes: context.getContextAttributes(),
                    source: [source.data.constructor.name, source.colorSpace, source.pixelFormat],
                    wide: [...wide.data],
                    wideType: wide.data.constructor.name,
                    srgb: [...srgb.data],
                    converted,
                    allocated: [allocated.data.constructor.name, allocated.colorSpace, allocated.pixelFormat],
                    copied: [copied.data.constructor.name, copied.colorSpace, copied.pixelFormat],
                    mismatch,
                    afterResize: [...afterResize.data],
                    png: canvas.toDataURL().startsWith("data:image/png;base64,iVBORw0KGgo"),
                });
            })()
            "#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(result["float16Global"], "function");
    assert_eq!(result["attributes"]["colorSpace"], "display-p3");
    assert_eq!(result["attributes"]["colorType"], "float16");
    assert_eq!(
        result["source"],
        serde_json::json!(["Float16Array", "display-p3", "rgba-float16"])
    );
    assert_eq!(result["wideType"], "Float16Array");
    assert!((result["wide"][0].as_f64().unwrap() - 1.25).abs() < 0.002);
    assert!((result["wide"][1].as_f64().unwrap() - 0.25).abs() < 0.002);
    assert!((result["wide"][3].as_f64().unwrap() - 0.5).abs() < 0.002);
    assert_eq!(result["srgb"][3], 128);
    assert_eq!(result["converted"][0], 255);
    assert!(result["converted"][1].as_u64().unwrap() > 110);
    assert!(result["converted"][1].as_u64().unwrap() < 126);
    assert_eq!(result["converted"][2], 0);
    assert_eq!(result["converted"][3], 255);
    assert_eq!(
        result["allocated"],
        serde_json::json!(["Float16Array", "display-p3", "rgba-float16"])
    );
    assert_eq!(
        result["copied"],
        serde_json::json!(["Uint8ClampedArray", "display-p3", "rgba-unorm8"])
    );
    assert_eq!(result["mismatch"], "InvalidStateError");
    assert!((result["afterResize"][0].as_f64().unwrap() - 0.1).abs() < 0.002);
    assert!((result["afterResize"][3].as_f64().unwrap() - 1.0).abs() < 0.002);
    assert_eq!(result["png"], true);
}

#[test]
fn canvas_2d_paths_use_skia_geometry_and_fill_rules() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas id='canvas' width='12' height='12'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r##"(() => {
                const context = document.getElementById("canvas").getContext("2d");
                context.fillStyle = "#ff0000";
                context.beginPath();
                context.rect(1, 1, 10, 10);
                context.rect(3, 3, 6, 6);
                const outerHit = context.isPointInPath(2, 2, "evenodd");
                const holeHit = context.isPointInPath(6, 6, "evenodd");
                context.fill("evenodd");
                const outer = [...context.getImageData(2, 2, 1, 1).data];
                const hole = [...context.getImageData(6, 6, 1, 1).data];

                context.beginPath();
                context.translate(4, 0);
                context.moveTo(0, 0);
                context.lineTo(3, 0);
                context.lineTo(0, 3);
                context.closePath();
                context.resetTransform();
                context.fillStyle = "#00ff00";
                context.fill();
                const transformed = [...context.getImageData(4, 0, 1, 1).data];
                return JSON.stringify({ outerHit, holeHit, outer, hole, transformed });
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"outerHit":true,"holeHit":false,"outer":[255,0,0,255],"hole":[0,0,0,0],"transformed":[0,255,0,255]}"#,
    );
}

#[test]
fn canvas_2d_ellipse_round_rect_and_stroke_hits_use_skia_geometry() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='30' height='24'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r##"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.beginPath();
                context.ellipse(7, 8, 5, 2, Math.PI / 2, 0, Math.PI * 2);
                const ellipseHit = context.isPointInPath(7, 4);
                const ellipseMiss = context.isPointInPath(3, 8);
                context.fillStyle = "#00ff00";
                context.fill();

                context.beginPath();
                context.roundRect(15, 2, 14, 14, 20);
                context.fillStyle = "#0000ff";
                context.fill();

                context.beginPath();
                context.moveTo(2, 20);
                context.arcTo(12, 20, 12, 10, 4);
                context.lineWidth = 4;
                const strokeHit = context.isPointInStroke(7, 20);
                const tangentHit = context.isPointInStroke(11, 19);
                const strokeMiss = context.isPointInStroke(7, 16);

                let roundRectError = "";
                let ellipseError = "";
                let arcToError = "";
                try { context.roundRect(0, 0, 1, 1, -1); } catch (error) { roundRectError = error.name; }
                try { context.ellipse(0, 0, -1, 1, 0, 0, 1); } catch (error) { ellipseError = error.name; }
                try { context.arcTo(0, 0, 1, 1, -1); } catch (error) { arcToError = error.name; }
                return JSON.stringify({
                    ellipseHit,
                    ellipseMiss,
                    strokeHit,
                    tangentHit,
                    strokeMiss,
                    ellipsePixel: [...context.getImageData(7, 4, 1, 1).data],
                    ellipseOutside: [...context.getImageData(3, 8, 1, 1).data],
                    roundedCenter: [...context.getImageData(22, 3, 1, 1).data],
                    roundedCorner: [...context.getImageData(15, 2, 1, 1).data],
                    roundRectError,
                    ellipseError,
                    arcToError,
                    native: [context.ellipse.toString(), context.roundRect.toString(), context.arcTo.toString(), context.isPointInStroke.toString()],
                });
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"ellipseHit":true,"ellipseMiss":false,"strokeHit":true,"tangentHit":true,"strokeMiss":false,"ellipsePixel":[0,255,0,255],"ellipseOutside":[0,0,0,0],"roundedCenter":[0,0,255,255],"roundedCorner":[0,0,0,0],"roundRectError":"RangeError","ellipseError":"IndexSizeError","arcToError":"IndexSizeError","native":["function ellipse() { [native code] }","function roundRect() { [native code] }","function arcTo() { [native code] }","function isPointInStroke() { [native code] }"]}"#,
    );
}

#[test]
fn canvas_path2d_uses_independent_native_skia_paths_and_context_overloads() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='24' height='14'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r##"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                const svg = new Path2D("M1 1h4v4h-4z");
                const copy = new Path2D(svg);
                svg.rect(8, 1, 4, 4);
                const transformed = new Path2D();
                transformed.addPath(copy, { a: 1, b: 0, c: 0, d: 1, e: 6, f: 0 });

                context.fillStyle = "#ff0000";
                context.fill(svg);
                context.fillStyle = "#00ff00";
                context.fill(transformed);

                context.beginPath();
                context.rect(1, 8, 4, 4);
                context.fill(transformed);
                context.fillStyle = "#0000ff";
                context.fill();

                const outlined = new Path2D();
                outlined.moveTo(14, 9);
                outlined.arcTo(20, 9, 20, 3, 3);
                context.lineWidth = 2;
                const strokeHit = context.isPointInStroke(outlined, 18, 9);
                context.stroke(outlined);

                const ring = new Path2D();
                ring.roundRect(14, 1, 8, 6, 2);
                ring.ellipse(18, 4, 2, 1, 0, 0, Math.PI * 2);
                const ringOuter = context.isPointInPath(ring, 15, 3, "evenodd");
                const ringHole = context.isPointInPath(ring, 18, 4, "evenodd");

                context.save();
                context.clip(copy);
                context.fillStyle = "#ffff00";
                context.fillRect(0, 0, 24, 14);
                context.restore();

                const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];
                return JSON.stringify({
                    copyIndependent: context.isPointInPath(copy, 2, 2) && !context.isPointInPath(copy, 9, 2),
                    transformedHit: context.isPointInPath(transformed, 8, 2),
                    strokeHit,
                    ringOuter,
                    ringHole,
                    clipped: pixel(2, 2),
                    originalExtra: pixel(11, 2),
                    transformedPixel: pixel(8, 3),
                    currentPathPixel: pixel(2, 9),
                    pathObject: svg instanceof Path2D,
                    native: [Path2D.toString(), Path2D.prototype.addPath.toString(), context.fill.toString()],
                });
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"copyIndependent":true,"transformedHit":true,"strokeHit":true,"ringOuter":true,"ringHole":false,"clipped":[255,255,0,255],"originalExtra":[255,0,0,255],"transformedPixel":[0,255,0,255],"currentPathPixel":[0,0,255,255],"pathObject":true,"native":["function Path2D() { [native code] }","function addPath() { [native code] }","function fill() { [native code] }"]}"#,
    );
}

#[test]
fn canvas_2d_clip_state_and_compositing_use_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='8' height='4'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r##"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.fillStyle = "#ff0000";
                context.fillRect(0, 0, 8, 4);
                context.save();
                context.beginPath();
                context.rect(0, 0, 4, 4);
                context.clip();
                context.fillStyle = "#00ff00";
                context.fillRect(0, 0, 8, 4);
                context.restore();
                const clippedInside = [...context.getImageData(1, 1, 1, 1).data];
                const clippedOutside = [...context.getImageData(6, 1, 1, 1).data];
                context.globalCompositeOperation = "destination-out";
                context.fillRect(4, 0, 4, 4);
                const composited = [...context.getImageData(6, 1, 1, 1).data];
                context.globalCompositeOperation = "invalid";
                return JSON.stringify({
                    clippedInside,
                    clippedOutside,
                    composited,
                    operation: context.globalCompositeOperation,
                    native: context.clip.toString(),
                });
            })()"##,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"clippedInside":[0,255,0,255],"clippedOutside":[255,0,0,255],"composited":[0,0,0,0],"operation":"destination-out","native":"function clip() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_draw_image_copies_and_scales_canvas_pixels() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas id='source' width='2' height='2'></canvas><canvas id='target' width='4' height='4'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const source = document.getElementById("source");
                const sourceContext = source.getContext("2d");
                sourceContext.putImageData(new ImageData(new Uint8ClampedArray([
                    255, 0, 0, 255, 0, 255, 0, 255,
                    0, 0, 255, 255, 255, 255, 255, 255,
                ]), 2, 2), 0, 0);
                const targetContext = document.getElementById("target").getContext("2d");
                targetContext.imageSmoothingEnabled = false;
                targetContext.drawImage(source, 0, 0, 4, 4);
                return JSON.stringify([
                    [...targetContext.getImageData(0, 0, 1, 1).data],
                    [...targetContext.getImageData(3, 0, 1, 1).data],
                    [...targetContext.getImageData(0, 3, 1, 1).data],
                    [...targetContext.getImageData(3, 3, 1, 1).data],
                    targetContext.drawImage.toString(),
                ]);
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"[[255,0,0,255],[0,255,0,255],[0,0,255,255],[255,255,255,255],"function drawImage() { [native code] }"]"#,
    );
}

#[test]
fn canvas_image_bitmap_sources_crop_resize_decode_and_pattern() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.imageBitmapResult = "pending";
        const bitmapSourceCanvas = document.createElement("canvas");
        bitmapSourceCanvas.width = 2;
        bitmapSourceCanvas.height = 2;
        bitmapSourceCanvas.getContext("2d").putImageData(new ImageData(new Uint8ClampedArray([
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ]), 2, 2), 0, 0);
        const encoded = bitmapSourceCanvas.toDataURL("image/png");
        const binary = atob(encoded.slice(encoded.indexOf(",") + 1));
        const encodedBytes = new Uint8Array(binary.length);
        for (let index = 0; index < binary.length; index++) encodedBytes[index] = binary.charCodeAt(index);
        const blob = new Blob([encodedBytes], { type: "image/png" });
        const yellow = new ImageData(new Uint8ClampedArray([255, 255, 0, 255]), 1, 1);

        Promise.all([
            createImageBitmap(bitmapSourceCanvas, 0, 0, 2, 2, {
                resizeWidth: 4, resizeHeight: 4, imageOrientation: "flipY", resizeQuality: "high",
            }),
            createImageBitmap(yellow),
            createImageBitmap(blob),
        ]).then(([flipped, yellowBitmap, decoded]) => {
            const target = document.createElement("canvas");
            target.width = 12;
            target.height = 4;
            const context = target.getContext("2d");
            context.drawImage(flipped, 0, 0);
            context.fillStyle = context.createPattern(yellowBitmap, "repeat");
            context.fillRect(4, 0, 4, 4);
            context.drawImage(decoded, 8, 0, 4, 4);
            const pixels = [
                [...context.getImageData(0, 0, 1, 1).data],
                [...context.getImageData(4, 0, 1, 1).data],
                [...context.getImageData(8, 0, 1, 1).data],
            ];
            flipped.close();
            let closed;
            try { context.drawImage(flipped, 0, 0); }
            catch (error) { closed = error.name; }
            yellowBitmap.close();
            decoded.close();
            imageBitmapResult = JSON.stringify({
                bitmap: flipped instanceof ImageBitmap,
                size: [flipped.width, flipped.height],
                pixels,
                closed,
                nativeCreate: createImageBitmap.toString(),
                nativeClose: ImageBitmap.prototype.close.toString(),
            });
        }).catch(error => imageBitmapResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("imageBitmapResult").unwrap().to_string().unwrap(),
        r#"{"bitmap":true,"size":[4,4],"pixels":[[0,0,255,255],[255,255,0,255],[255,0,0,255]],"closed":"InvalidStateError","nativeCreate":"function createImageBitmap() { [native code] }","nativeClose":"function close() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_gradients_render_in_their_creation_coordinate_space() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='10' height='2'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.scale(2, 1);
                const gradient = context.createLinearGradient(0, 0, 5, 0);
                gradient.addColorStop(0, "red");
                gradient.addColorStop(1, "blue");
                context.resetTransform();
                context.fillStyle = gradient;
                context.fillRect(0, 0, 10, 2);
                const left = [...context.getImageData(0, 0, 1, 1).data];
                const middle = [...context.getImageData(5, 0, 1, 1).data];
                const right = [...context.getImageData(9, 0, 1, 1).data];
                return JSON.stringify({
                    constructor: gradient instanceof CanvasGradient,
                    styleIdentity: context.fillStyle === gradient,
                    leftRed: left[0] > 200 && left[2] < 60,
                    middlePurple: middle[0] > 90 && middle[2] > 90,
                    rightBlue: right[2] > 200 && right[0] < 60,
                    native: gradient.addColorStop.toString(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"constructor":true,"styleIdentity":true,"leftRed":true,"middlePurple":true,"rightBlue":true,"native":"function addColorStop() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_conic_gradients_use_skia_sweep_shaders() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='20' height='20'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                const gradient = context.createConicGradient(0, 10, 10);
                gradient.addColorStop(0, "red");
                gradient.addColorStop(0.25, "lime");
                gradient.addColorStop(0.5, "blue");
                gradient.addColorStop(0.75, "white");
                gradient.addColorStop(1, "red");
                context.fillStyle = gradient;
                context.fillRect(0, 0, 20, 20);
                const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];
                const right = pixel(17, 10), down = pixel(10, 17), left = pixel(2, 10), up = pixel(10, 2);
                return JSON.stringify({
                    gradient: gradient instanceof CanvasGradient,
                    rightRed: right[0] > right[1] * 2 && right[0] > right[2] * 2,
                    downGreen: down[1] > down[0] * 2 && down[1] > down[2] * 2,
                    leftBlue: left[2] > left[0] * 2 && left[2] > left[1] * 2,
                    upWhite: up[0] > 200 && up[1] > 200 && up[2] > 200,
                    native: context.createConicGradient.toString(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"gradient":true,"rightRed":true,"downGreen":true,"leftBlue":true,"upWhite":true,"native":"function createConicGradient() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_patterns_repeat_transform_and_render_with_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    page.eval(
        r#"
        const patternSource = document.createElement("canvas");
        patternSource.width = 2;
        patternSource.height = 2;
        const sourceContext = patternSource.getContext("2d");
        sourceContext.putImageData(new ImageData(new Uint8ClampedArray([
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ]), 2, 2), 0, 0);

        const patternTarget = document.createElement("canvas");
        patternTarget.width = 6;
        patternTarget.height = 4;
        const targetContext = patternTarget.getContext("2d");
        const repeatX = targetContext.createPattern(patternSource, "repeat-x");
        targetContext.fillStyle = repeatX;
        targetContext.fillRect(0, 0, 6, 4);
        const repeated = [...targetContext.getImageData(0, 0, 3, 3).data];

        targetContext.clearRect(0, 0, 6, 4);
        const transformed = targetContext.createPattern(patternSource, "repeat");
        transformed.setTransform({ a: 1, b: 0, c: 0, d: 1, e: 1, f: 0 });
        targetContext.fillStyle = transformed;
        targetContext.fillRect(0, 0, 4, 2);
        const shifted = [...targetContext.getImageData(0, 0, 2, 1).data];
        globalThis.patternResult = JSON.stringify({
            pattern: repeatX instanceof CanvasPattern,
            getter: targetContext.fillStyle === transformed,
            repeated,
            shifted,
            native: transformed.setTransform.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("patternResult").unwrap().to_string().unwrap(),
        r#"{"pattern":true,"getter":true,"repeated":[255,0,0,255,0,255,0,255,255,0,0,255,0,0,255,255,255,255,255,255,0,0,255,255,0,0,0,0,0,0,0,0,0,0,0,0],"shifted":[0,255,0,255,255,0,0,255],"native":"function setTransform() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_stroke_caps_joins_and_dash_state_use_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    page.eval(
        r#"
        const strokeCanvas = document.createElement("canvas");
        strokeCanvas.width = 24;
        strokeCanvas.height = 12;
        const strokeContext = strokeCanvas.getContext("2d");
        strokeContext.strokeStyle = "red";
        strokeContext.lineWidth = 4;
        strokeContext.lineCap = "square";
        strokeContext.lineJoin = "bevel";
        strokeContext.miterLimit = 3;
        strokeContext.beginPath();
        strokeContext.moveTo(5, 3);
        strokeContext.lineTo(15, 3);
        strokeContext.stroke();

        strokeContext.lineWidth = 2;
        strokeContext.lineCap = "butt";
        strokeContext.setLineDash([4]);
        strokeContext.lineDashOffset = 0;
        strokeContext.beginPath();
        strokeContext.moveTo(1, 9);
        strokeContext.lineTo(21, 9);
        strokeContext.stroke();
        const alphaAt = (x, y) => strokeContext.getImageData(x, y, 1, 1).data[3];
        strokeContext.save();
        strokeContext.setLineDash([1, 2]);
        strokeContext.lineJoin = "round";
        strokeContext.restore();
        globalThis.strokeResult = JSON.stringify({
            capOutside: alphaAt(2, 3),
            capExtension: alphaAt(3, 3),
            dashOn: alphaAt(2, 9),
            dashOff: alphaAt(7, 9),
            dashAgain: alphaAt(10, 9),
            dash: strokeContext.getLineDash(),
            join: strokeContext.lineJoin,
            miter: strokeContext.miterLimit,
            native: strokeContext.setLineDash.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("strokeResult").unwrap().to_string().unwrap(),
        r#"{"capOutside":0,"capExtension":255,"dashOn":255,"dashOff":0,"dashAgain":255,"dash":[4,4],"join":"bevel","miter":3,"native":"function setLineDash() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_shadows_render_and_follow_saved_state() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    page.eval(
        r#"
        const shadowCanvas = document.createElement("canvas");
        shadowCanvas.width = 24;
        shadowCanvas.height = 12;
        const shadowContext = shadowCanvas.getContext("2d");
        shadowContext.fillStyle = "red";
        shadowContext.shadowColor = "blue";
        shadowContext.shadowBlur = 0;
        shadowContext.shadowOffsetX = 8;
        shadowContext.shadowOffsetY = 0;
        shadowContext.save();
        shadowContext.shadowColor = "transparent";
        shadowContext.shadowOffsetX = 0;
        shadowContext.restore();
        shadowContext.fillRect(2, 2, 4, 4);
        const pixel = (x, y) => [...shadowContext.getImageData(x, y, 1, 1).data];
        globalThis.shadowResult = JSON.stringify({
            source: pixel(3, 3),
            shadow: pixel(11, 3),
            gap: pixel(8, 3),
            color: shadowContext.shadowColor,
            blur: shadowContext.shadowBlur,
            offset: [shadowContext.shadowOffsetX, shadowContext.shadowOffsetY],
            native: shadowContext.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("shadowResult").unwrap().to_string().unwrap(),
        r#"{"source":[255,0,0,255],"shadow":[0,0,255,255],"gap":[0,0,0,0],"color":"blue","blur":0,"offset":[8,0],"native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_filters_chain_color_and_blur_operations_in_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();

    page.eval(
        r#"
        const filterCanvas = document.createElement("canvas");
        filterCanvas.width = 24;
        filterCanvas.height = 12;
        const filterContext = filterCanvas.getContext("2d");
        filterContext.fillStyle = "rgb(100, 50, 20)";
        filterContext.filter = "invert(100%) opacity(50%)";
        filterContext.save();
        filterContext.filter = "none";
        filterContext.restore();
        filterContext.fillRect(2, 2, 4, 4);
        const transformed = [...filterContext.getImageData(3, 3, 1, 1).data];

        filterContext.filter = "drop-shadow(4px 0 red)";
        filterContext.fillStyle = "blue";
        filterContext.fillRect(6, 8, 2, 2);
        const dropShadow = [...filterContext.getImageData(10, 8, 1, 1).data];

        filterContext.filter = "blur(2px)";
        filterContext.fillStyle = "black";
        filterContext.fillRect(14, 4, 2, 2);
        const blurredOutside = filterContext.getImageData(11, 5, 1, 1).data[3];
        const blurredCenter = filterContext.getImageData(14, 5, 1, 1).data[3];
        filterContext.filter = "invalid(1)";
        globalThis.filterResult = JSON.stringify({
            transformed,
            dropShadow,
            blurredOutside: blurredOutside > 0,
            blurredCenter: blurredCenter > blurredOutside && blurredCenter < 255,
            saved: filterContext.filter,
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("filterResult").unwrap().to_string().unwrap(),
        r#"{"transformed":[155,205,235,128],"dropShadow":[255,0,0,255],"blurredOutside":true,"blurredCenter":true,"saved":"blur(2px)"}"#,
    );
}

#[test]
fn canvas_2d_resolves_and_re_evaluates_linear_svg_filters_at_draw_time() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<body></body>").unwrap();

    page.eval(
        r#"
        const namespace = "http://www.w3.org/2000/svg";
        const svg = document.createElementNS(namespace, "svg");
        const filter = document.createElementNS(namespace, "filter");
        filter.setAttribute("id", "dynamic-filter");
        const matrix = document.createElementNS(namespace, "feColorMatrix");
        matrix.setAttribute("values", "0 0 0 0 1  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0");
        const offset = document.createElementNS(namespace, "feOffset");
        offset.setAttribute("dx", "4");
        filter.append(matrix, offset);
        svg.appendChild(filter);

        const unsupported = document.createElementNS(namespace, "filter");
        unsupported.setAttribute("id", "unsupported-filter");
        unsupported.appendChild(document.createElementNS(namespace, "feBlend"));
        svg.appendChild(unsupported);

        const blurFilter = document.createElementNS(namespace, "filter");
        blurFilter.setAttribute("id", "blur-filter");
        const gaussianBlur = document.createElementNS(namespace, "feGaussianBlur");
        gaussianBlur.setAttribute("stdDeviation", "2");
        blurFilter.appendChild(gaussianBlur);
        svg.appendChild(blurFilter);

        const shadowFilter = document.createElementNS(namespace, "filter");
        shadowFilter.setAttribute("id", "shadow-filter");
        const dropShadow = document.createElementNS(namespace, "feDropShadow");
        dropShadow.setAttribute("dx", "4");
        dropShadow.setAttribute("flood-color", "blue");
        dropShadow.setAttribute("flood-opacity", "0.5");
        shadowFilter.appendChild(dropShadow);
        svg.appendChild(shadowFilter);
        document.body.appendChild(svg);

        const canvas = document.createElement("canvas");
        canvas.width = 52;
        canvas.height = 20;
        const context = canvas.getContext("2d");
        const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];

        context.fillStyle = "green";
        context.filter = "url(#dynamic-filter)";
        context.fillRect(1, 2, 2, 2);
        const firstSource = pixel(1, 2);
        const firstOutput = pixel(5, 2);

        offset.setAttribute("dx", "8");
        context.fillRect(1, 8, 2, 2);
        const mutatedOutput = pixel(9, 8);

        context.filter = "url(#unsupported-filter)";
        context.fillRect(20, 2, 2, 2);
        const unsupportedOutput = pixel(20, 2);

        context.filter = "url(https://example.test/filter.svg#remote)";
        context.fillRect(24, 2, 2, 2);

        context.filter = "url(#blur-filter)";
        context.fillStyle = "black";
        context.fillRect(32, 10, 2, 2);
        const blurredOutside = pixel(29, 10)[3] > 0;

        context.filter = "url(#shadow-filter)";
        context.fillStyle = "red";
        context.fillRect(40, 2, 2, 2);
        globalThis.svgFilterResult = JSON.stringify({
            firstSource,
            firstOutput,
            mutatedOutput,
            unsupportedOutput,
            externalOutput: pixel(24, 2),
            blurredOutside,
            shadowOutput: pixel(44, 2),
            serialized: context.filter,
            native: context.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("svgFilterResult").unwrap().to_string().unwrap(),
        r#"{"firstSource":[0,0,0,0],"firstOutput":[255,0,0,255],"mutatedOutput":[255,0,0,255],"unsupportedOutput":[0,128,0,255],"externalOutput":[0,128,0,255],"blurredOutside":true,"shadowOutput":[0,0,255,128],"serialized":"url(#shadow-filter)","native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_svg_filter_graph_resolves_named_and_builtin_inputs() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<body></body>").unwrap();

    page.eval(
        r#"
        const namespace = "http://www.w3.org/2000/svg";
        const element = name => document.createElementNS(namespace, name);
        const svg = element("svg");
        const graph = element("filter");
        graph.setAttribute("id", "graph-filter");

        const red = element("feColorMatrix");
        red.setAttribute("in", "SourceGraphic");
        red.setAttribute("result", "red");
        red.setAttribute("values", "0 0 0 0 1  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0");
        const shifted = element("feOffset");
        shifted.setAttribute("in", "SourceGraphic");
        shifted.setAttribute("dx", "4");
        shifted.setAttribute("result", "shifted");
        const masked = element("feComposite");
        masked.setAttribute("in", "red");
        masked.setAttribute("in2", "SourceGraphic");
        masked.setAttribute("operator", "in");
        masked.setAttribute("result", "masked");
        const alphaShifted = element("feOffset");
        alphaShifted.setAttribute("in", "SourceAlpha");
        alphaShifted.setAttribute("dx", "8");
        alphaShifted.setAttribute("result", "alpha-shifted");
        const merge = element("feMerge");
        merge.setAttribute("result", "merged");
        for (const input of ["shifted", "masked", "alpha-shifted"]) {
            const mergeNode = element("feMergeNode");
            mergeNode.setAttribute("in", input);
            merge.appendChild(mergeNode);
        }
        const blend = element("feBlend");
        blend.setAttribute("in", "merged");
        blend.setAttribute("in2", "SourceGraphic");
        blend.setAttribute("mode", "multiply");
        graph.append(red, shifted, masked, alphaShifted, merge, blend);
        svg.appendChild(graph);

        const arithmetic = element("filter");
        arithmetic.setAttribute("id", "arithmetic-filter");
        const blue = element("feColorMatrix");
        blue.setAttribute("result", "blue");
        blue.setAttribute("values", "0 0 0 0 0  0 0 0 0 0  0 0 0 0 1  0 0 0 1 0");
        const copyBlue = element("feComposite");
        copyBlue.setAttribute("in", "blue");
        copyBlue.setAttribute("in2", "SourceGraphic");
        copyBlue.setAttribute("operator", "arithmetic");
        copyBlue.setAttribute("k2", "1");
        arithmetic.append(blue, copyBlue);
        svg.appendChild(arithmetic);
        document.body.appendChild(svg);

        const canvas = document.createElement("canvas");
        canvas.width = 20;
        canvas.height = 8;
        const context = canvas.getContext("2d");
        const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];
        context.fillStyle = "green";
        context.filter = "url(#graph-filter)";
        context.fillRect(2, 2, 2, 2);
        const graphPixels = [pixel(2, 2), pixel(6, 2), pixel(10, 2)];

        context.filter = "url(#arithmetic-filter)";
        context.fillRect(14, 2, 2, 2);
        globalThis.svgGraphResult = JSON.stringify({
            graphPixels,
            arithmetic: pixel(14, 2),
            native: context.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("svgGraphResult").unwrap().to_string().unwrap(),
        r#"{"graphPixels":[[0,0,0,255],[0,128,0,255],[0,0,0,255]],"arithmetic":[0,0,255,255],"native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_svg_component_transfer_and_morphology_execute_in_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<body></body>").unwrap();

    page.eval(
        r#"
        const namespace = "http://www.w3.org/2000/svg";
        const element = name => document.createElementNS(namespace, name);
        const svg = element("svg");

        const transferFilter = element("filter");
        transferFilter.setAttribute("id", "transfer-filter");
        const transfer = element("feComponentTransfer");
        const red = element("feFuncR");
        red.setAttribute("type", "linear");
        red.setAttribute("slope", "2");
        const green = element("feFuncG");
        green.setAttribute("type", "table");
        green.setAttribute("tableValues", "1 0");
        const blue = element("feFuncB");
        blue.setAttribute("type", "gamma");
        blue.setAttribute("exponent", "2");
        const alpha = element("feFuncA");
        alpha.setAttribute("type", "discrete");
        alpha.setAttribute("tableValues", "0 1");
        transfer.append(red, green, blue, alpha);
        const identity = element("feComponentTransfer");
        transferFilter.append(transfer, identity);
        svg.appendChild(transferFilter);

        const morphologyFilter = element("filter");
        morphologyFilter.setAttribute("id", "morphology-filter");
        const morphology = element("feMorphology");
        morphology.setAttribute("operator", "dilate");
        morphology.setAttribute("radius", "2 1");
        morphologyFilter.appendChild(morphology);
        svg.appendChild(morphologyFilter);
        document.body.appendChild(svg);

        const canvas = document.createElement("canvas");
        canvas.width = 32;
        canvas.height = 12;
        const context = canvas.getContext("2d");
        const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];

        context.filter = "url(#transfer-filter)";
        context.fillStyle = "rgba(64, 128, 128, 0.5)";
        context.fillRect(1, 1, 2, 2);
        const transferred = pixel(1, 1);

        context.filter = "url(#morphology-filter)";
        context.fillStyle = "black";
        context.fillRect(10, 2, 2, 2);
        const dilatedOutside = pixel(8, 2);

        morphology.setAttribute("operator", "erode");
        morphology.setAttribute("radius", "1");
        context.fillRect(20, 2, 5, 5);
        globalThis.svgTransferResult = JSON.stringify({
            transferred,
            dilatedOutside,
            erodedEdge: pixel(20, 2),
            erodedCenter: pixel(22, 4),
            native: context.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("svgTransferResult").unwrap().to_string().unwrap(),
        r#"{"transferred":[128,127,64,255],"dilatedOutside":[0,0,0,255],"erodedEdge":[0,0,0,0],"erodedCenter":[0,0,0,255],"native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_svg_flood_convolution_and_displacement_execute_in_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<body></body>").unwrap();

    page.eval(
        r#"
        const namespace = "http://www.w3.org/2000/svg";
        const element = name => document.createElementNS(namespace, name);
        const svg = element("svg");

        const floodFilter = element("filter");
        floodFilter.setAttribute("id", "flood-filter");
        const flood = element("feFlood");
        flood.setAttribute("flood-color", "red");
        flood.setAttribute("flood-opacity", "0.5");
        flood.setAttribute("result", "paint");
        const floodMask = element("feComposite");
        floodMask.setAttribute("in", "paint");
        floodMask.setAttribute("in2", "SourceGraphic");
        floodMask.setAttribute("operator", "in");
        floodFilter.append(flood, floodMask);
        svg.appendChild(floodFilter);

        const convolutionFilter = element("filter");
        convolutionFilter.setAttribute("id", "convolution-filter");
        const convolution = element("feConvolveMatrix");
        convolution.setAttribute("order", "1");
        convolution.setAttribute("kernelMatrix", "0");
        convolution.setAttribute("divisor", "1");
        convolution.setAttribute("bias", "0.5");
        convolution.setAttribute("preserveAlpha", "true");
        convolutionFilter.appendChild(convolution);
        svg.appendChild(convolutionFilter);

        const shiftedConvolutionFilter = element("filter");
        shiftedConvolutionFilter.setAttribute("id", "shifted-convolution-filter");
        const shiftedConvolution = element("feConvolveMatrix");
        shiftedConvolution.setAttribute("order", "3 1");
        shiftedConvolution.setAttribute("kernelMatrix", "1 0 0");
        shiftedConvolution.setAttribute("divisor", "1");
        shiftedConvolution.setAttribute("targetX", "1");
        shiftedConvolution.setAttribute("targetY", "0");
        shiftedConvolution.setAttribute("edgeMode", "none");
        shiftedConvolutionFilter.appendChild(shiftedConvolution);
        svg.appendChild(shiftedConvolutionFilter);

        const displacementFilter = element("filter");
        displacementFilter.setAttribute("id", "displacement-filter");
        const map = element("feFlood");
        map.setAttribute("flood-color", "rgb(255, 128, 128)");
        map.setAttribute("result", "map");
        const displacement = element("feDisplacementMap");
        displacement.setAttribute("in", "SourceGraphic");
        displacement.setAttribute("in2", "map");
        displacement.setAttribute("scale", "4");
        displacement.setAttribute("xChannelSelector", "R");
        displacement.setAttribute("yChannelSelector", "B");
        displacementFilter.append(map, displacement);
        svg.appendChild(displacementFilter);
        document.body.appendChild(svg);

        const canvas = document.createElement("canvas");
        canvas.width = 40;
        canvas.height = 12;
        const context = canvas.getContext("2d");
        const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];

        context.filter = "url(#flood-filter)";
        context.fillStyle = "black";
        context.fillRect(2, 2, 3, 3);
        flood.setAttribute("flood-color", "blue");
        flood.setAttribute("flood-opacity", "1");
        context.fillRect(6, 2, 3, 3);

        context.filter = "url(#convolution-filter)";
        context.fillStyle = "rgba(20, 40, 60, 0.5)";
        context.fillRect(10, 2, 3, 3);

        context.filter = "url(#displacement-filter)";
        context.fillStyle = "blue";
        context.fillRect(22, 2, 3, 3);

        const source = document.createElement("canvas");
        source.width = 5;
        source.height = 1;
        const sourceContext = source.getContext("2d");
        sourceContext.fillStyle = "lime";
        sourceContext.fillRect(2, 0, 1, 1);
        context.filter = "url(#shifted-convolution-filter)";
        context.drawImage(source, 10, 8);

        globalThis.svgAdvancedFilterResult = JSON.stringify({
            flood: pixel(2, 2),
            mutatedFlood: pixel(6, 2),
            convolution: pixel(10, 2),
            displacedLeft: pixel(20, 2),
            displacedRightEdge: pixel(24, 2),
            convolutionShifted: pixel(11, 8),
            convolutionOriginal: pixel(12, 8),
            native: context.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("svgAdvancedFilterResult")
            .unwrap()
            .to_string()
            .unwrap(),
        r#"{"flood":[255,0,0,128],"mutatedFlood":[0,0,255,255],"convolution":[128,128,128,128],"displacedLeft":[0,0,255,255],"displacedRightEdge":[0,0,0,0],"convolutionShifted":[0,255,0,255],"convolutionOriginal":[0,0,0,0],"native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_svg_diffuse_and_specular_lighting_execute_in_skia() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<body></body>").unwrap();

    page.eval(
        r#"
        const namespace = "http://www.w3.org/2000/svg";
        const element = name => document.createElementNS(namespace, name);
        const svg = element("svg");

        const distantFilter = element("filter");
        distantFilter.setAttribute("id", "distant-light");
        const bump = element("feGaussianBlur");
        bump.setAttribute("in", "SourceAlpha");
        bump.setAttribute("stdDeviation", "0");
        bump.setAttribute("result", "bump");
        const distantLighting = element("feDiffuseLighting");
        distantLighting.setAttribute("in", "bump");
        distantLighting.setAttribute("lighting-color", "red");
        const distant = element("feDistantLight");
        distant.setAttribute("elevation", "90");
        distantLighting.appendChild(distant);
        distantFilter.append(bump, distantLighting);
        svg.appendChild(distantFilter);

        const pointFilter = element("filter");
        pointFilter.setAttribute("id", "point-light");
        const pointLighting = element("feDiffuseLighting");
        pointLighting.setAttribute("lighting-color", "white");
        const point = element("fePointLight");
        point.setAttribute("x", "15");
        point.setAttribute("y", "3");
        point.setAttribute("z", "10");
        pointLighting.appendChild(point);
        pointFilter.appendChild(pointLighting);
        svg.appendChild(pointFilter);

        const spotFilter = element("filter");
        spotFilter.setAttribute("id", "spot-light");
        const spotLighting = element("feSpecularLighting");
        spotLighting.setAttribute("lighting-color", "blue");
        spotLighting.setAttribute("specularConstant", "1");
        spotLighting.setAttribute("specularExponent", "1");
        const spot = element("feSpotLight");
        spot.setAttribute("x", "27");
        spot.setAttribute("y", "3");
        spot.setAttribute("z", "10");
        spot.setAttribute("pointsAtX", "27");
        spot.setAttribute("pointsAtY", "3");
        spot.setAttribute("pointsAtZ", "0");
        spotLighting.appendChild(spot);
        spotFilter.appendChild(spotLighting);
        svg.appendChild(spotFilter);
        document.body.appendChild(svg);

        const canvas = document.createElement("canvas");
        canvas.width = 40;
        canvas.height = 8;
        const context = canvas.getContext("2d");
        const pixel = (x, y) => [...context.getImageData(x, y, 1, 1).data];

        context.filter = "url(#distant-light)";
        context.fillStyle = "black";
        context.fillRect(2, 1, 4, 4);
        const distantRed = pixel(3, 2);

        distantLighting.setAttribute("lighting-color", "lime");
        context.fillRect(8, 1, 4, 4);
        const mutatedDistant = pixel(9, 2);

        context.filter = "url(#point-light)";
        context.fillRect(13, 1, 4, 4);
        const pointPixel = pixel(15, 3);

        context.filter = "url(#spot-light)";
        context.fillRect(25, 1, 4, 4);
        const spotPixel = pixel(27, 3);

        globalThis.svgLightingResult = JSON.stringify({
            distantRed,
            mutatedDistant,
            point: pointPixel,
            spot: spotPixel,
            native: context.fillRect.toString(),
        });
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("svgLightingResult").unwrap().to_string().unwrap(),
        r#"{"distantRed":[255,0,0,255],"mutatedDistant":[0,255,0,255],"point":[254,254,254,255],"spot":[1,1,255,255],"native":"function fillRect() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_text_uses_bundled_skia_font_metrics_and_pixels() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='96' height='32'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.font = "20px sans-serif";
                context.fillStyle = "black";
                const metrics = context.measureText("Brimp");
                const proportionalI = context.measureText("ii").width;
                const proportionalW = context.measureText("WW").width;
                context.fillText("Brimp", 2, 24);
                const pixels = context.getImageData(0, 0, 96, 32).data;
                let painted = 0;
                for (let index = 3; index < pixels.length; index += 4) if (pixels[index] !== 0) painted++;
                context.font = "20px monospace";
                const monospaceI = context.measureText("ii").width;
                const monospaceW = context.measureText("WW").width;
                return JSON.stringify({
                    font: context.font,
                    metrics: metrics instanceof TextMetrics && metrics.width > 20 && metrics.actualBoundingBoxAscent > 5,
                    painted: painted > 30,
                    families: Math.abs(proportionalI - proportionalW) > 1
                        && Math.abs(monospaceI - monospaceW) < 0.01,
                    nativeMeasure: context.measureText.toString(),
                    nativeFill: context.fillText.toString(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"font":"20px monospace","metrics":true,"painted":true,"families":true,"nativeMeasure":"function measureText() { [native code] }","nativeFill":"function fillText() { [native code] }"}"#,
    );
}

#[test]
fn canvas_2d_uses_the_bundled_colrv1_face_for_emoji() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='128' height='48'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.font = "30px sans-serif";
                context.fillStyle = "black";
                context.fillText("😀", 2, 36);
                const fallbackWidth = context.measureText("😀").width;

                context.font = "30px emoji";
                context.fillText("👩‍💻", 42, 36);
                const explicitWidth = context.measureText("👩‍💻").width;

                const pixels = context.getImageData(0, 0, 128, 48).data;
                let painted = 0;
                let colored = 0;
                for (let index = 0; index < pixels.length; index += 4) {
                    if (pixels[index + 3] === 0) continue;
                    painted++;
                    if (pixels[index] !== pixels[index + 1]
                        || pixels[index + 1] !== pixels[index + 2]) colored++;
                }
                return JSON.stringify({
                    font: context.font,
                    fallbackWidth,
                    explicitWidth,
                    painted,
                    colored,
                    native: context.fillText.toString(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let observed: serde_json::Value = serde_json::from_str(&observed).unwrap();
    assert_eq!(observed["font"], "30px emoji");
    assert!(observed["fallbackWidth"].as_f64().unwrap() > 10.0);
    assert!(observed["explicitWidth"].as_f64().unwrap() > 10.0);
    assert!(observed["painted"].as_u64().unwrap() > 100);
    assert!(observed["colored"].as_u64().unwrap() > 50);
    assert_eq!(observed["native"], "function fillText() { [native code] }");
}

#[test]
fn canvas_2d_text_shapes_combining_marks_and_respects_direction() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).build())
        .unwrap();
    page.set_content("<canvas width='96' height='64'></canvas>")
        .unwrap();

    let observed = page
        .eval(
            r#"(() => {
                const context = document.querySelector("canvas").getContext("2d");
                context.font = "20px sans-serif";
                const decomposed = context.measureText("A\u0301");
                const composed = context.measureText("\u00c1");

                context.direction = "ltr";
                context.fillText("abc \u202eDEF\u202c ghi", 2, 24);
                const mixed = context.getImageData(0, 0, 96, 32).data;
                context.clearRect(0, 0, 96, 64);
                context.fillText("abc FED ghi", 2, 24);
                const expected = context.getImageData(0, 0, 96, 32).data;

                let bidiMatch = true;
                for (let index = 0; index < mixed.length; index++) {
                    if (mixed[index] !== expected[index]) { bidiMatch = false; break; }
                }
                context.clearRect(0, 0, 96, 64);
                context.direction = "ltr";
                context.textAlign = "right";
                context.fillText("abc", 50, 24, 10);
                const constrained = context.getImageData(0, 0, 96, 32).data;
                let firstPaintedX = 96;
                for (let index = 3; index < constrained.length; index += 4) {
                    if (constrained[index] !== 0) {
                        firstPaintedX = Math.min(firstPaintedX, ((index - 3) / 4) % 96);
                    }
                }
                return JSON.stringify({
                    combined: Math.abs(decomposed.width - composed.width) < 0.001
                        && Math.abs(decomposed.actualBoundingBoxLeft - composed.actualBoundingBoxLeft) < 0.001
                        && Math.abs(decomposed.actualBoundingBoxRight - composed.actualBoundingBoxRight) < 0.001,
                    direction: context.direction,
                    bidiMatch,
                    maxWidthAligned: firstPaintedX >= 35 && firstPaintedX < 50,
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        observed,
        r#"{"combined":true,"direction":"ltr","bidiMatch":true,"maxWidthAligned":true}"#,
    );
}
