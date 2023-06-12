use std::ops::{Bound, RangeBounds};


/// A scan range.
pub struct Range {
    pub start: Bound<u64>,
    pub end: Bound<u64>,
}

impl Range {
    /// Creates a new range from the given Rust range. We can't use the RangeBounds directly in
    /// scan() since that prevents us from Store into a trait object.
    pub fn from(range: impl RangeBounds<u64>) -> Self {
        Self {
            start: match range.start_bound() {
                Bound::Included(v) => Bound::Included(*v),
                Bound::Excluded(v) => Bound::Excluded(*v),
                Bound::Unbounded => Bound::Unbounded,
            },
            end: match range.end_bound() {
                Bound::Included(v) => Bound::Included(*v),
                Bound::Excluded(v) => Bound::Excluded(*v),
                Bound::Unbounded => Bound::Unbounded,
            },
        }
    }
}