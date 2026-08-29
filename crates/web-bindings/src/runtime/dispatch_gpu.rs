use super::*;

pub(super) fn dispatch(
    state: &BindingState,
    call: &NativeCall<'_>,
    operation: &str,
) -> Result<NativeValue, NativeError> {
    match operation {
        "gpuCanvasAcquire" => {
            if !state.features.webgpu {
                return Ok(NativeValue::Boolean(false));
            }
            let canvas = required_canvas_argument(state, call, 2)?;
            let width = required_u32(call, 3, "canvas width")?;
            let height = required_u32(call, 4, "canvas height")?;
            let acquired = state
                .canvases
                .borrow_mut()
                .acquire_webgpu(canvas, width, height)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Boolean(acquired))
        }
        "gpuPresentCanvas" => {
            let device = required_u64(call, 2, "GPU device")?;
            let texture = required_u64(call, 3, "GPU canvas texture")?;
            let canvas = required_canvas_argument(state, call, 4)?;
            let width = required_u32(call, 5, "canvas width")?;
            let height = required_u32(call, 6, "canvas height")?;
            let pixels = state
                .gpu
                .borrow()
                .read_texture_rgba(device, texture, width, height)
                .map_err(NativeError::new)?;
            state
                .canvases
                .borrow_mut()
                .write_rgba(canvas, width, height, 0, 0, width, height, &pixels)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuRequestAdapter" => {
            if !state.features.webgpu {
                return Err(NativeError::new("WebGPU is disabled"));
            }
            let preference = required_string(call, 2, "power preference")?;
            let force_fallback = required_boolean(call, 3, "forceFallbackAdapter")?;
            match state.gpu.borrow_mut().request_adapter(&preference, force_fallback).map_err(NativeError::new)? {
                Some((id, metadata)) => Ok(NativeValue::String(serde_json::json!({
                    "id": id,
                    "metadata": serde_json::from_str::<serde_json::Value>(&metadata).expect("GPU metadata JSON is valid"),
                }).to_string())),
                None => Ok(NativeValue::Null),
            }
        }
        "gpuRequestDevice" => {
            let adapter = required_u64(call, 2, "GPU adapter")?;
            let features = required_string(call, 3, "required GPU features")?;
            let features: Vec<String> = serde_json::from_str(&features).map_err(|error| {
                NativeError::new(format!("invalid required GPU features: {error}"))
            })?;
            let limits = required_string(call, 4, "required GPU limits")?;
            let limits: HashMap<String, u64> = serde_json::from_str(&limits).map_err(|error| {
                NativeError::new(format!("invalid required GPU limits: {error}"))
            })?;
            let label = required_string(call, 5, "GPU device label")?;
            let (id, limits, features) = state
                .gpu
                .borrow_mut()
                .request_device(adapter, &features, &limits, &label)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(serde_json::json!({
                "id": id,
                "limits": serde_json::from_str::<serde_json::Value>(&limits).expect("GPU limits JSON is valid"),
                "features": features,
                "label": label,
            }).to_string()))
        }
        "gpuDestroyDevice" => {
            let device = required_u64(call, 2, "GPU device")?;
            state
                .gpu
                .borrow()
                .destroy_device(device)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuTakeDeviceLost" => {
            let device = required_u64(call, 2, "GPU device")?;
            let record = state
                .gpu
                .borrow()
                .take_device_lost(device)
                .map_err(NativeError::new)?;
            Ok(match record {
                Some((reason, message)) => NativeValue::String(
                    serde_json::json!({ "reason": reason, "message": message }).to_string(),
                ),
                None => NativeValue::Null,
            })
        }
        "gpuPushErrorScope" => {
            let device = required_u64(call, 2, "GPU device")?;
            let filter = required_string(call, 3, "GPU error filter")?;
            state
                .gpu
                .borrow_mut()
                .push_error_scope(device, &filter)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuPopErrorScope" => {
            let device = required_u64(call, 2, "GPU device")?;
            let error = state
                .gpu
                .borrow_mut()
                .pop_error_scope(device)
                .map_err(NativeError::new)?;
            Ok(match error {
                Some((kind, message)) => NativeValue::String(
                    serde_json::json!({ "kind": kind, "message": message }).to_string(),
                ),
                None => NativeValue::Null,
            })
        }
        "gpuTakeUncapturedErrors" => {
            let device = required_u64(call, 2, "GPU device")?;
            let errors = state
                .gpu
                .borrow()
                .take_uncaptured_errors(device)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&errors).expect("GPU errors are serializable"),
            ))
        }
        "gpuCreateBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let size = required_u64(call, 3, "GPU buffer size")?;
            let usage = required_u32(call, 4, "GPU buffer usage")?;
            let mapped_at_creation = required_boolean(call, 5, "mappedAtCreation")?;
            let id = state
                .gpu
                .borrow_mut()
                .create_buffer(device, size, usage, mapped_at_creation)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(id as f64))
        }
        "gpuWriteBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let buffer = required_u64(call, 3, "GPU buffer")?;
            let offset = required_u64(call, 4, "GPU buffer offset")?;
            let bytes = call
                .argument(5)
                .ok_or_else(|| NativeError::new("missing GPU buffer data"))?
                .to_bytes()?;
            state
                .gpu
                .borrow()
                .write_buffer(device, buffer, offset, &bytes)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuWriteTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let texture = required_u64(call, 3, "GPU texture")?;
            let mip_level = required_u32(call, 4, "texture mip level")?;
            let origin = [
                required_u32(call, 5, "texture origin x")?,
                required_u32(call, 6, "texture origin y")?,
                required_u32(call, 7, "texture origin z")?,
            ];
            let aspect = required_string(call, 8, "texture aspect")?;
            let bytes = call
                .argument(9)
                .ok_or_else(|| NativeError::new("missing GPU texture data"))?
                .to_bytes()?;
            let offset = required_u64(call, 10, "texture data offset")?;
            let bytes_per_row = required_u32(call, 11, "texture bytes per row")?;
            let rows_per_image = required_u32(call, 12, "texture rows per image")?;
            let size = [
                required_u32(call, 13, "texture write width")?,
                required_u32(call, 14, "texture write height")?,
                required_u32(call, 15, "texture write depth")?,
            ];
            state
                .gpu
                .borrow()
                .write_texture(
                    device,
                    texture,
                    mip_level,
                    origin,
                    &aspect,
                    &bytes,
                    offset,
                    (bytes_per_row != 0).then_some(bytes_per_row),
                    (rows_per_image != 0).then_some(rows_per_image),
                    size,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuUnmapBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let buffer = required_u64(call, 3, "GPU buffer")?;
            let mode = required_string(call, 4, "GPU map mode")?;
            let offset = required_u64(call, 5, "GPU map offset")?;
            let bytes = call
                .argument(6)
                .ok_or_else(|| NativeError::new("missing mapped GPU buffer data"))?
                .to_bytes()?;
            state
                .gpu
                .borrow()
                .unmap_buffer(device, buffer, &mode, offset, &bytes)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCopyExternalImageToTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let texture = required_u64(call, 3, "GPU texture")?;
            let mip_level = required_u32(call, 4, "texture mip level")?;
            let destination_origin = [
                required_u32(call, 5, "texture origin x")?,
                required_u32(call, 6, "texture origin y")?,
                required_u32(call, 7, "texture origin z")?,
            ];
            let aspect = required_string(call, 8, "texture aspect")?;
            let flip_y = required_boolean(call, 13, "external image orientation")?;
            let premultiply_alpha = required_boolean(call, 14, "external image alpha conversion")?;
            let source_origin = [
                required_u32(call, 16, "external image origin x")?,
                required_u32(call, 17, "external image origin y")?,
            ];
            let width = required_u32(call, 18, "external image copy width")?;
            let height = required_u32(call, 19, "external image copy height")?;
            let destination_color_space = CanvasColorSpace::parse(&required_string(
                call,
                20,
                "external image destination color space",
            )?)
            .map_err(NativeError::new)?;
            let (source_width, source_height, source_pixels) =
                gpu_external_rgba_texture_source(state, call, 9, destination_color_space)?;
            let mut pixels = crop_rgba(
                &source_pixels,
                source_width,
                source_height,
                source_origin[0],
                source_origin[1],
                width,
                height,
            )?;
            if flip_y {
                flip_rows(&mut pixels, width, height);
            }
            if premultiply_alpha {
                premultiply_rgba(&mut pixels);
            }
            state
                .gpu
                .borrow()
                .copy_external_rgba_texture(
                    device,
                    texture,
                    mip_level,
                    destination_origin,
                    &aspect,
                    pixels,
                    width,
                    height,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuMapBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let buffer = required_u64(call, 3, "GPU buffer")?;
            let mode = required_string(call, 4, "GPU map mode")?;
            let offset = required_u64(call, 5, "map offset")?;
            let size = required_u64(call, 6, "map size")?;
            let bytes = state
                .gpu
                .borrow()
                .map_buffer(device, buffer, &mode, offset, size)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Bytes(bytes))
        }
        "gpuDestroyBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let buffer = required_u64(call, 3, "GPU buffer")?;
            state
                .gpu
                .borrow_mut()
                .destroy_buffer(device, buffer)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCreateTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let width = required_u32(call, 3, "texture width")?;
            let height = required_u32(call, 4, "texture height")?;
            let depth = required_u32(call, 5, "texture depth or array layers")?;
            let mip_levels = required_u32(call, 6, "texture mip level count")?;
            let samples = required_u32(call, 7, "texture sample count")?;
            let dimension = required_string(call, 8, "texture dimension")?;
            let format = required_string(call, 9, "texture format")?;
            let usage = required_u32(call, 10, "texture usage")?;
            let view_formats = required_string(call, 11, "texture view formats")?;
            let view_formats: Vec<String> =
                serde_json::from_str(&view_formats).map_err(|error| {
                    NativeError::new(format!("invalid GPU texture view formats: {error}"))
                })?;
            let label = required_string(call, 12, "GPU texture label")?;
            let texture = state
                .gpu
                .borrow_mut()
                .create_texture(
                    device,
                    width,
                    height,
                    depth,
                    mip_levels,
                    samples,
                    &dimension,
                    &format,
                    usage,
                    &view_formats,
                    &label,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(texture as f64))
        }
        "gpuDestroyTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let texture = required_u64(call, 3, "GPU texture")?;
            state
                .gpu
                .borrow_mut()
                .destroy_texture(device, texture)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCreateTextureView" => {
            let device = required_u64(call, 2, "GPU device")?;
            let texture = required_u64(call, 3, "GPU texture")?;
            let encoded = required_string(call, 4, "GPU texture view descriptor")?;
            let descriptor: GpuTextureViewDescriptor =
                serde_json::from_str(&encoded).map_err(|error| {
                    NativeError::new(format!("invalid GPU texture view descriptor: {error}"))
                })?;
            let view = state
                .gpu
                .borrow_mut()
                .create_texture_view(device, texture, &descriptor)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(view as f64))
        }
        "gpuCreateSampler" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoded = required_string(call, 3, "GPU sampler descriptor")?;
            let descriptor: GpuSamplerDescriptor = serde_json::from_str(&encoded)
                .map_err(|error| NativeError::new(format!("invalid GPU sampler: {error}")))?;
            let sampler = state
                .gpu
                .borrow_mut()
                .create_sampler(device, &descriptor)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(sampler as f64))
        }
        "gpuCreateCommandEncoder" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = state
                .gpu
                .borrow_mut()
                .create_command_encoder(device)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(encoder as f64))
        }
        "gpuCommandEncoderInsertDebugMarker" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let marker_label = required_string(call, 4, "GPU debug marker label")?;
            state
                .gpu
                .borrow_mut()
                .command_encoder_insert_debug_marker(device, encoder, &marker_label)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCommandEncoderPushDebugGroup" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let group_label = required_string(call, 4, "GPU debug group label")?;
            state
                .gpu
                .borrow_mut()
                .command_encoder_push_debug_group(device, encoder, &group_label)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCommandEncoderPopDebugGroup" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            state
                .gpu
                .borrow_mut()
                .command_encoder_pop_debug_group(device, encoder)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCreateShaderModule" => {
            let device = required_u64(call, 2, "GPU device")?;
            let code = required_string(call, 3, "WGSL shader code")?;
            let module = state
                .gpu
                .borrow_mut()
                .create_shader_module(device, &code)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(module as f64))
        }
        "gpuShaderCompilationInfo" => {
            let device = required_u64(call, 2, "GPU device")?;
            let shader = required_u64(call, 3, "GPU shader module")?;
            let messages = state
                .gpu
                .borrow()
                .shader_compilation_info(device, shader)
                .map_err(NativeError::new)?;
            Ok(NativeValue::String(
                serde_json::to_string(&messages).map_err(|error| {
                    NativeError::new(format!("could not encode shader compilation info: {error}"))
                })?,
            ))
        }
        "gpuCreateQuerySet" => {
            let device = required_u64(call, 2, "GPU device")?;
            let query_type = required_string(call, 3, "GPU query type")?;
            let count = required_u32(call, 4, "GPU query count")?;
            let query_set = state
                .gpu
                .borrow_mut()
                .create_query_set(device, &query_type, count)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(query_set as f64))
        }
        "gpuDestroyQuerySet" => {
            let device = required_u64(call, 2, "GPU device")?;
            let query_set = required_u64(call, 3, "GPU query set")?;
            state
                .gpu
                .borrow_mut()
                .destroy_query_set(device, query_set)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCreateBindGroupLayout" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoded = required_string(call, 3, "GPU bind group layout entries")?;
            let entries: Vec<GpuBindGroupLayoutEntry> =
                serde_json::from_str(&encoded).map_err(|error| {
                    NativeError::new(format!("invalid GPU bind group layout entries: {error}"))
                })?;
            let layout = state
                .gpu
                .borrow_mut()
                .create_bind_group_layout(device, &entries)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(layout as f64))
        }
        "gpuCreatePipelineLayout" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoded = required_string(call, 3, "GPU bind group layouts")?;
            let immediate_size = required_u32(call, 4, "GPU immediate data size")?;
            let layouts: Vec<u64> = serde_json::from_str(&encoded).map_err(|error| {
                NativeError::new(format!("invalid GPU pipeline layout: {error}"))
            })?;
            let layout = state
                .gpu
                .borrow_mut()
                .create_pipeline_layout(device, &layouts, immediate_size)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(layout as f64))
        }
        "gpuCreateComputePipeline" => {
            let device = required_u64(call, 2, "GPU device")?;
            let module = required_u64(call, 3, "GPU shader module")?;
            let entry_point = required_string(call, 4, "compute entry point")?;
            let layout = required_u64(call, 5, "GPU pipeline layout")?;
            let pipeline = state
                .gpu
                .borrow_mut()
                .create_compute_pipeline(
                    device,
                    (layout != 0).then_some(layout),
                    module,
                    &entry_point,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(pipeline as f64))
        }
        "gpuCreateRenderPipeline" => {
            let device = required_u64(call, 2, "GPU device")?;
            let vertex_module = required_u64(call, 3, "vertex GPU shader module")?;
            let vertex_entry = required_string(call, 4, "vertex entry point")?;
            let fragment_module = required_u64(call, 5, "fragment GPU shader module")?;
            let fragment_entry = required_string(call, 6, "fragment entry point")?;
            let vertex_buffers = required_string(call, 7, "GPU vertex buffer layouts")?;
            let vertex_buffers: Vec<GpuVertexBufferLayout> = serde_json::from_str(&vertex_buffers)
                .map_err(|error| {
                    NativeError::new(format!("invalid GPU vertex buffer layouts: {error}"))
                })?;
            let target_descriptors = required_string(call, 8, "GPU color targets")?;
            let target_descriptors: Vec<Option<GpuColorTarget>> =
                serde_json::from_str(&target_descriptors).map_err(|error| {
                    NativeError::new(format!("invalid GPU color targets: {error}"))
                })?;
            let primitive = required_string(call, 9, "GPU primitive state")?;
            let primitive: GpuPrimitiveState =
                serde_json::from_str(&primitive).map_err(|error| {
                    NativeError::new(format!("invalid GPU primitive state: {error}"))
                })?;
            let depth_stencil = required_string(call, 10, "GPU depth stencil state")?;
            let depth_stencil: Option<GpuDepthStencilState> = serde_json::from_str(&depth_stencil)
                .map_err(|error| {
                    NativeError::new(format!("invalid GPU depth stencil state: {error}"))
                })?;
            let layout = required_u64(call, 11, "GPU pipeline layout")?;
            let multisample = required_string(call, 12, "GPU multisample state")?;
            let multisample: GpuMultisampleState =
                serde_json::from_str(&multisample).map_err(|error| {
                    NativeError::new(format!("invalid GPU multisample state: {error}"))
                })?;
            let pipeline = state
                .gpu
                .borrow_mut()
                .create_render_pipeline(
                    device,
                    (layout != 0).then_some(layout),
                    vertex_module,
                    &vertex_entry,
                    fragment_module,
                    &fragment_entry,
                    &vertex_buffers,
                    &target_descriptors,
                    depth_stencil.as_ref(),
                    &multisample,
                    &primitive,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(pipeline as f64))
        }
        "gpuComputeBindGroupLayout" => {
            let device = required_u64(call, 2, "GPU device")?;
            let pipeline = required_u64(call, 3, "GPU compute pipeline")?;
            let index = required_u32(call, 4, "bind group layout index")?;
            let layout = state
                .gpu
                .borrow_mut()
                .compute_bind_group_layout(device, pipeline, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(layout as f64))
        }
        "gpuRenderBindGroupLayout" => {
            let device = required_u64(call, 2, "GPU device")?;
            let pipeline = required_u64(call, 3, "GPU render pipeline")?;
            let index = required_u32(call, 4, "bind group layout index")?;
            let layout = state
                .gpu
                .borrow_mut()
                .render_bind_group_layout(device, pipeline, index)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(layout as f64))
        }
        "gpuCreateBindGroup" => {
            let device = required_u64(call, 2, "GPU device")?;
            let layout = required_u64(call, 3, "GPU bind group layout")?;
            let encoded = required_string(call, 4, "GPU bind group entries")?;
            let entries: Vec<GpuBindGroupEntry> =
                serde_json::from_str(&encoded).map_err(|error| {
                    NativeError::new(format!("invalid GPU bind group entries: {error}"))
                })?;
            let group = state
                .gpu
                .borrow_mut()
                .create_bind_group(device, layout, &entries)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(group as f64))
        }
        "gpuEncodeComputePass" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let commands = required_string(call, 4, "GPU compute commands")?;
            let timestamp_writes = required_string(call, 5, "GPU compute timestamp writes")?;
            let commands: Vec<GpuComputeCommand> =
                serde_json::from_str(&commands).map_err(|error| {
                    NativeError::new(format!("invalid GPU compute commands: {error}"))
                })?;
            let timestamp_writes: Option<GpuTimestampWrites> =
                serde_json::from_str(&timestamp_writes).map_err(|error| {
                    NativeError::new(format!("invalid GPU compute timestamp writes: {error}"))
                })?;
            state
                .gpu
                .borrow_mut()
                .encode_compute_pass(device, encoder, &commands, timestamp_writes.as_ref())
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCreateRenderBundle" => {
            let device = required_u64(call, 2, "GPU device")?;
            let descriptor = required_string(call, 3, "GPU render bundle descriptor")?;
            let commands = required_string(call, 4, "GPU render bundle commands")?;
            let label = required_string(call, 5, "GPU render bundle label")?;
            let descriptor: GpuRenderBundleEncoderDescriptor = serde_json::from_str(&descriptor)
                .map_err(|error| {
                    NativeError::new(format!("invalid GPU render bundle descriptor: {error}"))
                })?;
            let commands: Vec<GpuRenderCommand> =
                serde_json::from_str(&commands).map_err(|error| {
                    NativeError::new(format!("invalid GPU render bundle commands: {error}"))
                })?;
            let bundle = state
                .gpu
                .borrow_mut()
                .create_render_bundle(device, &descriptor, &commands, &label)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(bundle as f64))
        }
        "gpuEncodeRenderPass" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let attachments = required_string(call, 4, "GPU render attachments")?;
            let depth_stencil_attachment =
                required_string(call, 5, "GPU depth stencil attachment")?;
            let commands = required_string(call, 6, "GPU render commands")?;
            let occlusion_query_set = required_u64(call, 7, "GPU occlusion query set")?;
            let timestamp_writes = required_string(call, 8, "GPU render timestamp writes")?;
            let attachments: Vec<GpuColorAttachment> =
                serde_json::from_str(&attachments).map_err(|error| {
                    NativeError::new(format!("invalid GPU render attachments: {error}"))
                })?;
            let commands: Vec<GpuRenderCommand> =
                serde_json::from_str(&commands).map_err(|error| {
                    NativeError::new(format!("invalid GPU render commands: {error}"))
                })?;
            let depth_stencil_attachment: Option<GpuDepthStencilAttachment> =
                serde_json::from_str(&depth_stencil_attachment).map_err(|error| {
                    NativeError::new(format!("invalid GPU depth stencil attachment: {error}"))
                })?;
            let timestamp_writes: Option<GpuTimestampWrites> =
                serde_json::from_str(&timestamp_writes).map_err(|error| {
                    NativeError::new(format!("invalid GPU render timestamp writes: {error}"))
                })?;
            state
                .gpu
                .borrow_mut()
                .encode_render_pass(
                    device,
                    encoder,
                    &attachments,
                    depth_stencil_attachment.as_ref(),
                    (occlusion_query_set != 0).then_some(occlusion_query_set),
                    timestamp_writes.as_ref(),
                    &commands,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCopyBufferToBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let source = required_u64(call, 4, "source GPU buffer")?;
            let source_offset = required_u64(call, 5, "source offset")?;
            let destination = required_u64(call, 6, "destination GPU buffer")?;
            let destination_offset = required_u64(call, 7, "destination offset")?;
            let size = required_u64(call, 8, "copy size")?;
            state
                .gpu
                .borrow_mut()
                .copy_buffer_to_buffer(
                    device,
                    encoder,
                    source,
                    source_offset,
                    destination,
                    destination_offset,
                    size,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuClearBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let buffer = required_u64(call, 4, "GPU buffer")?;
            let offset = required_u64(call, 5, "clear offset")?;
            let has_size = required_boolean(call, 6, "clear size presence")?;
            let size = required_u64(call, 7, "clear size")?;
            state
                .gpu
                .borrow_mut()
                .clear_buffer(device, encoder, buffer, offset, has_size.then_some(size))
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCopyBufferToTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let buffer = required_u64(call, 4, "source GPU buffer")?;
            let offset = required_u64(call, 5, "source data offset")?;
            let bytes_per_row = required_u32(call, 6, "source bytesPerRow")?;
            let rows_per_image = required_u32(call, 7, "source rowsPerImage")?;
            let texture = required_u64(call, 8, "destination GPU texture")?;
            let mip_level = required_u32(call, 9, "destination mip level")?;
            let origin =
                required_numbers::<3>(call, 10, "destination origin")?.map(|value| value as u32);
            let extent = required_numbers::<3>(call, 13, "copy extent")?.map(|value| value as u32);
            state
                .gpu
                .borrow_mut()
                .copy_buffer_to_texture(
                    device,
                    encoder,
                    buffer,
                    offset,
                    (bytes_per_row != 0).then_some(bytes_per_row),
                    (rows_per_image != 0).then_some(rows_per_image),
                    texture,
                    mip_level,
                    origin,
                    extent,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCopyTextureToBuffer" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let texture = required_u64(call, 4, "source GPU texture")?;
            let mip_level = required_u32(call, 5, "source mip level")?;
            let origin = required_numbers::<3>(call, 6, "source origin")?.map(|value| value as u32);
            let buffer = required_u64(call, 9, "destination GPU buffer")?;
            let offset = required_u64(call, 10, "destination data offset")?;
            let bytes_per_row = required_u32(call, 11, "destination bytesPerRow")?;
            let rows_per_image = required_u32(call, 12, "destination rowsPerImage")?;
            let extent = required_numbers::<3>(call, 13, "copy extent")?.map(|value| value as u32);
            state
                .gpu
                .borrow_mut()
                .copy_texture_to_buffer(
                    device,
                    encoder,
                    texture,
                    mip_level,
                    origin,
                    buffer,
                    offset,
                    (bytes_per_row != 0).then_some(bytes_per_row),
                    (rows_per_image != 0).then_some(rows_per_image),
                    extent,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuCopyTextureToTexture" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let source = required_u64(call, 4, "source GPU texture")?;
            let source_mip_level = required_u32(call, 5, "source mip level")?;
            let source_origin =
                required_numbers::<3>(call, 6, "source origin")?.map(|value| value as u32);
            let source_aspect = required_string(call, 9, "source texture aspect")?;
            let destination = required_u64(call, 10, "destination GPU texture")?;
            let destination_mip_level = required_u32(call, 11, "destination mip level")?;
            let destination_origin =
                required_numbers::<3>(call, 12, "destination origin")?.map(|value| value as u32);
            let destination_aspect = required_string(call, 15, "destination texture aspect")?;
            let extent = required_numbers::<3>(call, 16, "copy extent")?.map(|value| value as u32);
            state
                .gpu
                .borrow_mut()
                .copy_texture_to_texture(
                    device,
                    encoder,
                    source,
                    source_mip_level,
                    source_origin,
                    &source_aspect,
                    destination,
                    destination_mip_level,
                    destination_origin,
                    &destination_aspect,
                    extent,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuResolveQuerySet" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let query_set = required_u64(call, 4, "GPU query set")?;
            let first_query = required_u32(call, 5, "first GPU query")?;
            let query_count = required_u32(call, 6, "GPU query count")?;
            let destination = required_u64(call, 7, "destination GPU buffer")?;
            let destination_offset = required_u64(call, 8, "destination GPU buffer offset")?;
            state
                .gpu
                .borrow_mut()
                .resolve_query_set(
                    device,
                    encoder,
                    query_set,
                    first_query,
                    query_count,
                    destination,
                    destination_offset,
                )
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuFinishCommandEncoder" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoder = required_u64(call, 3, "GPU command encoder")?;
            let command = state
                .gpu
                .borrow_mut()
                .finish_command_encoder(device, encoder)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Number(command as f64))
        }
        "gpuSubmit" => {
            let device = required_u64(call, 2, "GPU device")?;
            let encoded = required_string(call, 3, "GPU command buffers")?;
            let commands: Vec<u64> = serde_json::from_str(&encoded).map_err(|error| {
                NativeError::new(format!("invalid GPU command buffers: {error}"))
            })?;
            state
                .gpu
                .borrow_mut()
                .submit(device, &commands)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        "gpuWaitForSubmittedWork" => {
            let device = required_u64(call, 2, "GPU device")?;
            state
                .gpu
                .borrow()
                .wait_for_submitted_work(device)
                .map_err(NativeError::new)?;
            Ok(NativeValue::Undefined)
        }
        _ => Err(NativeError::new(format!(
            "unknown native WebGPU operation: {operation}"
        ))),
    }
}
