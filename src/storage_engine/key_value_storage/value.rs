use std::ops::{Deref, DerefMut};
use std::cmp::Ordering;

/// Leaf node key/value pairs. The value set (leaf node) order determines the maximum number
/// of key/value items, which is tracked via the internal vector capacity. Items are ordered by key,
/// and looked up via linear search due to the low cardinality. Derefs to the inner vec.
#[derive(Debug, PartialEq)]
pub struct Values(pub Vec<(Vec<u8>, Vec<u8>)>);

impl Deref for Values {
    type Target = Vec<(Vec<u8>, Vec<u8>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Values {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Values {
    /// Creates a new value set with the given order (maximum capacity).
    pub fn new(order: usize) -> Self {
        Self(Vec::with_capacity(order))
    }

    /// Deletes a key from the set, if it exists.
    pub fn delete(&mut self, key: &[u8]) {
        for (i, (k, _)) in self.iter().enumerate() {
            match (&**k).cmp(key) {
                Ordering::Greater => break,
                Ordering::Equal => {
                    self.remove(i);
                    break;
                }
                Ordering::Less => {}
            }
        }
    }

    /// Fetches a value from the set, if the key exists.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.iter()
            .find_map(|(k, v)| match (&**k).cmp(key) {
                Ordering::Greater => Some(None),
                Ordering::Equal => Some(Some(v.to_vec())),
                Ordering::Less => None,
            })
            .flatten()
    }

    /// Fetches the first key/value pair from the set, if any.
    pub fn get_first(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.0.first().cloned()
    }

    /// Fetches the last key/value pair from the set, if any.
    pub fn get_last(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.0.last().cloned()
    }

    /// Fetches the next value after the given key, if it exists.
    pub fn get_next(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        self.iter()
            .find_map(|(k, v)| match (&**k).cmp(key) {
                Ordering::Greater => Some(Some((k.to_vec(), v.to_vec()))),
                Ordering::Equal => None,
                Ordering::Less => None,
            })
            .flatten()
    }

    /// Fetches the previous value before the given key, if it exists.
    pub fn get_prev(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        self.iter()
            .rev()
            .find_map(|(k, v)| match (&**k).cmp(key) {
                Ordering::Less => Some(Some((k.to_vec(), v.to_vec()))),
                Ordering::Equal => None,
                Ordering::Greater => None,
            })
            .flatten()
    }

    /// Sets a key to a value, inserting of updating it. If the value set is full, it is split
    /// in the middle and the split key and right values are returned.
    pub fn set(&mut self, key: &[u8], value: Vec<u8>) -> Option<(Vec<u8>, Values)> {
        // Find position to insert at, or if the key already exists just update it.
        let mut insert_at = self.len();
        for (i, (k, v)) in self.iter_mut().enumerate() {
            match (&**k).cmp(key) {
                Ordering::Greater => {
                    insert_at = i;
                    break;
                }
                Ordering::Equal => {
                    *v = value;
                    return None;
                }
                Ordering::Less => {}
            }
        }

        // If we have capacity, just insert the value.
        if self.len() < self.capacity() {
            self.insert(insert_at, (key.to_vec(), value));
            return None;
        }

        // If we're full, split in the middle and return split key and right values. If inserting
        // to the right of the split, move split by one to better balance odd-ordered nodes.
        let mut split_at = self.len() / 2;
        if insert_at >= split_at {
            split_at += 1;
        }
        let mut rvalues = Values::new(self.capacity());
        rvalues.extend(self.drain(split_at..));
        if insert_at >= self.len() {
            rvalues.insert(insert_at - self.len(), (key.to_vec(), value));
        } else {
            self.insert(insert_at, (key.to_vec(), value));
        }
        Some((rvalues[0].0.clone(), rvalues))
    }
}
