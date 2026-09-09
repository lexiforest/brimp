use std::io;

#[derive(Debug, thiserror::Error)]
pub enum PersonaConfigError {
    #[error("failed to read persona config: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse persona config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported persona schema version {found}; this build supports version {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("viewport width, height, and device scale factor must be greater than zero")]
    InvalidViewport,
    #[error("screen dimensions must be positive and available dimensions must not exceed them")]
    InvalidScreen,
    #[error("invalid persona value: {0}")]
    InvalidValue(String),
}
