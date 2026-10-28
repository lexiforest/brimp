use brimp_jsc::JsRuntime;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::OnceLock;

use super::error::ProfileError;

pub(crate) const REGISTRY_SOURCE: &str = include_str!("../../js/internal/profile_registry.js");
pub(crate) const CORE_SOURCE: &str = include_str!("../../js/web_apis/profile.js");

/// A partial profile document. Interfaces own their defaults and validation rules.
/// Capture metadata records observations and omissions; it never controls API exposure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserProfile {
    #[serde(default = "format")]
    format: String,
    #[serde(default = "version")]
    version: u32,
    #[serde(default)]
    values: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    capture: Option<Value>,
    #[serde(skip)]
    core: OnceLock<Value>,
}
fn format() -> String {
    "brimp-profile".into()
}
fn version() -> u32 {
    1
}
impl Default for BrowserProfile {
    fn default() -> Self {
        Self {
            format: format(),
            version: version(),
            values: Map::new(),
            capture: None,
            core: OnceLock::new(),
        }
    }
}
impl BrowserProfile {
    pub fn from_json(source: &str) -> Result<Self, ProfileError> {
        let profile: Self = serde_json::from_str(source)?;
        profile.validate()?;
        Ok(profile)
    }
    pub fn values(&self) -> &Map<String, Value> {
        &self.values
    }
    pub fn capture(&self) -> Option<&Value> {
        self.capture.as_ref()
    }
    pub fn set(&mut self, scope: &str, key: &str, value: Value) {
        self.core.take();
        let values = self
            .values
            .entry(scope.to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        values
            .as_object_mut()
            .expect("profile scopes are objects")
            .insert(key.to_owned(), value);
    }
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.format != format() || self.version != version() {
            return Err(ProfileError::InvalidValue(
                "expected brimp-profile version 1".into(),
            ));
        }
        if self.values.values().any(|value| !value.is_object()) {
            return Err(ProfileError::InvalidValue(
                "profile values must map interface names to objects".into(),
            ));
        }
        let core = self.resolve_core()?;
        // These values also become HTTP headers. Other validation is interface-owned.
        for (_, value) in self.headers(core) {
            http::HeaderValue::from_str(&value).map_err(|_| {
                ProfileError::InvalidValue("profile contains an invalid HTTP header value".into())
            })?;
        }
        Ok(())
    }
    pub(crate) fn resolve_core(&self) -> Result<&Value, ProfileError> {
        if let Some(value) = self.core.get() {
            return Ok(value);
        }
        let js = JsRuntime::new().map_err(|e| ProfileError::InvalidValue(e.to_string()))?;
        let source = format!(
            "JSON.stringify((() => {{ const registry = ({REGISTRY_SOURCE})({}); return ({CORE_SOURCE})(registry.forOwner('core')); }})())",
            serde_json::to_string(self)?
        );
        let result = js
            .eval(&source)
            .and_then(|value| value.to_string())
            .map_err(|e| ProfileError::InvalidValue(e.to_string()))?;
        let mut value: Value = serde_json::from_str(&result)?;
        derive_transport_target(&mut value, &self.values);
        let _ = self.core.set(value);
        Ok(self.core.get().expect("resolved core profile"))
    }
    pub(crate) fn core(&self) -> &Value {
        self.resolve_core().expect("validated core profile")
    }
    pub(crate) fn network_profile(&self) -> &str {
        self.core()["network"]["impersonation"]
            .as_str()
            .expect("network profile string")
    }
    fn headers(&self, core: &Value) -> Vec<(&'static str, String)> {
        let text = |scope: &str, field: &str| {
            core[scope][field]
                .as_str()
                .expect("validated profile string")
                .to_owned()
        };
        let languages = core["identity"]["languages"]
            .as_array()
            .expect("validated language array");
        let language = languages
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let value = value.as_str().expect("validated language string");
                if i == 0 {
                    value.to_owned()
                } else {
                    format!("{value};q={:.1}", (1.0 - i as f64 * 0.1).max(0.1))
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        vec![
            ("user-agent", text("identity", "user_agent")),
            ("accept", text("network", "accept")),
            ("accept-language", language),
            ("accept-encoding", text("network", "accept_encoding")),
        ]
        .into_iter()
        .chain(chromium_client_hints(core))
        .collect()
    }
    pub(crate) fn request_headers(&self) -> Vec<(&'static str, String)> {
        self.headers(self.core())
    }
}

/// The transport fingerprint cannot be read from page JavaScript, so it is derived
/// from the user-agent family unless the profile sets it explicitly. A WebKit
/// default with a Chromium identity is the mismatch this avoids.
fn derive_transport_target(core: &mut Value, provided: &Map<String, Value>) {
    let explicit = provided
        .get("network")
        .and_then(|network| network.get("impersonation"))
        .is_some();
    if explicit {
        return;
    }
    let Some(user_agent) = core["identity"]["user_agent"].as_str() else {
        return;
    };
    let target = if user_agent.contains("Firefox/") {
        "firefox147"
    } else if user_agent.contains("Chrome/")
        || user_agent.contains("Chromium/")
        || user_agent.contains("Edg/")
    {
        "chrome150"
    } else if user_agent.contains("Safari/") {
        "safari270"
    } else {
        return;
    };
    core["network"]["impersonation"] = Value::String(target.to_owned());
}

/// Client hints Chromium sends for every request. Safari sends none, so they are
/// derived from the profile identity and disappear with a WebKit user agent.
fn chromium_client_hints(core: &Value) -> Vec<(&'static str, String)> {
    let Some(user_agent) = core["identity"]["user_agent"].as_str() else {
        return Vec::new();
    };
    let Some(major) = chromium_major_version(user_agent) else {
        return Vec::new();
    };
    let platform = core["identity"]["platform"].as_str().unwrap_or("");
    let mobile = user_agent.contains("Mobile") || platform == "Linux armv81";
    let platform_label = match platform {
        "MacIntel" => "macOS",
        "Win32" => "Windows",
        "iPhone" | "iPad" => "iOS",
        _ if mobile => "Android",
        _ => "Linux",
    };
    vec![
        (
            "sec-ch-ua",
            format!(
                "\"Not:A-Brand\";v=\"8\", \"Chromium\";v=\"{major}\", \"Google Chrome\";v=\"{major}\""
            ),
        ),
        (
            "sec-ch-ua-mobile",
            if mobile { "?1" } else { "?0" }.to_owned(),
        ),
        ("sec-ch-ua-platform", format!("\"{platform_label}\"")),
    ]
}

/// Major version of a Chromium user agent, or `None` for WebKit/Gecko agents.
fn chromium_major_version(user_agent: &str) -> Option<&str> {
    let start = user_agent.find("Chrome/")? + "Chrome/".len();
    let version = &user_agent[start..];
    let end = version.find('.').unwrap_or(version.len());
    let major = &version[..end];
    (!major.is_empty() && major.bytes().all(|byte| byte.is_ascii_digit())).then_some(major)
}
