mod driver;
mod instruction;
mod query;
mod state;


pub use driver::*;
pub use instruction::*;
pub use query::*;
pub use state::*;





/*
#[cfg(test)]
pub mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug)]
    pub struct TestState {
        commands: Arc<Mutex<Vec<Vec<u8>>>>,
        applied_index: Arc<Mutex<u64>>,
    }

    impl TestState {
        pub fn new(applied_index: u64) -> Self {
            Self {
                commands: Arc::new(Mutex::new(Vec::new())),
                applied_index: Arc::new(Mutex::new(applied_index)),
            }
        }

        pub fn list(&self) -> Vec<Vec<u8>> {
            self.commands.lock().unwrap().clone()
        }
    }

    impl State for TestState {
        fn applied_index(&self) -> u64 {
            *self.applied_index.lock().unwrap()
        }

        // Appends the command to the internal commands list.
        fn mutate(&mut self, index: u64, command: Vec<u8>) -> Result<Vec<u8>> {
            self.commands.lock()?.push(command.clone());
            *self.applied_index.lock()? = index;
            Ok(command)
        }

        // Appends the command to the internal commands list.
        fn query(&self, command: Vec<u8>) -> Result<Vec<u8>> {
            self.commands.lock()?.push(command.clone());
            Ok(command)
        }
    }

    async fn setup() -> Result<(
        Box<TestState>,
        mpsc::UnboundedSender<Instruction>,
        mpsc::UnboundedReceiver<Message>,
    )> {
        let state = Box::new(TestState::new(0));
        let (state_tx, state_rx) = mpsc::unbounded_channel();
        let (node_tx, node_rx) = mpsc::unbounded_channel();
        tokio::spawn(Driver::new(state_rx, node_tx).drive(state.clone()));
        Ok((state, state_tx, node_rx))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn driver_abort() -> Result<()> {
        let (state, state_tx, node_rx) = setup().await?;

        state_tx.send(Instruction::Notify {
            id: vec![0x01],
            index: 1,
            address: Address::Peer("a".into()),
        })?;
        state_tx.send(Instruction::Query {
            id: vec![0x02],
            address: Address::Client,
            command: vec![0xf0],
            term: 1,
            index: 1,
            quorum: 2,
        })?;
        state_tx.send(Instruction::Vote { term: 1, index: 1, address: Address::Local })?;
        state_tx.send(Instruction::Abort)?;
        std::mem::drop(state_tx);

        let node_rx = UnboundedReceiverStream::new(node_rx);
        assert_eq!(
            node_rx.collect::<Vec<_>>().await,
            vec![
                Message {
                    from: Address::Local,
                    to: Address::Peer("a".into()),
                    term: 0,
                    event: Event::ClientResponse { id: vec![0x01], response: Err(Error::Abort) }
                },
                Message {
                    from: Address::Local,
                    to: Address::Client,
                    term: 0,
                    event: Event::ClientResponse { id: vec![0x02], response: Err(Error::Abort) }
                }
            ]
        );
        assert_eq!(state.list(), Vec::<Vec<u8>>::new());
        assert_eq!(state.applied_index(), 0);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn driver_apply() -> Result<()> {
        let (state, state_tx, node_rx) = setup().await?;

        state_tx.send(Instruction::Notify {
            id: vec![0x01],
            index: 2,
            address: Address::Client,
        })?;
        state_tx.send(Instruction::Apply { entry: Entry { index: 1, term: 1, command: None } })?;
        state_tx.send(Instruction::Apply {
            entry: Entry { index: 2, term: 1, command: Some(vec![0xaf]) },
        })?;
        std::mem::drop(state_tx);

        let node_rx = UnboundedReceiverStream::new(node_rx);
        assert_eq!(
            node_rx.collect::<Vec<_>>().await,
            vec![Message {
                from: Address::Local,
                to: Address::Client,
                term: 0,
                event: Event::ClientResponse {
                    id: vec![0x01],
                    response: Ok(Response::State(vec![0xaf]))
                }
            }]
        );
        assert_eq!(state.list(), vec![vec![0xaf]]);
        assert_eq!(state.applied_index(), 2);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn driver_query() -> Result<()> {
        let (_, state_tx, node_rx) = setup().await?;

        state_tx.send(Instruction::Query {
            id: vec![0x01],
            address: Address::Client,
            command: vec![0xf0],
            term: 2,
            index: 1,
            quorum: 2,
        })?;
        state_tx.send(Instruction::Apply {
            entry: Entry { index: 1, term: 2, command: Some(vec![0xaf]) },
        })?;
        state_tx.send(Instruction::Vote { term: 2, index: 1, address: Address::Local })?;
        state_tx.send(Instruction::Vote {
            term: 2,
            index: 1,
            address: Address::Peer("a".into()),
        })?;
        std::mem::drop(state_tx);

        let node_rx = UnboundedReceiverStream::new(node_rx);
        assert_eq!(
            node_rx.collect::<Vec<_>>().await,
            vec![Message {
                from: Address::Local,
                to: Address::Client,
                term: 0,
                event: Event::ClientResponse {
                    id: vec![0x01],
                    response: Ok(Response::State(vec![0xf0]))
                }
            }]
        );

        Ok(())
    }

    // A query for an index submitted in a given term cannot be satisfied by votes below that term.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn driver_query_noterm() -> Result<()> {
        let (_, state_tx, node_rx) = setup().await?;

        state_tx.send(Instruction::Query {
            id: vec![0x01],
            address: Address::Client,
            command: vec![0xf0],
            term: 2,
            index: 1,
            quorum: 2,
        })?;
        state_tx.send(Instruction::Apply {
            entry: Entry { index: 1, term: 1, command: Some(vec![0xaf]) },
        })?;
        state_tx.send(Instruction::Vote { term: 2, index: 1, address: Address::Local })?;
        state_tx.send(Instruction::Vote {
            term: 1,
            index: 1,
            address: Address::Peer("a".into()),
        })?;
        std::mem::drop(state_tx);

        let node_rx = UnboundedReceiverStream::new(node_rx);
        assert_eq!(node_rx.collect::<Vec<_>>().await, vec![]);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn driver_query_noquorum() -> Result<()> {
        let (_, state_tx, node_rx) = setup().await?;

        state_tx.send(Instruction::Query {
            id: vec![0x01],
            address: Address::Client,
            command: vec![0xf0],
            term: 1,
            index: 1,
            quorum: 2,
        })?;
        state_tx.send(Instruction::Apply {
            entry: Entry { index: 1, term: 1, command: Some(vec![0xaf]) },
        })?;
        state_tx.send(Instruction::Vote { term: 1, index: 1, address: Address::Local })?;
        std::mem::drop(state_tx);

        let node_rx = UnboundedReceiverStream::new(node_rx);
        assert_eq!(node_rx.collect::<Vec<_>>().await, vec![]);

        Ok(())
    }
}
*/