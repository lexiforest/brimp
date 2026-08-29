use std::sync::Arc;

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn webgl_float_color_buffer_extensions_follow_angle_and_render_when_available() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const first = document.createElement("canvas").getContext("webgl");
                if (!first) return "no-angle";
                const firstSupported = first.getSupportedExtensions();
                const float1 = firstSupported.includes("WEBGL_color_buffer_float")
                    ? first.getExtension("WEBGL_color_buffer_float") : null;
                const half1 = firstSupported.includes("EXT_color_buffer_half_float")
                    ? first.getExtension("EXT_color_buffer_half_float") : null;
                const blend1 = firstSupported.includes("EXT_float_blend")
                    ? first.getExtension("EXT_float_blend") : null;
                const webgl1 = {
                    float: float1 === null ? null : {
                        stable: float1 === first.getExtension("webgl_color_buffer_float"),
                        constants: [float1.RGBA32F_EXT, float1.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT,
                            float1.UNSIGNED_NORMALIZED_EXT],
                    },
                    half: half1 === null ? null : {
                        stable: half1 === first.getExtension("ext_color_buffer_half_float"),
                        constants: [half1.RGBA16F_EXT, half1.RGB16F_EXT,
                            half1.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT,
                            half1.UNSIGNED_NORMALIZED_EXT],
                    },
                    blend: blend1 === null ? null
                        : blend1 === first.getExtension("ext_float_blend"),
                };

                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 1;
                const second = canvas.getContext("webgl2");
                if (!second) return JSON.stringify({ webgl1, webgl2: null });
                const secondSupported = second.getSupportedExtensions();
                const float2 = secondSupported.includes("EXT_color_buffer_float")
                    ? second.getExtension("EXT_color_buffer_float") : null;
                const half2 = secondSupported.includes("EXT_color_buffer_half_float")
                    ? second.getExtension("EXT_color_buffer_half_float") : null;
                const blend2 = secondSupported.includes("EXT_float_blend")
                    ? second.getExtension("EXT_float_blend") : null;
                let render = null;
                if (float2 || half2) {
                    const texture = second.createTexture();
                    second.bindTexture(second.TEXTURE_2D, texture);
                    second.texParameteri(second.TEXTURE_2D, second.TEXTURE_MIN_FILTER, second.NEAREST);
                    second.texParameteri(second.TEXTURE_2D, second.TEXTURE_MAG_FILTER, second.NEAREST);
                    second.texImage2D(second.TEXTURE_2D, 0,
                        float2 ? second.RGBA32F : second.RGBA16F, 1, 1, 0,
                        second.RGBA, float2 ? second.FLOAT : second.HALF_FLOAT, null);
                    const framebuffer = second.createFramebuffer();
                    second.bindFramebuffer(second.FRAMEBUFFER, framebuffer);
                    second.framebufferTexture2D(second.FRAMEBUFFER, second.COLOR_ATTACHMENT0,
                        second.TEXTURE_2D, texture, 0);
                    const complete = second.checkFramebufferStatus(second.FRAMEBUFFER)
                        === second.FRAMEBUFFER_COMPLETE;
                    const pixel = new Float32Array(4);
                    if (complete) {
                        second.clearBufferfv(second.COLOR, 0, [1.5, 0.25, 0.5, 1]);
                        second.readPixels(0, 0, 1, 1, second.RGBA, second.FLOAT, pixel);
                    }
                    render = { complete, pixel: [...pixel], error: second.getError() };
                }
                return JSON.stringify({
                    webgl1,
                    webgl2: {
                        float: float2 === null ? null
                            : float2 === second.getExtension("ext_color_buffer_float"),
                        half: half2 === null ? null
                            : half2 === second.getExtension("ext_color_buffer_half_float"),
                        blend: blend2 === null ? null
                            : blend2 === second.getExtension("ext_float_blend"),
                        render,
                    },
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    if result != "no-angle" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        if let Some(float) = result["webgl1"]["float"].as_object() {
            assert_eq!(float["stable"], true);
            assert_eq!(
                float["constants"],
                serde_json::json!([0x8814, 0x8211, 0x8C17])
            );
        }
        if let Some(half) = result["webgl1"]["half"].as_object() {
            assert_eq!(half["stable"], true);
            assert_eq!(
                half["constants"],
                serde_json::json!([0x881A, 0x881B, 0x8211, 0x8C17])
            );
        }
        if !result["webgl1"]["blend"].is_null() {
            assert_eq!(result["webgl1"]["blend"], true);
        }
        if let Some(second) = result["webgl2"].as_object() {
            for extension in ["float", "half", "blend"] {
                assert!(second[extension].is_null() || second[extension] == true);
            }
            if let Some(render) = second["render"].as_object() {
                assert_eq!(render["complete"], true);
                assert_eq!(render["error"], 0);
                let pixel = render["pixel"].as_array().unwrap();
                for (actual, expected) in pixel.iter().zip([1.5, 0.25, 0.5, 1.0]) {
                    assert!((actual.as_f64().unwrap() - expected).abs() < 0.01);
                }
            }
        }
    }
}

