use super::Factories;
use crate::{rt::SendMessage, Context, Error, Handler, Message};
use tokio::{
    spawn,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinSet,
};

pub fn start<S>(
    state: S,
    handlers: Factories<S>,
    message_rx: UnboundedReceiver<Message>,
    reply_tx: UnboundedSender<SendMessage>,
) where
    S: Clone + Send + 'static,
{
    spawn(async move { handle_messages(state, handlers, message_rx, reply_tx).await });
}

async fn handle_messages<S>(
    state: S,
    handlers: Factories<S>,
    mut message_rx: UnboundedReceiver<Message>,
    reply_tx: UnboundedSender<SendMessage>,
) where
    S: Clone + Send + 'static,
{
    let mut tasks = JoinSet::new();

    while let Some(message) = message_rx.recv().await {
        match message.msg_type() {
            "init" => handle_init(message, &reply_tx),
            msg_type => match handlers.get(msg_type) {
                Some(factory) => {
                    let handler = factory.create();
                    let reply_tx = reply_tx.clone();
                    let state = state.clone();
                    tasks.spawn(async move { handle(message, state, handler, reply_tx).await });
                }
                None => handle_not_supported(message, &reply_tx),
            },
        }
    }

    while tasks.join_next().await.is_some() {}
}

fn handle_init(message: Message, send_rx: &UnboundedSender<SendMessage>) {
    let reply = match message.body()["node_id"].as_str() {
        Some(node_id) => SendMessage::set_node_id(
            message.src().to_string(),
            message.msg_id(),
            node_id.to_string(),
        ),
        None => {
            eprintln!("received init message without node_id");
            let body = Error::malformed_request("init message with missing node_id");
            SendMessage::send(message.src().to_string(), message.msg_id(), body)
        }
    };

    let _ = send_rx.send(reply);
}

fn handle_not_supported(message: Message, send_rx: &UnboundedSender<SendMessage>) {
    let error = Error::not_supported(message.msg_type());
    let reply = SendMessage::send(message.src().to_string(), message.msg_id(), error);
    let _ = send_rx.send(reply);
}

async fn handle<S>(
    message: Message,
    state: S,
    handler: Box<dyn Handler<S> + Send>,
    send_tx: UnboundedSender<SendMessage>,
) {
    let context = Context::new(send_tx.clone());
    let (dest, in_reply_to) = (message.src().to_string(), message.msg_id());
    if let Some(reply) = handler.handle(message, state, context).await {
        let reply = SendMessage::send(dest, in_reply_to, reply);
        let _ = send_tx.send(reply);
    }
}
