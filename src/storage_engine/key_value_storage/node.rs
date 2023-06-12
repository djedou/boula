use crate::storage_engine::key_value_storage::{
    Children, Values
};
use std::mem::replace;

/// B-tree node variants. Most internal logic is delegated to the contained Children/Values structs,
/// while this outer structure manages the overall tree, particularly root special-casing.
///
/// All nodes in a tree have the same order (i.e. the same maximum number of children/values). The
/// root node can contain anywhere between 0 and the maximum number of items, while inner and leaf
/// nodes try to stay between order/2 and order items.
#[derive(Debug, PartialEq)]
pub enum Node {
    Root(Children),
    Inner(Children),
    Leaf(Values),
}

impl Node {
    /// Deletes a key from the node, if it exists.
    pub fn delete(&mut self, key: &[u8]) {
        match self {
            Self::Root(children) => {
                children.delete(key);
                // If we now have a single child, pull it up into the root.
                while children.len() == 1 && matches!(children[0], Node::Inner { .. }) {
                    if let Node::Inner(c) = children.remove(0) {
                        *children = c;
                    }
                }
                // If we have a single empty child, remove it.
                if children.len() == 1 && children[0].size() == 0 {
                    children.remove(0);
                }
            }
            Self::Inner(children) => children.delete(key),
            Self::Leaf(values) => values.delete(key),
        }
    }

    /// Fetches a value for a key, if it exists.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self {
            Self::Root(children) | Self::Inner(children) => children.get(key),
            Self::Leaf(values) => values.get(key),
        }
    }

    /// Fetches the first key/value pair, if any.
    pub fn get_first(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Root(children) | Self::Inner(children) => children.get_first(),
            Self::Leaf(values) => values.get_first(),
        }
    }

    /// Fetches the last key/value pair, if any.
    pub fn get_last(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Root(children) | Self::Inner(children) => children.get_last(),
            Self::Leaf(values) => values.get_last(),
        }
    }

    /// Fetches the next key/value pair after the given key.
    pub fn get_next(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Root(children) | Self::Inner(children) => children.get_next(key),
            Self::Leaf(values) => values.get_next(key),
        }
    }

    /// Fetches the previous key/value pair before the given key.
    pub fn get_prev(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Root(children) | Self::Inner(children) => children.get_prev(key),
            Self::Leaf(values) => values.get_prev(key),
        }
    }

    /// Sets a key to a value in the node, inserting or updating the key as appropriate. If the
    /// node splits, return the split key and new (right) node.
    pub fn set(&mut self, key: &[u8], value: Vec<u8>) -> Option<(Vec<u8>, Node)> {
        match self {
            Self::Root(ref mut children) => {
                // Set the key/value pair in the children. If the children split, create a new
                // child set for the root node with two new inner nodes for the split children.
                if let Some((split_key, split_children)) = children.set(key, value) {
                    let mut root_children = Children::new(children.capacity());
                    root_children.keys.push(split_key);
                    root_children.nodes.push(Node::Inner(replace(children, Children::empty())));
                    root_children.nodes.push(Node::Inner(split_children));
                    *children = root_children;
                }
                None
            }
            Self::Inner(children) => children.set(key, value).map(|(sk, c)| (sk, Node::Inner(c))),
            Self::Leaf(values) => values.set(key, value).map(|(sk, v)| (sk, Node::Leaf(v))),
        }
    }

    /// Returns the order (i.e. capacity) of the node.
    pub fn order(&self) -> usize {
        match self {
            Self::Root(children) | Self::Inner(children) => children.capacity(),
            Self::Leaf(values) => values.capacity(),
        }
    }

    /// Returns the size (number of items) of the node.
    pub fn size(&self) -> usize {
        match self {
            Self::Root(children) | Self::Inner(children) => children.len(),
            Self::Leaf(values) => values.len(),
        }
    }
}
