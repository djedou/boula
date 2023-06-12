use serde::{Deserialize, Serialize};

/// An MVCC transaction mode.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    /// A read-write transaction.
    ReadWrite,
    /// A read-only transaction.
    ReadOnly,
    /// A read-only transaction running in a snapshot of a given version.
    ///
    /// The version must refer to a committed transaction ID. Any changes visible to the original
    /// transaction will be visible in the snapshot (i.e. transactions that had not committed before
    /// the snapshot transaction started will not be visible, even though they have a lower version).
    Snapshot { version: u64 },
}

impl Mode {
    /// Checks whether the transaction mode can mutate data.
    pub fn mutable(&self) -> bool {
        match self {
            Self::ReadWrite => true,
            Self::ReadOnly => false,
            Self::Snapshot { .. } => false,
        }
    }

    /// Checks whether a mode satisfies a mode (i.e. ReadWrite satisfies ReadOnly).
    pub fn satisfies(&self, other: &Mode) -> bool {
        match (self, other) {
            (Mode::ReadWrite, Mode::ReadOnly) => true,
            (Mode::Snapshot { .. }, Mode::ReadOnly) => true,
            (_, _) if self == other => true,
            (_, _) => false,
        }
    }
}
