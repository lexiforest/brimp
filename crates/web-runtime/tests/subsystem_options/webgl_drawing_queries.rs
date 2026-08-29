use std::{sync::Arc, time::Duration};

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn webgl_extended_blending_and_polygon_modes_execute_through_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"JSON.stringify(["webgl", "webgl2"].map(type => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 8;
                const context = canvas.getContext(type);
                if (context === null) return { available: false };

                const beforeBlend = context.getParameter(0x88FC);
                const beforeBlendError = context.getError();
                const beforePolygon = context.getParameter(0x0B40);
                const beforePolygonError = context.getError();
                const blend = context.getExtension("WEBGL_blend_func_extended");
                const polygon = context.getExtension("WEBGL_polygon_mode");

                const compile = (shaderType, source) => {
                    const shader = context.createShader(shaderType);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const version2 = type === "webgl2";
                const vertex = compile(context.VERTEX_SHADER, version2
                    ? `#version 300 es
                        in vec2 position;
                        void main() { gl_Position = vec4(position, 0.0, 1.0); }`
                    : `attribute vec2 position;
                        void main() { gl_Position = vec4(position, 0.0, 1.0); }`);
                const positions = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, positions);
                context.bufferData(context.ARRAY_BUFFER,
                    new Float32Array([-1, -1, 3, -1, -1, 3]), context.STATIC_DRAW);

                let blendResult = null;
                if (blend !== null) {
                    const fragment = compile(context.FRAGMENT_SHADER, version2
                        ? `#version 300 es
                            #extension GL_EXT_blend_func_extended : require
                            precision mediump float;
                            layout(location = 0) out vec4 primary;
                            layout(location = 0, index = 1) out vec4 secondary;
                            void main() {
                                primary = vec4(1.0, 0.0, 0.0, 1.0);
                                secondary = vec4(0.0, 0.0, 0.0, 0.5);
                            }`
                        : `#extension GL_EXT_blend_func_extended : require
                            precision mediump float;
                            void main() {
                                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                                gl_SecondaryFragColorEXT = vec4(0.0, 0.0, 0.0, 0.5);
                            }`);
                    const program = context.createProgram();
                    context.attachShader(program, vertex);
                    context.attachShader(program, fragment);
                    context.bindAttribLocation(program, 0, "position");
                    context.linkProgram(program);
                    context.useProgram(program);
                    context.vertexAttribPointer(0, 2, context.FLOAT, false, 0, 0);
                    context.enableVertexAttribArray(0);
                    context.viewport(0, 0, 8, 8);
                    context.clearColor(0, 0, 1, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.enable(context.BLEND);
                    context.blendFunc(
                        blend.SRC1_ALPHA_WEBGL,
                        blend.ONE_MINUS_SRC1_ALPHA_WEBGL,
                    );
                    const state = [
                        context.getParameter(context.BLEND_SRC_RGB),
                        context.getParameter(context.BLEND_DST_RGB),
                    ];
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const pixel = new Uint8Array(4);
                    context.readPixels(
                        2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel,
                    );
                    blendResult = {
                        constants: [
                            blend.SRC1_COLOR_WEBGL,
                            blend.SRC1_ALPHA_WEBGL,
                            blend.ONE_MINUS_SRC1_COLOR_WEBGL,
                            blend.ONE_MINUS_SRC1_ALPHA_WEBGL,
                            blend.MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL,
                        ],
                        maximum: context.getParameter(
                            blend.MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL),
                        compiled: context.getShaderParameter(
                            fragment, context.COMPILE_STATUS),
                        linked: context.getProgramParameter(program, context.LINK_STATUS),
                        state,
                        pixel: [...pixel],
                        stable: blend === context.getExtension("webgl_blend_func_extended"),
                    };
                    context.disable(context.BLEND);
                }

                let polygonResult = null;
                if (polygon !== null) {
                    const fragment = compile(context.FRAGMENT_SHADER, version2
                        ? `#version 300 es
                            precision mediump float;
                            out vec4 color;
                            void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }`
                        : `precision mediump float;
                            void main() { gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }`);
                    const program = context.createProgram();
                    context.attachShader(program, vertex);
                    context.attachShader(program, fragment);
                    context.bindAttribLocation(program, 0, "position");
                    context.linkProgram(program);
                    context.useProgram(program);
                    context.vertexAttribPointer(0, 2, context.FLOAT, false, 0, 0);
                    context.enableVertexAttribArray(0);
                    const initial = context.getParameter(polygon.POLYGON_MODE_WEBGL);
                    polygon.polygonModeWEBGL(context.FRONT_AND_BACK, polygon.LINE_WEBGL);
                    const lineState = context.getParameter(polygon.POLYGON_MODE_WEBGL);
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const linePixel = new Uint8Array(4);
                    context.readPixels(
                        2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, linePixel,
                    );
                    context.enable(polygon.POLYGON_OFFSET_LINE_WEBGL);
                    const offsetLine = context.getParameter(
                        polygon.POLYGON_OFFSET_LINE_WEBGL);
                    context.disable(polygon.POLYGON_OFFSET_LINE_WEBGL);
                    polygon.polygonModeWEBGL(context.FRONT, polygon.FILL_WEBGL);
                    const invalidFace = context.getError();
                    polygon.polygonModeWEBGL(context.FRONT_AND_BACK, 0);
                    const invalidMode = context.getError();
                    polygon.polygonModeWEBGL(context.FRONT_AND_BACK, polygon.FILL_WEBGL);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const fillPixel = new Uint8Array(4);
                    context.readPixels(
                        2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, fillPixel,
                    );
                    polygonResult = {
                        constants: [
                            polygon.POLYGON_MODE_WEBGL,
                            polygon.POLYGON_OFFSET_LINE_WEBGL,
                            polygon.LINE_WEBGL,
                            polygon.FILL_WEBGL,
                        ],
                        initial,
                        lineState,
                        offsetLine,
                        invalidFace,
                        invalidMode,
                        linePixel: [...linePixel],
                        fillPixel: [...fillPixel],
                        stable: polygon === context.getExtension("webgl_polygon_mode"),
                        native: polygon.polygonModeWEBGL.toString(),
                    };
                }

                return {
                    available: true,
                    beforeBlend,
                    beforeBlendError,
                    beforePolygon,
                    beforePolygonError,
                    listed: {
                        blend: context.getSupportedExtensions()
                            .includes("WEBGL_blend_func_extended"),
                        polygon: context.getSupportedExtensions().includes("WEBGL_polygon_mode"),
                    },
                    blend: blendResult,
                    polygon: polygonResult,
                    error: context.getError(),
                };
            }))"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for context in result.as_array().unwrap() {
        if context["available"] != true {
            continue;
        }
        assert_eq!(context["beforeBlend"], serde_json::Value::Null);
        assert_eq!(context["beforeBlendError"], 0x0500);
        assert_eq!(context["beforePolygon"], serde_json::Value::Null);
        assert_eq!(context["beforePolygonError"], 0x0500);
        assert_eq!(context["listed"]["blend"], !context["blend"].is_null());
        assert_eq!(context["listed"]["polygon"], !context["polygon"].is_null());
        if !context["blend"].is_null() {
            assert_eq!(
                context["blend"]["constants"],
                serde_json::json!([0x88F9, 0x8589, 0x88FA, 0x88FB, 0x88FC])
            );
            assert!(context["blend"]["maximum"].as_i64().unwrap() >= 1);
            assert_eq!(context["blend"]["compiled"], true);
            assert_eq!(context["blend"]["linked"], true);
            assert_eq!(
                context["blend"]["state"],
                serde_json::json!([0x8589, 0x88FB])
            );
            let pixel = context["blend"]["pixel"].as_array().unwrap();
            assert!((127..=128).contains(&pixel[0].as_u64().unwrap()));
            assert_eq!(pixel[1], 0);
            assert!((127..=128).contains(&pixel[2].as_u64().unwrap()));
            assert_eq!(pixel[3], 255);
            assert_eq!(context["blend"]["stable"], true);
        }
        if !context["polygon"].is_null() {
            assert_eq!(
                context["polygon"]["constants"],
                serde_json::json!([0x0B40, 0x2A02, 0x1B01, 0x1B02])
            );
            assert_eq!(context["polygon"]["initial"], 0x1B02);
            assert_eq!(context["polygon"]["lineState"], 0x1B01);
            assert_eq!(context["polygon"]["offsetLine"], true);
            assert_eq!(context["polygon"]["invalidFace"], 0x0500);
            assert_eq!(context["polygon"]["invalidMode"], 0x0500);
            assert_eq!(
                context["polygon"]["linePixel"],
                serde_json::json!([0, 0, 0, 255])
            );
            assert_eq!(
                context["polygon"]["fillPixel"],
                serde_json::json!([255, 0, 0, 255])
            );
            assert_eq!(context["polygon"]["stable"], true);
            assert_eq!(
                context["polygon"]["native"],
                "function polygonModeWEBGL() { [native code] }"
            );
        }
        assert_eq!(context["error"], 0);
    }
}

