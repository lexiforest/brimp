use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum AutomationError {
    #[error("host callbacks cannot reenter the same page")]
    Reentry,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("transport failure: {0}")]
    Transport(String),
    #[error("HTTP response status {0}")]
    HttpStatus(u16),
    #[error("navigation failure: {0}")]
    Navigation(String),
    #[error("JavaScript exception: {0}")]
    JavaScript(String),
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    #[error("operation was cancelled")]
    Cancellation,
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    #[error("object is closed")]
    Closed,
    #[error("handle belongs to an expired realm or was disposed")]
    StaleHandle,
    #[error("extraction failure: {0}")]
    Extraction(String),
    #[error("runtime failure: {0}")]
    Internal(String),
}

impl AutomationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Reentry => "reentry",
            Self::InvalidInput(_) => "invalid_input",
            Self::Transport(_) => "transport",
            Self::HttpStatus(_) => "http_status",
            Self::Navigation(_) => "navigation",
            Self::JavaScript(_) => "javascript",
            Self::Timeout(_) => "timeout",
            Self::Cancellation => "cancelled",
            Self::Unsupported(_) => "unsupported",
            Self::Closed => "closed",
            Self::StaleHandle => "stale_handle",
            Self::Extraction(_) => "extraction",
            Self::Internal(_) => "internal",
        }
    }
}
