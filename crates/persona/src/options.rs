use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::fingerprint::*;
use crate::seed::PersonaSeed;

/// Browser-family API exposure gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FeaturesConfig {
    pub user_agent_data: Option<bool>,
    pub device_memory: Option<bool>,
    pub network_information: Option<bool>,
    pub trusted_types: Option<bool>,
    pub fetch_later: Option<bool>,
    pub servo_internal_apis: Option<bool>,
    pub media_tracks: Option<bool>,
    pub origin_api: Option<bool>,
    pub quota_exceeded_error: Option<bool>,
    pub visibility_state_entry: Option<bool>,
    pub scoped_custom_element_registry: Option<bool>,
    pub page_transition_events: Option<bool>,
    pub screen_extended: Option<bool>,
    pub moz_window_geometry: Option<bool>,
    pub webgl: Option<bool>,
    pub webgl2: Option<bool>,
    pub webgpu: Option<bool>,
    pub opfs: Option<bool>,
    pub webrtc: Option<bool>,
    pub battery: Option<bool>,
    pub vibration: Option<bool>,
    pub permissions: Option<bool>,
    pub workers: Option<bool>,
    pub service_worker: Option<bool>,
    pub offscreen_canvas: Option<bool>,
    pub touch: Option<bool>,
    pub motion: Option<bool>,
    pub orientation: Option<bool>,
    pub share: Option<bool>,
    pub contacts: Option<bool>,
    pub content_index: Option<bool>,
    pub bluetooth: Option<bool>,
    pub geolocation: Option<bool>,
    pub notifications: Option<bool>,
    pub gamepad: Option<bool>,
}

