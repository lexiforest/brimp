use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString, c_char, c_void},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use browser_dom::NodeId;
use glow::HasContext;
use khronos_egl as egl;

type EglDisplay = *mut c_void;
type EglConfig = *mut c_void;
type EglContext = *mut c_void;
type EglSurface = *mut c_void;
#[cfg(target_os = "macos")]
type EglNativeDisplay = i32;
#[cfg(not(target_os = "macos"))]
type EglNativeDisplay = *mut c_void;
type EglProc = unsafe extern "system" fn();
type EglGetProcAddress = unsafe extern "system" fn(*const c_char) -> Option<EglProc>;
type GlDrawArraysInstancedAngle = unsafe extern "system" fn(u32, i32, i32, i32);
type GlDrawElementsInstancedAngle = unsafe extern "system" fn(u32, i32, u32, *const c_void, i32);
type GlVertexAttribDivisorAngle = unsafe extern "system" fn(u32, u32);
type GlDrawBuffersExt = unsafe extern "system" fn(i32, *const u32);
type GlDrawRangeElements = unsafe extern "system" fn(u32, u32, u32, i32, u32, *const c_void);
type GlGetVertexAttribiv = unsafe extern "system" fn(u32, u32, *mut i32);
type GlGetVertexAttribPointerv = unsafe extern "system" fn(u32, u32, *mut *mut c_void);
type GlRequestExtensionAngle = unsafe extern "system" fn(*const c_char);
type GlCompressedTexImage2D =
    unsafe extern "system" fn(u32, i32, u32, i32, i32, i32, i32, *const c_void);
type GlCompressedTexImage3D =
    unsafe extern "system" fn(u32, i32, u32, i32, i32, i32, i32, i32, *const c_void);
type GlGetQueryiv = unsafe extern "system" fn(u32, u32, *mut i32);
type GlGetShaderiv = unsafe extern "system" fn(u32, u32, *mut i32);
type GlGetTranslatedShaderSourceAngle = unsafe extern "system" fn(u32, i32, *mut i32, *mut c_char);
type GlGetString = unsafe extern "system" fn(u32) -> *const u8;
type GlEnableiOes = unsafe extern "system" fn(u32, u32);
type GlBlendEquationiOes = unsafe extern "system" fn(u32, u32);
type GlBlendEquationSeparateiOes = unsafe extern "system" fn(u32, u32, u32);
type GlBlendFunciOes = unsafe extern "system" fn(u32, u32, u32);
type GlBlendFuncSeparateiOes = unsafe extern "system" fn(u32, u32, u32, u32, u32);
type GlColorMaskiOes = unsafe extern "system" fn(u32, u8, u8, u8, u8);
type GlGetIntegeriV = unsafe extern "system" fn(u32, u32, *mut i32);
type GlClipControlExt = unsafe extern "system" fn(u32, u32);
type GlPolygonOffsetClampExt = unsafe extern "system" fn(f32, f32, f32);
type GlProvokingVertexAngle = unsafe extern "system" fn(u32);
type GlPolygonModeAngle = unsafe extern "system" fn(u32, u32);
type GlMultiDrawArraysAngle = unsafe extern "system" fn(u32, *const i32, *const i32, i32);
type GlMultiDrawElementsAngle =
    unsafe extern "system" fn(u32, *const i32, u32, *const *const c_void, i32);
type GlMultiDrawArraysInstancedAngle =
    unsafe extern "system" fn(u32, *const i32, *const i32, *const i32, i32);
type GlMultiDrawElementsInstancedAngle =
    unsafe extern "system" fn(u32, *const i32, u32, *const *const c_void, *const i32, i32);
type GlDrawArraysInstancedBaseInstanceAngle = unsafe extern "system" fn(u32, i32, i32, i32, u32);
type GlDrawElementsInstancedBaseVertexBaseInstanceAngle =
    unsafe extern "system" fn(u32, i32, u32, *const c_void, i32, i32, u32);
type GlMultiDrawArraysInstancedBaseInstanceAngle =
    unsafe extern "system" fn(u32, *const i32, *const i32, *const i32, *const u32, i32);
type GlMultiDrawElementsInstancedBaseVertexBaseInstanceAngle = unsafe extern "system" fn(
    u32,
    *const i32,
    u32,
    *const *const c_void,
    *const i32,
    *const i32,
    *const u32,
    i32,
);

struct EglApi {
    get_display: unsafe extern "system" fn(EglNativeDisplay) -> EglDisplay,
    initialize: unsafe extern "system" fn(EglDisplay, *mut i32, *mut i32) -> u32,
    choose_config:
        unsafe extern "system" fn(EglDisplay, *const i32, *mut EglConfig, i32, *mut i32) -> u32,
    bind_api: unsafe extern "system" fn(u32) -> u32,
    create_pbuffer_surface:
        unsafe extern "system" fn(EglDisplay, EglConfig, *const i32) -> EglSurface,
    destroy_surface: unsafe extern "system" fn(EglDisplay, EglSurface) -> u32,
    create_context:
        unsafe extern "system" fn(EglDisplay, EglConfig, EglContext, *const i32) -> EglContext,
    destroy_context: unsafe extern "system" fn(EglDisplay, EglContext) -> u32,
    make_current: unsafe extern "system" fn(EglDisplay, EglSurface, EglSurface, EglContext) -> u32,
    get_error: unsafe extern "system" fn() -> u32,
    get_proc_address: EglGetProcAddress,
}

impl EglApi {
    unsafe fn load(library: &libloading::Library) -> Result<Self, String> {
        let get_proc_address = unsafe {
            load_export::<EglGetProcAddress>(
                library,
                &[b"eglGetProcAddress\0", b"EGL_GetProcAddress\0"],
            )
        }
        .ok_or("ANGLE library does not export an EGL procedure resolver")?;

        macro_rules! procedure {
            ($name:literal, $prefixed:literal, $kind:ty) => {{
                unsafe { load_export::<$kind>(library, &[$name, $prefixed]) }
                    .or_else(|| unsafe { resolve_egl::<$kind>(get_proc_address, $name) })
                    .ok_or_else(|| {
                        format!(
                            "ANGLE procedure {} is unavailable",
                            String::from_utf8_lossy(&$name[..$name.len() - 1])
                        )
                    })?
            }};
        }

        Ok(Self {
            get_display: procedure!(
                b"eglGetDisplay\0",
                b"EGL_GetDisplay\0",
                unsafe extern "system" fn(EglNativeDisplay) -> EglDisplay
            ),
            initialize: procedure!(
                b"eglInitialize\0",
                b"EGL_Initialize\0",
                unsafe extern "system" fn(EglDisplay, *mut i32, *mut i32) -> u32
            ),
            choose_config: procedure!(
                b"eglChooseConfig\0",
                b"EGL_ChooseConfig\0",
                unsafe extern "system" fn(
                    EglDisplay,
                    *const i32,
                    *mut EglConfig,
                    i32,
                    *mut i32,
                ) -> u32
            ),
            bind_api: procedure!(
                b"eglBindAPI\0",
                b"EGL_BindAPI\0",
                unsafe extern "system" fn(u32) -> u32
            ),
            create_pbuffer_surface: procedure!(
                b"eglCreatePbufferSurface\0",
                b"EGL_CreatePbufferSurface\0",
                unsafe extern "system" fn(EglDisplay, EglConfig, *const i32) -> EglSurface
            ),
            destroy_surface: procedure!(
                b"eglDestroySurface\0",
                b"EGL_DestroySurface\0",
                unsafe extern "system" fn(EglDisplay, EglSurface) -> u32
            ),
            create_context: procedure!(
                b"eglCreateContext\0",
                b"EGL_CreateContext\0",
                unsafe extern "system" fn(
                    EglDisplay,
                    EglConfig,
                    EglContext,
                    *const i32,
                ) -> EglContext
            ),
            destroy_context: procedure!(
                b"eglDestroyContext\0",
                b"EGL_DestroyContext\0",
                unsafe extern "system" fn(EglDisplay, EglContext) -> u32
            ),
            make_current: procedure!(
                b"eglMakeCurrent\0",
                b"EGL_MakeCurrent\0",
                unsafe extern "system" fn(EglDisplay, EglSurface, EglSurface, EglContext) -> u32
            ),
            get_error: procedure!(
                b"eglGetError\0",
                b"EGL_GetError\0",
                unsafe extern "system" fn() -> u32
            ),
            get_proc_address,
        })
    }
}

unsafe fn load_export<T: Copy>(library: &libloading::Library, names: &[&[u8]]) -> Option<T> {
    names
        .iter()
        .find_map(|name| unsafe { library.get::<T>(name) }.ok().map(|symbol| *symbol))
}

unsafe fn resolve_egl<T: Copy>(get_proc_address: EglGetProcAddress, name: &[u8]) -> Option<T> {
    let name = name.as_ptr().cast::<c_char>();
    let procedure = unsafe { get_proc_address(name) }?;
    (std::mem::size_of::<T>() == std::mem::size_of::<EglProc>())
        .then(|| unsafe { std::mem::transmute_copy(&procedure) })
}

fn requestable_extensions(backend: &Backend) -> HashSet<String> {
    const REQUESTABLE_EXTENSIONS_ANGLE: u32 = 0x93a8;

    let Some(get_string) =
        (unsafe { resolve_egl::<GlGetString>(backend.egl.get_proc_address, b"glGetString\0") })
    else {
        return HashSet::new();
    };
    let value = unsafe { get_string(REQUESTABLE_EXTENSIONS_ANGLE) };
    if value.is_null() {
        return HashSet::new();
    }
    unsafe { CStr::from_ptr(value.cast::<c_char>()) }
        .to_string_lossy()
        .split_ascii_whitespace()
        .map(str::to_owned)
        .collect()
}

struct Backend {
    _library: libloading::Library,
    egl: EglApi,
    display: EglDisplay,
}

// EGL displays and procedure tables are process-wide handles. ANGLE supports using
// independent contexts on separate threads; retaining one shared backend also prevents
// one page from terminating the display while another page is still rendering.
unsafe impl Send for Backend {}
unsafe impl Sync for Backend {}

static BACKEND: OnceLock<Result<Option<Arc<Backend>>, String>> = OnceLock::new();
static ANGLE_ACCESS: Mutex<()> = Mutex::new(());

pub(crate) struct AngleAccessGuard {
    _guard: MutexGuard<'static, ()>,
}

