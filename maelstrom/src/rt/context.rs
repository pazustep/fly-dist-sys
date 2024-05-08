use crate::{Error, Message};
use serde_json::{json, Value};
use std::sync::mpsc;

pub(crate) struct HandlerContext<S> {
    node_id: Option<String>,
    state: S,
    msg_counter: u64,
    sender: mpsc::Sender<Message>,
}

impl<S> HandlerContext<S> {
    pub fn new(state: S, sender: mpsc::Sender<Message>) -> Self {
        Self {
            node_id: None,
            state,
            msg_counter: 0,
            sender,
        }
    }

    pub fn set_node_id(&mut self, node_id: &str) {
        self.node_id = Some(node_id.to_string())
    }

    pub fn send(
        &mut self,
        dest: &str,
        in_reply_to: Option<u64>,
        body: impl Into<Value>,
    ) -> Result<(), Error> {
        self.msg_counter += 1;
        let mut body = body.into();
        body["msg_id"] = Value::from(self.msg_counter);

        if let Some(in_reply_to) = in_reply_to {
            body["in_reply_to"] = Value::from(in_reply_to);
        }

        let json = json!({
            "src": Value::from(self.node_id.as_deref().unwrap_or("")),
            "dest": Value::from(dest),
            "body": body,
        });

        let message = Message::from_valid_json(json);
        if let Err(err) = self.sender.send(message) {
            eprintln!("error sending message: {}", err);
        }

        Ok(())
    }

    pub(crate) fn message_context<'s, 'a>(
        &'s mut self,
        message: &'a Message,
    ) -> MessageContext<'a, S>
    where
        's: 'a,
    {
        MessageContext {
            parent: self,
            src: message.src(),
            msg_id: message.msg_id(),
        }
    }
}

pub struct MessageContext<'a, S> {
    parent: &'a mut HandlerContext<S>,
    src: &'a str,
    msg_id: Option<u64>,
}

impl<'a, S> MessageContext<'a, S> {
    pub fn state(&self) -> &S {
        &self.parent.state
    }

    pub fn reply(self, body: impl Into<Value>) -> Result<(), Error> {
        self.parent.send(self.src, self.msg_id, body)
    }
}
