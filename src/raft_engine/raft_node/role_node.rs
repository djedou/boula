use crate::{
    error::{Error, Result},
    raft_engine::{
        machine_state::Instruction,
        messaging::{Address, Event, Message},
        raft_log::RaftLog
    },
};

use log::debug;
use std::collections::HashMap;
use tokio::sync::mpsc;


// A Raft node with role R
pub struct RoleNode<R> {
    pub id: String,
    pub peers: Vec<String>,
    pub term: u64,
    pub log: RaftLog,
    pub node_tx: mpsc::UnboundedSender<Message>,
    pub state_tx: mpsc::UnboundedSender<Instruction>,
    /// Keeps track of queued client requests received e.g. during elections.
    pub queued_reqs: Vec<(Address, Event)>,
    /// Keeps track of proxied client requests, to abort on new leader election.
    pub proxied_reqs: HashMap<Vec<u8>, Address>,
    pub role: R,
}

impl<R> RoleNode<R> {
    /// Transforms the node into another role.
    pub fn become_role<T>(self, role: T) -> Result<RoleNode<T>> {
        Ok(RoleNode {
            id: self.id,
            peers: self.peers,
            term: self.term,
            log: self.log,
            node_tx: self.node_tx,
            state_tx: self.state_tx,
            queued_reqs: self.queued_reqs,
            proxied_reqs: self.proxied_reqs,
            role,
        })
    }

    /// Aborts any proxied requests.
    pub fn abort_proxied(&mut self) -> Result<()> {
        for (id, address) in std::mem::take(&mut self.proxied_reqs) {
            self.send(address, Event::ClientResponse { id, response: Err(Error::Abort) })?;
        }
        Ok(())
    }

    /// Sends any queued requests to the given leader.
    pub fn forward_queued(&mut self, leader: Address) -> Result<()> {
        for (from, event) in std::mem::take(&mut self.queued_reqs) {
            if let Event::ClientRequest { id, .. } = &event {
                self.proxied_reqs.insert(id.clone(), from.clone());
                self.node_tx.send(Message {
                    from: match from {
                        Address::Client => Address::Local,
                        address => address,
                    },
                    to: leader.clone(),
                    term: 0,
                    event,
                })?;
            }
        }
        Ok(())
    }

    /// Returns the quorum size of the cluster.
    pub fn quorum(&self) -> u64 {
        (self.peers.len() as u64 + 1) / 2 + 1
    }

    /// Sends an event
    pub fn send(&self, to: Address, event: Event) -> Result<()> {
        let msg = Message { term: self.term, from: Address::Local, to, event };
        debug!("Sending {:?}", msg);
        Ok(self.node_tx.send(msg)?)
    }

    /// Validates a message
    pub fn validate(&self, msg: &Message) -> Result<()> {
        match msg.from {
            Address::Peers => return Err(Error::Internal("Message from broadcast address".into())),
            Address::Local => return Err(Error::Internal("Message from local node".into())),
            Address::Client if !matches!(msg.event, Event::ClientRequest { .. }) => {
                return Err(Error::Internal("Non-request message from client".into()));
            }
            _ => {}
        }

        // Allowing requests and responses form past terms is fine, since they don't rely on it
        if msg.term < self.term
            && !matches!(msg.event, Event::ClientRequest { .. } | Event::ClientResponse { .. })
        {
            return Err(Error::Internal(format!("Message from past term {}", msg.term)));
        }

        match &msg.to {
            Address::Peer(id) if id == &self.id => Ok(()),
            Address::Local => Ok(()),
            Address::Peers => Ok(()),
            Address::Peer(id) => {
                Err(Error::Internal(format!("Received message for other node {}", id)))
            }
            Address::Client => Err(Error::Internal("Received message for client".into())),
        }
    }
}