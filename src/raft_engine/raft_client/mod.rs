use crate::error::{Error, Result};
use crate::{
    raft_engine::{
        messaging::{Request, Response},
        raft_node::Status
    }
};

use tokio::sync::{mpsc, oneshot};

/// A client for a local Raft server.
#[derive(Clone)]
pub struct Client {
    pub request_tx: mpsc::UnboundedSender<(Request, oneshot::Sender<Result<Response>>)>,
}

impl Client {
    /// Creates a new Raft client.
    pub fn new(
        request_tx: mpsc::UnboundedSender<(Request, oneshot::Sender<Result<Response>>)>,
    ) -> Self {
        Self { request_tx }
    }

    /// Executes a request against the Raft cluster.
    async fn request(&self, request: Request) -> Result<Response> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx.send((request, response_tx))?;
        response_rx.await?
    }

    /// Mutates the Raft state machine.
    pub async fn mutate(&self, command: Vec<u8>) -> Result<Vec<u8>> {
        match self.request(Request::Mutate(command)).await? {
            Response::State(response) => Ok(response),
            resp => Err(Error::Internal(format!("Unexpected Raft mutate response {:?}", resp))),
        }
    }

    /// Queries the Raft state machine.
    pub async fn query(&self, command: Vec<u8>) -> Result<Vec<u8>> {
        match self.request(Request::Query(command)).await? {
            Response::State(response) => Ok(response),
            resp => Err(Error::Internal(format!("Unexpected Raft query response {:?}", resp))),
        }
    }

    /// Fetches Raft node status.
    pub async fn status(&self) -> Result<Status> {
        match self.request(Request::Status).await? {
            Response::Status(status) => Ok(status),
            resp => Err(Error::Internal(format!("Unexpected Raft status response {:?}", resp))),
        }
    }
}
