use serde::{Serialize, Deserialize};

/// A replicated log entry
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// The index of the entry.
    pub index: u64,
    /// The term in which the entry was added.
    pub term: u64,
    /// The state machine command. None is used to commit noops during leader election.
    pub command: Option<Vec<u8>>,
}