impl Drop for AngleAccessGuard {
    fn drop(&mut self) {
        let Some(Ok(Some(backend))) = BACKEND.get() else {
            return;
        };
        unsafe {
            (backend.egl.make_current)(
                backend.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
    }
}

pub(crate) fn lock() -> AngleAccessGuard {
    AngleAccessGuard {
        _guard: ANGLE_ACCESS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    }
}

struct WebGlContext {
    config: EglConfig,
    egl_context: EglContext,
    surface: EglSurface,
    gl: glow::Context,
    requestable_extensions: HashSet<String>,
    version: u8,
    next_object: u64,
    shaders: HashMap<u64, glow::NativeShader>,
    programs: HashMap<u64, glow::NativeProgram>,
    buffers: HashMap<u64, glow::NativeBuffer>,
    textures: HashMap<u64, glow::NativeTexture>,
    framebuffers: HashMap<u64, glow::NativeFramebuffer>,
    renderbuffers: HashMap<u64, glow::NativeRenderbuffer>,
    samplers: HashMap<u64, glow::NativeSampler>,
    queries: HashMap<u64, glow::NativeQuery>,
    syncs: HashMap<u64, glow::NativeFence>,
    transform_feedbacks: HashMap<u64, glow::NativeTransformFeedback>,
    bound_framebuffer: Option<u64>,
    bound_read_framebuffer: Option<u64>,
    vertex_arrays: HashMap<u64, glow::NativeVertexArray>,
    uniforms: HashMap<u64, glow::NativeUniformLocation>,
}

pub(crate) enum UniformValue {
    Float(Vec<f32>),
    Int(Vec<i32>),
    Uint(Vec<u32>),
}

#[derive(Default)]
pub(crate) struct AngleStore {
    backend: Option<Arc<Backend>>,
    contexts: HashMap<NodeId, WebGlContext>,
}

const WEBGL_EXTENSION_MAPPINGS: &[(&str, &str)] = &[
    ("GL_OES_vertex_array_object", "OES_vertex_array_object"),
    ("GL_OES_element_index_uint", "OES_element_index_uint"),
    ("GL_OES_standard_derivatives", "OES_standard_derivatives"),
    ("GL_EXT_frag_depth", "EXT_frag_depth"),
    ("GL_EXT_shader_texture_lod", "EXT_shader_texture_lod"),
    ("GL_EXT_blend_minmax", "EXT_blend_minmax"),
    ("GL_ANGLE_translated_shader_source", "WEBGL_debug_shaders"),
    (
        "GL_KHR_parallel_shader_compile",
        "KHR_parallel_shader_compile",
    ),
    ("GL_EXT_clip_control", "EXT_clip_control"),
    ("GL_EXT_polygon_offset_clamp", "EXT_polygon_offset_clamp"),
    ("GL_EXT_depth_clamp", "EXT_depth_clamp"),
    (
        "GL_EXT_texture_mirror_clamp_to_edge",
        "EXT_texture_mirror_clamp_to_edge",
    ),
    ("GL_EXT_texture_norm16", "EXT_texture_norm16"),
    ("GL_EXT_render_snorm", "EXT_render_snorm"),
    ("GL_EXT_conservative_depth", "EXT_conservative_depth"),
    (
        "GL_NV_shader_noperspective_interpolation",
        "NV_shader_noperspective_interpolation",
    ),
    ("GL_OES_sample_variables", "OES_sample_variables"),
    (
        "GL_OES_shader_multisample_interpolation",
        "OES_shader_multisample_interpolation",
    ),
    ("GL_ANGLE_clip_cull_distance", "WEBGL_clip_cull_distance"),
    ("GL_ANGLE_provoking_vertex", "WEBGL_provoking_vertex"),
    ("GL_ANGLE_stencil_texturing", "WEBGL_stencil_texturing"),
    (
        "GL_QCOM_render_shared_exponent",
        "WEBGL_render_shared_exponent",
    ),
    ("GL_ANGLE_instanced_arrays", "ANGLE_instanced_arrays"),
    ("GL_EXT_draw_buffers", "WEBGL_draw_buffers"),
    ("GL_OES_texture_float", "OES_texture_float"),
    ("GL_OES_texture_half_float", "OES_texture_half_float"),
    ("GL_OES_texture_float_linear", "OES_texture_float_linear"),
    (
        "GL_OES_texture_half_float_linear",
        "OES_texture_half_float_linear",
    ),
    (
        "GL_EXT_texture_filter_anisotropic",
        "EXT_texture_filter_anisotropic",
    ),
    (
        "GL_EXT_texture_compression_s3tc",
        "WEBGL_compressed_texture_s3tc",
    ),
    (
        "GL_EXT_texture_compression_s3tc_srgb",
        "WEBGL_compressed_texture_s3tc_srgb",
    ),
    (
        "GL_EXT_texture_compression_bptc",
        "EXT_texture_compression_bptc",
    ),
    (
        "GL_EXT_texture_compression_rgtc",
        "EXT_texture_compression_rgtc",
    ),
    (
        "GL_KHR_texture_compression_astc_ldr",
        "WEBGL_compressed_texture_astc",
    ),
    (
        "GL_IMG_texture_compression_pvrtc",
        "WEBGL_compressed_texture_pvrtc",
    ),
    ("GL_EXT_sRGB", "EXT_sRGB"),
    (
        "GL_OES_compressed_ETC1_RGB8_texture",
        "WEBGL_compressed_texture_etc1",
    ),
    ("GL_OES_fbo_render_mipmap", "OES_fbo_render_mipmap"),
    ("GL_EXT_blend_func_extended", "WEBGL_blend_func_extended"),
    ("GL_ANGLE_polygon_mode", "WEBGL_polygon_mode"),
];

impl AngleStore {
    pub(crate) fn supported_webgl_extensions(
        &self,
        id: NodeId,
    ) -> Result<Vec<&'static str>, String> {
        let context = self.contexts.get(&id).ok_or("unknown WebGL context")?;
        let native = context.gl.supported_extensions();
        let available =
            |name: &str| native.contains(name) || context.requestable_extensions.contains(name);
        let mut supported = WEBGL_EXTENSION_MAPPINGS
            .iter()
            .copied()
            .filter_map(|(native_name, web_name)| {
                (available(native_name)
                    && !(context.version == 2
                        && matches!(
                            web_name,
                            "OES_vertex_array_object"
                                | "OES_element_index_uint"
                                | "OES_standard_derivatives"
                                | "EXT_frag_depth"
                                | "EXT_shader_texture_lod"
                                | "EXT_blend_minmax"
                                | "ANGLE_instanced_arrays"
                                | "WEBGL_draw_buffers"
                                | "OES_texture_float"
                                | "OES_texture_half_float"
                                | "EXT_sRGB"
                                | "OES_fbo_render_mipmap"
                        ))
                    && !(context.version == 1
                        && matches!(
                            web_name,
                            "EXT_texture_norm16"
                                | "EXT_render_snorm"
                                | "EXT_conservative_depth"
                                | "NV_shader_noperspective_interpolation"
                                | "OES_sample_variables"
                                | "OES_shader_multisample_interpolation"
                                | "WEBGL_clip_cull_distance"
                                | "WEBGL_provoking_vertex"
                                | "WEBGL_stencil_texturing"
                                | "WEBGL_render_shared_exponent"
                        )))
                .then_some(web_name)
            })
            .collect::<Vec<_>>();
        if context.version == 1
            && (available("GL_ANGLE_depth_texture") || available("GL_OES_depth_texture"))
        {
            supported.push("WEBGL_depth_texture");
        }
        if available("GL_ANGLE_compressed_texture_etc") {
            supported.push("WEBGL_compressed_texture_etc");
        }
        if available("GL_EXT_disjoint_timer_query") {
            supported.push(if context.version == 1 {
                "EXT_disjoint_timer_query"
            } else {
                "EXT_disjoint_timer_query_webgl2"
            });
        }
        if available("GL_EXT_color_buffer_half_float") {
            supported.push("EXT_color_buffer_half_float");
        }
        if available("GL_EXT_color_buffer_float") {
            if context.version == 1 && available("GL_OES_texture_float") {
                supported.push("WEBGL_color_buffer_float");
            } else if context.version == 2 {
                supported.push("EXT_color_buffer_float");
            }
        }
        if available("GL_EXT_float_blend") {
            supported.push("EXT_float_blend");
        }
        if context.version == 2 && available("GL_OES_draw_buffers_indexed") {
            supported.push("OES_draw_buffers_indexed");
        }
        if available("GL_ANGLE_multi_draw")
            && (context.version == 2 || available("GL_ANGLE_instanced_arrays"))
        {
            supported.push("WEBGL_multi_draw");
        }
        if context.version == 2 && available("GL_ANGLE_base_vertex_base_instance") {
            supported.push("WEBGL_draw_instanced_base_vertex_base_instance");
            if available("GL_ANGLE_multi_draw") {
                supported.push("WEBGL_multi_draw_instanced_base_vertex_base_instance");
            }
        }
        Ok(supported)
    }

    pub(crate) fn enable_webgl_extension(
        &mut self,
        id: NodeId,
        web_name: &str,
    ) -> Result<(), String> {
        let (backend, context) = self.current_mut(id)?;
        let mut native_names = Vec::new();
        if web_name == "WEBGL_color_buffer_float" {
            native_names.push("GL_OES_texture_float");
        } else if web_name == "EXT_color_buffer_half_float" && context.version == 1 {
            native_names.push("GL_OES_texture_half_float");
        } else if web_name == "WEBGL_multi_draw" && context.version == 1 {
            native_names.push("GL_ANGLE_instanced_arrays");
        } else if web_name == "WEBGL_multi_draw_instanced_base_vertex_base_instance" {
            native_names.push("GL_ANGLE_multi_draw");
        }
        if let Some((native_name, _)) = WEBGL_EXTENSION_MAPPINGS
            .iter()
            .find(|(_, candidate)| *candidate == web_name)
        {
            native_names.push(*native_name);
        } else {
            native_names.push(match web_name {
                "WEBGL_depth_texture"
                    if context
                        .gl
                        .supported_extensions()
                        .contains("GL_ANGLE_depth_texture")
                        || context
                            .requestable_extensions
                            .contains("GL_ANGLE_depth_texture") =>
                {
                    "GL_ANGLE_depth_texture"
                }
                "WEBGL_depth_texture" => "GL_OES_depth_texture",
                "WEBGL_compressed_texture_etc" => "GL_ANGLE_compressed_texture_etc",
                "EXT_disjoint_timer_query" | "EXT_disjoint_timer_query_webgl2" => {
                    "GL_EXT_disjoint_timer_query"
                }
                "EXT_color_buffer_half_float" => "GL_EXT_color_buffer_half_float",
                "WEBGL_color_buffer_float" | "EXT_color_buffer_float" => {
                    "GL_EXT_color_buffer_float"
                }
                "EXT_float_blend" => "GL_EXT_float_blend",
                "OES_draw_buffers_indexed" => "GL_OES_draw_buffers_indexed",
                "WEBGL_multi_draw" => "GL_ANGLE_multi_draw",
                "WEBGL_draw_instanced_base_vertex_base_instance" => {
                    "GL_ANGLE_base_vertex_base_instance"
                }
                "WEBGL_multi_draw_instanced_base_vertex_base_instance" => {
                    "GL_ANGLE_base_vertex_base_instance"
                }
                _ => return Err("unknown WebGL extension".to_owned()),
            });
        }
        native_names.retain(|native_name| context.requestable_extensions.contains(*native_name));
        if native_names.is_empty() {
            return Ok(());
        }
        let request = unsafe {
            resolve_egl::<GlRequestExtensionAngle>(
                backend.egl.get_proc_address,
                b"glRequestExtensionANGLE\0",
            )
        }
        .ok_or("ANGLE extension request entry point is unavailable")?;
        for native_name in native_names {
            let name = CString::new(native_name).expect("ANGLE extension names contain no NUL");
            unsafe { request(name.as_ptr()) };
        }
        Ok(())
    }

    pub(crate) fn create(
        &mut self,
        id: NodeId,
        width: u32,
        height: u32,
        version: u8,
    ) -> Result<bool, String> {
        if self.contexts.contains_key(&id) {
            return Ok(true);
        }
        if width == 0 || height == 0 {
            return Ok(false);
        }
        if self.backend.is_none() {
            self.backend = load_backend()?;
        }
        let Some(backend) = self.backend.as_ref() else {
            return Ok(false);
        };
        if unsafe { (backend.egl.bind_api)(egl::OPENGL_ES_API) } == egl::FALSE {
            return Err(egl_error(backend, "eglBindAPI"));
        }
        let renderable = if version == 2 {
            egl::OPENGL_ES3_BIT
        } else {
            egl::OPENGL_ES2_BIT
        };
        let config_attributes = [
            egl::SURFACE_TYPE,
            egl::PBUFFER_BIT,
            egl::RENDERABLE_TYPE,
            renderable,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::DEPTH_SIZE,
            24,
            egl::STENCIL_SIZE,
            8,
            egl::NONE,
        ];
        let mut config = std::ptr::null_mut();
        let mut config_count = 0;
        if unsafe {
            (backend.egl.choose_config)(
                backend.display,
                config_attributes.as_ptr(),
                &mut config,
                1,
                &mut config_count,
            )
        } == egl::FALSE
        {
            return Err(egl_error(backend, "eglChooseConfig"));
        }
        if config_count == 0 || config.is_null() {
            return Err("ANGLE exposed no compatible EGL configuration".to_owned());
        }
        let surface_attributes = [
            egl::WIDTH,
            width as i32,
            egl::HEIGHT,
            height as i32,
            egl::NONE,
        ];
        let surface = unsafe {
            (backend.egl.create_pbuffer_surface)(
                backend.display,
                config,
                surface_attributes.as_ptr(),
            )
        };
        if surface.is_null() {
            return Err(egl_error(backend, "eglCreatePbufferSurface"));
        }
        let context_attributes = [
            egl::CONTEXT_CLIENT_VERSION,
            if version == 2 { 3 } else { 2 },
            egl::NONE,
        ];
        let egl_context = unsafe {
            (backend.egl.create_context)(
                backend.display,
                config,
                std::ptr::null_mut(),
                context_attributes.as_ptr(),
            )
        };
        if egl_context.is_null() {
            unsafe { (backend.egl.destroy_surface)(backend.display, surface) };
            return Err(egl_error(backend, "eglCreateContext"));
        }
        if unsafe { (backend.egl.make_current)(backend.display, surface, surface, egl_context) }
            == egl::FALSE
        {
            unsafe {
                (backend.egl.destroy_context)(backend.display, egl_context);
                (backend.egl.destroy_surface)(backend.display, surface);
            }
            return Err(egl_error(backend, "eglMakeCurrent"));
        }
        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                CString::new(name)
                    .ok()
                    .and_then(|name| (backend.egl.get_proc_address)(name.as_ptr()))
                    .map(|function| function as *const () as *const c_void)
                    .unwrap_or(std::ptr::null())
            })
        };
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
        }
        let requestable_extensions = if gl
            .supported_extensions()
            .contains("GL_ANGLE_request_extension")
        {
            requestable_extensions(backend)
        } else {
            HashSet::new()
        };
        self.contexts.insert(
            id,
            WebGlContext {
                config,
                egl_context,
                surface,
                gl,
                requestable_extensions,
                version,
                next_object: 0,
                shaders: HashMap::new(),
                programs: HashMap::new(),
                buffers: HashMap::new(),
                textures: HashMap::new(),
                framebuffers: HashMap::new(),
                renderbuffers: HashMap::new(),
                samplers: HashMap::new(),
                queries: HashMap::new(),
                syncs: HashMap::new(),
                transform_feedbacks: HashMap::new(),
                bound_framebuffer: None,
                bound_read_framebuffer: None,
                vertex_arrays: HashMap::new(),
                uniforms: HashMap::new(),
            },
        );
        Ok(true)
    }

    pub(crate) fn resize(&mut self, id: NodeId, width: u32, height: u32) -> Result<(), String> {
        let Some(context) = self.contexts.get_mut(&id) else {
            return Ok(());
        };
        if width == 0 || height == 0 {
            unsafe { context.gl.viewport(0, 0, 0, 0) };
            return Ok(());
        }
        let backend = self.backend.as_ref().ok_or("ANGLE is unavailable")?;
        let surface_attributes = [
            egl::WIDTH,
            width as i32,
            egl::HEIGHT,
            height as i32,
            egl::NONE,
        ];
        let new_surface = unsafe {
            (backend.egl.create_pbuffer_surface)(
                backend.display,
                context.config,
                surface_attributes.as_ptr(),
            )
        };
        if new_surface.is_null() {
            return Err(egl_error(backend, "eglCreatePbufferSurface"));
        }
        if unsafe {
            (backend.egl.make_current)(
                backend.display,
                new_surface,
                new_surface,
                context.egl_context,
            )
        } == egl::FALSE
        {
            unsafe { (backend.egl.destroy_surface)(backend.display, new_surface) };
            return Err(egl_error(backend, "eglMakeCurrent"));
        }
        let old_surface = std::mem::replace(&mut context.surface, new_surface);
        unsafe {
            (backend.egl.destroy_surface)(backend.display, old_surface);
            context.gl.viewport(0, 0, width as i32, height as i32);
        }
        Ok(())
    }

    pub(crate) fn lose_context(&mut self, id: NodeId) -> Result<bool, String> {
        let Some(context) = self.contexts.remove(&id) else {
            return Ok(false);
        };
        let backend = self.backend.as_ref().ok_or("ANGLE is unavailable")?;
        unsafe {
            if (backend.egl.make_current)(
                backend.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == egl::FALSE
            {
                return Err(egl_error(backend, "eglMakeCurrent"));
            }
            (backend.egl.destroy_context)(backend.display, context.egl_context);
            (backend.egl.destroy_surface)(backend.display, context.surface);
        }
        Ok(true)
    }

    pub(crate) fn clear_color(&self, id: NodeId, color: [f32; 4]) -> Result<(), String> {
        let (backend, context) = self.current(id)?;
        unsafe {
            context
                .gl
                .clear_color(color[0], color[1], color[2], color[3]);
        }
        let _ = backend;
        Ok(())
    }

    pub(crate) fn clear(&self, id: NodeId, mask: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context.gl.clear(mask);
        }
        Ok(())
    }

    pub(crate) fn clear_depth(&self, id: NodeId, depth: f32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.clear_depth_f32(depth) };
        Ok(())
    }

    pub(crate) fn clear_stencil(&self, id: NodeId, stencil: i32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.clear_stencil(stencil) };
        Ok(())
    }

    pub(crate) fn set_enabled(
        &self,
        id: NodeId,
        capability: u32,
        enabled: bool,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            if enabled {
                context.gl.enable(capability);
            } else {
                context.gl.disable(capability);
            }
        }
        Ok(())
    }

    pub(crate) fn is_enabled(&self, id: NodeId, capability: u32) -> Result<bool, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.is_enabled(capability) })
    }

    pub(crate) fn scissor(&self, id: NodeId, values: [i32; 4]) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context
                .gl
                .scissor(values[0], values[1], values[2], values[3])
        };
        Ok(())
    }

    pub(crate) fn color_mask(&self, id: NodeId, values: [bool; 4]) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context
                .gl
                .color_mask(values[0], values[1], values[2], values[3])
        };
        Ok(())
    }

    pub(crate) fn depth_mask(&self, id: NodeId, value: bool) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.depth_mask(value) };
        Ok(())
    }

    pub(crate) fn depth_func(&self, id: NodeId, value: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.depth_func(value) };
        Ok(())
    }

    pub(crate) fn depth_range(&self, id: NodeId, near: f32, far: f32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.depth_range_f32(near, far) };
        Ok(())
    }

    pub(crate) fn blend_func(
        &self,
        id: NodeId,
        source: u32,
        destination: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.blend_func(source, destination) };
        Ok(())
    }

    pub(crate) fn blend_color(&self, id: NodeId, color: [f32; 4]) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context
                .gl
                .blend_color(color[0], color[1], color[2], color[3])
        };
        Ok(())
    }

    pub(crate) fn blend_func_separate(
        &self,
        id: NodeId,
        source_rgb: u32,
        destination_rgb: u32,
        source_alpha: u32,
        destination_alpha: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context.gl.blend_func_separate(
                source_rgb,
                destination_rgb,
                source_alpha,
                destination_alpha,
            )
        };
        Ok(())
    }

    pub(crate) fn blend_equation(&self, id: NodeId, mode: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.blend_equation(mode) };
        Ok(())
    }

    pub(crate) fn blend_equation_separate(
        &self,
        id: NodeId,
        mode_rgb: u32,
        mode_alpha: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.blend_equation_separate(mode_rgb, mode_alpha) };
        Ok(())
    }

    pub(crate) fn clip_control(
        &mut self,
        id: NodeId,
        origin: u32,
        depth: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlClipControlExt>(backend.egl.get_proc_address, b"glClipControlEXT\0")
        }
        .ok_or("ANGLE clip-control entry point is unavailable")?;
        unsafe { function(origin, depth) };
        Ok(())
    }

    pub(crate) fn polygon_offset_clamp(
        &mut self,
        id: NodeId,
        factor: f32,
        units: f32,
        clamp: f32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlPolygonOffsetClampExt>(
                backend.egl.get_proc_address,
                b"glPolygonOffsetClampEXT\0",
            )
        }
        .ok_or("ANGLE polygon-offset-clamp entry point is unavailable")?;
        unsafe { function(factor, units, clamp) };
        Ok(())
    }

    pub(crate) fn provoking_vertex(&mut self, id: NodeId, mode: u32) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlProvokingVertexAngle>(
                backend.egl.get_proc_address,
                b"glProvokingVertexANGLE\0",
            )
        }
        .ok_or("ANGLE provoking-vertex entry point is unavailable")?;
        unsafe { function(mode) };
        Ok(())
    }

    pub(crate) fn polygon_mode(&mut self, id: NodeId, face: u32, mode: u32) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlPolygonModeAngle>(backend.egl.get_proc_address, b"glPolygonModeANGLE\0")
        }
        .ok_or("ANGLE polygon-mode entry point is unavailable")?;
        unsafe { function(face, mode) };
        Ok(())
    }

    pub(crate) fn set_enabled_indexed(
        &mut self,
        id: NodeId,
        target: u32,
        index: u32,
        enabled: bool,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let name = if enabled {
            b"glEnableiOES\0".as_slice()
        } else {
            b"glDisableiOES\0".as_slice()
        };
        let function = unsafe { resolve_egl::<GlEnableiOes>(backend.egl.get_proc_address, name) }
            .ok_or("ANGLE indexed enable entry point is unavailable")?;
        unsafe { function(target, index) };
        Ok(())
    }

    pub(crate) fn blend_equation_indexed(
        &mut self,
        id: NodeId,
        index: u32,
        mode: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlBlendEquationiOes>(
                backend.egl.get_proc_address,
                b"glBlendEquationiOES\0",
            )
        }
        .ok_or("ANGLE indexed blend-equation entry point is unavailable")?;
        unsafe { function(index, mode) };
        Ok(())
    }

    pub(crate) fn blend_equation_separate_indexed(
        &mut self,
        id: NodeId,
        index: u32,
        rgb: u32,
        alpha: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlBlendEquationSeparateiOes>(
                backend.egl.get_proc_address,
                b"glBlendEquationSeparateiOES\0",
            )
        }
        .ok_or("ANGLE indexed separate blend-equation entry point is unavailable")?;
        unsafe { function(index, rgb, alpha) };
        Ok(())
    }

    pub(crate) fn blend_func_indexed(
        &mut self,
        id: NodeId,
        index: u32,
        source: u32,
        destination: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlBlendFunciOes>(backend.egl.get_proc_address, b"glBlendFunciOES\0")
        }
        .ok_or("ANGLE indexed blend-function entry point is unavailable")?;
        unsafe { function(index, source, destination) };
        Ok(())
    }

    pub(crate) fn blend_func_separate_indexed(
        &mut self,
        id: NodeId,
        index: u32,
        source_rgb: u32,
        destination_rgb: u32,
        source_alpha: u32,
        destination_alpha: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlBlendFuncSeparateiOes>(
                backend.egl.get_proc_address,
                b"glBlendFuncSeparateiOES\0",
            )
        }
        .ok_or("ANGLE indexed separate blend-function entry point is unavailable")?;
        unsafe {
            function(
                index,
                source_rgb,
                destination_rgb,
                source_alpha,
                destination_alpha,
            )
        };
        Ok(())
    }

    pub(crate) fn color_mask_indexed(
        &mut self,
        id: NodeId,
        index: u32,
        mask: [bool; 4],
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlColorMaskiOes>(backend.egl.get_proc_address, b"glColorMaskiOES\0")
        }
        .ok_or("ANGLE indexed color-mask entry point is unavailable")?;
        unsafe {
            function(
                index,
                u8::from(mask[0]),
                u8::from(mask[1]),
                u8::from(mask[2]),
                u8::from(mask[3]),
            )
        };
        Ok(())
    }

    pub(crate) fn indexed_parameter_i32(
        &mut self,
        id: NodeId,
        parameter: u32,
        index: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.get_parameter_indexed_i32(parameter, index) })
    }

    pub(crate) fn indexed_color_mask(&mut self, id: NodeId, index: u32) -> Result<u8, String> {
        let (backend, _) = self.current_mut(id)?;
        let function = unsafe {
            resolve_egl::<GlGetIntegeriV>(backend.egl.get_proc_address, b"glGetIntegeri_v\0")
        }
        .ok_or("ANGLE indexed state reflection entry point is unavailable")?;
        let mut values = [0_i32; 4];
        unsafe { function(glow::COLOR_WRITEMASK, index, values.as_mut_ptr()) };
        Ok(values
            .into_iter()
            .enumerate()
            .fold(0, |mask, (index, value)| {
                mask | (u8::from(value != 0) << index)
            }))
    }

    pub(crate) fn stencil_func(
        &self,
        id: NodeId,
        function: u32,
        reference: i32,
        mask: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.stencil_func(function, reference, mask) };
        Ok(())
    }

    pub(crate) fn stencil_func_separate(
        &self,
        id: NodeId,
        face: u32,
        function: u32,
        reference: i32,
        mask: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context
                .gl
                .stencil_func_separate(face, function, reference, mask)
        };
        Ok(())
    }

    pub(crate) fn stencil_mask(&self, id: NodeId, mask: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.stencil_mask(mask) };
        Ok(())
    }

    pub(crate) fn stencil_mask_separate(
        &self,
        id: NodeId,
        face: u32,
        mask: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.stencil_mask_separate(face, mask) };
        Ok(())
    }

    pub(crate) fn stencil_op(
        &self,
        id: NodeId,
        fail: u32,
        depth_fail: u32,
        pass: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.stencil_op(fail, depth_fail, pass) };
        Ok(())
    }

    pub(crate) fn stencil_op_separate(
        &self,
        id: NodeId,
        face: u32,
        fail: u32,
        depth_fail: u32,
        pass: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.stencil_op_separate(face, fail, depth_fail, pass) };
        Ok(())
    }

    pub(crate) fn polygon_offset(&self, id: NodeId, factor: f32, units: f32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.polygon_offset(factor, units) };
        Ok(())
    }

    pub(crate) fn sample_coverage(
        &self,
        id: NodeId,
        value: f32,
        invert: bool,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.sample_coverage(value, invert) };
        Ok(())
    }

    pub(crate) fn cull_face(&self, id: NodeId, face: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.cull_face(face) };
        Ok(())
    }

    pub(crate) fn front_face(&self, id: NodeId, winding: u32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.front_face(winding) };
        Ok(())
    }

    pub(crate) fn line_width(&self, id: NodeId, width: f32) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.line_width(width) };
        Ok(())
    }

    pub(crate) fn flush(&self, id: NodeId) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.flush() };
        Ok(())
    }

    pub(crate) fn finish(&self, id: NodeId) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe { context.gl.finish() };
        Ok(())
    }

    pub(crate) fn parameter_string(&self, id: NodeId, parameter: u32) -> Result<String, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_parameter_string(parameter) })
    }

    pub(crate) fn parameter_i32(&self, id: NodeId, parameter: u32) -> Result<i32, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_parameter_i32(parameter) })
    }

    pub(crate) fn parameter_i64(&self, id: NodeId, parameter: u32) -> Result<i64, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_parameter_i64(parameter) })
    }

    pub(crate) fn parameter_bool(&self, id: NodeId, parameter: u32) -> Result<bool, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_parameter_bool(parameter) })
    }

    pub(crate) fn parameter_bool4_mask(&self, id: NodeId, parameter: u32) -> Result<u8, String> {
        let (_, context) = self.current(id)?;
        let values = unsafe { context.gl.get_parameter_bool_array::<4>(parameter) };
        Ok(values
            .into_iter()
            .enumerate()
            .fold(0, |mask, (index, value)| mask | (u8::from(value) << index)))
    }

    pub(crate) fn parameter_f32(&self, id: NodeId, parameter: u32) -> Result<f32, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_parameter_f32(parameter) })
    }

    pub(crate) fn parameter_f32_array<const N: usize>(
        &self,
        id: NodeId,
        parameter: u32,
    ) -> Result<[f32; N], String> {
        let (_, context) = self.current(id)?;
        let mut values = [0.0; N];
        unsafe { context.gl.get_parameter_f32_slice(parameter, &mut values) };
        Ok(values)
    }

    pub(crate) fn parameter_i32_array<const N: usize>(
        &self,
        id: NodeId,
        parameter: u32,
    ) -> Result<[i32; N], String> {
        let (_, context) = self.current(id)?;
        let mut values = [0; N];
        unsafe { context.gl.get_parameter_i32_slice(parameter, &mut values) };
        Ok(values)
    }

    pub(crate) fn error(&self, id: NodeId) -> Result<u32, String> {
        let (_, context) = self.current(id)?;
        Ok(unsafe { context.gl.get_error() })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_pixels(
        &self,
        id: NodeId,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        format: u32,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        let (_, context) = self.current(id)?;
        let components = match format {
            glow::ALPHA
            | glow::LUMINANCE
            | glow::RED
            | glow::RED_INTEGER
            | glow::DEPTH_COMPONENT => 1,
            glow::LUMINANCE_ALPHA | glow::RG | glow::RG_INTEGER => 2,
            glow::RGB | glow::RGB_INTEGER => 3,
            glow::RGBA | glow::RGBA_INTEGER => 4,
            glow::DEPTH_STENCIL => 1,
            _ => return Err("unsupported WebGL readPixels format".to_owned()),
        };
        let pixel_bytes = match kind {
            glow::UNSIGNED_SHORT_5_6_5
            | glow::UNSIGNED_SHORT_4_4_4_4
            | glow::UNSIGNED_SHORT_5_5_5_1 => 2,
            glow::UNSIGNED_INT_2_10_10_10_REV
            | glow::UNSIGNED_INT_10F_11F_11F_REV
            | glow::UNSIGNED_INT_5_9_9_9_REV
            | glow::UNSIGNED_INT_24_8 => 4,
            glow::FLOAT_32_UNSIGNED_INT_24_8_REV => 8,
            glow::UNSIGNED_BYTE | glow::BYTE => components,
            glow::HALF_FLOAT | glow::UNSIGNED_SHORT | glow::SHORT => components * 2,
            glow::FLOAT | glow::UNSIGNED_INT | glow::INT => components * 4,
            _ => return Err("unsupported WebGL readPixels type".to_owned()),
        };
        let length = usize::try_from(u64::from(width) * u64::from(height) * pixel_bytes)
            .map_err(|_| "WebGL readback is too large")?;
        let mut pixels = vec![0; length];
        unsafe {
            context.gl.read_pixels(
                x,
                y,
                width as i32,
                height as i32,
                format,
                kind,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );
        }
        Ok(pixels)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_pixels_offset(
        &self,
        id: NodeId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        kind: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current(id)?;
        unsafe {
            context.gl.read_pixels(
                x,
                y,
                width,
                height,
                format,
                kind,
                glow::PixelPackData::BufferOffset(offset),
            )
        };
        Ok(())
    }

    pub(crate) fn read_canvas_rgba(
        &self,
        id: NodeId,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        let length = usize::try_from(u64::from(width) * u64::from(height) * 4)
            .map_err(|_| "WebGL readback is too large")?;
        if !self.contexts.contains_key(&id) {
            return Ok(vec![0; length]);
        }
        let (_, context) = self.current(id)?;
        let framebuffer = context
            .bound_framebuffer
            .and_then(|framebuffer| context.framebuffers.get(&framebuffer).copied());
        let read_framebuffer = context
            .bound_read_framebuffer
            .and_then(|framebuffer| context.framebuffers.get(&framebuffer).copied());
        let mut pixels = vec![0; length];
        unsafe {
            context.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            context.gl.read_pixels(
                x,
                y,
                width as i32,
                height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );
            if context.version == 2 {
                context
                    .gl
                    .bind_framebuffer(glow::DRAW_FRAMEBUFFER, framebuffer);
                context
                    .gl
                    .bind_framebuffer(glow::READ_FRAMEBUFFER, read_framebuffer);
            } else {
                context.gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
            }
        }
        Ok(pixels)
    }

    pub(crate) fn create_shader(&mut self, id: NodeId, kind: u32) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let shader = unsafe { context.gl.create_shader(kind) }?;
        let object = next_object(context)?;
        context.shaders.insert(object, shader);
        Ok(object)
    }

    pub(crate) fn shader_source(
        &mut self,
        id: NodeId,
        shader: u64,
        source: &str,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        unsafe { context.gl.shader_source(shader, source) };
        Ok(())
    }

    pub(crate) fn compile_shader(&mut self, id: NodeId, shader: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        unsafe { context.gl.compile_shader(shader) };
        Ok(())
    }

    pub(crate) fn shader_status(&mut self, id: NodeId, shader: u64) -> Result<bool, String> {
        let (_, context) = self.current_mut(id)?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        Ok(unsafe { context.gl.get_shader_compile_status(shader) })
    }

    pub(crate) fn shader_log(&mut self, id: NodeId, shader: u64) -> Result<String, String> {
        let (_, context) = self.current_mut(id)?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        Ok(unsafe { context.gl.get_shader_info_log(shader) })
    }

    pub(crate) fn translated_shader_source(
        &mut self,
        id: NodeId,
        shader: u64,
    ) -> Result<String, String> {
        const TRANSLATED_SHADER_SOURCE_LENGTH_ANGLE: u32 = 0x93a0;

        let (backend, context) = self.current_mut(id)?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        let get_shader = unsafe {
            resolve_egl::<GlGetShaderiv>(backend.egl.get_proc_address, b"glGetShaderiv\0")
        }
        .ok_or("ANGLE shader-parameter entry point is unavailable")?;
        let get_translated = unsafe {
            resolve_egl::<GlGetTranslatedShaderSourceAngle>(
                backend.egl.get_proc_address,
                b"glGetTranslatedShaderSourceANGLE\0",
            )
        }
        .ok_or("ANGLE translated-shader entry point is unavailable")?;

        let shader = shader.0.get();
        let mut length = 0;
        unsafe { get_shader(shader, TRANSLATED_SHADER_SOURCE_LENGTH_ANGLE, &mut length) };
        if length <= 1 {
            return Ok(String::new());
        }
        let mut source = vec![0_u8; length as usize];
        let mut written = 0;
        unsafe {
            get_translated(
                shader,
                length,
                &mut written,
                source.as_mut_ptr().cast::<c_char>(),
            )
        };
        source.truncate(written.max(0) as usize);
        Ok(String::from_utf8_lossy(&source).into_owned())
    }

    pub(crate) fn shader_precision_format(
        &mut self,
        id: NodeId,
        shader_type: u32,
        precision_type: u32,
    ) -> Result<Option<(i32, i32, i32)>, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe {
            context
                .gl
                .get_shader_precision_format(shader_type, precision_type)
        }
        .map(|format| (format.range_min, format.range_max, format.precision)))
    }

    pub(crate) fn delete_shader(&mut self, id: NodeId, shader: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(shader) = context.shaders.remove(&shader) {
            unsafe { context.gl.delete_shader(shader) };
        }
        Ok(())
    }

    pub(crate) fn create_program(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let program = unsafe { context.gl.create_program() }?;
        let object = next_object(context)?;
        context.programs.insert(object, program);
        Ok(object)
    }

    pub(crate) fn attach_shader(
        &mut self,
        id: NodeId,
        program: u64,
        shader: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        unsafe { context.gl.attach_shader(program, shader) };
        Ok(())
    }

    pub(crate) fn detach_shader(
        &mut self,
        id: NodeId,
        program: u64,
        shader: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let shader = *context.shaders.get(&shader).ok_or("unknown WebGLShader")?;
        unsafe { context.gl.detach_shader(program, shader) };
        Ok(())
    }

    pub(crate) fn bind_attribute_location(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
        name: &str,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        unsafe { context.gl.bind_attrib_location(program, index, name) };
        Ok(())
    }

    pub(crate) fn transform_feedback_varyings(
        &mut self,
        id: NodeId,
        program: u64,
        varyings: &[String],
        buffer_mode: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = context
            .programs
            .get(&program)
            .copied()
            .ok_or("unknown WebGLProgram")?;
        let varyings = varyings.iter().map(String::as_str).collect::<Vec<_>>();
        unsafe {
            context
                .gl
                .transform_feedback_varyings(program, &varyings, buffer_mode)
        };
        Ok(())
    }

    pub(crate) fn transform_feedback_varying(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
    ) -> Result<Option<(i32, u32, String)>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = context
            .programs
            .get(&program)
            .copied()
            .ok_or("unknown WebGLProgram")?;
        Ok(
            unsafe { context.gl.get_transform_feedback_varying(program, index) }
                .map(|varying| (varying.size, varying.tftype, varying.name)),
        )
    }

    pub(crate) fn link_program(&mut self, id: NodeId, program: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        unsafe { context.gl.link_program(program) };
        Ok(())
    }

    pub(crate) fn validate_program(&mut self, id: NodeId, program: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        unsafe { context.gl.validate_program(program) };
        Ok(())
    }

    pub(crate) fn program_status(&mut self, id: NodeId, program: u64) -> Result<bool, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_program_link_status(program) })
    }

    pub(crate) fn program_validate_status(
        &mut self,
        id: NodeId,
        program: u64,
    ) -> Result<bool, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_program_validate_status(program) })
    }

    pub(crate) fn program_log(&mut self, id: NodeId, program: u64) -> Result<String, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_program_info_log(program) })
    }

    pub(crate) fn program_parameter_i32(
        &mut self,
        id: NodeId,
        program: u64,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_program_parameter_i32(program, parameter) })
    }

    pub(crate) fn active_attribute(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
    ) -> Result<Option<(i32, u32, String)>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_active_attribute(program, index) }
            .map(|attribute| (attribute.size, attribute.atype, attribute.name)))
    }

    pub(crate) fn active_uniform(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
    ) -> Result<Option<(i32, u32, String)>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_active_uniform(program, index) }
            .map(|uniform| (uniform.size, uniform.utype, uniform.name)))
    }

    pub(crate) fn fragment_data_location(
        &mut self,
        id: NodeId,
        program: u64,
        name: &str,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_frag_data_location(program, name) })
    }

    pub(crate) fn use_program(&mut self, id: NodeId, program: Option<u64>) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = program
            .map(|program| {
                context
                    .programs
                    .get(&program)
                    .copied()
                    .ok_or("unknown WebGLProgram")
            })
            .transpose()?;
        unsafe { context.gl.use_program(program) };
        Ok(())
    }

    pub(crate) fn delete_program(&mut self, id: NodeId, program: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(program) = context.programs.remove(&program) {
            unsafe { context.gl.delete_program(program) };
        }
        Ok(())
    }

    pub(crate) fn create_buffer(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let buffer = unsafe { context.gl.create_buffer() }?;
        let object = next_object(context)?;
        context.buffers.insert(object, buffer);
        Ok(object)
    }

    pub(crate) fn bind_buffer(
        &mut self,
        id: NodeId,
        target: u32,
        buffer: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let buffer = buffer
            .map(|buffer| {
                context
                    .buffers
                    .get(&buffer)
                    .copied()
                    .ok_or("unknown WebGLBuffer")
            })
            .transpose()?;
        unsafe { context.gl.bind_buffer(target, buffer) };
        Ok(())
    }

    pub(crate) fn bind_buffer_base(
        &mut self,
        id: NodeId,
        target: u32,
        index: u32,
        buffer: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let buffer = buffer
            .map(|buffer| {
                context
                    .buffers
                    .get(&buffer)
                    .copied()
                    .ok_or("unknown WebGLBuffer")
            })
            .transpose()?;
        unsafe { context.gl.bind_buffer_base(target, index, buffer) };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bind_buffer_range(
        &mut self,
        id: NodeId,
        target: u32,
        index: u32,
        buffer: Option<u64>,
        offset: i32,
        size: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let buffer = buffer
            .map(|buffer| {
                context
                    .buffers
                    .get(&buffer)
                    .copied()
                    .ok_or("unknown WebGLBuffer")
            })
            .transpose()?;
        unsafe {
            context
                .gl
                .bind_buffer_range(target, index, buffer, offset, size)
        };
        Ok(())
    }

    pub(crate) fn buffer_data(
        &mut self,
        id: NodeId,
        target: u32,
        bytes: &[u8],
        usage: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.buffer_data_u8_slice(target, bytes, usage) };
        Ok(())
    }

    pub(crate) fn buffer_sub_data(
        &mut self,
        id: NodeId,
        target: u32,
        offset: i32,
        bytes: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.buffer_sub_data_u8_slice(target, offset, bytes) };
        Ok(())
    }

    pub(crate) fn copy_buffer_sub_data(
        &mut self,
        id: NodeId,
        read_target: u32,
        write_target: u32,
        read_offset: i32,
        write_offset: i32,
        size: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.copy_buffer_sub_data(
                read_target,
                write_target,
                read_offset,
                write_offset,
                size,
            )
        };
        Ok(())
    }

    pub(crate) fn buffer_parameter_i32(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.get_buffer_parameter_i32(target, parameter) })
    }

    pub(crate) fn buffer_sub_data_read(
        &mut self,
        id: NodeId,
        target: u32,
        offset: i32,
        length: usize,
    ) -> Result<Vec<u8>, String> {
        let (_, context) = self.current_mut(id)?;
        let mut bytes = vec![0; length];
        if length == 0 {
            return Ok(bytes);
        }
        let length = i32::try_from(length).map_err(|_| "buffer read length is too large")?;
        let mapped = unsafe {
            context
                .gl
                .map_buffer_range(target, offset, length, glow::MAP_READ_BIT)
        };
        if mapped.is_null() {
            return Err("ANGLE could not map the WebGL buffer for reading".into());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.cast_const(), bytes.as_mut_ptr(), length as usize);
            context.gl.unmap_buffer(target);
        }
        Ok(bytes)
    }

    pub(crate) fn delete_buffer(&mut self, id: NodeId, buffer: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(buffer) = context.buffers.remove(&buffer) {
            unsafe { context.gl.delete_buffer(buffer) };
        }
        Ok(())
    }

    pub(crate) fn create_texture(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let texture = unsafe { context.gl.create_texture() }?;
        let object = next_object(context)?;
        context.textures.insert(object, texture);
        Ok(object)
    }

    pub(crate) fn active_texture(&mut self, id: NodeId, unit: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.active_texture(unit) };
        Ok(())
    }

    pub(crate) fn bind_texture(
        &mut self,
        id: NodeId,
        target: u32,
        texture: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let texture = texture
            .map(|texture| {
                context
                    .textures
                    .get(&texture)
                    .copied()
                    .ok_or("unknown WebGLTexture")
            })
            .transpose()?;
        unsafe { context.gl.bind_texture(target, texture) };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        kind: u32,
        pixels: Option<&[u8]>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_image_2d(
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                kind,
                glow::PixelUnpackData::Slice(pixels),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_image_2d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        kind: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_image_2d(
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                kind,
                glow::PixelUnpackData::BufferOffset(offset),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let image_size =
            i32::try_from(pixels.len()).map_err(|_| "compressed image is too large")?;
        unsafe {
            context.gl.compressed_tex_image_2d(
                target,
                level,
                internal_format as i32,
                width,
                height,
                border,
                image_size,
                pixels,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_image_2d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        image_size: i32,
        offset: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let upload = unsafe {
            resolve_egl::<GlCompressedTexImage2D>(
                backend.egl.get_proc_address,
                b"glCompressedTexImage2D\0",
            )
        }
        .ok_or("WebGL compressed 2D texture entry point is unavailable")?;
        unsafe {
            upload(
                target,
                level,
                internal_format,
                width,
                height,
                border,
                image_size,
                offset as usize as *const c_void,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_sub_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        kind: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_sub_image_2d(
                target,
                level,
                x,
                y,
                width,
                height,
                format,
                kind,
                glow::PixelUnpackData::Slice(Some(pixels)),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_sub_image_2d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        kind: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_sub_image_2d(
                target,
                level,
                x,
                y,
                width,
                height,
                format,
                kind,
                glow::PixelUnpackData::BufferOffset(offset),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_sub_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.compressed_tex_sub_image_2d(
                target,
                level,
                x,
                y,
                width,
                height,
                format,
                glow::CompressedPixelUnpackData::Slice(pixels),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_sub_image_2d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        image_size: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let end = offset
            .checked_add(image_size)
            .ok_or("compressed texture buffer range overflow")?;
        unsafe {
            context.gl.compressed_tex_sub_image_2d(
                target,
                level,
                x,
                y,
                width,
                height,
                format,
                glow::CompressedPixelUnpackData::BufferRange(offset..end),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_image_3d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        kind: u32,
        pixels: Option<&[u8]>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_image_3d(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                kind,
                glow::PixelUnpackData::Slice(pixels),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_image_3d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        format: u32,
        kind: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_image_3d(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                format,
                kind,
                glow::PixelUnpackData::BufferOffset(offset),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_image_3d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let image_size =
            i32::try_from(pixels.len()).map_err(|_| "compressed image is too large")?;
        unsafe {
            context.gl.compressed_tex_image_3d(
                target,
                level,
                internal_format as i32,
                width,
                height,
                depth,
                border,
                image_size,
                pixels,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_image_3d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
        border: i32,
        image_size: i32,
        offset: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let upload = unsafe {
            resolve_egl::<GlCompressedTexImage3D>(
                backend.egl.get_proc_address,
                b"glCompressedTexImage3D\0",
            )
        }
        .ok_or("WebGL compressed 3D texture entry point is unavailable")?;
        unsafe {
            upload(
                target,
                level,
                internal_format,
                width,
                height,
                depth,
                border,
                image_size,
                offset as usize as *const c_void,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_sub_image_3d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        kind: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_sub_image_3d(
                target,
                level,
                x,
                y,
                z,
                width,
                height,
                depth,
                format,
                kind,
                glow::PixelUnpackData::Slice(Some(pixels)),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_sub_image_3d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        kind: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.tex_sub_image_3d(
                target,
                level,
                x,
                y,
                z,
                width,
                height,
                depth,
                format,
                kind,
                glow::PixelUnpackData::BufferOffset(offset),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_sub_image_3d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.compressed_tex_sub_image_3d(
                target,
                level,
                x,
                y,
                z,
                width,
                height,
                depth,
                format,
                glow::CompressedPixelUnpackData::Slice(pixels),
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compressed_texture_sub_image_3d_offset(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        format: u32,
        image_size: u32,
        offset: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let end = offset
            .checked_add(image_size)
            .ok_or("compressed texture buffer range overflow")?;
        unsafe {
            context.gl.compressed_tex_sub_image_3d(
                target,
                level,
                x,
                y,
                z,
                width,
                height,
                depth,
                format,
                glow::CompressedPixelUnpackData::BufferRange(offset..end),
            )
        };
        Ok(())
    }

    pub(crate) fn texture_storage_2d(
        &mut self,
        id: NodeId,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .tex_storage_2d(target, levels, internal_format, width, height)
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn texture_storage_3d(
        &mut self,
        id: NodeId,
        target: u32,
        levels: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        depth: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .tex_storage_3d(target, levels, internal_format, width, height, depth)
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_texture_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        internal_format: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.copy_tex_image_2d(
                target,
                level,
                internal_format,
                x,
                y,
                width,
                height,
                border,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_texture_sub_image_2d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .copy_tex_sub_image_2d(target, level, x_offset, y_offset, x, y, width, height)
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn copy_texture_sub_image_3d(
        &mut self,
        id: NodeId,
        target: u32,
        level: i32,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.copy_tex_sub_image_3d(
                target, level, x_offset, y_offset, z_offset, x, y, width, height,
            )
        };
        Ok(())
    }

    pub(crate) fn texture_parameter(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
        value: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.tex_parameter_i32(target, parameter, value) };
        Ok(())
    }

    pub(crate) fn texture_parameter_f32(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
        value: f32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.tex_parameter_f32(target, parameter, value) };
        Ok(())
    }

    pub(crate) fn texture_parameter_i32_value(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.get_tex_parameter_i32(target, parameter) })
    }

    pub(crate) fn texture_parameter_f32_value(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
    ) -> Result<f32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.get_tex_parameter_f32(target, parameter) })
    }

    pub(crate) fn pixel_store(
        &mut self,
        id: NodeId,
        parameter: u32,
        value: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.pixel_store_i32(parameter, value) };
        Ok(())
    }

    pub(crate) fn generate_mipmap(&mut self, id: NodeId, target: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.generate_mipmap(target) };
        Ok(())
    }

    pub(crate) fn delete_texture(&mut self, id: NodeId, texture: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(texture) = context.textures.remove(&texture) {
            unsafe { context.gl.delete_texture(texture) };
        }
        Ok(())
    }

    pub(crate) fn create_sampler(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = unsafe { context.gl.create_sampler() }?;
        let object = next_object(context)?;
        context.samplers.insert(object, sampler);
        Ok(object)
    }

    pub(crate) fn bind_sampler(
        &mut self,
        id: NodeId,
        unit: u32,
        sampler: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = sampler
            .map(|sampler| {
                context
                    .samplers
                    .get(&sampler)
                    .copied()
                    .ok_or("unknown WebGLSampler")
            })
            .transpose()?;
        unsafe { context.gl.bind_sampler(unit, sampler) };
        Ok(())
    }

    pub(crate) fn sampler_parameter_i32(
        &mut self,
        id: NodeId,
        sampler: u64,
        parameter: u32,
        value: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = context
            .samplers
            .get(&sampler)
            .copied()
            .ok_or("unknown WebGLSampler")?;
        unsafe { context.gl.sampler_parameter_i32(sampler, parameter, value) };
        Ok(())
    }

    pub(crate) fn sampler_parameter_f32(
        &mut self,
        id: NodeId,
        sampler: u64,
        parameter: u32,
        value: f32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = context
            .samplers
            .get(&sampler)
            .copied()
            .ok_or("unknown WebGLSampler")?;
        unsafe { context.gl.sampler_parameter_f32(sampler, parameter, value) };
        Ok(())
    }

    pub(crate) fn sampler_parameter_i32_value(
        &mut self,
        id: NodeId,
        sampler: u64,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = context
            .samplers
            .get(&sampler)
            .copied()
            .ok_or("unknown WebGLSampler")?;
        Ok(unsafe { context.gl.get_sampler_parameter_i32(sampler, parameter) })
    }

    pub(crate) fn sampler_parameter_f32_value(
        &mut self,
        id: NodeId,
        sampler: u64,
        parameter: u32,
    ) -> Result<f32, String> {
        let (_, context) = self.current_mut(id)?;
        let sampler = context
            .samplers
            .get(&sampler)
            .copied()
            .ok_or("unknown WebGLSampler")?;
        Ok(unsafe { context.gl.get_sampler_parameter_f32(sampler, parameter) })
    }

    pub(crate) fn delete_sampler(&mut self, id: NodeId, sampler: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(sampler) = context.samplers.remove(&sampler) {
            unsafe { context.gl.delete_sampler(sampler) };
        }
        Ok(())
    }

    pub(crate) fn create_query(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let query = unsafe { context.gl.create_query() }?;
        let object = next_object(context)?;
        context.queries.insert(object, query);
        Ok(object)
    }

    pub(crate) fn begin_query(
        &mut self,
        id: NodeId,
        target: u32,
        query: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let query = context
            .queries
            .get(&query)
            .copied()
            .ok_or("unknown WebGLQuery")?;
        unsafe { context.gl.begin_query(target, query) };
        Ok(())
    }

    pub(crate) fn end_query(&mut self, id: NodeId, target: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.end_query(target) };
        Ok(())
    }

    pub(crate) fn query_parameter(
        &mut self,
        id: NodeId,
        query: u64,
        parameter: u32,
    ) -> Result<u32, String> {
        let (_, context) = self.current_mut(id)?;
        let query = context
            .queries
            .get(&query)
            .copied()
            .ok_or("unknown WebGLQuery")?;
        Ok(unsafe { context.gl.get_query_parameter_u32(query, parameter) })
    }

    pub(crate) fn query_counter(
        &mut self,
        id: NodeId,
        query: u64,
        target: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let query = context
            .queries
            .get(&query)
            .copied()
            .ok_or("unknown WebGLQuery")?;
        unsafe { context.gl.query_counter(query, target) };
        Ok(())
    }

    pub(crate) fn query_parameter_u64(
        &mut self,
        id: NodeId,
        query: u64,
        parameter: u32,
    ) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let query = context
            .queries
            .get(&query)
            .copied()
            .ok_or("unknown WebGLQuery")?;
        Ok(unsafe { context.gl.get_query_parameter_u64(query, parameter) })
    }

    pub(crate) fn query_counter_bits(&mut self, id: NodeId, target: u32) -> Result<i32, String> {
        let (backend, _) = self.current_mut(id)?;
        let get_query = unsafe {
            resolve_egl::<GlGetQueryiv>(backend.egl.get_proc_address, b"glGetQueryiv\0").or_else(
                || resolve_egl::<GlGetQueryiv>(backend.egl.get_proc_address, b"glGetQueryivEXT\0"),
            )
        }
        .ok_or("ANGLE query reflection entry point is unavailable")?;
        let mut bits = 0;
        unsafe { get_query(target, glow::QUERY_COUNTER_BITS, &mut bits) };
        Ok(bits)
    }

    pub(crate) fn delete_query(&mut self, id: NodeId, query: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(query) = context.queries.remove(&query) {
            unsafe { context.gl.delete_query(query) };
        }
        Ok(())
    }

    pub(crate) fn fence_sync(
        &mut self,
        id: NodeId,
        condition: u32,
        flags: u32,
    ) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let sync = unsafe { context.gl.fence_sync(condition, flags) }?;
        let object = next_object(context)?;
        context.syncs.insert(object, sync);
        Ok(object)
    }

    pub(crate) fn client_wait_sync(
        &mut self,
        id: NodeId,
        sync: u64,
        flags: u32,
        timeout: i32,
    ) -> Result<u32, String> {
        let (_, context) = self.current_mut(id)?;
        let sync = context
            .syncs
            .get(&sync)
            .copied()
            .ok_or("unknown WebGLSync")?;
        Ok(unsafe { context.gl.client_wait_sync(sync, flags, timeout) })
    }

    pub(crate) fn wait_sync(
        &mut self,
        id: NodeId,
        sync: u64,
        flags: u32,
        timeout: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let sync = context
            .syncs
            .get(&sync)
            .copied()
            .ok_or("unknown WebGLSync")?;
        unsafe { context.gl.wait_sync(sync, flags, timeout) };
        Ok(())
    }

    pub(crate) fn sync_parameter(
        &mut self,
        id: NodeId,
        sync: u64,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let sync = context
            .syncs
            .get(&sync)
            .copied()
            .ok_or("unknown WebGLSync")?;
        Ok(unsafe { context.gl.get_sync_parameter_i32(sync, parameter) })
    }

    pub(crate) fn delete_sync(&mut self, id: NodeId, sync: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(sync) = context.syncs.remove(&sync) {
            unsafe { context.gl.delete_sync(sync) };
        }
        Ok(())
    }

    pub(crate) fn create_transform_feedback(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let feedback = unsafe { context.gl.create_transform_feedback() }?;
        let object = next_object(context)?;
        context.transform_feedbacks.insert(object, feedback);
        Ok(object)
    }

    pub(crate) fn bind_transform_feedback(
        &mut self,
        id: NodeId,
        target: u32,
        feedback: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let feedback = feedback
            .map(|feedback| {
                context
                    .transform_feedbacks
                    .get(&feedback)
                    .copied()
                    .ok_or("unknown WebGLTransformFeedback")
            })
            .transpose()?;
        unsafe { context.gl.bind_transform_feedback(target, feedback) };
        Ok(())
    }

    pub(crate) fn begin_transform_feedback(
        &mut self,
        id: NodeId,
        primitive_mode: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.begin_transform_feedback(primitive_mode) };
        Ok(())
    }

    pub(crate) fn end_transform_feedback(&mut self, id: NodeId) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.end_transform_feedback() };
        Ok(())
    }

    pub(crate) fn pause_transform_feedback(&mut self, id: NodeId) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.pause_transform_feedback() };
        Ok(())
    }

    pub(crate) fn resume_transform_feedback(&mut self, id: NodeId) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.resume_transform_feedback() };
        Ok(())
    }

    pub(crate) fn delete_transform_feedback(
        &mut self,
        id: NodeId,
        feedback: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(feedback) = context.transform_feedbacks.remove(&feedback) {
            unsafe { context.gl.delete_transform_feedback(feedback) };
        }
        Ok(())
    }

    pub(crate) fn create_framebuffer(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let framebuffer = unsafe { context.gl.create_framebuffer() }?;
        let object = next_object(context)?;
        context.framebuffers.insert(object, framebuffer);
        Ok(object)
    }

    pub(crate) fn bind_framebuffer(
        &mut self,
        id: NodeId,
        target: u32,
        framebuffer: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let object = framebuffer;
        let framebuffer = object
            .map(|framebuffer| {
                context
                    .framebuffers
                    .get(&framebuffer)
                    .copied()
                    .ok_or("unknown WebGLFramebuffer")
            })
            .transpose()?;
        unsafe { context.gl.bind_framebuffer(target, framebuffer) };
        match target {
            glow::FRAMEBUFFER => {
                context.bound_framebuffer = object;
                context.bound_read_framebuffer = object;
            }
            glow::DRAW_FRAMEBUFFER => context.bound_framebuffer = object,
            glow::READ_FRAMEBUFFER => context.bound_read_framebuffer = object,
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn framebuffer_texture_layer(
        &mut self,
        id: NodeId,
        target: u32,
        attachment: u32,
        texture: Option<u64>,
        level: i32,
        layer: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let texture = texture
            .map(|texture| {
                context
                    .textures
                    .get(&texture)
                    .copied()
                    .ok_or("unknown WebGLTexture")
            })
            .transpose()?;
        unsafe {
            context
                .gl
                .framebuffer_texture_layer(target, attachment, texture, level, layer)
        };
        Ok(())
    }

    pub(crate) fn draw_buffers(&mut self, id: NodeId, buffers: &[u32]) -> Result<(), String> {
        let (backend, context) = self.current_mut(id)?;
        if context.version == 1 {
            let draw_buffers = unsafe {
                resolve_egl::<GlDrawBuffersExt>(backend.egl.get_proc_address, b"glDrawBuffersEXT\0")
            }
            .ok_or("WEBGL_draw_buffers entry point is unavailable")?;
            let count = i32::try_from(buffers.len()).map_err(|_| "too many WebGL draw buffers")?;
            unsafe { draw_buffers(count, buffers.as_ptr()) };
        } else {
            unsafe { context.gl.draw_buffers(buffers) };
        }
        Ok(())
    }

    pub(crate) fn read_buffer(&mut self, id: NodeId, buffer: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.read_buffer(buffer) };
        Ok(())
    }

    pub(crate) fn invalidate_framebuffer(
        &mut self,
        id: NodeId,
        target: u32,
        attachments: &[u32],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.invalidate_framebuffer(target, attachments) };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn invalidate_sub_framebuffer(
        &mut self,
        id: NodeId,
        target: u32,
        attachments: &[u32],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .invalidate_sub_framebuffer(target, attachments, x, y, width, height)
        };
        Ok(())
    }

    pub(crate) fn clear_buffer_i32(
        &mut self,
        id: NodeId,
        buffer: u32,
        draw_buffer: u32,
        values: &[i32],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .clear_buffer_i32_slice(buffer, draw_buffer, values)
        };
        Ok(())
    }

    pub(crate) fn clear_buffer_u32(
        &mut self,
        id: NodeId,
        buffer: u32,
        draw_buffer: u32,
        values: &[u32],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .clear_buffer_u32_slice(buffer, draw_buffer, values)
        };
        Ok(())
    }

    pub(crate) fn clear_buffer_f32(
        &mut self,
        id: NodeId,
        buffer: u32,
        draw_buffer: u32,
        values: &[f32],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .clear_buffer_f32_slice(buffer, draw_buffer, values)
        };
        Ok(())
    }

    pub(crate) fn clear_buffer_depth_stencil(
        &mut self,
        id: NodeId,
        buffer: u32,
        draw_buffer: u32,
        depth: f32,
        stencil: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .clear_buffer_depth_stencil(buffer, draw_buffer, depth, stencil)
        };
        Ok(())
    }

    pub(crate) fn internal_format_samples(
        &mut self,
        id: NodeId,
        target: u32,
        internal_format: u32,
    ) -> Result<Vec<i32>, String> {
        let (_, context) = self.current_mut(id)?;
        let mut count = [0];
        unsafe {
            context.gl.get_internal_format_i32_slice(
                target,
                internal_format,
                glow::NUM_SAMPLE_COUNTS,
                &mut count,
            )
        };
        if count[0] < 0 {
            return Err("ANGLE returned an invalid sample-count length".to_owned());
        }
        let mut samples = vec![0; count[0] as usize];
        if !samples.is_empty() {
            unsafe {
                context.gl.get_internal_format_i32_slice(
                    target,
                    internal_format,
                    glow::SAMPLES,
                    &mut samples,
                )
            };
        }
        Ok(samples)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn blit_framebuffer(
        &mut self,
        id: NodeId,
        source: [i32; 4],
        destination: [i32; 4],
        mask: u32,
        filter: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.blit_framebuffer(
                source[0],
                source[1],
                source[2],
                source[3],
                destination[0],
                destination[1],
                destination[2],
                destination[3],
                mask,
                filter,
            )
        };
        Ok(())
    }

    pub(crate) fn framebuffer_texture_2d(
        &mut self,
        id: NodeId,
        target: u32,
        attachment: u32,
        texture_target: u32,
        texture: Option<u64>,
        level: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let texture = texture
            .map(|texture| {
                context
                    .textures
                    .get(&texture)
                    .copied()
                    .ok_or("unknown WebGLTexture")
            })
            .transpose()?;
        unsafe {
            context
                .gl
                .framebuffer_texture_2d(target, attachment, texture_target, texture, level)
        };
        Ok(())
    }

    pub(crate) fn framebuffer_status(&mut self, id: NodeId, target: u32) -> Result<u32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.check_framebuffer_status(target) })
    }

    pub(crate) fn delete_framebuffer(
        &mut self,
        id: NodeId,
        framebuffer: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let object = framebuffer;
        if let Some(framebuffer) = context.framebuffers.remove(&object) {
            unsafe { context.gl.delete_framebuffer(framebuffer) };
        }
        if context.bound_framebuffer == Some(object) {
            context.bound_framebuffer = None;
        }
        if context.bound_read_framebuffer == Some(object) {
            context.bound_read_framebuffer = None;
        }
        Ok(())
    }

    pub(crate) fn create_renderbuffer(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let renderbuffer = unsafe { context.gl.create_renderbuffer() }?;
        let object = next_object(context)?;
        context.renderbuffers.insert(object, renderbuffer);
        Ok(object)
    }

    pub(crate) fn bind_renderbuffer(
        &mut self,
        id: NodeId,
        target: u32,
        renderbuffer: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let renderbuffer = renderbuffer
            .map(|renderbuffer| {
                context
                    .renderbuffers
                    .get(&renderbuffer)
                    .copied()
                    .ok_or("unknown WebGLRenderbuffer")
            })
            .transpose()?;
        unsafe { context.gl.bind_renderbuffer(target, renderbuffer) };
        Ok(())
    }

    pub(crate) fn renderbuffer_storage(
        &mut self,
        id: NodeId,
        target: u32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .renderbuffer_storage(target, internal_format, width, height)
        };
        Ok(())
    }

    pub(crate) fn renderbuffer_parameter_i32(
        &mut self,
        id: NodeId,
        target: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe { context.gl.get_renderbuffer_parameter_i32(target, parameter) })
    }

    pub(crate) fn framebuffer_attachment_parameter_i32(
        &mut self,
        id: NodeId,
        target: u32,
        attachment: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        Ok(unsafe {
            context
                .gl
                .get_framebuffer_attachment_parameter_i32(target, attachment, parameter)
        })
    }

    pub(crate) fn framebuffer_attachment_object(
        &mut self,
        id: NodeId,
        target: u32,
        attachment: u32,
    ) -> Result<(u32, Option<u64>), String> {
        let (_, context) = self.current_mut(id)?;
        let kind = unsafe {
            context.gl.get_framebuffer_attachment_parameter_i32(
                target,
                attachment,
                glow::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE,
            )
        } as u32;
        let name = unsafe {
            context.gl.get_framebuffer_attachment_parameter_i32(
                target,
                attachment,
                glow::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME,
            )
        } as u32;
        let object = match kind {
            glow::TEXTURE => context
                .textures
                .iter()
                .find_map(|(object, texture)| (texture.0.get() == name).then_some(*object)),
            glow::RENDERBUFFER => {
                context
                    .renderbuffers
                    .iter()
                    .find_map(|(object, renderbuffer)| {
                        (renderbuffer.0.get() == name).then_some(*object)
                    })
            }
            _ => None,
        };
        Ok((kind, object))
    }

    pub(crate) fn renderbuffer_storage_multisample(
        &mut self,
        id: NodeId,
        target: u32,
        samples: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context.gl.renderbuffer_storage_multisample(
                target,
                samples,
                internal_format,
                width,
                height,
            )
        };
        Ok(())
    }

    pub(crate) fn framebuffer_renderbuffer(
        &mut self,
        id: NodeId,
        target: u32,
        attachment: u32,
        renderbuffer_target: u32,
        renderbuffer: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let renderbuffer = renderbuffer
            .map(|renderbuffer| {
                context
                    .renderbuffers
                    .get(&renderbuffer)
                    .copied()
                    .ok_or("unknown WebGLRenderbuffer")
            })
            .transpose()?;
        unsafe {
            context.gl.framebuffer_renderbuffer(
                target,
                attachment,
                renderbuffer_target,
                renderbuffer,
            )
        };
        Ok(())
    }

    pub(crate) fn delete_renderbuffer(
        &mut self,
        id: NodeId,
        renderbuffer: u64,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(renderbuffer) = context.renderbuffers.remove(&renderbuffer) {
            unsafe { context.gl.delete_renderbuffer(renderbuffer) };
        }
        Ok(())
    }

    pub(crate) fn create_vertex_array(&mut self, id: NodeId) -> Result<u64, String> {
        let (_, context) = self.current_mut(id)?;
        let array = unsafe { context.gl.create_vertex_array() }?;
        let object = next_object(context)?;
        context.vertex_arrays.insert(object, array);
        Ok(object)
    }

    pub(crate) fn bind_vertex_array(
        &mut self,
        id: NodeId,
        array: Option<u64>,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let array = array
            .map(|array| {
                context
                    .vertex_arrays
                    .get(&array)
                    .copied()
                    .ok_or("unknown WebGLVertexArrayObject")
            })
            .transpose()?;
        unsafe { context.gl.bind_vertex_array(array) };
        Ok(())
    }

    pub(crate) fn delete_vertex_array(&mut self, id: NodeId, array: u64) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        if let Some(array) = context.vertex_arrays.remove(&array) {
            unsafe { context.gl.delete_vertex_array(array) };
        }
        Ok(())
    }

    pub(crate) fn attribute_location(
        &mut self,
        id: NodeId,
        program: u64,
        name: &str,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_attrib_location(program, name) }
            .map(|location| location as i32)
            .unwrap_or(-1))
    }

    pub(crate) fn enable_attribute(&mut self, id: NodeId, index: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.enable_vertex_attrib_array(index) };
        Ok(())
    }

    pub(crate) fn disable_attribute(&mut self, id: NodeId, index: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.disable_vertex_attrib_array(index) };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attribute_pointer(
        &mut self,
        id: NodeId,
        index: u32,
        size: i32,
        kind: u32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .vertex_attrib_pointer_f32(index, size, kind, normalized, stride, offset)
        };
        Ok(())
    }

    pub(crate) fn integer_attribute_pointer(
        &mut self,
        id: NodeId,
        index: u32,
        size: i32,
        kind: u32,
        stride: i32,
        offset: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .vertex_attrib_pointer_i32(index, size, kind, stride, offset)
        };
        Ok(())
    }

    pub(crate) fn integer_attribute_i32(
        &mut self,
        id: NodeId,
        index: u32,
        values: [i32; 4],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .vertex_attrib_4_i32(index, values[0], values[1], values[2], values[3])
        };
        Ok(())
    }

    pub(crate) fn integer_attribute_u32(
        &mut self,
        id: NodeId,
        index: u32,
        values: [u32; 4],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .vertex_attrib_4_u32(index, values[0], values[1], values[2], values[3])
        };
        Ok(())
    }

    pub(crate) fn attribute_f32(
        &mut self,
        id: NodeId,
        index: u32,
        values: [f32; 4],
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe {
            context
                .gl
                .vertex_attrib_4_f32(index, values[0], values[1], values[2], values[3])
        };
        Ok(())
    }

    pub(crate) fn attribute_f32_value(
        &mut self,
        id: NodeId,
        index: u32,
    ) -> Result<[f32; 4], String> {
        let (_, context) = self.current_mut(id)?;
        let mut values = [0.0; 4];
        unsafe {
            context.gl.get_vertex_attrib_parameter_f32_slice(
                index,
                glow::CURRENT_VERTEX_ATTRIB,
                &mut values,
            )
        };
        Ok(values)
    }

    pub(crate) fn attribute_parameter_i32(
        &mut self,
        id: NodeId,
        index: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (backend, _) = self.current_mut(id)?;
        let get = unsafe {
            resolve_egl::<GlGetVertexAttribiv>(
                backend.egl.get_proc_address,
                b"glGetVertexAttribiv\0",
            )
        }
        .ok_or("WebGL vertex-attribute query entry point is unavailable")?;
        let mut value = 0;
        unsafe { get(index, parameter, &mut value) };
        Ok(value)
    }

    pub(crate) fn attribute_buffer(
        &mut self,
        id: NodeId,
        index: u32,
    ) -> Result<Option<u64>, String> {
        let value =
            self.attribute_parameter_i32(id, index, glow::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING)?
                as u32;
        let context = self.contexts.get(&id).ok_or("unknown WebGL context")?;
        Ok(context
            .buffers
            .iter()
            .find_map(|(object, buffer)| (buffer.0.get() == value).then_some(*object)))
    }

    pub(crate) fn attribute_offset(
        &mut self,
        id: NodeId,
        index: u32,
        parameter: u32,
    ) -> Result<usize, String> {
        let (backend, _) = self.current_mut(id)?;
        let get = unsafe {
            resolve_egl::<GlGetVertexAttribPointerv>(
                backend.egl.get_proc_address,
                b"glGetVertexAttribPointerv\0",
            )
        }
        .ok_or("WebGL vertex-attribute pointer query entry point is unavailable")?;
        let mut value = std::ptr::null_mut();
        unsafe { get(index, parameter, &mut value) };
        Ok(value as usize)
    }

    pub(crate) fn uniform_location(
        &mut self,
        id: NodeId,
        program: u64,
        name: &str,
    ) -> Result<Option<u64>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let Some(location) = (unsafe { context.gl.get_uniform_location(program, name) }) else {
            return Ok(None);
        };
        let object = next_object(context)?;
        context.uniforms.insert(object, location);
        Ok(Some(object))
    }

    pub(crate) fn uniform_indices(
        &mut self,
        id: NodeId,
        program: u64,
        names: &[String],
    ) -> Result<Vec<u32>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let names = names.iter().map(String::as_str).collect::<Vec<_>>();
        Ok(unsafe { context.gl.get_uniform_indices(program, &names) }
            .into_iter()
            .map(|index| index.unwrap_or(u32::MAX))
            .collect())
    }

    pub(crate) fn uniform_value(
        &mut self,
        id: NodeId,
        program: u64,
        location: u64,
        kind: u32,
    ) -> Result<UniformValue, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        let (category, length) =
            uniform_category_and_length(kind).ok_or("unsupported WebGL uniform type")?;
        match category {
            0 => {
                let mut values = vec![0.0; length];
                unsafe { context.gl.get_uniform_f32(program, location, &mut values) };
                Ok(UniformValue::Float(values))
            }
            1 => {
                let mut values = vec![0; length];
                unsafe { context.gl.get_uniform_i32(program, location, &mut values) };
                Ok(UniformValue::Int(values))
            }
            2 => {
                let mut values = vec![0; length];
                unsafe { context.gl.get_uniform_u32(program, location, &mut values) };
                Ok(UniformValue::Uint(values))
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn active_uniform_parameters(
        &mut self,
        id: NodeId,
        program: u64,
        indices: &[u32],
        parameter: u32,
    ) -> Result<Vec<i32>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe {
            context
                .gl
                .get_active_uniforms_parameter(program, indices, parameter)
        })
    }

    pub(crate) fn uniform_block_index(
        &mut self,
        id: NodeId,
        program: u64,
        name: &str,
    ) -> Result<u32, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_uniform_block_index(program, name) }.unwrap_or(u32::MAX))
    }

    pub(crate) fn uniform_block_parameter_i32(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
        parameter: u32,
    ) -> Result<i32, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe {
            context
                .gl
                .get_active_uniform_block_parameter_i32(program, index, parameter)
        })
    }

    pub(crate) fn uniform_block_active_indices(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
    ) -> Result<Vec<u32>, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        let count = unsafe {
            context.gl.get_active_uniform_block_parameter_i32(
                program,
                index,
                glow::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
            )
        };
        if count < 0 {
            return Err("ANGLE returned an invalid active-uniform count".to_owned());
        }
        let mut values = vec![0; count as usize];
        if !values.is_empty() {
            unsafe {
                context.gl.get_active_uniform_block_parameter_i32_slice(
                    program,
                    index,
                    glow::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
                    &mut values,
                )
            };
        }
        Ok(values.into_iter().map(|value| value as u32).collect())
    }

    pub(crate) fn uniform_block_name(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
    ) -> Result<String, String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        Ok(unsafe { context.gl.get_active_uniform_block_name(program, index) })
    }

    pub(crate) fn uniform_block_binding(
        &mut self,
        id: NodeId,
        program: u64,
        index: u32,
        binding: u32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        let program = *context
            .programs
            .get(&program)
            .ok_or("unknown WebGLProgram")?;
        unsafe { context.gl.uniform_block_binding(program, index, binding) };
        Ok(())
    }

    pub(crate) fn uniform_f32(
        &mut self,
        id: NodeId,
        location: u64,
        components: u32,
        values: &[f32],
    ) -> Result<(), String> {
        if !(1..=4).contains(&components)
            || values.is_empty()
            || !values.len().is_multiple_of(components as usize)
        {
            return Err("invalid floating-point uniform data".to_owned());
        }
        let (_, context) = self.current_mut(id)?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        unsafe {
            match components {
                1 => context.gl.uniform_1_f32_slice(Some(location), values),
                2 => context.gl.uniform_2_f32_slice(Some(location), values),
                3 => context.gl.uniform_3_f32_slice(Some(location), values),
                4 => context.gl.uniform_4_f32_slice(Some(location), values),
                _ => return Err("uniform component count must be between 1 and 4".to_owned()),
            }
        }
        Ok(())
    }

    pub(crate) fn uniform_i32(
        &mut self,
        id: NodeId,
        location: u64,
        components: u32,
        values: &[i32],
    ) -> Result<(), String> {
        if !(1..=4).contains(&components)
            || values.is_empty()
            || !values.len().is_multiple_of(components as usize)
        {
            return Err("invalid integer uniform data".to_owned());
        }
        let (_, context) = self.current_mut(id)?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        unsafe {
            match components {
                1 => context.gl.uniform_1_i32_slice(Some(location), values),
                2 => context.gl.uniform_2_i32_slice(Some(location), values),
                3 => context.gl.uniform_3_i32_slice(Some(location), values),
                4 => context.gl.uniform_4_i32_slice(Some(location), values),
                _ => return Err("uniform component count must be between 1 and 4".to_owned()),
            }
        }
        Ok(())
    }

    pub(crate) fn uniform_u32(
        &mut self,
        id: NodeId,
        location: u64,
        components: u32,
        values: &[u32],
    ) -> Result<(), String> {
        if !(1..=4).contains(&components)
            || values.is_empty()
            || !values.len().is_multiple_of(components as usize)
        {
            return Err("invalid unsigned-integer uniform data".to_owned());
        }
        let (_, context) = self.current_mut(id)?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        unsafe {
            match components {
                1 => context.gl.uniform_1_u32_slice(Some(location), values),
                2 => context.gl.uniform_2_u32_slice(Some(location), values),
                3 => context.gl.uniform_3_u32_slice(Some(location), values),
                4 => context.gl.uniform_4_u32_slice(Some(location), values),
                _ => return Err("uniform component count must be between 1 and 4".to_owned()),
            }
        }
        Ok(())
    }

    pub(crate) fn uniform_matrix_f32(
        &mut self,
        id: NodeId,
        location: u64,
        dimension: u32,
        transpose: bool,
        values: &[f32],
    ) -> Result<(), String> {
        let matrix_size = dimension as usize * dimension as usize;
        if !(2..=4).contains(&dimension)
            || values.is_empty()
            || !values.len().is_multiple_of(matrix_size)
        {
            return Err("invalid matrix uniform data".to_owned());
        }
        let (_, context) = self.current_mut(id)?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        unsafe {
            match dimension {
                2 => context
                    .gl
                    .uniform_matrix_2_f32_slice(Some(location), transpose, values),
                3 => context
                    .gl
                    .uniform_matrix_3_f32_slice(Some(location), transpose, values),
                4 => context
                    .gl
                    .uniform_matrix_4_f32_slice(Some(location), transpose, values),
                _ => return Err("uniform matrix dimension must be between 2 and 4".to_owned()),
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn uniform_matrix_rect_f32(
        &mut self,
        id: NodeId,
        location: u64,
        columns: u32,
        rows: u32,
        transpose: bool,
        values: &[f32],
    ) -> Result<(), String> {
        let matrix_size = columns as usize * rows as usize;
        if !(2..=4).contains(&columns)
            || !(2..=4).contains(&rows)
            || columns == rows
            || values.is_empty()
            || !values.len().is_multiple_of(matrix_size)
        {
            return Err("invalid non-square matrix uniform data".to_owned());
        }
        let (_, context) = self.current_mut(id)?;
        let location = context
            .uniforms
            .get(&location)
            .ok_or("unknown WebGLUniformLocation")?;
        unsafe {
            match (columns, rows) {
                (2, 3) => {
                    context
                        .gl
                        .uniform_matrix_2x3_f32_slice(Some(location), transpose, values)
                }
                (3, 2) => {
                    context
                        .gl
                        .uniform_matrix_3x2_f32_slice(Some(location), transpose, values)
                }
                (2, 4) => {
                    context
                        .gl
                        .uniform_matrix_2x4_f32_slice(Some(location), transpose, values)
                }
                (4, 2) => {
                    context
                        .gl
                        .uniform_matrix_4x2_f32_slice(Some(location), transpose, values)
                }
                (3, 4) => {
                    context
                        .gl
                        .uniform_matrix_3x4_f32_slice(Some(location), transpose, values)
                }
                (4, 3) => {
                    context
                        .gl
                        .uniform_matrix_4x3_f32_slice(Some(location), transpose, values)
                }
                _ => return Err("unsupported non-square uniform matrix dimensions".to_owned()),
            }
        }
        Ok(())
    }

    pub(crate) fn viewport(
        &mut self,
        id: NodeId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.viewport(x, y, width, height) };
        Ok(())
    }

    pub(crate) fn hint(&mut self, id: NodeId, target: u32, mode: u32) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.hint(target, mode) };
        Ok(())
    }

    pub(crate) fn draw_arrays(
        &mut self,
        id: NodeId,
        mode: u32,
        first: i32,
        count: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.draw_arrays(mode, first, count) };
        Ok(())
    }

    pub(crate) fn draw_elements(
        &mut self,
        id: NodeId,
        mode: u32,
        count: i32,
        kind: u32,
        offset: i32,
    ) -> Result<(), String> {
        let (_, context) = self.current_mut(id)?;
        unsafe { context.gl.draw_elements(mode, count, kind, offset) };
        Ok(())
    }

    pub(crate) fn multi_draw_arrays(
        &mut self,
        id: NodeId,
        mode: u32,
        firsts: &[i32],
        counts: &[i32],
    ) -> Result<(), String> {
        if firsts.len() != counts.len() {
            return Err("multi-draw array lengths do not match".to_owned());
        }
        let draw_count = i32::try_from(firsts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawArraysAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawArraysANGLE\0",
            )
        }
        .ok_or("ANGLE multi-draw-arrays entry point is unavailable")?;
        unsafe { draw(mode, firsts.as_ptr(), counts.as_ptr(), draw_count) };
        Ok(())
    }

    pub(crate) fn multi_draw_elements(
        &mut self,
        id: NodeId,
        mode: u32,
        counts: &[i32],
        kind: u32,
        offsets: &[i32],
    ) -> Result<(), String> {
        if counts.len() != offsets.len() {
            return Err("multi-draw element lengths do not match".to_owned());
        }
        let offsets = element_offset_pointers(offsets)?;
        let draw_count = i32::try_from(counts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawElementsAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawElementsANGLE\0",
            )
        }
        .ok_or("ANGLE multi-draw-elements entry point is unavailable")?;
        unsafe { draw(mode, counts.as_ptr(), kind, offsets.as_ptr(), draw_count) };
        Ok(())
    }

    pub(crate) fn multi_draw_arrays_instanced(
        &mut self,
        id: NodeId,
        mode: u32,
        firsts: &[i32],
        counts: &[i32],
        instances: &[i32],
    ) -> Result<(), String> {
        if firsts.len() != counts.len() || firsts.len() != instances.len() {
            return Err("instanced multi-draw array lengths do not match".to_owned());
        }
        let draw_count = i32::try_from(firsts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawArraysInstancedAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawArraysInstancedANGLE\0",
            )
        }
        .ok_or("ANGLE instanced multi-draw-arrays entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                firsts.as_ptr(),
                counts.as_ptr(),
                instances.as_ptr(),
                draw_count,
            )
        };
        Ok(())
    }

    pub(crate) fn multi_draw_elements_instanced(
        &mut self,
        id: NodeId,
        mode: u32,
        counts: &[i32],
        kind: u32,
        offsets: &[i32],
        instances: &[i32],
    ) -> Result<(), String> {
        if counts.len() != offsets.len() || counts.len() != instances.len() {
            return Err("instanced multi-draw element lengths do not match".to_owned());
        }
        let offsets = element_offset_pointers(offsets)?;
        let draw_count = i32::try_from(counts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawElementsInstancedAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawElementsInstancedANGLE\0",
            )
        }
        .ok_or("ANGLE instanced multi-draw-elements entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                counts.as_ptr(),
                kind,
                offsets.as_ptr(),
                instances.as_ptr(),
                draw_count,
            )
        };
        Ok(())
    }

    pub(crate) fn multi_draw_arrays_instanced_base_instance(
        &mut self,
        id: NodeId,
        mode: u32,
        firsts: &[i32],
        counts: &[i32],
        instances: &[i32],
        base_instances: &[u32],
    ) -> Result<(), String> {
        if firsts.len() != counts.len()
            || firsts.len() != instances.len()
            || firsts.len() != base_instances.len()
        {
            return Err("base-instance multi-draw array lengths do not match".to_owned());
        }
        let draw_count = i32::try_from(firsts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawArraysInstancedBaseInstanceAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawArraysInstancedBaseInstanceANGLE\0",
            )
        }
        .ok_or("ANGLE base-instance multi-draw-arrays entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                firsts.as_ptr(),
                counts.as_ptr(),
                instances.as_ptr(),
                base_instances.as_ptr(),
                draw_count,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn multi_draw_elements_instanced_base_vertex_base_instance(
        &mut self,
        id: NodeId,
        mode: u32,
        counts: &[i32],
        kind: u32,
        offsets: &[i32],
        instances: &[i32],
        base_vertices: &[i32],
        base_instances: &[u32],
    ) -> Result<(), String> {
        if counts.len() != offsets.len()
            || counts.len() != instances.len()
            || counts.len() != base_vertices.len()
            || counts.len() != base_instances.len()
        {
            return Err(
                "base-vertex/base-instance multi-draw element lengths do not match".to_owned(),
            );
        }
        let offsets = element_offset_pointers(offsets)?;
        let draw_count = i32::try_from(counts.len()).map_err(|_| "too many multi-draw calls")?;
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlMultiDrawElementsInstancedBaseVertexBaseInstanceAngle>(
                backend.egl.get_proc_address,
                b"glMultiDrawElementsInstancedBaseVertexBaseInstanceANGLE\0",
            )
        }
        .ok_or("ANGLE base-vertex/base-instance multi-draw-elements entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                counts.as_ptr(),
                kind,
                offsets.as_ptr(),
                instances.as_ptr(),
                base_vertices.as_ptr(),
                base_instances.as_ptr(),
                draw_count,
            )
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_range_elements(
        &mut self,
        id: NodeId,
        mode: u32,
        start: u32,
        end: u32,
        count: i32,
        kind: u32,
        offset: i32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlDrawRangeElements>(
                backend.egl.get_proc_address,
                b"glDrawRangeElements\0",
            )
        }
        .ok_or("WebGL 2 draw-range entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                start,
                end,
                count,
                kind,
                offset as usize as *const c_void,
            )
        };
        Ok(())
    }

    pub(crate) fn draw_arrays_instanced(
        &mut self,
        id: NodeId,
        mode: u32,
        first: i32,
        count: i32,
        instances: i32,
    ) -> Result<(), String> {
        let (backend, context) = self.current_mut(id)?;
        if context.version == 1 {
            let draw = unsafe {
                resolve_egl::<GlDrawArraysInstancedAngle>(
                    backend.egl.get_proc_address,
                    b"glDrawArraysInstancedANGLE\0",
                )
            }
            .ok_or("ANGLE_instanced_arrays draw-arrays entry point is unavailable")?;
            unsafe { draw(mode, first, count, instances) };
        } else {
            unsafe {
                context
                    .gl
                    .draw_arrays_instanced(mode, first, count, instances)
            };
        }
        Ok(())
    }

    pub(crate) fn draw_elements_instanced(
        &mut self,
        id: NodeId,
        mode: u32,
        count: i32,
        kind: u32,
        offset: i32,
        instances: i32,
    ) -> Result<(), String> {
        let (backend, context) = self.current_mut(id)?;
        if context.version == 1 {
            let draw = unsafe {
                resolve_egl::<GlDrawElementsInstancedAngle>(
                    backend.egl.get_proc_address,
                    b"glDrawElementsInstancedANGLE\0",
                )
            }
            .ok_or("ANGLE_instanced_arrays draw-elements entry point is unavailable")?;
            unsafe {
                draw(
                    mode,
                    count,
                    kind,
                    offset as usize as *const c_void,
                    instances,
                )
            };
        } else {
            unsafe {
                context
                    .gl
                    .draw_elements_instanced(mode, count, kind, offset, instances)
            };
        }
        Ok(())
    }

    pub(crate) fn draw_arrays_instanced_base_instance(
        &mut self,
        id: NodeId,
        mode: u32,
        first: i32,
        count: i32,
        instances: i32,
        base_instance: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlDrawArraysInstancedBaseInstanceAngle>(
                backend.egl.get_proc_address,
                b"glDrawArraysInstancedBaseInstanceANGLE\0",
            )
        }
        .ok_or("ANGLE base-instance draw-arrays entry point is unavailable")?;
        unsafe { draw(mode, first, count, instances, base_instance) };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_elements_instanced_base_vertex_base_instance(
        &mut self,
        id: NodeId,
        mode: u32,
        count: i32,
        kind: u32,
        offset: i32,
        instances: i32,
        base_vertex: i32,
        base_instance: u32,
    ) -> Result<(), String> {
        let (backend, _) = self.current_mut(id)?;
        let draw = unsafe {
            resolve_egl::<GlDrawElementsInstancedBaseVertexBaseInstanceAngle>(
                backend.egl.get_proc_address,
                b"glDrawElementsInstancedBaseVertexBaseInstanceANGLE\0",
            )
        }
        .ok_or("ANGLE base-vertex/base-instance draw-elements entry point is unavailable")?;
        unsafe {
            draw(
                mode,
                count,
                kind,
                offset as usize as *const c_void,
                instances,
                base_vertex,
                base_instance,
            )
        };
        Ok(())
    }

    pub(crate) fn vertex_attrib_divisor(
        &mut self,
        id: NodeId,
        index: u32,
        divisor: u32,
    ) -> Result<(), String> {
        let (backend, context) = self.current_mut(id)?;
        if context.version == 1 {
            let set_divisor = unsafe {
                resolve_egl::<GlVertexAttribDivisorAngle>(
                    backend.egl.get_proc_address,
                    b"glVertexAttribDivisorANGLE\0",
                )
            }
            .ok_or("ANGLE_instanced_arrays divisor entry point is unavailable")?;
            unsafe { set_divisor(index, divisor) };
        } else {
            unsafe { context.gl.vertex_attrib_divisor(index, divisor) };
        }
        Ok(())
    }

    fn current_mut(&mut self, id: NodeId) -> Result<(&Backend, &mut WebGlContext), String> {
        let backend = self.backend.as_ref().ok_or("ANGLE is unavailable")?;
        let context = self.contexts.get_mut(&id).ok_or("unknown WebGL context")?;
        if unsafe {
            (backend.egl.make_current)(
                backend.display,
                context.surface,
                context.surface,
                context.egl_context,
            )
        } == egl::FALSE
        {
            return Err(egl_error(backend, "eglMakeCurrent"));
        }
        Ok((backend, context))
    }

    fn current(&self, id: NodeId) -> Result<(&Backend, &WebGlContext), String> {
        let backend = self.backend.as_ref().ok_or("ANGLE is unavailable")?;
        let context = self.contexts.get(&id).ok_or("unknown WebGL context")?;
        if unsafe {
            (backend.egl.make_current)(
                backend.display,
                context.surface,
                context.surface,
                context.egl_context,
            )
        } == egl::FALSE
        {
            return Err(egl_error(backend, "eglMakeCurrent"));
        }
        Ok((backend, context))
    }
}

