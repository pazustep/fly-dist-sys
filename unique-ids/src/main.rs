use maelstrom::Node;
use maelstrom::{Context, Error, Handler, Message};
use serde_json::{json, Value};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    Node::from_handler(GenerateHandler::default()).start().await
}

struct Generate {
    node_id: String,
}

impl TryFrom<Message> for Generate {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value.msg_type() {
            "generate" => Ok(Self {
                node_id: value.dest().to_string(),
            }),
            msg_type => Err(Error::not_supported(msg_type)),
        }
    }
}

#[derive(Default)]
struct GenerateHandler {
    counter: u64,
}

impl Handler for GenerateHandler {
    type Command = Generate;

    fn handle(&mut self, command: Self::Command, ctx: Context) {
        let counter = self.counter;
        self.counter += 1;

        let node_id = command.node_id;
        let unique_id = format!("{}-{}", node_id, counter);
        let reply = json!({
            "type": "generate_ok",
            "id": Value::from(unique_id),
        });

        ctx.reply(reply);
    }
}
