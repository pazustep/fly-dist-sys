use maelstrom::{Error, Message, MessageContext, Node};
use serde_json::Value;

pub fn main() {
    Node::std(())
        .add_handler("echo", handle_echo)
        .start()
        .join()
}

fn handle_echo(message: &Message, ctx: MessageContext<()>) -> Result<(), Error> {
    let mut response = message.body().clone();
    response["type"] = Value::from("echo_ok");
    ctx.reply(response)
}