#[test]
fn webgl2_visibility_queries_execute_and_reflect_active_state() {
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
                    out vec4 outputColor;
                    void main() { outputColor = vec4(1.0); }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                context.viewport(0, 0, 4, 4);

                const visible = context.createQuery();
                context.beginQuery(context.ANY_SAMPLES_PASSED, visible);
                const active = context.getQuery(
                    context.ANY_SAMPLES_PASSED,
                    context.CURRENT_QUERY,
                ) === visible;
                context.drawArrays(context.TRIANGLES, 0, 3);
                context.endQuery(context.ANY_SAMPLES_PASSED);
                const inactive = context.getQuery(
                    context.ANY_SAMPLES_PASSED,
                    context.CURRENT_QUERY,
                ) === null;
                const visibleResult = context.getQueryParameter(visible, context.QUERY_RESULT);
                const visibleAvailable = context.getQueryParameter(
                    visible,
                    context.QUERY_RESULT_AVAILABLE,
                );

                const empty = context.createQuery();
                context.beginQuery(context.ANY_SAMPLES_PASSED_CONSERVATIVE, empty);
                context.drawArrays(context.TRIANGLES, 0, 0);
                context.endQuery(context.ANY_SAMPLES_PASSED_CONSERVATIVE);
                const emptyResult = context.getQueryParameter(empty, context.QUERY_RESULT);
                const objects = visible instanceof WebGLQuery
                    && empty instanceof WebGLQuery
                    && context.isQuery(visible)
                    && context.isQuery(empty);
                const native = [
                    context.createQuery.toString(),
                    context.beginQuery.toString(),
                    context.getQueryParameter.toString(),
                ];
                context.deleteQuery(visible);
                context.deleteQuery(empty);
                return JSON.stringify({
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    active,
                    inactive,
                    visibleResult,
                    visibleAvailable,
                    emptyResult,
                    objects,
                    deleted: !context.isQuery(visible) && !context.isQuery(empty),
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
            "active",
            "inactive",
            "visibleResult",
            "visibleAvailable",
            "objects",
            "deleted",
        ] {
            assert_eq!(result[name], true, "failed query check: {name}");
        }
        assert_eq!(result["emptyResult"], false);
        assert_eq!(result["error"], 0);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function createQuery() { [native code] }",
                "function beginQuery() { [native code] }",
                "function getQueryParameter() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl_timer_query_extensions_execute_and_preserve_64_bit_results_when_supported() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const exerciseWebGL1 = () => {
                    const context = document.createElement("canvas").getContext("webgl");
                    if (!context) return { available: false };
                    const extension = context.getExtension("EXT_disjoint_timer_query");
                    if (!extension) return { available: true, supported: false };
                    const elapsedBits = extension.getQueryEXT(
                        extension.TIME_ELAPSED_EXT, extension.QUERY_COUNTER_BITS_EXT);
                    const timestampBits = extension.getQueryEXT(
                        extension.TIMESTAMP_EXT, extension.QUERY_COUNTER_BITS_EXT);
                    const elapsed = extension.createQueryEXT();
                    extension.beginQueryEXT(extension.TIME_ELAPSED_EXT, elapsed);
                    const active = extension.getQueryEXT(
                        extension.TIME_ELAPSED_EXT, extension.CURRENT_QUERY_EXT) === elapsed;
                    context.clear(context.COLOR_BUFFER_BIT);
                    extension.endQueryEXT(extension.TIME_ELAPSED_EXT);
                    const inactive = extension.getQueryEXT(
                        extension.TIME_ELAPSED_EXT, extension.CURRENT_QUERY_EXT) === null;
                    const elapsedResult = extension.getQueryObjectEXT(
                        elapsed, extension.QUERY_RESULT_EXT);
                    const elapsedAvailable = extension.getQueryObjectEXT(
                        elapsed, extension.QUERY_RESULT_AVAILABLE_EXT);

                    const timestamp = timestampBits > 0 ? extension.createQueryEXT() : null;
                    if (timestamp) extension.queryCounterEXT(timestamp, extension.TIMESTAMP_EXT);
                    const timestampResult = timestamp
                        ? extension.getQueryObjectEXT(timestamp, extension.QUERY_RESULT_EXT) : null;
                    const timestampAvailable = timestamp
                        ? extension.getQueryObjectEXT(timestamp,
                            extension.QUERY_RESULT_AVAILABLE_EXT) : null;
                    const currentTimestamp = timestampBits > 0
                        ? context.getParameter(extension.TIMESTAMP_EXT) : null;
                    const disjoint = context.getParameter(extension.GPU_DISJOINT_EXT);

                    const activeDeletion = extension.createQueryEXT();
                    extension.beginQueryEXT(extension.TIME_ELAPSED_EXT, activeDeletion);
                    extension.deleteQueryEXT(activeDeletion);
                    const deletedActive = !extension.isQueryEXT(activeDeletion)
                        && extension.getQueryEXT(extension.TIME_ELAPSED_EXT,
                            extension.CURRENT_QUERY_EXT) === null;
                    extension.deleteQueryEXT(elapsed);
                    if (timestamp) extension.deleteQueryEXT(timestamp);
                    return {
                        available: true,
                        supported: true,
                        stable: extension === context.getExtension("ext_disjoint_timer_query"),
                        constants: extension.QUERY_COUNTER_BITS_EXT === 0x8864
                            && extension.TIME_ELAPSED_EXT === 0x88BF
                            && extension.TIMESTAMP_EXT === 0x8E28
                            && extension.GPU_DISJOINT_EXT === 0x8FBB,
                        elapsedBits,
                        timestampBits,
                        active,
                        inactive,
                        elapsedResult,
                        elapsedAvailable,
                        timestampResult,
                        timestampAvailable,
                        currentTimestamp,
                        disjoint,
                        deletedActive,
                        deleted: !extension.isQueryEXT(elapsed)
                            && (timestamp === null || !extension.isQueryEXT(timestamp)),
                        noGlobalConstructor: typeof WebGLTimerQueryEXT === "undefined",
                        native: [
                            extension.createQueryEXT.toString(),
                            extension.queryCounterEXT.toString(),
                            extension.getQueryObjectEXT.toString(),
                        ],
                        error: context.getError(),
                    };
                };
                const exerciseWebGL2 = () => {
                    const context = document.createElement("canvas").getContext("webgl2");
                    if (!context) return { available: false };
                    const extension = context.getExtension("EXT_disjoint_timer_query_webgl2");
                    if (!extension) return { available: true, supported: false };
                    const elapsedBits = context.getQuery(
                        extension.TIME_ELAPSED_EXT, extension.QUERY_COUNTER_BITS_EXT);
                    const timestampBits = context.getQuery(
                        extension.TIMESTAMP_EXT, extension.QUERY_COUNTER_BITS_EXT);
                    const elapsed = context.createQuery();
                    context.beginQuery(extension.TIME_ELAPSED_EXT, elapsed);
                    const active = context.getQuery(
                        extension.TIME_ELAPSED_EXT, context.CURRENT_QUERY) === elapsed;
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.endQuery(extension.TIME_ELAPSED_EXT);
                    const elapsedResult = context.getQueryParameter(elapsed, context.QUERY_RESULT);
                    const elapsedAvailable = context.getQueryParameter(
                        elapsed, context.QUERY_RESULT_AVAILABLE);

                    const timestamp = timestampBits > 0 ? context.createQuery() : null;
                    if (timestamp) extension.queryCounterEXT(timestamp, extension.TIMESTAMP_EXT);
                    const timestampCurrent = context.getQuery(
                        extension.TIMESTAMP_EXT, context.CURRENT_QUERY) === null;
                    const timestampResult = timestamp
                        ? context.getQueryParameter(timestamp, context.QUERY_RESULT) : null;
                    const timestampAvailable = timestamp
                        ? context.getQueryParameter(timestamp,
                            context.QUERY_RESULT_AVAILABLE) : null;
                    const currentTimestamp = timestampBits > 0
                        ? context.getParameter(extension.TIMESTAMP_EXT) : null;
                    const disjoint = context.getParameter(extension.GPU_DISJOINT_EXT);
                    context.deleteQuery(elapsed);
                    if (timestamp) context.deleteQuery(timestamp);
                    return {
                        available: true,
                        supported: true,
                        stable: extension
                            === context.getExtension("ext_disjoint_timer_query_webgl2"),
                        constants: extension.QUERY_COUNTER_BITS_EXT === 0x8864
                            && extension.TIME_ELAPSED_EXT === 0x88BF
                            && extension.TIMESTAMP_EXT === 0x8E28
                            && extension.GPU_DISJOINT_EXT === 0x8FBB,
                        elapsedBits,
                        timestampBits,
                        active,
                        elapsedResult,
                        elapsedAvailable,
                        timestampCurrent,
                        timestampResult,
                        timestampAvailable,
                        currentTimestamp,
                        disjoint,
                        deleted: !context.isQuery(elapsed)
                            && (timestamp === null || !context.isQuery(timestamp)),
                        native: extension.queryCounterEXT.toString(),
                        error: context.getError(),
                    };
                };
                return JSON.stringify({ webgl: exerciseWebGL1(), webgl2: exerciseWebGL2() });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for kind in ["webgl", "webgl2"] {
        if result[kind]["supported"] != true {
            continue;
        }
        assert_eq!(result[kind]["stable"], true);
        assert_eq!(result[kind]["constants"], true);
        assert!(result[kind]["elapsedBits"].as_i64().unwrap() >= 30);
        let timestamp_bits = result[kind]["timestampBits"].as_i64().unwrap();
        assert!(timestamp_bits == 0 || timestamp_bits >= 30);
        assert_eq!(result[kind]["active"], true);
        assert_eq!(result[kind]["elapsedAvailable"], true);
        assert!(result[kind]["elapsedResult"].as_f64().unwrap() >= 0.0);
        if timestamp_bits > 0 {
            assert_eq!(result[kind]["timestampAvailable"], true);
            assert!(result[kind]["timestampResult"].as_f64().unwrap() > 0.0);
            assert!(result[kind]["currentTimestamp"].as_f64().unwrap() > 0.0);
        } else {
            assert_eq!(result[kind]["timestampAvailable"], serde_json::Value::Null);
            assert_eq!(result[kind]["timestampResult"], serde_json::Value::Null);
            assert_eq!(result[kind]["currentTimestamp"], serde_json::Value::Null);
        }
        assert!(result[kind]["disjoint"].is_boolean());
        assert_eq!(result[kind]["deleted"], true);
        assert_eq!(result[kind]["error"], 0);
    }
    if result["webgl"]["supported"] == true {
        assert_eq!(result["webgl"]["inactive"], true);
        assert_eq!(result["webgl"]["deletedActive"], true);
        assert_eq!(result["webgl"]["noGlobalConstructor"], true);
        assert_eq!(
            result["webgl"]["native"],
            serde_json::json!([
                "function createQueryEXT() { [native code] }",
                "function queryCounterEXT() { [native code] }",
                "function getQueryObjectEXT() { [native code] }",
            ])
        );
    }
    if result["webgl2"]["supported"] == true {
        assert_eq!(result["webgl2"]["timestampCurrent"], true);
        assert_eq!(
            result["webgl2"]["native"],
            "function queryCounterEXT() { [native code] }"
        );
    }
}

