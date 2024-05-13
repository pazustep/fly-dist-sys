use maelstrom::{Error, Handler, Message, Node};
use serde_json::Value;
use tokio::task::JoinError;

#[tokio::main]
pub async fn main() -> Result<(), JoinError> {
    Node::from_handler(EchoHandler).start().await
}

struct Echo {
    body: Value,
}

impl TryFrom<Message> for Echo {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value.msg_type() {
            "echo" => {
                let body = value.body().clone();
                Ok(Self { body })
            }
            msg_type => Err(Error::not_supported(msg_type)),
        }
    }
}

struct EchoHandler;

impl Handler for EchoHandler {
    type Command = Echo;

    fn handle(&mut self, command: Self::Command, ctx: maelstrom::Context) {
        let mut reply = command.body;
        reply["type"] = "echo_ok".into();
        ctx.reply(reply);
    }
}
