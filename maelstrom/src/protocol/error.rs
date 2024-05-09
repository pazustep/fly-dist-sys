use serde_json::{json, Value};

/// An error type for Maelstrom message bodies.
///
/// This type can be converted into a JSON [Value] that matches the [Maelstrom
/// error][error] specification. Arbitrary error values can be constructed with
/// [Error::new], or you can you one of the specific constructors like
/// [Error::not_supported].
///
/// [error]: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md#errors
#[derive(Debug)]
pub struct Error {
    code: u32,
    text: Option<String>,
}

impl Error {
    /// Creates a new error with a `code` and optional `text`.
    fn new(code: u32, text: Option<String>) -> Self {
        Self { code, text }
    }

    /// Creates a `not-supported` error for the given message type.
    pub fn not_supported(msg_type: &str) -> Self {
        let text = format!("message type not supported: {}", msg_type);
        Self::new(10, Some(text))
    }

    /// Creates a `malformed-request` error with the given text
    pub fn malformed_request(text: &str) -> Self {
        Self::new(13, Some(text.to_string()))
    }
}

impl From<Error> for Value {
    /// Converts an [Error] into a JSON [Value] matching the Maelstrom error
    /// message specification.
    fn from(value: Error) -> Self {
        json!({
            "type": "error",
            "code": value.code,
            "text": value.text
        })
    }
}
