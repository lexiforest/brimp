use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Response {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProtocolError {
    pub code: i64,
    pub message: String,
}

impl Response {
    pub(crate) fn success(request: &Request, result: Value) -> Self {
        Self {
            id: request.id,
            result: Some(result),
            error: None,
            session_id: request.session_id.clone(),
        }
    }

    pub(crate) fn error(request: &Request, code: i64, message: impl Into<String>) -> Self {
        Self {
            id: request.id,
            result: None,
            error: Some(ProtocolError {
                code,
                message: message.into(),
            }),
            session_id: request.session_id.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Event {
    pub method: String,
    pub params: Value,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_request_and_session_ids() {
        let request: Request = serde_json::from_value(json!({
            "id": 42,
            "method": "Runtime.evaluate",
            "params": {},
            "sessionId": "session-7"
        }))
        .unwrap();
        let response = serde_json::to_value(Response::success(&request, json!({}))).unwrap();
        assert_eq!(response["id"], 42);
        assert_eq!(response["sessionId"], "session-7");
    }

    #[test]
    fn rejects_missing_or_invalid_ids() {
        assert!(serde_json::from_value::<Request>(json!({"method": "Page.enable"})).is_err());
        assert!(
            serde_json::from_value::<Request>(json!({"id": -1, "method": "Page.enable"})).is_err()
        );
    }
}
