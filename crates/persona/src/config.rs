use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::error::PersonaConfigError;
use crate::fingerprint::*;
use crate::options::*;
use crate::seed::{BrowserPersonaPreset, PersonaSeed};

/// Default persona JSON path used when `BRIMP_PERSONA_JSON` is unset.
pub const DEFAULT_PERSONA_JSON: &str = "persona/example.json";
/// Environment variable that overrides the persona JSON path.
pub const PERSONA_JSON_ENV: &str = "BRIMP_PERSONA_JSON";
/// Location of the impersonate fingerprint catalog below the user's home directory.
pub const IMPERSONATE_FINGERPRINTS_JSON: &str = ".config/impersonate/fingerprints.json";
/// Environment variable that overrides the impersonate configuration directory.
pub const IMPERSONATE_CONFIG_DIR_ENV: &str = "IMPERSONATE_CONFIG_DIR";
/// Current version of the persona configuration schema.
pub const PERSONA_SCHEMA_VERSION: u32 = 1;

/// User-provided persona configuration loaded from JSON.
///
/// Unspecified fields inherit the selected preset. Call [`Self::resolve`] to
/// turn optional overrides into a complete [`ResolvedPersona`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonaConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub preset: BrowserPersonaPreset,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<FeaturesConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<PersonaSeed>,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen: Option<ScreenConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphics: Option<GraphicsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugins: Option<PluginsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chrome: Option<ChromeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigator: Option<NavigatorConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css: Option<CssConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech: Option<SpeechConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<GeolocationConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webrtc: Option<WebRtcConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<FontConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<CanvasConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domrect: Option<DomRectConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<EngineConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub svg: Option<SvgConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_functions: Option<NativeFunctionsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise: Option<NoiseConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept_language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self {
            schema_version: PERSONA_SCHEMA_VERSION,
            preset: BrowserPersonaPreset::ChromeStable,
            identity: None,
            transport: None,
            network: None,
            features: None,
            seed: None,
            viewport: Viewport::default(),
            screen: None,
            window: None,
            graphics: None,
            plugins: None,
            chrome: None,
            navigator: None,
            css: None,
            media: None,
            audio: None,
            speech: None,
            geolocation: None,
            webrtc: None,
            battery: None,
            storage: None,
            fonts: None,
            canvas: None,
            domrect: None,
            engine: None,
            svg: None,
            native_functions: None,
            noise: None,
            locale: None,
            languages: None,
            accept_language: None,
            timezone: None,
        }
    }
}

