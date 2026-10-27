//! Host identity, session and request configuration.

use crate::network::CurlConfig;
use crate::runtime::{AutomationError, CancellationToken};
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct IdentityOptions {
    pub user_agent: Option<String>,
    pub languages: Option<Vec<String>>,
    pub platform: Option<String>,
}
impl IdentityOptions {
    pub(crate) fn apply(
        &self,
        profile: &mut crate::profile::BrowserProfile,
    ) -> Result<(), AutomationError> {
        if let Some(value) = &self.user_agent {
            profile.set("identity", "user_agent", json!(value));
            profile.set(
                "identity",
                "app_version",
                json!(value.strip_prefix("Mozilla/").unwrap_or(value)),
            );
        }
        if let Some(values) = &self.languages {
            if values.is_empty()
                || values
                    .iter()
                    .any(|value| value.trim().is_empty() || value.chars().any(char::is_control))
            {
                return Err(AutomationError::InvalidInput(
                    "languages must be nonempty header-safe strings".into(),
                ));
            }
            profile.set("identity", "languages", json!(values));
            profile.set("identity", "locale", json!(values[0]));
        }
        if let Some(value) = &self.platform {
            profile.set("identity", "platform", json!(value));
        }
        profile
            .validate()
            .map_err(|error| AutomationError::InvalidInput(error.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct SessionOptions {
    pub profile: Option<crate::profile::BrowserProfile>,
    /// Optional file for origin localStorage, CacheStorage and IndexedDB.
    pub storage_path: Option<std::path::PathBuf>,
    /// Optional wall-clock deadline shared by every operation in this session.
    pub deadline: Option<Instant>,
    pub cancellation: CancellationToken,
    pub wait_until: crate::runtime::NavigationWait,
    pub clock: crate::runtime::clock::ClockOptions,
    pub diagnostics: Option<crate::runtime::diagnostics::DiagnosticsOptions>,
    pub identity: IdentityOptions,
    pub headers: Vec<(String, String)>,
    pub timeout: Duration,
    pub network: CurlConfig,
    pub plugins: Vec<crate::runtime::plugins::Plugin>,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            profile: None,
            storage_path: None,
            deadline: None,
            cancellation: CancellationToken::new(),
            wait_until: Default::default(),
            clock: Default::default(),
            diagnostics: None,
            identity: IdentityOptions::default(),
            headers: Vec::new(),
            timeout: Duration::from_secs(30),
            network: CurlConfig::default(),
            plugins: crate::runtime::plugins::bundled::browser(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RequestOptions {
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
    pub allow_redirects: bool,
    pub max_redirects: usize,
    pub cancellation: CancellationToken,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            body: None,
            timeout: None,
            allow_redirects: true,
            max_redirects: 20,
            cancellation: CancellationToken::new(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub name: String,
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub last_modified: u64,
}

pub(super) fn validate_timeout(timeout: Duration) -> Result<(), AutomationError> {
    if timeout.is_zero() {
        return Err(AutomationError::InvalidInput(
            "timeout must be positive".into(),
        ));
    }
    Ok(())
}
