use super::{NodeType};
use bytes::Bytes;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};



#[derive(Debug, PartialEq, Eq)]
pub struct BNode {
    pub data: Bytes
}



impl BNode {
    pub fn load_from_bytes(value: &[u8]) -> BNode {        
        let data = Bytes::copy_from_slice(value);

        BNode {
            data
        }
    }


    pub fn node_type(&self) -> Result<NodeType, String> {
        let Some(node_type_byte) = self.data.get(0..2) else {
            return Err(String::from("Not able to read node_type"));
        };

        let array: [u8; 2] = match node_type_byte.try_into() {
            Ok(v) => v,
            Err(_) => {
                return Err(String::from("slice with incorrect length"));
            }
        };

        Ok(NodeType::from(&array)) 

    }

    pub fn number_of_keys(&self) -> Result<u16, String> {
        let Some(nkeys) = self.data.get(2..4) else {
            return Err(String::from("Not able to read number_of_keys"));
        };

        let mut rdr = Cursor::new(nkeys);

        let value: u16 = match rdr.read_u16::<BigEndian>() {
            Ok(v) => v,
            Err(_) => {
                return Err(String::from("slice with incorrect length"));
            }
        };

        Ok(value) 

    }
}

/*

func (node BNode) setHeader(btype uint16, nkeys uint16) {
    binary.LittleEndian.PutUint16(node.data[0:2], btype)
    binary.LittleEndian.PutUint16(node.data[2:4], nkeys)
}
*/



#[cfg(test)]
mod b_node_tests {
    use super::*;
    use more_asserts::{assert_le, assert_ge};


    #[test]
    fn root_or_eq() {
        let node = BNode::load_from_bytes(&[0, 0, 5, 7, 6]);
        assert_eq!(node.node_type(), Ok(NodeType::BNodeRoot));
    }

    #[test]
    fn node_or_eq() {
        let node = BNode::load_from_bytes(&[0, 1, 5, 7, 6]);
        assert_eq!(node.node_type(), Ok(NodeType::BNodeNode));
    }

    #[test]
    fn leaf_or_eq() {
        let node = BNode::load_from_bytes(&[0, 2, 5, 7, 6]);
        assert_eq!(node.node_type(), Ok(NodeType::BNodeLeaf));
    }

    #[test]
    fn unknowed_or_eq() {
        let node = BNode::load_from_bytes(&[0, 3, 5, 7, 6]);
        assert_eq!(node.node_type(), Ok(NodeType::Unknowed));
    }

    #[test]
    fn number_of_keys() {
        let node = BNode::load_from_bytes(&[0, 2, 5, 7, 6]);
        assert_eq!(node.number_of_keys(), Ok(1287));
    }

    #[test]
    fn number_of_keys_testing() {
        let bytes = 1287u16.to_be_bytes();
        assert_eq!([0x05, 0x07], bytes);
    }

    
}