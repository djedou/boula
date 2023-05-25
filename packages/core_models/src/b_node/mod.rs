use bytes::Bytes;
use std::io::Cursor;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, ByteOrder, WriteBytesExt};

/*
#[derive(Debug, PartialEq, Eq)]
pub struct KeyPair {
    pub key_len: u16, // 2B
    pub value_len: u16, // 2B
    pub keys: Vec<u8>,
    pub values: Vec<u8>
}

#[derive(Debug, PartialEq, Eq)]
pub struct BNodeMetadata {
    pub node_type: NodeType, // 2B
    pub number_of_keys: u16, // 2B
    pub pointers: Vec<u64>, // number_of_keys * 8B
    pub offsets: Vec<u16>, // number_of_keys * 2B
    pub key_values: KeyPair
}
*/