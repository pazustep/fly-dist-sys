mod handler;
mod input;
mod output;
mod sender;

use crate::{Error, Handler, Message};
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use tokio::task::JoinHandle;

pub struct Node<R, W, H> {
    input: R,
    output: W,
    handler: H,
}

impl<R, W, H, C> Node<R, W, H>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    H: Handler<Command = C> + Send + 'static,
    C: TryFrom<Message, Error = Error> + Send,
{
    pub fn new(input: R, output: W, handler: H) -> Self {
        Self {
            input,
            output,
            handler,
        }
    }

    pub fn start(self) -> JoinHandle<()> {
        let (handle, output_tx) = output::start(self.output);
        let send_tx = sender::start(output_tx);
        let input_rx = input::start(self.input);
        handler::start(self.handler, input_rx, send_tx);
        handle
    }
}

impl<H, C> Node<Stdin, Stdout, H>
where
    H: Handler<Command = C> + Send + 'static,
    C: TryFrom<Message, Error = Error> + Send,
{
    pub fn from_handler(handler: H) -> Self {
        Self::new(stdin(), stdout(), handler)
    }
}
