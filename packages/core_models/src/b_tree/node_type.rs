

#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    BNodeRoot, // = 0,  u16
    BNodeNode, // = 1,  u16
    BNodeLeaf, // = 2,  u16
    Unknowed // = f, u16
}

impl From<&[u8; 2]> for NodeType {
    fn from(value: &[u8; 2]) -> NodeType {
        match value {
            [0x0, 0x0] => NodeType::BNodeRoot,
            [0x0, 0x1] => NodeType::BNodeNode,
            [0x0, 0x2] => NodeType::BNodeLeaf,
            _ => NodeType::Unknowed
        }
    }
}


impl From<NodeType> for [u8; 2] {
    fn from(value: NodeType) -> Self {
        match value {
            NodeType::BNodeRoot => [0x0, 0x0],
            NodeType::BNodeNode => [0x0, 0x1],
            NodeType::BNodeLeaf => [0x0, 0x2],
            NodeType::Unknowed => [0xf, 0xf],
        }
    }
}

#[cfg(test)]
mod node_type_tests {
    use super::*;
    use more_asserts::{assert_le, assert_ge};


    #[test]
    fn node_type_node_eq_hex() {
        let node_type = NodeType::from(&[0x0, 0x1]);
        assert_eq!(node_type, NodeType::BNodeNode);
    }

    #[test]
    fn node_type_node_to_bytes_eq_hex() {
        let node_type = NodeType::BNodeNode;
        let node_type_byte: [u8; 2] = node_type.into();
        assert_eq!(node_type_byte, [0x0, 0x1]);
    }

    #[test]
    fn node_type_eq_dec() {
        let node_type = NodeType::from(&[0, 1]);
        assert_eq!(node_type, NodeType::BNodeNode);
    }

    #[test]
    fn node_type_root_eq_hex() {
        let node_type = NodeType::from(&[0x0, 0x0]);
        assert_eq!(node_type, NodeType::BNodeRoot);
    }

    #[test]
    fn node_type_root_to_bytes_eq_hex() {
        let node_type = NodeType::BNodeRoot;
        let node_type_byte: [u8; 2] = node_type.into();
        assert_eq!(node_type_byte, [0x0, 0x0]);
    }

    #[test]
    fn node_type_leaf_eq_hex() {
        let node_type = NodeType::from(&[0x0, 0x2]);
        assert_eq!(node_type, NodeType::BNodeLeaf);
    }

    #[test]
    fn node_type_leaf_to_bytes_eq_hex() {
        let node_type = NodeType::BNodeLeaf;
        let node_type_byte: [u8; 2] = node_type.into();
        assert_eq!(node_type_byte, [0x0, 0x2]);
    }

    #[test]
    fn node_type_unknowed_eq_hex() {
        let node_type = NodeType::from(&[0x0, 0x8]);
        assert_eq!(node_type, NodeType::Unknowed);
    }

    #[test]
    fn node_type_unknowed_to_bytes_eq_hex() {
        let node_type = NodeType::Unknowed;
        let node_type_byte: [u8; 2] = node_type.into();
        assert_eq!(node_type_byte, [0xf, 0xf]);
    }
}