#[test]
fn webgl_precision_and_depth_textures_use_angle_when_available() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const webgl1 = document.createElement("canvas").getContext("webgl");
                if (!webgl1) return "no-angle";
                const supported = webgl1.getSupportedExtensions();
                const webgl1Results = {};
                const allocate = (name, type, pixels) => {
                    if (!supported.includes(name)) return null;
                    const extension = webgl1.getExtension(name);
                    const texture = webgl1.createTexture();
                    webgl1.bindTexture(webgl1.TEXTURE_2D, texture);
                    webgl1.texParameteri(webgl1.TEXTURE_2D, webgl1.TEXTURE_MIN_FILTER, webgl1.NEAREST);
                    webgl1.texParameteri(webgl1.TEXTURE_2D, webgl1.TEXTURE_MAG_FILTER, webgl1.NEAREST);
                    webgl1.texImage2D(
                        webgl1.TEXTURE_2D, 0, webgl1.RGBA, 1, 1, 0,
                        webgl1.RGBA, type(extension), pixels,
                    );
                    return {
                        stable: extension === webgl1.getExtension(name.toLowerCase()),
                        error: webgl1.getError(),
                    };
                };
                webgl1Results.float = allocate(
                    "OES_texture_float",
                    () => webgl1.FLOAT,
                    new Float32Array([0.25, 0.5, 0.75, 1]),
                );
                webgl1Results.half = allocate(
                    "OES_texture_half_float",
                    extension => extension.HALF_FLOAT_OES,
                    new Uint16Array([0x3400, 0x3800, 0x3A00, 0x3C00]),
                );
                webgl1Results.floatLinear = supported.includes("OES_texture_float_linear")
                    ? webgl1.getExtension("OES_texture_float_linear")
                        === webgl1.getExtension("oes_texture_float_linear")
                    : null;
                webgl1Results.halfLinear = supported.includes("OES_texture_half_float_linear")
                    ? webgl1.getExtension("OES_texture_half_float_linear")
                        === webgl1.getExtension("oes_texture_half_float_linear")
                    : null;
                if (supported.includes("WEBGL_depth_texture")) {
                    const extension = webgl1.getExtension("WEBGL_depth_texture");
                    const color = webgl1.createTexture();
                    webgl1.bindTexture(webgl1.TEXTURE_2D, color);
                    webgl1.texImage2D(webgl1.TEXTURE_2D, 0, webgl1.RGBA, 2, 2, 0,
                        webgl1.RGBA, webgl1.UNSIGNED_BYTE, null);
                    const depth = webgl1.createTexture();
                    webgl1.bindTexture(webgl1.TEXTURE_2D, depth);
                    webgl1.texImage2D(webgl1.TEXTURE_2D, 0, webgl1.DEPTH_COMPONENT, 2, 2, 0,
                        webgl1.DEPTH_COMPONENT, webgl1.UNSIGNED_SHORT, null);
                    const framebuffer = webgl1.createFramebuffer();
                    webgl1.bindFramebuffer(webgl1.FRAMEBUFFER, framebuffer);
                    webgl1.framebufferTexture2D(webgl1.FRAMEBUFFER, webgl1.COLOR_ATTACHMENT0,
                        webgl1.TEXTURE_2D, color, 0);
                    webgl1.framebufferTexture2D(webgl1.FRAMEBUFFER, webgl1.DEPTH_ATTACHMENT,
                        webgl1.TEXTURE_2D, depth, 0);
                    webgl1Results.depth = {
                        packedType: extension.UNSIGNED_INT_24_8_WEBGL === 0x84FA,
                        complete: webgl1.checkFramebufferStatus(webgl1.FRAMEBUFFER)
                            === webgl1.FRAMEBUFFER_COMPLETE,
                        error: webgl1.getError(),
                    };
                } else {
                    webgl1Results.depth = null;
                }

                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl2");
                if (!context) return JSON.stringify({ webgl1: webgl1Results, webgl2: null });
                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    uniform float depth;
                    const vec2 positions[3] = vec2[3](
                        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
                    void main() { gl_Position = vec4(positions[gl_VertexID], depth, 1.0); }`);
                const sampleFragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    uniform sampler2D sourceTexture;
                    out vec4 outputColor;
                    void main() { outputColor = texture(sourceTexture, vec2(0.5)); }`);
                const sampleProgram = context.createProgram();
                context.attachShader(sampleProgram, vertex);
                context.attachShader(sampleProgram, sampleFragment);
                context.linkProgram(sampleProgram);
                context.useProgram(sampleProgram);
                context.uniform1f(context.getUniformLocation(sampleProgram, "depth"), 0);
                context.uniform1i(context.getUniformLocation(sampleProgram, "sourceTexture"), 0);
                const uploadAndRead = (internalFormat, type, pixels) => {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, internalFormat, 1, 1, 0,
                        context.RGBA, type, pixels);
                    context.bindFramebuffer(context.FRAMEBUFFER, null);
                    context.viewport(0, 0, 4, 4);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const pixel = new Uint8Array(4);
                    context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);
                    return [...pixel];
                };
                const floatPixel = uploadAndRead(
                    context.RGBA32F,
                    context.FLOAT,
                    new Float32Array([0.25, 0.5, 0.75, 1]),
                );
                const halfPixel = uploadAndRead(
                    context.RGBA16F,
                    context.HALF_FLOAT,
                    new Uint16Array([0x3400, 0x3800, 0x3A00, 0x3C00]),
                );

                const color = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, color);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8, 4, 4, 0,
                    context.RGBA, context.UNSIGNED_BYTE, null);
                const depth = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, depth);
                context.texImage2D(context.TEXTURE_2D, 0, context.DEPTH_COMPONENT24, 4, 4, 0,
                    context.DEPTH_COMPONENT, context.UNSIGNED_INT, null);
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.TEXTURE_2D, color, 0);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.DEPTH_ATTACHMENT,
                    context.TEXTURE_2D, depth, 0);
                const depthComplete = context.checkFramebufferStatus(context.FRAMEBUFFER)
                    === context.FRAMEBUFFER_COMPLETE;
                const colorFragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    uniform vec4 color;
                    out vec4 outputColor;
                    void main() { outputColor = color; }`);
                const colorProgram = context.createProgram();
                context.attachShader(colorProgram, vertex);
                context.attachShader(colorProgram, colorFragment);
                context.linkProgram(colorProgram);
                context.useProgram(colorProgram);
                const depthLocation = context.getUniformLocation(colorProgram, "depth");
                const colorLocation = context.getUniformLocation(colorProgram, "color");
                context.viewport(0, 0, 4, 4);
                context.clearColor(0, 0, 0, 1);
                context.clearDepth(1);
                context.clear(context.COLOR_BUFFER_BIT | context.DEPTH_BUFFER_BIT);
                context.enable(context.DEPTH_TEST);
                context.uniform1f(depthLocation, -0.5);
                context.uniform4f(colorLocation, 0, 1, 0, 1);
                context.drawArrays(context.TRIANGLES, 0, 3);
                context.uniform1f(depthLocation, 0.5);
                context.uniform4f(colorLocation, 1, 0, 0, 1);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const depthPixel = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, depthPixel);
                return JSON.stringify({
                    webgl1: webgl1Results,
                    webgl2: {
                        constants: context.HALF_FLOAT === 0x140B
                            && context.RGBA16F === 0x881A
                            && context.RGBA32F === 0x8814
                            && context.DEPTH_COMPONENT24 === 0x81A6
                            && WebGLRenderingContext.prototype.HALF_FLOAT === undefined,
                        compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                            && context.getShaderParameter(sampleFragment, context.COMPILE_STATUS)
                            && context.getShaderParameter(colorFragment, context.COMPILE_STATUS),
                        linked: context.getProgramParameter(sampleProgram, context.LINK_STATUS)
                            && context.getProgramParameter(colorProgram, context.LINK_STATUS),
                        floatPixel,
                        halfPixel,
                        depthComplete,
                        depthPixel: [...depthPixel],
                        error: context.getError(),
                    },
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    if result != "no-angle" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        for name in ["float", "half"] {
            if !result["webgl1"][name].is_null() {
                assert_eq!(result["webgl1"][name]["stable"], true);
                assert_eq!(result["webgl1"][name]["error"], 0);
            }
        }
        for name in ["floatLinear", "halfLinear"] {
            if !result["webgl1"][name].is_null() {
                assert_eq!(result["webgl1"][name], true);
            }
        }
        if !result["webgl1"]["depth"].is_null() {
            assert_eq!(result["webgl1"]["depth"]["packedType"], true);
            assert_eq!(result["webgl1"]["depth"]["complete"], true);
            assert_eq!(result["webgl1"]["depth"]["error"], 0);
        }
        if !result["webgl2"].is_null() {
            assert_eq!(result["webgl2"]["constants"], true);
            assert_eq!(result["webgl2"]["compiled"], true);
            assert_eq!(result["webgl2"]["linked"], true);
            assert_eq!(
                result["webgl2"]["floatPixel"],
                serde_json::json!([64, 128, 191, 255])
            );
            assert_eq!(
                result["webgl2"]["halfPixel"],
                serde_json::json!([64, 128, 191, 255])
            );
            assert_eq!(result["webgl2"]["depthComplete"], true);
            assert_eq!(
                result["webgl2"]["depthPixel"],
                serde_json::json!([0, 255, 0, 255])
            );
            assert_eq!(result["webgl2"]["error"], 0);
        }
    }
}

#[test]
fn webgl_texture_storage_and_framebuffer_copies_use_angle_when_available() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const runWebGL1 = () => {
                    const canvas = document.createElement("canvas");
                    canvas.width = canvas.height = 4;
                    const context = canvas.getContext("webgl");
                    if (!context) return null;
                    const source = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, source);
                    context.texImage2D(context.TEXTURE_2D, 0, context.RGBA, 2, 2, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null);
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, source, 0);
                    context.clearColor(12 / 255, 34 / 255, 56 / 255, 1);
                    context.clear(context.COLOR_BUFFER_BIT);

                    const copied = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, copied);
                    context.copyTexImage2D(context.TEXTURE_2D, 0, context.RGBA, 0, 0, 2, 2, 0);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, copied, 0);
                    const copiedPixel = new Uint8Array(4);
                    context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, copiedPixel);

                    const patched = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, patched);
                    context.texImage2D(context.TEXTURE_2D, 0, context.RGBA, 2, 2, 0,
                        context.RGBA, context.UNSIGNED_BYTE, new Uint8Array(16));
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, source, 0);
                    context.copyTexSubImage2D(context.TEXTURE_2D, 0, 1, 1, 0, 0, 1, 1);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, patched, 0);
                    const patchedPixel = new Uint8Array(4);
                    const untouchedPixel = new Uint8Array(4);
                    context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, patchedPixel);
                    context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, untouchedPixel);
                    return {
                        copiedPixel: [...copiedPixel],
                        patchedPixel: [...patchedPixel],
                        untouchedPixel: [...untouchedPixel],
                        error: context.getError(),
                        native: [
                            context.copyTexImage2D.toString(),
                            context.copyTexSubImage2D.toString(),
                        ],
                    };
                };

                const runWebGL2 = () => {
                    const canvas = document.createElement("canvas");
                    canvas.width = canvas.height = 4;
                    const context = canvas.getContext("webgl2");
                    if (!context) return null;
                    const source = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, source);
                    context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8, 2, 2, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null);
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, source, 0);
                    context.clearColor(0, 1, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);

                    const immutable = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, immutable);
                    context.texStorage2D(context.TEXTURE_2D, 1, context.RGBA8, 2, 2);
                    context.texSubImage2D(context.TEXTURE_2D, 0, 0, 0, 2, 2,
                        context.RGBA, context.UNSIGNED_BYTE, new Uint8Array([
                            0, 0, 255, 255, 0, 0, 255, 255,
                            0, 0, 255, 255, 0, 0, 255, 255,
                        ]));
                    context.copyTexSubImage2D(context.TEXTURE_2D, 0, 1, 1, 0, 0, 1, 1);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, immutable, 0);
                    const immutableCopied = new Uint8Array(4);
                    const immutableUntouched = new Uint8Array(4);
                    context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, immutableCopied);
                    context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, immutableUntouched);

                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, source, 0);
                    const layered = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D_ARRAY, layered);
                    context.texStorage3D(context.TEXTURE_2D_ARRAY, 1, context.RGBA8, 2, 2, 2);
                    const blueLayers = new Uint8Array(32);
                    for (let offset = 0; offset < blueLayers.length; offset += 4) {
                        blueLayers.set([0, 0, 255, 255], offset);
                    }
                    context.texSubImage3D(context.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 2, 2, 2,
                        context.RGBA, context.UNSIGNED_BYTE, blueLayers);
                    context.copyTexSubImage3D(context.TEXTURE_2D_ARRAY, 0, 0, 0, 1, 0, 0, 1, 1);
                    context.framebufferTextureLayer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        layered, 0, 1);
                    const layerCopied = new Uint8Array(4);
                    const layerUntouched = new Uint8Array(4);
                    context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, layerCopied);
                    context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, layerUntouched);
                    return {
                        immutableCopied: [...immutableCopied],
                        immutableUntouched: [...immutableUntouched],
                        layerCopied: [...layerCopied],
                        layerUntouched: [...layerUntouched],
                        complete: context.checkFramebufferStatus(context.FRAMEBUFFER)
                            === context.FRAMEBUFFER_COMPLETE,
                        error: context.getError(),
                        native: [
                            context.texStorage2D.toString(),
                            context.texStorage3D.toString(),
                            context.copyTexSubImage3D.toString(),
                        ],
                    };
                };
                return JSON.stringify({ webgl1: runWebGL1(), webgl2: runWebGL2() });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if !result["webgl1"].is_null() {
        assert_eq!(
            result["webgl1"]["copiedPixel"],
            serde_json::json!([12, 34, 56, 255])
        );
        assert_eq!(
            result["webgl1"]["patchedPixel"],
            serde_json::json!([12, 34, 56, 255])
        );
        assert_eq!(
            result["webgl1"]["untouchedPixel"],
            serde_json::json!([0, 0, 0, 0])
        );
        assert_eq!(result["webgl1"]["error"], 0);
        assert_eq!(
            result["webgl1"]["native"],
            serde_json::json!([
                "function copyTexImage2D() { [native code] }",
                "function copyTexSubImage2D() { [native code] }",
            ])
        );
    }
    if !result["webgl2"].is_null() {
        assert_eq!(
            result["webgl2"]["immutableCopied"],
            serde_json::json!([0, 255, 0, 255])
        );
        assert_eq!(
            result["webgl2"]["immutableUntouched"],
            serde_json::json!([0, 0, 255, 255])
        );
        assert_eq!(
            result["webgl2"]["layerCopied"],
            serde_json::json!([0, 255, 0, 255])
        );
        assert_eq!(
            result["webgl2"]["layerUntouched"],
            serde_json::json!([0, 0, 255, 255])
        );
        assert_eq!(result["webgl2"]["complete"], true);
        assert_eq!(result["webgl2"]["error"], 0);
        assert_eq!(
            result["webgl2"]["native"],
            serde_json::json!([
                "function texStorage2D() { [native code] }",
                "function texStorage3D() { [native code] }",
                "function copyTexSubImage3D() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl2_sampler_objects_override_texture_sampling_state() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl2");
                if (!context) return "no-angle";
                const texture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, texture);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.LINEAR);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.LINEAR);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_S, context.CLAMP_TO_EDGE);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_T, context.CLAMP_TO_EDGE);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8, 2, 1, 0,
                    context.RGBA, context.UNSIGNED_BYTE, new Uint8Array([
                        255, 0, 0, 255, 0, 0, 255, 255,
                    ]));
                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    const vec2 positions[3] = vec2[3](
                        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
                    void main() { gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0); }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    uniform sampler2D sourceTexture;
                    out vec4 outputColor;
                    void main() { outputColor = texture(sourceTexture, vec2(0.5)); }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                context.uniform1i(context.getUniformLocation(program, "sourceTexture"), 0);
                context.viewport(0, 0, 4, 4);

                const sampler = context.createSampler();
                context.samplerParameteri(sampler, context.TEXTURE_MIN_FILTER, context.NEAREST);
                context.samplerParameteri(sampler, context.TEXTURE_MAG_FILTER, context.NEAREST);
                context.samplerParameteri(sampler, context.TEXTURE_WRAP_S, context.CLAMP_TO_EDGE);
                context.samplerParameteri(sampler, context.TEXTURE_WRAP_T, context.CLAMP_TO_EDGE);
                context.samplerParameterf(sampler, context.TEXTURE_MIN_LOD, -2.5);
                context.bindSampler(0, sampler);
                const bound = context.getParameter(context.SAMPLER_BINDING) === sampler;
                context.drawArrays(context.TRIANGLES, 0, 3);
                const nearest = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, nearest);

                context.bindSampler(0, null);
                const unbound = context.getParameter(context.SAMPLER_BINDING) === null;
                context.drawArrays(context.TRIANGLES, 0, 3);
                const linear = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, linear);
                const reflected = context.getSamplerParameter(sampler, context.TEXTURE_MIN_FILTER)
                        === context.NEAREST
                    && context.getSamplerParameter(sampler, context.TEXTURE_WRAP_S)
                        === context.CLAMP_TO_EDGE
                    && context.getSamplerParameter(sampler, context.TEXTURE_MIN_LOD) === -2.5;
                const object = sampler instanceof WebGLSampler && context.isSampler(sampler);
                const native = [
                    context.createSampler.toString(),
                    context.bindSampler.toString(),
                    context.getSamplerParameter.toString(),
                ];
                context.bindSampler(1, sampler);
                context.deleteSampler(sampler);
                const deleted = !context.isSampler(sampler);
                context.activeTexture(context.TEXTURE1);
                const deletionUnbound = context.getParameter(context.SAMPLER_BINDING) === null;
                return JSON.stringify({
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    bound,
                    unbound,
                    reflected,
                    object,
                    deleted,
                    deletionUnbound,
                    nearest: [...nearest],
                    linear: [...linear],
                    error: context.getError(),
                    native,
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    if result != "no-angle" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        for name in [
            "compiled",
            "linked",
            "bound",
            "unbound",
            "reflected",
            "object",
            "deleted",
            "deletionUnbound",
        ] {
            assert_eq!(result[name], true, "failed sampler check: {name}");
        }
        assert_eq!(result["nearest"], serde_json::json!([0, 0, 255, 255]));
        assert_eq!(result["linear"], serde_json::json!([128, 0, 128, 255]));
        assert_eq!(result["error"], 0);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function createSampler() { [native code] }",
                "function bindSampler() { [native code] }",
                "function getSamplerParameter() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl_draw_buffers_writes_multiple_color_attachments() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl");
                if (!context) return "no-angle";
                const extension = context.getExtension("WEBGL_draw_buffers");
                if (!extension) return "no-extension";
                const texture = () => {
                    const value = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, value);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(
                        context.TEXTURE_2D, 0, context.RGBA, 4, 4, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null,
                    );
                    return value;
                };
                const first = texture();
                const second = texture();
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(
                    context.FRAMEBUFFER,
                    extension.COLOR_ATTACHMENT0_WEBGL,
                    context.TEXTURE_2D,
                    first,
                    0,
                );
                context.framebufferTexture2D(
                    context.FRAMEBUFFER,
                    extension.COLOR_ATTACHMENT1_WEBGL,
                    context.TEXTURE_2D,
                    second,
                    0,
                );
                extension.drawBuffersWEBGL([
                    extension.COLOR_ATTACHMENT0_WEBGL,
                    extension.COLOR_ATTACHMENT1_WEBGL,
                ]);
                const complete = context.checkFramebufferStatus(context.FRAMEBUFFER)
                    === context.FRAMEBUFFER_COMPLETE;
                const vertex = context.createShader(context.VERTEX_SHADER);
                context.shaderSource(vertex, `attribute vec2 position;
                    void main() { gl_Position = vec4(position, 0.0, 1.0); }`);
                context.compileShader(vertex);
                const fragment = context.createShader(context.FRAGMENT_SHADER);
                context.shaderSource(fragment, `#extension GL_EXT_draw_buffers : require
                    precision mediump float;
                    void main() {
                        gl_FragData[0] = vec4(1.0, 0.0, 0.0, 1.0);
                        gl_FragData[1] = vec4(0.0, 1.0, 0.0, 1.0);
                    }`);
                context.compileShader(fragment);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                const vertices = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, vertices);
                context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                    -1, -1, 1, -1, 0, 1,
                ]), context.STATIC_DRAW);
                const position = context.getAttribLocation(program, "position");
                context.enableVertexAttribArray(position);
                context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const red = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, red);

                context.framebufferTexture2D(
                    context.FRAMEBUFFER,
                    extension.COLOR_ATTACHMENT1_WEBGL,
                    context.TEXTURE_2D,
                    null,
                    0,
                );
                context.framebufferTexture2D(
                    context.FRAMEBUFFER,
                    extension.COLOR_ATTACHMENT0_WEBGL,
                    context.TEXTURE_2D,
                    second,
                    0,
                );
                extension.drawBuffersWEBGL([extension.COLOR_ATTACHMENT0_WEBGL]);
                const green = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, green);
                return JSON.stringify({
                    complete,
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    limits: [
                        context.getParameter(extension.MAX_DRAW_BUFFERS_WEBGL),
                        context.getParameter(extension.MAX_COLOR_ATTACHMENTS_WEBGL),
                    ],
                    red: [...red],
                    green: [...green],
                    error: context.getError(),
                    native: extension.drawBuffersWEBGL.toString(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_ne!(
        result, "no-extension",
        "ANGLE did not expose WEBGL_draw_buffers"
    );
    if result != "no-angle" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["complete"], true);
        assert_eq!(result["compiled"], true);
        assert_eq!(result["linked"], true);
        assert!(result["limits"][0].as_u64().unwrap() >= 2);
        assert!(result["limits"][1].as_u64().unwrap() >= 2);
        assert_eq!(result["red"], serde_json::json!([255, 0, 0, 255]));
        assert_eq!(result["green"], serde_json::json!([0, 255, 0, 255]));
        assert_eq!(result["error"], 0);
        assert_eq!(
            result["native"],
            "function drawBuffersWEBGL() { [native code] }"
        );
    }
}

#[test]
fn webgl2_layers_and_multisample_framebuffers_use_angle_when_available() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).webgl(true).build())
        .unwrap();
    page.set_content("<canvas id='canvas' width='4' height='4'></canvas>")
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.getElementById("canvas");
                const context = canvas.getContext("webgl2");
                const api = typeof WebGL2RenderingContext.prototype.texImage3D === "function"
                    && typeof WebGL2RenderingContext.prototype.texSubImage3D === "function"
                    && typeof WebGL2RenderingContext.prototype.framebufferTextureLayer === "function"
                    && typeof WebGL2RenderingContext.prototype.renderbufferStorageMultisample === "function"
                    && typeof WebGL2RenderingContext.prototype.blitFramebuffer === "function"
                    && WebGL2RenderingContext.prototype.texImage3D.toString() === "function texImage3D() { [native code] }"
                    && WebGL2RenderingContext.prototype.blitFramebuffer.toString() === "function blitFramebuffer() { [native code] }";
                if (!context) return JSON.stringify({ available: false, api });

                const texture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D_ARRAY, texture);
                context.texParameteri(context.TEXTURE_2D_ARRAY, context.TEXTURE_MIN_FILTER, context.NEAREST);
                context.texParameteri(context.TEXTURE_2D_ARRAY, context.TEXTURE_MAG_FILTER, context.NEAREST);
                const pixels = new Uint8Array(2 * 2 * 2 * 4);
                for (let index = 0; index < 4; index += 1) pixels.set([255, 0, 0, 255], index * 4);
                for (let index = 4; index < 8; index += 1) pixels.set([0, 255, 0, 255], index * 4);
                context.texImage3D(context.TEXTURE_2D_ARRAY, 0, context.RGBA8, 2, 2, 2, 0, context.RGBA, context.UNSIGNED_BYTE, pixels);
                context.texSubImage3D(context.TEXTURE_2D_ARRAY, 0, 1, 1, 1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE,
                    new Uint8Array([0, 0, 255, 255]));

                const layerFramebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, layerFramebuffer);
                context.framebufferTextureLayer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0, texture, 0, 1);
                const layerComplete = context.checkFramebufferStatus(context.FRAMEBUFFER) === context.FRAMEBUFFER_COMPLETE;
                const green = new Uint8Array(4);
                const blue = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, green);
                context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, blue);

                const multisampleFramebuffer = context.createFramebuffer();
                const multisampleColor = context.createRenderbuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, multisampleFramebuffer);
                context.bindRenderbuffer(context.RENDERBUFFER, multisampleColor);
                const samples = Math.min(4, context.getParameter(context.MAX_SAMPLES));
                context.renderbufferStorageMultisample(context.RENDERBUFFER, samples, context.RGBA8, 2, 2);
                context.framebufferRenderbuffer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0, context.RENDERBUFFER, multisampleColor);
                const multisampleComplete = context.checkFramebufferStatus(context.FRAMEBUFFER) === context.FRAMEBUFFER_COMPLETE;
                context.viewport(0, 0, 2, 2);
                context.clearColor(1, 1, 0, 1);
                context.clear(context.COLOR_BUFFER_BIT);

                context.bindFramebuffer(context.READ_FRAMEBUFFER, multisampleFramebuffer);
                context.bindFramebuffer(context.DRAW_FRAMEBUFFER, layerFramebuffer);
                context.readBuffer(context.COLOR_ATTACHMENT0);
                context.drawBuffers([context.COLOR_ATTACHMENT0]);
                context.blitFramebuffer(0, 0, 2, 2, 0, 0, 2, 2, context.COLOR_BUFFER_BIT, context.NEAREST);
                context.bindFramebuffer(context.READ_FRAMEBUFFER, layerFramebuffer);
                const resolved = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, resolved);
                const bindingsBeforeExport = context.getParameter(context.TEXTURE_BINDING_2D_ARRAY) === texture
                    && context.getParameter(context.READ_FRAMEBUFFER_BINDING) === layerFramebuffer
                    && context.getParameter(context.DRAW_FRAMEBUFFER_BINDING) === layerFramebuffer;
                const png = canvas.toDataURL().startsWith("data:image/png;base64,iVBORw0KGgo");
                const bindingsAfterExport = context.getParameter(context.READ_FRAMEBUFFER_BINDING) === layerFramebuffer
                    && context.getParameter(context.DRAW_FRAMEBUFFER_BINDING) === layerFramebuffer;
                const error = context.getError();

                context.bindFramebuffer(context.FRAMEBUFFER, null);
                context.deleteRenderbuffer(multisampleColor);
                context.deleteFramebuffer(multisampleFramebuffer);
                context.deleteFramebuffer(layerFramebuffer);
                context.deleteTexture(texture);
                return JSON.stringify({
                    available: true,
                    api,
                    layerComplete,
                    multisampleComplete,
                    samples,
                    green: [...green],
                    blue: [...blue],
                    resolved: [...resolved],
                    bindingsBeforeExport,
                    bindingsAfterExport,
                    png,
                    error,
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    if result["available"] == true {
        assert_eq!(result["layerComplete"], true);
        assert_eq!(result["multisampleComplete"], true);
        assert!(
            result["samples"]
                .as_u64()
                .is_some_and(|samples| samples > 0)
        );
        assert_eq!(result["green"], serde_json::json!([0, 255, 0, 255]));
        assert_eq!(result["blue"], serde_json::json!([0, 0, 255, 255]));
        assert_eq!(result["resolved"], serde_json::json!([255, 255, 0, 255]));
        assert_eq!(result["bindingsBeforeExport"], true);
        assert_eq!(result["bindingsAfterExport"], true);
        assert_eq!(result["png"], true);
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl2_framebuffer_clear_invalidate_and_internal_format_queries_use_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const context = document.createElement("canvas").getContext("webgl2");
                const api = [
                    "clearBufferiv", "clearBufferuiv", "clearBufferfv", "clearBufferfi",
                    "invalidateFramebuffer", "invalidateSubFramebuffer", "getInternalformatParameter",
                ].every(name => typeof WebGL2RenderingContext.prototype[name] === "function"
                    && WebGL2RenderingContext.prototype[name].toString().includes("[native code]"));
                if (!context) return JSON.stringify({ available: false, api });

                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                const texture = (internalFormat, format, type) => {
                    const value = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, value);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, internalFormat, 1, 1, 0, format, type, null);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, value, 0);
                    return value;
                };

                const normalized = texture(context.RGBA8, context.RGBA, context.UNSIGNED_BYTE);
                context.clearBufferfv(context.COLOR, 0, new Float32Array([9, 0, 1, 0, 1]), 1);
                const normalizedPixel = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, normalizedPixel);

                const unsigned = texture(context.RGBA8UI, context.RGBA_INTEGER, context.UNSIGNED_BYTE);
                context.clearBufferuiv(context.COLOR, 0, new Uint32Array([11, 22, 33, 44]));
                const unsignedPixel = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA_INTEGER, context.UNSIGNED_BYTE, unsignedPixel);

                const signed = texture(context.RGBA8I, context.RGBA_INTEGER, context.BYTE);
                context.clearBufferiv(context.COLOR, 0, new Int32Array([-5, 10, -20, 30]));
                const signedPixel = new Int8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA_INTEGER, context.BYTE, signedPixel);

                const depthStencil = context.createRenderbuffer();
                context.bindRenderbuffer(context.RENDERBUFFER, depthStencil);
                context.renderbufferStorage(context.RENDERBUFFER, context.DEPTH24_STENCIL8, 1, 1);
                context.framebufferRenderbuffer(context.FRAMEBUFFER, context.DEPTH_STENCIL_ATTACHMENT,
                    context.RENDERBUFFER, depthStencil);
                const complete = context.checkFramebufferStatus(context.FRAMEBUFFER) === context.FRAMEBUFFER_COMPLETE;
                context.clearBufferfv(context.DEPTH, 0, new Float32Array([0.25]));
                context.clearBufferiv(context.STENCIL, 0, new Int32Array([7]));
                context.clearBufferfi(context.DEPTH_STENCIL, 0, 0.75, 3);
                context.invalidateSubFramebuffer(context.FRAMEBUFFER, [context.COLOR_ATTACHMENT0], 0, 0, 1, 1);
                context.invalidateFramebuffer(context.FRAMEBUFFER, [context.DEPTH_STENCIL_ATTACHMENT]);

                const samples = context.getInternalformatParameter(
                    context.RENDERBUFFER, context.RGBA8, context.SAMPLES);
                const error = context.getError();
                return JSON.stringify({
                    available: true,
                    api,
                    complete,
                    normalizedPixel: [...normalizedPixel],
                    unsignedPixel: [...unsignedPixel],
                    signedPixel: [...signedPixel],
                    samplesType: samples instanceof Int32Array,
                    samples: [...samples],
                    samplesDescending: [...samples].every((value, index, values) => index === 0 || values[index - 1] >= value),
                    error,
                    objects: [normalized, unsigned, signed, depthStencil].every(Boolean),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    if result["available"] == true {
        assert_eq!(result["complete"], true);
        assert_eq!(
            result["normalizedPixel"],
            serde_json::json!([0, 255, 0, 255])
        );
        assert_eq!(result["unsignedPixel"], serde_json::json!([11, 22, 33, 44]));
        assert_eq!(result["signedPixel"], serde_json::json!([-5, 10, -20, 30]));
        assert_eq!(result["samplesType"], true);
        assert_eq!(result["samplesDescending"], true);
        assert_eq!(result["error"], 0);
        assert_eq!(result["objects"], true);
    }
}

#[test]
fn webgl2_stencil_texturing_and_shared_exponent_render_through_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const webgl1 = document.createElement("canvas").getContext("webgl");
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl2");
                if (!context) return JSON.stringify({ available: false });
                const stencil = context.getExtension("WEBGL_stencil_texturing");
                const shared = context.getExtension("WEBGL_render_shared_exponent");
                const webgl1Absent = webgl1 === null || (
                    webgl1.getExtension("WEBGL_stencil_texturing") === null
                    && webgl1.getExtension("WEBGL_render_shared_exponent") === null
                );
                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const makeProgram = (fragmentSource) => {
                    const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                        in vec2 position;
                        void main() { gl_Position = vec4(position, 0.0, 1.0); }`);
                    const fragment = compile(context.FRAGMENT_SHADER, fragmentSource);
                    const program = context.createProgram();
                    context.attachShader(program, vertex);
                    context.attachShader(program, fragment);
                    context.linkProgram(program);
                    return {
                        program,
                        vertex: context.getShaderParameter(vertex, context.COMPILE_STATUS),
                        vertexLog: context.getShaderInfoLog(vertex),
                        fragment: context.getShaderParameter(fragment, context.COMPILE_STATUS),
                        fragmentLog: context.getShaderInfoLog(fragment),
                        linked: context.getProgramParameter(program, context.LINK_STATUS),
                        log: context.getProgramInfoLog(program),
                    };
                };
                const positions = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, positions);
                context.bufferData(context.ARRAY_BUFFER,
                    new Float32Array([-1, -1, 3, -1, -1, 3]), context.STATIC_DRAW);
                const readPixel = () => {
                    const pixel = new Uint8Array(4);
                    context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);
                    return [...pixel];
                };
                let stencilResult = null;
                if (stencil) {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, context.DEPTH24_STENCIL8,
                        1, 1, 0, context.DEPTH_STENCIL, context.UNSIGNED_INT_24_8,
                        new Uint32Array([0x800000AB]));
                    const allocationError = context.getError();
                    const initial = context.getTexParameter(
                        context.TEXTURE_2D, stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL);
                    context.texParameteri(context.TEXTURE_2D,
                        stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL,
                        stencil.STENCIL_INDEX_WEBGL);
                    const changed = context.getTexParameter(
                        context.TEXTURE_2D, stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL);
                    const linked = makeProgram(`#version 300 es
                        precision highp float;
                        precision highp usampler2D;
                        uniform usampler2D sourceTexture;
                        out vec4 color;
                        void main() {
                            uint value = texture(sourceTexture, vec2(0.5)).r;
                            color = vec4(float(value) / 255.0, 0.0, 0.0, 1.0);
                        }`);
                    context.bindFramebuffer(context.FRAMEBUFFER, null);
                    context.useProgram(linked.program);
                    context.bindBuffer(context.ARRAY_BUFFER, positions);
                    const position = context.getAttribLocation(linked.program, "position");
                    context.enableVertexAttribArray(position);
                    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const pixel = readPixel();
                    context.texParameterf(context.TEXTURE_2D,
                        stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL, context.DEPTH_COMPONENT);
                    const reset = context.getTexParameter(
                        context.TEXTURE_2D, stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL);
                    context.texParameteri(context.TEXTURE_2D,
                        stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL, 0);
                    const invalid = context.getError();
                    stencilResult = {
                        constants: [stencil.DEPTH_STENCIL_TEXTURE_MODE_WEBGL,
                            stencil.STENCIL_INDEX_WEBGL],
                        allocationError,
                        initial,
                        changed,
                        reset,
                        invalid,
                        linked,
                        pixel,
                        stable: stencil === context.getExtension("webgl_stencil_texturing"),
                        frozen: Object.isFrozen(stencil),
                    };
                }
                let sharedResult = null;
                if (shared) {
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, context.RGB9_E5,
                        1, 1, 0, context.RGB, context.UNSIGNED_INT_5_9_9_9_REV, null);
                    const allocationError = context.getError();
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, texture, 0);
                    const textureStatus = context.checkFramebufferStatus(context.FRAMEBUFFER);
                    context.colorMask(true, true, true, true);
                    context.clearColor(0.25, 0.5, 1.0, 1.0);
                    context.clear(context.COLOR_BUFFER_BIT);
                    const clearError = context.getError();
                    context.bindFramebuffer(context.FRAMEBUFFER, null);
                    const linked = makeProgram(`#version 300 es
                        precision highp float;
                        uniform sampler2D sourceTexture;
                        out vec4 color;
                        void main() {
                            color = vec4(texture(sourceTexture, vec2(0.5)).rgb, 1.0);
                        }`);
                    context.useProgram(linked.program);
                    context.bindBuffer(context.ARRAY_BUFFER, positions);
                    const position = context.getAttribLocation(linked.program, "position");
                    context.enableVertexAttribArray(position);
                    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const pixel = readPixel();
                    const sampleError = context.getError();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.colorMask(true, false, true, true);
                    context.clear(context.COLOR_BUFFER_BIT);
                    const maskError = context.getError();
                    context.colorMask(true, true, true, true);
                    const renderbuffer = context.createRenderbuffer();
                    context.bindRenderbuffer(context.RENDERBUFFER, renderbuffer);
                    context.renderbufferStorage(context.RENDERBUFFER, context.RGB9_E5, 1, 1);
                    const renderbufferError = context.getError();
                    context.framebufferRenderbuffer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.RENDERBUFFER, renderbuffer);
                    sharedResult = {
                        allocationError,
                        textureStatus,
                        clearError,
                        linked,
                        pixel,
                        sampleError,
                        maskError,
                        renderbufferError,
                        renderbufferFormat: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_INTERNAL_FORMAT),
                        renderbufferStatus: context.checkFramebufferStatus(context.FRAMEBUFFER),
                        stable: shared
                            === context.getExtension("webgl_render_shared_exponent"),
                        frozen: Object.isFrozen(shared),
                    };
                }
                return JSON.stringify({
                    available: true,
                    webgl1Absent,
                    stencil: stencilResult,
                    shared: sharedResult,
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["available"] != true {
        return;
    }
    assert_eq!(result["webgl1Absent"], true);
    if !result["stencil"].is_null() {
        assert_eq!(
            result["stencil"]["constants"],
            serde_json::json!([0x90EA, 0x1901])
        );
        assert_eq!(result["stencil"]["allocationError"], 0);
        assert_eq!(result["stencil"]["initial"], 0x1902);
        assert_eq!(result["stencil"]["changed"], 0x1901);
        assert_eq!(result["stencil"]["reset"], 0x1902);
        assert_eq!(result["stencil"]["invalid"], 0x0500);
        assert_eq!(result["stencil"]["linked"]["vertex"], true);
        assert_eq!(result["stencil"]["linked"]["fragment"], true);
        assert_eq!(result["stencil"]["linked"]["linked"], true);
        assert_eq!(
            result["stencil"]["pixel"],
            serde_json::json!([171, 0, 0, 255])
        );
        assert_eq!(result["stencil"]["stable"], true);
        assert_eq!(result["stencil"]["frozen"], true);
    }
    if !result["shared"].is_null() {
        assert_eq!(result["shared"]["allocationError"], 0);
        assert_eq!(result["shared"]["textureStatus"], 0x8CD5);
        assert_eq!(result["shared"]["clearError"], 0);
        assert_eq!(result["shared"]["linked"]["vertex"], true);
        assert_eq!(result["shared"]["linked"]["fragment"], true);
        assert_eq!(result["shared"]["linked"]["linked"], true);
        assert_eq!(result["shared"]["sampleError"], 0);
        let pixel = result["shared"]["pixel"].as_array().unwrap();
        assert!((pixel[0].as_i64().unwrap() - 64).abs() <= 1);
        assert!((pixel[1].as_i64().unwrap() - 128).abs() <= 1);
        assert!((pixel[2].as_i64().unwrap() - 255).abs() <= 1);
        assert_eq!(pixel[3], 255);
        assert_eq!(result["shared"]["maskError"], 0x0502);
        assert_eq!(result["shared"]["renderbufferError"], 0);
        assert_eq!(result["shared"]["renderbufferFormat"], 0x8C3D);
        assert_eq!(result["shared"]["renderbufferStatus"], 0x8CD5);
        assert_eq!(result["shared"]["stable"], true);
        assert_eq!(result["shared"]["frozen"], true);
    }
    assert_eq!(result["error"], 0);
}

#[test]
fn webgl2_normalized_texture_and_render_formats_follow_angle_capabilities() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const webgl1 = document.createElement("canvas").getContext("webgl");
                const context = document.createElement("canvas").getContext("webgl2");
                if (!context) return JSON.stringify({ available: false });
                const norm = context.getExtension("EXT_texture_norm16");
                const renderSnorm = context.getExtension("EXT_render_snorm");
                const webgl1Absent = webgl1 === null || (
                    webgl1.getExtension("EXT_texture_norm16") === null
                    && webgl1.getExtension("EXT_render_snorm") === null
                );
                const result = {
                    available: true,
                    webgl1Absent,
                    norm: null,
                    renderSnorm: null,
                };
                context.pixelStorei(context.UNPACK_ALIGNMENT, 1);
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                const textureCase = (internalFormat, format, type, data, renderable, readable) => {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(
                        context.TEXTURE_2D, 0, internalFormat, 1, 1, 0, format, type, data);
                    const allocationError = context.getError();
                    context.texSubImage2D(
                        context.TEXTURE_2D, 0, 0, 0, 1, 1, format, type, data);
                    const subUploadError = context.getError();
                    let status = null;
                    let pixel = null;
                    let readError = null;
                    let renderError = null;
                    if (renderable) {
                        context.framebufferTexture2D(
                            context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                            context.TEXTURE_2D, texture, 0);
                        status = context.checkFramebufferStatus(context.FRAMEBUFFER);
                        if (status === context.FRAMEBUFFER_COMPLETE && readable) {
                            const bytes = new Uint8Array(4);
                            context.readPixels(
                                0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, bytes);
                            pixel = [...bytes];
                            readError = context.getError();
                        }
                        if (status === context.FRAMEBUFFER_COMPLETE) {
                            context.clearColor(0.25, 0.5, 0.75, 1);
                            context.clear(context.COLOR_BUFFER_BIT);
                            renderError = context.getError();
                        }
                    }
                    return {
                        allocationError, subUploadError, status, pixel, readError, renderError,
                    };
                };
                const renderbufferCase = internalFormat => {
                    const renderbuffer = context.createRenderbuffer();
                    context.bindRenderbuffer(context.RENDERBUFFER, renderbuffer);
                    context.renderbufferStorage(context.RENDERBUFFER, internalFormat, 1, 1);
                    const error = context.getError();
                    context.framebufferRenderbuffer(
                        context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.RENDERBUFFER, renderbuffer);
                    return {
                        error,
                        format: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_INTERNAL_FORMAT),
                        status: context.checkFramebufferStatus(context.FRAMEBUFFER),
                    };
                };
                if (norm) {
                    const constants = [
                        norm.R16_EXT, norm.RG16_EXT, norm.RGB16_EXT, norm.RGBA16_EXT,
                        norm.R16_SNORM_EXT, norm.RG16_SNORM_EXT,
                        norm.RGB16_SNORM_EXT, norm.RGBA16_SNORM_EXT,
                    ];
                    const unsignedCases = [
                        textureCase(norm.R16_EXT, context.RED, context.UNSIGNED_SHORT,
                            new Uint16Array([32768]), true, true),
                        textureCase(norm.RG16_EXT, context.RG, context.UNSIGNED_SHORT,
                            new Uint16Array([16384, 49151]), true, true),
                        textureCase(norm.RGB16_EXT, context.RGB, context.UNSIGNED_SHORT,
                            new Uint16Array([8192, 32768, 57343]), false, false),
                        textureCase(norm.RGBA16_EXT, context.RGBA, context.UNSIGNED_SHORT,
                            new Uint16Array([65535, 32768, 0, 65535]), true, true),
                    ];
                    const signedCases = [
                        textureCase(norm.R16_SNORM_EXT, context.RED, context.SHORT,
                            new Int16Array([16384]), renderSnorm !== null, false),
                        textureCase(norm.RG16_SNORM_EXT, context.RG, context.SHORT,
                            new Int16Array([-16384, 16384]), renderSnorm !== null, false),
                        textureCase(norm.RGB16_SNORM_EXT, context.RGB, context.SHORT,
                            new Int16Array([-32768, 0, 32767]), false, false),
                        textureCase(norm.RGBA16_SNORM_EXT, context.RGBA, context.SHORT,
                            new Int16Array([-32768, 0, 16384, 32767]), renderSnorm !== null, false),
                    ];
                    const immutable = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, immutable);
                    context.texStorage2D(context.TEXTURE_2D, 1, norm.RG16_EXT, 1, 1);
                    context.texSubImage2D(context.TEXTURE_2D, 0, 0, 0, 1, 1,
                        context.RG, context.UNSIGNED_SHORT, new Uint16Array([12345, 54321]));
                    const immutableError = context.getError();
                    result.norm = {
                        constants,
                        stable: norm === context.getExtension("ext_texture_norm16"),
                        frozen: Object.isFrozen(norm),
                        unsignedCases,
                        signedCases,
                        immutableError,
                        immutable: context.getTexParameter(
                            context.TEXTURE_2D, context.TEXTURE_IMMUTABLE_FORMAT),
                        renderbuffer: renderbufferCase(norm.R16_EXT),
                    };
                }
                if (renderSnorm) {
                    result.renderSnorm = {
                        stable: renderSnorm === context.getExtension("ext_render_snorm"),
                        frozen: Object.isFrozen(renderSnorm),
                        rgba8: textureCase(
                            context.RGBA8_SNORM, context.RGBA, context.BYTE,
                            new Int8Array([-128, 0, 64, 127]), true, false),
                        renderbuffer: renderbufferCase(context.RGBA8_SNORM),
                    };
                }
                return JSON.stringify(result);
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["available"] != true {
        return;
    }
    assert_eq!(result["webgl1Absent"], true);
    if !result["norm"].is_null() {
        assert_eq!(
            result["norm"]["constants"],
            serde_json::json!([
                0x822A, 0x822C, 0x8054, 0x805B, 0x8F98, 0x8F99, 0x8F9A, 0x8F9B
            ])
        );
        assert_eq!(result["norm"]["stable"], true);
        assert_eq!(result["norm"]["frozen"], true);
        for case in result["norm"]["unsignedCases"].as_array().unwrap() {
            assert_eq!(case["allocationError"], 0);
            assert_eq!(case["subUploadError"], 0);
        }
        for case in result["norm"]["signedCases"].as_array().unwrap() {
            assert_eq!(case["allocationError"], 0);
            assert_eq!(case["subUploadError"], 0);
        }
        for index in [0, 1, 3] {
            assert_eq!(result["norm"]["unsignedCases"][index]["status"], 0x8CD5);
            assert_eq!(result["norm"]["unsignedCases"][index]["renderError"], 0);
        }
        assert_eq!(
            result["norm"]["unsignedCases"][3]["pixel"],
            serde_json::json!([255, 128, 0, 255])
        );
        assert_eq!(result["norm"]["immutableError"], 0);
        assert_eq!(result["norm"]["immutable"], true);
        assert_eq!(result["norm"]["renderbuffer"]["error"], 0);
        assert_eq!(result["norm"]["renderbuffer"]["format"], 0x822A);
        assert_eq!(result["norm"]["renderbuffer"]["status"], 0x8CD5);
        if !result["renderSnorm"].is_null() {
            for index in [0, 1, 3] {
                assert_eq!(result["norm"]["signedCases"][index]["status"], 0x8CD5);
                assert_eq!(result["norm"]["signedCases"][index]["renderError"], 0);
            }
        }
    }
    if !result["renderSnorm"].is_null() {
        assert_eq!(result["renderSnorm"]["stable"], true);
        assert_eq!(result["renderSnorm"]["frozen"], true);
        assert_eq!(result["renderSnorm"]["rgba8"]["allocationError"], 0);
        assert_eq!(result["renderSnorm"]["rgba8"]["subUploadError"], 0);
        assert_eq!(result["renderSnorm"]["rgba8"]["status"], 0x8CD5);
        assert_eq!(result["renderSnorm"]["rgba8"]["renderError"], 0);
        assert_eq!(result["renderSnorm"]["renderbuffer"]["error"], 0);
        assert_eq!(result["renderSnorm"]["renderbuffer"]["format"], 0x8F97);
        assert_eq!(result["renderSnorm"]["renderbuffer"]["status"], 0x8CD5);
    }
}

