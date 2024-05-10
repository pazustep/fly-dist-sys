use async_trait::async_trait;
use maelstrom::{Handler, HandlerFactory, Message, Node};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    Node::default()
        .add_handler("generate", GenerateFactory::default())
        .start()
        .await
}

struct Generate {
    counter: Arc<AtomicU64>,
}

impl Generate {
    fn new(counter: Arc<AtomicU64>) -> Self {
        Self { counter }
    }
}

#[async_trait]
impl Handler for Generate {
    async fn handle(&self, message: Message) -> Value {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        let node_id = message.dest();
        let unique_id = format!("{}-{}", node_id, counter);
        json!({
            "type": "generate_ok",
            "id": Value::from(unique_id),
        })
    }
}

#[derive(Default)]
struct GenerateFactory {
    counter: Arc<AtomicU64>,
}

impl HandlerFactory for GenerateFactory {
    fn create(&self) -> Box<dyn Handler + Send> {
        Box::new(Generate::new(self.counter.clone()))
    }
}
