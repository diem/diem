// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    epoch_manager::EpochManager,
    network::NetworkTask,
    network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
    network_tests::{NetworkPlayground, TwinId},
    test_utils::{
        consensus_runtime, timed_block_on, MockStateComputer, MockStorage, MockTransactionManager,
    },
    util::time_service::ClockTimeService,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block::Block,
    common::{Author, Payload, Round},
};
use futures::{channel::mpsc, StreamExt};
use libra_config::{
    config::{
        ConsensusProposerType::{self, FixedProposer, RotatingProposer, RoundProposer},
        NodeConfig, WaypointConfig,
    },
    generator::{self, ValidatorSwarm},
};
use libra_mempool::mocks::MockSharedMempool;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::{OnChainConfig, OnChainConfigPayload, ValidatorSet},
    validator_info::ValidatorInfo,
    waypoint::Waypoint,
};
use network::{
    peer_manager::{conn_notifs_channel, ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{NewNetworkEvents, NewNetworkSender},
};
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use tokio::runtime::{Builder, Runtime};

/// Auxiliary struct that is preparing SMR for the test
struct SMRNode {
    id: TwinId,
    _runtime: Runtime,
    commit_cb_receiver: mpsc::UnboundedReceiver<LedgerInfoWithSignatures>,
    storage: Arc<MockStorage>,
    _shared_mempool: MockSharedMempool,
    _state_sync: mpsc::UnboundedReceiver<Payload>,
}

fn author_from_config(config: &NodeConfig) -> Author {
    config.validator_network.as_ref().unwrap().peer_id()
}

impl SMRNode {
    fn start(
        playground: &mut NetworkPlayground,
        config: NodeConfig,
        storage: Arc<MockStorage>,
        twin_id: TwinId,
    ) -> Self {
        let (network_reqs_tx, network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (connection_reqs_tx, _) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (_conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
        let (_, conn_notifs_channel) = conn_notifs_channel::new();
        let network_sender = ConsensusNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_notifs_channel);

        playground.add_node(twin_id, consensus_tx, network_reqs_rx, conn_mgr_reqs_rx);

        let (state_sync_client, state_sync) = mpsc::unbounded();
        let (commit_cb_sender, commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();
        let shared_mempool = MockSharedMempool::new(None);
        let consensus_to_mempool_sender = shared_mempool.consensus_sender.clone();
        let state_computer = Arc::new(MockStateComputer::new(
            state_sync_client,
            commit_cb_sender,
            Arc::clone(&storage),
        ));
        let txn_manager = Arc::new(MockTransactionManager::new(Some(
            consensus_to_mempool_sender,
        )));
        let (mut reconfig_sender, reconfig_events) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let mut configs = HashMap::new();
        configs.insert(
            ValidatorSet::CONFIG_ID,
            lcs::to_bytes(storage.get_validator_set()).unwrap(),
        );
        let payload = OnChainConfigPayload::new(1, Arc::new(configs));
        reconfig_sender.push((), payload).unwrap();

        let runtime = Builder::new()
            .thread_name(format!("node-{}", twin_id.id))
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let time_service = Arc::new(ClockTimeService::new(runtime.handle().clone()));

        let (timeout_sender, timeout_receiver) =
            channel::new(1_024, &counters::PENDING_ROUND_TIMEOUTS);
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);

        let epoch_mgr = EpochManager::new(
            &config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage.clone(),
            reconfig_events,
        );
        let (network_task, network_receiver) = NetworkTask::new(network_events, self_receiver);

        runtime.spawn(network_task.start());
        runtime.spawn(epoch_mgr.start(timeout_receiver, network_receiver));
        Self {
            id: twin_id,
            _runtime: runtime,
            commit_cb_receiver,
            storage,
            _shared_mempool: shared_mempool,
            _state_sync: state_sync,
        }
    }

    /// Starts a given number of nodes and their twins
    fn start_num_nodes_with_twins(
        num_nodes: usize,
        num_twins: usize,
        playground: &mut NetworkPlayground,
        proposer_type: ConsensusProposerType,
        round_proposers_idx: Option<HashMap<Round, usize>>,
    ) -> Vec<Self> {
        assert!(num_nodes >= num_twins);
        let ValidatorSwarm { mut nodes } = generator::validator_swarm_for_testing(num_nodes);

        let validator_set = ValidatorSet::new(
            nodes
                .iter()
                .map(|config| {
                    let sr_test_config = config.consensus.safety_rules.test.as_ref().unwrap();
                    ValidatorInfo::new_with_test_network_keys(
                        sr_test_config.author,
                        sr_test_config.consensus_key.as_ref().unwrap().public_key(),
                        1,
                    )
                })
                .collect(),
        );
        // sort by the peer id
        nodes.sort_by(|n1, n2| author_from_config(n1).cmp(&author_from_config(n2)));

        let proposer_type = match proposer_type {
            RoundProposer(_) => {
                let mut round_proposers: HashMap<Round, Author> = HashMap::new();

                round_proposers_idx.map(|proposers| {
                    proposers.iter().for_each(|(round, idx)| {
                        round_proposers.insert(*round, author_from_config(&nodes[*idx]));
                    })
                });
                RoundProposer(round_proposers)
            }
            _ => proposer_type,
        };

        // We don't add twins to ValidatorSet or round_proposers above
        // because a node with twins should be treated the same at the
        // consensus level
        for i in 0..num_twins {
            let twin = nodes[i].clone();
            nodes.push(twin);
        }

        let mut smr_nodes = vec![];

        for (smr_id, mut config) in nodes.into_iter().enumerate() {
            let (_, storage) = MockStorage::start_for_testing(validator_set.clone());

            let waypoint = Waypoint::new_epoch_boundary(&storage.get_ledger_info())
                .expect("Unable to produce waypoint with the provided LedgerInfo");
            config
                .consensus
                .safety_rules
                .test
                .as_mut()
                .unwrap()
                .waypoint = Some(waypoint);
            config.base.waypoint = WaypointConfig::FromConfig(waypoint);
            config.consensus.proposer_type = proposer_type.clone();
            config.consensus.safety_rules.verify_vote_proposal_signature = false;
            // Change initial timeout from default 1s to 2s. Our experience
            // suggests that 1s is too small for twins testing
            config.consensus.round_initial_timeout_ms = 2000;

            let author = author_from_config(&config);

            let twin_id = TwinId { id: smr_id, author };

            smr_nodes.push(Self::start(playground, config, storage, twin_id));
        }
        smr_nodes
    }
}

#[test]
/// This test checks that the first proposal has its parent and
/// QC pointing to the genesis block.
///
/// Setup:
///
/// 4 honest nodes, and 0 twins
///
/// Run the test:
/// cargo xtest -p consensus basic_start_test -- --nocapture
fn basic_start_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 0;
    let nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RotatingProposer,
        None,
    );
    let genesis = Block::make_genesis_block_from_ledger_info(&nodes[0].storage.get_ledger_info());
    timed_block_on(&mut runtime, async {
        let msg = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let first_proposal = match &msg[0].1 {
            ConsensusMsg::ProposalMsg(proposal) => proposal,
            _ => panic!("Unexpected message found"),
        };
        assert_eq!(first_proposal.proposal().parent_id(), genesis.id());
        assert_eq!(
            first_proposal
                .proposal()
                .quorum_cert()
                .certified_block()
                .id(),
            genesis.id()
        );
    });
}

