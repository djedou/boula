use serde::{Deserialize, Serialize};
use crate::raft_engine::{
    messaging::{Event, Address}
};

/// A message passed between Raft nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// The current term of the sender.
    pub term: u64,
    /// The sender address.
    pub from: Address,
    /// The recipient address.
    pub to: Address,
    /// The message event.
    pub event: Event,
}