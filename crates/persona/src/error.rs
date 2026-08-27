use std::{fmt, io};

#[derive(Debug)]
pub enum PersonaConfigError {
    Io(io::Error),
    Json(serde_json::Error),
    MissingHomeDirectory,
    InvalidFingerprintCatalog,
    InvalidTransportProfile(String),
    TransportProfileNotFound(String),
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    InvalidViewport,
    InvalidScreen,
    InvalidNativeProfile(String),
}

impl fmt::Display for PersonaConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to read persona config: {error}"),
            Self::Json(error) => write!(formatter, "failed to parse persona config: {error}"),
            Self::MissingHomeDirectory => write!(formatter, "HOME is not set"),
            Self::InvalidFingerprintCatalog => {
                write!(
                    formatter,
                    "impersonate fingerprint catalog must be a JSON object"
                )
            }
            Self::InvalidTransportProfile(message) => {
                write!(formatter, "invalid transport profile: {message}")
            }
            Self::TransportProfileNotFound(profile) => {
                write!(
                    formatter,
                    "transport profile `{profile}` is not in the impersonate fingerprint catalog"
                )
            }
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "unsupported persona schema version {found}; this build supports version {supported}"
            ),
            Self::InvalidViewport => write!(
                formatter,
                "viewport width, height, and device scale factor must be greater than zero"
            ),
            Self::InvalidScreen => write!(
                formatter,
                "screen dimensions must be positive and available dimensions must not exceed them"
            ),
            Self::InvalidNativeProfile(message) => {
                write!(formatter, "invalid native persona profile: {message}")
            }
        }
    }
}

impl std::error::Error for PersonaConfigError {}

impl From<io::Error> for PersonaConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PersonaConfigError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
