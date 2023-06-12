use crate::{
    error::{Error, Result},
    raft_engine::{
        machine_state::{Driver, MachineState},
        messaging::Message,
        raft_node::{Candidate, Follower, Leader, RoleNode},
        raft_log::RaftLog
    }
};
use log::{debug, info};
use std::collections::HashMap;
use tokio::sync::mpsc;





/// The local Raft node state machine.
pub enum Node {
    Candidate(RoleNode<Candidate>),
    Follower(RoleNode<Follower>),
    Leader(RoleNode<Leader>),
}

impl Node {
    /// Creates a new Raft node, starting as a follower, or leader if no peers.
    pub async fn new(
        id: &str,
        peers: Vec<String>,
        log: RaftLog,
        mut state: Box<dyn MachineState>,
        node_tx: mpsc::UnboundedSender<Message>,
    ) -> Result<Self> {
        let applied_index = state.applied_index();
        if applied_index > log.commit_index {
            return Err(Error::Internal(format!(
                "State machine applied index {} greater than log committed index {}",
                applied_index, log.commit_index
            )));
        }

        let (state_tx, state_rx) = mpsc::unbounded_channel();
        let mut driver = Driver::new(state_rx, node_tx.clone());
        if log.commit_index > applied_index {
            info!("Replaying log entries {} to {}", applied_index + 1, log.commit_index);
            driver.replay(&mut *state, log.scan((applied_index + 1)..=log.commit_index))?;
        };
        tokio::spawn(driver.drive(state));

        let (term, voted_for) = log.load_term()?;
        let node = RoleNode {
            id: id.to_owned(),
            peers,
            term,
            log,
            node_tx,
            state_tx,
            queued_reqs: Vec::new(),
            proxied_reqs: HashMap::new(),
            role: Follower::new(None, voted_for.as_deref()),
        };
        if node.peers.is_empty() {
            info!("No peers specified, starting as leader");
            let last_index = node.log.last_index;
            Ok(node.become_role(Leader::new(vec![], last_index))?.into())
        } else {
            Ok(node.into())
        }
    }

    /// Returns the node ID.
    pub fn id(&self) -> String {
        match self {
            Node::Candidate(n) => n.id.clone(),
            Node::Follower(n) => n.id.clone(),
            Node::Leader(n) => n.id.clone(),
        }
    }

    /// Processes a message.
    pub fn step(self, msg: Message) -> Result<Self> {
        debug!("Stepping {:?}", msg);
        match self {
            Node::Candidate(n) => n.step(msg),
            Node::Follower(n) => n.step(msg),
            Node::Leader(n) => n.step(msg),
        }
    }

    /// Moves time forward by a tick.
    pub fn tick(self) -> Result<Self> {
        match self {
            Node::Candidate(n) => n.tick(),
            Node::Follower(n) => n.tick(),
            Node::Leader(n) => n.tick(),
        }
    }
}

impl From<RoleNode<Candidate>> for Node {
    fn from(rn: RoleNode<Candidate>) -> Self {
        Node::Candidate(rn)
    }
}

impl From<RoleNode<Follower>> for Node {
    fn from(rn: RoleNode<Follower>) -> Self {
        Node::Follower(rn)
    }
}

impl From<RoleNode<Leader>> for Node {
    fn from(rn: RoleNode<Leader>) -> Self {
        Node::Leader(rn)
    }
}
