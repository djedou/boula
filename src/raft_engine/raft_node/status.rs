use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub server: String,
    pub leader: String,
    pub term: u64,
    pub node_last_index: HashMap<String, u64>,
    pub commit_index: u64,
    pub apply_index: u64,
    pub storage: String,
    pub storage_size: u64,
}