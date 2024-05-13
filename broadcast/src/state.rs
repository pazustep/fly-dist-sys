use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Default, Clone)]
pub struct State(Arc<Mutex<Inner>>);

#[derive(Default)]
struct Inner {
    seen: HashSet<u64>,
    neighbors: HashMap<String, HashSet<u64>>,
}

impl State {
    /// Adds `neighbors` to the map of neighbors that this node will replicate
    /// broadcast messages to
    pub fn add_neighbors(&self, neighbors: &[&str]) {
        let mut inner = self.0.lock().unwrap();
        inner
            .neighbors
            .extend(neighbors.iter().map(|n| (n.to_string(), HashSet::new())));
    }

    /// Adds a value to the list of values seen by this node, and to the list of
    /// values pending replication for each neighbor. Returns a map of neighbors
    /// and values for replication.
    pub fn add_seen(&self, value: u64) -> HashMap<String, HashSet<u64>> {
        let mut inner = self.0.lock().unwrap();
        if inner.seen.insert(value) {
            eprintln!("[add_seen] {} seen for the first time", value);

            let mut replicate = HashMap::new();
            for (node_id, pending) in inner.neighbors.iter_mut() {
                if pending.insert(value) {
                    eprintln!("[add_seen] {} is newly pending for {}", value, node_id);
                    replicate.insert(node_id.to_string(), pending.clone());
                }
            }

            replicate
        } else {
            eprintln!("[add_seen] {} was already seen, skipping", value);
            HashMap::new()
        }
    }

    /// Add all values to the list of values seen by this node, and to the list
    /// of pending values for each neighbor. Returns a map of neighbors that
    /// were updated and their new pending values
    pub fn add_seen_all(&self, values: &[u64]) -> HashMap<String, HashSet<u64>> {
        let mut inner = self.0.lock().unwrap();
        inner.seen.extend(values);

        let neighbors = &mut inner.neighbors;
        let mut updated = HashSet::new();

        for (node_id, pending) in neighbors.iter_mut() {
            for value in values {
                if pending.insert(*value) {
                    eprintln!("[add_seen_all] {} is newly pending for {}", value, node_id);
                    updated.insert(node_id.to_string());
                }
            }
        }

        neighbors
            .iter()
            .filter(|(k, _)| updated.contains(k.as_str()))
            .map(|(key, values)| (key.to_string(), values.clone()))
            .collect()
    }

    pub fn ack(&self, node_id: &str, values: &[u64]) {
        let mut inner = self.0.lock().unwrap();
        if let Some(pending) = inner.neighbors.get_mut(node_id) {
            pending.retain(|v| !values.contains(v));
        }
    }

    /// Retrieves the set of values already seen by this node
    pub fn seen(&self) -> Vec<u64> {
        let inner = self.0.lock().unwrap();
        inner.seen.iter().copied().collect()
    }
}
