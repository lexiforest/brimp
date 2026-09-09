use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseHeaders {
    pub user_agent: String,
    pub accept: String,
    pub accept_language: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenConfig {
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
    pub is_extended: bool,
    pub orientation_type: String,
    pub orientation_angle: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    pub outer_width: u32,
    pub outer_height: u32,
    pub screen_x: i32,
    pub screen_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigatorConfig {
    pub platform: String,
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
    pub offscreen_canvas_enabled: bool,
    pub vendor: String,
    pub product_sub: String,
    pub pdf_viewer_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanvasConfig {
    pub noise_enabled: bool,
    pub noise_amplitude: u8,
}
