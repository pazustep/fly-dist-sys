use async_trait::async_trait;
use maelstrom::{Context, Handler, HandlerFactory, Message, Node};
use serde_json::Value;
use tokio::task::JoinError;

#[tokio::main]
pub async fn main() -> Result<(), JoinError> {
    Node::default()
        .add_handler("echo", EchoFactory)
        .start()
        .await
}

struct EchoFactory;

impl HandlerFactory<()> for EchoFactory {
    fn create(&self) -> Box<dyn Handler<()> + Send> {
        Box::new(Echo)
    }
}

struct Echo;

#[async_trait]
impl Handler<()> for Echo {
    async fn handle(&self, message: Message, _: (), _: Context) -> Option<Value> {
        let mut response = message.body().clone();
        response["type"] = Value::from("echo_ok");
        Some(response)
    }
}