#[test]
fn webgl2_sync_objects_fence_and_wait_for_angle_work() {
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
                context.clearColor(0.25, 0.5, 0.75, 1);
                context.clear(context.COLOR_BUFFER_BIT);
                const sync = context.fenceSync(context.SYNC_GPU_COMMANDS_COMPLETE, 0);
                const object = sync instanceof WebGLSync && context.isSync(sync);
                const condition = context.getSyncParameter(sync, context.SYNC_CONDITION);
                const flags = context.getSyncParameter(sync, context.SYNC_FLAGS);
                const statusBefore = context.getSyncParameter(sync, context.SYNC_STATUS);
                const invalidWait = context.clientWaitSync(sync, 0, 1);
                const invalidWaitError = context.getError();
                const immediate = context.clientWaitSync(sync, 0, 0);
                let completed = immediate;
                for (let attempt = 0;
                    completed === context.TIMEOUT_EXPIRED && attempt < 10_000;
                    attempt += 1) {
                    completed = context.clientWaitSync(
                        sync,
                        context.SYNC_FLUSH_COMMANDS_BIT,
                        0,
                    );
                }
                const statusAfter = context.getSyncParameter(sync, context.SYNC_STATUS);
                context.waitSync(sync, 0, context.TIMEOUT_IGNORED);
                const native = [
                    context.fenceSync.toString(),
                    context.clientWaitSync.toString(),
                    context.getSyncParameter.toString(),
                ];
                const maximum = context.getParameter(context.MAX_CLIENT_WAIT_TIMEOUT_WEBGL);
                context.deleteSync(sync);
                return JSON.stringify({
                    object,
                    condition,
                    flags,
                    statusBefore,
                    invalidWait,
                    invalidWaitError,
                    immediate,
                    completed,
                    statusAfter,
                    maximum,
                    deleted: !context.isSync(sync),
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
        assert_eq!(result["object"], true);
        assert_eq!(result["condition"], 0x9117);
        assert_eq!(result["flags"], 0);
        assert!(matches!(
            result["statusBefore"].as_u64(),
            Some(0x9118) | Some(0x9119)
        ));
        assert_eq!(result["invalidWait"], 0x911D);
        assert_eq!(result["invalidWaitError"], 0x0502);
        assert!(matches!(
            result["immediate"].as_u64(),
            Some(0x911A) | Some(0x911B) | Some(0x911C)
        ));
        assert!(matches!(
            result["completed"].as_u64(),
            Some(0x911A) | Some(0x911C)
        ));
        assert_eq!(result["statusAfter"], 0x9119);
        assert_eq!(result["maximum"], 0);
        assert_eq!(result["deleted"], true);
        assert_eq!(result["error"], 0);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function fenceSync() { [native code] }",
                "function clientWaitSync() { [native code] }",
                "function getSyncParameter() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl_instanced_arrays_draw_with_divisors_in_both_context_versions() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const run = type => {
                    const canvas = document.createElement("canvas");
                    canvas.width = 8;
                    canvas.height = 4;
                    const context = canvas.getContext(type);
                    if (!context) return null;
                    const extension = type === "webgl"
                        ? context.getExtension("ANGLE_instanced_arrays")
                        : null;
                    if (type === "webgl" && !extension) return { missingExtension: true };
                    const drawArrays = type === "webgl"
                        ? extension.drawArraysInstancedANGLE.bind(extension)
                        : context.drawArraysInstanced.bind(context);
                    const drawElements = type === "webgl"
                        ? extension.drawElementsInstancedANGLE.bind(extension)
                        : context.drawElementsInstanced.bind(context);
                    const divisor = type === "webgl"
                        ? extension.vertexAttribDivisorANGLE.bind(extension)
                        : context.vertexAttribDivisor.bind(context);
                    const vertex = context.createShader(context.VERTEX_SHADER);
                    context.shaderSource(vertex, `
                        attribute vec2 position;
                        attribute vec2 instanceOffset;
                        void main() {
                            gl_Position = vec4(position + instanceOffset, 0.0, 1.0);
                        }
                    `);
                    context.compileShader(vertex);
                    const fragment = context.createShader(context.FRAGMENT_SHADER);
                    context.shaderSource(fragment, `
                        precision mediump float;
                        void main() { gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0); }
                    `);
                    context.compileShader(fragment);
                    const program = context.createProgram();
                    context.attachShader(program, vertex);
                    context.attachShader(program, fragment);
                    context.linkProgram(program);
                    context.useProgram(program);

                    const positions = context.createBuffer();
                    context.bindBuffer(context.ARRAY_BUFFER, positions);
                    context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                        -0.3, -0.5, 0.3, -0.5, 0, 0.5,
                    ]), context.STATIC_DRAW);
                    const position = context.getAttribLocation(program, "position");
                    context.enableVertexAttribArray(position);
                    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);

                    const offsets = context.createBuffer();
                    context.bindBuffer(context.ARRAY_BUFFER, offsets);
                    context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                        -0.55, 0, 0.55, 0,
                    ]), context.STATIC_DRAW);
                    const instanceOffset = context.getAttribLocation(program, "instanceOffset");
                    context.enableVertexAttribArray(instanceOffset);
                    context.vertexAttribPointer(instanceOffset, 2, context.FLOAT, false, 0, 0);
                    divisor(instanceOffset, 1);

                    const indices = context.createBuffer();
                    context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, indices);
                    context.bufferData(
                        context.ELEMENT_ARRAY_BUFFER,
                        new Uint16Array([0, 1, 2]),
                        context.STATIC_DRAW,
                    );
                    const pixels = () => {
                        const left = new Uint8Array(4);
                        const right = new Uint8Array(4);
                        context.readPixels(1, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, left);
                        context.readPixels(6, 1, 1, 1, context.RGBA, context.UNSIGNED_BYTE, right);
                        return [[...left], [...right]];
                    };
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    drawArrays(context.TRIANGLES, 0, 3, 2);
                    const arrayPixels = pixels();
                    context.clear(context.COLOR_BUFFER_BIT);
                    drawElements(context.TRIANGLES, 3, context.UNSIGNED_SHORT, 0, 2);
                    const indexedPixels = pixels();
                    return {
                        compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                            && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                        linked: context.getProgramParameter(program, context.LINK_STATUS),
                        arrayPixels,
                        indexedPixels,
                        error: context.getError(),
                        native: type === "webgl"
                            ? extension.drawArraysInstancedANGLE.toString()
                            : context.drawArraysInstanced.toString(),
                    };
                };
                return JSON.stringify({ webgl: run("webgl"), webgl2: run("webgl2") });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    for (name, native) in [
        (
            "webgl",
            "function drawArraysInstancedANGLE() { [native code] }",
        ),
        ("webgl2", "function drawArraysInstanced() { [native code] }"),
    ] {
        if result[name].is_null() {
            continue;
        }
        assert_eq!(result[name]["missingExtension"], serde_json::Value::Null);
        assert_eq!(result[name]["compiled"], true);
        assert_eq!(result[name]["linked"], true);
        assert_eq!(
            result[name]["arrayPixels"],
            serde_json::json!([[0, 255, 0, 255], [0, 255, 0, 255]])
        );
        assert_eq!(
            result[name]["indexedPixels"],
            serde_json::json!([[0, 255, 0, 255], [0, 255, 0, 255]])
        );
        assert_eq!(result[name]["error"], 0);
        assert_eq!(result[name]["native"], native);
    }
}