impl PersonaConfig {
    /// Loads persona configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, PersonaConfigError> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads persona configuration from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, PersonaConfigError> {
        let contents = fs::read_to_string(path)?;
        Self::from_json(&contents)
    }

    /// Returns the default persona JSON path for this process.
    pub fn default_json_path() -> PathBuf {
        env::var_os(PERSONA_JSON_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_PERSONA_JSON))
    }

    /// Loads the default persona JSON file if it exists.
    pub fn load_default_json() -> Result<Option<Self>, PersonaConfigError> {
        let path = Self::default_json_path();
        if !path.exists() {
            return Ok(None);
        }
        Self::from_json_file(path).map(Some)
    }

    /// Returns the default impersonate fingerprint catalog path.
    pub fn impersonate_fingerprints_path() -> Result<PathBuf, PersonaConfigError> {
        if let Some(directory) = env::var_os(IMPERSONATE_CONFIG_DIR_ENV) {
            return Ok(PathBuf::from(directory).join("fingerprints.json"));
        }
        if let Some(directory) = env::var_os("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(directory).join("impersonate/fingerprints.json"));
        }
        let home = env::var_os("HOME").ok_or(PersonaConfigError::MissingHomeDirectory)?;
        Ok(PathBuf::from(home).join(IMPERSONATE_FINGERPRINTS_JSON))
    }

    /// Verifies that a transport profile is present in the default catalog.
    pub fn validate_transport_profile(profile: &str) -> Result<(), PersonaConfigError> {
        Self::validate_transport_profile_at(profile, Self::impersonate_fingerprints_path()?)
    }

    /// Verifies that a transport profile is present in the catalog at `path`.
    pub fn validate_transport_profile_at(
        profile: &str,
        path: impl AsRef<Path>,
    ) -> Result<(), PersonaConfigError> {
        if profile.trim().is_empty() {
            return Err(PersonaConfigError::InvalidTransportProfile(
                "transport profile must not be empty".to_string(),
            ));
        }
        let contents = fs::read_to_string(path)?;
        let catalog: serde_json::Value = serde_json::from_str(&contents)?;
        let Some(entries) = catalog.as_object() else {
            return Err(PersonaConfigError::InvalidFingerprintCatalog);
        };
        let alias_prefix = ["chrome", "firefox"].into_iter().find_map(|family| {
            profile
                .strip_prefix(family)
                .filter(|version| {
                    !version.is_empty() && version.chars().all(|char| char.is_ascii_digit())
                })
                .map(|version| format!("{family}_{version}_"))
        });
        let built_in_alias = ["chrome", "firefox"].into_iter().any(|family| {
            profile.strip_prefix(family).is_some_and(|version| {
                !version.is_empty() && version.chars().all(|char| char.is_ascii_digit())
            })
        });
        if entries.contains_key(profile)
            || built_in_alias
            || alias_prefix
                .as_deref()
                .is_some_and(|prefix| entries.keys().any(|key| key.starts_with(prefix)))
        {
            Ok(())
        } else {
            Err(PersonaConfigError::TransportProfileNotFound(
                profile.to_string(),
            ))
        }
    }

    /// Resolves this partial configuration into complete fingerprint values.
    pub fn resolve(&self) -> ResolvedPersona {
        let seed = self.seed.clone().unwrap_or_else(|| {
            PersonaSeed::from_stable_input(format!(
                "preset:{:?}:{}x{}@{}",
                self.preset,
                self.viewport.width,
                self.viewport.height,
                self.viewport.device_scale_factor
            ))
        });
        let mut persona = ResolvedPersona::from_preset_and_seed(self.preset, seed);
        if let Some(identity) = &self.identity {
            identity.apply_to(&mut persona);
        }
        if let Some(transport) = &self.transport {
            transport.apply_to(&mut persona);
        }
        persona.viewport = self.viewport.clone();
        persona.screen = ScreenFingerprint::from_viewport(&persona.viewport);
        if let Some(screen) = &self.screen {
            screen.apply_to(&mut persona.screen);
        }
        persona.window = WindowFingerprint::from_viewport(&persona.viewport);
        if let Some(window) = &self.window {
            window.apply_to(&mut persona.window);
        }
        if let Some(graphics) = &self.graphics {
            graphics.apply_to(&mut persona.graphics);
        }
        if let Some(features) = &self.features {
            features.apply_to(&mut persona.features);
        }
        if let Some(plugins) = &self.plugins {
            persona.plugins = plugins.clone();
        }
        if let Some(chrome) = &self.chrome {
            chrome.apply_to(&mut persona.chrome);
        }
        self.apply_locale_timezone(&mut persona);
        if let Some(navigator) = &self.navigator {
            navigator.apply_to(&mut persona.js);
            if self.accept_language.is_none() && !persona.js.languages.is_empty() {
                persona.network.accept_language = format_accept_language(&persona.js.languages);
            }
        }
        if let Some(pdf_enabled) = self
            .plugins
            .as_ref()
            .and_then(|plugins| plugins.pdf_enabled)
        {
            persona.js.pdf_viewer_enabled = pdf_enabled;
        }
        if let Some(css) = &self.css {
            css.apply_to(&mut persona.css);
        }
        if let Some(media) = &self.media {
            media.apply_to(&mut persona.media);
        }
        if let Some(audio) = &self.audio {
            audio.apply_to(&mut persona.audio);
        }
        if let Some(speech) = &self.speech {
            persona.speech = speech.clone();
        }
        if let Some(geolocation) = &self.geolocation {
            geolocation.apply_to(&mut persona.geo);
        }
        if let Some(webrtc) = &self.webrtc {
            webrtc.apply_to(&mut persona.webrtc);
        }
        if let Some(battery) = &self.battery {
            battery.apply_to(&mut persona.battery);
        }
        if let Some(storage) = &self.storage {
            storage.apply_to(&mut persona.storage);
        }
        if let Some(fonts) = &self.fonts {
            fonts.apply_to(&mut persona.fonts);
        }
        if let Some(canvas) = &self.canvas {
            canvas.apply_to(&mut persona.canvas);
        }
        if let Some(domrect) = &self.domrect {
            domrect.apply_to(&mut persona.domrect);
        }
        if let Some(engine) = &self.engine {
            engine.apply_to(&mut persona.engine);
        }
        if let Some(svg) = &self.svg {
            svg.apply_to(&mut persona.svg);
        }
        if let Some(native_functions) = &self.native_functions {
            persona.native_functions = native_functions.clone();
        }
        if let Some(noise) = &self.noise {
            persona.noise = noise.clone();
            noise.apply_to(&mut persona);
        }
        // Explicit network values win over locale-derived defaults.
        if let Some(network) = &self.network {
            network.apply_to(&mut persona.network);
        }
        persona
    }

    /// Validates schema-level invariants that cannot be expressed by Serde.
    pub fn validate(&self) -> Result<(), PersonaConfigError> {
        if self.schema_version != PERSONA_SCHEMA_VERSION {
            return Err(PersonaConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: PERSONA_SCHEMA_VERSION,
            });
        }
        if self.viewport.width == 0
            || self.viewport.height == 0
            || self.viewport.device_scale_factor == 0
        {
            return Err(PersonaConfigError::InvalidViewport);
        }
        if let Some(screen) = &self.screen {
            if screen.width == Some(0) || screen.height == Some(0) {
                return Err(PersonaConfigError::InvalidScreen);
            }
            let width = screen.width.unwrap_or(self.viewport.width);
            let height = screen.height.unwrap_or(self.viewport.height);
            if screen.avail_width.is_some_and(|value| value > width)
                || screen.avail_height.is_some_and(|value| value > height)
            {
                return Err(PersonaConfigError::InvalidScreen);
            }
        }
        if let Some(network) = &self.network
            && network
                .accept_encoding
                .as_deref()
                .is_some_and(|value| value.is_empty() || value.chars().any(char::is_control))
        {
            return Err(PersonaConfigError::InvalidNativeProfile(
                "network.accept_encoding must be a non-empty HTTP header value".to_string(),
            ));
        }
        if let Some(navigator) = &self.navigator {
            if navigator
                .connection_downlink_mbps
                .as_deref()
                .is_some_and(|value| {
                    value
                        .parse::<f64>()
                        .map_or(true, |number| !number.is_finite() || number < 0.0)
                })
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "navigator.connection_downlink_mbps must be a non-negative decimal string"
                        .to_string(),
                ));
            }
            if navigator
                .connection_effective_type
                .as_deref()
                .is_some_and(|value| !matches!(value, "slow-2g" | "2g" | "3g" | "4g"))
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "navigator.connection_effective_type must be slow-2g, 2g, 3g, or 4g"
                        .to_string(),
                ));
            }
            if navigator
                .notification_permission
                .as_deref()
                .is_some_and(|value| !matches!(value, "default" | "denied" | "granted"))
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "navigator.notification_permission must be default, denied, or granted"
                        .to_string(),
                ));
            }
        }
        if let Some(plugins) = &self.plugins {
            if plugins
                .entries
                .iter()
                .any(|plugin| plugin.name.trim().is_empty())
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "plugin names must not be empty".to_string(),
                ));
            }
            if plugins
                .entries
                .iter()
                .flat_map(|plugin| &plugin.mime_types)
                .any(|mime| mime.type_.trim().is_empty())
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "plugin MIME types must not be empty".to_string(),
                ));
            }
        }
        if let Some(audio) = &self.audio {
            if audio.sample_rate == Some(0) || audio.max_channel_count == Some(0) {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "audio sample_rate and max_channel_count must be positive".to_string(),
                ));
            }
            let values = audio
                .compressor_reduction
                .iter()
                .chain(&audio.frequency_data)
                .chain(&audio.time_domain_data)
                .chain(&audio.rendered_buffer);
            if values
                .into_iter()
                .any(|value| value.parse::<f64>().is_err())
            {
                return Err(PersonaConfigError::InvalidNativeProfile(
                    "audio numeric profile values must be decimal strings".to_string(),
                ));
            }
        }
        if let Some(geolocation) = &self.geolocation {
            validate_coordinate("latitude", geolocation.latitude.as_deref(), -90.0, 90.0)?;
            validate_coordinate("longitude", geolocation.longitude.as_deref(), -180.0, 180.0)?;
        }
        if let Some(webrtc) = &self.webrtc {
            for (name, value) in [
                (
                    "ice_candidate_semantics",
                    webrtc.ice_candidate_semantics.as_deref(),
                ),
                ("sdp_semantics", webrtc.sdp_semantics.as_deref()),
            ] {
                if value.is_some_and(|value| !matches!(value, "chromium" | "firefox")) {
                    return Err(PersonaConfigError::InvalidNativeProfile(format!(
                        "webrtc.{name} must be chromium or firefox"
                    )));
                }
            }
        }
        if self
            .battery
            .as_ref()
            .and_then(|battery| battery.level_percent)
            .is_some_and(|level| level > 100)
        {
            return Err(PersonaConfigError::InvalidNativeProfile(
                "battery.level_percent must be between 0 and 100".to_string(),
            ));
        }
        if let Some(storage) = &self.storage
            && storage
                .quota_bytes
                .zip(storage.usage_bytes)
                .is_some_and(|(quota, usage)| usage > quota)
        {
            return Err(PersonaConfigError::InvalidNativeProfile(
                "storage.usage_bytes must not exceed storage.quota_bytes".to_string(),
            ));
        }
        Ok(())
    }

    fn apply_locale_timezone(&self, persona: &mut ResolvedPersona) {
        let mut languages = self
            .languages
            .clone()
            .unwrap_or_else(|| parse_accept_language(self.accept_language.as_deref()));
        if languages.is_empty()
            && let Some(locale) = self.locale.as_ref().filter(|value| !value.is_empty())
        {
            languages.push(locale.clone());
            if let Some(primary) = primary_language_subtag(locale)
                && primary != *locale
            {
                languages.push(primary);
            }
        }

        if let Some(locale) = self.locale.as_ref().filter(|value| !value.is_empty()) {
            persona.js.language = locale.clone();
            persona.geo.locale = locale.clone();
        } else if let Some(locale) = languages.first() {
            persona.js.language = locale.clone();
            persona.geo.locale = locale.clone();
        }

        if !languages.is_empty() {
            persona.js.languages = languages.clone();
        }

        if let Some(accept_language) = self
            .accept_language
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            persona.network.accept_language = accept_language.clone();
        } else if !persona.js.languages.is_empty() {
            persona.network.accept_language = format_accept_language(&persona.js.languages);
        }

        if let Some(timezone) = self.timezone.as_ref().filter(|value| !value.is_empty()) {
            persona.js.timezone = timezone.clone();
            persona.geo.timezone = timezone.clone();
        }
    }
}