#[test]
fn webgl2_pixel_pack_and_unpack_buffers_transfer_2d_and_layered_pixels() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const context = document.createElement("canvas").getContext("webgl2");
                const constants = WebGL2RenderingContext.prototype.PIXEL_PACK_BUFFER === 0x88EB
                    && WebGL2RenderingContext.prototype.PIXEL_UNPACK_BUFFER === 0x88EC
                    && WebGL2RenderingContext.prototype.UNPACK_ROW_LENGTH === 0x0CF2
                    && WebGL2RenderingContext.prototype.PACK_SKIP_PIXELS === 0x0D04
                    && WebGLRenderingContext.prototype.PIXEL_PACK_BUFFER === undefined;
                if (!context) return JSON.stringify({ available: false, constants });

                const red = [10, 20, 30, 255];
                const green = [40, 50, 60, 255];
                const blue = [70, 80, 90, 255];
                const unpack = context.createBuffer();
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, unpack);
                context.bufferData(context.PIXEL_UNPACK_BUFFER,
                    new Uint8Array([0, 0, 0, 0, ...red, ...green, ...blue]), context.STATIC_DRAW);

                const texture2d = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, texture2d);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8, 2, 1, 0,
                    context.RGBA, context.UNSIGNED_BYTE, 4);
                context.texSubImage2D(context.TEXTURE_2D, 0, 1, 0, 1, 1,
                    context.RGBA, context.UNSIGNED_BYTE, 12);
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.TEXTURE_2D, texture2d, 0);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, null);

                const typed2d = new Uint8Array(12);
                context.readPixels(0, 0, 2, 1, context.RGBA, context.UNSIGNED_BYTE, typed2d, 2);

                const textureArray = context.createTexture();
                context.bindTexture(context.TEXTURE_2D_ARRAY, textureArray);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, unpack);
                context.texImage3D(context.TEXTURE_2D_ARRAY, 0, context.RGBA8, 1, 1, 2, 0,
                    context.RGBA, context.UNSIGNED_BYTE, 4);
                context.texSubImage3D(context.TEXTURE_2D_ARRAY, 0, 0, 0, 1, 1, 1, 1,
                    context.RGBA, context.UNSIGNED_BYTE, 12);
                context.framebufferTextureLayer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    textureArray, 0, 1);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, null);

                const pack = context.createBuffer();
                context.bindBuffer(context.PIXEL_PACK_BUFFER, pack);
                context.bufferData(context.PIXEL_PACK_BUFFER, 12, context.STREAM_READ);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, 4);
                const packed = new Uint8Array(12);
                context.getBufferSubData(context.PIXEL_PACK_BUFFER, 0, packed);
                const packBinding = context.getParameter(context.PIXEL_PACK_BUFFER_BINDING) === pack;
                context.bindBuffer(context.PIXEL_PACK_BUFFER, null);

                const typedLayer = new Uint8Array(8);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, typedLayer, 4);
                return JSON.stringify({
                    available: true,
                    constants,
                    typed2d: [...typed2d],
                    packed: [...packed],
                    typedLayer: [...typedLayer],
                    packBinding,
                    unpackBindingCleared: context.getParameter(
                        context.PIXEL_UNPACK_BUFFER_BINDING) === null,
                    complete: context.checkFramebufferStatus(context.FRAMEBUFFER)
                        === context.FRAMEBUFFER_COMPLETE,
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["constants"], true, "{result}");
    if result["available"] == true {
        assert_eq!(
            result["typed2d"],
            serde_json::json!([0, 0, 10, 20, 30, 255, 70, 80, 90, 255, 0, 0])
        );
        assert_eq!(
            result["packed"],
            serde_json::json!([0, 0, 0, 0, 70, 80, 90, 255, 0, 0, 0, 0])
        );
        assert_eq!(
            result["typedLayer"],
            serde_json::json!([0, 0, 0, 0, 70, 80, 90, 255])
        );
        assert_eq!(result["packBinding"], true);
        assert_eq!(result["unpackBindingCleared"], true);
        assert_eq!(result["complete"], true);
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl2_compressed_etc_textures_use_typed_and_pbo_uploads_when_supported() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl2");
                const api = [
                    "compressedTexImage2D", "compressedTexSubImage2D",
                    "compressedTexImage3D", "compressedTexSubImage3D",
                ].every(name => typeof WebGL2RenderingContext.prototype[name] === "function"
                    && WebGL2RenderingContext.prototype[name].toString().includes("[native code]"));
                if (!context) return JSON.stringify({ available: false, api });
                const extension = context.getExtension("WEBGL_compressed_texture_etc");
                if (!extension) return JSON.stringify({ available: true, api, supported: false });

                const format = extension.COMPRESSED_RGB8_ETC2;
                const block = new Uint8Array(8);
                const rangedBlock = new Uint8Array([99, ...block, 99]);
                const texture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, texture);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                context.compressedTexImage2D(
                    context.TEXTURE_2D, 0, format, 4, 4, 0, rangedBlock, 1, 8);
                context.compressedTexSubImage2D(
                    context.TEXTURE_2D, 0, 0, 0, 4, 4, format, rangedBlock, 1, 8);

                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    void main() {
                        vec2 positions[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
                        gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    uniform sampler2D sourceTexture;
                    out vec4 color;
                    void main() { color = texture(sourceTexture, vec2(0.5)); }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                context.uniform1i(context.getUniformLocation(program, "sourceTexture"), 0);
                context.viewport(0, 0, 4, 4);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const pixel = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);

                const unpack = context.createBuffer();
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, unpack);
                context.bufferData(context.PIXEL_UNPACK_BUFFER,
                    new Uint8Array([0, 0, 0, 0, ...block, ...block]), context.STATIC_DRAW);
                const pbo2d = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, pbo2d);
                context.compressedTexImage2D(context.TEXTURE_2D, 0, format, 4, 4, 0, 8, 4);
                context.compressedTexSubImage2D(context.TEXTURE_2D, 0, 0, 0, 4, 4, format, 8, 4);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, null);

                const layers = new Uint8Array([99, ...block, ...block, 99]);
                const arrayTexture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D_ARRAY, arrayTexture);
                context.compressedTexImage3D(context.TEXTURE_2D_ARRAY, 0, format,
                    4, 4, 2, 0, layers, 1, 16);
                context.compressedTexSubImage3D(context.TEXTURE_2D_ARRAY, 0,
                    0, 0, 1, 4, 4, 1, format, rangedBlock, 1, 8);

                const pboArray = context.createTexture();
                context.bindTexture(context.TEXTURE_2D_ARRAY, pboArray);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, unpack);
                context.compressedTexImage3D(context.TEXTURE_2D_ARRAY, 0, format,
                    4, 4, 2, 0, 16, 4);
                context.compressedTexSubImage3D(context.TEXTURE_2D_ARRAY, 0,
                    0, 0, 1, 4, 4, 1, format, 8, 4);
                context.bindBuffer(context.PIXEL_UNPACK_BUFFER, null);

                const formats = context.getParameter(context.COMPRESSED_TEXTURE_FORMATS);
                return JSON.stringify({
                    available: true,
                    api,
                    supported: true,
                    extensionStable: extension
                        === context.getExtension("webgl_compressed_texture_etc"),
                    constants: format === 0x9274
                        && extension.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC === 0x9279,
                    formatsType: formats instanceof Uint32Array,
                    formats: [...formats],
                    pixel: [...pixel],
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    objects: [texture, pbo2d, arrayTexture, pboArray].every(Boolean),
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    if result["supported"] == true {
        assert_eq!(result["extensionStable"], true);
        assert_eq!(result["constants"], true);
        assert_eq!(result["formatsType"], true);
        assert_eq!(result["formats"].as_array().unwrap().len(), 10);
        assert_eq!(result["formats"][4], 0x9274);
        assert!(result["pixel"][0].as_u64().unwrap() <= 10);
        assert!(result["pixel"][1].as_u64().unwrap() <= 10);
        assert!(result["pixel"][2].as_u64().unwrap() <= 10);
        assert_eq!(result["pixel"][3], 255);
        assert_eq!(result["linked"], true);
        assert_eq!(result["objects"], true);
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl_compression_and_anisotropy_extensions_follow_angle_capabilities() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const extensionNames = [
                    "EXT_texture_filter_anisotropic",
                    "WEBGL_compressed_texture_s3tc",
                    "WEBGL_compressed_texture_s3tc_srgb",
                    "EXT_texture_compression_bptc",
                    "EXT_texture_compression_rgtc",
                    "WEBGL_compressed_texture_astc",
                    "WEBGL_compressed_texture_pvrtc",
                    "WEBGL_compressed_texture_etc",
                ];
                const exercise = type => {
                    const canvas = document.createElement("canvas");
                    const context = canvas.getContext(type);
                    if (!context) return { available: false };
                    const supported = context.getSupportedExtensions()
                        .filter(name => extensionNames.includes(name));
                    const extensions = new Map(supported.map(name => [name, context.getExtension(name)]));
                    const stable = supported.every(name => extensions.get(name)
                        === context.getExtension(name.toLowerCase()));

                    let anisotropy = null;
                    const anisotropic = extensions.get("EXT_texture_filter_anisotropic");
                    if (anisotropic) {
                        const texture = context.createTexture();
                        context.bindTexture(context.TEXTURE_2D, texture);
                        const maximum = context.getParameter(anisotropic.MAX_TEXTURE_MAX_ANISOTROPY_EXT);
                        const requested = Math.min(2, maximum);
                        context.texParameterf(context.TEXTURE_2D,
                            anisotropic.TEXTURE_MAX_ANISOTROPY_EXT, requested);
                        anisotropy = {
                            maximum,
                            reflected: context.getTexParameter(context.TEXTURE_2D,
                                anisotropic.TEXTURE_MAX_ANISOTROPY_EXT),
                            error: context.getError(),
                        };
                    }

                    const uploads = [];
                    const candidates = [
                        ["WEBGL_compressed_texture_s3tc", "COMPRESSED_RGB_S3TC_DXT1_EXT", 4, 4, 8],
                        ["WEBGL_compressed_texture_s3tc_srgb", "COMPRESSED_SRGB_S3TC_DXT1_EXT", 4, 4, 8],
                        ["EXT_texture_compression_bptc", "COMPRESSED_RGBA_BPTC_UNORM_EXT", 4, 4, 16],
                        ["EXT_texture_compression_rgtc", "COMPRESSED_RED_RGTC1_EXT", 4, 4, 8],
                        ["WEBGL_compressed_texture_astc", "COMPRESSED_RGBA_ASTC_4x4_KHR", 4, 4, 16],
                        ["WEBGL_compressed_texture_pvrtc", "COMPRESSED_RGB_PVRTC_4BPPV1_IMG", 8, 8, 32],
                        ["WEBGL_compressed_texture_etc", "COMPRESSED_RGB8_ETC2", 4, 4, 8],
                    ];
                    for (const [name, constant, width, height, bytes] of candidates) {
                        const extension = extensions.get(name);
                        if (!extension) continue;
                        const texture = context.createTexture();
                        context.bindTexture(context.TEXTURE_2D, texture);
                        context.compressedTexImage2D(context.TEXTURE_2D, 0,
                            extension[constant], width, height, 0, new Uint8Array(bytes));
                        uploads.push({ name, error: context.getError() });
                    }

                    const formats = [...context.getParameter(context.COMPRESSED_TEXTURE_FORMATS)];
                    const reflected = [...extensions]
                        .filter(([name]) => name !== "EXT_texture_filter_anisotropic")
                        .every(([, extension]) => Object.entries(extension)
                            .filter(([name, value]) => name.startsWith("COMPRESSED_")
                                && typeof value === "number")
                            .every(([, value]) => formats.includes(value)));
                    const astc = extensions.get("WEBGL_compressed_texture_astc");
                    return {
                        available: true,
                        supported,
                        stable,
                        anisotropy,
                        uploads,
                        reflected,
                        astcProfiles: astc?.getSupportedProfiles() ?? null,
                        astcNative: astc?.getSupportedProfiles.toString() ?? null,
                    };
                };
                return JSON.stringify({ webgl: exercise("webgl"), webgl2: exercise("webgl2") });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for kind in ["webgl", "webgl2"] {
        if result[kind]["available"] != true {
            continue;
        }
        assert_eq!(result[kind]["stable"], true);
        assert_eq!(result[kind]["reflected"], true);
        if !result[kind]["anisotropy"].is_null() {
            assert!(result[kind]["anisotropy"]["maximum"].as_f64().unwrap() >= 1.0);
            assert_eq!(
                result[kind]["anisotropy"]["reflected"],
                result[kind]["anisotropy"]["maximum"]
                    .as_f64()
                    .unwrap()
                    .min(2.0)
            );
            assert_eq!(result[kind]["anisotropy"]["error"], 0);
        }
        assert!(
            result[kind]["uploads"]
                .as_array()
                .unwrap()
                .iter()
                .all(|upload| upload["error"] == 0)
        );
        if !result[kind]["astcProfiles"].is_null() {
            assert_eq!(result[kind]["astcProfiles"], serde_json::json!(["ldr"]));
            assert_eq!(
                result[kind]["astcNative"],
                "function getSupportedProfiles() { [native code] }"
            );
        }
    }
}