/// One navigator plugin and its MIME registrations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PluginConfig {
    pub name: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub mime_types: Vec<MimeTypeConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MimeTypeConfig {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub suffixes: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PluginsConfig {
    #[serde(default)]
    pub entries: Vec<PluginConfig>,
    pub pdf_enabled: Option<bool>,
    pub block_system_entries: Option<bool>,
}

/// Shape of Chromium's non-standard `window.chrome` object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ChromeConfig {
    pub exposed: Option<bool>,
    pub enumerable: Option<bool>,
    pub runtime: Option<bool>,
    pub app: Option<bool>,
    pub load_times: Option<bool>,
    pub csi: Option<bool>,
    pub window_key_strategy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WebGlConfig {
    #[serde(default)]
    pub parameters: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub context_attributes: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub shader_precision_formats: Vec<ShaderPrecisionConfig>,
    pub block_unknown_parameters: Option<bool>,
    pub block_unknown_extensions: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ShaderPrecisionConfig {
    pub shader_type: String,
    pub precision_type: String,
    pub range_min: i32,
    pub range_max: i32,
    pub precision: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    pub sample_rate: Option<u32>,
    pub output_latency_ms: Option<u32>,
    pub max_channel_count: Option<u32>,
    pub compressor_reduction: Option<String>,
    #[serde(default)]
    pub frequency_data: Vec<String>,
    #[serde(default)]
    pub time_domain_data: Vec<String>,
    #[serde(default)]
    pub rendered_buffer: Vec<String>,
    pub render_leading_silence_samples: Option<u32>,
    pub fake_completion_delay_ms: Option<u32>,
    pub native_shape: Option<bool>,
    pub noise_enabled: Option<bool>,
    pub seed: Option<PersonaSeed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SpeechVoiceConfig {
    pub voice_uri: String,
    pub name: String,
    pub lang: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default = "default_true")]
    pub local_service: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SpeechConfig {
    #[serde(default)]
    pub voices: Vec<SpeechVoiceConfig>,
    pub block_system_voices: Option<bool>,
    pub fake_completion_delay_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GeolocationConfig {
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub accuracy_meters: Option<u32>,
    pub altitude: Option<String>,
    pub altitude_accuracy_meters: Option<String>,
    pub heading_degrees: Option<String>,
    pub speed_meters_per_second: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WebRtcConfig {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub substitute_ice_candidates: Option<bool>,
    pub substitute_sdp: Option<bool>,
    pub ice_candidate_semantics: Option<String>,
    pub sdp_semantics: Option<String>,
    pub audio_codecs: Option<Vec<String>>,
    pub video_codecs: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BatteryConfig {
    pub charging: Option<bool>,
    pub charging_time_seconds: Option<u32>,
    pub discharging_time_seconds: Option<u32>,
    pub infinite_discharging_time: Option<bool>,
    pub level_percent: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub quota_bytes: Option<u64>,
    pub usage_bytes: Option<u64>,
    pub legacy_temporary_quota_bytes: Option<u64>,
    pub legacy_persistent_quota_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CssMediaConfig {
    pub pointer: Option<String>,
    pub any_pointer: Option<String>,
    pub hover: Option<String>,
    pub any_hover: Option<String>,
    pub color_gamut: Option<String>,
    pub prefers_reduced_motion: Option<String>,
    pub orientation: Option<String>,
    pub display_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SvgConfig {
    pub bbox_width_factor: Option<String>,
    pub text_width_factor: Option<String>,
    pub baseline_factor: Option<String>,
    pub expose_geometry_methods: Option<bool>,
    pub seed: Option<PersonaSeed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NativeFunctionsConfig {
    pub native_source: Option<String>,
    pub preserve_name: Option<bool>,
    pub preserve_length: Option<bool>,
    pub preserve_descriptors: Option<bool>,
    pub illegal_invocation_errors: Option<bool>,
    pub constructor_prototypes: Option<bool>,
    pub sanitize_stacks: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NoiseConfig {
    pub enabled: Option<bool>,
    pub canvas: Option<bool>,
    pub webgl: Option<bool>,
    pub audio: Option<bool>,
    pub fonts: Option<bool>,
    pub domrect: Option<bool>,
    pub svg: Option<bool>,
}

const fn default_true() -> bool {
    true
}

/// Optional browser identity overrides shared by network and JavaScript surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct IdentityConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Transport fingerprint selected for HTTP/TLS impersonation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    pub impersonation_profile: String,
}

/// Optional network-visible identity overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept_language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept_encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_mobile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_full_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_full_version_list: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_arch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_bitness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_platform_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sec_ch_ua_model: Option<String>,
}

/// CSS viewport dimensions and device scale factor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1440,
            height: 900,
            device_scale_factor: 1,
        }
    }
}

/// Optional screen fingerprint overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScreenConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avail_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avail_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avail_left: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avail_top: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_extended: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation_angle: Option<u16>,
}

/// Optional window fingerprint overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_y: Option<i32>,
}

/// Optional CSS feature-detection overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CssConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moz_prefix_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webkit_prefix_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<CssMediaConfig>,
}

/// Optional WebGL and graphics fingerprint overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GraphicsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl_vendor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl_renderer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl_masked_vendor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl_masked_renderer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl1: Option<WebGlConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgl2: Option<WebGlConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgpu_adapter_vendor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgpu_adapter_architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgpu_adapter_device: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgpu_adapter_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webgpu_max_bind_groups_plus_vertex_buffers: Option<u32>,
}

/// Optional `navigator` and JS capability overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NavigatorConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    /// Firefox-only `navigator.oscpu`. An empty value removes the property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oscpu: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_concurrency: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_memory_gb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_touch_points: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_rtt_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_downlink_mbps: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_effective_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_save_data: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_permission: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub do_not_track: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_privacy_control: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expose_global_privacy_control: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bluetooth_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bluetooth_available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_devices_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offscreen_canvas_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ua_platform_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ua_architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ua_bitness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ua_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_sub: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_viewer_enabled: Option<bool>,
}

/// Optional media device and speech synthesis overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MediaConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_voices: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_inputs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_inputs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_outputs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_source_supported_types: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoding_capabilities: Option<Vec<MediaDecodingCapability>>,
}

/// Optional font list and font seed overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FontConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub families: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<PersonaSeed>,
    #[serde(default)]
    pub emoji_fallback: Vec<String>,
    pub text_metric_profile: Option<String>,
    pub spacing_seed: Option<PersonaSeed>,
    pub subpixel_antialiasing: Option<bool>,
    pub hinting: Option<String>,
}

/// Optional canvas fingerprint behavior overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CanvasConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_amplitude: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blink_low_entropy_probe: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<PersonaSeed>,
}

