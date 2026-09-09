use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

pub const DEFUDDLE_BUNDLE: &str = include_str!("../defuddle/0.19.3/index.full.js");
pub const INSTALL_EXTRACTOR: &str = include_str!("install.js");

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum AutomationError {
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
    #[error("screenshot failure: {0}")]
    Screenshot(String),
    #[error("extraction failure: {0}")]
    Extraction(String),
    #[error("runtime failure: {0}")]
    Internal(String),
}

impl AutomationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
            Self::Transport(_) => "transport",
            Self::HttpStatus(_) => "http_status",
            Self::Navigation(_) => "navigation",
            Self::JavaScript(_) => "javascript",
            Self::Timeout(_) => "timeout",
            Self::Cancellation => "cancelled",
            Self::Unsupported(_) => "unsupported",
            Self::Closed => "closed",
            Self::Screenshot(_) => "screenshot",
            Self::Extraction(_) => "extraction",
            Self::Internal(_) => "internal",
        }
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.0.clone()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_selector: Option<String>,
    pub remove_images: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub debug: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedDocument {
    pub title: String,
    pub description: String,
    pub domain: String,
    pub favicon: String,
    pub image: String,
    pub language: String,
    pub parse_time: f64,
    pub published: String,
    pub author: String,
    pub site: String,
    pub schema_org_data: Value,
    pub word_count: u64,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extractor_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_tags: Option<Vec<MetaTagItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<BTreeMap<String, f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetaTagItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInfo {
    pub content_selector: String,
    pub removals: Vec<DebugRemoval>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DebugRemoval {
    pub step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub text: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerConfig {
    pub persona: Option<brimp_persona::PersonaConfig>,
    pub proxy: Option<String>,
    pub ca_bundle: Option<PathBuf>,
    pub headers: Vec<(String, String)>,
    pub worker: bool,
    pub streaming_networking: bool,
    pub canvas: bool,
    pub storage_path: Option<PathBuf>,
    pub storage_quota: Option<u64>,
    pub navigation_timeout_ms: Option<u64>,
}

impl WorkerConfig {
    pub fn requires_lite(&self) -> bool {
        self.persona.is_some()
            || self.proxy.is_some()
            || self.ca_bundle.is_some()
            || self.worker
            || self.streaming_networking
            || self.canvas
            || self.storage_path.is_some()
    }
}
