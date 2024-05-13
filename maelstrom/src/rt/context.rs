use super::SendMessage;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct Context {
    send_tx: UnboundedSender<SendMessage>,
}

impl Context {
    pub(super) fn new(send_tx: UnboundedSender<SendMessage>) -> Self {
        Self { send_tx }
    }

    pub fn send(&self, dest: String, in_reply_to: Option<u64>, body: impl Into<Value>) {
        let message = SendMessage::send(dest, in_reply_to, body);
        if let Err(err) = self.send_tx.send(message) {
            eprintln!("failed to send message; ignoring: {}", err);
        }
    }
}
