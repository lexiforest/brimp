use std::{
    borrow::Cow,
    collections::HashMap,
    num::NonZeroU64,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use wgpu::{
    Adapter, AddressMode, AstcBlock, AstcChannel, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandBuffer, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    ComputePassDescriptor, ComputePassTimestampWrites, ComputePipeline, ComputePipelineDescriptor,
    DepthBiasState, DepthStencilState, Device, DeviceDescriptor, DeviceLostReason, Error,
    ErrorFilter, ErrorScopeGuard, Extent3d, Face, Features, FilterMode, FragmentState, FrontFace,
    IndexFormat, Instance, LoadOp, MapMode, MipmapFilterMode, MultisampleState, Operations,
    Origin3d, PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PollType,
    PowerPreference, PrimitiveState, PrimitiveTopology, QuerySet, QuerySetDescriptor, QueryType,
    Queue, RenderBundle, RenderBundleDepthStencil, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPassTimestampWrites, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilFaceState, StencilOperation,
    StencilState, StorageTextureAccess, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

struct DeviceState {
    device: Device,
    queue: Queue,
    buffers: HashMap<u64, Buffer>,
    textures: HashMap<u64, TextureRecord>,
    texture_views: HashMap<u64, TextureViewRecord>,
    samplers: HashMap<u64, Sampler>,
    encoders: HashMap<u64, CommandEncoder>,
    command_buffers: HashMap<u64, CommandBuffer>,
    shaders: HashMap<u64, ShaderRecord>,
    compute_pipelines: HashMap<u64, ComputePipeline>,
    render_pipelines: HashMap<u64, RenderPipeline>,
    render_bundles: HashMap<u64, RenderBundle>,
    bind_group_layouts: HashMap<u64, BindGroupLayout>,
    pipeline_layouts: HashMap<u64, PipelineLayout>,
    bind_groups: HashMap<u64, BindGroup>,
    query_sets: HashMap<u64, QuerySet>,
    uncaptured_errors: Arc<Mutex<Vec<(String, String)>>>,
    lost: Arc<Mutex<Option<(String, String)>>>,
    error_scopes: Vec<ErrorScopeGuard>,
    next_buffer: u64,
    next_command: u64,
    next_resource: u64,
}

struct TextureRecord {
    texture: Texture,
    format: TextureFormat,
}

struct TextureViewRecord {
    view: TextureView,
    texture: u64,
}

struct ShaderRecord {
    module: ShaderModule,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GpuCompilationMessage {
    message: String,
    #[serde(rename = "type")]
    message_type: String,
    line_num: u64,
    line_pos: u64,
    offset: u64,
    length: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GpuSamplerDescriptor {
    address_mode_u: String,
    address_mode_v: String,
    address_mode_w: String,
    mag_filter: String,
    min_filter: String,
    mipmap_filter: String,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: Option<String>,
    max_anisotropy: u16,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum GpuBindGroupEntry {
    Buffer {
        binding: u32,
        resource: u64,
        offset: u64,
        size: Option<u64>,
    },
    Sampler {
        binding: u32,
        resource: u64,
    },
    TextureView {
        binding: u32,
        resource: u64,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum GpuBindGroupLayoutEntry {
    Buffer {
        binding: u32,
        visibility: u32,
        ty: String,
        has_dynamic_offset: bool,
        min_binding_size: Option<u64>,
    },
    Sampler {
        binding: u32,
        visibility: u32,
        ty: String,
    },
    Texture {
        binding: u32,
        visibility: u32,
        sample_type: String,
        view_dimension: String,
        multisampled: bool,
    },
    StorageTexture {
        binding: u32,
        visibility: u32,
        access: String,
        format: String,
        view_dimension: String,
    },
}

#[derive(Deserialize)]
pub(crate) struct GpuColorTarget {
    format: String,
    blend: Option<GpuBlendState>,
    write_mask: u32,
}

#[derive(Deserialize)]
struct GpuBlendState {
    color: GpuBlendComponent,
    alpha: GpuBlendComponent,
}

#[derive(Deserialize)]
struct GpuBlendComponent {
    src_factor: String,
    dst_factor: String,
    operation: String,
}

#[derive(Deserialize)]
pub(crate) struct GpuDepthStencilState {
    format: String,
    depth_write_enabled: bool,
    depth_compare: String,
    stencil_front: GpuStencilFaceState,
    stencil_back: GpuStencilFaceState,
    stencil_read_mask: u32,
    stencil_write_mask: u32,
    depth_bias: i32,
    depth_bias_slope_scale: f32,
    depth_bias_clamp: f32,
}

#[derive(Deserialize)]
struct GpuStencilFaceState {
    compare: String,
    fail_op: String,
    depth_fail_op: String,
    pass_op: String,
}

#[derive(Deserialize)]
pub(crate) struct GpuDepthStencilAttachment {
    view: u64,
    depth_load: bool,
    depth_clear_value: f32,
    depth_store: bool,
    depth_read_only: bool,
    stencil_load: bool,
    stencil_clear_value: u32,
    stencil_store: bool,
    stencil_read_only: bool,
}

#[derive(Deserialize)]
pub(crate) struct GpuColorAttachment {
    view: u64,
    resolve_target: Option<u64>,
    clear: bool,
    color: [f64; 4],
    store: bool,
}

#[derive(Deserialize)]
pub(crate) struct GpuTimestampWrites {
    query_set: u64,
    beginning_of_pass_write_index: Option<u32>,
    end_of_pass_write_index: Option<u32>,
}

#[derive(Deserialize)]
pub(crate) struct GpuMultisampleState {
    count: u32,
    mask: u64,
    alpha_to_coverage_enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct GpuPrimitiveState {
    topology: String,
    strip_index_format: Option<String>,
    front_face: String,
    cull_mode: String,
    unclipped_depth: bool,
}

#[derive(Deserialize)]
pub(crate) struct GpuTextureViewDescriptor {
    label: String,
    format: Option<String>,
    dimension: Option<String>,
    usage: Option<u32>,
    aspect: String,
    base_mip_level: u32,
    mip_level_count: Option<u32>,
    base_array_layer: u32,
    array_layer_count: Option<u32>,
}

#[derive(Deserialize)]
pub(crate) struct GpuVertexBufferLayout {
    array_stride: u64,
    step_mode: String,
    attributes: Vec<GpuVertexAttribute>,
}

#[derive(Deserialize)]
struct GpuVertexAttribute {
    format: String,
    offset: u64,
    shader_location: u32,
}

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase")]
pub(crate) enum GpuComputeCommand {
    InsertDebugMarker {
        marker_label: String,
    },
    PushDebugGroup {
        group_label: String,
    },
    PopDebugGroup,
    SetImmediates {
        offset: u32,
        data: Vec<u8>,
    },
    SetPipeline {
        pipeline: u64,
    },
    SetBindGroup {
        index: u32,
        group: u64,
        dynamic_offsets: Vec<u32>,
    },
    DispatchWorkgroups {
        x: u32,
        y: u32,
        z: u32,
    },
    DispatchWorkgroupsIndirect {
        buffer: u64,
        offset: u64,
    },
}

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase")]
pub(crate) enum GpuRenderCommand {
    InsertDebugMarker {
        marker_label: String,
    },
    PushDebugGroup {
        group_label: String,
    },
    PopDebugGroup,
    SetImmediates {
        offset: u32,
        data: Vec<u8>,
    },
    SetPipeline {
        pipeline: u64,
    },
    SetBindGroup {
        index: u32,
        group: u64,
        dynamic_offsets: Vec<u32>,
    },
    SetVertexBuffer {
        slot: u32,
        buffer: u64,
        offset: u64,
        size: Option<u64>,
    },
    SetIndexBuffer {
        buffer: u64,
        format: String,
        offset: u64,
        size: Option<u64>,
    },
    Draw {
        vertices: u32,
        instances: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndexed {
        indices: u32,
        instances: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    },
    DrawIndirect {
        buffer: u64,
        offset: u64,
    },
    DrawIndexedIndirect {
        buffer: u64,
        offset: u64,
    },
    ExecuteBundles {
        bundles: Vec<u64>,
    },
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    },
    SetScissorRect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    SetBlendConstant {
        color: [f64; 4],
    },
    SetStencilReference {
        reference: u32,
    },
    BeginOcclusionQuery {
        query_index: u32,
    },
    EndOcclusionQuery,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GpuRenderBundleEncoderDescriptor {
    color_formats: Vec<Option<String>>,
    depth_stencil_format: Option<String>,
    depth_read_only: bool,
    stencil_read_only: bool,
    sample_count: u32,
}

#[derive(Default)]
pub(crate) struct GpuStore {
    instance: Option<Instance>,
    adapters: HashMap<u64, Adapter>,
    devices: HashMap<u64, DeviceState>,
    next_adapter: u64,
    next_device: u64,
}

impl GpuStore {
    pub(crate) fn request_adapter(
        &mut self,
        preference: &str,
        force_fallback: bool,
    ) -> Result<Option<(u64, String)>, String> {
        let power_preference = match preference {
            "low-power" => PowerPreference::LowPower,
            "high-performance" => PowerPreference::HighPerformance,
            _ => PowerPreference::None,
        };
        let instance = self.instance.get_or_insert_with(Instance::default);
        let options = RequestAdapterOptions {
            power_preference,
            force_fallback_adapter: force_fallback,
            compatible_surface: None,
            // Browser-facing limits must use wgpu's WebGPU limit buckets rather than exposing
            // raw driver limits as an unnecessarily identifying surface.
            apply_limit_buckets: true,
        };
        let adapter = match pollster::block_on(instance.request_adapter(&options)) {
            Ok(adapter) => adapter,
            Err(_) => return Ok(None),
        };
        self.next_adapter = self
            .next_adapter
            .checked_add(1)
            .ok_or("GPU adapter id overflow")?;
        let id = self.next_adapter;
        let metadata = adapter_metadata(&adapter);
        self.adapters.insert(id, adapter);
        Ok(Some((id, metadata)))
    }

    pub(crate) fn request_device(
        &mut self,
        adapter: u64,
        required_features: &[String],
        required_limits: &HashMap<String, u64>,
        label: &str,
    ) -> Result<(u64, String, Vec<&'static str>), String> {
        let adapter = self.adapters.get(&adapter).ok_or("unknown GPU adapter")?;
        let required_features = gpu_features(required_features)?;
        if !adapter.features().contains(required_features) {
            return Err("GPU adapter does not support every required feature".to_owned());
        }
        let required_limits = gpu_required_limits(required_limits)?;
        let allowed_limits = adapter.limits();
        let mut limit_error = None;
        required_limits.check_limits_with_fail_fn(
            &allowed_limits,
            true,
            |name, requested, allowed| {
                limit_error = Some(format!(
                    "required GPU limit {name} ({requested}) exceeds adapter support ({allowed})"
                ));
            },
        );
        if let Some(error) = limit_error {
            return Err(error);
        }
        let descriptor = DeviceDescriptor {
            label: Some(label),
            required_features,
            required_limits,
            ..Default::default()
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .map_err(|error| format!("could not create GPU device: {error}"))?;
        let uncaptured_errors = Arc::new(Mutex::new(Vec::new()));
        let error_sink = Arc::clone(&uncaptured_errors);
        device.on_uncaptured_error(Arc::new(move |error| {
            if let Ok(mut errors) = error_sink.lock() {
                errors.push(gpu_error_record(error));
            }
        }));
        let lost = Arc::new(Mutex::new(None));
        let lost_sink = Arc::clone(&lost);
        device.set_device_lost_callback(move |reason, message| {
            if let Ok(mut record) = lost_sink.lock() {
                record.get_or_insert_with(|| {
                    let reason = match reason {
                        DeviceLostReason::Destroyed => "destroyed",
                        DeviceLostReason::Unknown => "unknown",
                    };
                    (reason.to_owned(), message)
                });
            }
        });
        self.next_device = self
            .next_device
            .checked_add(1)
            .ok_or("GPU device id overflow")?;
        let id = self.next_device;
        let metadata = limits_json(&device.limits());
        self.devices.insert(
            id,
            DeviceState {
                device,
                queue,
                buffers: HashMap::new(),
                textures: HashMap::new(),
                texture_views: HashMap::new(),
                samplers: HashMap::new(),
                encoders: HashMap::new(),
                command_buffers: HashMap::new(),
                shaders: HashMap::new(),
                compute_pipelines: HashMap::new(),
                render_pipelines: HashMap::new(),
                render_bundles: HashMap::new(),
                bind_group_layouts: HashMap::new(),
                pipeline_layouts: HashMap::new(),
                bind_groups: HashMap::new(),
                query_sets: HashMap::new(),
                uncaptured_errors,
                lost,
                error_scopes: Vec::new(),
                next_buffer: 0,
                next_command: 0,
                next_resource: 0,
            },
        );
        Ok((id, metadata, gpu_feature_names(required_features)))
    }

    pub(crate) fn destroy_device(&self, device: u64) -> Result<(), String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        {
            let mut lost = state
                .lost
                .lock()
                .map_err(|_| "GPU device loss state is unavailable")?;
            lost.get_or_insert_with(|| {
                (
                    "destroyed".to_owned(),
                    "GPUDevice.destroy() was called".to_owned(),
                )
            });
        }
        state.device.destroy();
        Ok(())
    }

    pub(crate) fn take_device_lost(&self, device: u64) -> Result<Option<(String, String)>, String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let mut lost = state
            .lost
            .lock()
            .map_err(|_| "GPU device loss state is unavailable")?;
        Ok(lost.take())
    }

    pub(crate) fn push_error_scope(&mut self, device: u64, filter: &str) -> Result<(), String> {
        let filter = match filter {
            "validation" => ErrorFilter::Validation,
            "out-of-memory" => ErrorFilter::OutOfMemory,
            "internal" => ErrorFilter::Internal,
            _ => return Err("invalid GPU error filter".to_owned()),
        };
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        state
            .error_scopes
            .push(state.device.push_error_scope(filter));
        Ok(())
    }

    pub(crate) fn pop_error_scope(
        &mut self,
        device: u64,
    ) -> Result<Option<(String, String)>, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let scope = state
            .error_scopes
            .pop()
            .ok_or("GPU error scope stack is empty")?;
        let error = pollster::block_on(scope.pop());
        Ok(error.map(gpu_error_record))
    }

    pub(crate) fn take_uncaptured_errors(
        &self,
        device: u64,
    ) -> Result<Vec<(String, String)>, String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let mut errors = state
            .uncaptured_errors
            .lock()
            .map_err(|_| "GPU uncaptured error queue is unavailable")?;
        Ok(std::mem::take(&mut *errors))
    }

    pub(crate) fn create_buffer(
        &mut self,
        device: u64,
        size: u64,
        usage: u32,
        mapped_at_creation: bool,
    ) -> Result<u64, String> {
        if size == 0 {
            return Err("GPUBuffer size must be greater than zero".to_owned());
        }
        let usage =
            BufferUsages::from_bits(usage).ok_or("GPUBuffer usage contains unknown bits")?;
        if usage.is_empty() {
            return Err("GPUBuffer usage must not be zero".to_owned());
        }
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let buffer = state.device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation,
        });
        state.next_buffer = state
            .next_buffer
            .checked_add(1)
            .ok_or("GPU buffer id overflow")?;
        let id = state.next_buffer;
        state.buffers.insert(id, buffer);
        Ok(id)
    }

    pub(crate) fn write_buffer(
        &self,
        device: u64,
        buffer: u64,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPU buffer")?;
        state.queue.write_buffer(buffer, offset, bytes);
        Ok(())
    }

    pub(crate) fn unmap_buffer(
        &self,
        device: u64,
        buffer: u64,
        mode: &str,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPU buffer")?;
        match mode {
            "read" => {}
            "write" => {
                let end = offset
                    .checked_add(bytes.len() as u64)
                    .ok_or("GPU buffer mapped range overflow")?;
                let mut mapped = buffer
                    .get_mapped_range_mut(offset..end)
                    .map_err(|error| format!("GPU buffer is not mapped for writing: {error}"))?;
                mapped.copy_from_slice(bytes);
            }
            _ => return Err("invalid GPU buffer map mode".to_owned()),
        }
        buffer.unmap();
        Ok(())
    }

    pub(crate) fn map_buffer(
        &self,
        device: u64,
        buffer: u64,
        mode: &str,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPU buffer")?;
        let end = offset
            .checked_add(size)
            .ok_or("GPU buffer map range overflow")?;
        let mode = match mode {
            "read" => MapMode::Read,
            "write" => MapMode::Write,
            _ => return Err("invalid GPU buffer map mode".to_owned()),
        };
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        buffer.map_async(mode, offset..end, move |result| {
            let _ = sender.send(result.map_err(|error| error.to_string()));
        });
        state
            .device
            .poll(PollType::wait_indefinitely())
            .map_err(|error| format!("could not poll GPU device: {error}"))?;
        receiver
            .recv()
            .map_err(|_| "GPU buffer mapping callback was dropped".to_owned())??;
        let mapped = buffer
            .get_mapped_range(offset..end)
            .map_err(|error| format!("could not read mapped GPU buffer: {error}"))?;
        Ok(mapped.to_vec())
    }

    pub(crate) fn create_command_encoder(&mut self, device: u64) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        state.next_command = state
            .next_command
            .checked_add(1)
            .ok_or("GPU command id overflow")?;
        let id = state.next_command;
        state.encoders.insert(id, encoder);
        Ok(id)
    }

    pub(crate) fn command_encoder_insert_debug_marker(
        &mut self,
        device: u64,
        encoder: u64,
        marker_label: &str,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?
            .insert_debug_marker(marker_label);
        Ok(())
    }

    pub(crate) fn command_encoder_push_debug_group(
        &mut self,
        device: u64,
        encoder: u64,
        group_label: &str,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?
            .push_debug_group(group_label);
        Ok(())
    }

    pub(crate) fn command_encoder_pop_debug_group(
        &mut self,
        device: u64,
        encoder: u64,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?
            .pop_debug_group();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_texture(
        &mut self,
        device: u64,
        width: u32,
        height: u32,
        depth: u32,
        mip_levels: u32,
        samples: u32,
        dimension: &str,
        format: &str,
        usage: u32,
        view_formats: &[String],
        label: &str,
    ) -> Result<u64, String> {
        let dimension = texture_dimension(dimension)?;
        let format = texture_format(format)?;
        let view_formats = view_formats
            .iter()
            .map(|format| texture_format(format))
            .collect::<Result<Vec<_>, _>>()?;
        let usage =
            TextureUsages::from_bits(usage).ok_or("GPUTexture usage contains unknown bits")?;
        if usage.is_empty() {
            return Err("GPUTexture usage must not be zero".to_owned());
        }
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let texture = state.device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            mip_level_count: mip_levels,
            sample_count: samples,
            dimension,
            format,
            usage,
            view_formats: &view_formats,
        });
        let id = next_resource(state)?;
        state.textures.insert(id, TextureRecord { texture, format });
        Ok(id)
    }

    pub(crate) fn create_texture_view(
        &mut self,
        device: u64,
        texture: u64,
        descriptor: &GpuTextureViewDescriptor,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let texture_id = texture;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        let format = descriptor
            .format
            .as_deref()
            .map(texture_format)
            .transpose()?;
        let dimension = descriptor
            .dimension
            .as_deref()
            .map(texture_view_dimension)
            .transpose()?;
        let usage = descriptor
            .usage
            .map(|usage| {
                TextureUsages::from_bits(usage)
                    .ok_or("GPUTextureView usage contains unknown bits".to_owned())
            })
            .transpose()?;
        let view = texture.texture.create_view(&TextureViewDescriptor {
            label: Some(&descriptor.label),
            format,
            dimension,
            usage,
            aspect: texture_aspect(&descriptor.aspect)?,
            base_mip_level: descriptor.base_mip_level,
            mip_level_count: descriptor.mip_level_count,
            base_array_layer: descriptor.base_array_layer,
            array_layer_count: descriptor.array_layer_count,
        });
        let id = next_resource(state)?;
        state.texture_views.insert(
            id,
            TextureViewRecord {
                view,
                texture: texture_id,
            },
        );
        Ok(id)
    }

    pub(crate) fn create_sampler(
        &mut self,
        device: u64,
        descriptor: &GpuSamplerDescriptor,
    ) -> Result<u64, String> {
        if !descriptor.lod_min_clamp.is_finite()
            || !descriptor.lod_max_clamp.is_finite()
            || descriptor.lod_min_clamp < 0.0
            || descriptor.lod_max_clamp < descriptor.lod_min_clamp
            || descriptor.max_anisotropy == 0
        {
            return Err("invalid GPUSampler LOD or anisotropy value".to_owned());
        }
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let sampler = state.device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: address_mode(&descriptor.address_mode_u)?,
            address_mode_v: address_mode(&descriptor.address_mode_v)?,
            address_mode_w: address_mode(&descriptor.address_mode_w)?,
            mag_filter: filter_mode(&descriptor.mag_filter)?,
            min_filter: filter_mode(&descriptor.min_filter)?,
            mipmap_filter: mipmap_filter_mode(&descriptor.mipmap_filter)?,
            lod_min_clamp: descriptor.lod_min_clamp,
            lod_max_clamp: descriptor.lod_max_clamp,
            compare: descriptor
                .compare
                .as_deref()
                .map(compare_function)
                .transpose()?,
            anisotropy_clamp: descriptor.max_anisotropy,
            border_color: None,
        });
        let id = next_resource(state)?;
        state.samplers.insert(id, sampler);
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn write_texture(
        &self,
        device: u64,
        texture: u64,
        mip_level: u32,
        origin: [u32; 3],
        aspect: &str,
        bytes: &[u8],
        offset: u64,
        bytes_per_row: Option<u32>,
        rows_per_image: Option<u32>,
        size: [u32; 3],
    ) -> Result<(), String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        if size.contains(&0) {
            return Err("GPU texture write size must be non-zero".to_owned());
        }
        state.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level,
                origin: Origin3d {
                    x: origin[0],
                    y: origin[1],
                    z: origin[2],
                },
                aspect: texture_aspect(aspect)?,
            },
            bytes,
            TexelCopyBufferLayout {
                offset,
                bytes_per_row,
                rows_per_image,
            },
            Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: size[2],
            },
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_external_rgba_texture(
        &self,
        device: u64,
        texture: u64,
        mip_level: u32,
        origin: [u32; 3],
        aspect: &str,
        mut pixels: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Err("GPU external image copy size must be non-zero".to_owned());
        }
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        match texture.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {}
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => {
                for pixel in pixels.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                }
            }
            _ => {
                return Err(
                    "external image copies currently require an RGBA8 or BGRA8 texture".to_owned(),
                );
            }
        }
        let bytes_per_row = width
            .checked_mul(4)
            .ok_or("GPU external image row size overflow")?;
        state.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level,
                origin: Origin3d {
                    x: origin[0],
                    y: origin[1],
                    z: origin[2],
                },
                aspect: texture_aspect(aspect)?,
            },
            &pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_buffer_to_buffer(
        &mut self,
        device: u64,
        encoder: u64,
        source: u64,
        source_offset: u64,
        destination: u64,
        destination_offset: u64,
        size: u64,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let source = state
            .buffers
            .get(&source)
            .ok_or("unknown source GPU buffer")?;
        let destination = state
            .buffers
            .get(&destination)
            .ok_or("unknown destination GPU buffer")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.copy_buffer_to_buffer(source, source_offset, destination, destination_offset, size);
        Ok(())
    }

    pub(crate) fn clear_buffer(
        &mut self,
        device: u64,
        encoder: u64,
        buffer: u64,
        offset: u64,
        size: Option<u64>,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPUBuffer")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.clear_buffer(buffer, offset, size);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_buffer_to_texture(
        &mut self,
        device: u64,
        encoder: u64,
        buffer: u64,
        offset: u64,
        bytes_per_row: Option<u32>,
        rows_per_image: Option<u32>,
        texture: u64,
        mip_level: u32,
        origin: [u32; 3],
        extent: [u32; 3],
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPUBuffer")?;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.copy_buffer_to_texture(
            TexelCopyBufferInfo {
                buffer,
                layout: TexelCopyBufferLayout {
                    offset,
                    bytes_per_row,
                    rows_per_image,
                },
            },
            TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level,
                origin: Origin3d {
                    x: origin[0],
                    y: origin[1],
                    z: origin[2],
                },
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: extent[2],
            },
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_texture_to_buffer(
        &mut self,
        device: u64,
        encoder: u64,
        texture: u64,
        mip_level: u32,
        origin: [u32; 3],
        buffer: u64,
        offset: u64,
        bytes_per_row: Option<u32>,
        rows_per_image: Option<u32>,
        extent: [u32; 3],
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        let buffer = state.buffers.get(&buffer).ok_or("unknown GPUBuffer")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level,
                origin: Origin3d {
                    x: origin[0],
                    y: origin[1],
                    z: origin[2],
                },
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer,
                layout: TexelCopyBufferLayout {
                    offset,
                    bytes_per_row,
                    rows_per_image,
                },
            },
            Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: extent[2],
            },
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_texture_to_texture(
        &mut self,
        device: u64,
        encoder: u64,
        source: u64,
        source_mip_level: u32,
        source_origin: [u32; 3],
        source_aspect: &str,
        destination: u64,
        destination_mip_level: u32,
        destination_origin: [u32; 3],
        destination_aspect: &str,
        extent: [u32; 3],
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let source = state
            .textures
            .get(&source)
            .ok_or("unknown source GPUTexture")?;
        let destination = state
            .textures
            .get(&destination)
            .ok_or("unknown destination GPUTexture")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: &source.texture,
                mip_level: source_mip_level,
                origin: Origin3d {
                    x: source_origin[0],
                    y: source_origin[1],
                    z: source_origin[2],
                },
                aspect: texture_aspect(source_aspect)?,
            },
            TexelCopyTextureInfo {
                texture: &destination.texture,
                mip_level: destination_mip_level,
                origin: Origin3d {
                    x: destination_origin[0],
                    y: destination_origin[1],
                    z: destination_origin[2],
                },
                aspect: texture_aspect(destination_aspect)?,
            },
            Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: extent[2],
            },
        );
        Ok(())
    }

    pub(crate) fn finish_command_encoder(
        &mut self,
        device: u64,
        encoder: u64,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let encoder = state
            .encoders
            .remove(&encoder)
            .ok_or("unknown GPU command encoder")?;
        state.next_command = state
            .next_command
            .checked_add(1)
            .ok_or("GPU command id overflow")?;
        let id = state.next_command;
        state.command_buffers.insert(id, encoder.finish());
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_query_set(
        &mut self,
        device: u64,
        encoder: u64,
        query_set: u64,
        first_query: u32,
        query_count: u32,
        destination: u64,
        destination_offset: u64,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let query_set = state
            .query_sets
            .get(&query_set)
            .ok_or("unknown GPUQuerySet")?;
        let destination = state
            .buffers
            .get(&destination)
            .ok_or("unknown destination GPUBuffer")?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        encoder.resolve_query_set(
            query_set,
            first_query..first_query.saturating_add(query_count),
            destination,
            destination_offset,
        );
        Ok(())
    }

    pub(crate) fn submit(&mut self, device: u64, commands: &[u64]) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let mut buffers = Vec::with_capacity(commands.len());
        for command in commands {
            buffers.push(
                state
                    .command_buffers
                    .remove(command)
                    .ok_or("unknown GPU command buffer")?,
            );
        }
        state.queue.submit(buffers);
        Ok(())
    }

    pub(crate) fn wait_for_submitted_work(&self, device: u64) -> Result<(), String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        state
            .device
            .poll(PollType::wait_indefinitely())
            .map_err(|error| format!("could not wait for submitted GPU work: {error}"))?;
        Ok(())
    }

    pub(crate) fn create_shader_module(&mut self, device: u64, code: &str) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let module = state.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(code)),
        });
        let id = next_resource(state)?;
        state.shaders.insert(
            id,
            ShaderRecord {
                module,
                source: code.to_owned(),
            },
        );
        Ok(id)
    }

    pub(crate) fn shader_compilation_info(
        &self,
        device: u64,
        shader: u64,
    ) -> Result<Vec<GpuCompilationMessage>, String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let shader = state
            .shaders
            .get(&shader)
            .ok_or("unknown GPUShaderModule")?;
        let info = pollster::block_on(shader.module.get_compilation_info());
        Ok(info
            .messages
            .into_iter()
            .map(|message| {
                let (line_num, line_pos, offset, length) = message
                    .location
                    .map(|location| source_location_utf16(&shader.source, location))
                    .unwrap_or((0, 0, 0, 0));
                GpuCompilationMessage {
                    message: message.message,
                    message_type: match message.message_type {
                        wgpu::CompilationMessageType::Error => "error",
                        wgpu::CompilationMessageType::Warning => "warning",
                        wgpu::CompilationMessageType::Info => "info",
                    }
                    .to_owned(),
                    line_num,
                    line_pos,
                    offset,
                    length,
                }
            })
            .collect())
    }

    pub(crate) fn create_query_set(
        &mut self,
        device: u64,
        query_type: &str,
        count: u32,
    ) -> Result<u64, String> {
        let ty = match query_type {
            "occlusion" => QueryType::Occlusion,
            "timestamp" => QueryType::Timestamp,
            _ => return Err(format!("unsupported GPU query type: {query_type}")),
        };
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let query_set = state.device.create_query_set(&QuerySetDescriptor {
            label: None,
            ty,
            count,
        });
        let id = next_resource(state)?;
        state.query_sets.insert(id, query_set);
        Ok(id)
    }

    pub(crate) fn create_bind_group_layout(
        &mut self,
        device: u64,
        entries: &[GpuBindGroupLayoutEntry],
    ) -> Result<u64, String> {
        let native_entries = entries
            .iter()
            .map(bind_group_layout_entry)
            .collect::<Result<Vec<_>, _>>()?;
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let layout = state
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &native_entries,
            });
        let id = next_resource(state)?;
        state.bind_group_layouts.insert(id, layout);
        Ok(id)
    }

    pub(crate) fn create_pipeline_layout(
        &mut self,
        device: u64,
        bind_group_layouts: &[u64],
        immediate_size: u32,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let native_layouts = bind_group_layouts
            .iter()
            .map(|layout| {
                state
                    .bind_group_layouts
                    .get(layout)
                    .map(Some)
                    .ok_or_else(|| "unknown GPUBindGroupLayout".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let layout = state
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &native_layouts,
                immediate_size,
            });
        let id = next_resource(state)?;
        state.pipeline_layouts.insert(id, layout);
        Ok(id)
    }

    pub(crate) fn create_compute_pipeline(
        &mut self,
        device: u64,
        layout: Option<u64>,
        module: u64,
        entry_point: &str,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let module = state
            .shaders
            .get(&module)
            .ok_or("unknown GPUShaderModule")?;
        let layout = layout
            .map(|layout| {
                state
                    .pipeline_layouts
                    .get(&layout)
                    .ok_or("unknown GPUPipelineLayout")
            })
            .transpose()?;
        let pipeline = state
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout,
                module: &module.module,
                entry_point: Some(entry_point),
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            });
        let id = next_resource(state)?;
        state.compute_pipelines.insert(id, pipeline);
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_render_pipeline(
        &mut self,
        device: u64,
        layout: Option<u64>,
        vertex_module: u64,
        vertex_entry: &str,
        fragment_module: u64,
        fragment_entry: &str,
        vertex_buffers: &[GpuVertexBufferLayout],
        target_descriptors: &[Option<GpuColorTarget>],
        depth_stencil: Option<&GpuDepthStencilState>,
        multisample: &GpuMultisampleState,
        primitive: &GpuPrimitiveState,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let vertex_module = state
            .shaders
            .get(&vertex_module)
            .ok_or("unknown vertex GPUShaderModule")?;
        let fragment_module = state
            .shaders
            .get(&fragment_module)
            .ok_or("unknown fragment GPUShaderModule")?;
        let layout = layout
            .map(|layout| {
                state
                    .pipeline_layouts
                    .get(&layout)
                    .ok_or("unknown GPUPipelineLayout")
            })
            .transpose()?;
        let targets = target_descriptors
            .iter()
            .map(|target| {
                target
                    .as_ref()
                    .map(|target| {
                        Ok(ColorTargetState {
                            format: texture_format(&target.format)?,
                            blend: target.blend.as_ref().map(blend_state).transpose()?,
                            write_mask: ColorWrites::from_bits(target.write_mask)
                                .ok_or("GPU color write mask contains unknown bits")?,
                        })
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, String>>()?;
        let attribute_sets = vertex_buffers
            .iter()
            .map(|layout| {
                layout
                    .attributes
                    .iter()
                    .map(|attribute| {
                        Ok(VertexAttribute {
                            format: vertex_format(&attribute.format)?,
                            offset: attribute.offset,
                            shader_location: attribute.shader_location,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?;
        let vertex_buffers = vertex_buffers
            .iter()
            .zip(&attribute_sets)
            .map(|(layout, attributes)| {
                Ok(Some(VertexBufferLayout {
                    array_stride: layout.array_stride,
                    step_mode: vertex_step_mode(&layout.step_mode)?,
                    attributes,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let pipeline = state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout,
                vertex: VertexState {
                    module: &vertex_module.module,
                    entry_point: Some(vertex_entry),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &vertex_buffers,
                },
                primitive: PrimitiveState {
                    topology: primitive_topology(&primitive.topology)?,
                    strip_index_format: primitive
                        .strip_index_format
                        .as_deref()
                        .map(index_format)
                        .transpose()?,
                    front_face: front_face(&primitive.front_face)?,
                    cull_mode: cull_face(&primitive.cull_mode)?,
                    unclipped_depth: primitive.unclipped_depth,
                    ..PrimitiveState::default()
                },
                depth_stencil: depth_stencil.map(depth_stencil_state).transpose()?,
                multisample: MultisampleState {
                    count: multisample.count,
                    mask: multisample.mask,
                    alpha_to_coverage_enabled: multisample.alpha_to_coverage_enabled,
                },
                fragment: Some(FragmentState {
                    module: &fragment_module.module,
                    entry_point: Some(fragment_entry),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &targets,
                }),
                multiview_mask: None,
                cache: None,
            });
        let id = next_resource(state)?;
        state.render_pipelines.insert(id, pipeline);
        Ok(id)
    }

    pub(crate) fn compute_bind_group_layout(
        &mut self,
        device: u64,
        pipeline: u64,
        index: u32,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let pipeline = state
            .compute_pipelines
            .get(&pipeline)
            .ok_or("unknown GPUComputePipeline")?;
        let layout = pipeline.get_bind_group_layout(index);
        let id = next_resource(state)?;
        state.bind_group_layouts.insert(id, layout);
        Ok(id)
    }

    pub(crate) fn render_bind_group_layout(
        &mut self,
        device: u64,
        pipeline: u64,
        index: u32,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let pipeline = state
            .render_pipelines
            .get(&pipeline)
            .ok_or("unknown GPURenderPipeline")?;
        let layout = pipeline.get_bind_group_layout(index);
        let id = next_resource(state)?;
        state.bind_group_layouts.insert(id, layout);
        Ok(id)
    }

    pub(crate) fn create_bind_group(
        &mut self,
        device: u64,
        layout: u64,
        entries: &[GpuBindGroupEntry],
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let layout = state
            .bind_group_layouts
            .get(&layout)
            .ok_or("unknown GPUBindGroupLayout")?;
        let mut native_entries = Vec::with_capacity(entries.len());
        for entry in entries {
            let (binding, resource) = match entry {
                GpuBindGroupEntry::Buffer {
                    binding,
                    resource,
                    offset,
                    size,
                } => {
                    let buffer = state.buffers.get(resource).ok_or("unknown GPUBuffer")?;
                    (
                        *binding,
                        BindingResource::Buffer(BufferBinding {
                            buffer,
                            offset: *offset,
                            size: size.and_then(NonZeroU64::new),
                        }),
                    )
                }
                GpuBindGroupEntry::Sampler { binding, resource } => (
                    *binding,
                    BindingResource::Sampler(
                        state.samplers.get(resource).ok_or("unknown GPUSampler")?,
                    ),
                ),
                GpuBindGroupEntry::TextureView { binding, resource } => (
                    *binding,
                    BindingResource::TextureView(
                        &state
                            .texture_views
                            .get(resource)
                            .ok_or("unknown GPUTextureView")?
                            .view,
                    ),
                ),
            };
            native_entries.push(BindGroupEntry { binding, resource });
        }
        let group = state.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &native_entries,
        });
        let id = next_resource(state)?;
        state.bind_groups.insert(id, group);
        Ok(id)
    }

    pub(crate) fn encode_compute_pass(
        &mut self,
        device: u64,
        encoder: u64,
        commands: &[GpuComputeCommand],
        timestamp_writes: Option<&GpuTimestampWrites>,
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let timestamp_writes = timestamp_writes
            .map(|writes| {
                Ok::<_, String>(ComputePassTimestampWrites {
                    query_set: state
                        .query_sets
                        .get(&writes.query_set)
                        .ok_or_else(|| "unknown timestamp GPUQuerySet".to_owned())?,
                    beginning_of_pass_write_index: writes.beginning_of_pass_write_index,
                    end_of_pass_write_index: writes.end_of_pass_write_index,
                })
            })
            .transpose()?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes,
        });
        for command in commands {
            match command {
                GpuComputeCommand::InsertDebugMarker { marker_label } => {
                    pass.insert_debug_marker(marker_label)
                }
                GpuComputeCommand::PushDebugGroup { group_label } => {
                    pass.push_debug_group(group_label)
                }
                GpuComputeCommand::PopDebugGroup => pass.pop_debug_group(),
                GpuComputeCommand::SetImmediates { offset, data } => {
                    pass.set_immediates(*offset, data)
                }
                GpuComputeCommand::SetPipeline { pipeline } => pass.set_pipeline(
                    state
                        .compute_pipelines
                        .get(pipeline)
                        .ok_or("unknown GPUComputePipeline")?,
                ),
                GpuComputeCommand::SetBindGroup {
                    index,
                    group,
                    dynamic_offsets,
                } => pass.set_bind_group(
                    *index,
                    state.bind_groups.get(group).ok_or("unknown GPUBindGroup")?,
                    dynamic_offsets,
                ),
                GpuComputeCommand::DispatchWorkgroups { x, y, z } => {
                    pass.dispatch_workgroups(*x, *y, *z)
                }
                GpuComputeCommand::DispatchWorkgroupsIndirect { buffer, offset } => pass
                    .dispatch_workgroups_indirect(
                        state.buffers.get(buffer).ok_or("unknown GPUBuffer")?,
                        *offset,
                    ),
            }
        }
        Ok(())
    }

    pub(crate) fn create_render_bundle(
        &mut self,
        device: u64,
        descriptor: &GpuRenderBundleEncoderDescriptor,
        commands: &[GpuRenderCommand],
        label: &str,
    ) -> Result<u64, String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        if descriptor.sample_count == 0 {
            return Err("render bundle sampleCount must be greater than zero".to_owned());
        }
        let color_formats = descriptor
            .color_formats
            .iter()
            .map(|format| format.as_deref().map(texture_format).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        let depth_stencil = descriptor
            .depth_stencil_format
            .as_deref()
            .map(texture_format)
            .transpose()?
            .map(|format| RenderBundleDepthStencil {
                format,
                depth_read_only: descriptor.depth_read_only,
                stencil_read_only: descriptor.stencil_read_only,
            });
        let mut encoder =
            state
                .device
                .create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                    label: None,
                    color_formats: &color_formats,
                    depth_stencil,
                    sample_count: descriptor.sample_count,
                    multiview: None,
                });
        for command in commands {
            match command {
                GpuRenderCommand::SetImmediates { offset, data } => {
                    encoder.set_immediates(*offset, data)
                }
                GpuRenderCommand::SetPipeline { pipeline } => encoder.set_pipeline(
                    state
                        .render_pipelines
                        .get(pipeline)
                        .ok_or("unknown GPURenderPipeline")?,
                ),
                GpuRenderCommand::SetBindGroup {
                    index,
                    group,
                    dynamic_offsets,
                } => encoder.set_bind_group(
                    *index,
                    state.bind_groups.get(group).ok_or("unknown GPUBindGroup")?,
                    dynamic_offsets,
                ),
                GpuRenderCommand::SetVertexBuffer {
                    slot,
                    buffer,
                    offset,
                    size,
                } => {
                    let buffer = state.buffers.get(buffer).ok_or("unknown GPUBuffer")?;
                    let end = size
                        .map(|size| {
                            offset
                                .checked_add(size)
                                .ok_or("vertex buffer range overflow")
                        })
                        .transpose()?;
                    if let Some(end) = end {
                        encoder.set_vertex_buffer(*slot, buffer.slice(*offset..end));
                    } else {
                        encoder.set_vertex_buffer(*slot, buffer.slice(*offset..));
                    }
                }
                GpuRenderCommand::SetIndexBuffer {
                    buffer,
                    format,
                    offset,
                    size,
                } => {
                    let buffer = state.buffers.get(buffer).ok_or("unknown GPUBuffer")?;
                    let format = index_format(format)?;
                    let end = size
                        .map(|size| {
                            offset
                                .checked_add(size)
                                .ok_or("index buffer range overflow")
                        })
                        .transpose()?;
                    if let Some(end) = end {
                        encoder.set_index_buffer(buffer.slice(*offset..end), format);
                    } else {
                        encoder.set_index_buffer(buffer.slice(*offset..), format);
                    }
                }
                GpuRenderCommand::Draw {
                    vertices,
                    instances,
                    first_vertex,
                    first_instance,
                } => encoder.draw(
                    *first_vertex..first_vertex.saturating_add(*vertices),
                    *first_instance..first_instance.saturating_add(*instances),
                ),
                GpuRenderCommand::DrawIndexed {
                    indices,
                    instances,
                    first_index,
                    base_vertex,
                    first_instance,
                } => encoder.draw_indexed(
                    *first_index..first_index.saturating_add(*indices),
                    *base_vertex,
                    *first_instance..first_instance.saturating_add(*instances),
                ),
                GpuRenderCommand::DrawIndirect { buffer, offset } => encoder.draw_indirect(
                    state.buffers.get(buffer).ok_or("unknown GPUBuffer")?,
                    *offset,
                ),
                GpuRenderCommand::DrawIndexedIndirect { buffer, offset } => encoder
                    .draw_indexed_indirect(
                        state.buffers.get(buffer).ok_or("unknown GPUBuffer")?,
                        *offset,
                    ),
                GpuRenderCommand::InsertDebugMarker { .. }
                | GpuRenderCommand::PushDebugGroup { .. }
                | GpuRenderCommand::PopDebugGroup
                | GpuRenderCommand::ExecuteBundles { .. }
                | GpuRenderCommand::SetViewport { .. }
                | GpuRenderCommand::SetScissorRect { .. }
                | GpuRenderCommand::SetBlendConstant { .. }
                | GpuRenderCommand::SetStencilReference { .. }
                | GpuRenderCommand::BeginOcclusionQuery { .. }
                | GpuRenderCommand::EndOcclusionQuery => {
                    return Err("command is not available in GPURenderBundleEncoder".to_owned());
                }
            }
        }
        let bundle = encoder.finish(&RenderBundleDescriptor {
            label: (!label.is_empty()).then_some(label),
        });
        state.next_resource = state
            .next_resource
            .checked_add(1)
            .ok_or("GPU resource id overflow")?;
        let id = state.next_resource;
        state.render_bundles.insert(id, bundle);
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_render_pass(
        &mut self,
        device: u64,
        encoder: u64,
        attachments: &[GpuColorAttachment],
        depth_stencil_attachment: Option<&GpuDepthStencilAttachment>,
        occlusion_query_set: Option<u64>,
        timestamp_writes: Option<&GpuTimestampWrites>,
        commands: &[GpuRenderCommand],
    ) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let views = attachments
            .iter()
            .map(|attachment| {
                let view = state
                    .texture_views
                    .get(&attachment.view)
                    .ok_or("unknown GPUTextureView")?;
                let resolve_target = attachment
                    .resolve_target
                    .map(|id| {
                        state
                            .texture_views
                            .get(&id)
                            .map(|view| &view.view)
                            .ok_or("unknown resolve GPUTextureView")
                    })
                    .transpose()?;
                Ok((
                    &view.view,
                    resolve_target,
                    attachment.clear,
                    attachment.color,
                    attachment.store,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let native_attachments = views
            .iter()
            .map(|&(view, resolve_target, clear, color, store)| {
                Some(RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target,
                    ops: Operations {
                        load: if clear {
                            LoadOp::Clear(Color {
                                r: color[0],
                                g: color[1],
                                b: color[2],
                                a: color[3],
                            })
                        } else {
                            LoadOp::Load
                        },
                        store: if store {
                            StoreOp::Store
                        } else {
                            StoreOp::Discard
                        },
                    },
                })
            })
            .collect::<Vec<_>>();
        let depth_stencil_view = depth_stencil_attachment
            .map(|attachment| {
                state
                    .texture_views
                    .get(&attachment.view)
                    .map(|view| (&view.view, attachment))
                    .ok_or("unknown depth/stencil GPUTextureView")
            })
            .transpose()?;
        let native_depth_stencil_attachment =
            depth_stencil_view.map(|(view, attachment)| RenderPassDepthStencilAttachment {
                view,
                depth_ops: (!attachment.depth_read_only).then_some(Operations {
                    load: if attachment.depth_load {
                        LoadOp::Clear(attachment.depth_clear_value)
                    } else {
                        LoadOp::Load
                    },
                    store: if attachment.depth_store {
                        StoreOp::Store
                    } else {
                        StoreOp::Discard
                    },
                }),
                stencil_ops: (!attachment.stencil_read_only).then_some(Operations {
                    load: if attachment.stencil_load {
                        LoadOp::Clear(attachment.stencil_clear_value)
                    } else {
                        LoadOp::Load
                    },
                    store: if attachment.stencil_store {
                        StoreOp::Store
                    } else {
                        StoreOp::Discard
                    },
                }),
            });
        let occlusion_query_set = occlusion_query_set
            .map(|query_set| {
                state
                    .query_sets
                    .get(&query_set)
                    .ok_or("unknown GPUQuerySet")
            })
            .transpose()?;
        let timestamp_writes = timestamp_writes
            .map(|writes| {
                Ok::<_, String>(RenderPassTimestampWrites {
                    query_set: state
                        .query_sets
                        .get(&writes.query_set)
                        .ok_or_else(|| "unknown timestamp GPUQuerySet".to_owned())?,
                    beginning_of_pass_write_index: writes.beginning_of_pass_write_index,
                    end_of_pass_write_index: writes.end_of_pass_write_index,
                })
            })
            .transpose()?;
        let encoder = state
            .encoders
            .get_mut(&encoder)
            .ok_or("unknown GPU command encoder")?;
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &native_attachments,
            depth_stencil_attachment: native_depth_stencil_attachment,
            occlusion_query_set,
            timestamp_writes,
            ..RenderPassDescriptor::default()
        });
        for command in commands {
            match command {
                GpuRenderCommand::InsertDebugMarker { marker_label } => {
                    pass.insert_debug_marker(marker_label)
                }
                GpuRenderCommand::PushDebugGroup { group_label } => {
                    pass.push_debug_group(group_label)
                }
                GpuRenderCommand::PopDebugGroup => pass.pop_debug_group(),
                GpuRenderCommand::SetImmediates { offset, data } => {
                    pass.set_immediates(*offset, data)
                }
                GpuRenderCommand::SetPipeline { pipeline } => pass.set_pipeline(
                    state
                        .render_pipelines
                        .get(pipeline)
                        .ok_or("unknown GPURenderPipeline")?,
                ),
                GpuRenderCommand::SetBindGroup {
                    index,
                    group,
                    dynamic_offsets,
                } => pass.set_bind_group(
                    *index,
                    state.bind_groups.get(group).ok_or("unknown GPUBindGroup")?,
                    dynamic_offsets,
                ),
                GpuRenderCommand::SetVertexBuffer {
                    slot,
                    buffer,
                    offset,
                    size,
                } => {
                    let buffer = state.buffers.get(buffer).ok_or("unknown GPUBuffer")?;
                    let end = size
                        .map(|size| {
                            offset
                                .checked_add(size)
                                .ok_or("vertex buffer range overflow")
                        })
                        .transpose()?;
                    if let Some(end) = end {
                        pass.set_vertex_buffer(*slot, buffer.slice(*offset..end));
                    } else {
                        pass.set_vertex_buffer(*slot, buffer.slice(*offset..));
                    }
                }
                GpuRenderCommand::SetIndexBuffer {
                    buffer,
                    format,
                    offset,
                    size,
                } => {
                    let buffer = state.buffers.get(buffer).ok_or("unknown GPUBuffer")?;
                    let format = index_format(format)?;
                    let end = size
                        .map(|size| {
                            offset
                                .checked_add(size)
                                .ok_or("index buffer range overflow")
                        })
                        .transpose()?;
                    if let Some(end) = end {
                        pass.set_index_buffer(buffer.slice(*offset..end), format);
                    } else {
                        pass.set_index_buffer(buffer.slice(*offset..), format);
                    }
                }
                GpuRenderCommand::Draw {
                    vertices,
                    instances,
                    first_vertex,
                    first_instance,
                } => pass.draw(
                    *first_vertex..first_vertex.saturating_add(*vertices),
                    *first_instance..first_instance.saturating_add(*instances),
                ),
                GpuRenderCommand::DrawIndexed {
                    indices,
                    instances,
                    first_index,
                    base_vertex,
                    first_instance,
                } => pass.draw_indexed(
                    *first_index..first_index.saturating_add(*indices),
                    *base_vertex,
                    *first_instance..first_instance.saturating_add(*instances),
                ),
                GpuRenderCommand::DrawIndirect { buffer, offset } => pass.draw_indirect(
                    state.buffers.get(buffer).ok_or("unknown GPUBuffer")?,
                    *offset,
                ),
                GpuRenderCommand::DrawIndexedIndirect { buffer, offset } => pass
                    .draw_indexed_indirect(
                        state.buffers.get(buffer).ok_or("unknown GPUBuffer")?,
                        *offset,
                    ),
                GpuRenderCommand::ExecuteBundles { bundles } => {
                    let bundles = bundles
                        .iter()
                        .map(|bundle| {
                            state
                                .render_bundles
                                .get(bundle)
                                .ok_or("unknown GPURenderBundle")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    pass.execute_bundles(bundles);
                }
                GpuRenderCommand::SetViewport {
                    x,
                    y,
                    width,
                    height,
                    min_depth,
                    max_depth,
                } => pass.set_viewport(*x, *y, *width, *height, *min_depth, *max_depth),
                GpuRenderCommand::SetScissorRect {
                    x,
                    y,
                    width,
                    height,
                } => pass.set_scissor_rect(*x, *y, *width, *height),
                GpuRenderCommand::SetBlendConstant { color } => pass.set_blend_constant(Color {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: color[3],
                }),
                GpuRenderCommand::SetStencilReference { reference } => {
                    pass.set_stencil_reference(*reference)
                }
                GpuRenderCommand::BeginOcclusionQuery { query_index } => {
                    pass.begin_occlusion_query(*query_index)
                }
                GpuRenderCommand::EndOcclusionQuery => pass.end_occlusion_query(),
            }
        }
        Ok(())
    }

    pub(crate) fn destroy_buffer(&mut self, device: u64, buffer: u64) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let buffer = state.buffers.remove(&buffer).ok_or("unknown GPU buffer")?;
        buffer.destroy();
        Ok(())
    }

    pub(crate) fn read_texture_rgba(
        &self,
        device: u64,
        texture: u64,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        let state = self.devices.get(&device).ok_or("unknown GPU device")?;
        let texture = state.textures.get(&texture).ok_or("unknown GPUTexture")?;
        if !matches!(
            texture.format,
            TextureFormat::Rgba8Unorm
                | TextureFormat::Rgba8UnormSrgb
                | TextureFormat::Bgra8Unorm
                | TextureFormat::Bgra8UnormSrgb
        ) {
            return Err("GPU canvas textures must use an RGBA8 or BGRA8 format".to_owned());
        }
        let row_bytes = width.checked_mul(4).ok_or("GPU canvas row size overflow")?;
        let padded_row_bytes = row_bytes
            .checked_add(255)
            .ok_or("GPU canvas row size overflow")?
            / 256
            * 256;
        let buffer_size = u64::from(padded_row_bytes)
            .checked_mul(u64::from(height))
            .ok_or("GPU canvas buffer size overflow")?;
        let buffer = state.device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        state.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        buffer.map_async(MapMode::Read, .., move |result| {
            let _ = sender.send(result.map_err(|error| error.to_string()));
        });
        state
            .device
            .poll(PollType::wait_indefinitely())
            .map_err(|error| format!("could not poll GPU device: {error}"))?;
        receiver
            .recv()
            .map_err(|_| "GPU canvas mapping callback was dropped".to_owned())??;
        let mapped = buffer
            .get_mapped_range(..)
            .map_err(|error| format!("could not read GPU canvas texture: {error}"))?;
        let mut pixels = Vec::with_capacity(row_bytes as usize * height as usize);
        for row in mapped
            .chunks_exact(padded_row_bytes as usize)
            .take(height as usize)
        {
            pixels.extend_from_slice(&row[..row_bytes as usize]);
        }
        drop(mapped);
        buffer.unmap();
        if matches!(
            texture.format,
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb
        ) {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }
        Ok(pixels)
    }

    pub(crate) fn destroy_texture(&mut self, device: u64, texture: u64) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let record = state
            .textures
            .remove(&texture)
            .ok_or("unknown GPUTexture")?;
        state
            .texture_views
            .retain(|_, view| view.texture != texture);
        record.texture.destroy();
        Ok(())
    }

    pub(crate) fn destroy_query_set(&mut self, device: u64, query_set: u64) -> Result<(), String> {
        let state = self.devices.get_mut(&device).ok_or("unknown GPU device")?;
        let query_set = state
            .query_sets
            .remove(&query_set)
            .ok_or("unknown GPUQuerySet")?;
        query_set.destroy();
        Ok(())
    }
}

fn next_resource(state: &mut DeviceState) -> Result<u64, String> {
    state.next_resource = state
        .next_resource
        .checked_add(1)
        .ok_or("GPU resource id overflow")?;
    Ok(state.next_resource)
}

fn gpu_error_record(error: Error) -> (String, String) {
    let kind = match &error {
        Error::Validation { .. } => "validation",
        Error::OutOfMemory { .. } => "out-of-memory",
        Error::Internal { .. } => "internal",
    };
    (kind.to_owned(), error.to_string())
}

fn texture_dimension(value: &str) -> Result<TextureDimension, String> {
    match value {
        "1d" => Ok(TextureDimension::D1),
        "2d" => Ok(TextureDimension::D2),
        "3d" => Ok(TextureDimension::D3),
        _ => Err(format!("unsupported GPUTexture dimension: {value}")),
    }
}

fn texture_aspect(value: &str) -> Result<TextureAspect, String> {
    match value {
        "all" => Ok(TextureAspect::All),
        "stencil-only" => Ok(TextureAspect::StencilOnly),
        "depth-only" => Ok(TextureAspect::DepthOnly),
        _ => Err(format!("unsupported GPU texture aspect: {value}")),
    }
}

fn address_mode(value: &str) -> Result<AddressMode, String> {
    match value {
        "clamp-to-edge" => Ok(AddressMode::ClampToEdge),
        "repeat" => Ok(AddressMode::Repeat),
        "mirror-repeat" => Ok(AddressMode::MirrorRepeat),
        _ => Err(format!("unsupported GPU address mode: {value}")),
    }
}

fn filter_mode(value: &str) -> Result<FilterMode, String> {
    match value {
        "nearest" => Ok(FilterMode::Nearest),
        "linear" => Ok(FilterMode::Linear),
        _ => Err(format!("unsupported GPU filter mode: {value}")),
    }
}

fn mipmap_filter_mode(value: &str) -> Result<MipmapFilterMode, String> {
    match value {
        "nearest" => Ok(MipmapFilterMode::Nearest),
        "linear" => Ok(MipmapFilterMode::Linear),
        _ => Err(format!("unsupported GPU mipmap filter mode: {value}")),
    }
}

fn compare_function(value: &str) -> Result<CompareFunction, String> {
    match value {
        "never" => Ok(CompareFunction::Never),
        "less" => Ok(CompareFunction::Less),
        "equal" => Ok(CompareFunction::Equal),
        "less-equal" => Ok(CompareFunction::LessEqual),
        "greater" => Ok(CompareFunction::Greater),
        "not-equal" => Ok(CompareFunction::NotEqual),
        "greater-equal" => Ok(CompareFunction::GreaterEqual),
        "always" => Ok(CompareFunction::Always),
        _ => Err(format!("unsupported GPU compare function: {value}")),
    }
}

fn bind_group_layout_entry(
    value: &GpuBindGroupLayoutEntry,
) -> Result<BindGroupLayoutEntry, String> {
    let (binding, visibility, ty) = match value {
        GpuBindGroupLayoutEntry::Buffer {
            binding,
            visibility,
            ty,
            has_dynamic_offset,
            min_binding_size,
        } => (
            *binding,
            *visibility,
            BindingType::Buffer {
                ty: buffer_binding_type(ty)?,
                has_dynamic_offset: *has_dynamic_offset,
                min_binding_size: min_binding_size.and_then(NonZeroU64::new),
            },
        ),
        GpuBindGroupLayoutEntry::Sampler {
            binding,
            visibility,
            ty,
        } => (
            *binding,
            *visibility,
            BindingType::Sampler(sampler_binding_type(ty)?),
        ),
        GpuBindGroupLayoutEntry::Texture {
            binding,
            visibility,
            sample_type,
            view_dimension,
            multisampled,
        } => (
            *binding,
            *visibility,
            BindingType::Texture {
                sample_type: texture_sample_type(sample_type)?,
                view_dimension: texture_view_dimension(view_dimension)?,
                multisampled: *multisampled,
            },
        ),
        GpuBindGroupLayoutEntry::StorageTexture {
            binding,
            visibility,
            access,
            format,
            view_dimension,
        } => (
            *binding,
            *visibility,
            BindingType::StorageTexture {
                access: storage_texture_access(access)?,
                format: texture_format(format)?,
                view_dimension: texture_view_dimension(view_dimension)?,
            },
        ),
    };
    Ok(BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::from_bits(visibility)
            .ok_or("GPU shader visibility contains unknown bits")?,
        ty,
        count: None,
    })
}

fn buffer_binding_type(value: &str) -> Result<BufferBindingType, String> {
    match value {
        "uniform" => Ok(BufferBindingType::Uniform),
        "storage" => Ok(BufferBindingType::Storage { read_only: false }),
        "read-only-storage" => Ok(BufferBindingType::Storage { read_only: true }),
        _ => Err(format!("unsupported GPU buffer binding type: {value}")),
    }
}

fn sampler_binding_type(value: &str) -> Result<SamplerBindingType, String> {
    match value {
        "filtering" => Ok(SamplerBindingType::Filtering),
        "non-filtering" => Ok(SamplerBindingType::NonFiltering),
        "comparison" => Ok(SamplerBindingType::Comparison),
        _ => Err(format!("unsupported GPU sampler binding type: {value}")),
    }
}

fn texture_sample_type(value: &str) -> Result<TextureSampleType, String> {
    match value {
        "float" => Ok(TextureSampleType::Float { filterable: true }),
        "unfilterable-float" => Ok(TextureSampleType::Float { filterable: false }),
        "depth" => Ok(TextureSampleType::Depth),
        "sint" => Ok(TextureSampleType::Sint),
        "uint" => Ok(TextureSampleType::Uint),
        _ => Err(format!("unsupported GPU texture sample type: {value}")),
    }
}

fn texture_view_dimension(value: &str) -> Result<TextureViewDimension, String> {
    match value {
        "1d" => Ok(TextureViewDimension::D1),
        "2d" => Ok(TextureViewDimension::D2),
        "2d-array" => Ok(TextureViewDimension::D2Array),
        "cube" => Ok(TextureViewDimension::Cube),
        "cube-array" => Ok(TextureViewDimension::CubeArray),
        "3d" => Ok(TextureViewDimension::D3),
        _ => Err(format!("unsupported GPU texture view dimension: {value}")),
    }
}

fn storage_texture_access(value: &str) -> Result<StorageTextureAccess, String> {
    match value {
        "write-only" => Ok(StorageTextureAccess::WriteOnly),
        "read-only" => Ok(StorageTextureAccess::ReadOnly),
        "read-write" => Ok(StorageTextureAccess::ReadWrite),
        _ => Err(format!("unsupported GPU storage texture access: {value}")),
    }
}

fn blend_state(value: &GpuBlendState) -> Result<BlendState, String> {
    Ok(BlendState {
        color: blend_component(&value.color)?,
        alpha: blend_component(&value.alpha)?,
    })
}

fn blend_component(value: &GpuBlendComponent) -> Result<BlendComponent, String> {
    Ok(BlendComponent {
        src_factor: blend_factor(&value.src_factor)?,
        dst_factor: blend_factor(&value.dst_factor)?,
        operation: blend_operation(&value.operation)?,
    })
}

fn blend_factor(value: &str) -> Result<BlendFactor, String> {
    match value {
        "zero" => Ok(BlendFactor::Zero),
        "one" => Ok(BlendFactor::One),
        "src" => Ok(BlendFactor::Src),
        "one-minus-src" => Ok(BlendFactor::OneMinusSrc),
        "src-alpha" => Ok(BlendFactor::SrcAlpha),
        "one-minus-src-alpha" => Ok(BlendFactor::OneMinusSrcAlpha),
        "dst" => Ok(BlendFactor::Dst),
        "one-minus-dst" => Ok(BlendFactor::OneMinusDst),
        "dst-alpha" => Ok(BlendFactor::DstAlpha),
        "one-minus-dst-alpha" => Ok(BlendFactor::OneMinusDstAlpha),
        "src-alpha-saturated" => Ok(BlendFactor::SrcAlphaSaturated),
        "constant" => Ok(BlendFactor::Constant),
        "one-minus-constant" => Ok(BlendFactor::OneMinusConstant),
        "src1" => Ok(BlendFactor::Src1),
        "one-minus-src1" => Ok(BlendFactor::OneMinusSrc1),
        "src1-alpha" => Ok(BlendFactor::Src1Alpha),
        "one-minus-src1-alpha" => Ok(BlendFactor::OneMinusSrc1Alpha),
        _ => Err(format!("unsupported GPU blend factor: {value}")),
    }
}

fn blend_operation(value: &str) -> Result<BlendOperation, String> {
    match value {
        "add" => Ok(BlendOperation::Add),
        "subtract" => Ok(BlendOperation::Subtract),
        "reverse-subtract" => Ok(BlendOperation::ReverseSubtract),
        "min" => Ok(BlendOperation::Min),
        "max" => Ok(BlendOperation::Max),
        _ => Err(format!("unsupported GPU blend operation: {value}")),
    }
}

fn depth_stencil_state(value: &GpuDepthStencilState) -> Result<DepthStencilState, String> {
    if !value.depth_bias_slope_scale.is_finite() || !value.depth_bias_clamp.is_finite() {
        return Err("GPU depth bias values must be finite".to_owned());
    }
    Ok(DepthStencilState {
        format: texture_format(&value.format)?,
        depth_write_enabled: Some(value.depth_write_enabled),
        depth_compare: Some(compare_function(&value.depth_compare)?),
        stencil: StencilState {
            front: stencil_face_state(&value.stencil_front)?,
            back: stencil_face_state(&value.stencil_back)?,
            read_mask: value.stencil_read_mask,
            write_mask: value.stencil_write_mask,
        },
        bias: DepthBiasState {
            constant: value.depth_bias,
            slope_scale: value.depth_bias_slope_scale,
            clamp: value.depth_bias_clamp,
        },
    })
}

fn stencil_face_state(value: &GpuStencilFaceState) -> Result<StencilFaceState, String> {
    Ok(StencilFaceState {
        compare: compare_function(&value.compare)?,
        fail_op: stencil_operation(&value.fail_op)?,
        depth_fail_op: stencil_operation(&value.depth_fail_op)?,
        pass_op: stencil_operation(&value.pass_op)?,
    })
}

fn stencil_operation(value: &str) -> Result<StencilOperation, String> {
    match value {
        "keep" => Ok(StencilOperation::Keep),
        "zero" => Ok(StencilOperation::Zero),
        "replace" => Ok(StencilOperation::Replace),
        "invert" => Ok(StencilOperation::Invert),
        "increment-clamp" => Ok(StencilOperation::IncrementClamp),
        "decrement-clamp" => Ok(StencilOperation::DecrementClamp),
        "increment-wrap" => Ok(StencilOperation::IncrementWrap),
        "decrement-wrap" => Ok(StencilOperation::DecrementWrap),
        _ => Err(format!("unsupported GPU stencil operation: {value}")),
    }
}

fn texture_format(value: &str) -> Result<TextureFormat, String> {
    let format: TextureFormat = match value {
        "r8unorm" => TextureFormat::R8Unorm,
        "r8snorm" => TextureFormat::R8Snorm,
        "r8uint" => TextureFormat::R8Uint,
        "r8sint" => TextureFormat::R8Sint,
        "r16uint" => TextureFormat::R16Uint,
        "r16sint" => TextureFormat::R16Sint,
        "r16float" => TextureFormat::R16Float,
        "rg8unorm" => TextureFormat::Rg8Unorm,
        "rg8snorm" => TextureFormat::Rg8Snorm,
        "rg8uint" => TextureFormat::Rg8Uint,
        "rg8sint" => TextureFormat::Rg8Sint,
        "r32uint" => TextureFormat::R32Uint,
        "r32sint" => TextureFormat::R32Sint,
        "r32float" => TextureFormat::R32Float,
        "rg16uint" => TextureFormat::Rg16Uint,
        "rg16sint" => TextureFormat::Rg16Sint,
        "rg16float" => TextureFormat::Rg16Float,
        "rgba8unorm" => TextureFormat::Rgba8Unorm,
        "rgba8unorm-srgb" => TextureFormat::Rgba8UnormSrgb,
        "rgba8snorm" => TextureFormat::Rgba8Snorm,
        "rgba8uint" => TextureFormat::Rgba8Uint,
        "rgba8sint" => TextureFormat::Rgba8Sint,
        "bgra8unorm" => TextureFormat::Bgra8Unorm,
        "bgra8unorm-srgb" => TextureFormat::Bgra8UnormSrgb,
        "rgb9e5ufloat" => TextureFormat::Rgb9e5Ufloat,
        "rgb10a2uint" => TextureFormat::Rgb10a2Uint,
        "rgb10a2unorm" => TextureFormat::Rgb10a2Unorm,
        "rg11b10ufloat" => TextureFormat::Rg11b10Ufloat,
        "rg32uint" => TextureFormat::Rg32Uint,
        "rg32sint" => TextureFormat::Rg32Sint,
        "rg32float" => TextureFormat::Rg32Float,
        "rgba16uint" => TextureFormat::Rgba16Uint,
        "rgba16sint" => TextureFormat::Rgba16Sint,
        "rgba16float" => TextureFormat::Rgba16Float,
        "rgba32uint" => TextureFormat::Rgba32Uint,
        "rgba32sint" => TextureFormat::Rgba32Sint,
        "rgba32float" => TextureFormat::Rgba32Float,
        "stencil8" => TextureFormat::Stencil8,
        "depth16unorm" => TextureFormat::Depth16Unorm,
        "depth24plus" => TextureFormat::Depth24Plus,
        "depth24plus-stencil8" => TextureFormat::Depth24PlusStencil8,
        "depth32float" => TextureFormat::Depth32Float,
        "depth32float-stencil8" => TextureFormat::Depth32FloatStencil8,
        "bc1-rgba-unorm" => TextureFormat::Bc1RgbaUnorm,
        "bc1-rgba-unorm-srgb" => TextureFormat::Bc1RgbaUnormSrgb,
        "bc2-rgba-unorm" => TextureFormat::Bc2RgbaUnorm,
        "bc2-rgba-unorm-srgb" => TextureFormat::Bc2RgbaUnormSrgb,
        "bc3-rgba-unorm" => TextureFormat::Bc3RgbaUnorm,
        "bc3-rgba-unorm-srgb" => TextureFormat::Bc3RgbaUnormSrgb,
        "bc4-r-unorm" => TextureFormat::Bc4RUnorm,
        "bc4-r-snorm" => TextureFormat::Bc4RSnorm,
        "bc5-rg-unorm" => TextureFormat::Bc5RgUnorm,
        "bc5-rg-snorm" => TextureFormat::Bc5RgSnorm,
        "bc6h-rgb-ufloat" => TextureFormat::Bc6hRgbUfloat,
        "bc6h-rgb-float" => TextureFormat::Bc6hRgbFloat,
        "bc7-rgba-unorm" => TextureFormat::Bc7RgbaUnorm,
        "bc7-rgba-unorm-srgb" => TextureFormat::Bc7RgbaUnormSrgb,
        "etc2-rgb8unorm" => TextureFormat::Etc2Rgb8Unorm,
        "etc2-rgb8unorm-srgb" => TextureFormat::Etc2Rgb8UnormSrgb,
        "etc2-rgb8a1unorm" => TextureFormat::Etc2Rgb8A1Unorm,
        "etc2-rgb8a1unorm-srgb" => TextureFormat::Etc2Rgb8A1UnormSrgb,
        "etc2-rgba8unorm" => TextureFormat::Etc2Rgba8Unorm,
        "etc2-rgba8unorm-srgb" => TextureFormat::Etc2Rgba8UnormSrgb,
        "eac-r11unorm" => TextureFormat::EacR11Unorm,
        "eac-r11snorm" => TextureFormat::EacR11Snorm,
        "eac-rg11unorm" => TextureFormat::EacRg11Unorm,
        "eac-rg11snorm" => TextureFormat::EacRg11Snorm,
        _ => return astc_texture_format(value),
    };
    Ok(format)
}

fn astc_texture_format(value: &str) -> Result<TextureFormat, String> {
    let value = value
        .strip_prefix("astc-")
        .ok_or_else(|| format!("unsupported GPUTexture format: {value}"))?;
    let (block, channel) = if let Some(block) = value.strip_suffix("-unorm-srgb") {
        (block, AstcChannel::UnormSrgb)
    } else if let Some(block) = value.strip_suffix("-unorm") {
        (block, AstcChannel::Unorm)
    } else {
        return Err(format!("unsupported GPUTexture format: astc-{value}"));
    };
    let block = match block {
        "4x4" => AstcBlock::B4x4,
        "5x4" => AstcBlock::B5x4,
        "5x5" => AstcBlock::B5x5,
        "6x5" => AstcBlock::B6x5,
        "6x6" => AstcBlock::B6x6,
        "8x5" => AstcBlock::B8x5,
        "8x6" => AstcBlock::B8x6,
        "8x8" => AstcBlock::B8x8,
        "10x5" => AstcBlock::B10x5,
        "10x6" => AstcBlock::B10x6,
        "10x8" => AstcBlock::B10x8,
        "10x10" => AstcBlock::B10x10,
        "12x10" => AstcBlock::B12x10,
        "12x12" => AstcBlock::B12x12,
        _ => return Err(format!("unsupported ASTC block size: {block}")),
    };
    Ok(TextureFormat::Astc { block, channel })
}

fn primitive_topology(value: &str) -> Result<PrimitiveTopology, String> {
    match value {
        "point-list" => Ok(PrimitiveTopology::PointList),
        "line-list" => Ok(PrimitiveTopology::LineList),
        "line-strip" => Ok(PrimitiveTopology::LineStrip),
        "triangle-list" => Ok(PrimitiveTopology::TriangleList),
        "triangle-strip" => Ok(PrimitiveTopology::TriangleStrip),
        _ => Err(format!("unsupported GPU primitive topology: {value}")),
    }
}

fn cull_face(value: &str) -> Result<Option<Face>, String> {
    match value {
        "none" => Ok(None),
        "front" => Ok(Some(Face::Front)),
        "back" => Ok(Some(Face::Back)),
        _ => Err(format!("unsupported GPU cull mode: {value}")),
    }
}

fn front_face(value: &str) -> Result<FrontFace, String> {
    match value {
        "ccw" => Ok(FrontFace::Ccw),
        "cw" => Ok(FrontFace::Cw),
        _ => Err(format!("unsupported GPU front face: {value}")),
    }
}

fn vertex_step_mode(value: &str) -> Result<VertexStepMode, String> {
    match value {
        "vertex" => Ok(VertexStepMode::Vertex),
        "instance" => Ok(VertexStepMode::Instance),
        _ => Err(format!("unsupported GPU vertex step mode: {value}")),
    }
}

fn vertex_format(value: &str) -> Result<VertexFormat, String> {
    match value {
        "uint8x2" => Ok(VertexFormat::Uint8x2),
        "uint8x4" => Ok(VertexFormat::Uint8x4),
        "sint8x2" => Ok(VertexFormat::Sint8x2),
        "sint8x4" => Ok(VertexFormat::Sint8x4),
        "unorm8x2" => Ok(VertexFormat::Unorm8x2),
        "unorm8x4" => Ok(VertexFormat::Unorm8x4),
        "snorm8x2" => Ok(VertexFormat::Snorm8x2),
        "snorm8x4" => Ok(VertexFormat::Snorm8x4),
        "uint16x2" => Ok(VertexFormat::Uint16x2),
        "uint16x4" => Ok(VertexFormat::Uint16x4),
        "sint16x2" => Ok(VertexFormat::Sint16x2),
        "sint16x4" => Ok(VertexFormat::Sint16x4),
        "unorm16x2" => Ok(VertexFormat::Unorm16x2),
        "unorm16x4" => Ok(VertexFormat::Unorm16x4),
        "snorm16x2" => Ok(VertexFormat::Snorm16x2),
        "snorm16x4" => Ok(VertexFormat::Snorm16x4),
        "float16x2" => Ok(VertexFormat::Float16x2),
        "float16x4" => Ok(VertexFormat::Float16x4),
        "float32" => Ok(VertexFormat::Float32),
        "float32x2" => Ok(VertexFormat::Float32x2),
        "float32x3" => Ok(VertexFormat::Float32x3),
        "float32x4" => Ok(VertexFormat::Float32x4),
        "uint32" => Ok(VertexFormat::Uint32),
        "uint32x2" => Ok(VertexFormat::Uint32x2),
        "uint32x3" => Ok(VertexFormat::Uint32x3),
        "uint32x4" => Ok(VertexFormat::Uint32x4),
        "sint32" => Ok(VertexFormat::Sint32),
        "sint32x2" => Ok(VertexFormat::Sint32x2),
        "sint32x3" => Ok(VertexFormat::Sint32x3),
        "sint32x4" => Ok(VertexFormat::Sint32x4),
        _ => Err(format!("unsupported GPU vertex format: {value}")),
    }
}

fn index_format(value: &str) -> Result<IndexFormat, String> {
    match value {
        "uint16" => Ok(IndexFormat::Uint16),
        "uint32" => Ok(IndexFormat::Uint32),
        _ => Err(format!("unsupported GPU index format: {value}")),
    }
}

fn source_location_utf16(source: &str, location: wgpu::SourceLocation) -> (u64, u64, u64, u64) {
    let mut byte_offset = (location.offset as usize).min(source.len());
    while !source.is_char_boundary(byte_offset) {
        byte_offset -= 1;
    }
    let mut byte_end = byte_offset
        .saturating_add(location.length as usize)
        .min(source.len());
    while !source.is_char_boundary(byte_end) {
        byte_end -= 1;
    }
    let line_start = source[..byte_offset]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    (
        u64::from(location.line_number),
        source[line_start..byte_offset].encode_utf16().count() as u64 + 1,
        source[..byte_offset].encode_utf16().count() as u64,
        source[byte_offset..byte_end].encode_utf16().count() as u64,
    )
}

fn adapter_metadata(adapter: &Adapter) -> String {
    let info = adapter.get_info();
    json!({
        "info": {
            "vendor": format!("{:04x}", info.vendor),
            "architecture": "",
            "device": format!("{:04x}", info.device),
            "description": info.name,
        },
        "isFallbackAdapter": info.device_type == wgpu::DeviceType::Cpu,
        "limits": serde_json::from_str::<serde_json::Value>(&limits_json(&adapter.limits())).expect("limits JSON is valid"),
        "features": gpu_feature_names(adapter.features()),
    }).to_string()
}

const WEBGPU_FEATURES: &[(&str, Features)] = &[
    ("depth-clip-control", Features::DEPTH_CLIP_CONTROL),
    ("depth32float-stencil8", Features::DEPTH32FLOAT_STENCIL8),
    ("texture-compression-bc", Features::TEXTURE_COMPRESSION_BC),
    (
        "texture-compression-bc-sliced-3d",
        Features::TEXTURE_COMPRESSION_BC_SLICED_3D,
    ),
    (
        "texture-compression-etc2",
        Features::TEXTURE_COMPRESSION_ETC2,
    ),
    (
        "texture-compression-astc",
        Features::TEXTURE_COMPRESSION_ASTC,
    ),
    (
        "texture-compression-astc-sliced-3d",
        Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D,
    ),
    ("timestamp-query", Features::TIMESTAMP_QUERY),
    ("indirect-first-instance", Features::INDIRECT_FIRST_INSTANCE),
    ("shader-f16", Features::SHADER_F16),
    (
        "rg11b10ufloat-renderable",
        Features::RG11B10UFLOAT_RENDERABLE,
    ),
    ("bgra8unorm-storage", Features::BGRA8UNORM_STORAGE),
    ("float32-filterable", Features::FLOAT32_FILTERABLE),
    ("float32-blendable", Features::FLOAT32_BLENDABLE),
    ("clip-distances", Features::CLIP_DISTANCES),
    ("dual-source-blending", Features::DUAL_SOURCE_BLENDING),
    ("primitive-index", Features::PRIMITIVE_INDEX),
    ("immediates", Features::IMMEDIATES),
];

fn gpu_features(names: &[String]) -> Result<Features, String> {
    let mut features = names.iter().try_fold(
        Features::empty(),
        |features, name| -> Result<Features, String> {
            let feature = WEBGPU_FEATURES
                .iter()
                .find_map(|(candidate, feature)| (*candidate == name).then_some(*feature))
                .ok_or_else(|| format!("unsupported GPU feature: {name}"))?;
            Ok(features | feature)
        },
    )?;
    if features.contains(Features::TEXTURE_COMPRESSION_BC_SLICED_3D) {
        features |= Features::TEXTURE_COMPRESSION_BC;
    }
    if features.contains(Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D) {
        features |= Features::TEXTURE_COMPRESSION_ASTC;
    }
    Ok(features)
}

fn gpu_feature_names(features: Features) -> Vec<&'static str> {
    WEBGPU_FEATURES
        .iter()
        .filter_map(|(name, feature)| features.contains(*feature).then_some(*name))
        .collect()
}

fn gpu_required_limits(values: &HashMap<String, u64>) -> Result<wgpu::Limits, String> {
    let defaults = wgpu::Limits::defaults();
    let mut requested = defaults.clone();
    for (name, value) in values {
        macro_rules! set_u32 {
            ($field:ident) => {
                requested.$field =
                    u32::try_from(*value).map_err(|_| format!("GPU limit {name} is too large"))?
            };
        }
        match name.as_str() {
            "maxTextureDimension1D" => set_u32!(max_texture_dimension_1d),
            "maxTextureDimension2D" => set_u32!(max_texture_dimension_2d),
            "maxTextureDimension3D" => set_u32!(max_texture_dimension_3d),
            "maxTextureArrayLayers" => set_u32!(max_texture_array_layers),
            "maxBindGroups" => set_u32!(max_bind_groups),
            "maxBindGroupsPlusVertexBuffers" => set_u32!(max_bind_groups_plus_vertex_buffers),
            "maxBindingsPerBindGroup" => set_u32!(max_bindings_per_bind_group),
            "maxDynamicUniformBuffersPerPipelineLayout" => {
                set_u32!(max_dynamic_uniform_buffers_per_pipeline_layout)
            }
            "maxDynamicStorageBuffersPerPipelineLayout" => {
                set_u32!(max_dynamic_storage_buffers_per_pipeline_layout)
            }
            "maxSampledTexturesPerShaderStage" => {
                set_u32!(max_sampled_textures_per_shader_stage)
            }
            "maxSamplersPerShaderStage" => set_u32!(max_samplers_per_shader_stage),
            "maxStorageBuffersPerShaderStage" => {
                set_u32!(max_storage_buffers_per_shader_stage)
            }
            "maxStorageTexturesPerShaderStage" => {
                set_u32!(max_storage_textures_per_shader_stage)
            }
            "maxUniformBuffersPerShaderStage" => {
                set_u32!(max_uniform_buffers_per_shader_stage)
            }
            "maxUniformBufferBindingSize" => requested.max_uniform_buffer_binding_size = *value,
            "maxStorageBufferBindingSize" => requested.max_storage_buffer_binding_size = *value,
            "minUniformBufferOffsetAlignment" => {
                set_u32!(min_uniform_buffer_offset_alignment)
            }
            "minStorageBufferOffsetAlignment" => {
                set_u32!(min_storage_buffer_offset_alignment)
            }
            "maxVertexBuffers" => set_u32!(max_vertex_buffers),
            "maxBufferSize" => requested.max_buffer_size = *value,
            "maxVertexAttributes" => set_u32!(max_vertex_attributes),
            "maxVertexBufferArrayStride" => set_u32!(max_vertex_buffer_array_stride),
            "maxInterStageShaderVariables" => set_u32!(max_inter_stage_shader_variables),
            "maxColorAttachments" => set_u32!(max_color_attachments),
            "maxColorAttachmentBytesPerSample" => {
                set_u32!(max_color_attachment_bytes_per_sample)
            }
            "maxComputeWorkgroupStorageSize" => set_u32!(max_compute_workgroup_storage_size),
            "maxComputeInvocationsPerWorkgroup" => {
                set_u32!(max_compute_invocations_per_workgroup)
            }
            "maxComputeWorkgroupSizeX" => set_u32!(max_compute_workgroup_size_x),
            "maxComputeWorkgroupSizeY" => set_u32!(max_compute_workgroup_size_y),
            "maxComputeWorkgroupSizeZ" => set_u32!(max_compute_workgroup_size_z),
            "maxComputeWorkgroupsPerDimension" => {
                set_u32!(max_compute_workgroups_per_dimension)
            }
            "maxImmediateSize" => set_u32!(max_immediate_size),
            _ => return Err(format!("unknown GPU limit: {name}")),
        }
    }
    Ok(defaults.or_better_values_from(&requested))
}

fn limits_json(limits: &wgpu::Limits) -> String {
    json!({
        "maxTextureDimension1D": limits.max_texture_dimension_1d,
        "maxTextureDimension2D": limits.max_texture_dimension_2d,
        "maxTextureDimension3D": limits.max_texture_dimension_3d,
        "maxTextureArrayLayers": limits.max_texture_array_layers,
        "maxBindGroups": limits.max_bind_groups,
        "maxBindGroupsPlusVertexBuffers": limits.max_bind_groups_plus_vertex_buffers,
        "maxBindingsPerBindGroup": limits.max_bindings_per_bind_group,
        "maxDynamicUniformBuffersPerPipelineLayout": limits.max_dynamic_uniform_buffers_per_pipeline_layout,
        "maxDynamicStorageBuffersPerPipelineLayout": limits.max_dynamic_storage_buffers_per_pipeline_layout,
        "maxSampledTexturesPerShaderStage": limits.max_sampled_textures_per_shader_stage,
        "maxSamplersPerShaderStage": limits.max_samplers_per_shader_stage,
        "maxStorageBuffersPerShaderStage": limits.max_storage_buffers_per_shader_stage,
        "maxStorageTexturesPerShaderStage": limits.max_storage_textures_per_shader_stage,
        "maxUniformBuffersPerShaderStage": limits.max_uniform_buffers_per_shader_stage,
        "maxUniformBufferBindingSize": limits.max_uniform_buffer_binding_size,
        "maxStorageBufferBindingSize": limits.max_storage_buffer_binding_size,
        "minUniformBufferOffsetAlignment": limits.min_uniform_buffer_offset_alignment,
        "minStorageBufferOffsetAlignment": limits.min_storage_buffer_offset_alignment,
        "maxVertexBuffers": limits.max_vertex_buffers,
        "maxBufferSize": limits.max_buffer_size,
        "maxVertexAttributes": limits.max_vertex_attributes,
        "maxVertexBufferArrayStride": limits.max_vertex_buffer_array_stride,
        "maxInterStageShaderVariables": limits.max_inter_stage_shader_variables,
        "maxColorAttachments": limits.max_color_attachments,
        "maxColorAttachmentBytesPerSample": limits.max_color_attachment_bytes_per_sample,
        "maxComputeWorkgroupStorageSize": limits.max_compute_workgroup_storage_size,
        "maxComputeInvocationsPerWorkgroup": limits.max_compute_invocations_per_workgroup,
        "maxComputeWorkgroupSizeX": limits.max_compute_workgroup_size_x,
        "maxComputeWorkgroupSizeY": limits.max_compute_workgroup_size_y,
        "maxComputeWorkgroupSizeZ": limits.max_compute_workgroup_size_z,
        "maxComputeWorkgroupsPerDimension": limits.max_compute_workgroups_per_dimension,
        "maxImmediateSize": limits.max_immediate_size,
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webgpu_feature_names_round_trip_with_required_dependencies() {
        let names = WEBGPU_FEATURES
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect::<Vec<_>>();
        let features = gpu_features(&names).unwrap();
        assert_eq!(gpu_feature_names(features), names);

        let features = gpu_features(&["texture-compression-bc-sliced-3d".to_owned()]).unwrap();
        assert!(features.contains(Features::TEXTURE_COMPRESSION_BC_SLICED_3D));
        assert!(features.contains(Features::TEXTURE_COMPRESSION_BC));
        assert!(gpu_features(&["native-only-feature".to_owned()]).is_err());
    }

    #[test]
    fn webgpu_core_and_feature_texture_formats_are_recognized() {
        let formats = [
            "r8unorm",
            "r8snorm",
            "r8uint",
            "r8sint",
            "r16uint",
            "r16sint",
            "r16float",
            "rg8unorm",
            "rg8snorm",
            "rg8uint",
            "rg8sint",
            "r32uint",
            "r32sint",
            "r32float",
            "rg16uint",
            "rg16sint",
            "rg16float",
            "rgba8unorm",
            "rgba8unorm-srgb",
            "rgba8snorm",
            "rgba8uint",
            "rgba8sint",
            "bgra8unorm",
            "bgra8unorm-srgb",
            "rgb9e5ufloat",
            "rgb10a2uint",
            "rgb10a2unorm",
            "rg11b10ufloat",
            "rg32uint",
            "rg32sint",
            "rg32float",
            "rgba16uint",
            "rgba16sint",
            "rgba16float",
            "rgba32uint",
            "rgba32sint",
            "rgba32float",
            "stencil8",
            "depth16unorm",
            "depth24plus",
            "depth24plus-stencil8",
            "depth32float",
            "depth32float-stencil8",
            "bc1-rgba-unorm",
            "bc1-rgba-unorm-srgb",
            "bc2-rgba-unorm",
            "bc2-rgba-unorm-srgb",
            "bc3-rgba-unorm",
            "bc3-rgba-unorm-srgb",
            "bc4-r-unorm",
            "bc4-r-snorm",
            "bc5-rg-unorm",
            "bc5-rg-snorm",
            "bc6h-rgb-ufloat",
            "bc6h-rgb-float",
            "bc7-rgba-unorm",
            "bc7-rgba-unorm-srgb",
            "etc2-rgb8unorm",
            "etc2-rgb8unorm-srgb",
            "etc2-rgb8a1unorm",
            "etc2-rgb8a1unorm-srgb",
            "etc2-rgba8unorm",
            "etc2-rgba8unorm-srgb",
            "eac-r11unorm",
            "eac-r11snorm",
            "eac-rg11unorm",
            "eac-rg11snorm",
            "astc-4x4-unorm",
            "astc-12x12-unorm-srgb",
        ];
        for format in formats {
            assert!(
                texture_format(format).is_ok(),
                "format {format} was rejected"
            );
        }
        assert!(texture_format("astc-7x7-unorm").is_err());
        assert!(texture_format("not-a-format").is_err());
    }
}
