use std::{error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsException {
    message: String,
}

impl JsException {
    pub fn from_message(message: impl Into<String>) -> Self {
        Self::new(message)
    }

    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for JsException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for JsException {}
