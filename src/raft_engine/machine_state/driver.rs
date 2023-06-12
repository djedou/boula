use log::{debug, error};
use crate::{
    error::{Error, Result},
    raft_engine::{
        messaging::{Address, Event, Message, Response},
        raft_log::{Entry, Scan},
        machine_state::{Instruction, Query, MachineState}
    }
};
use std::collections::{BTreeMap, HashMap, HashSet};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;


/// Drives a state machine, taking operations from state_rx and sending results via node_tx.
pub struct Driver {
    pub state_rx: UnboundedReceiverStream<Instruction>,
    pub node_tx: mpsc::UnboundedSender<Message>,
    pub applied_index: u64,
    /// Notify clients when their mutation is applied. <index, (client, id)>
    pub notify: HashMap<u64, (Address, Vec<u8>)>,
    /// Execute client queries when they receive a quorum. <index, <id, query>>
    pub queries: BTreeMap<u64, BTreeMap<Vec<u8>, Query>>,
}

impl Driver {
    /// Creates a new state machine driver.
    pub fn new(
        state_rx: mpsc::UnboundedReceiver<Instruction>,
        node_tx: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            state_rx: UnboundedReceiverStream::new(state_rx),
            node_tx,
            applied_index: 0,
            notify: HashMap::new(),
            queries: BTreeMap::new(),
        }
    }

    /// Drives a state machine.
    pub async fn drive(mut self, mut state: Box<dyn MachineState>) -> Result<()> {
        debug!("Starting state machine driver");
        while let Some(instruction) = self.state_rx.next().await {
            if let Err(error) = self.execute(instruction, &mut *state).await {
                error!("Halting state machine due to error: {}", error);
                return Err(error);
            }
        }
        debug!("Stopping state machine driver");
        Ok(())
    }

    /// Synchronously (re)plays a set of log entries, for initial sync.
    pub fn replay<'a>(&mut self, state: &mut dyn MachineState, mut scan: Scan<'a>) -> Result<()> {
        while let Some(entry) = scan.next().transpose()? {
            debug!("Replaying {:?}", entry);
            if let Some(command) = entry.command {
                match state.mutate(entry.index, command) {
                    Err(error @ Error::Internal(_)) => return Err(error),
                    _ => self.applied_index = entry.index,
                }
            }
        }
        Ok(())
    }

    /// Executes a state machine instruction.
    pub async fn execute(&mut self, i: Instruction, state: &mut dyn MachineState) -> Result<()> {
        debug!("Executing {:?}", i);
        match i {
            Instruction::Abort => {
                self.notify_abort()?;
                self.query_abort()?;
            }

            Instruction::Apply { entry: Entry { index, command, .. } } => {
                if let Some(command) = command {
                    debug!("Applying state machine command {}: {:?}", index, command);
                    match tokio::task::block_in_place(|| state.mutate(index, command)) {
                        Err(error @ Error::Internal(_)) => return Err(error),
                        result => self.notify_applied(index, result)?,
                    };
                }
                // We have to track applied_index here, separately from the state machine, because
                // no-op log entries are significant for whether a query should be executed.
                self.applied_index = index;
                // Try to execute any pending queries, since they may have been submitted for a
                // commit_index which hadn't been applied yet.
                self.query_execute(state)?;
            }

            Instruction::Notify { id, address, index } => {
                if index > state.applied_index() {
                    self.notify.insert(index, (address, id));
                } else {
                    self.send(address, Event::ClientResponse { id, response: Err(Error::Abort) })?;
                }
            }

            Instruction::Query { id, address, command, index, term, quorum } => {
                self.queries.entry(index).or_default().insert(
                    id.clone(),
                    Query { id, term, address, command, quorum, votes: HashSet::new() },
                );
            }

            Instruction::Status { id, address, mut status } => {
                status.apply_index = state.applied_index();
                self.send(
                    address,
                    Event::ClientResponse { id, response: Ok(Response::Status(*status)) },
                )?;
            }

            Instruction::Vote { term, index, address } => {
                self.query_vote(term, index, address);
                self.query_execute(state)?;
            }
        }
        Ok(())
    }

    /// Aborts all pending notifications.
    fn notify_abort(&mut self) -> Result<()> {
        for (_, (address, id)) in std::mem::take(&mut self.notify) {
            self.send(address, Event::ClientResponse { id, response: Err(Error::Abort) })?;
        }
        Ok(())
    }

    /// Notifies a client about an applied log entry, if any.
    fn notify_applied(&mut self, index: u64, result: Result<Vec<u8>>) -> Result<()> {
        if let Some((to, id)) = self.notify.remove(&index) {
            self.send(to, Event::ClientResponse { id, response: result.map(Response::State) })?;
        }
        Ok(())
    }

    /// Aborts all pending queries.
    fn query_abort(&mut self) -> Result<()> {
        for (_, queries) in std::mem::take(&mut self.queries) {
            for (id, query) in queries {
                self.send(
                    query.address,
                    Event::ClientResponse { id, response: Err(Error::Abort) },
                )?;
            }
        }
        Ok(())
    }

    /// Executes any queries that are ready.
    fn query_execute(&mut self, state: &mut dyn MachineState) -> Result<()> {
        for query in self.query_ready(self.applied_index) {
            debug!("Executing query {:?}", query.command);
            let result = state.query(query.command);
            if let Err(error @ Error::Internal(_)) = result {
                return Err(error);
            }
            self.send(
                query.address,
                Event::ClientResponse { id: query.id, response: result.map(Response::State) },
            )?
        }
        Ok(())
    }

    /// Fetches and removes any ready queries, where index <= applied_index.
    fn query_ready(&mut self, applied_index: u64) -> Vec<Query> {
        let mut ready = Vec::new();
        let mut empty = Vec::new();
        for (index, queries) in self.queries.range_mut(..=applied_index) {
            let mut ready_ids = Vec::new();
            for (id, query) in queries.iter_mut() {
                if query.votes.len() as u64 >= query.quorum {
                    ready_ids.push(id.clone());
                }
            }
            for id in ready_ids {
                if let Some(query) = queries.remove(&id) {
                    ready.push(query)
                }
            }
            if queries.is_empty() {
                empty.push(*index)
            }
        }
        for index in empty {
            self.queries.remove(&index);
        }
        ready
    }

    /// Votes for queries up to and including a given commit index for a term by an address.
    fn query_vote(&mut self, term: u64, commit_index: u64, address: Address) {
        for (_, queries) in self.queries.range_mut(..=commit_index) {
            for (_, query) in queries.iter_mut() {
                if term >= query.term {
                    query.votes.insert(address.clone());
                }
            }
        }
    }

    /// Sends a message.
    fn send(&self, to: Address, event: Event) -> Result<()> {
        let msg = Message { from: Address::Local, to, term: 0, event };
        debug!("Sending {:?}", msg);
        Ok(self.node_tx.send(msg)?)
    }
}