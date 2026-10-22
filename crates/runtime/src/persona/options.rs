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