#[test]
/// This test checks that the split_network function works
/// as expected, that is: nodes in a partition with less nodes
/// than required for quorum do not commit anything.
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 0 twins.
/// Create two partitions p1=[n2], and p2=[n0, n1, n3] with
/// a proposer (n0) in p2.
///
/// Test:
///
/// Run consensus for enough rounds to potentially form a commit.
/// Check that n1 has no commits, and n0 has commits.
///
/// Run the test:
/// cargo xtest -p consensus drop_config_test -- --nocapture
fn drop_config_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 0;
    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        FixedProposer,
        None,
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    let n1_twin_id = nodes[1].id;
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    assert!(playground.split_network(vec![n2_twin_id], vec![n0_twin_id, n1_twin_id, n3_twin_id]));

    timed_block_on(&mut runtime, async {
        playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;

        // Pull enough votes to get a few commits
        // The proposer's votes are implicit and do not go in the queue.
        playground
            .wait_for_messages(50, NetworkPlayground::votes_only)
            .await;

        // Check that the commit log for n0 is not empty
        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        assert!(node0_commit.is_some());

        // Check that the commit log for n2 is empty
        let node2_commit = match nodes[2].commit_cb_receiver.try_next() {
            Ok(Some(node_commit)) => Some(node_commit),
            _ => None,
        };
        assert!(node2_commit.is_none());
    });
}

#[test]
/// This test checks that the vote of a node and its twin
/// should be counted as duplicate vote (because they have
/// the same public keys)
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 1 twin (twin0)
/// Create 2 partitions, p1=[n1, n3], p2=[n0, twin0, n2]
///
/// Test:
///
/// Extract enough votes to potentially form commits. Check
/// that no node commits any block. This is because we need
/// 3 nodes to form a quorum and no partition has enough votes
/// (note there are 3 nodes in p2, but one of them is a twin,
/// and its vote will be counted as duplicate of n0).
///
/// Run the test:
/// cargo xtest -p consensus twins_vote_dedup_test -- --nocapture
fn twins_vote_dedup_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 1;
    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RotatingProposer,
        None,
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    // twin of n0 has same author as node[0]
    let twin0_twin_id = nodes[4].id;
    assert_eq!(n0_twin_id.author, twin0_twin_id.author);
    let n1_twin_id = nodes[1].id;
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    assert!(playground.split_network(
        vec![n1_twin_id, n3_twin_id],
        vec![twin0_twin_id, n0_twin_id, n2_twin_id],
    ));

    timed_block_on(&mut runtime, async {
        playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;

        // Pull enough votes to get a few commits.
        // The proposer's votes are implicit and do not go in the queue.
        playground
            .wait_for_messages(50, NetworkPlayground::votes_only)
            .await;

        // No node should be able to commit because of the way partitions
        // have been created
        let mut commit_seen = false;
        for node in &mut nodes {
            if let Ok(Some(_node_commit)) = node.commit_cb_receiver.try_next() {
                commit_seen = true;
            }
        }
        assert!(!commit_seen);
    });
}

