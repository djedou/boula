use crate::{
    error::Result,
    raft_engine::raft_log::Entry
};


/// A log scan
pub type Scan<'a> = Box<dyn Iterator<Item = Result<Entry>> + 'a>;
