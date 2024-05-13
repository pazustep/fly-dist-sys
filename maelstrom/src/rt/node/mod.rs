mod handler;
mod input;
mod output;
mod sender;

use crate::HandlerFactory;
use std::{
    collections::HashMap,
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
};
use tokio::task::{JoinError, JoinHandle};

type Factories<S> = HashMap<String, Box<dyn HandlerFactory<S> + Send>>;

pub struct Node<R, W, S> {
    input: R,
    output: W,
    state: S,
    handlers: Factories<S>,
}

impl<R, W, S> Node<R, W, S> {
    pub fn add_handler(
        mut self,
        msg_type: impl Into<String>,
        handler: impl HandlerFactory<S> + Send + 'static,
    ) -> Self {
        self.handlers.insert(msg_type.into(), Box::new(handler));
        self
    }
}

impl<R, W, S> Node<R, W, S>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    S: Clone + Send + 'static,
{
    pub fn new(input: R, output: W, state: S) -> Self {
        let handlers = HashMap::new();
        Self {
            input,
            output,
            state,
            handlers,
        }
    }

    pub fn start(self) -> JoinHandle<()> {
        let (handle, output_tx) = output::start(self.output);
        let send_tx = sender::start(output_tx);
        let input_rx = input::start(self.input);
        handler::start(self.state, self.handlers, input_rx, send_tx);
        handle
    }
}

impl<S> Default for Node<Stdin, Stdout, S>
where
    S: Default + Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new(stdin(), stdout(), S::default())
    }
}

pub struct NodeHandle {
    handle: JoinHandle<()>,
}

impl NodeHandle {
    pub async fn join(self) -> Result<(), JoinError> {
        self.handle.await
    }
}
