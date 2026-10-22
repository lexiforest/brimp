use http::{HeaderMap, StatusCode};

/// Fully collected response returned by [`crate::Client::send_collect`].
#[derive(Debug)]
pub struct CollectedResponse {
    /// Final HTTP response status.
    pub status: StatusCode,
    /// Final HTTP response headers.
    pub headers: HeaderMap,
    /// Decoded response body bytes.
    pub body: Vec<u8>,
    /// Effective URL after redirects, if redirects were followed.
    pub effective_url: String,
}

/// Transfer details available as a response extension; byte counters finalize at EOF.
#[derive(Clone, Debug, Default)]
pub struct TransferInfo(pub(crate) std::sync::Arc<std::sync::Mutex<TransferMetadata>>);
impl TransferInfo {
    /// Snapshot the latest transfer information.
    pub fn snapshot(&self) -> TransferMetadata {
        self.0.lock().unwrap().clone()
    }
}
/// Native transfer metadata; body bytes can be decoded independently by curl.
#[derive(Clone, Debug, Default)]
pub struct TransferMetadata {
    /// Effective URL reported by curl.
    pub effective_url: String,
    /// Negotiated HTTP protocol version.
    pub http_version: Option<String>,
    /// Wire download bytes.
    pub downloaded_bytes: u64,
    /// Wire upload bytes.
    pub uploaded_bytes: u64,
    /// Received header bytes.
    pub header_bytes: u64,
}
