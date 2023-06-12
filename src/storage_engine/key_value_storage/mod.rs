mod children;
mod iterator;
mod memory;
mod node;
mod range;
mod scan;
mod store;
mod value;

pub use children::*;
pub use iterator::*;
pub use memory::*;
pub use node::*;
pub use range::*;
pub use scan::*;
pub use store::*;
pub use value::*;

/*
#[cfg(test)]
mod test;

#[cfg(test)]
pub use test::Test;
*/

#[cfg(test)]
mod key_value_storage_tests {
    use super::{
        children::*,
        memory::*,
        node::*,
        value::*
    };
    use crate::error::Result;
    use pretty_assertions::assert_eq;
/*
    impl super::super::TestSuite<Memory> for Memory {
        fn setup() -> Result<Self> {
            Ok(Memory::new())
        }
    }

    #[test]
    fn tests() -> Result<()> {
        use super::super::TestSuite;
        Memory::test()
    }*/

    #[test]
    fn set_split() -> Result<()> {
        // Create a root of order 3
        let mut root = Node::Root(Children::new(3));

        // A new root should be empty
        assert_eq!(Node::Root(Children { keys: vec![], nodes: vec![] }), root);

        // Setting the first three values should create a leaf node and fill it
        root.set(b"a", vec![0x01]);
        root.set(b"b", vec![0x02]);
        root.set(b"c", vec![0x03]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![],
                nodes: vec![Node::Leaf(Values(vec![
                    (b"a".to_vec(), vec![0x01]),
                    (b"b".to_vec(), vec![0x02]),
                    (b"c".to_vec(), vec![0x03]),
                ]),)],
            }),
            root
        );

        // Updating a node should not cause splitting
        root.set(b"b", vec![0x20]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![],
                nodes: vec![Node::Leaf(Values(vec![
                    (b"a".to_vec(), vec![0x01]),
                    (b"b".to_vec(), vec![0x20]),
                    (b"c".to_vec(), vec![0x03]),
                ]))],
            }),
            root
        );

        // Setting an additional value should split the leaf node
        root.set(b"b", vec![0x02]);
        root.set(b"d", vec![0x04]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"c".to_vec()],
                nodes: vec![
                    Node::Leaf(Values(vec![
                        (b"a".to_vec(), vec![0x01]),
                        (b"b".to_vec(), vec![0x02]),
                    ])),
                    Node::Leaf(Values(vec![
                        (b"c".to_vec(), vec![0x03]),
                        (b"d".to_vec(), vec![0x04]),
                    ])),
                ],
            }),
            root
        );

        // Adding two more values at the end should split the second leaf
        root.set(b"z", vec![0x1a]);
        root.set(b"y", vec![0x19]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"c".to_vec(), b"y".to_vec()],
                nodes: vec![
                    Node::Leaf(Values(vec![
                        (b"a".to_vec(), vec![0x01]),
                        (b"b".to_vec(), vec![0x02]),
                    ])),
                    Node::Leaf(Values(vec![
                        (b"c".to_vec(), vec![0x03]),
                        (b"d".to_vec(), vec![0x04]),
                    ])),
                    Node::Leaf(Values(vec![
                        (b"y".to_vec(), vec![0x19]),
                        (b"z".to_vec(), vec![0x1a]),
                    ])),
                ],
            }),
            root
        );

        // Adding two more values from the end should split the middle leaf. This will cause the
        // root node to overflow and split as well.
        root.set(b"x", vec![0x18]);
        root.set(b"w", vec![0x17]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"w".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"c".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"a".to_vec(), vec![0x01]),
                                (b"b".to_vec(), vec![0x02]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"y".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"w".to_vec(), vec![0x17]),
                                (b"x".to_vec(), vec![0x18]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"y".to_vec(), vec![0x19]),
                                (b"z".to_vec(), vec![0x1a]),
                            ])),
                        ],
                    })
                ],
            }),
            root
        );

        // Adding further values should cause the first inner node to finally split as well.
        root.set(b"e", vec![0x05]);
        root.set(b"f", vec![0x06]);
        root.set(b"g", vec![0x07]);
        root.set(b"h", vec![0x08]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"e".to_vec(), b"w".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"c".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"a".to_vec(), vec![0x01]),
                                (b"b".to_vec(), vec![0x02]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"g".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"h".to_vec(), vec![0x08]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"y".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"w".to_vec(), vec![0x17]),
                                (b"x".to_vec(), vec![0x18]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"y".to_vec(), vec![0x19]),
                                (b"z".to_vec(), vec![0x1a]),
                            ])),
                        ],
                    })
                ],
            }),
            root
        );

        // Adding yet more from the back, but in forward order, should cause another root node split.
        root.set(b"s", vec![0x13]);
        root.set(b"t", vec![0x14]);
        root.set(b"u", vec![0x15]);
        root.set(b"v", vec![0x16]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"s".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"e".to_vec()],
                        nodes: vec![
                            Node::Inner(Children {
                                keys: vec![b"c".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"a".to_vec(), vec![0x01]),
                                        (b"b".to_vec(), vec![0x02]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"c".to_vec(), vec![0x03]),
                                        (b"d".to_vec(), vec![0x04]),
                                    ]))
                                ],
                            }),
                            Node::Inner(Children {
                                keys: vec![b"g".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"e".to_vec(), vec![0x05]),
                                        (b"f".to_vec(), vec![0x06]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"g".to_vec(), vec![0x07]),
                                        (b"h".to_vec(), vec![0x08]),
                                    ]))
                                ],
                            }),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"w".to_vec()],
                        nodes: vec![
                            Node::Inner(Children {
                                keys: vec![b"u".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"s".to_vec(), vec![0x13]),
                                        (b"t".to_vec(), vec![0x14]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"u".to_vec(), vec![0x15]),
                                        (b"v".to_vec(), vec![0x16]),
                                    ]))
                                ],
                            }),
                            Node::Inner(Children {
                                keys: vec![b"y".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"w".to_vec(), vec![0x17]),
                                        (b"x".to_vec(), vec![0x18]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"y".to_vec(), vec![0x19]),
                                        (b"z".to_vec(), vec![0x1a]),
                                    ]))
                                ],
                            })
                        ]
                    })
                ],
            }),
            root
        );

        Ok(())
    }

    #[test]
    fn delete_merge() -> Result<()> {
        // Create a root of order 3, and add a bunch of values to it.
        let mut root = Node::Root(Children::new(3));

        root.set(b"a", vec![0x01]);
        root.set(b"b", vec![0x02]);
        root.set(b"c", vec![0x03]);
        root.set(b"d", vec![0x04]);
        root.set(b"e", vec![0x05]);
        root.set(b"f", vec![0x06]);
        root.set(b"g", vec![0x07]);
        root.set(b"h", vec![0x08]);
        root.set(b"i", vec![0x09]);
        root.set(b"j", vec![0x0a]);
        root.set(b"k", vec![0x0b]);
        root.set(b"l", vec![0x0c]);
        root.set(b"m", vec![0x0d]);
        root.set(b"n", vec![0x0e]);
        root.set(b"o", vec![0x0f]);
        root.set(b"p", vec![0x10]);

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"i".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"e".to_vec()],
                        nodes: vec![
                            Node::Inner(Children {
                                keys: vec![b"c".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"a".to_vec(), vec![0x01]),
                                        (b"b".to_vec(), vec![0x02]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"c".to_vec(), vec![0x03]),
                                        (b"d".to_vec(), vec![0x04]),
                                    ])),
                                ],
                            }),
                            Node::Inner(Children {
                                keys: vec![b"g".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"e".to_vec(), vec![0x05]),
                                        (b"f".to_vec(), vec![0x06]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"g".to_vec(), vec![0x07]),
                                        (b"h".to_vec(), vec![0x08]),
                                    ])),
                                ],
                            }),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"m".to_vec()],
                        nodes: vec![
                            Node::Inner(Children {
                                keys: vec![b"k".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"i".to_vec(), vec![0x09]),
                                        (b"j".to_vec(), vec![0x0a]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"k".to_vec(), vec![0x0b]),
                                        (b"l".to_vec(), vec![0x0c]),
                                    ])),
                                ],
                            }),
                            Node::Inner(Children {
                                keys: vec![b"o".to_vec()],
                                nodes: vec![
                                    Node::Leaf(Values(vec![
                                        (b"m".to_vec(), vec![0x0d]),
                                        (b"n".to_vec(), vec![0x0e]),
                                    ])),
                                    Node::Leaf(Values(vec![
                                        (b"o".to_vec(), vec![0x0f]),
                                        (b"p".to_vec(), vec![0x10]),
                                    ])),
                                ],
                            })
                        ]
                    })
                ],
            }),
            root
        );

        // Deleting the o node merges two leaf nodes, in turn merging parents.
        root.delete(b"o");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"e".to_vec(), b"i".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"c".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"a".to_vec(), vec![0x01]),
                                (b"b".to_vec(), vec![0x02]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"g".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"h".to_vec(), vec![0x08]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"k".to_vec(), b"m".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"i".to_vec(), vec![0x09]),
                                (b"j".to_vec(), vec![0x0a]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"k".to_vec(), vec![0x0b]),
                                (b"l".to_vec(), vec![0x0c]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"m".to_vec(), vec![0x0d]),
                                (b"n".to_vec(), vec![0x0e]),
                                (b"p".to_vec(), vec![0x10]),
                            ])),
                        ],
                    }),
                ],
            }),
            root
        );

        // Deleting i causes another leaf merge
        root.delete(b"i");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"e".to_vec(), b"i".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"c".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"a".to_vec(), vec![0x01]),
                                (b"b".to_vec(), vec![0x02]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"g".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"h".to_vec(), vec![0x08]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"m".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"j".to_vec(), vec![0x0a]),
                                (b"k".to_vec(), vec![0x0b]),
                                (b"l".to_vec(), vec![0x0c]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"m".to_vec(), vec![0x0d]),
                                (b"n".to_vec(), vec![0x0e]),
                                (b"p".to_vec(), vec![0x10]),
                            ])),
                        ],
                    }),
                ],
            }),
            root
        );

        // Clearing out j,k,l should cause another merge.
        root.delete(b"j");
        root.delete(b"l");
        root.delete(b"k");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"e".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"c".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"a".to_vec(), vec![0x01]),
                                (b"b".to_vec(), vec![0x02]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"g".to_vec(), b"i".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"h".to_vec(), vec![0x08]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"m".to_vec(), vec![0x0d]),
                                (b"n".to_vec(), vec![0x0e]),
                                (b"p".to_vec(), vec![0x10]),
                            ])),
                        ],
                    }),
                ],
            }),
            root
        );

        // Removing a should underflow a leaf node, triggering a rotation to rebalance the
        // underflowing inner node with its sibling.
        root.delete(b"a");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"g".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"e".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"b".to_vec(), vec![0x02]),
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"i".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"h".to_vec(), vec![0x08]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"m".to_vec(), vec![0x0d]),
                                (b"n".to_vec(), vec![0x0e]),
                                (b"p".to_vec(), vec![0x10]),
                            ])),
                        ],
                    }),
                ],
            }),
            root
        );

        // Removing h should rebalance the leaf nodes.
        root.delete(b"h");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"g".to_vec()],
                nodes: vec![
                    Node::Inner(Children {
                        keys: vec![b"e".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"b".to_vec(), vec![0x02]),
                                (b"c".to_vec(), vec![0x03]),
                                (b"d".to_vec(), vec![0x04]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"e".to_vec(), vec![0x05]),
                                (b"f".to_vec(), vec![0x06]),
                            ])),
                        ],
                    }),
                    Node::Inner(Children {
                        keys: vec![b"n".to_vec()],
                        nodes: vec![
                            Node::Leaf(Values(vec![
                                (b"g".to_vec(), vec![0x07]),
                                (b"m".to_vec(), vec![0x0d]),
                            ])),
                            Node::Leaf(Values(vec![
                                (b"n".to_vec(), vec![0x0e]),
                                (b"p".to_vec(), vec![0x10]),
                            ])),
                        ],
                    }),
                ],
            }),
            root
        );

        // Removing n should rebalance the leaf nodes, and in turn merge the inner nodes.
        root.delete(b"n");

        assert_eq!(
            Node::Root(Children {
                keys: vec![b"e".to_vec(), b"g".to_vec()],
                nodes: vec![
                    Node::Leaf(Values(vec![
                        (b"b".to_vec(), vec![0x02]),
                        (b"c".to_vec(), vec![0x03]),
                        (b"d".to_vec(), vec![0x04]),
                    ])),
                    Node::Leaf(Values(vec![
                        (b"e".to_vec(), vec![0x05]),
                        (b"f".to_vec(), vec![0x06]),
                    ])),
                    Node::Leaf(Values(vec![
                        (b"g".to_vec(), vec![0x07]),
                        (b"m".to_vec(), vec![0x0d]),
                        (b"p".to_vec(), vec![0x10]),
                    ])),
                ],
            }),
            root
        );

        // At this point we can remove the remaining keys, leaving an empty root node.
        root.delete(b"d");
        root.delete(b"p");
        root.delete(b"g");
        root.delete(b"c");
        root.delete(b"f");
        root.delete(b"m");
        root.delete(b"b");
        root.delete(b"e");

        assert_eq!(Node::Root(Children { keys: vec![], nodes: vec![] }), root);

        Ok(())
    }
}
