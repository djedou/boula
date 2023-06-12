use serde::{Deserialize, Serialize};
use crate::raft_engine::raft_node::Status;

/// A client response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Response {
    State(Vec<u8>),
    Status(Status),
}