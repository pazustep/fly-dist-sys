use super::SendMessage;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

/// A [Handler](super::handler::Handler) context. The handler context keeps
/// track of the current message and enables handlers to send replies or
/// other arbitrary messages.
#[derive(Clone)]
pub struct Context {
    /// the source node id for the current message
    src: String,

    /// the current message id
    msg_id: Option<u64>,

    /// the channel to send messages/replies
    send_tx: UnboundedSender<SendMessage>,
}

impl Context {
    pub(super) fn new(
        src: String,
        msg_id: Option<u64>,
        send_tx: UnboundedSender<SendMessage>,
    ) -> Self {
        Self {
            src,
            msg_id,
            send_tx,
        }
    }

    /// Send a reply to the current message. This is equivalent to [send] with
    /// `dest` set to the source of the current message and `in_reply_to` set to
    /// the current message id.
    pub fn reply(&self, body: impl Into<Value>) {
        self.send(self.src.to_string(), self.msg_id, body)
    }

    /// Send a message to another node.
    pub fn send(&self, dest: String, in_reply_to: Option<u64>, body: impl Into<Value>) {
        let message = SendMessage::send(dest, in_reply_to, body);
        if let Err(err) = self.send_tx.send(message) {
            eprintln!("send channel closed; dropping message {:?}", err.0);
        }
    }
}
