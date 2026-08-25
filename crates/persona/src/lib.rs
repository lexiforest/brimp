//! Coherent identity schema for surfaces Brimp currently implements.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Persona {
    pub transport_profile: String,
    pub user_agent: String,
    pub platform: String,
    pub locale: String,
    pub languages: Vec<String>,
    pub viewport: ViewportIdentity,
    pub seed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ViewportIdentity {
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f64,
}

impl Persona {
    pub fn from_json(json: &str) -> Result<Self, PersonaError> {
        let persona: Self =
            serde_json::from_str(json).map_err(|error| PersonaError::Schema(error.to_string()))?;
        persona.validate()?;
        Ok(persona)
    }

    pub fn validate(&self) -> Result<(), PersonaError> {
        if self.transport_profile.is_empty()
            || self.user_agent.is_empty()
            || self.platform.is_empty()
            || self.locale.is_empty()
        {
            return Err(PersonaError::Invalid(
                "identity strings must not be empty".into(),
            ));
        }
        if self.languages.first().map(String::as_str) != Some(self.locale.as_str()) {
            return Err(PersonaError::Invalid(
                "languages must begin with locale".into(),
            ));
        }
        if self.viewport.width == 0
            || self.viewport.height == 0
            || !self.viewport.device_pixel_ratio.is_finite()
            || self.viewport.device_pixel_ratio <= 0.0
        {
            return Err(PersonaError::Invalid(
                "viewport and device pixel ratio must be positive".into(),
            ));
        }
        if self.transport_profile.starts_with("chrome") && !self.user_agent.contains("Chrome/") {
            return Err(PersonaError::Invalid(
                "Chrome transport profile requires a Chrome User-Agent".into(),
            ));
        }
        Ok(())
    }
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            transport_profile: "chrome136".into(),
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36".into(),
            platform: "MacIntel".into(),
            locale: "en-US".into(),
            languages: vec!["en-US".into(), "en".into()],
            viewport: ViewportIdentity { width: 800, height: 600, device_pixel_ratio: 1.0 },
            seed: 0,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PersonaError {
    #[error("invalid persona schema: {0}")]
    Schema(String),
    #[error("incoherent persona: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_and_unsupported_fields_are_rejected() {
        let mut value = serde_json::to_value(Persona::default()).unwrap();
        value["webgl_vendor"] = "pretend".into();
        assert!(matches!(
            Persona::from_json(&value.to_string()),
            Err(PersonaError::Schema(_))
        ));
    }

    #[test]
    fn same_schema_and_seed_resolve_identically() {
        let json = serde_json::to_string(&Persona::default()).unwrap();
        assert_eq!(
            Persona::from_json(&json).unwrap(),
            Persona::from_json(&json).unwrap()
        );
    }
}