#[test]
/// This test checks that when a node becomes a proposer, its
/// twin becomes one too.
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 2 twins (twin0, twin1)
/// Create 2 partitions, p1=[n0, n1, n2], p2=[n3, twin0, twin1]
/// Let n0 (and implicitly twin0) be proposers
///
/// Test:
///
/// Extract enough votes so nodes in both partitions form commits.
/// The commits should be on two different blocks
///
/// Run the test:
/// cargo xtest -p consensus twins_proposer_test -- --nocapture
fn twins_proposer_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 2;

    // Specify round leaders
    // Will default to the first node, if no leader specified for given round
    let mut round_proposers: HashMap<Round, usize> = HashMap::new();
    // Leaders are n0 (and implicitly twin0) for round 1..10
    for i in 1..10 {
        round_proposers.insert(i, 0);
    }

    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RoundProposer(HashMap::new()),
        Some(round_proposers),
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    // twin of n0 has same author as node_authors[0]
    let twin0_twin_id = nodes[4].id;
    assert_eq!(n0_twin_id.author, twin0_twin_id.author);
    let n1_twin_id = nodes[1].id;
    // twin of n1 has same author as node_authors[1]
    let twin1_twin_id = nodes[5].id;
    assert_eq!(n1_twin_id.author, twin1_twin_id.author);
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    // Create per round partitions
    let mut round_partitions: HashMap<u64, Vec<Vec<TwinId>>> = HashMap::new();
    // Round 1 to 10 partitions: [node0, node1, node2], [node3, twin0, twin1]
    for i in 1..10 {
        round_partitions.insert(
            i,
            vec![
                vec![n0_twin_id, n1_twin_id, n2_twin_id],
                vec![n3_twin_id, twin0_twin_id, twin1_twin_id],
            ],
        );
    }
    assert!(playground.split_network_round(&round_partitions));

    timed_block_on(&mut runtime, async {
        // Pull two proposals (by n0 and twin0)
        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await;

        // Pull enough votes to get a few commits.
        playground
            .wait_for_messages(50, NetworkPlayground::votes_only)
            .await;

        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        let twin0_commit = nodes[4].commit_cb_receiver.next().await;

        match (node0_commit, twin0_commit) {
            (Some(node0_commit_inner), Some(twin0_commit_inner)) => {
                let node0_commit_id = node0_commit_inner.ledger_info().commit_info().id();
                let twin0_commit_id = twin0_commit_inner.ledger_info().commit_info().id();
                // Proposal from both node0 and twin_node0 are going to
                // get committed in their respective partitions
                assert_ne!(node0_commit_id, twin0_commit_id);
            }
            _ => panic!("[TwinsTest] Test failed due to no commit(s)"),
        }
    });
}

#[test]
/// This test checks that when a node and its twin are both leaders
/// for a round, only one of the two proposals gets committed
///
/// Setup:
///
/// Network of 4 nodes (n0, n1, n2, n3), and 1 twin (twin0)
///
/// Test:
///
/// Let n0 (and implicitly twin0) be proposers
/// Pull out enough votes so a commit can be formed
/// Check that the commit of n0 and twin0 matches
///
/// Run the test:
/// cargo xtest -p consensus twins_commit_test -- --nocapture
fn twins_commit_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 1;

    // Specify round leaders
    // Will default to the first node, if no leader specified for given round
    let mut round_proposers: HashMap<Round, usize> = HashMap::new();
    // Leaders are n0 and twin0 for round 1..10
    for i in 1..10 {
        round_proposers.insert(i, 0);
    }

    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RoundProposer(HashMap::new()),
        Some(round_proposers),
    );

    timed_block_on(&mut runtime, async {
        // Pull two proposals (by n0 and twin0)
        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only)
            .await;

        // Pull enough votes to get a few commits.
        // The proposer's votes are implicit and do not go in the queue.
        playground
            .wait_for_messages(50, NetworkPlayground::votes_only)
            .await;

        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        let twin0_commit = nodes[4].commit_cb_receiver.next().await;

        match (node0_commit, twin0_commit) {
            (Some(node0_commit_inner), Some(twin0_commit_inner)) => {
                let node0_commit_id = node0_commit_inner.ledger_info().commit_info().id();
                let twin0_commit_id = twin0_commit_inner.ledger_info().commit_info().id();
                // Proposals from both node0 and twin_node0 are going to race,
                // but only one of them will form a commit
                assert_eq!(node0_commit_id, twin0_commit_id);
            }
            _ => panic!("[TwinsTest] Test failed due to no commit(s)"),
        }
    });
}
