mod candidate;
mod follower;
mod leader;
mod node;
mod role_node;
mod status;



pub use candidate::*;
pub use follower::*;
pub use leader::*;
pub use node::*;
pub use role_node::*;
pub use status::*;

/// The interval between leader heartbeats, in ticks.
pub const HEARTBEAT_INTERVAL: u64 = 1;

/// The minimum election timeout, in ticks.
pub const ELECTION_TIMEOUT_MIN: u64 = 8 * HEARTBEAT_INTERVAL;

/// The maximum election timeout, in ticks.
pub const ELECTION_TIMEOUT_MAX: u64 = 15 * HEARTBEAT_INTERVAL;

/*
#[cfg(test)]
mod tests {
    pub use super::super::state::tests::TestState;
    use super::super::Entry;
    use super::follower::tests::{follower_leader, follower_voted_for};
    use super::*;
    use crate::storage::log;
    use futures::FutureExt;
    use pretty_assertions::assert_eq;
    use tokio::sync::mpsc;

    pub fn assert_messages<T: std::fmt::Debug + PartialEq>(
        rx: &mut mpsc::UnboundedReceiver<T>,
        msgs: Vec<T>,
    ) {
        let mut actual = Vec::new();
        while let Some(Some(message)) = rx.recv().now_or_never() {
            actual.push(message)
        }
        assert_eq!(msgs, actual);
    }

    pub struct NodeAsserter<'a> {
        node: &'a Node,
    }

    impl<'a> NodeAsserter<'a> {
        pub fn new(node: &'a Node) -> Self {
            Self { node }
        }

        fn log(&self) -> &'a Log {
            match self.node {
                Node::Candidate(n) => &n.log,
                Node::Follower(n) => &n.log,
                Node::Leader(n) => &n.log,
            }
        }

        pub fn committed(self, index: u64) -> Self {
            assert_eq!(index, self.log().commit_index, "Unexpected committed index");
            self
        }

        pub fn last(self, index: u64) -> Self {
            assert_eq!(index, self.log().last_index, "Unexpected last index");
            self
        }

        pub fn entry(self, entry: Entry) -> Self {
            assert!(entry.index <= self.log().last_index, "Index beyond last entry");
            assert_eq!(entry, self.log().get(entry.index).unwrap().unwrap());
            self
        }

        pub fn entries(self, entries: Vec<Entry>) -> Self {
            assert_eq!(entries, self.log().scan(0..).collect::<Result<Vec<_>>>().unwrap());
            self
        }

        #[allow(clippy::wrong_self_convention)]
        pub fn is_candidate(self) -> Self {
            match self.node {
                Node::Candidate(_) => self,
                Node::Follower(_) => panic!("Expected candidate, got follower"),
                Node::Leader(_) => panic!("Expected candidate, got leader"),
            }
        }

        #[allow(clippy::wrong_self_convention)]
        pub fn is_follower(self) -> Self {
            match self.node {
                Node::Candidate(_) => panic!("Expected follower, got candidate"),
                Node::Follower(_) => self,
                Node::Leader(_) => panic!("Expected follower, got leader"),
            }
        }

        #[allow(clippy::wrong_self_convention)]
        pub fn is_leader(self) -> Self {
            match self.node {
                Node::Candidate(_) => panic!("Expected leader, got candidate"),
                Node::Follower(_) => panic!("Expected leader, got follower"),
                Node::Leader(_) => self,
            }
        }

        pub fn leader(self, leader: Option<&str>) -> Self {
            assert_eq!(
                leader.map(str::to_owned),
                match self.node {
                    Node::Candidate(_) => None,
                    Node::Follower(n) => follower_leader(n),
                    Node::Leader(_) => None,
                },
                "Unexpected leader",
            );
            self
        }

        pub fn proxied(self, proxied: Vec<(Vec<u8>, Address)>) -> Self {
            assert_eq!(
                &proxied.into_iter().collect::<HashMap<Vec<u8>, Address>>(),
                match self.node {
                    Node::Candidate(n) => &n.proxied_reqs,
                    Node::Follower(n) => &n.proxied_reqs,
                    Node::Leader(n) => &n.proxied_reqs,
                }
            );
            self
        }

        pub fn queued(self, queued: Vec<(Address, Event)>) -> Self {
            assert_eq!(
                &queued,
                match self.node {
                    Node::Candidate(n) => &n.queued_reqs,
                    Node::Follower(n) => &n.queued_reqs,
                    Node::Leader(n) => &n.queued_reqs,
                }
            );
            self
        }

        pub fn term(self, term: u64) -> Self {
            assert_eq!(
                term,
                match self.node {
                    Node::Candidate(n) => n.term,
                    Node::Follower(n) => n.term,
                    Node::Leader(n) => n.term,
                },
                "Unexpected node term",
            );
            let (saved_term, saved_voted_for) = self.log().load_term().unwrap();
            assert_eq!(saved_term, term, "Incorrect term stored in log");
            assert_eq!(
                saved_voted_for,
                match self.node {
                    Node::Candidate(_) => None,
                    Node::Follower(n) => follower_voted_for(n),
                    Node::Leader(_) => None,
                },
                "Incorrect voted_for stored in log"
            );
            self
        }

        pub fn voted_for(self, voted_for: Option<&str>) -> Self {
            assert_eq!(
                voted_for.map(str::to_owned),
                match self.node {
                    Node::Candidate(_) => None,
                    Node::Follower(n) => follower_voted_for(n),
                    Node::Leader(_) => None,
                },
                "Unexpected voted_for"
            );
            let (_, saved_voted_for) = self.log().load_term().unwrap();
            assert_eq!(saved_voted_for.as_deref(), voted_for, "Unexpected voted_for saved in log");
            self
        }
    }

    pub fn assert_node(node: &Node) -> NodeAsserter {
        NodeAsserter::new(node)
    }

    fn setup_rolenode() -> Result<(RoleNode<()>, mpsc::UnboundedReceiver<Message>)> {
        setup_rolenode_peers(vec!["b".into(), "c".into()])
    }

    fn setup_rolenode_peers(
        peers: Vec<String>,
    ) -> Result<(RoleNode<()>, mpsc::UnboundedReceiver<Message>)> {
        let (node_tx, node_rx) = mpsc::unbounded_channel();
        let (state_tx, _) = mpsc::unbounded_channel();
        let node = RoleNode {
            role: (),
            id: "a".into(),
            peers,
            term: 1,
            log: Log::new(Box::new(log::Test::new()))?,
            node_tx,
            state_tx,
            proxied_reqs: HashMap::new(),
            queued_reqs: Vec::new(),
        };
        Ok((node, node_rx))
    }

    #[tokio::test]
    async fn new() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let node = Node::new(
            "a",
            vec!["b".into(), "c".into()],
            Log::new(Box::new(log::Test::new()))?,
            Box::new(TestState::new(0)),
            node_tx,
        )
        .await?;
        match node {
            Node::Follower(rolenode) => {
                assert_eq!(rolenode.id, "a".to_owned());
                assert_eq!(rolenode.term, 0);
                assert_eq!(rolenode.peers, vec!["b".to_owned(), "c".to_owned()]);
            }
            _ => panic!("Expected node to start as follower"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn new_loads_term() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let store = Box::new(log::Test::new());
        Log::new(store.clone())?.save_term(3, Some("c"))?;
        let node = Node::new(
            "a",
            vec!["b".into(), "c".into()],
            Log::new(store)?,
            Box::new(TestState::new(0)),
            node_tx,
        )
        .await?;
        match node {
            Node::Follower(rolenode) => assert_eq!(rolenode.term, 3),
            _ => panic!("Expected node to start as follower"),
        }
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn new_state_apply_all() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let mut log = Log::new(Box::new(log::Test::new()))?;
        log.append(1, Some(vec![0x01]))?;
        log.append(2, None)?;
        log.append(2, Some(vec![0x02]))?;
        log.commit(3)?;
        log.append(2, Some(vec![0x03]))?;
        let state = Box::new(TestState::new(0));

        Node::new("a", vec!["b".into(), "c".into()], log, state.clone(), node_tx).await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(state.list(), vec![vec![0x01], vec![0x02]]);
        assert_eq!(state.applied_index(), 3);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn new_state_apply_partial() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let mut log = Log::new(Box::new(log::Test::new()))?;
        log.append(1, Some(vec![0x01]))?;
        log.append(2, None)?;
        log.append(2, Some(vec![0x02]))?;
        log.commit(3)?;
        log.append(2, Some(vec![0x03]))?;
        let state = Box::new(TestState::new(2));

        Node::new("a", vec!["b".into(), "c".into()], log, state.clone(), node_tx).await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(state.list(), vec![vec![0x02]]);
        assert_eq!(state.applied_index(), 3);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn new_state_apply_missing() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let mut log = Log::new(Box::new(log::Test::new()))?;
        log.append(1, Some(vec![0x01]))?;
        log.append(2, None)?;
        log.append(2, Some(vec![0x02]))?;
        log.commit(3)?;
        log.append(2, Some(vec![0x03]))?;
        let state = Box::new(TestState::new(4));

        assert_eq!(
            Node::new("a", vec!["b".into(), "c".into()], log, state.clone(), node_tx).await.err(),
            Some(Error::Internal(
                "State machine applied index 4 greater than log committed index 3".into()
            ))
        );
        Ok(())
    }

    #[tokio::test]
    async fn new_single() -> Result<()> {
        let (node_tx, _) = mpsc::unbounded_channel();
        let node = Node::new(
            "a",
            vec![],
            Log::new(Box::new(log::Test::new()))?,
            Box::new(TestState::new(0)),
            node_tx,
        )
        .await?;
        match node {
            Node::Leader(rolenode) => {
                assert_eq!(rolenode.id, "a".to_owned());
                assert_eq!(rolenode.term, 0);
                assert!(rolenode.peers.is_empty());
            }
            _ => panic!("Expected leader"),
        }
        Ok(())
    }

    #[test]
    fn become_role() -> Result<()> {
        let (node, _) = setup_rolenode()?;
        let new = node.become_role("role")?;
        assert_eq!(new.id, "a".to_owned());
        assert_eq!(new.term, 1);
        assert_eq!(new.peers, vec!["b".to_owned(), "c".to_owned()]);
        assert_eq!(new.role, "role");
        Ok(())
    }

    #[test]
    fn quorum() -> Result<()> {
        let quorums = vec![(1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (6, 4), (7, 4), (8, 5)];
        for (size, quorum) in quorums.into_iter() {
            let peers: Vec<String> =
                (0..(size as u8 - 1)).map(|i| (i as char).to_string()).collect();
            assert_eq!(peers.len(), size as usize - 1);
            let (node, _) = setup_rolenode_peers(peers)?;
            assert_eq!(node.quorum(), quorum);
        }
        Ok(())
    }

    #[test]
    fn send() -> Result<()> {
        let (node, mut rx) = setup_rolenode()?;
        node.send(Address::Peer("b".into()), Event::Heartbeat { commit_index: 1, commit_term: 1 })?;
        assert_messages(
            &mut rx,
            vec![Message {
                from: Address::Local,
                to: Address::Peer("b".into()),
                term: 1,
                event: Event::Heartbeat { commit_index: 1, commit_term: 1 },
            }],
        );
        Ok(())
    }
}
*/