#[test]
fn webgl2_indexed_blending_controls_independent_draw_buffers_when_supported() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = canvas.height = 1;
                const context = canvas.getContext("webgl2");
                if (!context) return JSON.stringify({ available: false });
                const extension = context.getExtension("OES_draw_buffers_indexed");
                if (!extension) return JSON.stringify({ available: true, supported: false });

                const texture = () => {
                    const value = context.createTexture();
                    context.bindTexture(context.TEXTURE_2D, value);
                    context.texParameteri(context.TEXTURE_2D,
                        context.TEXTURE_MIN_FILTER, context.NEAREST);
                    context.texParameteri(context.TEXTURE_2D,
                        context.TEXTURE_MAG_FILTER, context.NEAREST);
                    context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8,
                        1, 1, 0, context.RGBA, context.UNSIGNED_BYTE, null);
                    return value;
                };
                const first = texture();
                const second = texture();
                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.TEXTURE_2D, first, 0);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT1,
                    context.TEXTURE_2D, second, 0);
                context.drawBuffers([context.COLOR_ATTACHMENT0, context.COLOR_ATTACHMENT1]);

                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    void main() {
                        vec2 positions[3] = vec2[3](vec2(-1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
                        gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    layout(location = 0) out vec4 first;
                    layout(location = 1) out vec4 second;
                    void main() {
                        first = vec4(1.0, 0.0, 0.0, 0.5);
                        second = vec4(0.0, 1.0, 0.0, 0.5);
                    }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                context.viewport(0, 0, 1, 1);

                context.clearBufferfv(context.COLOR, 0, new Float32Array([0, 0, 1, 1]));
                context.clearBufferfv(context.COLOR, 1, new Float32Array([0, 0, 1, 1]));
                extension.colorMaskiOES(0, true, false, false, true);
                extension.colorMaskiOES(1, false, true, false, true);
                const masks = [
                    context.getIndexedParameter(context.COLOR_WRITEMASK, 0),
                    context.getIndexedParameter(context.COLOR_WRITEMASK, 1),
                ];
                context.drawArrays(context.TRIANGLES, 0, 3);
                const read = attachment => {
                    context.readBuffer(attachment);
                    const pixel = new Uint8Array(4);
                    context.readPixels(0, 0, 1, 1,
                        context.RGBA, context.UNSIGNED_BYTE, pixel);
                    return [...pixel];
                };
                const masked = [read(context.COLOR_ATTACHMENT0), read(context.COLOR_ATTACHMENT1)];

                extension.colorMaskiOES(0, true, true, true, true);
                extension.colorMaskiOES(1, true, true, true, true);
                context.clearBufferfv(context.COLOR, 0, new Float32Array([0, 0, 0, 0]));
                context.clearBufferfv(context.COLOR, 1, new Float32Array([0, 0, 0, 0]));
                extension.enableiOES(context.BLEND, 0);
                extension.disableiOES(context.BLEND, 1);
                extension.blendEquationiOES(0, context.FUNC_ADD);
                extension.blendEquationSeparateiOES(1,
                    context.FUNC_SUBTRACT, context.FUNC_REVERSE_SUBTRACT);
                extension.blendFunciOES(0, context.SRC_ALPHA, context.ONE_MINUS_SRC_ALPHA);
                extension.blendFuncSeparateiOES(1,
                    context.ONE, context.ZERO, context.ZERO, context.ONE);
                const state = {
                    equation0: [
                        context.getIndexedParameter(context.BLEND_EQUATION_RGB, 0),
                        context.getIndexedParameter(context.BLEND_EQUATION_ALPHA, 0),
                    ],
                    equation1: [
                        context.getIndexedParameter(context.BLEND_EQUATION_RGB, 1),
                        context.getIndexedParameter(context.BLEND_EQUATION_ALPHA, 1),
                    ],
                    factors0: [
                        context.getIndexedParameter(context.BLEND_SRC_RGB, 0),
                        context.getIndexedParameter(context.BLEND_DST_RGB, 0),
                        context.getIndexedParameter(context.BLEND_SRC_ALPHA, 0),
                        context.getIndexedParameter(context.BLEND_DST_ALPHA, 0),
                    ],
                    factors1: [
                        context.getIndexedParameter(context.BLEND_SRC_RGB, 1),
                        context.getIndexedParameter(context.BLEND_DST_RGB, 1),
                        context.getIndexedParameter(context.BLEND_SRC_ALPHA, 1),
                        context.getIndexedParameter(context.BLEND_DST_ALPHA, 1),
                    ],
                };
                context.drawArrays(context.TRIANGLES, 0, 3);
                const blended = [read(context.COLOR_ATTACHMENT0), read(context.COLOR_ATTACHMENT1)];
                return JSON.stringify({
                    available: true,
                    supported: true,
                    stable: extension === context.getExtension("oes_draw_buffers_indexed"),
                    complete: context.checkFramebufferStatus(context.FRAMEBUFFER)
                        === context.FRAMEBUFFER_COMPLETE,
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    masks,
                    masked,
                    state,
                    blended,
                    noIsEnabled: extension.isEnablediOES === undefined,
                    native: [
                        extension.enableiOES.toString(),
                        extension.blendEquationSeparateiOES.toString(),
                        extension.blendFuncSeparateiOES.toString(),
                        extension.colorMaskiOES.toString(),
                    ],
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["supported"] != true {
        return;
    }
    assert_eq!(result["stable"], true);
    assert_eq!(result["complete"], true);
    assert_eq!(result["compiled"], true);
    assert_eq!(result["linked"], true);
    assert_eq!(
        result["masks"],
        serde_json::json!([[true, false, false, true], [false, true, false, true]])
    );
    assert_eq!(
        result["masked"],
        serde_json::json!([[255, 0, 255, 128], [0, 255, 255, 128]])
    );
    assert_eq!(
        result["state"]["equation0"],
        serde_json::json!([0x8006, 0x8006])
    );
    assert_eq!(
        result["state"]["equation1"],
        serde_json::json!([0x800A, 0x800B])
    );
    assert_eq!(
        result["state"]["factors0"],
        serde_json::json!([0x0302, 0x0303, 0x0302, 0x0303])
    );
    assert_eq!(result["state"]["factors1"], serde_json::json!([1, 0, 0, 1]));
    assert_eq!(result["blended"][0][0], 128);
    assert_eq!(result["blended"][0][1], 0);
    assert_eq!(result["blended"][1], serde_json::json!([0, 255, 0, 128]));
    assert_eq!(result["noIsEnabled"], true);
    assert_eq!(
        result["native"],
        serde_json::json!([
            "function enableiOES() { [native code] }",
            "function blendEquationSeparateiOES() { [native code] }",
            "function blendFuncSeparateiOES() { [native code] }",
            "function colorMaskiOES() { [native code] }",
        ])
    );
    assert_eq!(result["error"], 0);
}

#[test]
fn webgl_multi_draw_batches_array_element_and_instanced_draws_when_supported() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = 8;
                canvas.height = 4;
                const context = canvas.getContext("webgl2");
                if (!context) return JSON.stringify({ available: false });
                const extension = context.getExtension("WEBGL_multi_draw");
                if (!extension) return JSON.stringify({ available: true, supported: false });

                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    #extension GL_ANGLE_multi_draw : require
                    layout(location = 0) in vec2 position;
                    flat out int drawId;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        drawId = gl_DrawID;
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    flat in int drawId;
                    out vec4 color;
                    void main() {
                        color = drawId == 0
                            ? vec4(1.0, 0.0, 0.0, 1.0)
                            : vec4(0.0, 1.0, 0.0, 1.0);
                    }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);

                const positions = new Float32Array([
                    -1, -1,  0, -1, -1,  1,  0, -1,  0,  1, -1,  1,
                     0, -1,  1, -1,  0,  1,  1, -1,  1,  1,  0,  1,
                ]);
                const vertices = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, vertices);
                context.bufferData(context.ARRAY_BUFFER, positions, context.STATIC_DRAW);
                context.enableVertexAttribArray(0);
                context.vertexAttribPointer(0, 2, context.FLOAT, false, 0, 0);
                const indices = context.createBuffer();
                context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, indices);
                context.bufferData(context.ELEMENT_ARRAY_BUFFER,
                    new Uint16Array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
                    context.STATIC_DRAW);
                context.viewport(0, 0, 8, 4);

                const pixels = () => {
                    const left = new Uint8Array(4);
                    const right = new Uint8Array(4);
                    context.readPixels(1, 2, 1, 1,
                        context.RGBA, context.UNSIGNED_BYTE, left);
                    context.readPixels(6, 2, 1, 1,
                        context.RGBA, context.UNSIGNED_BYTE, right);
                    return [[...left], [...right]];
                };
                const execute = callback => {
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    callback();
                    return pixels();
                };
                const firstStorage = new Int32Array([99, 0, 6, 99]);
                const countStorage = new Int32Array([99, 6, 6, 99]);
                const instanceStorage = new Int32Array([99, 1, 1, 99]);
                const arrays = execute(() => extension.multiDrawArraysWEBGL(
                    context.TRIANGLES, firstStorage, 1, countStorage, 1, 2));
                const elements = execute(() => extension.multiDrawElementsWEBGL(
                    context.TRIANGLES, [99, 6, 6, 99], 1,
                    context.UNSIGNED_SHORT, [99, 0, 12, 99], 1, 2));
                const arraysInstanced = execute(() => extension.multiDrawArraysInstancedWEBGL(
                    context.TRIANGLES, firstStorage, 1, countStorage, 1,
                    instanceStorage, 1, 2));
                const elementsInstanced = execute(() =>
                    extension.multiDrawElementsInstancedWEBGL(
                        context.TRIANGLES, countStorage, 1, context.UNSIGNED_SHORT,
                        new Int32Array([99, 0, 12, 99]), 1,
                        instanceStorage, 1, 2));

                extension.multiDrawArraysWEBGL(context.TRIANGLES,
                    new Int32Array([0]), 0, new Int32Array([3]), 0, 2);
                const rangeError = context.getError();
                extension.multiDrawArraysWEBGL(context.TRIANGLES,
                    new Int32Array(), 0, new Int32Array(), 0, -1);
                const negativeError = context.getError();
                const webgl1 = document.createElement("canvas").getContext("webgl");
                const webgl1Extension = webgl1?.getExtension("WEBGL_multi_draw") ?? null;
                return JSON.stringify({
                    available: true,
                    supported: true,
                    stable: extension === context.getExtension("webgl_multi_draw"),
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    arrays,
                    elements,
                    arraysInstanced,
                    elementsInstanced,
                    rangeError,
                    negativeError,
                    webgl1ImplicitInstancing: webgl1Extension === null
                        || webgl1.getExtension("ANGLE_instanced_arrays") !== null,
                    native: [
                        extension.multiDrawArraysWEBGL.toString(),
                        extension.multiDrawElementsWEBGL.toString(),
                        extension.multiDrawArraysInstancedWEBGL.toString(),
                        extension.multiDrawElementsInstancedWEBGL.toString(),
                    ],
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["supported"] != true {
        return;
    }
    assert_eq!(result["stable"], true);
    assert_eq!(result["compiled"], true);
    assert_eq!(result["linked"], true);
    let expected = serde_json::json!([[255, 0, 0, 255], [0, 255, 0, 255]]);
    for variant in ["arrays", "elements", "arraysInstanced", "elementsInstanced"] {
        assert_eq!(
            result[variant], expected,
            "failed multi-draw variant: {variant}"
        );
    }
    assert_eq!(result["rangeError"], 0x0502);
    assert_eq!(result["negativeError"], 0x0501);
    assert_eq!(result["webgl1ImplicitInstancing"], true);
    assert_eq!(
        result["native"],
        serde_json::json!([
            "function multiDrawArraysWEBGL() { [native code] }",
            "function multiDrawElementsWEBGL() { [native code] }",
            "function multiDrawArraysInstancedWEBGL() { [native code] }",
            "function multiDrawElementsInstancedWEBGL() { [native code] }",
        ])
    );
    assert_eq!(result["error"], 0);
}

#[test]
fn webgl2_base_vertex_and_base_instance_extensions_execute_through_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const canvas = document.createElement("canvas");
                canvas.width = 8;
                canvas.height = 4;
                const context = canvas.getContext("webgl2");
                const webgl1 = document.createElement("canvas").getContext("webgl");
                if (!context) return JSON.stringify({ available: false });
                const singleName = "WEBGL_draw_instanced_base_vertex_base_instance";
                const multiName = "WEBGL_multi_draw_instanced_base_vertex_base_instance";
                const single = context.getExtension(singleName);
                const multi = context.getExtension(multiName);
                if (!single || !multi) {
                    return JSON.stringify({
                        available: true,
                        supported: false,
                        advertised: context.getSupportedExtensions(),
                    });
                }

                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    layout(location = 0) in vec2 position;
                    layout(location = 1) in vec2 shift;
                    layout(location = 2) in vec3 instanceColor;
                    out vec3 color;
                    void main() {
                        gl_Position = vec4(position + shift, 0.0, 1.0);
                        color = instanceColor;
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    in vec3 color;
                    out vec4 outputColor;
                    void main() { outputColor = vec4(color, 1.0); }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);

                const positions = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, positions);
                context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                    -0.25, -0.7, 0.25, -0.7, 0.0, 0.7,
                    -0.25, -0.7, 0.25, -0.7, 0.0, 0.7,
                ]), context.STATIC_DRAW);
                context.enableVertexAttribArray(0);
                context.vertexAttribPointer(0, 2, context.FLOAT, false, 0, 0);

                const shifts = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, shifts);
                context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                    3, 0, -0.55, 0, 0.55, 0,
                ]), context.STATIC_DRAW);
                context.enableVertexAttribArray(1);
                context.vertexAttribPointer(1, 2, context.FLOAT, false, 0, 0);
                context.vertexAttribDivisor(1, 1);

                const colors = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, colors);
                context.bufferData(context.ARRAY_BUFFER, new Float32Array([
                    0, 0, 0, 1, 0, 0, 0, 1, 0,
                ]), context.STATIC_DRAW);
                context.enableVertexAttribArray(2);
                context.vertexAttribPointer(2, 3, context.FLOAT, false, 0, 0);
                context.vertexAttribDivisor(2, 1);

                const indices = context.createBuffer();
                context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, indices);
                context.bufferData(context.ELEMENT_ARRAY_BUFFER,
                    new Uint16Array([0, 1, 2]), context.STATIC_DRAW);
                context.viewport(0, 0, 8, 4);

                const pixels = () => {
                    const left = new Uint8Array(4);
                    const right = new Uint8Array(4);
                    context.readPixels(1, 2, 1, 1,
                        context.RGBA, context.UNSIGNED_BYTE, left);
                    context.readPixels(6, 2, 1, 1,
                        context.RGBA, context.UNSIGNED_BYTE, right);
                    return [[...left], [...right]];
                };
                const execute = callback => {
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    callback();
                    return pixels();
                };

                const singleArrays = execute(() =>
                    single.drawArraysInstancedBaseInstanceWEBGL(
                        context.TRIANGLES, 0, 3, 1, 1));
                const singleElements = execute(() =>
                    single.drawElementsInstancedBaseVertexBaseInstanceWEBGL(
                        context.TRIANGLES, 3, context.UNSIGNED_SHORT, 0, 1, 3, 2));
                const paddedFirsts = new Int32Array([99, 0, 0, 99]);
                const paddedCounts = new Int32Array([99, 3, 3, 99]);
                const paddedInstances = new Int32Array([99, 1, 1, 99]);
                const paddedBases = new Uint32Array([99, 1, 2, 99]);
                const multiArrays = execute(() =>
                    multi.multiDrawArraysInstancedBaseInstanceWEBGL(
                        context.TRIANGLES,
                        paddedFirsts, 1, paddedCounts, 1,
                        paddedInstances, 1, paddedBases, 1, 2));
                const multiElements = execute(() =>
                    multi.multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL(
                        context.TRIANGLES,
                        paddedCounts, 1, context.UNSIGNED_SHORT,
                        [99, 0, 0, 99], 1,
                        paddedInstances, 1, [99, 3, 3, 99], 1,
                        paddedBases, 1, 2));

                multi.multiDrawArraysInstancedBaseInstanceWEBGL(
                    context.TRIANGLES,
                    new Int32Array([0]), 0, new Int32Array([3]), 0,
                    new Int32Array([1]), 0, new Uint32Array([1]), 0, 2);
                const rangeError = context.getError();
                multi.multiDrawArraysInstancedBaseInstanceWEBGL(
                    context.TRIANGLES,
                    new Int32Array(), 0, new Int32Array(), 0,
                    new Int32Array(), 0, new Uint32Array(), 0, -1);
                const negativeError = context.getError();
                const loss = context.getExtension("WEBGL_lose_context");
                globalThis.baseDrawLossState = {
                    canvas, context, single, multi, loss, events: [],
                };
                canvas.onwebglcontextlost = event => {
                    baseDrawLossState.events.push(event.type);
                    event.preventDefault();
                };
                canvas.onwebglcontextrestored = event =>
                    baseDrawLossState.events.push(event.type);
                return JSON.stringify({
                    available: true,
                    supported: true,
                    webgl1Absent: webgl1 === null
                        || (webgl1.getExtension(singleName) === null
                            && webgl1.getExtension(multiName) === null),
                    stable: single === context.getExtension(singleName.toLowerCase())
                        && multi === context.getExtension(multiName.toLowerCase()),
                    implicitMultiDraw: context.getExtension("WEBGL_multi_draw") !== null,
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    singleArrays,
                    singleElements,
                    multiArrays,
                    multiElements,
                    rangeError,
                    negativeError,
                    native: [
                        single.drawArraysInstancedBaseInstanceWEBGL.toString(),
                        single.drawElementsInstancedBaseVertexBaseInstanceWEBGL.toString(),
                        multi.multiDrawArraysInstancedBaseInstanceWEBGL.toString(),
                        multi.multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL.toString(),
                    ],
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    if result["supported"] != true {
        return;
    }
    assert_eq!(result["webgl1Absent"], true);
    assert_eq!(result["stable"], true);
    assert_eq!(result["implicitMultiDraw"], true);
    assert_eq!(result["compiled"], true);
    assert_eq!(result["linked"], true);
    assert_eq!(
        result["singleArrays"],
        serde_json::json!([[255, 0, 0, 255], [0, 0, 0, 255]])
    );
    assert_eq!(
        result["singleElements"],
        serde_json::json!([[0, 0, 0, 255], [0, 255, 0, 255]])
    );
    let expected_multi = serde_json::json!([[255, 0, 0, 255], [0, 255, 0, 255]]);
    assert_eq!(result["multiArrays"], expected_multi);
    assert_eq!(result["multiElements"], expected_multi);
    assert_eq!(result["rangeError"], 0x0502);
    assert_eq!(result["negativeError"], 0x0501);
    assert_eq!(
        result["native"],
        serde_json::json!([
            "function drawArraysInstancedBaseInstanceWEBGL() { [native code] }",
            "function drawElementsInstancedBaseVertexBaseInstanceWEBGL() { [native code] }",
            "function multiDrawArraysInstancedBaseInstanceWEBGL() { [native code] }",
            "function multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL() { [native code] }",
        ])
    );
    assert_eq!(result["error"], 0);

    page.eval("baseDrawLossState.loss.loseContext()").unwrap();
    let _ = page.run_until_idle_for(Duration::from_millis(100)).unwrap();
    assert_eq!(
        page.eval(
            r#"(() => {
                try {
                    baseDrawLossState.single.drawArraysInstancedBaseInstanceWEBGL(
                        baseDrawLossState.context.TRIANGLES, -1, -1, -1, 1);
                    baseDrawLossState.multi.multiDrawArraysInstancedBaseInstanceWEBGL(
                        baseDrawLossState.context.TRIANGLES, null, -1, null, -1,
                        null, -1, null, -1, -1);
                    return true;
                } catch (_) { return false; }
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "true"
    );
    page.eval("baseDrawLossState.loss.restoreContext()")
        .unwrap();
    let _ = page.run_until_idle_for(Duration::from_millis(100)).unwrap();
    let restored = page
        .eval(
            r#"JSON.stringify({
                stable: baseDrawLossState.single === baseDrawLossState.context.getExtension(
                        "webgl_draw_instanced_base_vertex_base_instance")
                    && baseDrawLossState.multi === baseDrawLossState.context.getExtension(
                        "webgl_multi_draw_instanced_base_vertex_base_instance"),
                dependency: baseDrawLossState.context.getExtension("WEBGL_multi_draw") !== null,
                callable: (() => {
                    try {
                        baseDrawLossState.single.drawArraysInstancedBaseInstanceWEBGL(
                            baseDrawLossState.context.TRIANGLES, 0, 0, 0, 0);
                        baseDrawLossState.multi.multiDrawArraysInstancedBaseInstanceWEBGL(
                            baseDrawLossState.context.TRIANGLES,
                            new Int32Array(), 0, new Int32Array(), 0,
                            new Int32Array(), 0, new Uint32Array(), 0, 0);
                        return true;
                    } catch (_) { return false; }
                })(),
                events: baseDrawLossState.events,
                error: baseDrawLossState.context.getError(),
            })"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&restored).unwrap(),
        serde_json::json!({
            "stable": true,
            "dependency": true,
            "callable": true,
            "events": ["webglcontextlost", "webglcontextrestored"],
            "error": 0,
        })
    );
}