fn uniform_category_and_length(kind: u32) -> Option<(u8, usize)> {
    Some(match kind {
        glow::FLOAT => (0, 1),
        glow::FLOAT_VEC2 => (0, 2),
        glow::FLOAT_VEC3 => (0, 3),
        glow::FLOAT_VEC4 => (0, 4),
        glow::FLOAT_MAT2 => (0, 4),
        glow::FLOAT_MAT3 => (0, 9),
        glow::FLOAT_MAT4 => (0, 16),
        glow::FLOAT_MAT2x3 => (0, 6),
        glow::FLOAT_MAT2x4 => (0, 8),
        glow::FLOAT_MAT3x2 => (0, 6),
        glow::FLOAT_MAT3x4 => (0, 12),
        glow::FLOAT_MAT4x2 => (0, 8),
        glow::FLOAT_MAT4x3 => (0, 12),
        glow::INT | glow::BOOL => (1, 1),
        glow::INT_VEC2 | glow::BOOL_VEC2 => (1, 2),
        glow::INT_VEC3 | glow::BOOL_VEC3 => (1, 3),
        glow::INT_VEC4 | glow::BOOL_VEC4 => (1, 4),
        glow::SAMPLER_2D
        | glow::SAMPLER_3D
        | glow::SAMPLER_CUBE
        | glow::SAMPLER_2D_SHADOW
        | glow::SAMPLER_2D_ARRAY
        | glow::SAMPLER_2D_ARRAY_SHADOW
        | glow::SAMPLER_CUBE_SHADOW
        | glow::INT_SAMPLER_2D
        | glow::INT_SAMPLER_3D
        | glow::INT_SAMPLER_CUBE
        | glow::INT_SAMPLER_2D_ARRAY => (1, 1),
        glow::UNSIGNED_INT => (2, 1),
        glow::UNSIGNED_INT_VEC2 => (2, 2),
        glow::UNSIGNED_INT_VEC3 => (2, 3),
        glow::UNSIGNED_INT_VEC4 => (2, 4),
        glow::UNSIGNED_INT_SAMPLER_2D
        | glow::UNSIGNED_INT_SAMPLER_3D
        | glow::UNSIGNED_INT_SAMPLER_CUBE
        | glow::UNSIGNED_INT_SAMPLER_2D_ARRAY => (2, 1),
        _ => return None,
    })
}

