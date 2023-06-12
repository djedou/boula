use crate::error::Result;

/// Iterator over a log range.
pub type Scan<'a> = Box<dyn Iterator<Item = Result<Vec<u8>>> + 'a>;
