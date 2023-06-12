use crate::{
    raft_engine::{
        raft_log::Entry,
        messaging::Address,
        raft_node::Status
    },
};


#[derive(Debug, PartialEq)]
/// A driver instruction.
pub enum Instruction {
    /// Abort all pending operations, e.g. due to leader change.
    Abort,
    /// Apply a log entry.
    Apply { entry: Entry },
    /// Notify the given address with the result of applying the entry at the given index.
    Notify { id: Vec<u8>, address: Address, index: u64 },
    /// Query the state machine when the given term and index has been confirmed by vote.
    Query { id: Vec<u8>, address: Address, command: Vec<u8>, term: u64, index: u64, quorum: u64 },
    /// Extend the given server status and return it to the given address.
    Status { id: Vec<u8>, address: Address, status: Box<Status> },
    /// Votes for queries at the given term and commit index.
    Vote { term: u64, index: u64, address: Address },
}