use crate::storage_engine::key_value_storage::{
    Node, Iter, Scan, Range, Children, KvStore
};
use std::sync::{Arc, RwLock};
use std::fmt::Display;
use crate::error::{Error, Result};

/// The default B+tree order, i.e. maximum number of children per node.
const DEFAULT_ORDER: usize = 8;

/// In-memory key-value store using a B+tree. The B+tree is a variant of a binary search tree with
/// lookup keys in inner nodes and actual key/value pairs on the leaf nodes. Each node has several
/// children and search keys, to make use of cache locality, up to a maximum known as the tree's
/// order. Leaf and inner nodes contain between order/2 and order items, and will be split, rotated,
/// or merged as appropriate, while the root node can have between 0 and order children.
///
/// This implementation differs from a standard B+tree in that leaf nodes do not have pointers to
/// the sibling leaf nodes. Iterator traversal is instead done via lookups from the root node. This
/// has O(log n) complexity rather than O(1) for iterators, but is left as a future performance
/// optimization if it is shown to be necessary.
pub struct KvMemory {
    /// The tree root, guarded by an RwLock to support multiple iterators across it.
    root: Arc<RwLock<Node>>,
}

impl Display for KvMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "memory")
    }
}

impl KvMemory {
    /// Creates a new in-memory store using the default order.
    pub fn new() -> Self {
        Self::new_with_order(DEFAULT_ORDER).unwrap()
    }

    /// Creates a new in-memory store using the given order.
    pub fn new_with_order(order: usize) -> Result<Self> {
        if order < 2 {
            return Err(Error::Internal("Order must be at least 2".into()));
        }
        Ok(Self { root: Arc::new(RwLock::new(Node::Root(Children::new(order)))) })
    }
}

impl KvStore for KvMemory {
    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.root.write()?.delete(key);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.root.read()?.get(key))
    }

    fn scan(&self, range: Range) -> Scan {
        Box::new(Iter::new(self.root.clone(), range))
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.root.write()?.set(key, value);
        Ok(())
    }
}
