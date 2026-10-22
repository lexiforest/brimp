use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use super::{PersonaConfigError, options::*};

/// Complete browser persona. The bundled example defines the supported fields
/// and supplies the defaults; JSON input must provide a complete configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonaConfig {
    pub network_profile: String,
    pub base_headers: BaseHeaders,
    pub platform: String,
    pub locale: String,
    pub languages: Vec<String>,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self::from_json(include_str!("example.json")).expect("bundled persona must be valid")
    }
}

impl PersonaConfig {
    pub fn from_json(json: &str) -> Result<Self, PersonaConfigError> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, PersonaConfigError> {
        Self::from_json(&fs::read_to_string(path)?)
    }

    pub fn validate(&self) -> Result<(), PersonaConfigError> {
        let invalid = |message: &str| PersonaConfigError::InvalidValue(message.to_owned());
        if self.network_profile.trim().is_empty()
            || self.network_profile.chars().any(char::is_control)
        {
            return Err(invalid("network_profile must be a non-empty profile name"));
        }
        for (name, value) in self.base_headers.headers() {
            if value.chars().any(char::is_control) {
                return Err(invalid(&format!(
                    "base_headers.{name} must be an HTTP header value"
                )));
            }
        }
        if self.base_headers.user_agent.is_empty() || self.base_headers.accept_encoding.is_empty() {
            return Err(invalid(
                "base_headers.user_agent and accept_encoding must not be empty",
            ));
        }
        if self.locale.trim().is_empty()
            || self.languages.is_empty()
            || self
                .languages
                .iter()
                .any(|language| language.trim().is_empty())
        {
            return Err(invalid("locale and languages must not be empty"));
        }
        Ok(())
    }
}

impl BaseHeaders {
    /// HTTP names and values in the persona's declared order. Empty client hints
    /// are omitted by the request layer.
    pub(crate) fn headers(&self) -> [(&'static str, &str); 13] {
        [
            ("user-agent", self.user_agent.as_str()),
            ("accept", self.accept.as_str()),
            ("accept-language", self.accept_language.as_str()),
            ("accept-encoding", self.accept_encoding.as_str()),
            ("sec-ch-ua", self.sec_ch_ua.as_str()),
            ("sec-ch-ua-mobile", self.sec_ch_ua_mobile.as_str()),
            ("sec-ch-ua-platform", self.sec_ch_ua_platform.as_str()),
            (
                "sec-ch-ua-full-version",
                self.sec_ch_ua_full_version.as_str(),
            ),
            (
                "sec-ch-ua-full-version-list",
                self.sec_ch_ua_full_version_list.as_str(),
            ),
            ("sec-ch-ua-arch", self.sec_ch_ua_arch.as_str()),
            ("sec-ch-ua-bitness", self.sec_ch_ua_bitness.as_str()),
            (
                "sec-ch-ua-platform-version",
                self.sec_ch_ua_platform_version.as_str(),
            ),
            ("sec-ch-ua-model", self.sec_ch_ua_model.as_str()),
        ]
    }
}
