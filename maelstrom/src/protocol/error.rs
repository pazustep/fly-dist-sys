use serde_json::{json, Value};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub struct Error {
    code: u32,
    text: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.text {
            Some(ref text) => write!(f, "error code {}: {}", self.code, text),
            None => write!(f, "error code {}", self.code),
        }
    }
}

impl Error {
    pub fn new(code: u32, text: Option<String>) -> Self {
        Self { code, text }
    }

    pub fn not_supported(msg_type: &str) -> Self {
        let text = format!("message type not supported: {}", msg_type);
        Self::new(10, Some(text))
    }
}

impl From<Error> for Value {
    fn from(value: Error) -> Self {
        json!({
            "type": "error",
            "code": value.code,
            "text": value.text
        })
    }
}