/// Optional DOMRect/SVGRect fingerprint behavior overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DomRectConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantization_steps_per_px: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_empty_client_rects: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<PersonaSeed>,
    pub sampled_profile: Option<String>,
    pub transform_model: Option<String>,
    pub font_profile: Option<String>,
    pub subpixel_unit: Option<u32>,
    pub rounding: Option<String>,
    pub preserve_negative_zero: Option<bool>,
    pub clamp_transforms: Option<bool>,
}

/// Optional JavaScript engine fingerprint overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Engine implementation family, such as `v8`, `javascriptcore`, or `spidermonkey`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Free-form behavior profile identifier, such as `chrome-150` or `safari-26`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Engine version exposed by the selected profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Requested stack trace syntax. Implementations may add new styles over time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_format: Option<String>,
    /// Maximum synchronous frames exposed by Error.stack for V8-compatible profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_trace_limit: Option<u32>,
    /// Whether Error.stack may retain asynchronous parent frames.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_stack_traces: Option<bool>,
    /// Whether DOM high-resolution timestamps use Chromium's absolute clock lattice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chromium_high_resolution_time: Option<bool>,
    /// Upper bound for synchronous Date.now() intervals adjusted to the engine profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_now_short_interval_threshold_ms: Option<u32>,
    /// Percentage applied to configured short Date.now() intervals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_now_short_interval_scale_percent: Option<u32>,
    /// Error text indexed by a stable operation key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub error_messages: BTreeMap<String, String>,
    /// Function source text indexed by a stable builtin key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub builtin_sources: BTreeMap<String, String>,
}

impl IdentityConfig {
    pub(crate) fn apply_to(&self, persona: &mut ResolvedPersona) {
        apply_non_empty(&self.family, &mut persona.browser_family);
        apply_non_empty(&self.brand, &mut persona.browser_name);
        apply_non_empty(&self.version, &mut persona.browser_version);
        apply_non_empty(&self.engine, &mut persona.engine_name);
        apply_non_empty(&self.platform, &mut persona.platform_name);
        apply_non_empty(&self.platform_version, &mut persona.js.ua_platform_version);
        apply_non_empty(&self.architecture, &mut persona.js.ua_architecture);
        apply_non_empty(&self.bitness, &mut persona.js.ua_bitness);
        if let Some(model) = &self.model {
            persona.js.ua_model = model.clone();
        }
    }
}

impl FeaturesConfig {
    pub(crate) fn apply_to(&self, features: &mut FeaturesConfig) {
        macro_rules! apply {
            ($($field:ident),+ $(,)?) => {
                $(if self.$field.is_some() { features.$field = self.$field; })+
            };
        }
        apply!(
            user_agent_data,
            device_memory,
            network_information,
            trusted_types,
            fetch_later,
            servo_internal_apis,
            media_tracks,
            origin_api,
            quota_exceeded_error,
            visibility_state_entry,
            scoped_custom_element_registry,
            page_transition_events,
            screen_extended,
            moz_window_geometry,
            webgl,
            webgl2,
            webgpu,
            opfs,
            webrtc,
            battery,
            vibration,
            permissions,
            workers,
            service_worker,
            offscreen_canvas,
            touch,
            motion,
            orientation,
            share,
            contacts,
            content_index,
            bluetooth,
            geolocation,
            notifications,
            gamepad,
        );
    }
}

impl TransportConfig {
    pub(crate) fn apply_to(&self, persona: &mut ResolvedPersona) {
        if !self.impersonation_profile.trim().is_empty() {
            persona.transport_profile = self.impersonation_profile.trim().to_string();
        }
    }
}

