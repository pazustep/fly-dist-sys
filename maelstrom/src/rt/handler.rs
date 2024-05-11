use crate::Message;
use async_trait::async_trait;
use serde_json::Value;

/// A factory for message handlers. A new handler is created to process each
/// matching message.
///
/// Please note that the factory must produce boxed values; this is necessary to
/// keep the factory trait itself safe for dynamic dispatch.
pub trait HandlerFactory<S> {
    fn create(&self) -> Box<dyn Handler<S> + Send>;
}

/// A maelstrom RPC message handler.
#[async_trait]
pub trait Handler<S> {
    /// Process a maelstrom message, returning a JSON value. The returned value
    /// is used as the body of the response message.
    ///
    /// Please note that handlers are infallible — they should
    /// handle errors internally, producing an [Error](crate::Error) value if
    /// necessary.
    async fn handle(&self, message: Message, state: S) -> Value;
}
