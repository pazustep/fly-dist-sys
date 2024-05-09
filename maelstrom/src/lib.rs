//! This libraries makes it easy to implement [Maelstrom] nodes in Rust.
//!
//! [Maelstrom]: https://github.com/jepsen-io/maelstrom/tree/main
mod protocol;
mod rt;

pub use protocol::*;
pub use rt::*;
