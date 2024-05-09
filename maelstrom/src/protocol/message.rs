use serde_json::Value;
use std::fmt;

/// A maelstrom message.
///
/// This is a thin wrapper over the raw JSON payload adding validation at
/// construction time and convenient access to standard message fields described
/// in the [maelstrom protocol][protocol].
///
/// [protocol]: https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md
#[derive(Clone, Debug)]
pub struct Message {
    json: Value,
}

impl Message {
    /// Construct a new `Message` value from the given JSON, validating that it
    /// contains all properties required by the protocol.
    pub fn from_json(json: Value) -> Result<Self, MessageValidationError> {
        let mut errors = vec![];

        let src = &json["src"];
        if !src.is_string() {
            errors.push(("src".to_string(), src.clone()));
        }

        let dest = &json["dest"];
        if !dest.is_string() {
            errors.push(("dest".to_string(), dest.clone()));
        }

        let body = &json["body"];
        if body.is_object() {
            let msg_type = &body["type"];
            if !msg_type.is_string() {
                errors.push(("body.type".to_string(), msg_type.clone()));
            }
        } else {
            errors.push(("body".to_string(), body.clone()));
        }

        if errors.is_empty() {
            Ok(Self::from_valid_json(json))
        } else {
            errors.shrink_to_fit();
            Err(MessageValidationError(errors))
        }
    }

    fn from_valid_json(json: Value) -> Self {
        Self { json }
    }

    /// Get a reference to the source of this message
    pub fn src(&self) -> &str {
        self.json["src"].as_str().unwrap()
    }

    /// Get a reference to the destination of this message
    pub fn dest(&self) -> &str {
        self.json["dest"].as_str().unwrap()
    }

    /// Get a reference to the type of this message
    pub fn msg_type(&self) -> &str {
        self.json["body"]["type"].as_str().unwrap()
    }

    /// Get a reference to the id of this message, if any
    pub fn msg_id(&self) -> Option<u64> {
        self.json["body"]["msg_id"].as_u64()
    }

    /// Get a reference to the id of the message this message is a reply to, if any
    pub fn in_reply_to(&self) -> Option<u64> {
        self.json["body"]["in_reply_to"].as_u64()
    }

    /// Get a reference to the body of this message
    pub fn body(&self) -> &Value {
        &self.json["body"]
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.json.fmt(f)
    }
}

impl TryFrom<Value> for Message {
    type Error = MessageValidationError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Message::from_json(value)
    }
}

impl From<Message> for Value {
    fn from(message: Message) -> Value {
        message.json
    }
}

/// The error type for [Message::from_json].
///
/// Constructing a message from an arbitrary JSON value may fail because of
/// missing or invalid keys. This error includes all errors found.
#[derive(Debug, thiserror::Error)]
pub struct MessageValidationError(Vec<(String, Value)>);

impl fmt::Display for MessageValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "invalid message payload")?;

        for (key, value) in self.0.iter() {
            write!(f, "\n  - `{}`: `{}`", key, value)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse() -> Result<(), MessageValidationError> {
        let message = Message::from_json(json!({
            "src": "c1",
            "dest": "n1",
            "body": {
                "type": "echo",
                "msg_id": 1,
                "echo": "Please echo 35"
            }
        }))?;

        assert_eq!(message.src(), "c1");
        assert_eq!(message.dest(), "n1");
        assert_eq!(message.msg_type(), "echo");
        assert_eq!(message.msg_id(), Some(1));

        let body = message.body();
        assert_eq!(body["echo"].as_str(), Some("Please echo 35"));

        Ok(())
    }
}
