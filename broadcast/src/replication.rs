use async_trait::async_trait;
use maelstrom::{Context, Handler, HandlerFactory, Message};
use serde_json::{json, Value};

use crate::state::State;

struct Replicate;

#[async_trait]
impl Handler<State> for Replicate {
    async fn handle(&self, message: Message, state: State, context: Context) -> Option<Value> {
        if let Some(messages) = message.body()["messages"].as_array() {
            let values = messages
                .iter()
                .filter_map(|v| v.as_u64())
                .collect::<Vec<_>>();

            let replicate = state.add_seen_all(&values);

            for (node_id, values) in replicate {
                let values = values.into_iter().collect::<Value>();
                eprintln!("replicating {} to {}", values, node_id);

                let message = json!({ "type": "replicate", "messages": values });
                context.send(node_id, None, message);
            }

            let mut reply = message.body().clone();
            reply["type"] = "replicate_ok".into();

            Some(reply)
        } else {
            eprintln!("replicate message without a `messages` property");
            None
        }
    }
}

pub struct ReplicateFactory;

impl HandlerFactory<State> for ReplicateFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(Replicate)
    }
}

struct ReplicateOk;

#[async_trait]
impl Handler<State> for ReplicateOk {
    async fn handle(&self, message: Message, state: State, _: Context) -> Option<Value> {
        if let Some(values) = message.body()["messages"].as_array() {
            let values = values.iter().filter_map(|v| v.as_u64()).collect::<Vec<_>>();
            state.ack(message.src(), &values);
            eprintln!(
                "removed {:?} from pending values for node {}",
                values,
                message.src()
            );
        }

        None
    }
}

pub struct ReplicateOkFactory;

impl HandlerFactory<State> for ReplicateOkFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(ReplicateOk)
    }
}
