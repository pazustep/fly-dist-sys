use maelstrom::{Error, Message};

pub enum Command {
    Topology(Vec<String>),
    Broadcast(u64),
    Read,
    Replicate(Vec<u64>),
    ReplicateOk(String, Vec<u64>),
}

impl TryFrom<Message> for Command {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value.msg_type() {
            "topology" => topology(value),
            "broadcast" => broadcast(value),
            "read" => read(value),
            "replicate" => replicate(value),
            "replicate_ok" => replicate_ok(value),
            msg_type => Err(Error::not_supported(msg_type)),
        }
    }
}

fn topology(message: Message) -> Result<Command, Error> {
    let me = message.dest();

    match message.body()["topology"][me].as_array() {
        Some(neighbors) => {
            let neighbors = neighbors
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            Ok(Command::Topology(neighbors))
        }
        None => Err(Error::malformed_request(
            "topology message missing `topology` key",
        )),
    }
}

fn broadcast(message: Message) -> Result<Command, Error> {
    match message.body()["message"].as_u64() {
        Some(value) => Ok(Command::Broadcast(value)),
        _ => Err(Error::malformed_request(
            "broadcast message missing `message` key",
        )),
    }
}

fn read(_: Message) -> Result<Command, Error> {
    Ok(Command::Read)
}

fn replicate(message: Message) -> Result<Command, Error> {
    messages(message).map(Command::Replicate)
}

fn replicate_ok(message: Message) -> Result<Command, Error> {
    let node_id = message.src().to_string();
    messages(message).map(|messages| Command::ReplicateOk(node_id, messages))
}

fn messages(message: Message) -> Result<Vec<u64>, Error> {
    match message.body()["messages"].as_array() {
        Some(messages) => {
            let values = messages.iter().filter_map(|v| v.as_u64()).collect();
            Ok(values)
        }
        None => Ok(vec![]),
    }
}