fn validate_coordinate(
    name: &str,
    value: Option<&str>,
    minimum: f64,
    maximum: f64,
) -> Result<(), PersonaConfigError> {
    let Some(value) = value else {
        return Ok(());
    };
    let parsed = value.parse::<f64>().map_err(|_| {
        PersonaConfigError::InvalidNativeProfile(format!(
            "geolocation.{name} must be a decimal string"
        ))
    })?;
    if !parsed.is_finite() || !(minimum..=maximum).contains(&parsed) {
        return Err(PersonaConfigError::InvalidNativeProfile(format!(
            "geolocation.{name} must be between {minimum} and {maximum}"
        )));
    }
    Ok(())
}

fn primary_language_subtag(locale: &str) -> Option<String> {
    locale
        .split(['-', '_'])
        .next()
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_accept_language(header: Option<&str>) -> Vec<String> {
    header
        .unwrap_or_default()
        .split(',')
        .filter_map(|entry| {
            entry
                .split(';')
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .collect()
}

fn format_accept_language(languages: &[String]) -> String {
    languages
        .iter()
        .enumerate()
        .map(|(index, language)| {
            if index == 0 {
                language.clone()
            } else {
                let quality = (10usize.saturating_sub(index)).max(1);
                format!("{language};q=0.{quality}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}