impl NetworkConfig {
    pub(crate) fn apply_to(&self, network: &mut NetworkFingerprint) {
        apply_non_empty(&self.user_agent, &mut network.user_agent);
        apply_non_empty(&self.accept, &mut network.accept_header);
        apply_non_empty(&self.accept_language, &mut network.accept_language);
        apply_non_empty(&self.accept_encoding, &mut network.accept_encoding);
        apply_present(&self.sec_ch_ua, &mut network.sec_ch_ua);
        apply_present(&self.sec_ch_ua_mobile, &mut network.sec_ch_ua_mobile);
        apply_present(&self.sec_ch_ua_platform, &mut network.sec_ch_ua_platform);
        apply_present(
            &self.sec_ch_ua_full_version,
            &mut network.sec_ch_ua_full_version,
        );
        apply_present(
            &self.sec_ch_ua_full_version_list,
            &mut network.sec_ch_ua_full_version_list,
        );
        apply_present(&self.sec_ch_ua_arch, &mut network.sec_ch_ua_arch);
        apply_present(&self.sec_ch_ua_bitness, &mut network.sec_ch_ua_bitness);
        apply_present(
            &self.sec_ch_ua_platform_version,
            &mut network.sec_ch_ua_platform_version,
        );
        if let Some(value) = &self.sec_ch_ua_model {
            network.sec_ch_ua_model = value.clone();
        }
    }
}

fn apply_non_empty(value: &Option<String>, target: &mut String) {
    if let Some(value) = value
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        *target = value.to_string();
    }
}

fn apply_present(value: &Option<String>, target: &mut String) {
    if let Some(value) = value {
        *target = value.trim().to_string();
    }
}

