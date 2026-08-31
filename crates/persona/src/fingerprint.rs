use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::options::*;
use crate::seed::{BrowserPersonaPreset, PersonaSeed};

/// Network-visible browser fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkFingerprint {
    pub user_agent: String,
    pub accept_language: String,
    pub accept_header: String,
    pub accept_encoding: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
    pub sec_ch_ua_full_version: String,
    pub sec_ch_ua_full_version_list: String,
    pub sec_ch_ua_arch: String,
    pub sec_ch_ua_bitness: String,
    pub sec_ch_ua_platform_version: String,
    pub sec_ch_ua_model: String,
}

/// Resolved screen fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenFingerprint {
    pub width: u32,
    pub height: u32,
    pub avail_width: u32,
    pub avail_height: u32,
    pub avail_left: i32,
    pub avail_top: i32,
    pub left: i32,
    pub top: i32,
    pub color_depth: u32,
    pub pixel_depth: u32,
    pub device_scale_factor: u32,
    pub is_extended: bool,
    pub orientation_type: String,
    pub orientation_angle: u16,
}

/// Resolved window fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowFingerprint {
    pub outer_width: u32,
    pub outer_height: u32,
    pub screen_x: i32,
    pub screen_y: i32,
}

/// Resolved CSS feature-detection fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CssFingerprint {
    pub moz_prefix_enabled: bool,
    pub webkit_prefix_enabled: bool,
    pub media: CssMediaConfig,
}

impl WindowFingerprint {
    pub fn from_viewport(viewport: &Viewport) -> Self {
        Self {
            outer_width: viewport.width,
            outer_height: viewport.height,
            screen_x: 0,
            screen_y: 0,
        }
    }
}

impl ScreenFingerprint {
    pub fn from_viewport(viewport: &Viewport) -> Self {
        Self {
            width: viewport.width,
            height: viewport.height,
            avail_width: viewport.width,
            avail_height: viewport.height,
            avail_left: 0,
            avail_top: 0,
            left: 0,
            top: 0,
            color_depth: 24,
            pixel_depth: 24,
            device_scale_factor: viewport.device_scale_factor,
            is_extended: false,
            orientation_type: if viewport.width >= viewport.height {
                "landscape-primary"
            } else {
                "portrait-primary"
            }
            .to_string(),
            orientation_angle: 0,
        }
    }
}

/// Resolved WebGL and graphics fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphicsFingerprint {
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub webgl_masked_vendor: String,
    pub webgl_masked_renderer: String,
    pub webgl_seed: PersonaSeed,
    pub webgl1: WebGlConfig,
    pub webgl2: WebGlConfig,
    pub webgpu_adapter_vendor: String,
    pub webgpu_adapter_architecture: String,
    pub webgpu_adapter_device: String,
    pub webgpu_adapter_description: String,
    pub webgpu_max_bind_groups_plus_vertex_buffers: u32,
}

/// Resolved JavaScript environment fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsFingerprint {
    pub app_version: String,
    pub oscpu: String,
    pub platform: String,
    pub language: String,
    pub languages: Vec<String>,
    pub hardware_concurrency: u32,
    pub device_memory_gb: u32,
    pub max_touch_points: u32,
    pub connection_type: String,
    pub connection_rtt_ms: u32,
    pub connection_downlink_mbps: String,
    pub connection_effective_type: String,
    pub connection_save_data: bool,
    pub notification_permission: String,
    pub do_not_track: String,
    pub expose_global_privacy_control: bool,
    pub global_privacy_control: bool,
    pub permissions_enabled: bool,
    pub bluetooth_enabled: bool,
    pub bluetooth_available: bool,
    pub media_devices_enabled: bool,
    pub webgpu_enabled: bool,
    pub offscreen_canvas_enabled: bool,
    pub service_worker_enabled: bool,
    pub ua_platform_version: String,
    pub ua_architecture: String,
    pub ua_bitness: String,
    pub ua_model: String,
    pub timezone: String,
    pub vendor: String,
    pub product_sub: String,
    pub pdf_viewer_enabled: bool,
}

