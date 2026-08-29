use std::{sync::Arc, time::Duration};

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn webgl_option_uses_webkits_pinned_angle_when_available() {
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
                const context = canvas.getContext("webgl");
                const api = typeof WebGLTexture === "function"
                    && typeof WebGLFramebuffer === "function"
                    && typeof WebGLRenderbuffer === "function"
                    && typeof WebGLRenderingContext.prototype.bufferSubData === "function"
                    && typeof WebGLRenderingContext.prototype.bindAttribLocation === "function"
                    && typeof WebGLRenderingContext.prototype.uniformMatrix4fv === "function"
                    && typeof WebGLRenderingContext.prototype.uniform4iv === "function"
                    && typeof WebGLRenderingContext.prototype.blendFuncSeparate === "function"
                    && typeof WebGLRenderingContext.prototype.stencilOpSeparate === "function"
                    && typeof WebGLRenderingContext.prototype.polygonOffset === "function"
                    && WebGLRenderingContext.prototype.texImage2D.toString() === "function texImage2D() { [native code] }"
                    && WebGLRenderingContext.prototype.texSubImage2D.toString() === "function texSubImage2D() { [native code] }"
                    && WebGLRenderingContext.prototype.checkFramebufferStatus.toString() === "function checkFramebufferStatus() { [native code] }"
                    && WebGLRenderingContext.prototype.bufferSubData.toString() === "function bufferSubData() { [native code] }"
                    && WebGLRenderingContext.prototype.stencilFuncSeparate.toString() === "function stencilFuncSeparate() { [native code] }";
                if (!context) return JSON.stringify({ available: false, api, canvas2d: canvas.getContext("2d") });
                const vertex = context.createShader(context.VERTEX_SHADER);
                context.shaderSource(vertex, "attribute vec2 position; uniform mat2 transform; void main() { gl_Position = vec4(transform * position, 0.0, 1.0); }");
                context.compileShader(vertex);
                const fragment = context.createShader(context.FRAGMENT_SHADER);
                context.shaderSource(fragment, "precision mediump float; uniform vec3 tint; uniform float alpha; uniform ivec2 selector; void main() { gl_FragColor = vec4(tint * float(selector.x), alpha); }");
                context.compileShader(fragment);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.bindAttribLocation(program, 0, "position");
                context.linkProgram(program);
                const compiled = context.getShaderParameter(vertex, context.COMPILE_STATUS)
                    && context.getShaderParameter(fragment, context.COMPILE_STATUS);
                const linked = context.getProgramParameter(program, context.LINK_STATUS);
                context.useProgram(program);
                const buffer = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, buffer);
                context.bufferData(context.ARRAY_BUFFER, 24, context.DYNAMIC_DRAW);
                context.bufferSubData(context.ARRAY_BUFFER, 0, new Float32Array([-1, -1, 3, -1, -1, 3]));
                const position = context.getAttribLocation(program, "position");
                context.enableVertexAttribArray(position);
                context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                context.uniformMatrix2fv(context.getUniformLocation(program, "transform"), false, [1, 0, 0, 1]);
                context.uniform3fv(context.getUniformLocation(program, "tint"), new Float32Array([0, 1, 0]));
                context.uniform1f(context.getUniformLocation(program, "alpha"), 1);
                context.uniform2iv(context.getUniformLocation(program, "selector"), [1, 0]);
                context.viewport(0, 0, 4, 4);
                context.clearColor(1, 0, 0, 1);
                context.clear(context.COLOR_BUFFER_BIT);
                context.drawArrays(context.TRIANGLES, 0, 3);
                context.validateProgram(program);
                const validated = context.getProgramParameter(program, context.VALIDATE_STATUS);
                context.detachShader(program, vertex);
                context.detachShader(program, fragment);
                const indices = context.createBuffer();
                context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, indices);
                context.bufferData(context.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 1, 2]), context.STATIC_DRAW);
                context.drawElements(context.TRIANGLES, 3, context.UNSIGNED_SHORT, 0);
                const pixel = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);

                context.clearColor(1, 0, 0, 1);
                context.clearStencil(0);
                context.clear(context.COLOR_BUFFER_BIT | context.STENCIL_BUFFER_BIT);
                context.enable(context.STENCIL_TEST);
                context.colorMask(false, false, false, false);
                context.stencilMask(0xff);
                context.stencilFunc(context.ALWAYS, 1, 0xff);
                context.stencilOp(context.KEEP, context.KEEP, context.REPLACE);
                context.drawArrays(context.TRIANGLES, 0, 3);
                context.colorMask(true, true, true, true);
                context.stencilMaskSeparate(context.FRONT, 0xff);
                context.stencilMaskSeparate(context.BACK, 0xff);
                context.stencilFuncSeparate(context.FRONT, context.EQUAL, 2, 0xff);
                context.stencilFuncSeparate(context.BACK, context.EQUAL, 2, 0xff);
                context.stencilOpSeparate(context.FRONT, context.KEEP, context.KEEP, context.KEEP);
                context.stencilOpSeparate(context.BACK, context.KEEP, context.KEEP, context.KEEP);
                context.uniform3f(context.getUniformLocation(program, "tint"), 0, 0, 1);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const stencilRejected = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, stencilRejected);
                context.stencilFuncSeparate(context.FRONT_AND_BACK, context.EQUAL, 1, 0xff);
                context.uniform3f(context.getUniformLocation(program, "tint"), 0, 1, 0);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const stencilPassed = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, stencilPassed);
                context.disable(context.STENCIL_TEST);

                context.clearColor(0, 0, 0, 0);
                context.clear(context.COLOR_BUFFER_BIT);
                context.enable(context.BLEND);
                context.blendColor(0.25, 0.5, 0.75, 0.4);
                context.blendFuncSeparate(context.CONSTANT_COLOR, context.ZERO, context.CONSTANT_ALPHA, context.ZERO);
                context.blendEquationSeparate(context.FUNC_ADD, context.FUNC_ADD);
                context.uniform3f(context.getUniformLocation(program, "tint"), 1, 1, 1);
                context.drawArrays(context.TRIANGLES, 0, 3);
                const separatelyBlended = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, separatelyBlended);
                context.disable(context.BLEND);
                context.polygonOffset(1.25, -2);
                context.sampleCoverage(0.5, true);
                const stateError = context.getError();
                const reflectedState = {
                    types: [
                        typeof context.getParameter(context.BLEND),
                        context.getParameter(context.BLEND_COLOR) instanceof Float32Array,
                        context.getParameter(context.VIEWPORT) instanceof Int32Array,
                        Array.isArray(context.getParameter(context.COLOR_WRITEMASK)),
                    ],
                    blend: context.getParameter(context.BLEND),
                    blendColor: [...context.getParameter(context.BLEND_COLOR)].map(value => Math.round(value * 100)),
                    viewport: [...context.getParameter(context.VIEWPORT)],
                    colorMask: context.getParameter(context.COLOR_WRITEMASK),
                    polygonOffset: [context.getParameter(context.POLYGON_OFFSET_FACTOR), context.getParameter(context.POLYGON_OFFSET_UNITS)],
                    sampleCoverage: [context.getParameter(context.SAMPLE_COVERAGE_VALUE), context.getParameter(context.SAMPLE_COVERAGE_INVERT)],
                    stencilReferences: [context.getParameter(context.STENCIL_REF), context.getParameter(context.STENCIL_BACK_REF)],
                };
                const reflectionError = context.getError();

                const texture = context.createTexture();
                context.activeTexture(context.TEXTURE1);
                context.bindTexture(context.TEXTURE_2D, texture);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_S, context.CLAMP_TO_EDGE);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_T, context.CLAMP_TO_EDGE);
                context.pixelStorei(context.UNPACK_ALIGNMENT, 1);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA, 2, 2, 0, context.RGBA, context.UNSIGNED_BYTE,
                    new Uint8Array([10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255]));
                context.texSubImage2D(context.TEXTURE_2D, 0, 1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE,
                    new Uint8Array([90, 80, 70, 255]));

                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0, context.TEXTURE_2D, texture, 0);
                const depth = context.createRenderbuffer();
                context.bindRenderbuffer(context.RENDERBUFFER, depth);
                context.renderbufferStorage(context.RENDERBUFFER, context.DEPTH_COMPONENT16, 2, 2);
                context.framebufferRenderbuffer(context.FRAMEBUFFER, context.DEPTH_ATTACHMENT, context.RENDERBUFFER, depth);
                const framebufferComplete = context.checkFramebufferStatus(context.FRAMEBUFFER) === context.FRAMEBUFFER_COMPLETE;
                const uploaded = new Uint8Array(4);
                context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, uploaded);
                context.pixelStorei(context.UNPACK_FLIP_Y_WEBGL, true);
                context.pixelStorei(context.UNPACK_PREMULTIPLY_ALPHA_WEBGL, true);
                context.texSubImage2D(context.TEXTURE_2D, 0, 0, 0, context.RGBA, context.UNSIGNED_BYTE,
                    new ImageData(new Uint8ClampedArray([200, 100, 50, 128, 10, 20, 30, 255]), 1, 2));
                const imageDataPixels = new Uint8Array(8);
                context.readPixels(0, 0, 1, 2, context.RGBA, context.UNSIGNED_BYTE, imageDataPixels);
                context.pixelStorei(context.UNPACK_FLIP_Y_WEBGL, false);
                context.pixelStorei(context.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
                context.texSubImage2D(context.TEXTURE_2D, 0, 0, 1, context.RGBA, context.UNSIGNED_BYTE,
                    new ImageData(new Float16Array([1, 0.5, 0, 1]), 1, 1,
                        { colorSpace: "display-p3", pixelFormat: "rgba-float16" }));
                const float16P3Pixel = new Uint8Array(4);
                context.readPixels(0, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, float16P3Pixel);
                const sourceCanvas = document.createElement("canvas");
                sourceCanvas.width = sourceCanvas.height = 1;
                sourceCanvas.getContext("2d").fillStyle = "rgba(40, 80, 120, 0.5)";
                sourceCanvas.getContext("2d").fillRect(0, 0, 1, 1);
                context.texSubImage2D(context.TEXTURE_2D, 0, 1, 0, context.RGBA, context.UNSIGNED_BYTE, sourceCanvas);
                const canvasSourcePixel = new Uint8Array(4);
                context.readPixels(1, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, canvasSourcePixel);
                const p3Canvas = document.createElement("canvas");
                p3Canvas.width = p3Canvas.height = 1;
                p3Canvas.getContext("2d", { colorSpace: "display-p3", colorType: "float16" })
                    .putImageData(new ImageData(new Uint8ClampedArray([255, 128, 0, 255]), 1, 1,
                        { colorSpace: "display-p3" }), 0, 0);
                context.texSubImage2D(context.TEXTURE_2D, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, p3Canvas);
                const p3CanvasPixel = new Uint8Array(4);
                context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, p3CanvasPixel);
                const canvasSnapshot = canvas.toDataURL().startsWith("data:image/png;base64,iVBORw0KGgo");
                context.clearColor(0, 0, 1, 1);
                context.clear(context.COLOR_BUFFER_BIT | context.DEPTH_BUFFER_BIT);
                const framebufferPixel = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, framebufferPixel);
                const bindings = context.getParameter(context.ACTIVE_TEXTURE) === context.TEXTURE1
                    && context.getParameter(context.TEXTURE_BINDING_2D) === texture
                    && context.getParameter(context.FRAMEBUFFER_BINDING) === framebuffer
                    && context.getParameter(context.RENDERBUFFER_BINDING) === depth
                    && context.getParameter(context.ARRAY_BUFFER_BINDING) === buffer
                    && context.getParameter(context.ELEMENT_ARRAY_BUFFER_BINDING) === indices
                    && context.getParameter(context.CURRENT_PROGRAM) === program
                    && context.getParameter(context.UNPACK_ALIGNMENT) === 1;
                context.enable(context.SCISSOR_TEST);
                const state = context.isEnabled(context.SCISSOR_TEST);
                context.disable(context.SCISSOR_TEST);
                const textureObjects = texture instanceof WebGLTexture
                    && framebuffer instanceof WebGLFramebuffer
                    && depth instanceof WebGLRenderbuffer
                    && context.isTexture(texture) && context.isFramebuffer(framebuffer) && context.isRenderbuffer(depth);
                context.bindFramebuffer(context.FRAMEBUFFER, null);
                context.deleteRenderbuffer(depth);
                context.deleteFramebuffer(framebuffer);
                context.deleteTexture(texture);
                const deleted = !context.isTexture(texture) && !context.isFramebuffer(framebuffer) && !context.isRenderbuffer(depth);
                const rendererInfo = context.getExtension("WEBGL_debug_renderer_info");
                return JSON.stringify({
                    available: true,
                    api,
                    pixel: [...pixel],
                    stencilRejected: [...stencilRejected],
                    stencilPassed: [...stencilPassed],
                    separatelyBlended: [...separatelyBlended],
                    stateError,
                    reflectedState,
                    reflectionError,
                    uploaded: [...uploaded],
                    imageDataPixels: [...imageDataPixels],
                    float16P3Pixel: [...float16P3Pixel],
                    canvasSourcePixel: [...canvasSourcePixel],
                    p3CanvasPixel: [...p3CanvasPixel],
                    framebufferPixel: [...framebufferPixel],
                    framebufferComplete,
                    canvasSnapshot,
                    bindings,
                    state,
                    textureObjects,
                    deleted,
                    compiled,
                    linked,
                    validated,
                    boundAttribute: position === 0,
                    objects: vertex instanceof WebGLShader && program instanceof WebGLProgram && buffer instanceof WebGLBuffer,
                    renderer: context.getParameter(context.RENDERER).length > 0,
                    identity: [
                        context.getParameter(context.VENDOR),
                        context.getParameter(context.RENDERER),
                        context.getParameter(rendererInfo.UNMASKED_VENDOR_WEBGL),
                        context.getParameter(rendererInfo.UNMASKED_RENDERER_WEBGL),
                    ],
                    png: canvas.toDataURL().startsWith("data:image/png;base64,iVBORw0KGgo"),
                    canvas2d: canvas.getContext("2d"),
                    native: [context.getParameter.toString(), context.texImage2D.toString(), WebGLTexture.toString()],
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    if result["available"] == true {
        assert_eq!(result["canvas2d"], serde_json::Value::Null);
        assert_eq!(result["pixel"], serde_json::json!([0, 255, 0, 255]));
        assert_eq!(
            result["stencilRejected"],
            serde_json::json!([255, 0, 0, 255])
        );
        assert_eq!(result["stencilPassed"], serde_json::json!([0, 255, 0, 255]));
        assert_eq!(
            result["separatelyBlended"],
            serde_json::json!([64, 128, 191, 102])
        );
        assert_eq!(result["stateError"], 0);
        assert_eq!(
            result["reflectedState"],
            serde_json::json!({
                "types": ["boolean", true, true, true],
                "blend": false,
                "blendColor": [25, 50, 75, 40],
                "viewport": [0, 0, 4, 4],
                "colorMask": [true, true, true, true],
                "polygonOffset": [1.25, -2],
                "sampleCoverage": [0.5, true],
                "stencilReferences": [1, 1],
            })
        );
        assert_eq!(result["reflectionError"], 0);
        assert_eq!(result["uploaded"], serde_json::json!([90, 80, 70, 255]));
        assert_eq!(
            result["imageDataPixels"],
            serde_json::json!([10, 20, 30, 255, 100, 50, 25, 128])
        );
        assert_eq!(result["float16P3Pixel"][0], 255);
        assert!(result["float16P3Pixel"][1].as_u64().unwrap() > 110);
        assert!(result["float16P3Pixel"][1].as_u64().unwrap() < 126);
        assert_eq!(result["float16P3Pixel"][2], 0);
        assert_eq!(result["float16P3Pixel"][3], 255);
        assert_eq!(
            result["canvasSourcePixel"],
            serde_json::json!([40, 80, 120, 128])
        );
        assert_eq!(result["p3CanvasPixel"][0], 255);
        assert!(result["p3CanvasPixel"][1].as_u64().unwrap() > 110);
        assert!(result["p3CanvasPixel"][1].as_u64().unwrap() < 126);
        assert_eq!(result["p3CanvasPixel"][2], 0);
        assert_eq!(result["p3CanvasPixel"][3], 255);
        assert_eq!(
            result["framebufferPixel"],
            serde_json::json!([0, 0, 255, 255])
        );
        assert_eq!(result["framebufferComplete"], true);
        assert_eq!(result["canvasSnapshot"], true);
        assert_eq!(result["bindings"], true);
        assert_eq!(result["state"], true);
        assert_eq!(result["textureObjects"], true);
        assert_eq!(result["deleted"], true);
        assert_eq!(result["compiled"], true);
        assert_eq!(result["linked"], true);
        assert_eq!(result["validated"], true);
        assert_eq!(result["boundAttribute"], true);
        assert_eq!(result["objects"], true);
        assert_eq!(result["renderer"], true);
        assert_eq!(
            result["identity"],
            serde_json::json!([
                "WebKit",
                "WebKit WebGL",
                "Intel Inc.",
                "Intel(R) Iris(TM) Plus Graphics OpenGL Engine",
            ])
        );
        assert_eq!(result["png"], true);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function getParameter() { [native code] }",
                "function texImage2D() { [native code] }",
                "function WebGLTexture() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl1_extensions_follow_angle_support_and_return_stable_usable_objects() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                globalThis.parallelShaderResult = null;
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl");
                if (!context) return "no-angle";
                const supported = context.getSupportedExtensions();
                const allowed = new Set([
                    "WEBGL_debug_renderer_info",
                    "WEBGL_lose_context",
                    "OES_vertex_array_object",
                    "OES_element_index_uint",
                    "OES_standard_derivatives",
                    "EXT_frag_depth",
                    "EXT_shader_texture_lod",
                    "EXT_blend_minmax",
                    "WEBGL_debug_shaders",
                    "EXT_sRGB",
                    "WEBGL_compressed_texture_etc1",
                    "OES_fbo_render_mipmap",
                    "WEBGL_blend_func_extended",
                    "WEBGL_polygon_mode",
                    "KHR_parallel_shader_compile",
                    "EXT_clip_control",
                    "EXT_polygon_offset_clamp",
                    "EXT_depth_clamp",
                    "EXT_texture_mirror_clamp_to_edge",
                    "ANGLE_instanced_arrays",
                    "WEBGL_draw_buffers",
                    "OES_texture_float",
                    "OES_texture_half_float",
                    "OES_texture_float_linear",
                    "OES_texture_half_float_linear",
                    "WEBGL_color_buffer_float",
                    "EXT_color_buffer_half_float",
                    "EXT_float_blend",
                    "WEBGL_depth_texture",
                    "EXT_texture_filter_anisotropic",
                    "WEBGL_compressed_texture_s3tc",
                    "WEBGL_compressed_texture_s3tc_srgb",
                    "EXT_texture_compression_bptc",
                    "EXT_texture_compression_rgtc",
                    "WEBGL_compressed_texture_astc",
                    "WEBGL_compressed_texture_pvrtc",
                    "WEBGL_compressed_texture_etc",
                    "EXT_disjoint_timer_query",
                    "WEBGL_multi_draw",
                ]);
                const compile = source => {
                    const shader = context.createShader(context.FRAGMENT_SHADER);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return context.getShaderParameter(shader, context.COMPILE_STATUS);
                };
                const exerciseClipControl = target => {
                    const extension = target.getExtension("EXT_clip_control");
                    if (!extension) return null;
                    const defaults = [
                        target.getParameter(extension.CLIP_ORIGIN_EXT),
                        target.getParameter(extension.CLIP_DEPTH_MODE_EXT),
                    ];
                    extension.clipControlEXT(extension.UPPER_LEFT_EXT, extension.ZERO_TO_ONE_EXT);
                    const changed = [
                        target.getParameter(extension.CLIP_ORIGIN_EXT),
                        target.getParameter(extension.CLIP_DEPTH_MODE_EXT),
                    ];
                    extension.clipControlEXT(0, extension.ZERO_TO_ONE_EXT);
                    const invalid = target.getError();
                    extension.clipControlEXT(
                        extension.LOWER_LEFT_EXT,
                        extension.NEGATIVE_ONE_TO_ONE_EXT,
                    );
                    return {
                        constants: [
                            extension.LOWER_LEFT_EXT,
                            extension.UPPER_LEFT_EXT,
                            extension.NEGATIVE_ONE_TO_ONE_EXT,
                            extension.ZERO_TO_ONE_EXT,
                            extension.CLIP_ORIGIN_EXT,
                            extension.CLIP_DEPTH_MODE_EXT,
                        ],
                        defaults,
                        changed,
                        invalid,
                        reset: [
                            target.getParameter(extension.CLIP_ORIGIN_EXT),
                            target.getParameter(extension.CLIP_DEPTH_MODE_EXT),
                        ],
                        stable: extension === target.getExtension("ext_clip_control"),
                        native: extension.clipControlEXT.toString(),
                    };
                };
                const clipControl = exerciseClipControl(context);
                const exerciseRasterStateExtensions = target => {
                    const polygon = target.getExtension("EXT_polygon_offset_clamp");
                    const depth = target.getExtension("EXT_depth_clamp");
                    const mirror = target.getExtension("EXT_texture_mirror_clamp_to_edge");
                    let polygonResult = null;
                    if (polygon) {
                        const initial = target.getParameter(polygon.POLYGON_OFFSET_CLAMP_EXT);
                        polygon.polygonOffsetClampEXT(1.25, 2.5, 0.75);
                        const changed = target.getParameter(polygon.POLYGON_OFFSET_CLAMP_EXT);
                        polygon.polygonOffsetClampEXT(0, 0, 0);
                        polygonResult = {
                            constant: polygon.POLYGON_OFFSET_CLAMP_EXT,
                            initial,
                            changed,
                            reset: target.getParameter(polygon.POLYGON_OFFSET_CLAMP_EXT),
                            stable: polygon === target.getExtension("ext_polygon_offset_clamp"),
                            native: polygon.polygonOffsetClampEXT.toString(),
                        };
                    }
                    let depthResult = null;
                    if (depth) {
                        const initial = {
                            enabled: target.isEnabled(depth.DEPTH_CLAMP_EXT),
                            parameter: target.getParameter(depth.DEPTH_CLAMP_EXT),
                        };
                        target.enable(depth.DEPTH_CLAMP_EXT);
                        const changed = {
                            enabled: target.isEnabled(depth.DEPTH_CLAMP_EXT),
                            parameter: target.getParameter(depth.DEPTH_CLAMP_EXT),
                        };
                        target.disable(depth.DEPTH_CLAMP_EXT);
                        depthResult = {
                            constant: depth.DEPTH_CLAMP_EXT,
                            initial,
                            changed,
                            reset: target.getParameter(depth.DEPTH_CLAMP_EXT),
                            stable: depth === target.getExtension("ext_depth_clamp"),
                        };
                    }
                    let mirrorResult = null;
                    if (mirror) {
                        const texture = target.createTexture();
                        target.bindTexture(target.TEXTURE_2D, texture);
                        target.texParameteri(
                            target.TEXTURE_2D,
                            target.TEXTURE_WRAP_S,
                            mirror.MIRROR_CLAMP_TO_EDGE_EXT,
                        );
                        let sampler = null;
                        if (target instanceof WebGL2RenderingContext) {
                            sampler = target.createSampler();
                            target.samplerParameteri(
                                sampler,
                                target.TEXTURE_WRAP_T,
                                mirror.MIRROR_CLAMP_TO_EDGE_EXT,
                            );
                        }
                        mirrorResult = {
                            constant: mirror.MIRROR_CLAMP_TO_EDGE_EXT,
                            texture: target.getTexParameter(
                                target.TEXTURE_2D,
                                target.TEXTURE_WRAP_S,
                            ),
                            sampler: sampler === null ? null
                                : target.getSamplerParameter(sampler, target.TEXTURE_WRAP_T),
                            stable: mirror
                                === target.getExtension("ext_texture_mirror_clamp_to_edge"),
                            error: target.getError(),
                        };
                    }
                    return { polygon: polygonResult, depth: depthResult, mirror: mirrorResult };
                };
                const rasterStateExtensions = exerciseRasterStateExtensions(context);
                const shaderResults = {};
                if (supported.includes("OES_standard_derivatives")) {
                    const extension = context.getExtension("OES_standard_derivatives");
                    context.hint(extension.FRAGMENT_SHADER_DERIVATIVE_HINT_OES, context.NICEST);
                    shaderResults.derivatives = compile(`#extension GL_OES_standard_derivatives : enable
                        precision mediump float;
                        void main() { gl_FragColor = vec4(dFdx(gl_FragCoord.x)); }`);
                }
                if (supported.includes("EXT_frag_depth")) {
                    shaderResults.fragDepth = compile(`#extension GL_EXT_frag_depth : enable
                        precision mediump float;
                        void main() { gl_FragDepthEXT = 0.5; gl_FragColor = vec4(1.0); }`);
                }
                if (supported.includes("EXT_shader_texture_lod")) {
                    shaderResults.textureLod = compile(`#extension GL_EXT_shader_texture_lod : enable
                        precision mediump float;
                        uniform sampler2D sourceTexture;
                        void main() { gl_FragColor = texture2DLodEXT(sourceTexture, vec2(0.0), 0.0); }`);
                }
                if (supported.includes("EXT_blend_minmax")) {
                    context.blendEquation(context.getExtension("EXT_blend_minmax").MAX_EXT);
                }
                let parallel = null;
                if (supported.includes("KHR_parallel_shader_compile")) {
                    const extension = context.getExtension("khr_parallel_shader_compile");
                    const vertex = context.createShader(context.VERTEX_SHADER);
                    context.shaderSource(vertex, "attribute vec2 position; void main() { gl_Position = vec4(position, 0.0, 1.0); }");
                    context.compileShader(vertex);
                    const fragment = context.createShader(context.FRAGMENT_SHADER);
                    context.shaderSource(fragment, "precision mediump float; void main() { gl_FragColor = vec4(1.0); }");
                    context.compileShader(fragment);
                    const program = context.createProgram();
                    context.attachShader(program, vertex);
                    context.attachShader(program, fragment);
                    context.linkProgram(program);
                    parallel = {
                        constant: extension.COMPLETION_STATUS_KHR === 0x91B1,
                        completion: false,
                        linked: false,
                        stable: extension === context.getExtension("KHR_parallel_shader_compile"),
                    };
                    parallelShaderResult = parallel;
                    context.flush();
                    const pollCompletion = () => {
                        parallel.completion = context.getProgramParameter(program, extension.COMPLETION_STATUS_KHR);
                        if (parallel.completion) {
                            parallel.linked = context.getProgramParameter(program, context.LINK_STATUS);
                        } else {
                            setTimeout(pollCompletion, 1);
                        }
                    };
                    pollCompletion();
                }
                let vao = null;
                let vaoBound = true;
                let vaoDeleted = true;
                let vaoNative = null;
                let uintPixel = null;
                if (supported.includes("OES_vertex_array_object")) {
                    const extension = context.getExtension("oes_vertex_array_object");
                    vao = extension.createVertexArrayOES();
                    extension.bindVertexArrayOES(vao);
                    vaoBound = extension.isVertexArrayOES(vao);
                    if (supported.includes("OES_element_index_uint")) {
                        context.getExtension("OES_element_index_uint");
                        const vertex = context.createShader(context.VERTEX_SHADER);
                        context.shaderSource(vertex, `attribute vec2 position;
                            void main() { gl_Position = vec4(position, 0.0, 1.0); }`);
                        context.compileShader(vertex);
                        const fragment = context.createShader(context.FRAGMENT_SHADER);
                        context.shaderSource(fragment, `precision mediump float;
                            void main() { gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }`);
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
                        const indices = context.createBuffer();
                        context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, indices);
                        context.bufferData(
                            context.ELEMENT_ARRAY_BUFFER,
                            new Uint32Array([0, 1, 2]),
                            context.STATIC_DRAW,
                        );
                        context.clearColor(0, 0, 0, 1);
                        context.clear(context.COLOR_BUFFER_BIT);
                        context.drawElements(context.TRIANGLES, 3, context.UNSIGNED_INT, 0);
                        const pixel = new Uint8Array(4);
                        context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);
                        uintPixel = [...pixel];
                    }
                    extension.deleteVertexArrayOES(vao);
                    vaoDeleted = !extension.isVertexArrayOES(vao);
                    vaoNative = extension.createVertexArrayOES.toString();
                }
                const webgl2Canvas = document.createElement("canvas");
                const webgl2 = webgl2Canvas.getContext("webgl2");
                const webgl2ClipControl = webgl2 ? exerciseClipControl(webgl2) : null;
                const webgl2RasterStateExtensions = webgl2
                    ? exerciseRasterStateExtensions(webgl2)
                    : null;
                return JSON.stringify({
                    supported,
                    allowed: supported.every(extension => allowed.has(extension)),
                    stable: supported.every(extension =>
                        context.getExtension(extension) === context.getExtension(extension.toLowerCase())),
                    debugStable: context.getExtension("WEBGL_debug_renderer_info")
                        === context.getExtension("webgl_debug_renderer_info"),
                    unknown: context.getExtension("NOT_AN_EXTENSION") === null,
                    shaderResults,
                    parallel,
                    clipControl,
                    rasterStateExtensions,
                    vao: vao === null || vao instanceof WebGLVertexArrayObject,
                    vaoBound,
                    vaoDeleted,
                    vaoNative,
                    uintPixel,
                    error: context.getError(),
                    webgl2Extensions: webgl2?.getSupportedExtensions() ?? null,
                    webgl2ClipControl,
                    webgl2RasterStateExtensions,
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    if result != "no-angle" {
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(
            result["supported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|extension| extension == "OES_vertex_array_object")
        );
        assert!(
            result["supported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|extension| extension == "OES_element_index_uint")
        );
        assert_eq!(result["allowed"], true);
        assert_eq!(result["stable"], true);
        assert_eq!(result["debugStable"], true);
        assert_eq!(result["unknown"], true);
        for clip_control in [&result["clipControl"], &result["webgl2ClipControl"]] {
            if clip_control.is_null() {
                continue;
            }
            assert_eq!(
                clip_control["constants"],
                serde_json::json!([0x8CA1, 0x8CA2, 0x935E, 0x935F, 0x935C, 0x935D])
            );
            assert_eq!(
                clip_control["defaults"],
                serde_json::json!([0x8CA1, 0x935E])
            );
            assert_eq!(clip_control["changed"], serde_json::json!([0x8CA2, 0x935F]));
            assert_eq!(clip_control["invalid"], 0x0500);
            assert_eq!(clip_control["reset"], serde_json::json!([0x8CA1, 0x935E]));
            assert_eq!(clip_control["stable"], true);
            assert_eq!(
                clip_control["native"],
                "function clipControlEXT() { [native code] }"
            );
        }
        for extensions in [
            &result["rasterStateExtensions"],
            &result["webgl2RasterStateExtensions"],
        ] {
            if extensions.is_null() {
                continue;
            }
            if !extensions["polygon"].is_null() {
                assert_eq!(extensions["polygon"]["constant"], 0x8E1B);
                assert_eq!(extensions["polygon"]["initial"], 0.0);
                assert_eq!(extensions["polygon"]["changed"], 0.75);
                assert_eq!(extensions["polygon"]["reset"], 0.0);
                assert_eq!(extensions["polygon"]["stable"], true);
                assert_eq!(
                    extensions["polygon"]["native"],
                    "function polygonOffsetClampEXT() { [native code] }"
                );
            }
            if !extensions["depth"].is_null() {
                assert_eq!(extensions["depth"]["constant"], 0x864F);
                assert_eq!(
                    extensions["depth"]["initial"],
                    serde_json::json!({"enabled": false, "parameter": false})
                );
                assert_eq!(
                    extensions["depth"]["changed"],
                    serde_json::json!({"enabled": true, "parameter": true})
                );
                assert_eq!(extensions["depth"]["reset"], false);
                assert_eq!(extensions["depth"]["stable"], true);
            }
            if !extensions["mirror"].is_null() {
                assert_eq!(extensions["mirror"]["constant"], 0x8743);
                assert_eq!(extensions["mirror"]["texture"], 0x8743);
                if !extensions["mirror"]["sampler"].is_null() {
                    assert_eq!(extensions["mirror"]["sampler"], 0x8743);
                }
                assert_eq!(extensions["mirror"]["stable"], true);
                assert_eq!(extensions["mirror"]["error"], 0);
            }
        }
        if !result["parallel"].is_null() {
            assert_eq!(result["parallel"]["constant"], true);
            assert_eq!(result["parallel"]["stable"], true);
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            loop {
                let complete = page
                    .eval("parallelShaderResult.completion")
                    .unwrap()
                    .to_string()
                    .unwrap()
                    == "true";
                if complete || std::time::Instant::now() >= deadline {
                    break;
                }
                let _ = page.run_one_task().unwrap();
                std::thread::sleep(Duration::from_millis(1));
            }
            let parallel = page
                .eval("JSON.stringify(parallelShaderResult)")
                .unwrap()
                .to_string()
                .unwrap();
            let parallel: serde_json::Value = serde_json::from_str(&parallel).unwrap();
            assert_eq!(parallel["completion"], true);
            assert_eq!(parallel["linked"], true);
        }
        assert!(
            result["shaderResults"]
                .as_object()
                .unwrap()
                .values()
                .all(|value| value == true)
        );
        assert_eq!(result["vao"], true);
        assert_eq!(result["vaoBound"], true);
        assert_eq!(result["vaoDeleted"], true);
        if !result["vaoNative"].is_null() {
            assert_eq!(
                result["vaoNative"],
                "function createVertexArrayOES() { [native code] }"
            );
        }
        assert_eq!(result["uintPixel"], serde_json::json!([255, 0, 0, 255]));
        assert_eq!(result["error"], 0);
        if !result["webgl2Extensions"].is_null() {
            let extensions = result["webgl2Extensions"].as_array().unwrap();
            assert!(
                extensions
                    .iter()
                    .any(|extension| extension == "WEBGL_debug_renderer_info")
            );
            assert!(
                extensions
                    .iter()
                    .any(|extension| extension == "WEBGL_lose_context")
            );
            assert!(
                extensions.iter().all(|extension| matches!(
                    extension.as_str(),
                    Some(
                        "WEBGL_debug_renderer_info"
                            | "WEBGL_lose_context"
                            | "WEBGL_debug_shaders"
                            | "WEBGL_compressed_texture_etc1"
                            | "WEBGL_blend_func_extended"
                            | "WEBGL_polygon_mode"
                            | "KHR_parallel_shader_compile"
                            | "EXT_clip_control"
                            | "EXT_polygon_offset_clamp"
                            | "EXT_depth_clamp"
                            | "EXT_texture_mirror_clamp_to_edge"
                            | "EXT_texture_norm16"
                            | "EXT_render_snorm"
                            | "EXT_conservative_depth"
                            | "NV_shader_noperspective_interpolation"
                            | "OES_sample_variables"
                            | "OES_shader_multisample_interpolation"
                            | "WEBGL_clip_cull_distance"
                            | "WEBGL_provoking_vertex"
                            | "WEBGL_stencil_texturing"
                            | "WEBGL_render_shared_exponent"
                            | "WEBGL_compressed_texture_etc"
                            | "OES_texture_float_linear"
                            | "OES_texture_half_float_linear"
                            | "EXT_color_buffer_float"
                            | "EXT_color_buffer_half_float"
                            | "EXT_float_blend"
                            | "EXT_texture_filter_anisotropic"
                            | "WEBGL_compressed_texture_s3tc"
                            | "WEBGL_compressed_texture_s3tc_srgb"
                            | "EXT_texture_compression_bptc"
                            | "EXT_texture_compression_rgtc"
                            | "WEBGL_compressed_texture_astc"
                            | "WEBGL_compressed_texture_pvrtc"
                            | "EXT_disjoint_timer_query_webgl2"
                            | "OES_draw_buffers_indexed"
                            | "WEBGL_multi_draw"
                    )
                )),
                "unexpected WebGL 2 extension set: {extensions:?}",
            );
        }
    }
}

#[test]
fn webgl_debug_shaders_returns_angles_translated_source_when_supported() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r##"JSON.stringify(["webgl", "webgl2"].map(type => {
                const canvas = document.createElement("canvas");
                const context = canvas.getContext(type);
                if (context === null) return { available: false };
                const listed = context.getSupportedExtensions()
                    .includes("WEBGL_debug_shaders");
                const extension = context.getExtension("WEBGL_debug_shaders");
                if (extension === null) return { available: true, listed, supported: false };

                const shader = context.createShader(context.VERTEX_SHADER);
                const emptyBeforeCompilation = extension.getTranslatedShaderSource(shader) === "";
                const source = type === "webgl2"
                    ? "#version 300 es\nin vec2 position; void main() { gl_Position = vec4(position, 0.0, 1.0); }"
                    : "attribute vec2 position; void main() { gl_Position = vec4(position, 0.0, 1.0); }";
                context.shaderSource(shader, source);
                context.compileShader(shader);
                const compiled = context.getShaderParameter(shader, context.COMPILE_STATUS);
                const translated = extension.getTranslatedShaderSource(shader);

                const invalid = context.createShader(context.FRAGMENT_SHADER);
                context.shaderSource(invalid, "this is not valid GLSL");
                context.compileShader(invalid);
                const emptyAfterFailure = extension.getTranslatedShaderSource(invalid) === "";

                const foreignContext = document.createElement("canvas").getContext(type);
                const foreignShader = foreignContext.createShader(foreignContext.VERTEX_SHADER);
                let foreignRejected = false;
                try { extension.getTranslatedShaderSource(foreignShader); }
                catch (error) { foreignRejected = error instanceof TypeError; }

                const stable = extension === context.getExtension("webgl_debug_shaders");
                const native = extension.getTranslatedShaderSource.toString();
                const loss = context.getExtension("WEBGL_lose_context");
                loss.loseContext();
                const emptyAfterLoss = extension.getTranslatedShaderSource(shader) === "";
                return {
                    available: true,
                    listed,
                    supported: true,
                    emptyBeforeCompilation,
                    compiled,
                    translated: typeof translated === "string" && translated.length > 0,
                    emptyAfterFailure,
                    foreignRejected,
                    stable,
                    native,
                    emptyAfterLoss,
                };
            }))"##,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for context in result.as_array().unwrap() {
        if context["available"] != true || context["supported"] != true {
            assert_ne!(context["listed"], true);
            continue;
        }
        assert_eq!(context["listed"], true);
        assert_eq!(context["emptyBeforeCompilation"], true);
        assert_eq!(context["compiled"], true);
        assert_eq!(context["translated"], true);
        assert_eq!(context["emptyAfterFailure"], true);
        assert_eq!(context["foreignRejected"], true);
        assert_eq!(context["stable"], true);
        assert_eq!(
            context["native"],
            "function getTranslatedShaderSource() { [native code] }"
        );
        assert_eq!(context["emptyAfterLoss"], true);
    }
}

