use crate::prelude::*;

struct Read;

#[async_trait]
impl Handler<State> for Read {
    async fn handle(&self, _: Message, state: State) -> Value {
        let seen = state.seen.lock().unwrap();
        json!({ "type": "read_ok", "messages": seen.iter().cloned().collect::<Value>() })
    }
}

pub struct ReadFactory;

impl HandlerFactory<State> for ReadFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(Read)
    }
}
