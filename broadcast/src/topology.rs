use crate::prelude::*;

struct Topology;

#[async_trait]
impl Handler<State> for Topology {
    async fn handle(&self, _: Message, _: State) -> Value {
        json!({ "type": "topology_ok" })
    }
}

pub struct TopologyFactory;

impl HandlerFactory<State> for TopologyFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(Topology)
    }
}
