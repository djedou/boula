use std::collections::HashSet;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use crate::{
    error::{Error, Result},
    storage_engine::mvcc_storage::{
        Key, deserialize, serialize
    },
    storage_engine::key_value_storage::{
        KvStore, Range
    }
};


/// A versioned snapshot, containing visibility information about concurrent transactions.
#[derive(Clone)]
pub struct Snapshot {
    /// The version (i.e. transaction ID) that the snapshot belongs to.
    pub version: u64,
    /// The set of transaction IDs that were active at the start of the transactions,
    /// and thus should be invisible to the snapshot.
    pub invisible: HashSet<u64>,
}

impl Snapshot {
    /// Takes a new snapshot, persisting it as `Key::TxnSnapshot(version)`.
    pub fn take(session: &mut RwLockWriteGuard<Box<dyn KvStore>>, version: u64) -> Result<Self> {
        let mut snapshot = Self { version, invisible: HashSet::new() };
        let mut scan =
            session.scan(Range::from(Key::TxnActive(0).encode()..Key::TxnActive(version).encode()));
        while let Some((key, _)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::TxnActive(id) => snapshot.invisible.insert(id),
                k => return Err(Error::Internal(format!("Expected TxnActive, got {:?}", k))),
            };
        }
        std::mem::drop(scan);
        session.set(&Key::TxnSnapshot(version).encode(), serialize(&snapshot.invisible)?)?;
        Ok(snapshot)
    }

    /// Restores an existing snapshot from `Key::TxnSnapshot(version)`, or errors if not found.
    pub fn restore(session: &RwLockReadGuard<Box<dyn KvStore>>, version: u64) -> Result<Self> {
        match session.get(&Key::TxnSnapshot(version).encode())? {
            Some(ref v) => Ok(Self { version, invisible: deserialize(v)? }),
            None => Err(Error::Value(format!("Snapshot not found for version {}", version))),
        }
    }

    /// Checks whether the given version is visible in this snapshot.
    pub fn is_visible(&self, version: u64) -> bool {
        version <= self.version && self.invisible.get(&version).is_none()
    }
}