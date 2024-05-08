use crate::{Error, Handler, HandlerContext, Message, MessageError};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{self, stdin, stdout, BufReader, BufWriter, Read, Stdin, Stdout, Write},
    sync::mpsc,
    thread::JoinHandle,
};

pub struct Node<S, R, W> {
    input: R,
    output: W,
    state: S,
    handlers: HashMap<String, Box<dyn Handler<S> + Send>>,
}

impl<S, R, W> Node<S, R, W> {
    pub fn add_handler(
        mut self,
        msg_type: impl Into<String>,
        handler: impl Handler<S> + Send + 'static,
    ) -> Self {
        self.handlers.insert(msg_type.into(), Box::new(handler));
        self
    }
}

impl<S, R, W> Node<S, R, W>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    S: Send + 'static,
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

    pub fn start(self) -> NodeHandle {
        let (process_tx, process_rx) = mpsc::channel::<Message>();
        let (writer_tx, writer_rx) = mpsc::channel::<Message>();

        let reader = self.input;
        let reader_handle = std::thread::spawn(move || read_messages(reader, process_tx));

        let writer = self.output;
        let writer_handle = std::thread::spawn(move || write_messages(writer, writer_rx));

        let process_handle = std::thread::spawn(move || {
            process_messages(self.state, self.handlers, process_rx, writer_tx)
        });

        NodeHandle {
            reader: reader_handle,
            writer: writer_handle,
            process: process_handle,
        }
    }
}

impl<S> Node<S, Stdin, Stdout>
where
    S: Send + 'static,
{
    pub fn std(state: S) -> Self {
        Self::new(stdin(), stdout(), state)
    }
}

fn read_messages(reader: impl Read, sender: mpsc::Sender<Message>) {
    let reader = BufReader::new(reader);
    let stream = serde_json::Deserializer::from_reader(reader)
        .into_iter::<Value>()
        .map(|r| r.map_err(MessageError::ParseError))
        .map(|r| r.and_then(Message::from_json));

    for value in stream {
        match value {
            Ok(message) => {
                if sender.send(message).is_err() {
                    eprintln!("process channel closed unexpectedly");
                    break;
                }
            }
            Err(e) => {
                eprintln!("error reading message from input; discarding data: {}", e)
            }
        }
    }
}

fn write_messages(writer: impl Write, receiver: mpsc::Receiver<Message>) {
    let mut writer = BufWriter::new(writer);

    while let Ok(message) = receiver.recv() {
        if let Err(error) = write_message(&mut writer, message) {
            eprintln!("error writing message to output: {}", error);
            break;
        }
    }
}

fn write_message<W>(mut writer: &mut W, message: Message) -> io::Result<()>
where
    W: Write,
{
    let value = Value::from(message);
    serde_json::to_writer(&mut writer, &value)?;
    writeln!(writer)?;
    writer.flush()
}

fn process_messages<S>(
    state: S,
    handlers: HashMap<String, Box<dyn Handler<S> + Send>>,
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
) {
    let mut handler_context = HandlerContext::new(state, sender);

    while let Ok(message) = receiver.recv() {
        match message.msg_type() {
            "init" => match message.body()["node_id"].as_str() {
                Some(node_id) => {
                    eprintln!("initializing node id to {}", node_id);
                    handler_context.set_node_id(node_id);
                    let body = vec![("type", "init_ok")];
                    let _ = handler_context.send(
                        message.src(),
                        message.msg_id(),
                        body.into_iter().collect::<Value>(),
                    );
                }
                None => {
                    eprint!("init message mising node_id; ignoring");
                }
            },
            msg_type => {
                let handler = handlers.get(msg_type);
                let message_context = handler_context.message_context(&message);
                match handler {
                    Some(handler) => {
                        if let Err(error) = handler.handle(&message, message_context) {
                            eprintln!("error processing message: {}", error);
                        }
                    }
                    None => {
                        let error = Error::not_supported(msg_type);
                        let _ = message_context.reply(error);
                    }
                }
            }
        }
    }
}

pub struct NodeHandle {
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    process: JoinHandle<()>,
}

impl NodeHandle {
    pub fn join(self) {
        let _ = self.reader.join();
        let _ = self.writer.join();
        let _ = self.process.join();
    }
}

#[cfg(test)]
mod tests {
    use std::io::stdout;

    use crate::MessageContext;

    use super::*;
    use indoc::indoc;

    #[test]
    fn echo() {
        let input = indoc! {r#"
            {
                "src": "c1",
                "dest": "n1",
                "body": {
                    "type": "echo",
                    "msg_id": 1,
                    "echo": "Please echo 53"
                }
            }
        "#};

        let _ = Node::new(input.as_bytes(), stdout(), ())
            .add_handler("echo", |message: &Message, ctx: MessageContext<()>| {
                eprintln!("received message {}", message);
                let echo = message.body()["echo"].clone();
                let response: Value = vec![("echo", echo)].into_iter().collect();
                ctx.reply(response)
            })
            .start();
    }
}