/// Resolved locale, timezone, and geolocation fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoFingerprint {
    pub timezone: String,
    pub locale: String,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub accuracy_meters: Option<u32>,
    pub altitude: Option<String>,
    pub altitude_accuracy_meters: Option<String>,
    pub heading_degrees: Option<String>,
    pub speed_meters_per_second: Option<String>,
}

/// Resolved WebRTC address fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebRtcFingerprint {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub substitute_ice_candidates: bool,
    pub substitute_sdp: bool,
    pub ice_candidate_semantics: String,
    pub sdp_semantics: String,
    pub audio_codecs: Vec<String>,
    pub video_codecs: Vec<String>,
}

/// Resolved canvas fingerprint behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanvasFingerprint {
    pub seed: PersonaSeed,
    pub noise_enabled: bool,
    pub noise_amplitude: u8,
    pub blink_low_entropy_probe: bool,
}

/// Resolved DOMRect/SVGRect fingerprint behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomRectFingerprint {
    pub seed: PersonaSeed,
    pub enabled: bool,
    pub quantization_steps_per_px: u32,
    pub fill_empty_client_rects: bool,
    pub sampled_profile: Option<String>,
    pub transform_model: Option<String>,
    pub font_profile: Option<String>,
    pub subpixel_unit: u32,
    pub rounding: String,
    pub preserve_negative_zero: bool,
    pub clamp_transforms: bool,
}

/// Resolved JavaScript engine fingerprint behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineFingerprint {
    pub enabled: bool,
    pub family: String,
    pub profile: String,
    pub version: String,
    pub stack_format: String,
    pub stack_trace_limit: u32,
    pub async_stack_traces: bool,
    pub chromium_high_resolution_time: bool,
    pub date_now_short_interval_threshold_ms: u32,
    pub date_now_short_interval_scale_percent: u32,
    pub error_messages: BTreeMap<String, String>,
    pub builtin_sources: BTreeMap<String, String>,
}

/// Resolved audio fingerprint behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFingerprint {
    pub seed: PersonaSeed,
    pub sample_rate: u32,
    pub output_latency_ms: u32,
    pub max_channel_count: u32,
    pub compressor_reduction: String,
    pub frequency_data: Vec<String>,
    pub time_domain_data: Vec<String>,
    pub rendered_buffer: Vec<String>,
    pub render_leading_silence_samples: u32,
    pub fake_completion_delay_ms: u32,
    pub native_shape: bool,
    pub noise_enabled: bool,
}

/// Resolved font fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontFingerprint {
    pub seed: PersonaSeed,
    pub families: Vec<String>,
    pub emoji_fallback: Vec<String>,
    pub text_metric_profile: Option<String>,
    pub spacing_seed: PersonaSeed,
    pub subpixel_antialiasing: bool,
    pub hinting: String,
}

/// Resolved media devices and speech synthesis fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaFingerprint {
    pub speech_voices: Vec<String>,
    pub audio_inputs: u32,
    pub video_inputs: u32,
    pub audio_outputs: u32,
    pub media_source_supported_types: Vec<String>,
    pub decoding_capabilities: Vec<MediaDecodingCapability>,
}

/// Persona result for one `MediaCapabilities.decodingInfo()` content type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaDecodingCapability {
    pub content_type: String,
    pub supported: bool,
    pub smooth: bool,
    pub power_efficient: bool,
}

/// Resolved battery fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryFingerprint {
    pub charging: bool,
    pub charging_time_seconds: u32,
    pub discharging_time_seconds: Option<u32>,
    pub level_percent: u32,
}

/// Resolved storage quota fingerprint values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageFingerprint {
    pub quota_bytes: Option<u64>,
    pub usage_bytes: u64,
    pub legacy_temporary_quota_bytes: Option<u64>,
    pub legacy_persistent_quota_bytes: Option<u64>,
}

