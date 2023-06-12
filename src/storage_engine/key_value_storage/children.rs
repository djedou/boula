use crate::storage_engine::key_value_storage::{
    Node, Values
};
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::mem::replace;

/// Root node and inner node children. The child set (node) order determines the maximum number of
/// child nodes, which is tracked via the internal vector capacity. Derefs to the child node vector.
///
/// The keys are used to guide lookups. There is always one key less than children, where the the
/// key at index i (and all keys up to the one at i+1) is contained within the child at index i+1.
/// For example:
///
/// Index  Keys  Nodes
/// 0      d     a=1,b=2,c=3        Keys:               d         f
/// 1      f     d=4,e=5            Nodes:  a=1,b=2,c=3 | d=4,e=5 | f=6,g=7
/// 2            f=6,g=7
///
/// Thus, to find the node responsible for a given key, scan the keys until encountering one greater
/// than the given key (if any) - the index of that key corresponds to the index of the node.
#[derive(Debug, PartialEq)]
pub struct Children {
    pub keys: Vec<Vec<u8>>,
    pub nodes: Vec<Node>,
}

impl Deref for Children {
    type Target = Vec<Node>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for Children {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

impl Children {
    /// Creates a new child set of the given order (maximum capacity).
    pub fn new(order: usize) -> Self {
        Self { keys: Vec::with_capacity(order - 1), nodes: Vec::with_capacity(order) }
    }

    /// Creates an empty child set, for use with replace().
    pub fn empty() -> Self {
        Self { keys: Vec::new(), nodes: Vec::new() }
    }

    /// Deletes a key from the children, if it exists.
    pub fn delete(&mut self, key: &[u8]) {
        if self.is_empty() {
            return;
        }

        // Delete the key in the relevant child.
        let (i, child) = self.lookup_mut(key);
        child.delete(key);

        // If the child does not underflow, or it has no siblings, we're done.
        if child.size() >= (child.order() + 1) / 2 || self.len() == 1 {
            return;
        }

        // Attempt to rotate or merge with the left or right siblings.
        let (size, order) = (self[i].size(), self[i].order());
        let (lsize, lorder) =
            if i > 0 { (self[i - 1].size(), self[i - 1].order()) } else { (0, 0) };
        let (rsize, rorder) =
            if i < self.len() - 1 { (self[i + 1].size(), self[i + 1].order()) } else { (0, 0) };

        if lsize > (lorder + 1) / 2 {
            self.rotate_right(i - 1);
        } else if rsize > (rorder + 1) / 2 {
            self.rotate_left(i + 1);
        } else if lsize + size <= lorder {
            self.merge(i - 1);
        } else if rsize + size <= order {
            self.merge(i);
        }
    }

    /// Fetches a value for a key, if it exists.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if !self.is_empty() {
            self.lookup(key).1.get(key)
        } else {
            None
        }
    }

    /// Fetches the first key/value pair, if any.
    pub fn get_first(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.nodes.first().and_then(|n| n.get_first())
    }

    /// Fetches the last key/value pair, if any.
    pub fn get_last(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.nodes.last().and_then(|n| n.get_last())
    }

    /// Fetches the next key/value pair after the given key, if it exists.
    pub fn get_next(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        if self.is_empty() {
            return None;
        }
        // First, look in the child responsible for the given key.
        let (i, child) = self.lookup(key);
        if let Some(item) = child.get_next(key) {
            Some(item)
        // Otherwise, try the next child.
        } else if i < self.len() - 1 {
            self[i + 1].get_next(key)
        // We don't have it.
        } else {
            None
        }
    }

    /// Fetches the previous key/value pair before the given key, if it exists.
    pub fn get_prev(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        if self.is_empty() {
            return None;
        }
        // First, look in the child responsible for the given key.
        let (i, child) = self.lookup(key);
        if let Some(item) = child.get_prev(key) {
            Some(item)
        // Otherwise, try the previous child.
        } else if i > 0 {
            self[i - 1].get_prev(key)
        // We don't have it
        } else {
            None
        }
    }

    /// Looks up the child responsible for a given key. This can only be called on non-empty
    /// child sets, which should be all child sets except for the initial root node.
    pub fn lookup(&self, key: &[u8]) -> (usize, &Node) {
        let i = self.keys.iter().position(|k| k.deref() > key).unwrap_or_else(|| self.keys.len());
        (i, &self[i])
    }

    /// Looks up the child responsible for a given key, and returns a mutable reference to it. This
    /// can only be called on non-empty child sets, which should be all child sets except for the
    /// initial root node.
    pub fn lookup_mut(&mut self, key: &[u8]) -> (usize, &mut Node) {
        let i = self.keys.iter().position(|k| k.deref() > key).unwrap_or_else(|| self.keys.len());
        (i, &mut self[i])
    }

    /// Merges the node at index i with it's right sibling.
    pub fn merge(&mut self, i: usize) {
        let parent_key = self.keys.remove(i);
        let right = &mut self.remove(i + 1);
        let left = &mut self[i];
        match (left, right) {
            (Node::Inner(lc), Node::Inner(rc)) => {
                lc.keys.push(parent_key);
                lc.keys.append(&mut rc.keys);
                lc.nodes.append(&mut rc.nodes);
            }
            (Node::Leaf(lv), Node::Leaf(rv)) => lv.append(rv),
            (left, right) => panic!("Can't merge {:?} and {:?}", left, right),
        }
    }

