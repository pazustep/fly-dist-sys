use crate::{Error, Message, MessageContext};

pub trait Handler<S> {
    fn handle(&self, message: &Message, context: MessageContext<S>) -> Result<(), Error>;
}

impl<F, S> Handler<S> for F
where
    F: Fn(&Message, MessageContext<S>) -> Result<(), Error>,
{
    fn handle(&self, message: &Message, context: MessageContext<S>) -> Result<(), Error> {
        self(message, context)
    }
}
