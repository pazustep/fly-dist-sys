use crate::prelude::*;

struct Read;

#[async_trait]
impl Handler<State> for Read {
    async fn handle(&self, _: Message, state: State, _: Context) -> Option<Value> {
        let messages = state.seen().into_iter().collect::<Value>();
        Some(json!({ "type": "read_ok", "messages": messages }))
    }
}

pub struct ReadFactory;

impl HandlerFactory<State> for ReadFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(Read)
    }
}