    /// Rotates children to the left, by transferring items from the node at the given index to
    /// its left sibling and adjusting the separator key.
    pub fn rotate_left(&mut self, i: usize) {
        if matches!(self[i], Node::Inner(_)) {
            let (key, node) = match &mut self[i] {
                Node::Inner(c) => (c.keys.remove(0), c.nodes.remove(0)),
                n => panic!("Left rotation from unexpected node {:?}", n),
            };
            let key = replace(&mut self.keys[i - 1], key); // rotate separator key
            match &mut self[i - 1] {
                Node::Inner(c) => {
                    c.keys.push(key);
                    c.nodes.push(node);
                }
                n => panic!("Left rotation into unexpected node {:?}", n),
            }
        } else if matches!(self[i], Node::Leaf(_)) {
            let (sep_key, (key, value)) = match &mut self[i] {
                Node::Leaf(v) => (v[1].0.clone(), v.remove(0)),
                n => panic!("Left rotation from unexpected node {:?}", n),
            };
            self.keys[i - 1] = sep_key;
            match &mut self[i - 1] {
                Node::Leaf(v) => v.push((key, value)),
                n => panic!("Left rotation into unexpected node {:?}", n),
            }
        } else {
            panic!("Don't know how to rotate node {:?}", self[i]);
        }
    }

    /// Rotates children to the right, by transferring items from the node at the given index to
    /// its right sibling and adjusting the separator key.
    pub fn rotate_right(&mut self, i: usize) {
        if matches!(self[i], Node::Inner(_)) {
            let (key, node) = match &mut self[i] {
                Node::Inner(c) => (c.keys.pop().unwrap(), c.nodes.pop().unwrap()),
                n => panic!("Right rotation from unexpected node {:?}", n),
            };
            let key = replace(&mut self.keys[i], key); // rotate separator key
            match &mut self[i + 1] {
                Node::Inner(c) => {
                    c.keys.insert(0, key);
                    c.nodes.insert(0, node);
                }
                n => panic!("Right rotation into unexpected node {:?}", n),
            }
        } else if matches!(self[i], Node::Leaf(_)) {
            let (key, value) = match &mut self[i] {
                Node::Leaf(v) => v.pop().unwrap(),
                n => panic!("Right rotation from unexpected node {:?}", n),
            };
            self.keys[i] = key.clone(); // update separator key
            match &mut self[i + 1] {
                Node::Leaf(v) => v.insert(0, (key, value)),
                n => panic!("Right rotation into unexpected node {:?}", n),
            }
        } else {
            panic!("Don't know how to rotate node {:?}", self[i]);
        }
    }

    /// Sets a key to a value in the children, delegating to the child responsible. If the node
    /// splits, returns the split key and new (right) node.
    pub fn set(&mut self, key: &[u8], value: Vec<u8>) -> Option<(Vec<u8>, Children)> {
        // For empty child sets, just create a new leaf node for the key.
        if self.is_empty() {
            let mut values = Values::new(self.capacity());
            values.push((key.to_vec(), value));
            self.push(Node::Leaf(values));
            return None;
        }

        // Find the child and insert the value into it. If the child splits, try to insert the
        // new right node into this child set.
        let (i, child) = self.lookup_mut(key);
        if let Some((split_key, split_child)) = child.set(key, value) {
            // The split child should be insert next to the original target.
            let insert_at = i + 1;

            // If the child set has room, just insert the split child into it. Recall that key
            // indices are one less than child nodes.
            if self.len() < self.capacity() {
                self.keys.insert(insert_at - 1, split_key.to_vec());
                self.nodes.insert(insert_at, split_child);
                return None;
            }

            // If the set is full, we need to split it and return the right node. The split-off
            // right child node goes after the original target node. We split the node in the
            // middle, but if we're inserting after the split point we move the split point by one
            // to help balance splitting of odd-ordered nodes.
            let mut split_at = self.len() / 2;
            if insert_at >= split_at {
                split_at += 1;
            }

            // Split the existing children and keys into two parts. The left parts will now have an
            // equal number of keys and children, where the last key points to the first node in
            // the right children. This last key will either have to be promoted to a split key, or
            // moved to the right keys, but keeping it here makes the arithmetic somewhat simpler.
            let mut rnodes = Vec::with_capacity(self.nodes.capacity());
            let mut rkeys = Vec::with_capacity(self.keys.capacity());
            rnodes.extend(self.nodes.drain(split_at..));
            rkeys.extend(self.keys.drain((self.keys.len() - rnodes.len() + 1)..));

            // Insert the split node and split key. Since the key is always at one index less than
            // the child, they may end up in different halves in which case the split key will be
            // promoted to a split key and the extra key from the left half is moved to the right
            // half. Otherwise, the extra key from the left half becomes the split key.
            let split_key = match insert_at.cmp(&self.nodes.len()) {
                Ordering::Greater => {
                    rkeys.insert(insert_at - 1 - self.keys.len(), split_key);
                    rnodes.insert(insert_at - self.nodes.len(), split_child);
                    self.keys.remove(self.keys.len() - 1)
                }
                Ordering::Equal => {
                    rkeys.insert(0, self.keys.remove(self.keys.len() - 1));
                    rnodes.insert(0, split_child);
                    split_key
                }
                Ordering::Less => {
                    self.keys.insert(insert_at - 1, split_key);
                    self.nodes.insert(insert_at, split_child);
                    self.keys.remove(self.keys.len() - 1)
                }
            };

            Some((split_key, Children { keys: rkeys, nodes: rnodes }))
        } else {
            None
        }
    }
}
