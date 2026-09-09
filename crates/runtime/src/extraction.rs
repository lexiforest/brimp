use brimp_jsc::{JsRuntime, ProtectedJsObject};
use brimp_worker_api::{DEFUDDLE_BUNDLE, INSTALL_EXTRACTOR};
pub use brimp_worker_api::{
    DebugInfo, DebugRemoval, ExtractedDocument, ExtractionOptions, MetaTagItem,
};
use serde_json::Value;

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
