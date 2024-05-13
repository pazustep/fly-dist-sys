use crate::rt::SendMessage;
use serde_json::{json, Value};
use std::sync::mpsc::Sender;
use tokio::{
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

pub fn start(output_tx: Sender<Value>) -> UnboundedSender<SendMessage> {
    let (send_tx, send_rx) = unbounded_channel();
    spawn(async move { send(send_rx, output_tx).await });
    send_tx
}

async fn send(mut send_rx: UnboundedReceiver<SendMessage>, output_tx: Sender<Value>) {
    let mut node_id: Option<String> = None;
    let mut last_msg_id: u64 = 0;

    loop {
        let reply = send_rx.recv().await;
        let (dest, in_reply_to, mut body) = match reply {
            Some(SendMessage::SetNodeId {
                dest,
                in_reply_to,
                node_id: new_node_id,
            }) => {
                let _ = node_id.insert(new_node_id);
                let body = json!({"type": "init_ok"});
                (dest, in_reply_to, body)
            }
            Some(SendMessage::Send {
                dest,
                in_reply_to,
                body,
            }) => (dest, in_reply_to, body),
            None => {
                break;
            }
        };

        last_msg_id += 1;
        body["msg_id"] = Value::from(last_msg_id);
        body["in_reply_to"] = Value::from(in_reply_to);

        let message = json!({
            "src": node_id,
            "dest": dest,
            "body": body
        });

        let _ = output_tx.send(message);
    }
}
