use crate::{rt::SendMessage, Context, Error, Handler, Message};
use tokio::{
    spawn,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

pub fn start<H, C>(
    handler: H,
    message_rx: UnboundedReceiver<Message>,
    send_tx: UnboundedSender<SendMessage>,
) where
    H: Handler<Command = C> + Send + 'static,
    C: TryFrom<Message, Error = Error> + Send,
{
    spawn(async move { handle_messages(handler, message_rx, send_tx).await });
}

async fn handle_messages<H, C>(
    mut handler: H,
    mut message_rx: UnboundedReceiver<Message>,
    send_tx: UnboundedSender<SendMessage>,
) where
    H: Handler<Command = C> + Send,
    C: TryFrom<Message, Error = Error> + Send,
{
    while let Some(message) = message_rx.recv().await {
        match message.msg_type() {
            "init" => handle_init(message, &send_tx),
            _ => handle(message, &mut handler, &send_tx),
        }
    }

    handler.stop().await;
}

fn handle_init(message: Message, send_tx: &UnboundedSender<SendMessage>) {
    let (src, in_reply_to) = (message.src().to_string(), message.msg_id());
    let reply = match message.body()["node_id"].as_str() {
        Some(node_id) => SendMessage::set_node_id(src, in_reply_to, node_id.to_string()),
        None => {
            eprintln!("received init message without node_id");
            let body = Error::malformed_request("init message with missing node_id");
            SendMessage::send(src, in_reply_to, body)
        }
    };

    let _ = send_tx.send(reply);
}

fn handle<H, C>(message: Message, handler: &mut H, send_tx: &UnboundedSender<SendMessage>)
where
    H: Handler<Command = C>,
    C: TryFrom<Message, Error = Error>,
{
    let (src, in_reply_to) = (message.src().to_string(), message.msg_id());
    match C::try_from(message) {
        Ok(command) => {
            let context = Context::new(src, in_reply_to, send_tx.clone());
            handler.handle(command, context);
        }
        Err(error) => {
            let reply = SendMessage::send(src, in_reply_to, error);
            let _ = send_tx.send(reply);
        }
    }
}