impl EngineConfig {
    pub(crate) fn apply_to(&self, engine: &mut EngineFingerprint) {
        if let Some(enabled) = self.enabled {
            engine.enabled = enabled;
        }
        apply_non_empty(&self.family, &mut engine.family);
        apply_non_empty(&self.profile, &mut engine.profile);
        apply_non_empty(&self.version, &mut engine.version);
        apply_non_empty(&self.stack_format, &mut engine.stack_format);
        if let Some(stack_trace_limit) = self.stack_trace_limit {
            engine.stack_trace_limit = stack_trace_limit;
        }
        if let Some(async_stack_traces) = self.async_stack_traces {
            engine.async_stack_traces = async_stack_traces;
        }
        if let Some(chromium_high_resolution_time) = self.chromium_high_resolution_time {
            engine.chromium_high_resolution_time = chromium_high_resolution_time;
        }
        if let Some(value) = self.date_now_short_interval_threshold_ms {
            engine.date_now_short_interval_threshold_ms = value;
        }
        if let Some(value) = self.date_now_short_interval_scale_percent {
            engine.date_now_short_interval_scale_percent = value.clamp(1, 100);
        }
        engine.error_messages.extend(
            self.error_messages
                .iter()
                .filter(|(key, value)| !key.is_empty() && !value.is_empty())
                .map(|(key, value)| (key.clone(), value.clone())),
        );
        engine.builtin_sources.extend(
            self.builtin_sources
                .iter()
                .filter(|(key, value)| !key.is_empty() && !value.is_empty())
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }
}

impl DomRectConfig {
    pub(crate) fn apply_to(&self, domrect: &mut DomRectFingerprint) {
        if let Some(enabled) = self.enabled {
            domrect.enabled = enabled;
        }
        if let Some(quantization_steps_per_px) = self.quantization_steps_per_px {
            domrect.quantization_steps_per_px = quantization_steps_per_px;
            if self.subpixel_unit.is_none() {
                domrect.subpixel_unit = quantization_steps_per_px;
            }
        }
        if let Some(fill_empty_client_rects) = self.fill_empty_client_rects {
            domrect.fill_empty_client_rects = fill_empty_client_rects;
        }
        if let Some(seed) = self.seed.as_ref() {
            domrect.seed = seed.clone();
        }
        domrect.sampled_profile = self
            .sampled_profile
            .clone()
            .or(domrect.sampled_profile.clone());
        domrect.transform_model = self
            .transform_model
            .clone()
            .or(domrect.transform_model.clone());
        domrect.font_profile = self.font_profile.clone().or(domrect.font_profile.clone());
        if let Some(value) = self.subpixel_unit {
            domrect.subpixel_unit = value;
            domrect.quantization_steps_per_px = value;
        }
        if let Some(value) = &self.rounding {
            domrect.rounding = value.clone();
        }
        if let Some(value) = self.preserve_negative_zero {
            domrect.preserve_negative_zero = value;
        }
        if let Some(value) = self.clamp_transforms {
            domrect.clamp_transforms = value;
        }
    }
}

impl CanvasConfig {
    pub(crate) fn apply_to(&self, canvas: &mut CanvasFingerprint) {
        if let Some(noise_enabled) = self.noise_enabled {
            canvas.noise_enabled = noise_enabled;
        }
        if let Some(noise_amplitude) = self.noise_amplitude {
            canvas.noise_amplitude = noise_amplitude;
        }
        if let Some(blink_low_entropy_probe) = self.blink_low_entropy_probe {
            canvas.blink_low_entropy_probe = blink_low_entropy_probe;
        }
        if let Some(seed) = self.seed.as_ref() {
            canvas.seed = seed.clone();
        }
    }
}

impl MediaConfig {
    pub(crate) fn apply_to(&self, media: &mut MediaFingerprint) {
        if let Some(speech_voices) = self
            .speech_voices
            .as_ref()
            .filter(|voices| !voices.is_empty())
        {
            media.speech_voices = speech_voices.clone();
        }
        if let Some(audio_inputs) = self.audio_inputs {
            media.audio_inputs = audio_inputs;
        }
        if let Some(video_inputs) = self.video_inputs {
            media.video_inputs = video_inputs;
        }
        if let Some(audio_outputs) = self.audio_outputs {
            media.audio_outputs = audio_outputs;
        }
        if let Some(types) = self
            .media_source_supported_types
            .as_ref()
            .filter(|types| !types.is_empty())
        {
            media.media_source_supported_types = types.clone();
        }
        if let Some(capabilities) = self
            .decoding_capabilities
            .as_ref()
            .filter(|capabilities| !capabilities.is_empty())
        {
            media.decoding_capabilities = capabilities.clone();
        }
    }
}

impl FontConfig {
    pub(crate) fn apply_to(&self, fonts: &mut FontFingerprint) {
        if let Some(families) = self
            .families
            .as_ref()
            .map(|families| {
                families
                    .iter()
                    .map(|family| family.trim())
                    .filter(|family| !family.is_empty())
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .filter(|families| !families.is_empty())
        {
            fonts.families = families;
        }
        if let Some(seed) = self.seed.as_ref() {
            fonts.seed = seed.clone();
        }
        if !self.emoji_fallback.is_empty() {
            fonts.emoji_fallback = self.emoji_fallback.clone();
        }
        fonts.text_metric_profile = self
            .text_metric_profile
            .clone()
            .or(fonts.text_metric_profile.clone());
        if let Some(value) = &self.spacing_seed {
            fonts.spacing_seed = value.clone();
        }
        if let Some(value) = self.subpixel_antialiasing {
            fonts.subpixel_antialiasing = value;
        }
        if let Some(value) = &self.hinting {
            fonts.hinting = value.clone();
        }
    }
}

impl NavigatorConfig {
    pub(crate) fn apply_to(&self, js: &mut JsFingerprint) {
        if let Some(app_version) = self.app_version.as_ref().filter(|value| !value.is_empty()) {
            js.app_version = app_version.clone();
        }
        if let Some(oscpu) = &self.oscpu {
            js.oscpu = oscpu.clone();
        }
        if let Some(platform) = self.platform.as_ref().filter(|value| !value.is_empty()) {
            js.platform = platform.clone();
        }
        if let Some(language) = self.language.as_ref().filter(|value| !value.is_empty()) {
            js.language = language.clone();
        }
        if let Some(languages) = self.languages.as_ref().filter(|values| !values.is_empty()) {
            js.languages = languages.clone();
            if self.language.is_none()
                && let Some(language) = js.languages.first()
            {
                js.language = language.clone();
            }
        }
        if let Some(hardware_concurrency) = self.hardware_concurrency {
            js.hardware_concurrency = hardware_concurrency;
        }
        if let Some(device_memory_gb) = self.device_memory_gb {
            js.device_memory_gb = device_memory_gb;
        }
        if let Some(max_touch_points) = self.max_touch_points {
            js.max_touch_points = max_touch_points;
        }
        if let Some(connection_type) = self
            .connection_type
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.connection_type = connection_type.clone();
        }
        if let Some(connection_rtt_ms) = self.connection_rtt_ms {
            js.connection_rtt_ms = connection_rtt_ms;
        }
        if let Some(connection_downlink_mbps) = self
            .connection_downlink_mbps
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.connection_downlink_mbps = connection_downlink_mbps.clone();
        }
        if let Some(connection_effective_type) = self
            .connection_effective_type
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.connection_effective_type = connection_effective_type.clone();
        }
        if let Some(connection_save_data) = self.connection_save_data {
            js.connection_save_data = connection_save_data;
        }
        if let Some(notification_permission) = self
            .notification_permission
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.notification_permission = notification_permission.clone();
        }
        if let Some(do_not_track) = self.do_not_track.as_ref() {
            js.do_not_track = do_not_track.clone();
        }
        if let Some(global_privacy_control) = self.global_privacy_control {
            js.global_privacy_control = global_privacy_control;
        }
        if let Some(expose_global_privacy_control) = self.expose_global_privacy_control {
            js.expose_global_privacy_control = expose_global_privacy_control;
        }
        if let Some(permissions_enabled) = self.permissions_enabled {
            js.permissions_enabled = permissions_enabled;
        }
        if let Some(bluetooth_enabled) = self.bluetooth_enabled {
            js.bluetooth_enabled = bluetooth_enabled;
        }
        if let Some(bluetooth_available) = self.bluetooth_available {
            js.bluetooth_available = bluetooth_available;
        }
        if let Some(media_devices_enabled) = self.media_devices_enabled {
            js.media_devices_enabled = media_devices_enabled;
        }
        if let Some(offscreen_canvas_enabled) = self.offscreen_canvas_enabled {
            js.offscreen_canvas_enabled = offscreen_canvas_enabled;
        }
        if let Some(ua_platform_version) = self
            .ua_platform_version
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.ua_platform_version = ua_platform_version.clone();
        }
        if let Some(ua_architecture) = self
            .ua_architecture
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            js.ua_architecture = ua_architecture.clone();
        }
        if let Some(ua_bitness) = self.ua_bitness.as_ref().filter(|value| !value.is_empty()) {
            js.ua_bitness = ua_bitness.clone();
        }
        if let Some(ua_model) = self.ua_model.as_ref() {
            js.ua_model = ua_model.clone();
        }
        if let Some(vendor) = self.vendor.as_ref().filter(|value| !value.is_empty()) {
            js.vendor = vendor.clone();
        }
        if let Some(product_sub) = self.product_sub.as_ref().filter(|value| !value.is_empty()) {
            js.product_sub = product_sub.clone();
        }
        if let Some(pdf_viewer_enabled) = self.pdf_viewer_enabled {
            js.pdf_viewer_enabled = pdf_viewer_enabled;
        }
    }
}

