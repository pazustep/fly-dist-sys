mod context;
mod handler;
mod node;

pub use context::*;
pub use handler::*;
pub use node::*;

use serde_json::Value;

#[derive(Debug)]
enum SendMessage {
    Send {
        dest: String,
        in_reply_to: Option<u64>,
        body: Value,
    },
    SetNodeId {
        dest: String,
        in_reply_to: Option<u64>,
        node_id: String,
    },
}

impl SendMessage {
    fn send(dest: String, in_reply_to: Option<u64>, body: impl Into<Value>) -> Self {
        Self::Send {
            dest,
            in_reply_to,
            body: body.into(),
        }
    }

    fn set_node_id(dest: String, in_reply_to: Option<u64>, node_id: String) -> Self {
        Self::SetNodeId {
            dest,
            in_reply_to,
            node_id,
        }
    }
}
