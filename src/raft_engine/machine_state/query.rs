use crate::raft_engine::messaging::Address;
use std::collections::HashSet;

/// A driver query.
pub struct Query {
    pub id: Vec<u8>,
    pub term: u64,
    pub address: Address,
    pub command: Vec<u8>,
    pub quorum: u64,
    pub votes: HashSet<Address>,
}