mod broadcast;
mod read;
mod topology;

use broadcast::BroadcastFactory;
use maelstrom::Node;
use read::ReadFactory;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::task::JoinError;
use topology::TopologyFactory;

mod prelude {
    pub use crate::State;
    pub use async_trait::async_trait;
    pub use maelstrom::{Handler, HandlerFactory, Message};
    pub use serde_json::{json, Value};
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    Node::default()
        .add_handler("topology", TopologyFactory)
        .add_handler("broadcast", BroadcastFactory)
        .add_handler("read", ReadFactory)
        .start()
        .await
}

#[derive(Default, Clone)]
pub struct State {
    seen: Arc<Mutex<HashSet<u64>>>,
}
