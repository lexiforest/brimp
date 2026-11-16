use thiserror::Error as ThisError;

/// Error type returned by `brimp-net` operations.
#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// The native curl handle or impersonation target could not be initialized.
    #[error("curl initialization failed: {0}")]
    Initialization(String),
    /// A native easy-handle failure, retaining its CURLcode.
    #[error("network error: {message}")]
    Transfer {
        /// Native CURLcode.
        code: u32,
        /// Human-readable transport diagnostic.
        message: String,
        /// A proxy rejected the HTTP CONNECT tunnel.
        proxy_connect: bool,
        /// Destination or HTTPS proxy certificate verification failed.
        certificate_verify: bool,
    },
    /// The requested impersonation configuration could not be applied.
    #[error("impersonation failed: {message}")]
    Impersonation {
        /// Native CURLcode, or zero for profile validation failures.
        code: u32,
        /// Profile configuration diagnostic.
        message: String,
    },
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

impl Error {
    pub(crate) fn impersonation(message: String) -> Self {
        Self::Impersonation { code: 0, message }
    }
}
