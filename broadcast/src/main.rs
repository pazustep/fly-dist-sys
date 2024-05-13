mod command;
mod handler;

use handler::BroadcastHandler;
use maelstrom::Node;
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    Node::from_handler(BroadcastHandler::new()).start().await
}
