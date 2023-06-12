use crate::error::Result;

/// Iterator over a key/value range.
pub type Scan = Box<dyn DoubleEndedIterator<Item = Result<(Vec<u8>, Vec<u8>)>> + Send>;
