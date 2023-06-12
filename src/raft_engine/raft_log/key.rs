

/// A metadata key
#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    TermVote,
}

impl Key {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::TermVote => vec![0x00],
        }
    }
}