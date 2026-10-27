use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("failed to read browser profile: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse browser profile: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid browser profile value: {0}")]
    InvalidValue(String),
}
