use crate::Context;
use std::future::Future;

/// A maelstrom RPC message handler. Handlers work by processing a sequence of
/// commands. The handler can send messages to other nodes (as a reply to the
/// current command or not) using the provided [Context].
pub trait Handler {
    /// The associated command type. While not constrained here, the command is
    /// expected to implement [`TryFrom<Value, Error = crate::Error>`](TryFrom).
    type Command;

    /// Processes a command. The runtime calls this for every received message,
    /// sequentially. To maintain performance, this method should never block or
    /// take too long to run. If your workload requires anything like that, you
    /// can spawn a task and send a reply when the task is completed using the
    /// provided [Context].
    fn handle(&mut self, command: Self::Command, ctx: Context);

    /// Stops the handler. The runtime calls this methods and waits for it to
    /// finish before exiting. If your handler spawns async tasks, you can
    /// `await` them here to make sure they complete before terminating.
    fn stop(&self) -> impl Future<Output = ()> + Send {
        async {}
    }
}
