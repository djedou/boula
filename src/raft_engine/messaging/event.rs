use serde::{Deserialize, Serialize};
use crate::{
    error::Result,
    raft_engine::{
        raft_log::Entry,
        messaging::{Request, Response}
    }
};


/// An event contained within messages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Leaders send periodic heartbeats to its followers.
    Heartbeat {
        /// The index of the leader's last committed log entry.
        commit_index: u64,
        /// The term of the leader's last committed log entry.
        commit_term: u64,
    },
    /// Followers confirm loyalty to leader after heartbeats.
    ConfirmLeader {
        /// The commit_index of the original leader heartbeat, to confirm
        /// read requests.
        commit_index: u64,
        /// If false, the follower does not have the entry at commit_index
        /// and would like the leader to replicate it.
        has_committed: bool,
    },
    /// Candidates solicit votes from all peers.
    SolicitVote {
        // The index of the candidate's last stored log entry
        last_index: u64,
        // The term of the candidate's last stored log entry
        last_term: u64,
    },
    /// Followers may grant votes to candidates.
    GrantVote,
    /// Leaders replicate a set of log entries to followers.
    ReplicateEntries {
        /// The index of the log entry immediately preceding the submitted commands.
        base_index: u64,
        /// The term of the log entry immediately preceding the submitted commands.
        base_term: u64,
        /// Commands to replicate.
        entries: Vec<Entry>,
    },
    /// Followers may accept a set of log entries from a leader.
    AcceptEntries {
        /// The index of the last log entry.
        last_index: u64,
    },
    /// Followers may also reject a set of log entries from a leader.
    RejectEntries,
    /// A client request.
    ClientRequest {
        /// The request ID.
        id: Vec<u8>,
        /// The request.
        request: Request,
    },
    /// A client response.
    ClientResponse {
        /// The response ID.
        id: Vec<u8>,
        /// The response.
        response: Result<Response>,
    },
}