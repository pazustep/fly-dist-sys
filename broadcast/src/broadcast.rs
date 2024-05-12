use crate::prelude::*;

struct Broadcast;

#[async_trait]
impl Handler<State> for Broadcast {
    async fn handle(&self, message: Message, state: State) -> Value {
        if let Some(value) = message.body()["message"].as_u64() {
            let mut seen = state.seen.lock().unwrap();
            seen.insert(value);
        }

        let response = json!({ "type": "broadcast_ok" });
        response
    }
}

pub struct BroadcastFactory;

impl HandlerFactory<State> for BroadcastFactory {
    fn create(&self) -> Box<dyn maelstrom::Handler<State> + Send> {
        Box::new(Broadcast)
    }
}
