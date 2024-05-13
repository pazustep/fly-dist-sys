use crate::{Message, MessageValidationError};
use serde_json::Value;
use std::io::{BufReader, Read};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, thiserror::Error)]
enum InputError {
    #[error(transparent)]
    Parse(serde_json::Error),

    #[error(transparent)]
    Validation(MessageValidationError),
}

pub fn start(reader: impl Read + Send + 'static) -> UnboundedReceiver<Message> {
    let (message_tx, message_rx) = unbounded_channel();
    tokio::task::spawn_blocking(move || read_messages(reader, message_tx));
    message_rx
}

fn read_messages(reader: impl Read, message_tx: UnboundedSender<Message>) {
    let reader = BufReader::new(reader);
    let stream = serde_json::Deserializer::from_reader(reader)
        .into_iter::<Value>()
        .map(|r| r.map_err(InputError::Parse))
        .map(|r| r.and_then(|v| Message::from_json(v).map_err(InputError::Validation)));

    for value in stream {
        match value {
            Ok(message) => {
                if message_tx.send(message).is_err() {
                    eprintln!("messages channel closed unexpectedly");
                    break;
                }
            }
            Err(e) => {
                eprintln!("error reading message from input; discarding data: {}", e)
            }
        }
    }
}
