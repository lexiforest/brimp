use thiserror::Error as ThisError;

/// Error type returned by `brimp-net` operations.
#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// The native curl handle or impersonation target could not be initialized.
    #[error("curl initialization failed: {0}")]
    Initialization(String),
    /// The request could not be represented as a curl transfer.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// The transfer failed while communicating with the network.
    #[error("network error: {0}")]
    Network(String),
    /// The response status, headers, or body could not be constructed.
    #[error("response error: {0}")]
    Response(String),
}

/// Result alias using [`Error`] as the default error type.
pub type Result<T> = std::result::Result<T, Error>;
