use std::sync::Arc;

use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

#[test]
fn webgl2_transform_feedback_captures_shader_varyings() {
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
                    out vec2 captured;
                    void main() {
                        float index = float(gl_VertexID);
                        captured = vec2(index + 1.0, index + 10.0);
                        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    out vec4 outputColor;
                    void main() { outputColor = vec4(1.0); }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.transformFeedbackVaryings(
                    program,
                    ["captured"],
                    context.INTERLEAVED_ATTRIBS,
                );
                context.linkProgram(program);
                context.useProgram(program);
                const varying = context.getTransformFeedbackVarying(program, 0);

                const buffer = context.createBuffer();
                context.bindBuffer(context.TRANSFORM_FEEDBACK_BUFFER, buffer);
                context.bufferData(
                    context.TRANSFORM_FEEDBACK_BUFFER,
                    new Float32Array(6),
                    context.DYNAMIC_COPY,
                );
                const feedback = context.createTransformFeedback();
                context.bindTransformFeedback(context.TRANSFORM_FEEDBACK, feedback);
                context.bindBufferRange(
                    context.TRANSFORM_FEEDBACK_BUFFER,
                    0,
                    buffer,
                    0,
                    24,
                );
                const indexed = context.getIndexedParameter(
                        context.TRANSFORM_FEEDBACK_BUFFER_BINDING,
                        0,
                    ) === buffer
                    && context.getIndexedParameter(context.TRANSFORM_FEEDBACK_BUFFER_START, 0) === 0
                    && context.getIndexedParameter(context.TRANSFORM_FEEDBACK_BUFFER_SIZE, 0) === 24;
                const bound = context.getParameter(context.TRANSFORM_FEEDBACK_BINDING) === feedback;
                context.enable(context.RASTERIZER_DISCARD);
                context.beginTransformFeedback(context.POINTS);
                const active = context.getParameter(context.TRANSFORM_FEEDBACK_ACTIVE)
                    && !context.getParameter(context.TRANSFORM_FEEDBACK_PAUSED);
                context.drawArrays(context.POINTS, 0, 1);
                context.pauseTransformFeedback();
                const paused = context.getParameter(context.TRANSFORM_FEEDBACK_ACTIVE)
                    && context.getParameter(context.TRANSFORM_FEEDBACK_PAUSED);
                context.drawArrays(context.POINTS, 1, 1);
                context.resumeTransformFeedback();
                context.drawArrays(context.POINTS, 2, 1);
                context.endTransformFeedback();
                const ended = !context.getParameter(context.TRANSFORM_FEEDBACK_ACTIVE)
                    && !context.getParameter(context.TRANSFORM_FEEDBACK_PAUSED);
                context.disable(context.RASTERIZER_DISCARD);

                context.bindBuffer(context.TRANSFORM_FEEDBACK_BUFFER, buffer);
                const captured = new Float32Array(6);
                context.getBufferSubData(context.TRANSFORM_FEEDBACK_BUFFER, 0, captured);
                const object = feedback instanceof WebGLTransformFeedback
                    && context.isTransformFeedback(feedback);
                const info = varying instanceof WebGLActiveInfo
                    && varying.name === "captured"
                    && varying.size === 1
                    && varying.type === context.FLOAT_VEC2;
                const native = [
                    context.transformFeedbackVaryings.toString(),
                    context.beginTransformFeedback.toString(),
                    context.getBufferSubData.toString(),
                ];
                context.bindTransformFeedback(context.TRANSFORM_FEEDBACK, null);
                context.deleteTransformFeedback(feedback);
                return JSON.stringify({
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    indexed,
                    bound,
                    active,
                    paused,
                    ended,
                    object,
                    info,
                    captured: [...captured],
                    deleted: !context.isTransformFeedback(feedback),
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
            "compiled", "linked", "indexed", "bound", "active", "paused", "ended", "object",
            "info", "deleted",
        ] {
            assert_eq!(
                result[name], true,
                "failed transform feedback check: {name}"
            );
        }
        assert_eq!(result["captured"], serde_json::json!([1, 10, 3, 12, 0, 0]));
        assert_eq!(result["error"], 0);
        assert_eq!(
            result["native"],
            serde_json::json!([
                "function transformFeedbackVaryings() { [native code] }",
                "function beginTransformFeedback() { [native code] }",
                "function getBufferSubData() { [native code] }",
            ])
        );
    }
}

#[test]
fn webgl2_shader_inputs_uniform_blocks_and_range_drawing_use_angle() {
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
                const methodNames = [
                    "uniform1ui", "uniform4uiv", "uniformMatrix2x3fv",
                    "vertexAttribIPointer", "vertexAttribI4i", "vertexAttribI4iv",
                    "vertexAttribI4ui", "vertexAttribI4uiv", "drawRangeElements",
                    "getUniformIndices", "getActiveUniforms", "getUniformBlockIndex",
                    "getActiveUniformBlockParameter", "getActiveUniformBlockName",
                    "uniformBlockBinding", "getActiveAttrib", "getActiveUniform",
                    "getAttachedShaders", "getUniform", "getShaderSource",
                    "getShaderPrecisionFormat",
                ];
                const api = methodNames.every(name =>
                    typeof WebGL2RenderingContext.prototype[name] === "function"
                    && WebGL2RenderingContext.prototype[name].toString().includes("[native code]"));
                const constants = WebGL2RenderingContext.prototype.UNIFORM_BUFFER === 0x8A11
                    && WebGL2RenderingContext.prototype.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES === 0x8A43
                    && WebGL2RenderingContext.prototype.INVALID_INDEX === 0xFFFFFFFF
                    && WebGLRenderingContext.prototype.UNIFORM_BUFFER === undefined;
                if (!context) return JSON.stringify({ available: false, api, constants });

                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return shader;
                };
                const vertex = compile(context.VERTEX_SHADER, `#version 300 es
                    layout(location = 0) in vec2 position;
                    layout(location = 1) in uvec4 vertexColor;
                    uniform mat2x3 transform;
                    flat out uvec4 color;
                    void main() {
                        vec3 transformed = transform * position;
                        gl_Position = vec4(transformed.xy, 0.0, 1.0);
                        color = vertexColor;
                    }`);
                const fragment = compile(context.FRAGMENT_SHADER, `#version 300 es
                    precision highp float;
                    precision highp int;
                    layout(std140) uniform Params { uvec4 tint; };
                    uniform uvec4 bias;
                    uniform float scale;
                    uniform ivec2 signedBias;
                    uniform mat2 square;
                    flat in uvec4 color;
                    out vec4 outputColor;
                    void main() {
                        float noEffect = float(signedBias.x) + square[0][0] - 1.0;
                        outputColor = vec4(tint + bias + color) / 255.0 * scale + vec4(noEffect);
                    }`);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);
                context.useProgram(program);
                const attached = context.getAttachedShaders(program);
                const precision = context.getShaderPrecisionFormat(
                    context.FRAGMENT_SHADER, context.HIGH_FLOAT);

                const positions = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, positions);
                context.bufferData(context.ARRAY_BUFFER,
                    new Float32Array([-1, -1, 3, -1, -1, 3]), context.STATIC_DRAW);
                context.enableVertexAttribArray(0);
                context.vertexAttribPointer(0, 2, context.FLOAT, false, 0, 0);

                const colors = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, colors);
                context.bufferData(context.ARRAY_BUFFER, new Uint32Array([
                    1, 2, 3, 5, 1, 2, 3, 5, 1, 2, 3, 5,
                ]), context.STATIC_DRAW);
                context.enableVertexAttribArray(1);
                context.vertexAttribIPointer(1, 4, context.UNSIGNED_INT, 0, 0);

                context.vertexAttribI4i(2, -1, -2, -3, -4);
                context.vertexAttribI4iv(2, new Int32Array([99, -5, -6, -7, -8]).subarray(1));
                context.vertexAttribI4ui(3, 1, 2, 3, 4);
                context.vertexAttribI4uiv(3, new Uint32Array([99, 5, 6, 7, 8]).subarray(1));

                const elements = context.createBuffer();
                context.bindBuffer(context.ELEMENT_ARRAY_BUFFER, elements);
                context.bufferData(context.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 1, 2]), context.STATIC_DRAW);

                const matrixValues = new Float32Array([99, 1, 0, 0, 0, 1, 0, 99]);
                context.uniformMatrix2x3fv(
                    context.getUniformLocation(program, "transform"), false, matrixValues, 1, 6);
                const biasValues = new Uint32Array([99, 10, 20, 30, 50, 99]);
                const biasLocation = context.getUniformLocation(program, "bias");
                const transformLocation = context.getUniformLocation(program, "transform");
                const scaleLocation = context.getUniformLocation(program, "scale");
                const signedLocation = context.getUniformLocation(program, "signedBias");
                const squareLocation = context.getUniformLocation(program, "square");
                context.uniform4uiv(biasLocation, biasValues, 1, 4);
                context.uniform1f(scaleLocation, 1);
                context.uniform2iv(signedLocation, new Int32Array([99, 0, 7, 99]), 1, 2);
                context.uniformMatrix2fv(
                    squareLocation, false, new Float32Array([99, 1, 0, 0, 1, 99]), 1, 4);

                const uniformBuffer = context.createBuffer();
                context.bindBuffer(context.UNIFORM_BUFFER, uniformBuffer);
                context.bufferData(context.UNIFORM_BUFFER,
                    new Uint32Array([20, 30, 40, 200]), context.STATIC_DRAW);
                const blockIndex = context.getUniformBlockIndex(program, "Params");
                context.uniformBlockBinding(program, blockIndex, 2);
                context.bindBufferBase(context.UNIFORM_BUFFER, 2, uniformBuffer);

                const names = [
                    "tint", "bias", "transform", "scale", "signedBias", "square", "missing",
                ];
                const indices = context.getUniformIndices(program, names);
                const activeIndices = indices.slice(0, 6);
                const types = context.getActiveUniforms(program, activeIndices, context.UNIFORM_TYPE);
                const sizes = context.getActiveUniforms(program, activeIndices, context.UNIFORM_SIZE);
                const blockIndices = context.getActiveUniforms(
                    program, activeIndices, context.UNIFORM_BLOCK_INDEX);
                const rowMajor = context.getActiveUniforms(
                    program, activeIndices, context.UNIFORM_IS_ROW_MAJOR);
                const blockActiveIndices = context.getActiveUniformBlockParameter(
                    program, blockIndex, context.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES);
                const reflection = {
                    indices,
                    types,
                    sizes,
                    blockIndices,
                    rowMajor,
                    blockName: context.getActiveUniformBlockName(program, blockIndex),
                    binding: context.getActiveUniformBlockParameter(
                        program, blockIndex, context.UNIFORM_BLOCK_BINDING),
                    byteSize: context.getActiveUniformBlockParameter(
                        program, blockIndex, context.UNIFORM_BLOCK_DATA_SIZE),
                    activeCount: context.getActiveUniformBlockParameter(
                        program, blockIndex, context.UNIFORM_BLOCK_ACTIVE_UNIFORMS),
                    activeIndicesType: blockActiveIndices instanceof Uint32Array,
                    activeIndices: [...blockActiveIndices],
                    vertexReference: context.getActiveUniformBlockParameter(
                        program, blockIndex, context.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER),
                    fragmentReference: context.getActiveUniformBlockParameter(
                        program, blockIndex, context.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER),
                    generalBinding: context.getParameter(context.UNIFORM_BUFFER_BINDING) === uniformBuffer,
                    indexedBinding: context.getIndexedParameter(context.UNIFORM_BUFFER_BINDING, 2)
                        === uniformBuffer,
                    programCounts: {
                        attached: context.getProgramParameter(program, context.ATTACHED_SHADERS),
                        attributes: context.getProgramParameter(program, context.ACTIVE_ATTRIBUTES),
                        uniforms: context.getProgramParameter(program, context.ACTIVE_UNIFORMS),
                        blocks: context.getProgramParameter(program, context.ACTIVE_UNIFORM_BLOCKS),
                    },
                    attributes: Array.from(
                        { length: context.getProgramParameter(program, context.ACTIVE_ATTRIBUTES) },
                        (_, index) => {
                            const info = context.getActiveAttrib(program, index);
                            return { name: info.name, size: info.size, type: info.type };
                        }),
                    uniforms: Array.from(
                        { length: context.getProgramParameter(program, context.ACTIVE_UNIFORMS) },
                        (_, index) => {
                            const info = context.getActiveUniform(program, index);
                            return { name: info.name, size: info.size, type: info.type };
                        }),
                    biasValue: context.getUniform(program, biasLocation) instanceof Uint32Array
                        ? [...context.getUniform(program, biasLocation)] : null,
                    transformValue: context.getUniform(program, transformLocation) instanceof Float32Array
                        ? [...context.getUniform(program, transformLocation)] : null,
                    scaleValue: context.getUniform(program, scaleLocation),
                    signedValue: context.getUniform(program, signedLocation) instanceof Int32Array
                        ? [...context.getUniform(program, signedLocation)] : null,
                    squareValue: context.getUniform(program, squareLocation) instanceof Float32Array
                        ? [...context.getUniform(program, squareLocation)] : null,
                    attachedIdentity: attached.length === 2
                        && attached.includes(vertex) && attached.includes(fragment),
                    shaderSource: context.getShaderSource(fragment).includes("uniform float scale"),
                    precision: precision instanceof WebGLShaderPrecisionFormat
                        && Number.isInteger(precision.rangeMin)
                        && Number.isInteger(precision.rangeMax)
                        && precision.precision > 0,
                };

                context.viewport(0, 0, 4, 4);
                context.drawRangeElements(context.TRIANGLES, 0, 2, 3, context.UNSIGNED_SHORT, 0);
                const pixel = new Uint8Array(4);
                context.readPixels(2, 2, 1, 1, context.RGBA, context.UNSIGNED_BYTE, pixel);
                return JSON.stringify({
                    available: true,
                    api,
                    constants,
                    compiled: context.getShaderParameter(vertex, context.COMPILE_STATUS)
                        && context.getShaderParameter(fragment, context.COMPILE_STATUS),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    reflection,
                    pixel: [...pixel],
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    assert_eq!(result["constants"], true, "{result}");
    if result["available"] == true {
        assert_eq!(result["compiled"], true);
        assert_eq!(result["linked"], true);
        assert_eq!(result["reflection"]["indices"][6], 0xFFFF_FFFF_u32);
        assert_eq!(
            result["reflection"]["sizes"],
            serde_json::json!([1, 1, 1, 1, 1, 1])
        );
        assert_eq!(result["reflection"]["blockIndices"][0], 0);
        assert_eq!(result["reflection"]["blockIndices"][1], -1);
        assert_eq!(result["reflection"]["blockIndices"][2], -1);
        assert_eq!(result["reflection"]["blockIndices"][3], -1);
        assert_eq!(result["reflection"]["blockIndices"][4], -1);
        assert_eq!(result["reflection"]["blockIndices"][5], -1);
        assert_eq!(
            result["reflection"]["rowMajor"],
            serde_json::json!([false, false, false, false, false, false])
        );
        assert_eq!(result["reflection"]["blockName"], "Params");
        assert_eq!(result["reflection"]["binding"], 2);
        assert_eq!(result["reflection"]["byteSize"], 16);
        assert_eq!(result["reflection"]["activeCount"], 1);
        assert_eq!(result["reflection"]["activeIndicesType"], true);
        assert_eq!(
            result["reflection"]["activeIndices"][0],
            result["reflection"]["indices"][0]
        );
        assert_eq!(result["reflection"]["vertexReference"], false);
        assert_eq!(result["reflection"]["fragmentReference"], true);
        assert_eq!(result["reflection"]["generalBinding"], true);
        assert_eq!(result["reflection"]["indexedBinding"], true);
        assert_eq!(
            result["reflection"]["programCounts"],
            serde_json::json!({ "attached": 2, "attributes": 2, "uniforms": 6, "blocks": 1 })
        );
        assert_eq!(
            result["reflection"]["attributes"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            result["reflection"]["uniforms"].as_array().unwrap().len(),
            6
        );
        assert_eq!(
            result["reflection"]["biasValue"],
            serde_json::json!([10, 20, 30, 50])
        );
        assert_eq!(
            result["reflection"]["transformValue"],
            serde_json::json!([1, 0, 0, 0, 1, 0])
        );
        assert_eq!(result["reflection"]["scaleValue"], 1.0);
        assert_eq!(
            result["reflection"]["signedValue"],
            serde_json::json!([0, 7])
        );
        assert_eq!(
            result["reflection"]["squareValue"],
            serde_json::json!([1, 0, 0, 1])
        );
        assert_eq!(result["reflection"]["attachedIdentity"], true);
        assert_eq!(result["reflection"]["shaderSource"], true);
        assert_eq!(result["reflection"]["precision"], true);
        assert_eq!(result["pixel"], serde_json::json!([31, 52, 73, 255]));
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl2_shader_extension_family_compiles_and_links_through_angle() {
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
                const names = [
                    "EXT_conservative_depth",
                    "NV_shader_noperspective_interpolation",
                    "OES_sample_variables",
                    "OES_shader_multisample_interpolation",
                ];
                const webgl1Absent = webgl1 === null
                    || names.every(name => webgl1.getExtension(name) === null);
                const compile = (type, source) => {
                    const shader = context.createShader(type);
                    context.shaderSource(shader, source);
                    context.compileShader(shader);
                    return {
                        shader,
                        compiled: context.getShaderParameter(shader, context.COMPILE_STATUS),
                        log: context.getShaderInfoLog(shader),
                    };
                };
                const link = (vertexSource, fragmentSource) => {
                    const vertex = compile(context.VERTEX_SHADER, vertexSource);
                    const fragment = compile(context.FRAGMENT_SHADER, fragmentSource);
                    const program = context.createProgram();
                    context.attachShader(program, vertex.shader);
                    context.attachShader(program, fragment.shader);
                    context.linkProgram(program);
                    return {
                        vertex: { compiled: vertex.compiled, log: vertex.log },
                        fragment: { compiled: fragment.compiled, log: fragment.log },
                        linked: context.getProgramParameter(program, context.LINK_STATUS),
                        log: context.getProgramInfoLog(program),
                    };
                };
                const baseVertex = `#version 300 es
                    in vec2 position;
                    void main() { gl_Position = vec4(position, 0.0, 1.0); }`;
                const extensions = Object.fromEntries(names.map(name => {
                    const extension = context.getExtension(name);
                    return [name, extension === null ? null : {
                        stable: extension === context.getExtension(name.toLowerCase()),
                        frozen: Object.isFrozen(extension),
                    }];
                }));
                const shaders = {};
                if (extensions.EXT_conservative_depth) {
                    shaders.conservativeDepth = link(baseVertex, `#version 300 es
                        #extension GL_EXT_conservative_depth : require
                        precision highp float;
                        layout(depth_greater) out highp float gl_FragDepth;
                        out vec4 color;
                        void main() {
                            gl_FragDepth = gl_FragCoord.z;
                            color = vec4(1.0);
                        }`);
                }
                if (extensions.NV_shader_noperspective_interpolation) {
                    shaders.noperspective = link(`#version 300 es
                        #extension GL_NV_shader_noperspective_interpolation : require
                        in vec2 position;
                        noperspective out vec2 value;
                        void main() {
                            value = position;
                            gl_Position = vec4(position, 0.0, 1.0);
                        }`, `#version 300 es
                        #extension GL_NV_shader_noperspective_interpolation : require
                        precision highp float;
                        noperspective in vec2 value;
                        out vec4 color;
                        void main() { color = vec4(value, 0.0, 1.0); }`);
                }
                if (extensions.OES_sample_variables) {
                    shaders.sampleVariables = link(baseVertex, `#version 300 es
                        #extension GL_OES_sample_variables : require
                        precision highp float;
                        out vec4 color;
                        void main() {
                            color = vec4(float(gl_SampleID >= 0), gl_SamplePosition,
                                float(gl_NumSamples >= 1));
                            gl_SampleMask[0] = gl_SampleMaskIn[0];
                        }`);
                }
                let interpolation = null;
                if (extensions.OES_shader_multisample_interpolation) {
                    const extension = context.getExtension("OES_shader_multisample_interpolation");
                    shaders.multisampleInterpolation = link(`#version 300 es
                        #extension GL_OES_shader_multisample_interpolation : require
                        in vec2 position;
                        sample out vec2 value;
                        void main() {
                            value = position;
                            gl_Position = vec4(position, 0.0, 1.0);
                        }`, `#version 300 es
                        #extension GL_OES_shader_multisample_interpolation : require
                        precision highp float;
                        sample in vec2 value;
                        out vec4 color;
                        void main() {
                            vec2 interpolated = interpolateAtCentroid(value)
                                + interpolateAtSample(value, 0)
                                + interpolateAtOffset(value, vec2(0.0));
                            color = vec4(interpolated, 0.0, 1.0);
                        }`);
                    interpolation = {
                        constants: [
                            extension.MIN_FRAGMENT_INTERPOLATION_OFFSET_OES,
                            extension.MAX_FRAGMENT_INTERPOLATION_OFFSET_OES,
                            extension.FRAGMENT_INTERPOLATION_OFFSET_BITS_OES,
                        ],
                        values: [
                            context.getParameter(extension.MIN_FRAGMENT_INTERPOLATION_OFFSET_OES),
                            context.getParameter(extension.MAX_FRAGMENT_INTERPOLATION_OFFSET_OES),
                            context.getParameter(extension.FRAGMENT_INTERPOLATION_OFFSET_BITS_OES),
                        ],
                    };
                }
                return JSON.stringify({
                    available: true,
                    webgl1Absent,
                    extensions,
                    shaders,
                    interpolation,
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
    for extension in result["extensions"].as_object().unwrap().values() {
        if extension.is_null() {
            continue;
        }
        assert_eq!(extension["stable"], true);
        assert_eq!(extension["frozen"], true);
    }
    for (name, shader) in result["shaders"].as_object().unwrap() {
        assert_eq!(shader["vertex"]["compiled"], true, "{name}: {shader}");
        assert_eq!(shader["fragment"]["compiled"], true, "{name}: {shader}");
        assert_eq!(shader["linked"], true, "{name}: {shader}");
    }
    if !result["interpolation"].is_null() {
        assert_eq!(
            result["interpolation"]["constants"],
            serde_json::json!([0x8E5B, 0x8E5C, 0x8E5D])
        );
        assert!(result["interpolation"]["values"][0].as_f64().is_some());
        assert!(result["interpolation"]["values"][1].as_f64().is_some());
        assert!(result["interpolation"]["values"][2].as_i64().is_some());
    }
    assert_eq!(result["error"], 0);
}

#[test]
fn webgl2_buffer_source_ranges_copy_and_reflection_use_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const context = document.createElement("canvas").getContext("webgl2");
                const api = ["copyBufferSubData", "getBufferSubData", "getBufferParameter"]
                    .every(name => typeof WebGL2RenderingContext.prototype[name] === "function"
                        && WebGL2RenderingContext.prototype[name].toString().includes("[native code]"));
                const constants = WebGL2RenderingContext.prototype.COPY_READ_BUFFER === 0x8F36
                    && WebGL2RenderingContext.prototype.COPY_WRITE_BUFFER === 0x8F37
                    && WebGL2RenderingContext.prototype.BUFFER_SIZE === 0x8764
                    && WebGLRenderingContext.prototype.COPY_READ_BUFFER === undefined;
                if (!context) return JSON.stringify({ available: false, api, constants });

                const source = context.createBuffer();
                context.bindBuffer(context.COPY_READ_BUFFER, source);
                context.bufferData(context.COPY_READ_BUFFER,
                    new Uint16Array([999, 10, 20, 30, 40, 999]), context.STATIC_READ, 1, 4);
                context.bufferSubData(context.COPY_READ_BUFFER, 2,
                    new Uint16Array([999, 77, 88, 999]), 1, 2);

                const destination = context.createBuffer();
                context.bindBuffer(context.COPY_WRITE_BUFFER, destination);
                context.bufferData(context.COPY_WRITE_BUFFER, 8, context.DYNAMIC_COPY);
                context.copyBufferSubData(
                    context.COPY_READ_BUFFER, context.COPY_WRITE_BUFFER, 0, 0, 8);

                const copied = new Uint16Array(6);
                context.getBufferSubData(context.COPY_WRITE_BUFFER, 0, copied, 1, 4);
                return JSON.stringify({
                    available: true,
                    api,
                    constants,
                    copied: [...copied],
                    sourceSize: (() => {
                        context.bindBuffer(context.COPY_READ_BUFFER, source);
                        return context.getBufferParameter(context.COPY_READ_BUFFER, context.BUFFER_SIZE);
                    })(),
                    sourceUsage: context.getBufferParameter(
                        context.COPY_READ_BUFFER, context.BUFFER_USAGE),
                    destinationSize: context.getBufferParameter(
                        context.COPY_WRITE_BUFFER, context.BUFFER_SIZE),
                    destinationUsage: context.getBufferParameter(
                        context.COPY_WRITE_BUFFER, context.BUFFER_USAGE),
                    bindings: context.getParameter(context.COPY_READ_BUFFER_BINDING) === source
                        && context.getParameter(context.COPY_WRITE_BUFFER_BINDING) === destination,
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    assert_eq!(result["constants"], true, "{result}");
    if result["available"] == true {
        assert_eq!(result["copied"], serde_json::json!([0, 10, 77, 88, 40, 0]));
        assert_eq!(result["sourceSize"], 8);
        assert_eq!(result["sourceUsage"], 0x88E5);
        assert_eq!(result["destinationSize"], 8);
        assert_eq!(result["destinationUsage"], 0x88EA);
        assert_eq!(result["bindings"], true);
        assert_eq!(result["error"], 0);
    }
}

#[test]
fn webgl_core_object_texture_and_attribute_reflection_uses_angle() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webgl(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"(() => {
                const context = document.createElement("canvas").getContext("webgl2");
                const methodNames = [
                    "depthRange", "flush", "finish", "texParameterf", "getTexParameter",
                    "getRenderbufferParameter", "getFramebufferAttachmentParameter",
                    "vertexAttrib1f", "vertexAttrib2f", "vertexAttrib3f", "vertexAttrib4f",
                    "vertexAttrib1fv", "vertexAttrib2fv", "vertexAttrib3fv", "vertexAttrib4fv",
                    "getVertexAttrib", "getVertexAttribOffset", "getFragDataLocation",
                ];
                const api = methodNames.every(name =>
                    typeof WebGL2RenderingContext.prototype[name] === "function"
                    && WebGL2RenderingContext.prototype[name].toString().includes("[native code]"));
                if (!context) return JSON.stringify({ available: false, api });

                context.depthRange(0.2, 0.8);

                const texture = context.createTexture();
                context.bindTexture(context.TEXTURE_2D, texture);
                context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_S, context.REPEAT);
                context.texParameterf(context.TEXTURE_2D, context.TEXTURE_MAX_LOD, 2.5);
                context.texImage2D(context.TEXTURE_2D, 0, context.RGBA8, 1, 1, 0,
                    context.RGBA, context.UNSIGNED_BYTE, null);

                const renderbuffer = context.createRenderbuffer();
                context.bindRenderbuffer(context.RENDERBUFFER, renderbuffer);
                context.renderbufferStorage(context.RENDERBUFFER, context.RGBA8, 3, 2);

                const framebuffer = context.createFramebuffer();
                context.bindFramebuffer(context.FRAMEBUFFER, framebuffer);
                context.framebufferTexture2D(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.TEXTURE_2D, texture, 0);
                const textureAttachment = {
                    type: context.getFramebufferAttachmentParameter(context.FRAMEBUFFER,
                        context.COLOR_ATTACHMENT0, context.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE),
                    identity: context.getFramebufferAttachmentParameter(context.FRAMEBUFFER,
                        context.COLOR_ATTACHMENT0, context.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) === texture,
                    level: context.getFramebufferAttachmentParameter(context.FRAMEBUFFER,
                        context.COLOR_ATTACHMENT0, context.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL),
                };
                context.framebufferRenderbuffer(context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.RENDERBUFFER, renderbuffer);
                const renderbufferIdentity = context.getFramebufferAttachmentParameter(
                    context.FRAMEBUFFER, context.COLOR_ATTACHMENT0,
                    context.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) === renderbuffer;

                const vertexBuffer = context.createBuffer();
                context.bindBuffer(context.ARRAY_BUFFER, vertexBuffer);
                context.bufferData(context.ARRAY_BUFFER, new Float32Array(16), context.STATIC_DRAW);
                context.vertexAttribPointer(2, 3, context.FLOAT, false, 16, 4);
                context.enableVertexAttribArray(2);
                context.vertexAttribDivisor(2, 7);
                const floatAttribute = {
                    enabled: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_ENABLED),
                    size: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_SIZE),
                    stride: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_STRIDE),
                    type: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_TYPE),
                    normalized: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_NORMALIZED),
                    integer: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_INTEGER),
                    divisor: context.getVertexAttrib(2, context.VERTEX_ATTRIB_ARRAY_DIVISOR),
                    buffer: context.getVertexAttrib(2,
                        context.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING) === vertexBuffer,
                    offset: context.getVertexAttribOffset(2, context.VERTEX_ATTRIB_ARRAY_POINTER),
                };
                context.vertexAttrib3fv(3, new Float32Array([99, 1, 2, 3]).subarray(1));
                context.vertexAttribI4uiv(4, new Uint32Array([99, 4, 5, 6, 7]).subarray(1));
                context.vertexAttribIPointer(5, 4, context.UNSIGNED_INT, 0, 0);
                const currentFloat = context.getVertexAttrib(3, context.CURRENT_VERTEX_ATTRIB);
                const currentUint = context.getVertexAttrib(4, context.CURRENT_VERTEX_ATTRIB);

                const vertex = context.createShader(context.VERTEX_SHADER);
                context.shaderSource(vertex, `#version 300 es
                    void main() { gl_Position = vec4(0.0); }`);
                context.compileShader(vertex);
                const fragment = context.createShader(context.FRAGMENT_SHADER);
                context.shaderSource(fragment, `#version 300 es
                    precision highp float;
                    layout(location = 0) out vec4 fragmentColor;
                    void main() { fragmentColor = vec4(1.0); }`);
                context.compileShader(fragment);
                const program = context.createProgram();
                context.attachShader(program, vertex);
                context.attachShader(program, fragment);
                context.linkProgram(program);

                context.flush();
                context.finish();
                return JSON.stringify({
                    available: true,
                    api,
                    depthRange: [...context.getParameter(context.DEPTH_RANGE)],
                    texture: {
                        wrapS: context.getTexParameter(context.TEXTURE_2D, context.TEXTURE_WRAP_S),
                        maxLod: context.getTexParameter(context.TEXTURE_2D, context.TEXTURE_MAX_LOD),
                    },
                    renderbuffer: {
                        width: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_WIDTH),
                        height: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_HEIGHT),
                        format: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_INTERNAL_FORMAT),
                        samples: context.getRenderbufferParameter(
                            context.RENDERBUFFER, context.RENDERBUFFER_SAMPLES),
                    },
                    textureAttachment,
                    renderbufferIdentity,
                    floatAttribute,
                    currentFloatType: currentFloat instanceof Float32Array,
                    currentFloat: [...currentFloat],
                    currentUintType: currentUint instanceof Uint32Array,
                    currentUint: [...currentUint],
                    integerPointer: context.getVertexAttrib(
                        5, context.VERTEX_ATTRIB_ARRAY_INTEGER),
                    fragmentLocation: context.getFragDataLocation(program, "fragmentColor"),
                    linked: context.getProgramParameter(program, context.LINK_STATUS),
                    error: context.getError(),
                });
            })()"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["api"], true);
    if result["available"] == true {
        assert!((result["depthRange"][0].as_f64().unwrap() - 0.2).abs() < 0.0001);
        assert!((result["depthRange"][1].as_f64().unwrap() - 0.8).abs() < 0.0001);
        assert_eq!(result["texture"]["wrapS"], 0x2901);
        assert_eq!(result["texture"]["maxLod"], 2.5);
        assert_eq!(result["renderbuffer"]["width"], 3);
        assert_eq!(result["renderbuffer"]["height"], 2);
        assert_eq!(result["renderbuffer"]["format"], 0x8058);
        assert_eq!(result["renderbuffer"]["samples"], 0);
        assert_eq!(result["textureAttachment"]["type"], 0x1702);
        assert_eq!(result["textureAttachment"]["identity"], true);
        assert_eq!(result["textureAttachment"]["level"], 0);
        assert_eq!(result["renderbufferIdentity"], true);
        assert_eq!(
            result["floatAttribute"],
            serde_json::json!({
                "enabled": true, "size": 3, "stride": 16, "type": 0x1406,
                "normalized": false, "integer": false, "divisor": 7,
                "buffer": true, "offset": 4,
            })
        );
        assert_eq!(result["currentFloatType"], true);
        assert_eq!(result["currentFloat"], serde_json::json!([1, 2, 3, 1]));
        assert_eq!(result["currentUintType"], true);
        assert_eq!(result["currentUint"], serde_json::json!([4, 5, 6, 7]));
        assert_eq!(result["integerPointer"], true);
        assert_eq!(result["fragmentLocation"], 0);
        assert_eq!(result["linked"], true);
        assert_eq!(result["error"], 0);
    }
}
