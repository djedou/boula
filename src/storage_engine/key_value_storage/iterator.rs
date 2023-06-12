use crate::error::Result;
use crate::storage_engine::key_value_storage::{
    Node, Range
};
use std::ops::Bound;
use std::sync::{Arc, RwLock};



/// A key range scan.
/// FIXME This is O(log n), and should use the normal B+tree approach of storing pointers in
/// the leaf nodes instead. See: https://github.com/erikgrinaker/toydb/issues/32
pub struct Iter {
    /// The root node of the tree we're iterating across.
    root: Arc<RwLock<Node>>,
    /// The range we're iterating over.
    range: Range,
    /// The front cursor keeps track of the last returned value from the front.
    front_cursor: Option<Vec<u8>>,
    /// The back cursor keeps track of the last returned value from the back.
    back_cursor: Option<Vec<u8>>,
}

impl Iter {
    /// Creates a new iterator.
    pub fn new(root: Arc<RwLock<Node>>, range: Range) -> Self {
        Self { root, range, front_cursor: None, back_cursor: None }
    }

    // next() with error handling.
    pub fn try_next(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let root = self.root.read()?;
        let next = match &self.front_cursor {
            None => match &self.range.start {
                Bound::Included(k) => {
                    root.get(k).map(|v| (k.clone(), v)).or_else(|| root.get_next(k))
                }
                Bound::Excluded(k) => root.get_next(k),
                Bound::Unbounded => root.get_first(),
            },
            Some(k) => root.get_next(k),
        };
        if let Some((k, _)) = &next {
            if !self.range.contains(k) {
                return Ok(None);
            }
            if let Some(bc) = &self.back_cursor {
                if bc <= k {
                    return Ok(None);
                }
            }
            self.front_cursor = Some(k.clone())
        }
        Ok(next)
    }

    /// next_back() with error handling.
    pub fn try_next_back(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let root = self.root.read()?;
        let prev = match &self.back_cursor {
            None => match &self.range.end {
                Bound::Included(k) => {
                    root.get(k).map(|v| (k.clone(), v)).or_else(|| root.get_prev(k))
                }
                Bound::Excluded(k) => root.get_prev(k),
                Bound::Unbounded => root.get_last(),
            },
            Some(k) => root.get_prev(k),
        };
        if let Some((k, _)) = &prev {
            if !self.range.contains(k) {
                return Ok(None);
            }
            if let Some(fc) = &self.front_cursor {
                if fc >= k {
                    return Ok(None);
                }
            }
            self.back_cursor = Some(k.clone())
        }
        Ok(prev)
    }
}

impl Iterator for Iter {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.try_next_back().transpose()
    }
}