impl GraphicsConfig {
    pub(crate) fn apply_to(&self, graphics: &mut GraphicsFingerprint) {
        if let Some(webgl_vendor) = self.webgl_vendor.as_ref().filter(|value| !value.is_empty()) {
            graphics.webgl_vendor = webgl_vendor.clone();
        }
        if let Some(webgl_renderer) = self
            .webgl_renderer
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            graphics.webgl_renderer = webgl_renderer.clone();
        }
        if let Some(webgl_masked_vendor) = self
            .webgl_masked_vendor
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            graphics.webgl_masked_vendor = webgl_masked_vendor.clone();
        }
        if let Some(webgl_masked_renderer) = self
            .webgl_masked_renderer
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            graphics.webgl_masked_renderer = webgl_masked_renderer.clone();
        }
        if let Some(value) = &self.webgl1 {
            graphics.webgl1 = value.clone();
        }
        if let Some(value) = &self.webgl2 {
            graphics.webgl2 = value.clone();
        }
        if let Some(value) = &self.webgpu_adapter_vendor {
            graphics.webgpu_adapter_vendor = value.clone();
        }
        if let Some(value) = &self.webgpu_adapter_architecture {
            graphics.webgpu_adapter_architecture = value.clone();
        }
        if let Some(value) = &self.webgpu_adapter_device {
            graphics.webgpu_adapter_device = value.clone();
        }
        if let Some(value) = &self.webgpu_adapter_description {
            graphics.webgpu_adapter_description = value.clone();
        }
        if let Some(value) = self.webgpu_max_bind_groups_plus_vertex_buffers {
            graphics.webgpu_max_bind_groups_plus_vertex_buffers = value;
        }
    }
}

