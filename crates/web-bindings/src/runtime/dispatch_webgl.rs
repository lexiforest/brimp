use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "webglAcquire" => {
            if !state.features.webgl {
                return Ok(NativeValue::Boolean(false));
            }
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let version = required_u32(call, 4, "WebGL version")?.clamp(1, 2) as u8;
            if !state
                .canvases
                .borrow_mut()
                .can_acquire_webgl(id, width, height, version)
                .map_err(NativeError::new)?
            {
                return Ok(NativeValue::Boolean(false));
            }
            let acquired = state
                .angles
                .borrow_mut()
                .create(id, width, height, version)
                .map_err(NativeError::new)?;
            if acquired {
                state
                    .canvases
                    .borrow_mut()
                    .commit_webgl(id, version)
                    .map_err(NativeError::new)?;
            }
            Ok(NativeValue::Boolean(acquired))
        }
        "webglSupportedExtensions" => {
            let id = required_canvas_target(state, call)?;
            let extensions = state
                .angles
                .borrow()
                .supported_webgl_extensions(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&extensions).expect("WebGL extension names encode as JSON"),
            ))
        }
        "webglLoseContext" => {
            let id = required_canvas_target(state, call)?;
            let lost = state
                .angles
                .borrow_mut()
                .lose_context(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(lost))
        }
        "webglRestoreContext" => {
            let id = required_canvas_target(state, call)?;
            let (width, height) = canvas_dimensions(call)?;
            let version = required_u32(call, 4, "WebGL version")?.clamp(1, 2) as u8;
            let restored = state
                .angles
                .borrow_mut()
                .create(id, width, height, version)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(restored))
        }
        "webglClearColor" => {
            let id = required_canvas_target(state, call)?;
            let color = required_numbers::<4>(call, 2, "clear color")?.map(|value| value as f32);
            state
                .angles
                .borrow()
                .clear_color(id, color)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClear" => {
            let id = required_canvas_target(state, call)?;
            let mask = required_u32(call, 2, "clear mask")?;
            state
                .angles
                .borrow()
                .clear(id, mask)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClearDepth" => {
            let id = required_canvas_target(state, call)?;
            let depth = required_number(call, 2, "clear depth")? as f32;
            state
                .angles
                .borrow()
                .clear_depth(id, depth)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClearStencil" => {
            let id = required_canvas_target(state, call)?;
            let stencil = required_i32(call, 2, "clear stencil")?;
            state
                .angles
                .borrow()
                .clear_stencil(id, stencil)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglSetEnabled" => {
            let id = required_canvas_target(state, call)?;
            let capability = required_u32(call, 2, "WebGL capability")?;
            let enabled = required_boolean(call, 3, "capability state")?;
            state
                .angles
                .borrow()
                .set_enabled(id, capability, enabled)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglSetEnabledIndexed" => {
            let id = required_canvas_target(state, call)?;
            let capability = required_u32(call, 2, "WebGL capability")?;
            let index = required_u32(call, 3, "draw buffer index")?;
            let enabled = required_boolean(call, 4, "capability state")?;
            state
                .angles
                .borrow_mut()
                .set_enabled_indexed(id, capability, index, enabled)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglIsEnabled" => {
            let id = required_canvas_target(state, call)?;
            let capability = required_u32(call, 2, "WebGL capability")?;
            let enabled = state
                .angles
                .borrow()
                .is_enabled(id, capability)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(enabled))
        }
        "webglScissor" => {
            let id = required_canvas_target(state, call)?;
            let values = required_numbers::<4>(call, 2, "scissor box")?.map(|value| value as i32);
            state
                .angles
                .borrow()
                .scissor(id, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglColorMask" => {
            let id = required_canvas_target(state, call)?;
            let values = [
                required_boolean(call, 2, "red color mask")?,
                required_boolean(call, 3, "green color mask")?,
                required_boolean(call, 4, "blue color mask")?,
                required_boolean(call, 5, "alpha color mask")?,
            ];
            state
                .angles
                .borrow()
                .color_mask(id, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglColorMaskIndexed" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let values = [
                required_boolean(call, 3, "red color mask")?,
                required_boolean(call, 4, "green color mask")?,
                required_boolean(call, 5, "blue color mask")?,
                required_boolean(call, 6, "alpha color mask")?,
            ];
            state
                .angles
                .borrow_mut()
                .color_mask_indexed(id, index, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDepthMask" => {
            let id = required_canvas_target(state, call)?;
            let value = required_boolean(call, 2, "depth mask")?;
            state
                .angles
                .borrow()
                .depth_mask(id, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDepthFunc" => {
            let id = required_canvas_target(state, call)?;
            let value = required_u32(call, 2, "depth function")?;
            state
                .angles
                .borrow()
                .depth_func(id, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDepthRange" => {
            let id = required_canvas_target(state, call)?;
            let near = required_number(call, 2, "near depth range")? as f32;
            let far = required_number(call, 3, "far depth range")? as f32;
            state
                .angles
                .borrow()
                .depth_range(id, near, far)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendFunc" => {
            let id = required_canvas_target(state, call)?;
            let source = required_u32(call, 2, "source blend factor")?;
            let destination = required_u32(call, 3, "destination blend factor")?;
            state
                .angles
                .borrow()
                .blend_func(id, source, destination)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendColor" => {
            let id = required_canvas_target(state, call)?;
            let color = required_numbers::<4>(call, 2, "blend color")?.map(|value| value as f32);
            state
                .angles
                .borrow()
                .blend_color(id, color)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendFuncSeparate" => {
            let id = required_canvas_target(state, call)?;
            let source_rgb = required_u32(call, 2, "source RGB blend factor")?;
            let destination_rgb = required_u32(call, 3, "destination RGB blend factor")?;
            let source_alpha = required_u32(call, 4, "source alpha blend factor")?;
            let destination_alpha = required_u32(call, 5, "destination alpha blend factor")?;
            state
                .angles
                .borrow()
                .blend_func_separate(
                    id,
                    source_rgb,
                    destination_rgb,
                    source_alpha,
                    destination_alpha,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendEquation" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "blend equation")?;
            state
                .angles
                .borrow()
                .blend_equation(id, mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendEquationSeparate" => {
            let id = required_canvas_target(state, call)?;
            let mode_rgb = required_u32(call, 2, "RGB blend equation")?;
            let mode_alpha = required_u32(call, 3, "alpha blend equation")?;
            state
                .angles
                .borrow()
                .blend_equation_separate(id, mode_rgb, mode_alpha)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendEquationIndexed" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let mode = required_u32(call, 3, "blend equation")?;
            state
                .angles
                .borrow_mut()
                .blend_equation_indexed(id, index, mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendEquationSeparateIndexed" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let mode_rgb = required_u32(call, 3, "RGB blend equation")?;
            let mode_alpha = required_u32(call, 4, "alpha blend equation")?;
            state
                .angles
                .borrow_mut()
                .blend_equation_separate_indexed(id, index, mode_rgb, mode_alpha)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendFuncIndexed" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let source = required_u32(call, 3, "source blend factor")?;
            let destination = required_u32(call, 4, "destination blend factor")?;
            state
                .angles
                .borrow_mut()
                .blend_func_indexed(id, index, source, destination)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlendFuncSeparateIndexed" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let source_rgb = required_u32(call, 3, "source RGB blend factor")?;
            let destination_rgb = required_u32(call, 4, "destination RGB blend factor")?;
            let source_alpha = required_u32(call, 5, "source alpha blend factor")?;
            let destination_alpha = required_u32(call, 6, "destination alpha blend factor")?;
            state
                .angles
                .borrow_mut()
                .blend_func_separate_indexed(
                    id,
                    index,
                    source_rgb,
                    destination_rgb,
                    source_alpha,
                    destination_alpha,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetIndexedParameterI" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "indexed parameter")?;
            let index = required_u32(call, 3, "indexed parameter index")?;
            let value = state
                .angles
                .borrow_mut()
                .indexed_parameter_i32(id, parameter, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglGetIndexedColorMask" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "draw buffer index")?;
            let value = state
                .angles
                .borrow_mut()
                .indexed_color_mask(id, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglStencilFunc" => {
            let id = required_canvas_target(state, call)?;
            let function = required_u32(call, 2, "stencil function")?;
            let reference = required_i32(call, 3, "stencil reference")?;
            let mask = required_u32(call, 4, "stencil comparison mask")?;
            state
                .angles
                .borrow()
                .stencil_func(id, function, reference, mask)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglStencilFuncSeparate" => {
            let id = required_canvas_target(state, call)?;
            let face = required_u32(call, 2, "stencil face")?;
            let function = required_u32(call, 3, "stencil function")?;
            let reference = required_i32(call, 4, "stencil reference")?;
            let mask = required_u32(call, 5, "stencil comparison mask")?;
            state
                .angles
                .borrow()
                .stencil_func_separate(id, face, function, reference, mask)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglStencilMask" => {
            let id = required_canvas_target(state, call)?;
            let mask = required_u32(call, 2, "stencil write mask")?;
            state
                .angles
                .borrow()
                .stencil_mask(id, mask)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglStencilMaskSeparate" => {
            let id = required_canvas_target(state, call)?;
            let face = required_u32(call, 2, "stencil face")?;
            let mask = required_u32(call, 3, "stencil write mask")?;
            state
                .angles
                .borrow()
                .stencil_mask_separate(id, face, mask)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglStencilOp" => {
            let id = required_canvas_target(state, call)?;
            let fail = required_u32(call, 2, "stencil fail operation")?;
            let depth_fail = required_u32(call, 3, "stencil depth-fail operation")?;
            let pass = required_u32(call, 4, "stencil pass operation")?;
            state
                .angles
                .borrow()
                .stencil_op(id, fail, depth_fail, pass)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglStencilOpSeparate" => {
            let id = required_canvas_target(state, call)?;
            let face = required_u32(call, 2, "stencil face")?;
            let fail = required_u32(call, 3, "stencil fail operation")?;
            let depth_fail = required_u32(call, 4, "stencil depth-fail operation")?;
            let pass = required_u32(call, 5, "stencil pass operation")?;
            state
                .angles
                .borrow()
                .stencil_op_separate(id, face, fail, depth_fail, pass)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglPolygonOffset" => {
            let id = required_canvas_target(state, call)?;
            let factor = required_number(call, 2, "polygon offset factor")? as f32;
            let units = required_number(call, 3, "polygon offset units")? as f32;
            state
                .angles
                .borrow()
                .polygon_offset(id, factor, units)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClipControl" => {
            let id = required_canvas_target(state, call)?;
            let origin = required_u32(call, 2, "clip-control origin")?;
            let depth = required_u32(call, 3, "clip-control depth mode")?;
            state
                .angles
                .borrow_mut()
                .clip_control(id, origin, depth)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglPolygonOffsetClamp" => {
            let id = required_canvas_target(state, call)?;
            let factor = required_number(call, 2, "polygon offset factor")? as f32;
            let units = required_number(call, 3, "polygon offset units")? as f32;
            let clamp = required_number(call, 4, "polygon offset clamp")? as f32;
            state
                .angles
                .borrow_mut()
                .polygon_offset_clamp(id, factor, units, clamp)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglProvokingVertex" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "provoking vertex mode")?;
            state
                .angles
                .borrow_mut()
                .provoking_vertex(id, mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglPolygonMode" => {
            let id = required_canvas_target(state, call)?;
            let face = required_u32(call, 2, "polygon face")?;
            let mode = required_u32(call, 3, "polygon mode")?;
            state
                .angles
                .borrow_mut()
                .polygon_mode(id, face, mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglSampleCoverage" => {
            let id = required_canvas_target(state, call)?;
            let value = required_number(call, 2, "sample coverage value")? as f32;
            let invert = required_boolean(call, 3, "sample coverage inversion")?;
            state
                .angles
                .borrow()
                .sample_coverage(id, value, invert)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCullFace" => {
            let id = required_canvas_target(state, call)?;
            let face = required_u32(call, 2, "culled face")?;
            state
                .angles
                .borrow()
                .cull_face(id, face)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFrontFace" => {
            let id = required_canvas_target(state, call)?;
            let winding = required_u32(call, 2, "front-face winding")?;
            state
                .angles
                .borrow()
                .front_face(id, winding)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglLineWidth" => {
            let id = required_canvas_target(state, call)?;
            let width = required_number(call, 2, "line width")? as f32;
            state
                .angles
                .borrow()
                .line_width(id, width)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFlush" => {
            let id = required_canvas_target(state, call)?;
            state.angles.borrow().flush(id).map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFinish" => {
            let id = required_canvas_target(state, call)?;
            state.angles.borrow().finish(id).map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetString" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_string(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(value))
        }
        "webglGetInteger" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_i32(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglGetInteger64" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_i64(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetBoolean" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_bool(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(value))
        }
        "webglGetBoolean4Mask" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_bool4_mask(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglGetFloat" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_f32(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglGetFloat2" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_f32_array::<2>(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(value.to_vec()))
        }
        "webglGetFloat4" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_f32_array::<4>(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Float32Array(value.to_vec()))
        }
        "webglGetInteger2" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_i32_array::<2>(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Int32Array(value.to_vec()))
        }
        "webglGetInteger4" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "WebGL parameter")?;
            let value = state
                .angles
                .borrow()
                .parameter_i32_array::<4>(id, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Int32Array(value.to_vec()))
        }
        "webglGetError" => {
            let id = required_canvas_target(state, call)?;
            let error = state.angles.borrow().error(id).map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(error)))
        }
        "webglReadPixels" => {
            let id = required_canvas_target(state, call)?;
            let x = required_i32(call, 2, "read x")?;
            let y = required_i32(call, 3, "read y")?;
            let width = required_u32(call, 4, "read width")?;
            let height = required_u32(call, 5, "read height")?;
            let format = required_u32(call, 6, "read format")?;
            let kind = required_u32(call, 7, "read type")?;
            let bytes = state
                .angles
                .borrow()
                .read_pixels(id, x, y, width, height, format, kind)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Bytes(bytes))
        }
        "webglReadPixelsOffset" => {
            let id = required_canvas_target(state, call)?;
            let x = required_i32(call, 2, "read x")?;
            let y = required_i32(call, 3, "read y")?;
            let width = required_i32(call, 4, "read width")?;
            let height = required_i32(call, 5, "read height")?;
            let format = required_u32(call, 6, "read format")?;
            let kind = required_u32(call, 7, "read type")?;
            let offset = required_u32(call, 8, "pixel pack buffer offset")?;
            state
                .angles
                .borrow()
                .read_pixels_offset(id, x, y, width, height, format, kind, offset)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateShader" => {
            let id = required_canvas_target(state, call)?;
            let kind = required_u32(call, 2, "shader type")?;
            let shader = state
                .angles
                .borrow_mut()
                .create_shader(id, kind)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(shader as f64))
        }
        "webglShaderSource" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            let source = required_string(call, 3, "shader source")?;
            state
                .angles
                .borrow_mut()
                .shader_source(id, shader, &source)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompileShader" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            state
                .angles
                .borrow_mut()
                .compile_shader(id, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglShaderStatus" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            let status = state
                .angles
                .borrow_mut()
                .shader_status(id, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(status))
        }
        "webglShaderLog" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            let log = state
                .angles
                .borrow_mut()
                .shader_log(id, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(log))
        }
        "webglGetTranslatedShaderSource" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            let source = state
                .angles
                .borrow_mut()
                .translated_shader_source(id, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(source))
        }
        "webglShaderPrecisionFormat" => {
            let id = required_canvas_target(state, call)?;
            let shader_type = required_u32(call, 2, "shader type")?;
            let precision_type = required_u32(call, 3, "shader precision type")?;
            let format = state
                .angles
                .borrow_mut()
                .shader_precision_format(id, shader_type, precision_type)
                .map_err(NativeError::new)?;
            match format {
                Some((range_min, range_max, precision)) => Ok(NativeValue::String(
                    serde_json::json!({
                        "rangeMin": range_min,
                        "rangeMax": range_max,
                        "precision": precision,
                    })
                    .to_string(),
                )),
                None => Ok(NativeValue::Null),
            }
        }
        "webglDeleteShader" => {
            let id = required_canvas_target(state, call)?;
            let shader = required_u64(call, 2, "WebGLShader")?;
            state
                .angles
                .borrow_mut()
                .delete_shader(id, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateProgram" => {
            let id = required_canvas_target(state, call)?;
            let program = state
                .angles
                .borrow_mut()
                .create_program(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(program as f64))
        }
        "webglAttachShader" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let shader = required_u64(call, 3, "WebGLShader")?;
            state
                .angles
                .borrow_mut()
                .attach_shader(id, program, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDetachShader" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let shader = required_u64(call, 3, "WebGLShader")?;
            state
                .angles
                .borrow_mut()
                .detach_shader(id, program, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBindAttribLocation" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "attribute index")?;
            let name = required_string(call, 4, "attribute name")?;
            state
                .angles
                .borrow_mut()
                .bind_attribute_location(id, program, index, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTransformFeedbackVaryings" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let encoded = required_string(call, 3, "transform feedback varyings")?;
            let varyings: Vec<String> = serde_json::from_str(&encoded)
                .map_err(|error| NativeError::new(format!("invalid varyings: {error}")))?;
            let buffer_mode = required_u32(call, 4, "transform feedback buffer mode")?;
            state
                .angles
                .borrow_mut()
                .transform_feedback_varyings(id, program, &varyings, buffer_mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetTransformFeedbackVarying" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "transform feedback varying index")?;
            let varying = state
                .angles
                .borrow_mut()
                .transform_feedback_varying(id, program, index)
                .map_err(NativeError::new)?;
            match varying {
                Some((size, kind, name)) => Ok(NativeValue::String(
                    serde_json::json!({ "size": size, "type": kind, "name": name }).to_string(),
                )),
                None => Ok(NativeValue::Null),
            }
        }
        "webglLinkProgram" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            state
                .angles
                .borrow_mut()
                .link_program(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglValidateProgram" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            state
                .angles
                .borrow_mut()
                .validate_program(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglProgramStatus" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let status = state
                .angles
                .borrow_mut()
                .program_status(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(status))
        }
        "webglProgramValidateStatus" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let status = state
                .angles
                .borrow_mut()
                .program_validate_status(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(status))
        }
        "webglProgramParameter" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let parameter = required_u32(call, 3, "program parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .program_parameter_i32(id, program, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetActiveAttrib" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "active attribute index")?;
            let attribute = state
                .angles
                .borrow_mut()
                .active_attribute(id, program, index)
                .map_err(NativeError::new)?;
            match attribute {
                Some((size, kind, name)) => Ok(NativeValue::String(
                    serde_json::json!({ "size": size, "type": kind, "name": name }).to_string(),
                )),
                None => Ok(NativeValue::Null),
            }
        }
        "webglGetActiveUniform" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "active uniform index")?;
            let uniform = state
                .angles
                .borrow_mut()
                .active_uniform(id, program, index)
                .map_err(NativeError::new)?;
            match uniform {
                Some((size, kind, name)) => Ok(NativeValue::String(
                    serde_json::json!({ "size": size, "type": kind, "name": name }).to_string(),
                )),
                None => Ok(NativeValue::Null),
            }
        }
        "webglProgramLog" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let log = state
                .angles
                .borrow_mut()
                .program_log(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(log))
        }
        "webglUseProgram" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            state
                .angles
                .borrow_mut()
                .use_program(id, (program != 0).then_some(program))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDeleteProgram" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            state
                .angles
                .borrow_mut()
                .delete_program(id, program)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateBuffer" => {
            let id = required_canvas_target(state, call)?;
            let buffer = state
                .angles
                .borrow_mut()
                .create_buffer(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(buffer as f64))
        }
        "webglBindBuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let buffer = required_u64(call, 3, "WebGLBuffer")?;
            state
                .angles
                .borrow_mut()
                .bind_buffer(id, target, (buffer != 0).then_some(buffer))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBindBufferBase" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let index = required_u32(call, 3, "buffer binding index")?;
            let buffer = required_u64(call, 4, "WebGLBuffer")?;
            state
                .angles
                .borrow_mut()
                .bind_buffer_base(id, target, index, (buffer != 0).then_some(buffer))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBindBufferRange" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let index = required_u32(call, 3, "buffer binding index")?;
            let buffer = required_u64(call, 4, "WebGLBuffer")?;
            let offset = required_i32(call, 5, "buffer range offset")?;
            let size = required_i32(call, 6, "buffer range size")?;
            state
                .angles
                .borrow_mut()
                .bind_buffer_range(
                    id,
                    target,
                    index,
                    (buffer != 0).then_some(buffer),
                    offset,
                    size,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBufferData" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing buffer data"))?
                .to_bytes()?;
            let usage = required_u32(call, 4, "buffer usage")?;
            state
                .angles
                .borrow_mut()
                .buffer_data(id, target, &bytes, usage)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBufferSubData" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let offset = required_i32(call, 3, "buffer offset")?;
            let bytes = call
                .argument(4)
                .ok_or_else(|| NativeError::new("missing buffer data"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .buffer_sub_data(id, target, offset, &bytes)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCopyBufferSubData" => {
            let id = required_canvas_target(state, call)?;
            let read_target = required_u32(call, 2, "copy source buffer target")?;
            let write_target = required_u32(call, 3, "copy destination buffer target")?;
            let read_offset = required_i32(call, 4, "copy source buffer offset")?;
            let write_offset = required_i32(call, 5, "copy destination buffer offset")?;
            let size = required_i32(call, 6, "buffer copy size")?;
            state
                .angles
                .borrow_mut()
                .copy_buffer_sub_data(
                    id,
                    read_target,
                    write_target,
                    read_offset,
                    write_offset,
                    size,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetBufferParameter" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let parameter = required_u32(call, 3, "buffer parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .buffer_parameter_i32(id, target, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetBufferSubData" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "buffer target")?;
            let offset = required_i32(call, 3, "buffer offset")?;
            let length = required_u32(call, 4, "buffer read length")? as usize;
            let bytes = state
                .angles
                .borrow_mut()
                .buffer_sub_data_read(id, target, offset, length)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Bytes(bytes))
        }
        "webglDeleteBuffer" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u64(call, 2, "WebGLBuffer")?;
            state
                .angles
                .borrow_mut()
                .delete_buffer(id, buffer)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateTexture" => {
            let id = required_canvas_target(state, call)?;
            let texture = state
                .angles
                .borrow_mut()
                .create_texture(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(texture as f64))
        }
        "webglActiveTexture" => {
            let id = required_canvas_target(state, call)?;
            let unit = required_u32(call, 2, "texture unit")?;
            state
                .angles
                .borrow_mut()
                .active_texture(id, unit)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBindTexture" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let texture = required_u64(call, 3, "WebGLTexture")?;
            state
                .angles
                .borrow_mut()
                .bind_texture(id, target, (texture != 0).then_some(texture))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_i32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let border = required_i32(call, 7, "texture border")?;
            let format = required_u32(call, 8, "texture format")?;
            let kind = required_u32(call, 9, "texture type")?;
            let has_pixels = required_boolean(call, 10, "texture pixel presence")?;
            let pixels = if has_pixels {
                Some(
                    call.argument(11)
                        .ok_or_else(|| NativeError::new("missing texture pixels"))?
                        .to_bytes()?,
                )
            } else {
                None
            };
            state
                .angles
                .borrow_mut()
                .texture_image_2d(
                    id,
                    target,
                    level,
                    internal_format,
                    width,
                    height,
                    border,
                    format,
                    kind,
                    pixels.as_deref(),
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexImage2DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_i32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let border = required_i32(call, 7, "texture border")?;
            let format = required_u32(call, 8, "texture format")?;
            let kind = required_u32(call, 9, "texture type")?;
            let offset = required_u32(call, 10, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .texture_image_2d_offset(
                    id,
                    target,
                    level,
                    internal_format,
                    width,
                    height,
                    border,
                    format,
                    kind,
                    offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let format = required_u32(call, 4, "compressed texture format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let border = required_i32(call, 7, "texture border")?;
            let pixels = call
                .argument(8)
                .ok_or_else(|| NativeError::new("missing compressed texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_image_2d(
                    id, target, level, format, width, height, border, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexImage2DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let format = required_u32(call, 4, "compressed texture format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let border = required_i32(call, 7, "texture border")?;
            let image_size = required_i32(call, 8, "compressed image size")?;
            let offset = required_u32(call, 9, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_image_2d_offset(
                    id, target, level, format, width, height, border, image_size, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexImageSource" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_i32(call, 4, "texture internal format")?;
            let format = required_u32(call, 5, "texture format")?;
            let kind = required_u32(call, 6, "texture type")?;
            let (width, height, pixels) = webgl_texture_source(state, call, 7)?;
            let width = i32::try_from(width)
                .map_err(|_| NativeError::new("texture source width is too large"))?;
            let height = i32::try_from(height)
                .map_err(|_| NativeError::new("texture source height is too large"))?;
            state
                .angles
                .borrow_mut()
                .texture_image_2d(
                    id,
                    target,
                    level,
                    internal_format,
                    width,
                    height,
                    0,
                    format,
                    kind,
                    Some(&pixels),
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexSubImageSource" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let format = required_u32(call, 6, "texture format")?;
            let kind = required_u32(call, 7, "texture type")?;
            let (width, height, pixels) = webgl_texture_source(state, call, 8)?;
            let width = i32::try_from(width)
                .map_err(|_| NativeError::new("texture source width is too large"))?;
            let height = i32::try_from(height)
                .map_err(|_| NativeError::new("texture source height is too large"))?;
            state
                .angles
                .borrow_mut()
                .texture_sub_image_2d(
                    id, target, level, x, y, width, height, format, kind, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexSubImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let width = required_i32(call, 6, "texture width")?;
            let height = required_i32(call, 7, "texture height")?;
            let format = required_u32(call, 8, "texture format")?;
            let kind = required_u32(call, 9, "texture type")?;
            let pixels = call
                .argument(10)
                .ok_or_else(|| NativeError::new("missing texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .texture_sub_image_2d(
                    id, target, level, x, y, width, height, format, kind, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexSubImage2DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let width = required_i32(call, 6, "texture width")?;
            let height = required_i32(call, 7, "texture height")?;
            let format = required_u32(call, 8, "texture format")?;
            let kind = required_u32(call, 9, "texture type")?;
            let offset = required_u32(call, 10, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .texture_sub_image_2d_offset(
                    id, target, level, x, y, width, height, format, kind, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexSubImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let width = required_i32(call, 6, "texture width")?;
            let height = required_i32(call, 7, "texture height")?;
            let format = required_u32(call, 8, "compressed texture format")?;
            let pixels = call
                .argument(9)
                .ok_or_else(|| NativeError::new("missing compressed texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_sub_image_2d(
                    id, target, level, x, y, width, height, format, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexSubImage2DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let width = required_i32(call, 6, "texture width")?;
            let height = required_i32(call, 7, "texture height")?;
            let format = required_u32(call, 8, "compressed texture format")?;
            let image_size = required_u32(call, 9, "compressed image size")?;
            let offset = required_u32(call, 10, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_sub_image_2d_offset(
                    id, target, level, x, y, width, height, format, image_size, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexImage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_i32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let depth = required_i32(call, 7, "texture depth")?;
            let border = required_i32(call, 8, "texture border")?;
            let format = required_u32(call, 9, "texture format")?;
            let kind = required_u32(call, 10, "texture type")?;
            let has_pixels = required_boolean(call, 11, "texture pixel presence")?;
            let pixels = if has_pixels {
                Some(
                    call.argument(12)
                        .ok_or_else(|| NativeError::new("missing texture pixels"))?
                        .to_bytes()?,
                )
            } else {
                None
            };
            state
                .angles
                .borrow_mut()
                .texture_image_3d(
                    id,
                    target,
                    level,
                    internal_format,
                    width,
                    height,
                    depth,
                    border,
                    format,
                    kind,
                    pixels.as_deref(),
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexImage3DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_i32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let depth = required_i32(call, 7, "texture depth")?;
            let border = required_i32(call, 8, "texture border")?;
            let format = required_u32(call, 9, "texture format")?;
            let kind = required_u32(call, 10, "texture type")?;
            let offset = required_u32(call, 11, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .texture_image_3d_offset(
                    id,
                    target,
                    level,
                    internal_format,
                    width,
                    height,
                    depth,
                    border,
                    format,
                    kind,
                    offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexImage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let format = required_u32(call, 4, "compressed texture format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let depth = required_i32(call, 7, "texture depth")?;
            let border = required_i32(call, 8, "texture border")?;
            let pixels = call
                .argument(9)
                .ok_or_else(|| NativeError::new("missing compressed texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_image_3d(
                    id, target, level, format, width, height, depth, border, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexImage3DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let format = required_u32(call, 4, "compressed texture format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let depth = required_i32(call, 7, "texture depth")?;
            let border = required_i32(call, 8, "texture border")?;
            let image_size = required_i32(call, 9, "compressed image size")?;
            let offset = required_u32(call, 10, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_image_3d_offset(
                    id, target, level, format, width, height, depth, border, image_size, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexSubImage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let z = required_i32(call, 6, "texture z offset")?;
            let width = required_i32(call, 7, "texture width")?;
            let height = required_i32(call, 8, "texture height")?;
            let depth = required_i32(call, 9, "texture depth")?;
            let format = required_u32(call, 10, "texture format")?;
            let kind = required_u32(call, 11, "texture type")?;
            let pixels = call
                .argument(12)
                .ok_or_else(|| NativeError::new("missing texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .texture_sub_image_3d(
                    id, target, level, x, y, z, width, height, depth, format, kind, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexSubImage3DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let z = required_i32(call, 6, "texture z offset")?;
            let width = required_i32(call, 7, "texture width")?;
            let height = required_i32(call, 8, "texture height")?;
            let depth = required_i32(call, 9, "texture depth")?;
            let format = required_u32(call, 10, "texture format")?;
            let kind = required_u32(call, 11, "texture type")?;
            let offset = required_u32(call, 12, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .texture_sub_image_3d_offset(
                    id, target, level, x, y, z, width, height, depth, format, kind, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexSubImage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let z = required_i32(call, 6, "texture z offset")?;
            let width = required_i32(call, 7, "texture width")?;
            let height = required_i32(call, 8, "texture height")?;
            let depth = required_i32(call, 9, "texture depth")?;
            let format = required_u32(call, 10, "compressed texture format")?;
            let pixels = call
                .argument(11)
                .ok_or_else(|| NativeError::new("missing compressed texture pixels"))?
                .to_bytes()?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_sub_image_3d(
                    id, target, level, x, y, z, width, height, depth, format, &pixels,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCompressedTexSubImage3DOffset" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x = required_i32(call, 4, "texture x offset")?;
            let y = required_i32(call, 5, "texture y offset")?;
            let z = required_i32(call, 6, "texture z offset")?;
            let width = required_i32(call, 7, "texture width")?;
            let height = required_i32(call, 8, "texture height")?;
            let depth = required_i32(call, 9, "texture depth")?;
            let format = required_u32(call, 10, "compressed texture format")?;
            let image_size = required_u32(call, 11, "compressed image size")?;
            let offset = required_u32(call, 12, "pixel unpack buffer offset")?;
            state
                .angles
                .borrow_mut()
                .compressed_texture_sub_image_3d_offset(
                    id, target, level, x, y, z, width, height, depth, format, image_size, offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexStorage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let levels = required_i32(call, 3, "texture levels")?;
            let internal_format = required_u32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            state
                .angles
                .borrow_mut()
                .texture_storage_2d(id, target, levels, internal_format, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexStorage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let levels = required_i32(call, 3, "texture levels")?;
            let internal_format = required_u32(call, 4, "texture internal format")?;
            let width = required_i32(call, 5, "texture width")?;
            let height = required_i32(call, 6, "texture height")?;
            let depth = required_i32(call, 7, "texture depth")?;
            state
                .angles
                .borrow_mut()
                .texture_storage_3d(id, target, levels, internal_format, width, height, depth)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCopyTexImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let internal_format = required_u32(call, 4, "texture internal format")?;
            let x = required_i32(call, 5, "source x")?;
            let y = required_i32(call, 6, "source y")?;
            let width = required_i32(call, 7, "copy width")?;
            let height = required_i32(call, 8, "copy height")?;
            let border = required_i32(call, 9, "texture border")?;
            state
                .angles
                .borrow_mut()
                .copy_texture_image_2d(
                    id,
                    target,
                    level,
                    internal_format,
                    x,
                    y,
                    width,
                    height,
                    border,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCopyTexSubImage2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x_offset = required_i32(call, 4, "texture x offset")?;
            let y_offset = required_i32(call, 5, "texture y offset")?;
            let x = required_i32(call, 6, "source x")?;
            let y = required_i32(call, 7, "source y")?;
            let width = required_i32(call, 8, "copy width")?;
            let height = required_i32(call, 9, "copy height")?;
            state
                .angles
                .borrow_mut()
                .copy_texture_sub_image_2d(
                    id, target, level, x_offset, y_offset, x, y, width, height,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCopyTexSubImage3D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let level = required_i32(call, 3, "texture level")?;
            let x_offset = required_i32(call, 4, "texture x offset")?;
            let y_offset = required_i32(call, 5, "texture y offset")?;
            let z_offset = required_i32(call, 6, "texture z offset")?;
            let x = required_i32(call, 7, "source x")?;
            let y = required_i32(call, 8, "source y")?;
            let width = required_i32(call, 9, "copy width")?;
            let height = required_i32(call, 10, "copy height")?;
            state
                .angles
                .borrow_mut()
                .copy_texture_sub_image_3d(
                    id, target, level, x_offset, y_offset, z_offset, x, y, width, height,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexParameteri" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let parameter = required_u32(call, 3, "texture parameter")?;
            let value = required_i32(call, 4, "texture parameter value")?;
            state
                .angles
                .borrow_mut()
                .texture_parameter(id, target, parameter, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglTexParameterf" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let parameter = required_u32(call, 3, "texture parameter")?;
            let value = required_number(call, 4, "texture parameter value")? as f32;
            state
                .angles
                .borrow_mut()
                .texture_parameter_f32(id, target, parameter, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetTexParameterI" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let parameter = required_u32(call, 3, "texture parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .texture_parameter_i32_value(id, target, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetTexParameterF" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            let parameter = required_u32(call, 3, "texture parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .texture_parameter_f32_value(id, target, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglPixelStorei" => {
            let id = required_canvas_target(state, call)?;
            let parameter = required_u32(call, 2, "pixel-store parameter")?;
            let value = required_i32(call, 3, "pixel-store value")?;
            state
                .angles
                .borrow_mut()
                .pixel_store(id, parameter, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGenerateMipmap" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "texture target")?;
            state
                .angles
                .borrow_mut()
                .generate_mipmap(id, target)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDeleteTexture" => {
            let id = required_canvas_target(state, call)?;
            let texture = required_u64(call, 2, "WebGLTexture")?;
            state
                .angles
                .borrow_mut()
                .delete_texture(id, texture)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateSampler" => {
            let id = required_canvas_target(state, call)?;
            let sampler = state
                .angles
                .borrow_mut()
                .create_sampler(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(sampler as f64))
        }
        "webglBindSampler" => {
            let id = required_canvas_target(state, call)?;
            let unit = required_u32(call, 2, "texture unit index")?;
            let sampler = required_u64(call, 3, "WebGLSampler")?;
            state
                .angles
                .borrow_mut()
                .bind_sampler(id, unit, (sampler != 0).then_some(sampler))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglSamplerParameteri" => {
            let id = required_canvas_target(state, call)?;
            let sampler = required_u64(call, 2, "WebGLSampler")?;
            let parameter = required_u32(call, 3, "sampler parameter")?;
            let value = required_i32(call, 4, "sampler parameter value")?;
            state
                .angles
                .borrow_mut()
                .sampler_parameter_i32(id, sampler, parameter, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglSamplerParameterf" => {
            let id = required_canvas_target(state, call)?;
            let sampler = required_u64(call, 2, "WebGLSampler")?;
            let parameter = required_u32(call, 3, "sampler parameter")?;
            let value = required_number(call, 4, "sampler parameter value")? as f32;
            state
                .angles
                .borrow_mut()
                .sampler_parameter_f32(id, sampler, parameter, value)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetSamplerParameterI" => {
            let id = required_canvas_target(state, call)?;
            let sampler = required_u64(call, 2, "WebGLSampler")?;
            let parameter = required_u32(call, 3, "sampler parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .sampler_parameter_i32_value(id, sampler, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetSamplerParameterF" => {
            let id = required_canvas_target(state, call)?;
            let sampler = required_u64(call, 2, "WebGLSampler")?;
            let parameter = required_u32(call, 3, "sampler parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .sampler_parameter_f32_value(id, sampler, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglDeleteSampler" => {
            let id = required_canvas_target(state, call)?;
            let sampler = required_u64(call, 2, "WebGLSampler")?;
            state
                .angles
                .borrow_mut()
                .delete_sampler(id, sampler)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateQuery" => {
            let id = required_canvas_target(state, call)?;
            let query = state
                .angles
                .borrow_mut()
                .create_query(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(query as f64))
        }
        "webglBeginQuery" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "query target")?;
            let query = required_u64(call, 3, "WebGLQuery")?;
            state
                .angles
                .borrow_mut()
                .begin_query(id, target, query)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglEndQuery" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "query target")?;
            state
                .angles
                .borrow_mut()
                .end_query(id, target)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetQueryParameter" => {
            let id = required_canvas_target(state, call)?;
            let query = required_u64(call, 2, "WebGLQuery")?;
            let parameter = required_u32(call, 3, "query parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .query_parameter(id, query, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglGetQueryParameter64" => {
            let id = required_canvas_target(state, call)?;
            let query = required_u64(call, 2, "WebGLQuery")?;
            let parameter = required_u32(call, 3, "query parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .query_parameter_u64(id, query, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglQueryCounter" => {
            let id = required_canvas_target(state, call)?;
            let query = required_u64(call, 2, "WebGLQuery")?;
            let target = required_u32(call, 3, "query target")?;
            state
                .angles
                .borrow_mut()
                .query_counter(id, query, target)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetQueryCounterBits" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "query target")?;
            let value = state
                .angles
                .borrow_mut()
                .query_counter_bits(id, target)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglDeleteQuery" => {
            let id = required_canvas_target(state, call)?;
            let query = required_u64(call, 2, "WebGLQuery")?;
            state
                .angles
                .borrow_mut()
                .delete_query(id, query)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFenceSync" => {
            let id = required_canvas_target(state, call)?;
            let condition = required_u32(call, 2, "sync condition")?;
            let flags = required_u32(call, 3, "sync flags")?;
            let sync = state
                .angles
                .borrow_mut()
                .fence_sync(id, condition, flags)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(sync as f64))
        }
        "webglClientWaitSync" => {
            let id = required_canvas_target(state, call)?;
            let sync = required_u64(call, 2, "WebGLSync")?;
            let flags = required_u32(call, 3, "sync flags")?;
            let timeout = required_number(call, 4, "sync timeout")?;
            if !timeout.is_finite() || timeout < 0.0 || timeout > f64::from(i32::MAX) {
                return Err(NativeError::new("invalid sync timeout"));
            }
            let result = state
                .angles
                .borrow_mut()
                .client_wait_sync(id, sync, flags, timeout.trunc() as i32)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(result as f64))
        }
        "webglWaitSync" => {
            let id = required_canvas_target(state, call)?;
            let sync = required_u64(call, 2, "WebGLSync")?;
            let flags = required_u32(call, 3, "sync flags")?;
            let timeout_value = required_number(call, 4, "sync timeout")?;
            let timeout = if timeout_value == -1.0 {
                u64::MAX
            } else {
                if !timeout_value.is_finite()
                    || timeout_value < 0.0
                    || timeout_value > u64::MAX as f64
                    || timeout_value.fract() != 0.0
                {
                    return Err(NativeError::new("invalid sync timeout"));
                }
                timeout_value as u64
            };
            state
                .angles
                .borrow_mut()
                .wait_sync(id, sync, flags, timeout)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetSyncParameter" => {
            let id = required_canvas_target(state, call)?;
            let sync = required_u64(call, 2, "WebGLSync")?;
            let parameter = required_u32(call, 3, "sync parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .sync_parameter(id, sync, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglDeleteSync" => {
            let id = required_canvas_target(state, call)?;
            let sync = required_u64(call, 2, "WebGLSync")?;
            state
                .angles
                .borrow_mut()
                .delete_sync(id, sync)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            let feedback = state
                .angles
                .borrow_mut()
                .create_transform_feedback(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(feedback as f64))
        }
        "webglBindTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "transform feedback target")?;
            let feedback = required_u64(call, 3, "WebGLTransformFeedback")?;
            state
                .angles
                .borrow_mut()
                .bind_transform_feedback(id, target, (feedback != 0).then_some(feedback))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBeginTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            let primitive_mode = required_u32(call, 2, "transform feedback primitive mode")?;
            state
                .angles
                .borrow_mut()
                .begin_transform_feedback(id, primitive_mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglEndTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            state
                .angles
                .borrow_mut()
                .end_transform_feedback(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglPauseTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            state
                .angles
                .borrow_mut()
                .pause_transform_feedback(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglResumeTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            state
                .angles
                .borrow_mut()
                .resume_transform_feedback(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDeleteTransformFeedback" => {
            let id = required_canvas_target(state, call)?;
            let feedback = required_u64(call, 2, "WebGLTransformFeedback")?;
            state
                .angles
                .borrow_mut()
                .delete_transform_feedback(id, feedback)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let framebuffer = state
                .angles
                .borrow_mut()
                .create_framebuffer(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(framebuffer as f64))
        }
        "webglBindFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let framebuffer = required_u64(call, 3, "WebGLFramebuffer")?;
            state
                .angles
                .borrow_mut()
                .bind_framebuffer(id, target, (framebuffer != 0).then_some(framebuffer))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFramebufferTexture2D" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachment = required_u32(call, 3, "framebuffer attachment")?;
            let texture_target = required_u32(call, 4, "texture target")?;
            let texture = required_u64(call, 5, "WebGLTexture")?;
            let level = required_i32(call, 6, "texture level")?;
            state
                .angles
                .borrow_mut()
                .framebuffer_texture_2d(
                    id,
                    target,
                    attachment,
                    texture_target,
                    (texture != 0).then_some(texture),
                    level,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFramebufferTextureLayer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachment = required_u32(call, 3, "framebuffer attachment")?;
            let texture = required_u64(call, 4, "WebGLTexture")?;
            let level = required_i32(call, 5, "texture level")?;
            let layer = required_i32(call, 6, "texture layer")?;
            state
                .angles
                .borrow_mut()
                .framebuffer_texture_layer(
                    id,
                    target,
                    attachment,
                    (texture != 0).then_some(texture),
                    level,
                    layer,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglInvalidateFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachments = required_u32_array(call, 3, "framebuffer attachments")?;
            state
                .angles
                .borrow_mut()
                .invalidate_framebuffer(id, target, &attachments)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglInvalidateSubFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachments = required_u32_array(call, 3, "framebuffer attachments")?;
            let bounds = required_numbers::<4>(call, 4, "framebuffer invalidation bounds")?
                .map(|value| value as i32);
            state
                .angles
                .borrow_mut()
                .invalidate_sub_framebuffer(
                    id,
                    target,
                    &attachments,
                    bounds[0],
                    bounds[1],
                    bounds[2],
                    bounds[3],
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetInternalformatParameter" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "renderbuffer target")?;
            let internal_format = required_u32(call, 3, "renderbuffer internal format")?;
            let parameter = required_u32(call, 4, "internal format parameter")?;
            if parameter != 0x80a9 {
                return Err(NativeError::new("unsupported internal format parameter"));
            }
            let samples = state
                .angles
                .borrow_mut()
                .internal_format_samples(id, target, internal_format)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Int32Array(samples))
        }
        "webglClearBufferiv" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u32(call, 2, "clear buffer")?;
            let draw_buffer = required_i32(call, 3, "clear draw buffer")? as u32;
            let values = required_i32_array(call, 4, "integer clear values")?;
            state
                .angles
                .borrow_mut()
                .clear_buffer_i32(id, buffer, draw_buffer, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClearBufferuiv" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u32(call, 2, "clear buffer")?;
            let draw_buffer = required_i32(call, 3, "clear draw buffer")? as u32;
            let values = required_u32_array(call, 4, "unsigned integer clear values")?;
            state
                .angles
                .borrow_mut()
                .clear_buffer_u32(id, buffer, draw_buffer, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClearBufferfv" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u32(call, 2, "clear buffer")?;
            let draw_buffer = required_i32(call, 3, "clear draw buffer")? as u32;
            let values = required_f32_array(call, 4, "floating-point clear values")?;
            state
                .angles
                .borrow_mut()
                .clear_buffer_f32(id, buffer, draw_buffer, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglClearBufferfi" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u32(call, 2, "clear buffer")?;
            let draw_buffer = required_i32(call, 3, "clear draw buffer")? as u32;
            let depth = required_number(call, 4, "depth clear value")? as f32;
            let stencil = required_i32(call, 5, "stencil clear value")?;
            state
                .angles
                .borrow_mut()
                .clear_buffer_depth_stencil(id, buffer, draw_buffer, depth, stencil)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawBuffers" => {
            let id = required_canvas_target(state, call)?;
            let buffers = required_u32_array(call, 2, "draw buffers")?;
            state
                .angles
                .borrow_mut()
                .draw_buffers(id, &buffers)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglReadBuffer" => {
            let id = required_canvas_target(state, call)?;
            let buffer = required_u32(call, 2, "read buffer")?;
            state
                .angles
                .borrow_mut()
                .read_buffer(id, buffer)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglBlitFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let source = required_numbers::<4>(call, 2, "source framebuffer bounds")?
                .map(|value| value as i32);
            let destination = required_numbers::<4>(call, 6, "destination framebuffer bounds")?
                .map(|value| value as i32);
            let mask = required_u32(call, 10, "framebuffer blit mask")?;
            let filter = required_u32(call, 11, "framebuffer blit filter")?;
            state
                .angles
                .borrow_mut()
                .blit_framebuffer(id, source, destination, mask, filter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCheckFramebufferStatus" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let status = state
                .angles
                .borrow_mut()
                .framebuffer_status(id, target)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(status)))
        }
        "webglDeleteFramebuffer" => {
            let id = required_canvas_target(state, call)?;
            let framebuffer = required_u64(call, 2, "WebGLFramebuffer")?;
            state
                .angles
                .borrow_mut()
                .delete_framebuffer(id, framebuffer)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateRenderbuffer" => {
            let id = required_canvas_target(state, call)?;
            let renderbuffer = state
                .angles
                .borrow_mut()
                .create_renderbuffer(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(renderbuffer as f64))
        }
        "webglBindRenderbuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "renderbuffer target")?;
            let renderbuffer = required_u64(call, 3, "WebGLRenderbuffer")?;
            state
                .angles
                .borrow_mut()
                .bind_renderbuffer(id, target, (renderbuffer != 0).then_some(renderbuffer))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglRenderbufferStorage" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "renderbuffer target")?;
            let internal_format = required_u32(call, 3, "renderbuffer internal format")?;
            let width = required_i32(call, 4, "renderbuffer width")?;
            let height = required_i32(call, 5, "renderbuffer height")?;
            state
                .angles
                .borrow_mut()
                .renderbuffer_storage(id, target, internal_format, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetRenderbufferParameter" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "renderbuffer target")?;
            let parameter = required_u32(call, 3, "renderbuffer parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .renderbuffer_parameter_i32(id, target, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglRenderbufferStorageMultisample" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "renderbuffer target")?;
            let samples = required_i32(call, 3, "renderbuffer sample count")?;
            let internal_format = required_u32(call, 4, "renderbuffer internal format")?;
            let width = required_i32(call, 5, "renderbuffer width")?;
            let height = required_i32(call, 6, "renderbuffer height")?;
            state
                .angles
                .borrow_mut()
                .renderbuffer_storage_multisample(
                    id,
                    target,
                    samples,
                    internal_format,
                    width,
                    height,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglFramebufferRenderbuffer" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachment = required_u32(call, 3, "framebuffer attachment")?;
            let renderbuffer_target = required_u32(call, 4, "renderbuffer target")?;
            let renderbuffer = required_u64(call, 5, "WebGLRenderbuffer")?;
            state
                .angles
                .borrow_mut()
                .framebuffer_renderbuffer(
                    id,
                    target,
                    attachment,
                    renderbuffer_target,
                    (renderbuffer != 0).then_some(renderbuffer),
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetFramebufferAttachmentParameter" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "framebuffer target")?;
            let attachment = required_u32(call, 3, "framebuffer attachment")?;
            let parameter = required_u32(call, 4, "framebuffer attachment parameter")?;
            if parameter == glow::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME {
                let (kind, object) = state
                    .angles
                    .borrow_mut()
                    .framebuffer_attachment_object(id, target, attachment)
                    .map_err(NativeError::new)?;
                Ok(NativeValue::String(
                    serde_json::json!({ "type": kind, "id": object }).to_string(),
                ))
            } else {
                let value = state
                    .angles
                    .borrow_mut()
                    .framebuffer_attachment_parameter_i32(id, target, attachment, parameter)
                    .map_err(NativeError::new)?;
                Ok(NativeValue::Number(value as f64))
            }
        }
        "webglDeleteRenderbuffer" => {
            let id = required_canvas_target(state, call)?;
            let renderbuffer = required_u64(call, 2, "WebGLRenderbuffer")?;
            state
                .angles
                .borrow_mut()
                .delete_renderbuffer(id, renderbuffer)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglCreateVertexArray" => {
            let id = required_canvas_target(state, call)?;
            let array = state
                .angles
                .borrow_mut()
                .create_vertex_array(id)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(array as f64))
        }
        "webglBindVertexArray" => {
            let id = required_canvas_target(state, call)?;
            let array = required_u64(call, 2, "WebGLVertexArrayObject")?;
            state
                .angles
                .borrow_mut()
                .bind_vertex_array(id, (array != 0).then_some(array))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDeleteVertexArray" => {
            let id = required_canvas_target(state, call)?;
            let array = required_u64(call, 2, "WebGLVertexArrayObject")?;
            state
                .angles
                .borrow_mut()
                .delete_vertex_array(id, array)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetAttribLocation" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let name = required_string(call, 3, "attribute name")?;
            let location = state
                .angles
                .borrow_mut()
                .attribute_location(id, program, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(location)))
        }
        "webglGetFragDataLocation" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let name = required_string(call, 3, "fragment output name")?;
            let location = state
                .angles
                .borrow_mut()
                .fragment_data_location(id, program, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(location)))
        }
        "webglEnableVertexAttribArray" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            state
                .angles
                .borrow_mut()
                .enable_attribute(id, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDisableVertexAttribArray" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            state
                .angles
                .borrow_mut()
                .disable_attribute(id, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglVertexAttribPointer" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let size = required_i32(call, 3, "attribute size")?;
            let kind = required_u32(call, 4, "attribute type")?;
            let normalized = required_boolean(call, 5, "attribute normalization")?;
            let stride = required_i32(call, 6, "attribute stride")?;
            let offset = required_i32(call, 7, "attribute offset")?;
            state
                .angles
                .borrow_mut()
                .attribute_pointer(id, index, size, kind, normalized, stride, offset)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglVertexAttribF" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let values = required_f32_array(call, 3, "attribute values")?;
            let values: [f32; 4] = values
                .try_into()
                .map_err(|_| NativeError::new("attribute requires four values"))?;
            state
                .angles
                .borrow_mut()
                .attribute_f32(id, index, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetVertexAttribCurrent" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let values = state
                .angles
                .borrow_mut()
                .attribute_f32_value(id, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&values).expect("attribute values are serializable"),
            ))
        }
        "webglGetVertexAttribI" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let parameter = required_u32(call, 3, "attribute parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .attribute_parameter_i32(id, index, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(f64::from(value)))
        }
        "webglGetVertexAttribBuffer" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let value = state
                .angles
                .borrow_mut()
                .attribute_buffer(id, index)
                .map_err(NativeError::new)?;
            match value {
                Some(value) => Ok(NativeValue::Number(value as f64)),
                None => Ok(NativeValue::Null),
            }
        }
        "webglGetVertexAttribOffset" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let parameter = required_u32(call, 3, "attribute pointer parameter")?;
            let value = state
                .angles
                .borrow_mut()
                .attribute_offset(id, index, parameter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(value as f64))
        }
        "webglVertexAttribIPointer" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let size = required_i32(call, 3, "attribute size")?;
            let kind = required_u32(call, 4, "attribute type")?;
            let stride = required_i32(call, 5, "attribute stride")?;
            let offset = required_i32(call, 6, "attribute offset")?;
            state
                .angles
                .borrow_mut()
                .integer_attribute_pointer(id, index, size, kind, stride, offset)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglVertexAttribI4i" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let values = required_i32_array(call, 3, "integer attribute values")?;
            let values: [i32; 4] = values
                .try_into()
                .map_err(|_| NativeError::new("integer attribute requires four values"))?;
            state
                .angles
                .borrow_mut()
                .integer_attribute_i32(id, index, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglVertexAttribI4ui" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "attribute index")?;
            let values = required_u32_array(call, 3, "unsigned integer attribute values")?;
            let values: [u32; 4] = values
                .try_into()
                .map_err(|_| NativeError::new("integer attribute requires four values"))?;
            state
                .angles
                .borrow_mut()
                .integer_attribute_u32(id, index, values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetUniformLocation" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let name = required_string(call, 3, "uniform name")?;
            match state
                .angles
                .borrow_mut()
                .uniform_location(id, program, &name)
                .map_err(NativeError::new)?
            {
                Some(location) => Ok(NativeValue::Number(location as f64)),
                None => Ok(NativeValue::Null),
            }
        }
        "webglGetUniformIndices" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let names: Vec<String> =
                serde_json::from_str(&required_string(call, 3, "uniform names")?)
                    .map_err(|error| NativeError::new(format!("invalid uniform names: {error}")))?;
            let indices = state
                .angles
                .borrow_mut()
                .uniform_indices(id, program, &names)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&indices).expect("uniform indices are serializable"),
            ))
        }
        "webglGetUniform" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let location = required_u64(call, 3, "WebGLUniformLocation")?;
            let kind = required_u32(call, 4, "uniform type")?;
            let value = state
                .angles
                .borrow_mut()
                .uniform_value(id, program, location, kind)
                .map_err(NativeError::new)?;
            let encoded = match value {
                UniformValue::Float(values) => {
                    serde_json::json!({ "kind": "float", "values": values })
                }
                UniformValue::Int(values) => {
                    serde_json::json!({ "kind": "int", "values": values })
                }
                UniformValue::Uint(values) => {
                    serde_json::json!({ "kind": "uint", "values": values })
                }
            };
            Ok(NativeValue::String(encoded.to_string()))
        }
        "webglGetActiveUniforms" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let indices = required_u32_array(call, 3, "uniform indices")?;
            let parameter = required_u32(call, 4, "active uniform parameter")?;
            let values = state
                .angles
                .borrow_mut()
                .active_uniform_parameters(id, program, &indices, parameter)
                .map_err(NativeError::new)?;
            let encoded = if parameter == 0x8a3e {
                serde_json::to_string(
                    &values
                        .into_iter()
                        .map(|value| value != 0)
                        .collect::<Vec<_>>(),
                )
            } else {
                serde_json::to_string(&values)
            }
            .expect("active uniform parameters are serializable");
            Ok(NativeValue::String(encoded))
        }
        "webglEnableWebExtension" => {
            let id = required_canvas_target(state, call)?;
            let name = required_string(call, 2, "WebGL extension name")?;
            state
                .angles
                .borrow_mut()
                .enable_webgl_extension(id, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglGetUniformBlockIndex" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let name = required_string(call, 3, "uniform block name")?;
            let index = state
                .angles
                .borrow_mut()
                .uniform_block_index(id, program, &name)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(index as f64))
        }
        "webglGetActiveUniformBlockParameter" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "uniform block index")?;
            let parameter = required_u32(call, 4, "uniform block parameter")?;
            let encoded = if parameter == 0x8a43 {
                let values = state
                    .angles
                    .borrow_mut()
                    .uniform_block_active_indices(id, program, index)
                    .map_err(NativeError::new)?;
                serde_json::to_string(&values)
            } else {
                let value = state
                    .angles
                    .borrow_mut()
                    .uniform_block_parameter_i32(id, program, index, parameter)
                    .map_err(NativeError::new)?;
                if matches!(parameter, 0x8a44 | 0x8a46) {
                    serde_json::to_string(&(value != 0))
                } else {
                    serde_json::to_string(&value)
                }
            }
            .expect("uniform block parameter is serializable");
            Ok(NativeValue::String(encoded))
        }
        "webglGetActiveUniformBlockName" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "uniform block index")?;
            let name = state
                .angles
                .borrow_mut()
                .uniform_block_name(id, program, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(name))
        }
        "webglUniformBlockBinding" => {
            let id = required_canvas_target(state, call)?;
            let program = required_u64(call, 2, "WebGLProgram")?;
            let index = required_u32(call, 3, "uniform block index")?;
            let binding = required_u32(call, 4, "uniform block binding")?;
            state
                .angles
                .borrow_mut()
                .uniform_block_binding(id, program, index, binding)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglUniformF" => {
            let id = required_canvas_target(state, call)?;
            let location = required_u64(call, 2, "WebGLUniformLocation")?;
            let components = required_u32(call, 3, "uniform component count")?;
            let values = required_f32_array(call, 4, "uniform values")?;
            state
                .angles
                .borrow_mut()
                .uniform_f32(id, location, components, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglUniformI" => {
            let id = required_canvas_target(state, call)?;
            let location = required_u64(call, 2, "WebGLUniformLocation")?;
            let components = required_u32(call, 3, "uniform component count")?;
            let values = required_i32_array(call, 4, "uniform values")?;
            state
                .angles
                .borrow_mut()
                .uniform_i32(id, location, components, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglUniformU" => {
            let id = required_canvas_target(state, call)?;
            let location = required_u64(call, 2, "WebGLUniformLocation")?;
            let components = required_u32(call, 3, "uniform component count")?;
            let values = required_u32_array(call, 4, "uniform values")?;
            state
                .angles
                .borrow_mut()
                .uniform_u32(id, location, components, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglUniformMatrixF" => {
            let id = required_canvas_target(state, call)?;
            let location = required_u64(call, 2, "WebGLUniformLocation")?;
            let dimension = required_u32(call, 3, "uniform matrix dimension")?;
            let transpose = required_boolean(call, 4, "uniform matrix transpose")?;
            let values = required_f32_array(call, 5, "uniform matrix values")?;
            state
                .angles
                .borrow_mut()
                .uniform_matrix_f32(id, location, dimension, transpose, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglUniformMatrixRectF" => {
            let id = required_canvas_target(state, call)?;
            let location = required_u64(call, 2, "WebGLUniformLocation")?;
            let columns = required_u32(call, 3, "uniform matrix columns")?;
            let rows = required_u32(call, 4, "uniform matrix rows")?;
            let transpose = required_boolean(call, 5, "uniform matrix transpose")?;
            let values = required_f32_array(call, 6, "uniform matrix values")?;
            state
                .angles
                .borrow_mut()
                .uniform_matrix_rect_f32(id, location, columns, rows, transpose, &values)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglViewport" => {
            let id = required_canvas_target(state, call)?;
            let values = required_numbers::<4>(call, 2, "viewport")?;
            state
                .angles
                .borrow_mut()
                .viewport(
                    id,
                    values[0] as i32,
                    values[1] as i32,
                    values[2] as i32,
                    values[3] as i32,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglHint" => {
            let id = required_canvas_target(state, call)?;
            let target = required_u32(call, 2, "WebGL hint target")?;
            let mode = required_u32(call, 3, "WebGL hint mode")?;
            state
                .angles
                .borrow_mut()
                .hint(id, target, mode)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawArrays" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let first = required_i32(call, 3, "first vertex")?;
            let count = required_i32(call, 4, "vertex count")?;
            state
                .angles
                .borrow_mut()
                .draw_arrays(id, mode, first, count)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawElements" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let count = required_i32(call, 3, "element count")?;
            let kind = required_u32(call, 4, "element type")?;
            let offset = required_i32(call, 5, "element offset")?;
            state
                .angles
                .borrow_mut()
                .draw_elements(id, mode, count, kind, offset)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawArrays" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let firsts = required_i32_array(call, 3, "first vertices")?;
            let counts = required_i32_array(call, 4, "vertex counts")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_arrays(id, mode, &firsts, &counts)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawElements" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let counts = required_i32_array(call, 3, "element counts")?;
            let kind = required_u32(call, 4, "element type")?;
            let offsets = required_i32_array(call, 5, "element offsets")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_elements(id, mode, &counts, kind, &offsets)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawArraysInstanced" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let firsts = required_i32_array(call, 3, "first vertices")?;
            let counts = required_i32_array(call, 4, "vertex counts")?;
            let instances = required_i32_array(call, 5, "instance counts")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_arrays_instanced(id, mode, &firsts, &counts, &instances)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawElementsInstanced" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let counts = required_i32_array(call, 3, "element counts")?;
            let kind = required_u32(call, 4, "element type")?;
            let offsets = required_i32_array(call, 5, "element offsets")?;
            let instances = required_i32_array(call, 6, "instance counts")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_elements_instanced(id, mode, &counts, kind, &offsets, &instances)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawArraysInstancedBaseInstance" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let firsts = required_i32_array(call, 3, "first vertices")?;
            let counts = required_i32_array(call, 4, "vertex counts")?;
            let instances = required_i32_array(call, 5, "instance counts")?;
            let base_instances = required_u32_array(call, 6, "base instances")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_arrays_instanced_base_instance(
                    id,
                    mode,
                    &firsts,
                    &counts,
                    &instances,
                    &base_instances,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglMultiDrawElementsInstancedBaseVertexBaseInstance" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let counts = required_i32_array(call, 3, "element counts")?;
            let kind = required_u32(call, 4, "element type")?;
            let offsets = required_i32_array(call, 5, "element offsets")?;
            let instances = required_i32_array(call, 6, "instance counts")?;
            let base_vertices = required_i32_array(call, 7, "base vertices")?;
            let base_instances = required_u32_array(call, 8, "base instances")?;
            state
                .angles
                .borrow_mut()
                .multi_draw_elements_instanced_base_vertex_base_instance(
                    id,
                    mode,
                    &counts,
                    kind,
                    &offsets,
                    &instances,
                    &base_vertices,
                    &base_instances,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawRangeElements" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let start = required_u32(call, 3, "minimum element")?;
            let end = required_u32(call, 4, "maximum element")?;
            let count = required_i32(call, 5, "element count")?;
            let kind = required_u32(call, 6, "element type")?;
            let offset = required_i32(call, 7, "element offset")?;
            state
                .angles
                .borrow_mut()
                .draw_range_elements(id, mode, start, end, count, kind, offset)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawArraysInstanced" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let first = required_i32(call, 3, "first vertex")?;
            let count = required_i32(call, 4, "vertex count")?;
            let instances = required_i32(call, 5, "instance count")?;
            state
                .angles
                .borrow_mut()
                .draw_arrays_instanced(id, mode, first, count, instances)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawElementsInstanced" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let count = required_i32(call, 3, "element count")?;
            let kind = required_u32(call, 4, "element type")?;
            let offset = required_i32(call, 5, "element offset")?;
            let instances = required_i32(call, 6, "instance count")?;
            state
                .angles
                .borrow_mut()
                .draw_elements_instanced(id, mode, count, kind, offset, instances)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawArraysInstancedBaseInstance" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let first = required_i32(call, 3, "first vertex")?;
            let count = required_i32(call, 4, "vertex count")?;
            let instances = required_i32(call, 5, "instance count")?;
            let base_instance = required_u32(call, 6, "base instance")?;
            state
                .angles
                .borrow_mut()
                .draw_arrays_instanced_base_instance(
                    id,
                    mode,
                    first,
                    count,
                    instances,
                    base_instance,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglDrawElementsInstancedBaseVertexBaseInstance" => {
            let id = required_canvas_target(state, call)?;
            let mode = required_u32(call, 2, "drawing mode")?;
            let count = required_i32(call, 3, "element count")?;
            let kind = required_u32(call, 4, "element type")?;
            let offset = required_i32(call, 5, "element offset")?;
            let instances = required_i32(call, 6, "instance count")?;
            let base_vertex = required_i32(call, 7, "base vertex")?;
            let base_instance = required_u32(call, 8, "base instance")?;
            state
                .angles
                .borrow_mut()
                .draw_elements_instanced_base_vertex_base_instance(
                    id,
                    mode,
                    count,
                    kind,
                    offset,
                    instances,
                    base_vertex,
                    base_instance,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "webglVertexAttribDivisor" => {
            let id = required_canvas_target(state, call)?;
            let index = required_u32(call, 2, "vertex attribute index")?;
            let divisor = required_u32(call, 3, "vertex attribute divisor")?;
            state
                .angles
                .borrow_mut()
                .vertex_attrib_divisor(id, index, divisor)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        _ => Err(NativeError::new(format!(
            "unknown native WebGL operation: {operation}"
        ))),
    }
}
