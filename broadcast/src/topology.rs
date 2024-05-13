use crate::prelude::*;

struct Topology;

#[async_trait]
impl Handler<State> for Topology {
    async fn handle(&self, message: Message, state: State, _: Context) -> Option<Value> {
        let me = message.dest();
        let neighbors = message.body()["topology"][me].as_array();

        if let Some(neighbors) = neighbors {
            let neighbors = neighbors
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>();

            state.add_neighbors(&neighbors);
            eprintln!("topology updated with neighbors {:?}", neighbors);
        }

        Some(json!({ "type": "topology_ok" }))
    }
}

pub struct TopologyFactory;

impl HandlerFactory<State> for TopologyFactory {
    fn create(&self) -> Box<dyn Handler<State> + Send> {
        Box::new(Topology)
    }
}
