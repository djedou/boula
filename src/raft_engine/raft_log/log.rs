use log::debug;
use crate::{
    error::{Error, Result},
    storage_engine::{
        log_storage::{LogStore, Range}
    },
    raft_engine::{
        raft_log::{Entry, Scan, Key}
    }
};
use std::ops::RangeBounds;
use serde::{Serialize, Deserialize};


/// The replicated Raft log
pub struct RaftLog {
    /// The underlying log store.
    pub store: Box<dyn LogStore>,
    /// The index of the last stored entry.
    pub last_index: u64,
    /// The term of the last stored entry.
    pub last_term: u64,
    /// The last entry known to be committed.
    pub commit_index: u64,
    /// The term of the last committed entry.
    pub commit_term: u64,
}

impl RaftLog {
    /// Creates a new log, using a LogStore for storage.
    pub fn new(store: Box<dyn LogStore>) -> Result<Self> {
        let (commit_index, commit_term) = match store.committed() {
            0 => (0, 0),
            index => store
                .get(index)?
                .map(|v| Self::deserialize::<Entry>(&v))
                .transpose()?
                .map(|e| (e.index, e.term))
                .ok_or_else(|| Error::Internal("Committed entry not found".into()))?,
        };
        let (last_index, last_term) = match store.len() {
            0 => (0, 0),
            index => store
                .get(index)?
                .map(|v| Self::deserialize::<Entry>(&v))
                .transpose()?
                .map(|e| (e.index, e.term))
                .ok_or_else(|| Error::Internal("Last entry not found".into()))?,
        };
        Ok(Self { store, last_index, last_term, commit_index, commit_term })
    }

    /// Appends a command to the log, returning the entry.
    pub fn append(&mut self, term: u64, command: Option<Vec<u8>>) -> Result<Entry> {
        let entry = Entry { index: self.last_index + 1, term, command };
        debug!("Appending log entry {}: {:?}", entry.index, entry);
        self.store.append(Self::serialize(&entry)?)?;
        self.last_index = entry.index;
        self.last_term = entry.term;
        Ok(entry)
    }

    /// Commits entries up to and including an index.
    pub fn commit(&mut self, index: u64) -> Result<u64> {
        let entry = self
            .get(index)?
            .ok_or_else(|| Error::Internal(format!("Entry {} not found", index)))?;
        self.store.commit(index)?;
        self.commit_index = entry.index;
        self.commit_term = entry.term;
        Ok(index)
    }

    /// Fetches an entry at an index
    pub fn get(&self, index: u64) -> Result<Option<Entry>> {
        self.store.get(index)?.map(|v| Self::deserialize(&v)).transpose()
    }

    /// Checks if the log contains an entry
    pub fn has(&self, index: u64, term: u64) -> Result<bool> {
        match self.get(index)? {
            Some(entry) => Ok(entry.term == term),
            None if index == 0 && term == 0 => Ok(true),
            None => Ok(false),
        }
    }

    /// Iterates over log entries
    pub fn scan(&self, range: impl RangeBounds<u64>) -> Scan {
        Box::new(self.store.scan(Range::from(range)).map(|r| r.and_then(|v| Self::deserialize(&v))))
    }

    /// Splices a set of entries onto an offset. The entries must be contiguous, and the first entry
    /// must be at most last_index+1. If an entry does not exist, append it. If an existing entry
    /// has a term mismatch, replace it and all following entries.
    pub fn splice(&mut self, entries: Vec<Entry>) -> Result<u64> {
        for i in 0..entries.len() {
            if i == 0 && entries.get(i).unwrap().index > self.last_index + 1 {
                return Err(Error::Internal("Spliced entries cannot begin past last index".into()));
            }
            if entries.get(i).unwrap().index != entries.get(0).unwrap().index + i as u64 {
                return Err(Error::Internal("Spliced entries must be contiguous".into()));
            }
        }
        for entry in entries {
            if let Some(ref current) = self.get(entry.index)? {
                if current.term == entry.term {
                    continue;
                }
                self.truncate(entry.index - 1)?;
            }
            self.append(entry.term, entry.command)?;
        }
        Ok(self.last_index)
    }

    /// Truncates the log such that its last item is at most index.
    /// Refuses to remove entries that have been applied or committed.
    pub fn truncate(&mut self, index: u64) -> Result<u64> {
        debug!("Truncating log from entry {}", index);
        let (index, term) = match self.store.truncate(index)? {
            0 => (0, 0),
            i => self
                .store
                .get(i)?
                .map(|v| Self::deserialize::<Entry>(&v))
                .transpose()?
                .map(|e| (e.index, e.term))
                .ok_or_else(|| Error::Internal(format!("Entry {} not found", index)))?,
        };
        self.last_index = index;
        self.last_term = term;
        Ok(index)
    }

    /// Loads information about the most recent term known by the log, containing the term number (0
    /// if none) and candidate voted for in current term (if any).
    pub fn load_term(&self) -> Result<(u64, Option<String>)> {
        let (term, voted_for) = self
            .store
            .get_metadata(&Key::TermVote.encode())?
            .map(|v| Self::deserialize(&v))
            .transpose()?
            .unwrap_or((0, None));
        debug!("Loaded term {} and voted_for {:?} from log", term, voted_for);
        Ok((term, voted_for))
    }

    /// Saves information about the most recent term.
    pub fn save_term(&mut self, term: u64, voted_for: Option<&str>) -> Result<()> {
        self.store.set_metadata(&Key::TermVote.encode(), Self::serialize(&(term, voted_for))?)
    }

    /// Serializes a value for the log store.
    fn serialize<V: Serialize>(value: &V) -> Result<Vec<u8>> {
        Ok(bincode::serialize(value)?)
    }

    /// Deserializes a value from the log store.
    fn deserialize<'a, V: Deserialize<'a>>(bytes: &'a [u8]) -> Result<V> {
        Ok(bincode::deserialize(bytes)?)
    }
}