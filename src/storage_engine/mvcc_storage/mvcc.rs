use std::sync::{Arc, RwLock};
use crate::{
    error::Result,
    storage_engine::mvcc_storage::{
        Key, deserialize, Transaction, Mode, Status
    },
    storage_engine::key_value_storage::{
        KvStore, Range
    }
};

/// An MVCC-based transactional key-value store.
pub struct MVCC {
    /// The underlying KV store. It is protected by a mutex so it can be shared between txns.
    store: Arc<RwLock<Box<dyn KvStore>>>,
}

impl Clone for MVCC {
    fn clone(&self) -> Self {
        MVCC { store: self.store.clone() }
    }
}

impl MVCC {
    /// Creates a new MVCC key-value store with the given key-value store for storage.
    pub fn new(store: Box<dyn KvStore>) -> Self {
        Self { store: Arc::new(RwLock::new(store)) }
    }

    /// Begins a new transaction in read-write mode.
    #[allow(dead_code)]
    pub fn begin(&self) -> Result<Transaction> {
        Transaction::begin(self.store.clone(), Mode::ReadWrite)
    }

    /// Begins a new transaction in the given mode.
    pub fn begin_with_mode(&self, mode: Mode) -> Result<Transaction> {
        Transaction::begin(self.store.clone(), mode)
    }

    /// Resumes a transaction with the given ID.
    pub fn resume(&self, id: u64) -> Result<Transaction> {
        Transaction::resume(self.store.clone(), id)
    }

    /// Fetches an unversioned metadata value
    pub fn get_metadata(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let session = self.store.read()?;
        session.get(&Key::Metadata(key.into()).encode())
    }

    /// Sets an unversioned metadata value
    pub fn set_metadata(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        let mut session = self.store.write()?;
        session.set(&Key::Metadata(key.into()).encode(), value)
    }

    /// Returns engine status
    //
    // Bizarrely, the return statement is in fact necessary - see:
    // https://github.com/rust-lang/reference/issues/452
    #[allow(clippy::needless_return)]
    pub fn status(&self) -> Result<Status> {
        let store = self.store.read()?;
        return Ok(Status {
            txns: match store.get(&Key::TxnNext.encode())? {
                Some(ref v) => deserialize(v)?,
                None => 1,
            } - 1,
            txns_active: store
                .scan(Range::from(
                    Key::TxnActive(0).encode()..Key::TxnActive(std::u64::MAX).encode(),
                ))
                .try_fold(0, |count, r| r.map(|_| count + 1))?,
            storage: store.to_string(),
        });
    }
}