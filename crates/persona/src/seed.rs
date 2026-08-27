use serde::{Deserialize, Serialize};
use std::fmt;

/// Built-in browser persona preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserPersonaPreset {
    /// Chrome stable identity and fingerprint defaults.
    #[default]
    ChromeStable,
    /// Firefox stable identity backed by Gecko/SpiderMonkey-shaped defaults.
    FirefoxStable,
}

/// Stable seed used to derive deterministic persona sub-seeds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaSeed(String);

impl PersonaSeed {
    /// Creates a seed from caller input, normalizing empty strings.
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        if value.trim().is_empty() {
            return Self::from_stable_input("empty");
        }
        Self(value)
    }

    /// Derives a deterministic seed from stable input such as a profile path.
    pub fn from_stable_input(input: impl AsRef<str>) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in input.as_ref().as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(format!("{hash:016x}"))
    }

    /// Returns the seed as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersonaSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
