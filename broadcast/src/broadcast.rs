use crate::prelude::*;

struct Broadcast;

#[async_trait]
impl Handler<State> for Broadcast {
    async fn handle(&self, message: Message, state: State, context: Context) -> Option<Value> {
        if let Some(value) = message.body()["message"].as_u64() {
            let replicate = state.add_seen(value);

            for (node_id, values) in replicate {
                let values = values.into_iter().collect::<Value>();
                eprintln!("replicating pending values {} to {}", values, node_id);

                let body = json!({ "type": "replicate", "messages": values });
                context.send(node_id, None, body);
            }
        }

        let response = json!({ "type": "broadcast_ok" });
        Some(response)
    }
}

pub struct BroadcastFactory;

impl HandlerFactory<State> for BroadcastFactory {
    fn create(&self) -> Box<dyn maelstrom::Handler<State> + Send> {
        Box::new(Broadcast)
    }
}