/// Complete resolved persona consumed by the Brimp runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPersona {
    pub seed: PersonaSeed,
    pub preset: BrowserPersonaPreset,
    /// Concrete curl-impersonate profile key.
    pub transport_profile: String,
    pub browser_family: String,
    pub browser_name: String,
    pub browser_version: String,
    pub engine_name: String,
    pub platform_name: String,
    pub viewport: Viewport,
    pub screen: ScreenFingerprint,
    pub window: WindowFingerprint,
    pub css: CssFingerprint,
    pub network: NetworkFingerprint,
    pub graphics: GraphicsFingerprint,
    pub js: JsFingerprint,
    pub geo: GeoFingerprint,
    pub webrtc: WebRtcFingerprint,
    pub canvas: CanvasFingerprint,
    pub domrect: DomRectFingerprint,
    pub engine: EngineFingerprint,
    pub audio: AudioFingerprint,
    pub fonts: FontFingerprint,
    pub media: MediaFingerprint,
    pub battery: BatteryFingerprint,
    pub storage: StorageFingerprint,
    pub features: FeaturesConfig,
    pub plugins: PluginsConfig,
    pub chrome: ChromeConfig,
    pub speech: SpeechConfig,
    pub svg: SvgConfig,
    pub native_functions: NativeFunctionsConfig,
    pub noise: NoiseConfig,
}

pub type BrowserPersona = ResolvedPersona;

impl ResolvedPersona {
    pub fn from_preset(preset: BrowserPersonaPreset) -> Self {
        Self::from_preset_and_seed(
            preset,
            PersonaSeed::from_stable_input(format!("preset:{preset:?}")),
        )
    }

    pub fn from_preset_and_seed(preset: BrowserPersonaPreset, seed: PersonaSeed) -> Self {
        match preset {
            BrowserPersonaPreset::ChromeStable => Self::chrome_stable(seed),
            BrowserPersonaPreset::FirefoxStable => Self::firefox_stable(seed),
        }
    }

