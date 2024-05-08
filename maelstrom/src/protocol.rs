use serde_json::{Error as JsonError, Value as JsonValue};
use std::{fmt, str::FromStr, string::String, vec::Vec};

/// A reference to a maelstrom node id
#[derive(Debug, PartialEq, Eq)]
pub struct NodeIdRef<'a>(&'a str);

impl<'a> PartialEq<&str> for NodeIdRef<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// A maelstrom message id
#[derive(Debug, PartialEq, Eq)]
pub struct MessageId(u64);

impl PartialEq<u64> for MessageId {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

/// A maelstrom message
#[derive(Debug)]
pub struct Message {
    json: JsonValue,
}

impl Message {
    /// Create a new message from a JSON value
    fn from_json(json: JsonValue) -> Result<Self, MessageError> {
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
            Ok(Message { json })
        } else {
            errors.shrink_to_fit();
            Err(MessageError::Invalid(ValidationErrors(errors)))
        }
    }

    /// Get a reference to the source of this message
    pub fn src(&self) -> NodeIdRef {
        let src = self.json["src"].as_str().unwrap();
        NodeIdRef(src)
    }

    /// Get a reference to the destination of this message
    pub fn dest(&self) -> NodeIdRef {
        let dest = self.json["dest"].as_str().unwrap();
        NodeIdRef(dest)
    }

    /// Get a reference to the type of this message
    pub fn msg_type(&self) -> &str {
        let body = &self.json["body"];
        body["type"].as_str().unwrap()
    }

    /// Get a reference to the id of this message, if any
    pub fn msg_id(&self) -> Option<MessageId> {
        let body = &self.json["body"];
        body["msg_id"].as_u64().map(MessageId)
    }

    /// Get a reference to the id of the message this message is a reply to, if any
    pub fn in_reply_to(&self) -> Option<MessageId> {
        let body = &self.json["body"];
        body["in_reply_to"].as_u64().map(MessageId)
    }

    /// Get a reference to the body of this message
    pub fn body(&self) -> &JsonValue {
        &self.json["body"]
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.json.fmt(f)
    }
}

impl FromStr for Message {
    type Err = MessageError;

    fn from_str(payload: &str) -> Result<Self, Self::Err> {
        let json = serde_json::from_str(payload).map_err(MessageError::ParseError)?;
        Message::from_json(json)
    }
}

impl TryFrom<JsonValue> for Message {
    type Error = MessageError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        Message::from_json(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("invalid message payload: {0}")]
    ParseError(#[source] JsonError),

    #[error("message payload contains one or more invalid keys:{0}")]
    Invalid(ValidationErrors),
}

#[derive(Debug)]
pub struct ValidationErrors(Vec<(String, JsonValue)>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.0.iter() {
            write!(f, "\n  - `{}`: `{}`", key, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() -> Result<(), MessageError> {
        let message = Message::from_str(
            r#"
            {
                "src": "c1",
                "dest": "n1",
                "body": {
                    "type": "echo",
                    "msg_id": 1,
                    "echo": "Please echo 35"
                }
            }"#
            .trim(),
        )?;

        assert_eq!(message.src(), "c1");
        assert_eq!(message.dest(), "n1");
        assert_eq!(message.msg_type(), "echo");
        assert_eq!(message.msg_id(), Some(MessageId(1)));

        let body = message.body();
        assert_eq!(body["echo"].as_str(), Some("Please echo 35"));

        Ok(())
    }
}