fn next_object(context: &mut WebGlContext) -> Result<u64, String> {
    context.next_object = context
        .next_object
        .checked_add(1)
        .ok_or("WebGL object id overflow")?;
    Ok(context.next_object)
}

fn element_offset_pointers(offsets: &[i32]) -> Result<Vec<*const c_void>, String> {
    offsets
        .iter()
        .map(|offset| {
            usize::try_from(*offset)
                .map(|offset| offset as *const c_void)
                .map_err(|_| "multi-draw element offsets must be non-negative".to_owned())
        })
        .collect()
}

impl Drop for AngleStore {
    fn drop(&mut self) {
        let _guard = lock();
        let Some(backend) = self.backend.as_ref() else {
            return;
        };
        unsafe {
            (backend.egl.make_current)(
                backend.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            for (_, context) in self.contexts.drain() {
                (backend.egl.destroy_context)(backend.display, context.egl_context);
                (backend.egl.destroy_surface)(backend.display, context.surface);
            }
        }
    }
}

fn load_backend() -> Result<Option<Arc<Backend>>, String> {
    BACKEND.get_or_init(load_backend_once).clone()
}

fn load_backend_once() -> Result<Option<Arc<Backend>>, String> {
    let mut candidates = Vec::<PathBuf>::new();
    if let Some(directory) = std::env::var_os("BRIMP_ANGLE_LIB_DIR") {
        push_library_candidates(&mut candidates, Path::new(&directory));
    }
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        push_library_candidates(&mut candidates, directory);
    }
    if let Some(directory) = std::env::var_os("BRIMP_JSC_LIB_DIR") {
        push_library_candidates(&mut candidates, Path::new(&directory));
    }
    push_library_candidates(
        &mut candidates,
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../WebKit/WebKitBuild/Release"),
    );
    candidates.dedup();
    for candidate in candidates {
        let Ok(library) = (unsafe { libloading::Library::new(&candidate) }) else {
            continue;
        };
        let Ok(egl) = (unsafe { EglApi::load(&library) }) else {
            continue;
        };
        let display = unsafe { (egl.get_display)(default_native_display()) };
        if display.is_null() {
            continue;
        }
        if unsafe { (egl.initialize)(display, std::ptr::null_mut(), std::ptr::null_mut()) }
            == egl::FALSE
        {
            continue;
        }
        return Ok(Some(Arc::new(Backend {
            _library: library,
            egl,
            display,
        })));
    }
    Ok(None)
}

fn push_library_candidates(candidates: &mut Vec<PathBuf>, directory: &Path) {
    candidates.extend(
        angle_library_names()
            .iter()
            .map(|library| directory.join(library)),
    );
}

#[cfg(target_os = "macos")]
fn default_native_display() -> EglNativeDisplay {
    0
}

#[cfg(not(target_os = "macos"))]
fn default_native_display() -> EglNativeDisplay {
    std::ptr::null_mut()
}

#[cfg(target_os = "macos")]
fn angle_library_names() -> &'static [&'static str] {
    &["libANGLE-shared.dylib", "libEGL.dylib"]
}
#[cfg(target_os = "windows")]
fn angle_library_names() -> &'static [&'static str] {
    &["libANGLE-shared.dll", "libEGL.dll"]
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn angle_library_names() -> &'static [&'static str] {
    &["libANGLE-shared.so", "libEGL.so.1", "libEGL.so"]
}

fn egl_error(backend: &Backend, operation: &str) -> String {
    let error = unsafe { (backend.egl.get_error)() };
    format!("ANGLE {operation} failed with EGL error 0x{error:04x}")
}