    pub fn firefox_stable(seed: PersonaSeed) -> Self {
        let mut persona = Self::chrome_stable(seed);
        persona.preset = BrowserPersonaPreset::FirefoxStable;
        persona.transport_profile = "firefox_152_macos_26.0".to_string();
        persona.browser_family = "firefox".to_string();
        persona.browser_name = "Firefox".to_string();
        persona.browser_version = "152.0".to_string();
        persona.engine_name = "gecko".to_string();
        persona.network.user_agent =
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:152.0) Gecko/20100101 Firefox/152.0"
                .to_string();
        persona.network.accept_header =
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string();
        persona.network.sec_ch_ua.clear();
        persona.network.sec_ch_ua_mobile.clear();
        persona.network.sec_ch_ua_platform.clear();
        persona.network.sec_ch_ua_full_version.clear();
        persona.network.sec_ch_ua_full_version_list.clear();
        persona.network.sec_ch_ua_arch.clear();
        persona.network.sec_ch_ua_bitness.clear();
        persona.network.sec_ch_ua_platform_version.clear();
        persona.network.sec_ch_ua_model.clear();
        persona.js.app_version = "5.0 (Macintosh)".to_string();
        persona.js.oscpu = "Intel Mac OS X 10.15".to_string();
        persona.js.hardware_concurrency = 10;
        persona.js.vendor.clear();
        persona.js.product_sub = "20100101".to_string();
        persona.graphics.webgl_vendor = "Apple".to_string();
        persona.graphics.webgl_renderer = "Apple M1, or similar".to_string();
        persona.graphics.webgl_masked_vendor = "Mozilla".to_string();
        persona.graphics.webgl_masked_renderer = "Apple M1, or similar".to_string();
        persona.webrtc.audio_codecs = firefox_audio_codecs();
        persona.webrtc.video_codecs = firefox_video_codecs();
        persona.webrtc.ice_candidate_semantics = "firefox".to_string();
        persona.webrtc.sdp_semantics = "firefox".to_string();
        persona.media.decoding_capabilities = firefox_media_decoding_capabilities();
        persona.viewport = Viewport {
            width: 1280,
            height: 956,
            device_scale_factor: 2,
        };
        persona.screen = ScreenFingerprint {
            width: 3008,
            height: 1692,
            avail_width: 3008,
            avail_height: 1616,
            avail_left: 0,
            avail_top: 0,
            left: 0,
            top: 0,
            color_depth: 30,
            pixel_depth: 30,
            device_scale_factor: 2,
            is_extended: false,
            orientation_type: "landscape-primary".to_string(),
            orientation_angle: 0,
        };
        persona.window = WindowFingerprint {
            outer_width: 1280,
            outer_height: 1040,
            screen_x: 4,
            screen_y: 30,
        };
        persona.js.do_not_track = "unspecified".to_string();
        persona.js.expose_global_privacy_control = true;
        persona.js.global_privacy_control = false;
        persona.css.moz_prefix_enabled = true;
        persona.css.webkit_prefix_enabled = false;
        persona.features.user_agent_data = Some(false);
        persona.features.device_memory = Some(false);
        persona.features.network_information = Some(false);
        persona.features.trusted_types = Some(false);
        persona.features.fetch_later = Some(false);
        persona.features.servo_internal_apis = Some(false);
        persona.features.media_tracks = Some(false);
        persona.features.origin_api = Some(false);
        persona.features.quota_exceeded_error = Some(false);
        persona.features.visibility_state_entry = Some(false);
        persona.features.scoped_custom_element_registry = Some(false);
        persona.features.page_transition_events = Some(false);
        persona.features.screen_extended = Some(false);
        persona.features.moz_window_geometry = Some(true);
        persona.features.battery = Some(false);
        persona.features.touch = Some(false);
        persona.features.service_worker = Some(true);
        persona.features.gamepad = Some(true);
        persona.js.service_worker_enabled = true;
        persona.chrome = ChromeConfig {
            exposed: Some(false),
            ..Default::default()
        };
        persona.canvas.blink_low_entropy_probe = false;
        persona.engine.family = "spidermonkey".to_string();
        persona.engine.profile = "firefox-152".to_string();
        persona.engine.version = "152".to_string();
        persona.engine.stack_format = "spidermonkey".to_string();
        persona.engine.stack_trace_limit = 0;
        persona.engine.chromium_high_resolution_time = false;
        persona.engine.error_messages.clear();
        persona.engine.builtin_sources.clear();
        persona.native_functions = NativeFunctionsConfig::default();
        persona
    }

    pub fn chrome_stable(seed: PersonaSeed) -> Self {
        let viewport = Viewport::default();
        Self {
            seed: seed.clone(),
            preset: BrowserPersonaPreset::ChromeStable,
            transport_profile: "chrome150".to_string(),
            browser_family: "chromium".to_string(),
            browser_name: "Chrome".to_string(),
            browser_version: "150.0.0.0".to_string(),
            engine_name: "blink".to_string(),
            platform_name: "macOS".to_string(),
            viewport: viewport.clone(),
            screen: ScreenFingerprint::from_viewport(&viewport),
            window: WindowFingerprint::from_viewport(&viewport),
            css: CssFingerprint {
                moz_prefix_enabled: false,
                webkit_prefix_enabled: true,
                media: CssMediaConfig::default(),
            },
            network: NetworkFingerprint {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36".to_string(),
                accept_language: "en-US,en;q=0.9".to_string(),
                accept_header: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string(),
                accept_encoding: "gzip, deflate, br, zstd".to_string(),
                sec_ch_ua: "\"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"150\", \"Google Chrome\";v=\"150\"".to_string(),
                sec_ch_ua_mobile: "?0".to_string(),
                sec_ch_ua_platform: "\"macOS\"".to_string(),
                sec_ch_ua_full_version: "\"150.0.0.0\"".to_string(),
                sec_ch_ua_full_version_list: "\"Not;A=Brand\";v=\"8.0.0.0\", \"Chromium\";v=\"150.0.0.0\", \"Google Chrome\";v=\"150.0.0.0\"".to_string(),
                sec_ch_ua_arch: "\"x86\"".to_string(),
                sec_ch_ua_bitness: "\"64\"".to_string(),
                sec_ch_ua_platform_version: "\"15.7.0\"".to_string(),
                sec_ch_ua_model: "\"\"".to_string(),
            },
            graphics: GraphicsFingerprint {
                webgl_vendor: "Intel Inc.".to_string(),
                webgl_renderer: "Intel(R) Iris(TM) Plus Graphics OpenGL Engine".to_string(),
                webgl_masked_vendor: "WebKit".to_string(),
                webgl_masked_renderer: "WebKit WebGL".to_string(),
                webgl_seed: PersonaSeed::from_stable_input(format!("{seed}:webgl")),
                webgl1: WebGlConfig::default(),
                webgl2: WebGlConfig::default(),
                webgpu_adapter_vendor: String::new(),
                webgpu_adapter_architecture: String::new(),
                webgpu_adapter_device: String::new(),
                webgpu_adapter_description: String::new(),
                webgpu_max_bind_groups_plus_vertex_buffers: 24,
            },
            js: JsFingerprint {
                app_version: "5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36".to_string(),
                oscpu: String::new(),
                platform: "MacIntel".to_string(),
                language: "en-US".to_string(),
                languages: vec!["en-US".to_string(), "en".to_string()],
                hardware_concurrency: 8,
                device_memory_gb: 8,
                max_touch_points: 0,
                connection_type: "wifi".to_string(),
                connection_rtt_ms: 100,
                connection_downlink_mbps: "10".to_string(),
                connection_effective_type: "4g".to_string(),
                connection_save_data: false,
                notification_permission: "default".to_string(),
                do_not_track: String::new(),
                expose_global_privacy_control: false,
                global_privacy_control: false,
                permissions_enabled: true,
                bluetooth_enabled: true,
                bluetooth_available: true,
                media_devices_enabled: true,
                webgpu_enabled: false,
                offscreen_canvas_enabled: true,
                service_worker_enabled: false,
                ua_platform_version: "15.7.0".to_string(),
                ua_architecture: "x86".to_string(),
                ua_bitness: "64".to_string(),
                ua_model: String::new(),
                timezone: "America/Los_Angeles".to_string(),
                vendor: "Google Inc.".to_string(),
                product_sub: "20030107".to_string(),
                pdf_viewer_enabled: true,
            },
            geo: GeoFingerprint {
                timezone: "America/Los_Angeles".to_string(),
                locale: "en-US".to_string(),
                latitude: None,
                longitude: None,
                accuracy_meters: None,
                altitude: None,
                altitude_accuracy_meters: None,
                heading_degrees: None,
                speed_meters_per_second: None,
            },
            webrtc: WebRtcFingerprint {
                ipv4: None,
                ipv6: None,
                substitute_ice_candidates: true,
                substitute_sdp: true,
                ice_candidate_semantics: "chromium".to_string(),
                sdp_semantics: "chromium".to_string(),
                audio_codecs: chrome_audio_codecs(),
                video_codecs: chrome_video_codecs(),
            },
            canvas: CanvasFingerprint {
                seed: PersonaSeed::from_stable_input(format!("{seed}:canvas")),
                noise_enabled: true,
                noise_amplitude: 1,
                blink_low_entropy_probe: true,
            },
            domrect: DomRectFingerprint {
                seed: PersonaSeed::from_stable_input(format!("{seed}:domrect")),
                enabled: true,
                quantization_steps_per_px: 64,
                fill_empty_client_rects: true,
                sampled_profile: None,
                transform_model: None,
                font_profile: None,
                subpixel_unit: 64,
                rounding: "nearest".to_string(),
                preserve_negative_zero: true,
                clamp_transforms: true,
            },
            engine: EngineFingerprint {
                enabled: true,
                family: "v8".to_string(),
                profile: "chrome-150".to_string(),
                version: "15.0".to_string(),
                stack_format: "v8".to_string(),
                stack_trace_limit: 10,
                async_stack_traces: false,
                chromium_high_resolution_time: true,
                date_now_short_interval_threshold_ms: 0,
                date_now_short_interval_scale_percent: 100,
                error_messages: BTreeMap::from([
                    (
                        "number.to_fixed.range".to_string(),
                        "toFixed() digits argument must be between 0 and 100".to_string(),
                    ),
                    ("fetch.network".to_string(), "Failed to fetch".to_string()),
                ]),
                builtin_sources: BTreeMap::from([(
                    "Array".to_string(),
                    "function Array() { [native code] }".to_string(),
                )]),
            },
            audio: AudioFingerprint {
                seed: PersonaSeed::from_stable_input(format!("{seed}:audio")),
                sample_rate: 48_000,
                output_latency_ms: 20,
                max_channel_count: 2,
                compressor_reduction: "-20.538288116455078".to_string(),
                frequency_data: vec!["-20.538288116455078".to_string(), "-160".to_string()],
                time_domain_data: vec!["0.122705061".to_string(), "-0.122705061".to_string()],
                rendered_buffer: vec!["0".to_string()],
                render_leading_silence_samples: 256,
                fake_completion_delay_ms: 0,
                native_shape: true,
                noise_enabled: true,
            },
            fonts: FontFingerprint {
                seed: PersonaSeed::from_stable_input(format!("{seed}:fonts")),
                families: vec![
                    "Arial".to_string(),
                    "Helvetica".to_string(),
                    "Times New Roman".to_string(),
                    "Courier New".to_string(),
                ],
                emoji_fallback: vec!["Apple Color Emoji".to_string()],
                text_metric_profile: None,
                spacing_seed: PersonaSeed::from_stable_input(format!("{seed}:font-spacing")),
                subpixel_antialiasing: true,
                hinting: "platform".to_string(),
            },
            media: MediaFingerprint {
                speech_voices: vec!["Samantha".to_string(), "Alex".to_string()],
                audio_inputs: 1,
                video_inputs: 1,
                audio_outputs: 1,
                media_source_supported_types: chrome_media_source_supported_types(),
                decoding_capabilities: chrome_media_decoding_capabilities(),
            },
            battery: BatteryFingerprint {
                charging: true,
                charging_time_seconds: 0,
                discharging_time_seconds: None,
                level_percent: 100,
            },
            storage: StorageFingerprint {
                quota_bytes: None,
                usage_bytes: 0,
                legacy_temporary_quota_bytes: None,
                legacy_persistent_quota_bytes: None,
            },
            features: FeaturesConfig::default(),
            plugins: chrome_pdf_plugins(),
            chrome: ChromeConfig {
                exposed: Some(true),
                enumerable: Some(true),
                runtime: Some(true),
                app: Some(true),
                load_times: Some(true),
                csi: Some(true),
                window_key_strategy: Some("native-order".to_string()),
            },
            speech: SpeechConfig::default(),
            svg: SvgConfig {
                bbox_width_factor: Some("1".to_string()),
                text_width_factor: Some("0.62".to_string()),
                baseline_factor: Some("0.78".to_string()),
                expose_geometry_methods: Some(true),
                seed: Some(PersonaSeed::from_stable_input(format!("{seed}:svg"))),
            },
            native_functions: NativeFunctionsConfig {
                native_source: Some("function () { [native code] }".to_string()),
                preserve_name: Some(true),
                preserve_length: Some(true),
                preserve_descriptors: Some(true),
                illegal_invocation_errors: Some(true),
                constructor_prototypes: Some(true),
                sanitize_stacks: Some(true),
            },
            noise: NoiseConfig {
                enabled: Some(true),
                canvas: Some(true),
                webgl: Some(true),
                audio: Some(true),
                fonts: Some(true),
                domrect: Some(true),
                svg: Some(true),
            },
        }
    }
}

