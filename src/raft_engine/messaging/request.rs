use serde::{Deserialize, Serialize};

/// A client request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Request {
    Query(Vec<u8>),
    Mutate(Vec<u8>),
    Status,
}