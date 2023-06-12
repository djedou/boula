use crate::error::Result;
use serde::{Serialize, Deserialize};

/// Serializes MVCC metadata.
pub fn serialize<V: Serialize>(value: &V) -> Result<Vec<u8>> {
    Ok(bincode::serialize(value)?)
}

/// Deserializes MVCC metadata.
pub fn deserialize<'a, V: Deserialize<'a>>(bytes: &'a [u8]) -> Result<V> {
    Ok(bincode::deserialize(bytes)?)
}