#[test]
fn webgl_requestable_legacy_texture_extensions_execute_through_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"JSON.stringify((() => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 4;
                const context = canvas.getContext("webgl");
                if (context === null) return { available: false };
                const webgl2 = document.createElement("canvas").getContext("webgl2");

                const srgb = context.getExtension("EXT_sRGB");
                let srgbResult = null;
                if (srgb !== null) {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(
                        context.TEXTURE_2D, 0, srgb.SRGB_ALPHA_EXT, 1, 1, 0,
                        srgb.SRGB_ALPHA_EXT, context.UNSIGNED_BYTE,
                        new Uint8Array([128, 96, 64, 255]),
                    );
                    const textureError = context.getError();
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.framebufferTexture2D(
                        context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, texture, 0,
                    );
                    const textureStatus = context.checkFramebufferStatus(context.FRAMEBUFFER);
                    const textureEncoding = context.getFramebufferAttachmentParameter(
                        context.FRAMEBUFFER,
                        context.COLOR_ATTACHMENT0,
                        srgb.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT,
                    );

                    const renderbuffer = context.createRenderbuffer();
                    context.bindRenderbuffer(context.RENDERBUFFER, renderbuffer);
                    context.renderbufferStorage(
                        context.RENDERBUFFER, srgb.SRGB8_ALPHA8_EXT, 2, 2,
                    );
                    const renderbufferError = context.getError();
                    context.framebufferRenderbuffer(
                        context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.RENDERBUFFER, renderbuffer,
                    );
                    const renderbufferStatus = context.checkFramebufferStatus(context.FRAMEBUFFER);
                    const renderbufferEncoding = context.getFramebufferAttachmentParameter(
                        context.FRAMEBUFFER,
                        context.COLOR_ATTACHMENT0,
                        srgb.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT,
                    );
                    srgbResult = {
                        constants: [
                            srgb.SRGB_EXT,
                            srgb.SRGB_ALPHA_EXT,
                            srgb.SRGB8_ALPHA8_EXT,
                            srgb.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT,
                        ],
                        textureError,
                        textureStatus,
                        textureEncoding,
                        renderbufferError,
                        renderbufferStatus,
                        renderbufferEncoding,
                        stable: srgb === context.getExtension("ext_srgb"),
                    };
                }

                const mipmap = context.getExtension("OES_fbo_render_mipmap");
                let mipmapResult = null;
                if (mipmap !== null) {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(
                        context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST,
                    );
                    context.texImage2D(
                        context.TEXTURE_2D, 0, context.RGBA, 4, 4, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null,
                    );
                    context.texImage2D(
                        context.TEXTURE_2D, 1, context.RGBA, 2, 2, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null,
                    );
                    context.texImage2D(
                        context.TEXTURE_2D, 2, context.RGBA, 1, 1, 0,
                        context.RGBA, context.UNSIGNED_BYTE, null,
                    );
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.framebufferTexture2D(
                        context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, texture, 1,
                    );
                    const attachmentError = context.getError();
                    const status = context.checkFramebufferStatus(context.FRAMEBUFFER);
                    context.viewport(0, 0, 2, 2);
                    context.clearColor(1, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    const pixel = new Uint8Array(4);
                    context.readPixels(
                        0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel,
                    );
                    mipmapResult = {
                        attachmentError,
                        status,
                        pixel: [...pixel],
                        stable: mipmap === context.getExtension("oes_fbo_render_mipmap"),
                    };
                }

                const exerciseEtc1 = target => {
                    if (target === null) return null;
                    const extension = target.getExtension("WEBGL_compressed_texture_etc1");
                    if (extension === null) return { supported: false };
                    const texture = target.createTexture();
                    target.bindTexture(target.TEXTURE_2D, texture);
                    target.compressedTexImage2D(
                        target.TEXTURE_2D, 0, extension.COMPRESSED_RGB_ETC1_WEBGL,
                        4, 4, 0, new Uint8Array(8),
                    );
                    const allocationError = target.getError();
                    const reflected = target.getParameter(target.COMPRESSED_TEXTURE_FORMATS)
                        .includes(extension.COMPRESSED_RGB_ETC1_WEBGL);
                    target.compressedTexImage2D(
                        target.TEXTURE_2D, 0, extension.COMPRESSED_RGB_ETC1_WEBGL,
                        4, 4, 0, new Uint8Array(7),
                    );
                    const invalidLengthError = target.getError();
                    target.compressedTexSubImage2D(
                        target.TEXTURE_2D, 0, 0, 0, 4, 4,
                        extension.COMPRESSED_RGB_ETC1_WEBGL, new Uint8Array(8),
                    );
                    const subImageError = target.getError();
                    let pboSubImageError = null;
                    if (target instanceof WebGL2RenderingContext) {
                        const pbo = target.createBuffer();
                        target.bindBuffer(target.PIXEL_UNPACK_BUFFER, pbo);
                        target.bufferData(
                            target.PIXEL_UNPACK_BUFFER, new Uint8Array(8), target.STATIC_DRAW,
                        );
                        target.compressedTexSubImage2D(
                            target.TEXTURE_2D, 0, 0, 0, 4, 4,
                            extension.COMPRESSED_RGB_ETC1_WEBGL, 8, 0,
                        );
                        pboSubImageError = target.getError();
                        target.bindBuffer(target.PIXEL_UNPACK_BUFFER, null);
                    }
                    return {
                        supported: true,
                        constant: extension.COMPRESSED_RGB_ETC1_WEBGL,
                        allocationError,
                        reflected,
                        invalidLengthError,
                        subImageError,
                        pboSubImageError,
                        stable: extension
                            === target.getExtension("webgl_compressed_texture_etc1"),
                    };
                };

                return {
                    available: true,
                    srgb: srgbResult,
                    mipmap: mipmapResult,
                    etc1: [exerciseEtc1(context), exerciseEtc1(webgl2)],
                    promotedAbsent: webgl2 === null || (
                        webgl2.getExtension("EXT_sRGB") === null
                        && webgl2.getExtension("OES_fbo_render_mipmap") === null
                    ),
                    error: context.getError(),
                };
            })())"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["available"] != true {
        return;
    }
    if !result["srgb"].is_null() {
        assert_eq!(
            result["srgb"]["constants"],
            serde_json::json!([0x8c40, 0x8c42, 0x8c43, 0x8210])
        );
        assert_eq!(result["srgb"]["textureError"], 0);
        assert_eq!(result["srgb"]["textureStatus"], 0x8cd5);
        assert_eq!(result["srgb"]["textureEncoding"], 0x8c40);
        assert_eq!(result["srgb"]["renderbufferError"], 0);
        assert_eq!(result["srgb"]["renderbufferStatus"], 0x8cd5);
        assert_eq!(result["srgb"]["renderbufferEncoding"], 0x8c40);
        assert_eq!(result["srgb"]["stable"], true);
    }
    if !result["mipmap"].is_null() {
        assert_eq!(result["mipmap"]["attachmentError"], 0);
        assert_eq!(result["mipmap"]["status"], 0x8cd5);
        assert_eq!(
            result["mipmap"]["pixel"],
            serde_json::json!([255, 0, 0, 255])
        );
        assert_eq!(result["mipmap"]["stable"], true);
    }
    for etc1 in result["etc1"].as_array().unwrap() {
        if etc1.is_null() || etc1["supported"] != true {
            continue;
        }
        assert_eq!(etc1["constant"], 0x8d64);
        assert_eq!(etc1["allocationError"], 0);
        assert_eq!(etc1["reflected"], true);
        assert_ne!(etc1["invalidLengthError"], 0);
        assert_eq!(etc1["subImageError"], 0x0502);
        if !etc1["pboSubImageError"].is_null() {
            assert_eq!(etc1["pboSubImageError"], 0x0502);
        }
        assert_eq!(etc1["stable"], true);
    }
    assert_eq!(result["promotedAbsent"], true);
    assert_eq!(result["error"], 0);
}