fn chrome_pdf_plugins() -> PluginsConfig {
    let mime_types = vec![
        MimeTypeConfig {
            type_: "application/pdf".to_string(),
            suffixes: "pdf".to_string(),
            description: "Portable Document Format".to_string(),
        },
        MimeTypeConfig {
            type_: "text/pdf".to_string(),
            suffixes: "pdf".to_string(),
            description: "Portable Document Format".to_string(),
        },
    ];
    PluginsConfig {
        entries: [
            "PDF Viewer",
            "Chrome PDF Viewer",
            "Chromium PDF Viewer",
            "Microsoft Edge PDF Viewer",
            "WebKit built-in PDF",
        ]
        .into_iter()
        .map(|name| PluginConfig {
            name: name.to_string(),
            filename: "internal-pdf-viewer".to_string(),
            description: "Portable Document Format".to_string(),
            mime_types: mime_types.clone(),
        })
        .collect(),
        pdf_enabled: Some(true),
        block_system_entries: Some(true),
    }
}

fn chrome_audio_codecs() -> Vec<String> {
    [
        "audio/opus/48000;minptime=10;useinbandfec=1",
        "audio/red/48000",
        "audio/G722/8000",
        "audio/PCMU/8000",
        "audio/PCMA/8000",
        "audio/CN/8000",
        "audio/telephone-event/48000",
        "audio/telephone-event/8000",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn chrome_video_codecs() -> Vec<String> {
    [
        "video/VP8",
        "video/rtx",
        "video/VP9;profile-id=0",
        "video/VP9;profile-id=2",
        "video/VP9;profile-id=1",
        "video/VP9;profile-id=3",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42e01f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=4d001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=4d001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=f4001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=f4001f",
        "video/AV1;level-idx=5;profile=0;tier=0",
        "video/AV1;level-idx=5;profile=1;tier=0",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=64001f",
        "video/H264;level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=64001f",
        "video/H265;level-id=180;profile-id=1;tier-flag=0;tx-mode=SRST",
        "video/H265;level-id=180;profile-id=2;tier-flag=0;tx-mode=SRST",
        "video/red",
        "video/ulpfec",
        "video/flexfec-03;repair-window=10000000",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn firefox_audio_codecs() -> Vec<String> {
    [
        "audio/opus/48000;maxplaybackrate=48000;stereo=1;useinbandfec=1",
        "audio/G722/8000",
        "audio/PCMU/8000",
        "audio/PCMA/8000",
        "audio/telephone-event/8000;0-15",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn firefox_video_codecs() -> Vec<String> {
    [
        "video/VP8;max-fs=12288;max-fr=60",
        "video/rtx",
        "video/VP9;max-fs=12288;max-fr=60",
        "video/H264;profile-level-id=42e01f;level-asymmetry-allowed=1;packetization-mode=1",
        "video/H264;profile-level-id=42e01f;level-asymmetry-allowed=1",
        "video/H264;profile-level-id=42001f;level-asymmetry-allowed=1;packetization-mode=1",
        "video/H264;profile-level-id=42001f;level-asymmetry-allowed=1",
        "video/AV1",
        "video/ulpfec",
        "video/red",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn chrome_media_source_supported_types() -> Vec<String> {
    [
        "audio/mp4; codecs=\"mp4a.40.2\"",
        "audio/mp4; codecs=\"opus\"",
        "audio/webm; codecs=\"opus\"",
        "audio/webm; codecs=\"vorbis\"",
        "video/mp4; codecs=\"avc1.42E01E\"",
        "video/mp4; codecs=\"avc1.4D401E\"",
        "video/mp4; codecs=\"avc1.64001E\"",
        "video/mp4; codecs=\"hev1.1.6.L93.B0\"",
        "video/mp4; codecs=\"av01.0.01M.08\"",
        "video/mp4; codecs=\"vp09.00.10.08\"",
        "video/webm; codecs=\"vp8\"",
        "video/webm; codecs=\"vp09.00.10.08\"",
        "video/webm; codecs=\"av01.0.01M.08\"",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn chrome_media_decoding_capabilities() -> Vec<MediaDecodingCapability> {
    [
        "audio/mp4; codecs=\"mp4a.40.2\"",
        "audio/mp4; codecs=\"opus\"",
        "audio/webm; codecs=\"opus\"",
        "audio/webm; codecs=\"vorbis\"",
        "audio/ogg; codecs=\"vorbis\"",
        "audio/ogg; codecs=\"flac\"",
        "video/mp4; codecs=\"avc1.42E01E\"",
        "video/mp4; codecs=\"avc1.4D401E\"",
        "video/mp4; codecs=\"avc1.64001E\"",
        "video/mp4; codecs=\"hev1.1.6.L93.B0\"",
        "video/mp4; codecs=\"av01.0.01M.08\"",
        "video/mp4; codecs=\"vp09.00.10.08\"",
        "video/webm; codecs=\"vp8\"",
        "video/webm; codecs=\"vp09.00.10.08\"",
        "video/webm; codecs=\"av01.0.01M.08\"",
    ]
    .into_iter()
    .map(|content_type| MediaDecodingCapability {
        content_type: content_type.to_string(),
        supported: true,
        smooth: true,
        power_efficient: content_type != "video/webm; codecs=\"vp8\"",
    })
    .collect()
}

fn firefox_media_decoding_capabilities() -> Vec<MediaDecodingCapability> {
    chrome_media_decoding_capabilities()
        .into_iter()
        .map(|mut capability| {
            capability.power_efficient = true;
            capability
        })
        .collect()
}
