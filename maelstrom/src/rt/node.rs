use crate::{Error, Handler, HandlerFactory, Message, MessageValidationError};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, stdin, stdout, BufReader, BufWriter, Read, Stdin, Stdout, Write},
    sync::mpsc::{channel as sync_channel, Receiver as SyncReceiver, Sender as SyncSender},
};
use tokio::{
    spawn,
    sync::mpsc::{
        unbounded_channel as async_channel, UnboundedReceiver as AsyncReceiver,
        UnboundedSender as AsyncSender,
    },
    task::{spawn_blocking, JoinError, JoinHandle, JoinSet},
};

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
        let message_rx = start_input_task(self.input);
        let (handle, write_tx) = start_write_task(self.output);
        let reply_tx = start_reply_task(write_tx);
        start_handler_task(self.state, self.handlers, message_rx, reply_tx);
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

#[derive(Debug, thiserror::Error)]
enum InputError {
    #[error(transparent)]
    Parse(serde_json::Error),

    #[error(transparent)]
    Validation(MessageValidationError),
}

fn start_input_task(reader: impl Read + Send + 'static) -> AsyncReceiver<Message> {
    let (message_tx, message_rx) = async_channel();
    spawn_blocking(move || input_task(reader, message_tx));
    message_rx
}

fn input_task(reader: impl Read, message_tx: AsyncSender<Message>) {
    let reader = BufReader::new(reader);
    let stream = serde_json::Deserializer::from_reader(reader)
        .into_iter::<Value>()
        .map(|r| r.map_err(InputError::Parse))
        .map(|r| r.and_then(|v| Message::from_json(v).map_err(InputError::Validation)));

    for value in stream {
        match value {
            Ok(message) => {
                if message_tx.send(message).is_err() {
                    eprintln!("messages channel closed unexpectedly");
                    break;
                }
            }
            Err(e) => {
                eprintln!("error reading message from input; discarding data: {}", e)
            }
        }
    }
}

fn start_handler_task<S>(
    state: S,
    handlers: Factories<S>,
    message_rx: AsyncReceiver<Message>,
    reply_tx: AsyncSender<Reply>,
) where
    S: Clone + Send + 'static,
{
    spawn(async move { handler_task(state, handlers, message_rx, reply_tx).await });
}

async fn handler_task<S>(
    state: S,
    handlers: Factories<S>,
    mut message_rx: AsyncReceiver<Message>,
    reply_tx: AsyncSender<Reply>,
) where
    S: Clone + Send + 'static,
{
    let mut tasks = JoinSet::new();

    while let Some(message) = message_rx.recv().await {
        match message.msg_type() {
            "init" => handle_init(message, &reply_tx),
            msg_type => match handlers.get(msg_type) {
                Some(factory) => {
                    let handler = factory.create();
                    let reply_tx = reply_tx.clone();
                    let state = state.clone();
                    tasks.spawn(async move { handle(message, state, handler, reply_tx).await });
                }
                None => handle_not_supported(message, &reply_tx),
            },
        }
    }

    while tasks.join_next().await.is_some() {}
}

fn handle_init(message: Message, rx: &AsyncSender<Reply>) {
    let reply = match message.body()["node_id"].as_str() {
        Some(node_id) => Reply::set_node_id(
            message.src().to_string(),
            message.msg_id(),
            node_id.to_string(),
        ),
        None => {
            eprintln!("received init message without node_id");
            let body = Error::malformed_request("init message with missing node_id");
            Reply::new(message.src().to_string(), message.msg_id(), body.into())
        }
    };

    let _ = rx.send(reply);
}

fn handle_not_supported(message: Message, rx: &AsyncSender<Reply>) {
    let error = Error::not_supported(message.msg_type());
    let reply = Reply::new(message.src().to_string(), message.msg_id(), error.into());
    let _ = rx.send(reply);
}

async fn handle<S>(
    message: Message,
    state: S,
    handler: Box<dyn Handler<S> + Send>,
    rx: AsyncSender<Reply>,
) {
    let (dest, in_reply_to) = (message.src().to_string(), message.msg_id());
    let reply = handler.handle(message, state).await;
    let reply = Reply::new(dest, in_reply_to, reply);
    let _ = rx.send(reply);
}

#[derive(Debug)]
enum Reply {
    Reply {
        dest: String,
        in_reply_to: Option<u64>,
        body: Value,
    },
    SetNodeId {
        dest: String,
        in_reply_to: Option<u64>,
        node_id: String,
    },
}

impl Reply {
    fn new(dest: String, in_reply_to: Option<u64>, body: Value) -> Self {
        Self::Reply {
            dest,
            in_reply_to,
            body,
        }
    }

    fn set_node_id(dest: String, in_reply_to: Option<u64>, node_id: String) -> Self {
        Self::SetNodeId {
            dest,
            in_reply_to,
            node_id,
        }
    }
}

fn start_reply_task(write_tx: SyncSender<Value>) -> AsyncSender<Reply> {
    let (reply_tx, reply_rx) = async_channel();
    spawn(async move { reply_task(reply_rx, write_tx).await });
    reply_tx
}

async fn reply_task(mut reply_rx: AsyncReceiver<Reply>, write_tx: SyncSender<Value>) {
    let mut node_id: Option<String> = None;
    let mut last_msg_id: u64 = 0;

    loop {
        let reply = reply_rx.recv().await;
        let (dest, in_reply_to, mut body) = match reply {
            Some(Reply::SetNodeId {
                dest,
                in_reply_to,
                node_id: new_node_id,
            }) => {
                let _ = node_id.insert(new_node_id);
                let body = json!({"type": "init_ok"});
                (dest, in_reply_to, body)
            }
            Some(Reply::Reply {
                dest,
                in_reply_to,
                body,
            }) => (dest, in_reply_to, body),
            None => {
                break;
            }
        };

        last_msg_id += 1;
        body["msg_id"] = Value::from(last_msg_id);
        body["in_reply_to"] = Value::from(in_reply_to);

        let message = json!({
            "src": node_id,
            "dest": dest,
            "body": body
        });

        let _ = write_tx.send(message);
    }
}

fn start_write_task(writer: impl Write + Send + 'static) -> (JoinHandle<()>, SyncSender<Value>) {
    let (write_tx, write_rx) = sync_channel();
    let handle = spawn_blocking(move || write_task(writer, write_rx));
    (handle, write_tx)
}

fn write_task(writer: impl Write, write_rx: SyncReceiver<Value>) {
    let mut writer = BufWriter::new(writer);

    while let Ok(value) = write_rx.recv() {
        if let Err(err) = write_value(&mut writer, value) {
            eprintln!("failed to write response to output: {}", err);
            break;
        }
    }
}

fn write_value(mut writer: &mut impl Write, value: Value) -> io::Result<()> {
    serde_json::to_writer(&mut writer, &value)?;
    writeln!(writer)?;
    writer.flush()
}

pub struct NodeHandle {
    handle: JoinHandle<()>,
}

impl NodeHandle {
    pub async fn join(self) -> Result<(), JoinError> {
        self.handle.await
    }
}