impl ScreenConfig {
    pub(crate) fn apply_to(&self, screen: &mut ScreenFingerprint) {
        if let Some(width) = self.width {
            screen.width = width;
        }
        if let Some(height) = self.height {
            screen.height = height;
        }
        if let Some(avail_width) = self.avail_width {
            screen.avail_width = avail_width;
        }
        if let Some(avail_height) = self.avail_height {
            screen.avail_height = avail_height;
        }
        if let Some(avail_left) = self.avail_left {
            screen.avail_left = avail_left;
        }
        if let Some(avail_top) = self.avail_top {
            screen.avail_top = avail_top;
        }
        if let Some(left) = self.left {
            screen.left = left;
        }
        if let Some(top) = self.top {
            screen.top = top;
        }
        if let Some(color_depth) = self.color_depth {
            screen.color_depth = color_depth;
        }
        if let Some(pixel_depth) = self.pixel_depth {
            screen.pixel_depth = pixel_depth;
        }
        if let Some(is_extended) = self.is_extended {
            screen.is_extended = is_extended;
        }
        if let Some(orientation_type) = self
            .orientation_type
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            screen.orientation_type = orientation_type.clone();
        }
        if let Some(orientation_angle) = self.orientation_angle {
            screen.orientation_angle = orientation_angle;
        }
    }
}

impl WindowConfig {
    pub(crate) fn apply_to(&self, window: &mut WindowFingerprint) {
        if let Some(outer_width) = self.outer_width {
            window.outer_width = outer_width;
        }
        if let Some(outer_height) = self.outer_height {
            window.outer_height = outer_height;
        }
        if let Some(screen_x) = self.screen_x {
            window.screen_x = screen_x;
        }
        if let Some(screen_y) = self.screen_y {
            window.screen_y = screen_y;
        }
    }
}

impl CssConfig {
    pub(crate) fn apply_to(&self, css: &mut CssFingerprint) {
        if let Some(moz_prefix_enabled) = self.moz_prefix_enabled {
            css.moz_prefix_enabled = moz_prefix_enabled;
        }
        if let Some(webkit_prefix_enabled) = self.webkit_prefix_enabled {
            css.webkit_prefix_enabled = webkit_prefix_enabled;
        }
        if let Some(media) = &self.media {
            css.media = media.clone();
        }
    }
}

impl AudioConfig {
    pub(crate) fn apply_to(&self, audio: &mut AudioFingerprint) {
        if let Some(value) = self.sample_rate {
            audio.sample_rate = value;
        }
        if let Some(value) = self.output_latency_ms {
            audio.output_latency_ms = value;
        }
        if let Some(value) = self.max_channel_count {
            audio.max_channel_count = value;
        }
        if let Some(value) = &self.compressor_reduction {
            audio.compressor_reduction = value.clone();
        }
        if !self.frequency_data.is_empty() {
            audio.frequency_data = self.frequency_data.clone();
        }
        if !self.time_domain_data.is_empty() {
            audio.time_domain_data = self.time_domain_data.clone();
        }
        if !self.rendered_buffer.is_empty() {
            audio.rendered_buffer = self.rendered_buffer.clone();
        }
        if let Some(value) = self.render_leading_silence_samples {
            audio.render_leading_silence_samples = value;
        }
        if let Some(value) = self.fake_completion_delay_ms {
            audio.fake_completion_delay_ms = value;
        }
        if let Some(value) = self.native_shape {
            audio.native_shape = value;
        }
        if let Some(value) = self.noise_enabled {
            audio.noise_enabled = value;
        }
        if let Some(value) = &self.seed {
            audio.seed = value.clone();
        }
    }
}

impl GeolocationConfig {
    pub(crate) fn apply_to(&self, geo: &mut GeoFingerprint) {
        if self.latitude.is_some() {
            geo.latitude = self.latitude.clone();
        }
        if self.longitude.is_some() {
            geo.longitude = self.longitude.clone();
        }
        if self.accuracy_meters.is_some() {
            geo.accuracy_meters = self.accuracy_meters;
        }
        if self.altitude.is_some() {
            geo.altitude = self.altitude.clone();
        }
        if self.altitude_accuracy_meters.is_some() {
            geo.altitude_accuracy_meters = self.altitude_accuracy_meters.clone();
        }
        if self.heading_degrees.is_some() {
            geo.heading_degrees = self.heading_degrees.clone();
        }
        if self.speed_meters_per_second.is_some() {
            geo.speed_meters_per_second = self.speed_meters_per_second.clone();
        }
    }
}

