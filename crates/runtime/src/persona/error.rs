use std::io;

#[derive(Debug, thiserror::Error)]
pub enum PersonaConfigError {
    #[error("failed to read persona config: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse persona config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid persona value: {0}")]
    InvalidValue(String),
}
