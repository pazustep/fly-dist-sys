mod broadcast;
mod read;
mod replication;
mod state;
mod topology;

use broadcast::BroadcastFactory;
use maelstrom::Node;
use read::ReadFactory;
use replication::{ReplicateFactory, ReplicateOkFactory};
use tokio::task::JoinError;
use topology::TopologyFactory;

mod prelude {
    pub use crate::state::State;
    pub use async_trait::async_trait;
    pub use maelstrom::{Context, Handler, HandlerFactory, Message};
    pub use serde_json::{json, Value};
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    Node::default()
        .add_handler("topology", TopologyFactory)
        .add_handler("broadcast", BroadcastFactory)
        .add_handler("read", ReadFactory)
        .add_handler("replicate", ReplicateFactory)
        .add_handler("replicate_ok", ReplicateOkFactory)
        .start()
        .await
}