#[test]
fn webgl2_clip_cull_and_provoking_vertex_execute_through_angle() {
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
                const clip = context.getExtension("WEBGL_clip_cull_distance");
                const provoking = context.getExtension("WEBGL_provoking_vertex");
                const webgl1Absent = webgl1 === null || (
                    webgl1.getExtension("WEBGL_clip_cull_distance") === null
                    && webgl1.getExtension("WEBGL_provoking_vertex") === null
                );
                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const program = (vertexSource, fragmentSource) => {
                    const vertex = compile(context.VERTEX_SHADER, vertexSource);
                    const fragment = compile(context.FRAGMENT_SHADER, fragmentSource);
                    const result = context.createProgram();
                    context.attachShader(result, vertex);
                    context.attachShader(result, fragment);
                    context.linkProgram(result);
                    return {
                        program: result,
                        vertex: context.getShaderParameter(vertex, context.COMPILE_STATUS),
                        vertexLog: context.getShaderInfoLog(vertex),
                        fragment: context.getShaderParameter(fragment, context.COMPILE_STATUS),
                        fragmentLog: context.getShaderInfoLog(fragment),
                        linked: context.getProgramParameter(result, context.LINK_STATUS),
                        log: context.getProgramInfoLog(result),
                    };
                };
                const pixels = () => {
                    const pixel = new Uint8Array(4);
                    context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);
                    return [...pixel];
                };
                const positionData = new Float32Array([-1, -1, 3, -1, -1, 3]);
                const positions = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, positions);
                context.bufferData(context.ARRAY_BUFFER, positionData, context.STATIC_DRAW);
                let clipResult = null;
                if (clip) {
                    const linked = program(`#version 300 es
                        #extension GL_ANGLE_clip_cull_distance : require
                        in vec2 position;
                        uniform float distance;
                        void main() {
                            gl_Position = vec4(position, 0.0, 1.0);
                            gl_ClipDistance[0] = distance;
                        }`, `#version 300 es
                        #extension GL_ANGLE_clip_cull_distance : require
                        precision highp float;
                        out vec4 color;
                        void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }`);
                    context.useProgram(linked.program);
                    context.bindBuffer(context.ARRAY_BUFFER, positions);
                    const position = context.getAttribLocation(linked.program, "position");
                    context.enableVertexAttribArray(position);
                    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                    const distance = context.getUniformLocation(linked.program, "distance");
                    context.uniform1f(distance, -1);
                    context.disable(clip.CLIP_DISTANCE0_WEBGL);
                    context.clearColor(0, 0, 0, 1);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const unclipped = pixels();
                    context.enable(clip.CLIP_DISTANCE0_WEBGL);
                    const enabled = {
                        state: context.isEnabled(clip.CLIP_DISTANCE0_WEBGL),
                        parameter: context.getParameter(clip.CLIP_DISTANCE0_WEBGL),
                    };
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const clipped = pixels();
                    context.disable(clip.CLIP_DISTANCE0_WEBGL);
                    clipResult = {
                        constants: [
                            clip.MAX_CLIP_DISTANCES_WEBGL,
                            clip.MAX_CULL_DISTANCES_WEBGL,
                            clip.MAX_COMBINED_CLIP_AND_CULL_DISTANCES_WEBGL,
                            clip.CLIP_DISTANCE0_WEBGL,
                            clip.CLIP_DISTANCE7_WEBGL,
                        ],
                        limits: [
                            context.getParameter(clip.MAX_CLIP_DISTANCES_WEBGL),
                            context.getParameter(clip.MAX_CULL_DISTANCES_WEBGL),
                            context.getParameter(clip.MAX_COMBINED_CLIP_AND_CULL_DISTANCES_WEBGL),
                        ],
                        linked,
                        unclipped,
                        clipped,
                        enabled,
                        reset: context.getParameter(clip.CLIP_DISTANCE0_WEBGL),
                        stable: clip === context.getExtension("webgl_clip_cull_distance"),
                    };
                }
                let provokingResult = null;
                if (provoking) {
                    const linked = program(`#version 300 es
                        in vec2 position;
                        in float choice;
                        flat out int value;
                        void main() {
                            value = int(choice);
                            gl_Position = vec4(position, 0.0, 1.0);
                        }`, `#version 300 es
                        precision highp float;
                        flat in int value;
                        out vec4 color;
                        void main() {
                            color = value == 1 ? vec4(1.0, 0.0, 0.0, 1.0)
                                : value == 3 ? vec4(0.0, 0.0, 1.0, 1.0)
                                : vec4(0.0, 1.0, 0.0, 1.0);
                        }`);
                    context.useProgram(linked.program);
                    context.bindBuffer(context.ARRAY_BUFFER, positions);
                    const position = context.getAttribLocation(linked.program, "position");
                    context.enableVertexAttribArray(position);
                    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
                    const choices = context.createBuffer();
                    context.bindBuffer(context.ARRAY_BUFFER, choices);
                    context.bufferData(
                        context.ARRAY_BUFFER, new Float32Array([1, 2, 3]), context.STATIC_DRAW);
                    const choice = context.getAttribLocation(linked.program, "choice");
                    context.enableVertexAttribArray(choice);
                    context.vertexAttribPointer(choice, 1, context.FLOAT, false, 0, 0);
                    const initial = context.getParameter(provoking.PROVOKING_VERTEX_WEBGL);
                    provoking.provokingVertexWEBGL(provoking.LAST_VERTEX_CONVENTION_WEBGL);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const last = pixels();
                    provoking.provokingVertexWEBGL(provoking.FIRST_VERTEX_CONVENTION_WEBGL);
                    const changed = context.getParameter(provoking.PROVOKING_VERTEX_WEBGL);
                    context.clear(context.COLOR_BUFFER_BIT);
                    context.drawArrays(context.TRIANGLES, 0, 3);
                    const first = pixels();
                    provoking.provokingVertexWEBGL(0);
                    const invalid = context.getError();
                    provoking.provokingVertexWEBGL(provoking.LAST_VERTEX_CONVENTION_WEBGL);
                    provokingResult = {
                        constants: [
                            provoking.FIRST_VERTEX_CONVENTION_WEBGL,
                            provoking.LAST_VERTEX_CONVENTION_WEBGL,
                            provoking.PROVOKING_VERTEX_WEBGL,
                        ],
                        linked,
                        initial,
                        changed,
                        reset: context.getParameter(provoking.PROVOKING_VERTEX_WEBGL),
                        last,
                        first,
                        invalid,
                        stable: provoking === context.getExtension("webgl_provoking_vertex"),
                        native: provoking.provokingVertexWEBGL.toString(),
                    };
                }
                return JSON.stringify({
                    available: true,
                    webgl1Absent,
                    clip: clipResult,
                    provoking: provokingResult,
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
    if !result["clip"].is_null() {
        assert_eq!(
            result["clip"]["constants"],
            serde_json::json!([0x0D32, 0x82F9, 0x82FA, 0x3000, 0x3007])
        );
        assert!(result["clip"]["limits"][0].as_u64().unwrap() >= 1);
        assert!(result["clip"]["limits"][0].as_u64().unwrap() <= 8);
        let cull_distances = result["clip"]["limits"][1].as_u64().unwrap();
        let combined_distances = result["clip"]["limits"][2].as_u64().unwrap();
        if cull_distances == 0 {
            assert_eq!(combined_distances, 0);
        } else {
            assert!(combined_distances >= result["clip"]["limits"][0].as_u64().unwrap());
        }
        assert_eq!(
            result["clip"]["linked"]["vertex"], true,
            "{}",
            result["clip"]["linked"]
        );
        assert_eq!(
            result["clip"]["linked"]["fragment"], true,
            "{}",
            result["clip"]["linked"]
        );
        assert_eq!(
            result["clip"]["linked"]["linked"], true,
            "{}",
            result["clip"]["linked"]
        );
        assert_eq!(
            result["clip"]["unclipped"],
            serde_json::json!([255, 0, 0, 255])
        );
        assert_eq!(result["clip"]["clipped"], serde_json::json!([0, 0, 0, 255]));
        assert_eq!(
            result["clip"]["enabled"],
            serde_json::json!({"state": true, "parameter": true})
        );
        assert_eq!(result["clip"]["reset"], false);
        assert_eq!(result["clip"]["stable"], true);
    }
    if !result["provoking"].is_null() {
        assert_eq!(
            result["provoking"]["constants"],
            serde_json::json!([0x8E4D, 0x8E4E, 0x8E4F])
        );
        assert_eq!(result["provoking"]["linked"]["vertex"], true);
        assert_eq!(result["provoking"]["linked"]["fragment"], true);
        assert_eq!(result["provoking"]["linked"]["linked"], true);
        assert_eq!(result["provoking"]["initial"], 0x8E4E);
        assert_eq!(result["provoking"]["changed"], 0x8E4D);
        assert_eq!(result["provoking"]["reset"], 0x8E4E);
        assert_eq!(
            result["provoking"]["last"],
            serde_json::json!([0, 0, 255, 255])
        );
        assert_eq!(
            result["provoking"]["first"],
            serde_json::json!([255, 0, 0, 255])
        );
        assert_eq!(result["provoking"]["invalid"], 0x0500);
        assert_eq!(result["provoking"]["stable"], true);
        assert_eq!(
            result["provoking"]["native"],
            "function provokingVertexWEBGL() { [native code] }"
        );
    }
    assert_eq!(result["error"], 0);
}
