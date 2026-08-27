//! Browser persona configuration and resolved fingerprint values.
//!
//! `persona` turns a versioned JSON configuration into concrete browser,
//! network, JavaScript, screen, graphics, media, and automation fingerprint
//! values consumed by the Brimp runtime.

mod config;
mod error;
mod fingerprint;
mod options;
mod seed;

pub use config::{
    DEFAULT_PERSONA_JSON, IMPERSONATE_FINGERPRINTS_JSON, PERSONA_JSON_ENV, PERSONA_SCHEMA_VERSION,
    PersonaConfig,
};
pub use error::PersonaConfigError;
pub use fingerprint::*;
pub use options::*;
pub use seed::{BrowserPersonaPreset, PersonaSeed};

#[cfg(test)]
mod tests;
