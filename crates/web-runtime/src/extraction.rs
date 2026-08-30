use std::collections::BTreeMap;

use jsc::{JsRuntime, ProtectedJsObject};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFUDDLE_BUNDLE: &str = include_str!("../vendor/defuddle/0.19.3/index.full.js");
const INSTALL_EXTRACTOR: &str = include_str!("extraction/install.js");

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

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("JavaScript exception: {0}")]
    JavaScript(String),
    #[error("invalid Defuddle result: {0}")]
    InvalidResult(String),
}

pub(crate) fn install(runtime: &JsRuntime) -> Result<ProtectedJsObject, ExtractionError> {
    let mut source = String::with_capacity(DEFUDDLE_BUNDLE.len() + INSTALL_EXTRACTOR.len() + 2);
    source.push_str(DEFUDDLE_BUNDLE);
    source.push_str(";\n");
    source.push_str(INSTALL_EXTRACTOR);
    runtime
        .eval(&source)
        .and_then(|value| value.to_object())
        .map_err(|error| ExtractionError::JavaScript(error.to_string()))
}

pub(crate) fn extract(
    runtime: &JsRuntime,
    extractor: &ProtectedJsObject,
    options: &ExtractionOptions,
    url: Option<&str>,
) -> Result<ExtractedDocument, ExtractionError> {
    let mut value = serde_json::to_value(options)
        .map_err(|error| ExtractionError::InvalidResult(error.to_string()))?;
    let object = value
        .as_object_mut()
        .expect("serialized extraction options must be an object");
    object.insert("separateMarkdown".into(), Value::Bool(true));
    object.insert("useAsync".into(), Value::Bool(false));
    if let Some(url) = url {
        object.insert("url".into(), Value::String(url.to_owned()));
    }
    let options = serde_json::to_string(&value)
        .map_err(|error| ExtractionError::InvalidResult(error.to_string()))?;
    let json = runtime
        .call_function_with_string(extractor, &options)
        .and_then(|value| value.to_string())
        .map_err(|error| ExtractionError::JavaScript(error.to_string()))?;
    serde_json::from_str(&json).map_err(|error| ExtractionError::InvalidResult(error.to_string()))
}