impl WebRtcConfig {
    pub(crate) fn apply_to(&self, webrtc: &mut WebRtcFingerprint) {
        if self.ipv4.is_some() {
            webrtc.ipv4 = self.ipv4.clone();
        }
        if self.ipv6.is_some() {
            webrtc.ipv6 = self.ipv6.clone();
        }
        if let Some(value) = self.substitute_ice_candidates {
            webrtc.substitute_ice_candidates = value;
        }
        if let Some(value) = self.substitute_sdp {
            webrtc.substitute_sdp = value;
        }
        if let Some(value) = self
            .ice_candidate_semantics
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            webrtc.ice_candidate_semantics = value.clone();
        }
        if let Some(value) = self
            .sdp_semantics
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            webrtc.sdp_semantics = value.clone();
        }
        if let Some(value) = self.audio_codecs.as_ref().filter(|value| !value.is_empty()) {
            webrtc.audio_codecs = value.clone();
        }
        if let Some(value) = self.video_codecs.as_ref().filter(|value| !value.is_empty()) {
            webrtc.video_codecs = value.clone();
        }
    }
}

impl BatteryConfig {
    pub(crate) fn apply_to(&self, battery: &mut BatteryFingerprint) {
        if let Some(value) = self.charging {
            battery.charging = value;
        }
        if let Some(value) = self.charging_time_seconds {
            battery.charging_time_seconds = value;
        }
        if self.infinite_discharging_time == Some(true) {
            battery.discharging_time_seconds = None;
        } else if self.discharging_time_seconds.is_some() {
            battery.discharging_time_seconds = self.discharging_time_seconds;
        }
        if let Some(value) = self.level_percent {
            battery.level_percent = value;
        }
    }
}

impl StorageConfig {
    pub(crate) fn apply_to(&self, storage: &mut StorageFingerprint) {
        if self.quota_bytes.is_some() {
            storage.quota_bytes = self.quota_bytes;
        }
        if let Some(value) = self.usage_bytes {
            storage.usage_bytes = value;
        }
        if self.legacy_temporary_quota_bytes.is_some() {
            storage.legacy_temporary_quota_bytes = self.legacy_temporary_quota_bytes;
        }
        if self.legacy_persistent_quota_bytes.is_some() {
            storage.legacy_persistent_quota_bytes = self.legacy_persistent_quota_bytes;
        }
    }
}

impl NoiseConfig {
    pub(crate) fn apply_to(&self, persona: &mut ResolvedPersona) {
        let master = self.enabled.unwrap_or(true);
        persona.canvas.noise_enabled =
            master && self.canvas.unwrap_or(persona.canvas.noise_enabled);
        persona.audio.noise_enabled = master && self.audio.unwrap_or(persona.audio.noise_enabled);
        persona.domrect.enabled = master && self.domrect.unwrap_or(persona.domrect.enabled);
    }
}

impl SvgConfig {
    pub(crate) fn apply_to(&self, svg: &mut SvgConfig) {
        if self.bbox_width_factor.is_some() {
            svg.bbox_width_factor = self.bbox_width_factor.clone();
        }
        if self.text_width_factor.is_some() {
            svg.text_width_factor = self.text_width_factor.clone();
        }
        if self.baseline_factor.is_some() {
            svg.baseline_factor = self.baseline_factor.clone();
        }
        if self.expose_geometry_methods.is_some() {
            svg.expose_geometry_methods = self.expose_geometry_methods;
        }
        if self.seed.is_some() {
            svg.seed = self.seed.clone();
        }
    }
}

impl ChromeConfig {
    pub(crate) fn apply_to(&self, chrome: &mut ChromeConfig) {
        macro_rules! apply {
            ($field:ident) => {
                if self.$field.is_some() {
                    chrome.$field = self.$field.clone();
                }
            };
        }
        apply!(exposed);
        apply!(enumerable);
        apply!(runtime);
        apply!(app);
        apply!(load_times);
        apply!(csi);
        apply!(window_key_strategy);
    }
}