#[test]
fn webgl_core_constants_formats_and_object_bindings_use_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const context = document.createElement("canvas").getContext("webgl2");
                const requiredBaseConstants = `
                    SUBPIXEL_BITS RED_BITS GREEN_BITS BLUE_BITS ALPHA_BITS DEPTH_BITS STENCIL_BITS
                    SAMPLE_BUFFERS GENERATE_MIPMAP_HINT MAX_VERTEX_UNIFORM_VECTORS
                    MAX_VARYING_VECTORS MAX_COMBINED_TEXTURE_IMAGE_UNITS
                    MAX_VERTEX_TEXTURE_IMAGE_UNITS MAX_FRAGMENT_UNIFORM_VECTORS
                    TEXTURE_BINDING_CUBE_MAP MAX_CUBE_MAP_TEXTURE_SIZE
                    IMPLEMENTATION_COLOR_READ_TYPE IMPLEMENTATION_COLOR_READ_FORMAT
                    RGBA4 RGB5_A1 RGB565
                `.trim().split(/\s+/);
                const requiredWebGL2Constants = `
                    RGB8 RGB10_A2 MAX_3D_TEXTURE_SIZE MAX_ELEMENTS_VERTICES MAX_ELEMENTS_INDICES
                    MIN MAX MAX_TEXTURE_LOD_BIAS MAX_FRAGMENT_UNIFORM_COMPONENTS
                    MAX_VERTEX_UNIFORM_COMPONENTS SAMPLER_3D SAMPLER_2D_SHADOW
                    FRAGMENT_SHADER_DERIVATIVE_HINT SRGB SRGB8 SRGB8_ALPHA8
                    MAX_ARRAY_TEXTURE_LAYERS MIN_PROGRAM_TEXEL_OFFSET MAX_PROGRAM_TEXEL_OFFSET
                    MAX_VARYING_COMPONENTS R11F_G11F_B10F RGB9_E5
                    MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS
                    MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS
                    MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS
                    RGBA32UI RGB32UI RGBA16UI RGB16UI RGB8UI
                    RGBA32I RGB32I RGBA16I RGB16I RGB8I
                    SAMPLER_2D_ARRAY SAMPLER_2D_ARRAY_SHADOW SAMPLER_CUBE_SHADOW
                    INT_SAMPLER_2D INT_SAMPLER_3D INT_SAMPLER_CUBE INT_SAMPLER_2D_ARRAY
                    UNSIGNED_INT_SAMPLER_2D UNSIGNED_INT_SAMPLER_3D
                    UNSIGNED_INT_SAMPLER_CUBE UNSIGNED_INT_SAMPLER_2D_ARRAY
                    FRAMEBUFFER_DEFAULT UNSIGNED_NORMALIZED FRAMEBUFFER_INCOMPLETE_MULTISAMPLE
                    R8 RG8 R8I R8UI R16I R16UI R32I R32UI RG8I RG8UI RG16I RG16UI RG32I RG32UI
                    VERTEX_ARRAY_BINDING R8_SNORM RG8_SNORM RGB8_SNORM RGBA8_SNORM SIGNED_NORMALIZED
                    MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS
                    MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS
                    MAX_VERTEX_OUTPUT_COMPONENTS MAX_FRAGMENT_INPUT_COMPONENTS
                    MAX_SERVER_WAIT_TIMEOUT OBJECT_TYPE SYNC_FENCE RGB10_A2UI INT_2_10_10_10_REV
                    TEXTURE_IMMUTABLE_FORMAT MAX_ELEMENT_INDEX TEXTURE_IMMUTABLE_LEVELS
                `.trim().split(/\s+/);
                const constants = {
                    complete: requiredBaseConstants.every(name =>
                            typeof WebGLRenderingContext.prototype[name] === "number")
                        && requiredWebGL2Constants.every(name =>
                            typeof WebGL2RenderingContext.prototype[name] === "number")
                        && typeof WebGLRenderingContext.prototype.TEXTURE31 === "number"
                        && typeof WebGL2RenderingContext.prototype.DRAW_BUFFER15 === "number"
                        && typeof WebGL2RenderingContext.prototype.COLOR_ATTACHMENT15 === "number",
                    base: WebGLRenderingContext.RGBA4 === 0x8056
                        && WebGLRenderingContext.TEXTURE_BINDING_CUBE_MAP === 0x8514
                        && WebGLRenderingContext.MAX_COMBINED_TEXTURE_IMAGE_UNITS === 0x8B4D,
                    formats: WebGL2RenderingContext.R8 === 0x8229
                        && WebGL2RenderingContext.RG8 === 0x822B
                        && WebGL2RenderingContext.R32UI === 0x8236
                        && WebGL2RenderingContext.SRGB8_ALPHA8 === 0x8C43
                        && WebGL2RenderingContext.RGBA8_SNORM === 0x8F97,
                    reflection: WebGL2RenderingContext.VERTEX_ARRAY_BINDING === 0x85B5
                        && WebGL2RenderingContext.TEXTURE_IMMUTABLE_FORMAT === 0x912F
                        && WebGL2RenderingContext.MAX_SERVER_WAIT_TIMEOUT === 0x9111,
                };
                if (!context) return JSON.stringify({ available: false, constants });

                const cube = context.createTexture();
                context.bindTexture(context.TEXTURE_CUBE_MAP, cube);
                const cubeBound = context.getParameter(context.TEXTURE_BINDING_CUBE_MAP) === cube;
                context.deleteTexture(cube);
                const cubeDeleted = context.getParameter(context.TEXTURE_BINDING_CUBE_MAP) === null;

                const vertexArray = context.createVertexArray();
                context.bindVertexArray(vertexArray);
                const vertexArrayBound = context.getParameter(context.VERTEX_ARRAY_BINDING) === vertexArray;
                context.deleteVertexArray(vertexArray);
                const vertexArrayDeleted = context.getParameter(context.VERTEX_ARRAY_BINDING) === null;

                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                const allocate = (internalFormat, format, type, data) => {
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, internalFormat, 1, 1, 0, format, type, data);
                    context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, texture, 0);
                    return texture;
                };

                const red = allocate(context.R8, context.RED, context.UNSIGNED_BYTE,
                    new Uint8Array([37]));
                const redComplete = context.checkFramebufferStatus(context.FRAMEBUFFER)
                    === context.FRAMEBUFFER_COMPLETE;
                const redPixel = new Uint8Array(1);
                context.readPixels(0, 0, 1, 1, context.RED, context.UNSIGNED_BYTE, redPixel);

                const rg = allocate(context.RG8, context.RG, context.UNSIGNED_BYTE,
                    new Uint8Array([19, 73]));
                const rgPixel = new Uint8Array(2);
                context.readPixels(0, 0, 1, 1, context.RG, context.UNSIGNED_BYTE, rgPixel);

                const integer = allocate(context.R32UI, context.RED_INTEGER, context.UNSIGNED_INT, null);
                context.clearBufferuiv(context.COLOR, 0, new Uint32Array([123456, 0, 0, 1]));
                const integerPixel = new Uint32Array(1);
                context.readPixels(0, 0, 1, 1, context.RED_INTEGER, context.UNSIGNED_INT, integerPixel);

                const immutable = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, immutable);
                context.texStorage2D(context.TEXTURE_2D, 1, context.SRGB8_ALPHA8, 1, 1);
                const immutableFormat = context.getTexParameter(
                    context.TEXTURE_2D, context.TEXTURE_IMMUTABLE_FORMAT);
                const immutableLevels = context.getTexParameter(
                    context.TEXTURE_2D, context.TEXTURE_IMMUTABLE_LEVELS);
                context.texSubImage2D(context.TEXTURE_2D, 0, 0, 0, 1, 1,
                    context.RGBA, context.UNSIGNED_BYTE, new Uint8Array([128, 64, 32, 255]));
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.TEXTURE_2D, immutable, 0);
                const srgbPixel = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, srgbPixel);

                return JSON.stringify({
                    available: true,
                    constants,
                    cubeBound,
                    cubeDeleted,
                    vertexArrayBound,
                    vertexArrayDeleted,
                    redComplete,
                    redPixel: [...redPixel],
                    rgPixel: [...rgPixel],
                    integerPixel: [...integerPixel],
                    immutableFormat,
                    immutableLevels,
                    srgbPixel: [...srgbPixel],
                    limits: [context.getParameter(context.MAX_ELEMENT_INDEX),
                        context.getParameter(context.MAX_SERVER_WAIT_TIMEOUT)],
                    objects: [red, rg, integer, immutable].every(Boolean),
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["constants"]["complete"], true);
    assert_eq!(result["constants"]["base"], true);
    assert_eq!(result["constants"]["formats"], true);
    assert_eq!(result["constants"]["reflection"], true);
    if result["available"] == true {
        assert_eq!(result["cubeBound"], true);
        assert_eq!(result["cubeDeleted"], true);
        assert_eq!(result["vertexArrayBound"], true);
        assert_eq!(result["vertexArrayDeleted"], true);
        assert_eq!(result["redComplete"], true);
        assert_eq!(result["redPixel"], serde_json::json!([37]));
        assert_eq!(result["rgPixel"], serde_json::json!([19, 73]));
        assert_eq!(result["integerPixel"], serde_json::json!([123456]));
        assert_eq!(result["immutableFormat"], true);
        assert_eq!(result["immutableLevels"], 1);
        assert_eq!(result["srgbPixel"], serde_json::json!([128, 64, 32, 255]));
        assert!(result["limits"][0].as_u64().is_some_and(|value| value > 0));
        assert!(result["limits"][1].as_u64().is_some());
        assert_eq!(result["objects"], true);
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl_context_loss_restores_a_fresh_angle_context_and_resize_preserves_resources() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().canvas(true).webgl(true).build())
        .unwrap();
    page.set_content("<canvas id='canvas' width='4' height='4'></canvas>")
        .unwrap();

    let initial = page
        .eval(
            r#"(() => {
                const canvas = document.getElementById("canvas");
                const context = canvas.getContext("webgl");
                const api = typeof WebGLContextEvent === "function"
                    && "onwebglcontextlost" in canvas
                    && "onwebglcontextrestored" in canvas
                    && WebGLRenderingContext.prototype.isContextLost.toString() === "function isContextLost() { [native code] }";
                if (!context) return JSON.stringify({ available: false, api });
                const extension = context.getExtension("WEBGL_lose_context");
                const parallel = context.getExtension("KHR_parallel_shader_compile");
                const clipControl = context.getExtension("EXT_clip_control");
                const polygonOffsetClamp = context.getExtension("EXT_polygon_offset_clamp");
                const depthClamp = context.getExtension("EXT_depth_clamp");
                const requestableMipmap = context.getExtension("OES_fbo_render_mipmap");
                const textureNorm16 = context.getExtension("EXT_texture_norm16");
                const renderSnorm = context.getExtension("EXT_render_snorm");
                const restoredExtensionNames = [
                    "EXT_conservative_depth",
                    "NV_shader_noperspective_interpolation",
                    "OES_sample_variables",
                    "OES_shader_multisample_interpolation",
                    "WEBGL_clip_cull_distance",
                    "WEBGL_provoking_vertex",
                    "WEBGL_stencil_texturing",
                    "WEBGL_render_shared_exponent",
                    "WEBGL_blend_func_extended",
                    "WEBGL_polygon_mode",
                ];
                const restoredExtensions = restoredExtensionNames.map(
                    name => context.getExtension(name));
                const clipCullDistance = restoredExtensions[4];
                const provokingVertex = restoredExtensions[5];
                if (clipCullDistance) context.enable(clipCullDistance.CLIP_DISTANCE0_WEBGL);
                const parallelProgram = parallel ? context.createProgram() : null;
                const texture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, texture);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA, 1, 1, 0, context.RGBA, context.UNSIGNED_BYTE,
                    new Uint8Array([12, 34, 56, 255]));
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0, context.TEXTURE_2D, texture, 0);
                const beforeResize = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, beforeResize);
                canvas.width = 8;
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                const afterResize = new Uint8Array(4);
                context.readPixels(0, 0, 1, 1, context.RGBA, context.UNSIGNED_BYTE, afterResize);

                globalThis.lossState = {
                    canvas, context, extension, clipControl, polygonOffsetClamp, depthClamp,
                    requestableMipmap, textureNorm16, renderSnorm,
                    restoredExtensionNames, restoredExtensions,
                    texture, framebuffer, events: [],
                };
                canvas.onwebglcontextlost = event => {
                    lossState.events.push({
                        type: event.type,
                        constructor: event.constructor.name,
                        cancelable: event.cancelable,
                        trusted: event.isTrusted,
                        status: event.statusMessage,
                        target: event.target === canvas,
                    });
                    event.preventDefault();
                };
                canvas.onwebglcontextrestored = event => {
                    lossState.events.push({
                        type: event.type,
                        constructor: event.constructor.name,
                        trusted: event.isTrusted,
                        target: event.target === canvas,
                    });
                };
                extension.loseContext();
                return JSON.stringify({
                    available: true,
                    api,
                    beforeResize: [...beforeResize],
                    afterResize: [...afterResize],
                    lostImmediately: context.isContextLost(),
                    lostError: context.getError(),
                    noErrorAfterLoss: context.getError(),
                    nullCreation: context.createBuffer() === null,
                    nullParameter: context.getParameter(context.MAX_TEXTURE_SIZE) === null,
                    nullAttributes: context.getContextAttributes() === null,
                    oldObjectsInvalid: !context.isTexture(texture) && !context.isFramebuffer(framebuffer),
                    extensionStable: extension === context.getExtension("webgl_lose_context"),
                    parallelLost: parallel === null
                        || context.getProgramParameter(parallelProgram, parallel.COMPLETION_STATUS_KHR) === true,
                    clipControlLostSafe: (() => {
                        try {
                            clipControl?.clipControlEXT(
                                clipControl.UPPER_LEFT_EXT,
                                clipControl.ZERO_TO_ONE_EXT,
                            );
                            return true;
                        } catch (_) {
                            return false;
                        }
                    })(),
                    rasterExtensionsLostSafe: (() => {
                        try {
                            polygonOffsetClamp?.polygonOffsetClampEXT(1, 1, 1);
                            if (depthClamp) context.enable(depthClamp.DEPTH_CLAMP_EXT);
                            return true;
                        } catch (_) {
                            return false;
                        }
                    })(),
                    normalizedExtensionsHidden: context.getExtension("EXT_texture_norm16") === null
                        && context.getExtension("EXT_render_snorm") === null,
                    requestableMipmapHidden:
                        context.getExtension("OES_fbo_render_mipmap") === null,
                    restoredExtensionsHidden: restoredExtensionNames.every(
                        name => context.getExtension(name) === null),
                    provokingVertexLostSafe: (() => {
                        try {
                            provokingVertex?.provokingVertexWEBGL(
                                provokingVertex.FIRST_VERTEX_CONVENTION_WEBGL);
                            return true;
                        } catch (_) {
                            return false;
                        }
                    })(),
                    extensions: context.getSupportedExtensions(),
                    lostPng: canvas.toDataURL().startsWith("data:image/png;base64,iVBORw0KGgo"),
                    native: [extension.loseContext.toString(), extension.restoreContext.toString(), WebGLContextEvent.toString()],
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let initial: serde_json::Value = serde_json::from_str(&initial).unwrap();
    assert_eq!(initial["api"], true);
    if initial["available"] != true {
        return;
    }
    assert_eq!(
        initial["beforeResize"],
        serde_json::json!([12, 34, 56, 255])
    );
    assert_eq!(initial["afterResize"], serde_json::json!([12, 34, 56, 255]));
    assert_eq!(initial["lostImmediately"], true);
    assert_eq!(initial["lostError"], 0x9242);
    assert_eq!(initial["noErrorAfterLoss"], 0);
    assert_eq!(initial["nullCreation"], true);
    assert_eq!(initial["nullParameter"], true);
    assert_eq!(initial["nullAttributes"], true);
    assert_eq!(initial["oldObjectsInvalid"], true);
    assert_eq!(initial["extensionStable"], true);
    assert_eq!(initial["parallelLost"], true);
    assert_eq!(initial["clipControlLostSafe"], true);
    assert_eq!(initial["rasterExtensionsLostSafe"], true);
    assert_eq!(initial["normalizedExtensionsHidden"], true);
    assert_eq!(initial["requestableMipmapHidden"], true);
    assert_eq!(initial["restoredExtensionsHidden"], true);
    assert_eq!(initial["provokingVertexLostSafe"], true);
    assert_eq!(initial["extensions"], serde_json::Value::Null);
    assert_eq!(initial["lostPng"], true);
    assert_eq!(
        initial["native"],
        serde_json::json!([
            "function loseContext() { [native code] }",
            "function restoreContext() { [native code] }",
            "function WebGLContextEvent() { [native code] }",
        ])
    );

    let _ = page.run_until_idle_for(Duration::from_millis(100)).unwrap();
    let lost_event = page
        .eval("JSON.stringify(lossState.events)")
        .unwrap()
        .to_string()
        .unwrap();
    let lost_event: serde_json::Value = serde_json::from_str(&lost_event).unwrap();
    assert_eq!(
        lost_event,
        serde_json::json!([{
            "type": "webglcontextlost",
            "constructor": "WebGLContextEvent",
            "cancelable": true,
            "trusted": true,
            "status": "Context lost through WEBGL_lose_context",
            "target": true,
        }])
    );

    page.eval("lossState.extension.restoreContext()").unwrap();
    let _ = page.run_until_idle_for(Duration::from_millis(100)).unwrap();
    let restored = page
        .eval(
            r#"JSON.stringify({
                lost: lossState.context.isContextLost(),
                events: lossState.events,
                attributes: lossState.context.getContextAttributes() !== null,
                newObject: lossState.context.createTexture() instanceof WebGLTexture,
                oldObjectsInvalid: !lossState.context.isTexture(lossState.texture)
                    && !lossState.context.isFramebuffer(lossState.framebuffer),
                clipControlStable: lossState.clipControl === null
                    || lossState.clipControl === lossState.context.getExtension("ext_clip_control"),
                clipControlDefaults: lossState.clipControl === null ? null : [
                    lossState.context.getParameter(lossState.clipControl.CLIP_ORIGIN_EXT),
                    lossState.context.getParameter(lossState.clipControl.CLIP_DEPTH_MODE_EXT),
                ],
                polygonOffsetClampStable: lossState.polygonOffsetClamp === null
                    || lossState.polygonOffsetClamp
                        === lossState.context.getExtension("ext_polygon_offset_clamp"),
                polygonOffsetClampDefault: lossState.polygonOffsetClamp === null ? null
                    : lossState.context.getParameter(
                        lossState.polygonOffsetClamp.POLYGON_OFFSET_CLAMP_EXT),
                depthClampStable: lossState.depthClamp === null
                    || lossState.depthClamp === lossState.context.getExtension("ext_depth_clamp"),
                depthClampDefault: lossState.depthClamp === null ? null
                    : lossState.context.getParameter(lossState.depthClamp.DEPTH_CLAMP_EXT),
                requestableMipmapStable: lossState.requestableMipmap === null
                    || lossState.requestableMipmap
                        === lossState.context.getExtension("oes_fbo_render_mipmap"),
                requestableMipmapExecution: (() => {
                    if (lossState.requestableMipmap === null) return null;
                    const context = lossState.context;
                    const texture = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, texture);
                    context.texParameteri(
                        context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST,
                    );
                    for (const [level, size] of [[0, 4], [1, 2], [2, 1]]) {
                        context.texImage2D(
                            context.TEXTURE_2D, level, context.RGBA, size, size, 0,
                            context.RGBA, context.UNSIGNED_BYTE, null,
                        );
                    }
                    const framebuffer = context.createFramebuffer();
                    context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                    context.framebufferTexture2D(
                        context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                        context.TEXTURE_2D, texture, 1,
                    );
                    return {
                        error: context.getError(),
                        status: context.checkFramebufferStatus(context.FRAMEBUFFER),
                    };
                })(),
                textureNorm16Stable: lossState.textureNorm16 === null
                    || lossState.textureNorm16
                        === lossState.context.getExtension("ext_texture_norm16"),
                renderSnormStable: lossState.renderSnorm === null
                    || lossState.renderSnorm === lossState.context.getExtension("ext_render_snorm"),
                restoredExtensionsStable: lossState.restoredExtensions.every((extension, index) =>
                    extension === null || extension === lossState.context.getExtension(
                        lossState.restoredExtensionNames[index].toLowerCase())),
                clipDistanceDefault: lossState.restoredExtensions[4] === null ? null
                    : lossState.context.getParameter(
                        lossState.restoredExtensions[4].CLIP_DISTANCE0_WEBGL),
                provokingVertexDefault: lossState.restoredExtensions[5] === null ? null
                    : lossState.context.getParameter(
                        lossState.restoredExtensions[5].PROVOKING_VERTEX_WEBGL),
                blendFuncExtendedRestored: lossState.restoredExtensions[8] === null ? null
                    : lossState.context.getParameter(
                        lossState.restoredExtensions[8].MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL),
                polygonModeDefault: lossState.restoredExtensions[9] === null ? null
                    : lossState.context.getParameter(
                        lossState.restoredExtensions[9].POLYGON_MODE_WEBGL),
                error: lossState.context.getError(),
            })"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let restored: serde_json::Value = serde_json::from_str(&restored).unwrap();
    let clip_control_defaults = if restored["clipControlDefaults"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!([0x8CA1, 0x935E])
    };
    let polygon_offset_clamp_default = if restored["polygonOffsetClampDefault"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!(0)
    };
    let depth_clamp_default = if restored["depthClampDefault"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!(false)
    };
    let requestable_mipmap_execution = if restored["requestableMipmapExecution"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!({ "error": 0, "status": 0x8CD5 })
    };
    let clip_distance_default = if restored["clipDistanceDefault"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!(false)
    };
    let provoking_vertex_default = if restored["provokingVertexDefault"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!(0x8E4E)
    };
    let blend_func_extended_restored = if restored["blendFuncExtendedRestored"].is_null() {
        serde_json::Value::Null
    } else {
        restored["blendFuncExtendedRestored"].clone()
    };
    if !blend_func_extended_restored.is_null() {
        assert!(blend_func_extended_restored.as_i64().unwrap() >= 1);
    }
    let polygon_mode_default = if restored["polygonModeDefault"].is_null() {
        serde_json::Value::Null
    } else {
        serde_json::json!(0x1B02)
    };
    assert_eq!(
        restored,
        serde_json::json!({
            "lost": false,
            "events": [
                {
                    "type": "webglcontextlost",
                    "constructor": "WebGLContextEvent",
                    "cancelable": true,
                    "trusted": true,
                    "status": "Context lost through WEBGL_lose_context",
                    "target": true,
                },
                {
                    "type": "webglcontextrestored",
                    "constructor": "WebGLContextEvent",
                    "trusted": true,
                    "target": true,
                }
            ],
            "attributes": true,
            "newObject": true,
            "oldObjectsInvalid": true,
            "clipControlStable": true,
            "clipControlDefaults": clip_control_defaults,
            "polygonOffsetClampStable": true,
            "polygonOffsetClampDefault": polygon_offset_clamp_default,
            "depthClampStable": true,
            "depthClampDefault": depth_clamp_default,
            "requestableMipmapStable": true,
            "requestableMipmapExecution": requestable_mipmap_execution,
            "textureNorm16Stable": true,
            "renderSnormStable": true,
            "restoredExtensionsStable": true,
            "clipDistanceDefault": clip_distance_default,
            "provokingVertexDefault": provoking_vertex_default,
            "blendFuncExtendedRestored": blend_func_extended_restored,
            "polygonModeDefault": polygon_mode_default,
            "error": 0,
        })
    );
}
