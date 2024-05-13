use crate::command::Command;
use maelstrom::{Context, Handler};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

pub struct BroadcastHandler {
    seen: HashSet<u64>,
    neighbors: HashMap<String, HashSet<u64>>,
}

impl BroadcastHandler {
    pub fn new() -> Self {
        Self {
            seen: Default::default(),
            neighbors: Default::default(),
        }
    }

    fn topology(&mut self, neighbors: Vec<String>, ctx: Context) {
        self.neighbors.extend(
            neighbors
                .into_iter()
                .map(|node_id| (node_id, HashSet::new())),
        );

        let reply = json!({ "type": "topology_ok"});
        ctx.reply(reply);
    }

    fn broadcast(&mut self, value: u64, ctx: Context) {
        self.add_seen(&[value], &ctx);

        let reply = json!({ "type": "broadcast_ok"});
        ctx.reply(reply);
    }

    fn read(&mut self, ctx: Context) {
        let messages = self.seen.iter().cloned().collect::<Value>();
        let reply = json!({ "type":"read_ok", "messages": messages});
        ctx.reply(reply)
    }

    fn replicate(&mut self, values: Vec<u64>, ctx: Context) {
        self.add_seen(&values, &ctx);

        let messages = values.into_iter().collect::<Value>();
        let reply = json!({"type":"replicate_ok", "messages": messages});
        ctx.reply(reply)
    }

    fn replicate_ok(&mut self, node_id: String, values: Vec<u64>) {
        if let Some(pending) = self.neighbors.get_mut(&node_id) {
            pending.retain(|v| !values.contains(v));
        }
    }

    fn add_seen(&mut self, values: &[u64], ctx: &Context) {
        let mut modified = false;

        for value in values {
            if self.seen.insert(*value) {
                modified = true;

                for pending in self.neighbors.values_mut() {
                    pending.insert(*value);
                }
            }
        }

        if modified {
            for (node_id, pending) in self.neighbors.iter() {
                let messages = pending.iter().cloned().collect::<Value>();
                let replicate = json!({"type": "replicate", "messages": messages});
                ctx.send(node_id.to_string(), None, replicate);
            }
        }
    }
}

impl Handler for BroadcastHandler {
    type Command = Command;

    fn handle(&mut self, command: Command, ctx: Context) {
        match command {
            Command::Topology(neighbors) => self.topology(neighbors, ctx),
            Command::Broadcast(value) => self.broadcast(value, ctx),
            Command::Read => self.read(ctx),
            Command::Replicate(values) => self.replicate(values, ctx),
            Command::ReplicateOk(node_id, values) => self.replicate_ok(node_id, values),
        }
    }
}
