// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockReader,
        chained_bft_smr::{ChainedBftSMR, ChainedBftSMRInput},
        network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
        network_tests::NetworkPlayground,
        test_utils::{
            consensus_runtime, timed_block_on, MockSharedStorage, MockStateComputer, MockStorage,
            MockTransactionManager, TestPayload,
        },
    },
    consensus_provider::ConsensusProvider,
    util::mock_time_service::SimulatedTimeService,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::{channel::mpsc, stream::StreamExt};
use libra_config::{
    config::{
        ConsensusConfig,
        ConsensusProposerType::{self, FixedProposer, MultipleOrderedProposers, RotatingProposer},
        NodeConfig, SafetyRulesConfig,
    },
    generator::{self, ValidatorSwarm},
};
use libra_crypto::{hash::CryptoHash, HashValue};
use libra_mempool::mocks::MockSharedMempool;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::{OnChainConfig, OnChainConfigPayload, ValidatorSet},
    validator_verifier::ValidatorVerifier,
};
use network::peer_manager::{
    conn_status_channel, ConnectionRequestSender, PeerManagerRequestSender,
};
use safety_rules::SafetyRulesManager;
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc, time::Duration};
use tokio::runtime;

/// Auxiliary struct that is preparing SMR for the test
struct SMRNode {
    config: NodeConfig,
    smr_id: usize,
    smr: ChainedBftSMR<TestPayload>,
    commit_cb_receiver: mpsc::UnboundedReceiver<LedgerInfoWithSignatures>,
    storage: Arc<MockStorage<TestPayload>>,
    state_sync: mpsc::UnboundedReceiver<Vec<usize>>,
    shared_mempool: MockSharedMempool,
    time_service: Arc<SimulatedTimeService>,
}

impl SMRNode {
    fn start(
        playground: &mut NetworkPlayground,
        config: NodeConfig,
        smr_id: usize,
        storage: Arc<MockStorage<TestPayload>>,
    ) -> Self {
        let author = config.validator_network.as_ref().unwrap().peer_id;

        let (network_reqs_tx, network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (connection_reqs_tx, _) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
        let (_, conn_status_rx) = conn_status_channel::new();
        let network_sender = ConsensusNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
            conn_mgr_reqs_tx,
        );
        let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_status_rx);
        playground.add_node(author, consensus_tx, network_reqs_rx, conn_mgr_reqs_rx);
        let (state_sync_client, state_sync) = mpsc::unbounded();
        let (commit_cb_sender, commit_cb_receiver) = mpsc::unbounded::<LedgerInfoWithSignatures>();
        let shared_mempool = MockSharedMempool::new(None);
        let consensus_to_mempool_sender = shared_mempool.consensus_sender.clone();
        let state_computer = Arc::new(MockStateComputer::new(
            state_sync_client,
            commit_cb_sender,
            Arc::clone(&storage),
        ));
        let txn_manager = Box::new(MockTransactionManager::new(Some(
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
        let time_service = Arc::new(SimulatedTimeService::new());

        let input = ChainedBftSMRInput {
            network_sender,
            network_events,
            safety_rules_manager: SafetyRulesManager::new(&mut config.clone()),
            state_computer,
            txn_manager,
            storage: storage.clone(),
            config: config.consensus.clone(),
            reconfig_events,
            time_service: time_service.clone(),
        };
        let runtime = runtime::Builder::new()
            .thread_name(format!("consensus-{}", smr_id))
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime!");

        let mut smr = ChainedBftSMR::new(author, input, runtime);

        smr.start().expect("Failed to start SMR!");
        Self {
            config,
            smr_id,
            smr,
            commit_cb_receiver,
            storage,
            state_sync,
            shared_mempool,
            time_service,
        }
    }

    fn restart_with_empty_shared_storage(mut self, playground: &mut NetworkPlayground) -> Self {
        let validator_set = self.storage.shared_storage.validator_set.clone();
        let shared_storage = Arc::new(MockSharedStorage::new(validator_set));
        self.storage = Arc::new(MockStorage::new_with_ledger_info(
            shared_storage,
            self.storage.get_ledger_info(),
        ));
        self.restart(playground)
    }

    fn restart(mut self, playground: &mut NetworkPlayground) -> Self {
        self.smr.stop();
        Self::start(playground, self.config, self.smr_id + 10, self.storage)
    }

    fn start_num_nodes(
        num_nodes: usize,
        playground: &mut NetworkPlayground,
        proposer_type: ConsensusProposerType,
    ) -> Vec<Self> {
        let ValidatorSwarm {
            mut nodes,
            validator_set,
            ..
        } = generator::validator_swarm_for_testing(num_nodes);

        // Some tests make assumptions about the ordering of configs in relation
        // to the FixedProposer which should be the first proposer in lexical order.
        nodes.sort_by(|a, b| {
            let a_auth = a.validator_network.as_ref().unwrap().peer_id;
            let b_auth = b.validator_network.as_ref().unwrap().peer_id;
            a_auth.cmp(&b_auth)
        });

        let mut smr_nodes = vec![];
        for (smr_id, config) in nodes.iter().enumerate() {
            let mut node_config = config.clone();
            node_config.consensus.proposer_type = proposer_type;
            // Use in memory storage for testing
            node_config.consensus.safety_rules = SafetyRulesConfig::default();

            let (_, storage) = MockStorage::start_for_testing(validator_set.clone());
            smr_nodes.push(Self::start(playground, node_config, smr_id, storage));
        }
        smr_nodes
    }
}

fn verify_finality_proof(node: &SMRNode, ledger_info_with_sig: &LedgerInfoWithSignatures) {
    let validators = ValidatorVerifier::from(&node.storage.shared_storage.validator_set);
    let ledger_info_hash = ledger_info_with_sig.ledger_info().hash();
    for (author, signature) in ledger_info_with_sig.signatures() {
        assert_eq!(
            Ok(()),
            validators.verify_signature(*author, ledger_info_hash, &signature)
        );
    }
}

#[test]
/// Should receive a new proposal upon start
fn basic_start_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let nodes = SMRNode::start_num_nodes(2, &mut playground, RotatingProposer);
    timed_block_on(&mut runtime, async {
        let msg = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        let genesis = nodes[0]
            .smr
            .block_store()
            .expect("No valid block store!")
            .root();
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
/// Upon startup, the first proposal is sent, delivered and voted by all the participants.
fn start_with_proposal_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let nodes = SMRNode::start_num_nodes(2, &mut playground, RotatingProposer);

    timed_block_on(&mut runtime, async {
        let proposals: Vec<_> = playground
            .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
            .await
            .into_iter()
            .map(|(_, msg)| match msg {
                ConsensusMsg::ProposalMsg(m) => m,
                _ => panic!("Unexpected message found"),
            })
            .collect();
        let proposed_block_id = proposals[0].proposal().id();

        // Verify that the proposed block id is indeed present in the block store.
        assert!(nodes[0]
            .smr
            .block_store()
            .unwrap()
            .get_block(proposed_block_id)
            .is_some());
        assert!(nodes[1]
            .smr
            .block_store()
            .unwrap()
            .get_block(proposed_block_id)
            .is_some());
    });
}

fn basic_full_round(num_nodes: usize, proposer_type: ConsensusProposerType) {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let _nodes = SMRNode::start_num_nodes(num_nodes, &mut playground, proposer_type);

    // In case we're using multi-proposer, every proposal and vote is sent to two participants.
    let num_messages_to_send = if proposer_type == MultipleOrderedProposers {
        2 * (num_nodes - 1)
    } else {
        num_nodes - 1
    };
    timed_block_on(&mut runtime, async {
        let _broadcast_proposals_1 = playground
            .wait_for_messages(
                num_messages_to_send,
                NetworkPlayground::proposals_only::<TestPayload>,
            )
            .await;
        let _votes_1 = playground
            .wait_for_messages(
                num_messages_to_send,
                NetworkPlayground::votes_only::<TestPayload>,
            )
            .await;
        let broadcast_proposals_2 = playground
            .wait_for_messages(
                num_messages_to_send,
                NetworkPlayground::proposals_only::<TestPayload>,
            )
            .await;
        let msg = &broadcast_proposals_2;
        let next_proposal = match &msg[0].1 {
            ConsensusMsg::ProposalMsg(proposal) => proposal,
            _ => panic!("Unexpected message found"),
        };
        assert!(next_proposal.proposal().round() >= 2);
    });
}

#[test]
/// Upon startup, the first proposal is sent, voted by all the participants, QC is formed and
/// then the next proposal is sent.
fn basic_full_round_test() {
    basic_full_round(2, FixedProposer);
}

#[test]
/// Basic happy path with multiple proposers
fn happy_path_with_multi_proposer() {
    basic_full_round(2, MultipleOrderedProposers);
}

async fn basic_commit(
    playground: &mut NetworkPlayground,
    nodes: &mut Vec<SMRNode>,
    block_ids: &mut Vec<HashValue>,
) {
    let num_rounds = 10;

    for round in 0..num_rounds {
        let _proposals = playground
            .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg::<TestPayload>)
            .await;

        // A proposal is carrying a QC that commits a block of round - 3.
        if round >= 3 {
            let block_id_to_commit = block_ids[round - 3];
            let commit_v1 = nodes[0].commit_cb_receiver.next().await.unwrap();
            let commit_v2 = nodes[1].commit_cb_receiver.next().await.unwrap();
            assert_eq!(
                commit_v1.ledger_info().consensus_block_id(),
                block_id_to_commit
            );
            verify_finality_proof(&nodes[0], &commit_v1);
            assert_eq!(
                commit_v2.ledger_info().consensus_block_id(),
                block_id_to_commit
            );
            verify_finality_proof(&nodes[1], &commit_v2);
        }

        // v1 and v2 send votes
        let votes = playground
            .wait_for_messages(1, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        let vote_msg = match &votes[0].1 {
            ConsensusMsg::VoteMsg(vote_msg) => vote_msg,
            _ => panic!("Unexpected message found"),
        };
        block_ids.push(vote_msg.vote().vote_data().proposed().id());
    }

    assert!(
        nodes[0].smr.block_store().unwrap().root().round() >= 7,
        "round of node 0 is {}",
        nodes[0].smr.block_store().unwrap().root().round()
    );
    assert!(
        nodes[1].smr.block_store().unwrap().root().round() >= 7,
        "round of node 1 is {}",
        nodes[1].smr.block_store().unwrap().root().round()
    );

    // This message is for proposal with round 11 to delivery the QC, but not gather the QC
    // so after restart, proposer will propose round 11 again.
    playground
        .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg::<TestPayload>)
        .await;
}

/// Verify the basic e2e flow: blocks are committed, txn manager is notified, block tree is
/// pruned, restart the node and we can still continue.
#[test]
fn basic_commit_and_restart() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = SMRNode::start_num_nodes(2, &mut playground, RotatingProposer);
    let mut block_ids = vec![];

    timed_block_on(
        &mut runtime,
        basic_commit(&mut playground, &mut nodes, &mut block_ids),
    );

    // create a new playground to avoid polling potential vote messages in previous one.
    playground = NetworkPlayground::new(runtime.handle().clone());
    let _nodes: Vec<_> = nodes
        .into_iter()
        .map(|node| node.restart(&mut playground))
        .collect();

    // 1. the nodes recover the previous state (vote round > 10)
    // 2. continue making progress (commit round >= 17)
    timed_block_on(&mut runtime, async {
        // The loop is to ensure that we collect a network vote(enough for QC with 2 nodes) then
        // move the round forward.
        loop {
            let msg = playground
                .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg)
                .await;
            if let ConsensusMsg::<TestPayload>::VoteMsg(m) = &msg[0].1 {
                assert!(m.vote().vote_data().proposed().round() > 10);
                if m.vote().ledger_info().round() >= 17 {
                    break;
                }
            }
        }
    });
}

/// Test restart with an empty shared storage to simulate empty consensus db
#[test]
fn basic_commit_and_restart_from_clean_storage() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = SMRNode::start_num_nodes(2, &mut playground, RotatingProposer);
    let mut block_ids = vec![];

    timed_block_on(
        &mut runtime,
        basic_commit(&mut playground, &mut nodes, &mut block_ids),
    );

    let _nodes: Vec<_> = nodes
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            if index == 0 {
                node.restart_with_empty_shared_storage(&mut playground)
            } else {
                node.restart(&mut playground)
            }
        })
        .collect();

    // 1. the nodes recover the previous state (vote round > 10)
    // 2. continue making progress (commit round >= 17)
    timed_block_on(&mut runtime, async {
        // The loop is to ensure that we collect a network vote(enough for QC with 2 nodes) then
        // move the round forward.
        loop {
            let msg = playground
                .wait_for_messages(1, NetworkPlayground::exclude_timeout_msg)
                .await;
            if let ConsensusMsg::<TestPayload>::VoteMsg(m) = &msg[0].1 {
                assert!(m.vote().vote_data().proposed().round() > 10);
                if m.vote().ledger_info().round() >= 17 {
                    break;
                }
            }
        }
    });
}

#[test]
fn basic_block_retrieval() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(4, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        let mut first_proposals = vec![];
        // First three proposals are delivered just to nodes[0..2].
        playground.drop_message_for(&nodes[0].smr.author(), nodes[3].smr.author());
        for _ in 0..2 {
            playground
                .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
                .await;
            let votes = playground
                .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
                .await;
            let vote_msg = match &votes[0].1 {
                ConsensusMsg::VoteMsg(vote_msg) => vote_msg,
                _ => panic!("Unexpected message found"),
            };
            let proposal_id = vote_msg.vote().vote_data().proposed().id();
            first_proposals.push(proposal_id);
        }
        // The next proposal is delivered to all: as a result nodes[2] should retrieve the missing
        // blocks from nodes[0] and vote for the 3th proposal.
        playground.stop_drop_message_for(&nodes[0].smr.author(), &nodes[3].smr.author());
        // Drop nodes[1]'s vote to ensure nodes[3] contribute to the quorum
        playground.drop_message_for(&nodes[0].smr.author(), nodes[1].smr.author());

        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        playground
            .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        // The first two proposals should be present at nodes[3] via block retrieval
        for block_id in &first_proposals {
            assert!(nodes[3]
                .smr
                .block_store()
                .unwrap()
                .get_block(*block_id)
                .is_some());
        }

        // 4th proposal will get quorum and verify that nodes[3] commits the first proposal.
        playground
            .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        playground
            .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        if let Some(commit_v3) = nodes[3].commit_cb_receiver.next().await {
            assert_eq!(
                commit_v3.ledger_info().consensus_block_id(),
                first_proposals[0],
            );
        }
    });
}

#[test]
fn block_retrieval_with_timeout() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let nodes = SMRNode::start_num_nodes(4, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        let mut first_proposals = vec![];
        // First three proposals are delivered just to nodes[0..2].
        playground.drop_message_for(&nodes[0].smr.author(), nodes[3].smr.author());
        for _ in 0..2 {
            playground
                .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
                .await;
            let votes = playground
                .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
                .await;
            let vote_msg = match &votes[0].1 {
                ConsensusMsg::VoteMsg(vote_msg) => vote_msg,
                _ => panic!("Unexpected message found"),
            };
            let proposal_id = vote_msg.vote().vote_data().proposed().id();
            first_proposals.push(proposal_id);
        }
        // stop proposals from nodes[0]
        playground.drop_message_for(&nodes[0].smr.author(), nodes[1].smr.author());
        playground.drop_message_for(&nodes[0].smr.author(), nodes[2].smr.author());

        nodes[3]
            .time_service
            .update_auto_advance_limit(Duration::from_secs(1))
            .await;
        playground
            .wait_for_messages(1, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await;

        // the first two proposals should be present at nodes[3]
        for block_id in &first_proposals {
            assert!(nodes[2]
                .smr
                .block_store()
                .unwrap()
                .get_block(*block_id)
                .is_some());
        }
    });
}

#[test]
/// Verify that a node that is lagging behind can catch up by state sync some blocks
/// have been pruned by the others.
fn basic_state_sync() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(4, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        let mut proposals = vec![];
        // The first ten proposals are delivered just to nodes[0..2], which should commit
        // the first seven blocks.
        playground.drop_message_for(&nodes[0].smr.author(), nodes[3].smr.author());
        for _ in 0..10 {
            playground
                .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
                .await;
            let votes = playground
                .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
                .await;
            let vote_msg = match &votes[0].1 {
                ConsensusMsg::VoteMsg(vote_msg) => vote_msg,
                _ => panic!("Unexpected message found"),
            };
            let proposal_id = vote_msg.vote().vote_data().proposed().id();
            proposals.push(proposal_id);
        }

        let mut node0_commits = vec![];
        for i in 0..7 {
            node0_commits.push(
                nodes[0]
                    .commit_cb_receiver
                    .next()
                    .await
                    .unwrap()
                    .ledger_info()
                    .consensus_block_id(),
            );
            assert_eq!(node0_commits[i], proposals[i]);
        }

        // Next proposal is delivered to all: as a result nodes[3] should be able to retrieve the
        // missing blocks from nodes[0] and commit the first eight proposals as well.
        playground.stop_drop_message_for(&nodes[0].smr.author(), &nodes[3].smr.author());
        playground
            .wait_for_messages(3, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        let mut node3_commits = vec![];
        // The only notification we will receive is for the last (8th) proposal.
        node3_commits.push(
            nodes[3]
                .commit_cb_receiver
                .next()
                .await
                .unwrap()
                .ledger_info()
                .consensus_block_id(),
        );
        assert_eq!(node3_commits[0], proposals[7]);

        // wait for the vote from all including node3
        playground
            .wait_for_messages(3, NetworkPlayground::votes_only::<TestPayload>)
            .await;
        playground
            .wait_for_messages(3, NetworkPlayground::proposals_only::<TestPayload>)
            .await;

        let committed_txns = nodes[3]
            .state_sync
            .next()
            .await
            .expect("MockStateSync failed to be notified by a mempool committed txns");
        let max_block_size = ConsensusConfig::default().max_block_size as usize;
        assert_eq!(committed_txns.len(), max_block_size);
    });
}

#[test]
/// Verify that a node syncs up when receiving a timeout message with a relevant ledger info
fn state_sync_on_timeout() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // This test depends on the fixed proposer on nodes[0]
    let mut nodes = SMRNode::start_num_nodes(4, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        // The first ten proposals are delivered just to nodes[0..2], which should commit
        // the first seven blocks.
        for node in &nodes[0..3] {
            playground.drop_message_for(&node.smr.author(), nodes[3].smr.author());
        }
        for _ in 0..10 {
            playground
                .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
                .await;
            playground
                .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
                .await;
        }

        // Stop dropping messages from node 1 to node 3: next time node 1 sends a timeout to node 3,
        // which carries a SyncInfo with HCC at round >= 7
        playground.stop_drop_message_for(&nodes[1].smr.author(), &nodes[3].smr.author());
        playground.drop_message_for(&nodes[1].smr.author(), nodes[0].smr.author());
        playground.drop_message_for(&nodes[1].smr.author(), nodes[2].smr.author());
        nodes[1]
            .time_service
            .update_auto_advance_limit(Duration::from_millis(1000 + 10))
            .await;
        // Wait for the sync info message from 1 to 3
        playground
            .wait_for_messages(1, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await;
        // In the end of the state synchronization node 3 should have commit at round >= 7.
        assert!(
            nodes[3]
                .commit_cb_receiver
                .next()
                .await
                .unwrap()
                .ledger_info()
                .round()
                >= 7
        );
    });
}

#[test]
/// Verify that in case a node receives timeout message from a remote peer that is lagging behind,
/// then this node sends a sync info, which helps the remote to properly catch up.
fn sync_info_sent_if_remote_stale() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    // This test depends on the fixed proposer on nodes[0]
    // We're going to drop messages from 0 to 2: as a result we expect node 2 to broadcast timeout
    // messages, for which node 1 should respond with sync_info, which should eventually
    // help node 2 to catch up.
    let mut nodes = SMRNode::start_num_nodes(4, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        // The first ten proposals are delivered just to nodes[0..2], which should commit
        // the first seven blocks.
        for node in &nodes[0..3] {
            playground.drop_message_for(&node.smr.author(), nodes[3].smr.author());
        }
        for _ in 0..10 {
            playground
                .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
                .await;
            playground
                .wait_for_messages(2, NetworkPlayground::votes_only::<TestPayload>)
                .await;
        }

        // Stop dropping messages from node 1 to node 3: next time node 3 sends a timeout to node 1
        // with stale SyncInfo, node 1 will reply back a recent SyncInfo to help node 3 catch up
        playground.stop_drop_message_for(&nodes[1].smr.author(), &nodes[3].smr.author());
        nodes[3]
            .time_service
            .update_auto_advance_limit(Duration::from_secs(1))
            .await;
        // Wait for node 3 to broadcast the timeout to node 1
        playground
            .wait_for_messages(1, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await;
        // Wait for node 1 to reply a sync info back
        playground
            .wait_for_messages(1, NetworkPlayground::sync_info_only::<TestPayload>)
            .await;
        // In the end of the state synchronization node 3 should have commit at round >= 7.
        assert!(
            nodes[3]
                .commit_cb_receiver
                .next()
                .await
                .unwrap()
                .ledger_info()
                .round()
                >= 7
        );
    });
}

#[test]
/// Verify that a QC can be formed by aggregating the votes piggybacked by TimeoutMsgs
fn aggregate_timeout_votes() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());

    // The proposer node[0] sends its proposal to nodes 1 and 2, which cannot respond back,
    // because their messages are dropped.
    // Upon timeout nodes 1 and 2 are sending timeout messages with attached votes for the original
    // proposal: both can then aggregate the QC for the first proposal.
    let nodes = SMRNode::start_num_nodes(3, &mut playground, FixedProposer);
    timed_block_on(&mut runtime, async {
        // Nodes 1 and 2 cannot send votes to node 0
        playground.drop_message_for(&nodes[1].smr.author(), nodes[0].smr.author());
        playground.drop_message_for(&nodes[2].smr.author(), nodes[0].smr.author());

        // Node 0 sends proposals to nodes 1 and 2
        let msg = playground
            .wait_for_messages(2, NetworkPlayground::proposals_only::<TestPayload>)
            .await;
        let first_proposal = match &msg[0].1 {
            ConsensusMsg::ProposalMsg(proposal) => proposal,
            _ => panic!("Unexpected message found"),
        };
        let proposal_id = first_proposal.proposal().id();

        // Wait for node 1,2 receive proposal and vote
        tokio::time::delay_for(Duration::from_secs(1)).await;
        // Wait for the timeout messages sent by 0 -> [1, 2], 1 -> 2, 2 -> 1.
        for node in &nodes {
            node.time_service
                .update_auto_advance_limit(Duration::from_millis(1000))
                .await;
        }
        playground
            .wait_for_messages(4, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await;

        // Node 0 cannot form a QC
        assert_eq!(
            nodes[0]
                .smr
                .block_store()
                .unwrap()
                .highest_quorum_cert()
                .certified_block()
                .round(),
            0
        );
        // Wait for the QC generated and new round started.
        tokio::time::delay_for(Duration::from_secs(1)).await;
        // Nodes 1 and 2 form a QC and move to the next round.
        // Wait for the timeout messages from 1 and 2
        for node in &nodes[1..] {
            node.time_service
                .update_auto_advance_limit(Duration::from_millis(1500))
                .await;
        }

        playground
            .wait_for_messages(2, NetworkPlayground::timeout_votes_only::<TestPayload>)
            .await;

        assert_eq!(
            nodes[1]
                .smr
                .block_store()
                .unwrap()
                .highest_quorum_cert()
                .certified_block()
                .id(),
            proposal_id
        );
        assert_eq!(
            nodes[2]
                .smr
                .block_store()
                .unwrap()
                .highest_quorum_cert()
                .certified_block()
                .id(),
            proposal_id
        );
    });
}

#[test]
/// Verify that the NIL blocks formed during timeouts can be used to form commit chains.
fn chain_with_nil_blocks() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());

    // The proposer node[0] sends 3 proposals, after that its proposals are dropped and it cannot
    // communicate with nodes 1, 2, 3. Nodes 1, 2, 3 should be able to commit the 3 proposal
    // via NIL blocks commit chain.
    let num_nodes = 4;
    let nodes = SMRNode::start_num_nodes(num_nodes, &mut playground, FixedProposer);
    let num_proposal = 3;
    timed_block_on(&mut runtime, async {
        // Wait for the first 3 proposals (each one sent to 3 nodes).
        for _ in 0..num_proposal {
            playground
                .wait_for_messages(
                    num_nodes - 1,
                    NetworkPlayground::proposals_only::<TestPayload>,
                )
                .await;
            playground
                .wait_for_messages(num_nodes - 1, NetworkPlayground::votes_only::<TestPayload>)
                .await;
        }
        for node in &nodes {
            playground.drop_message_for(&nodes[0].smr.author(), node.smr.author());
        }

        // After the first timeout nodes 1, 2, 3 should have last_proposal votes and
        // they can generate its QC independently.
        // Upon the second timeout nodes 1, 2, 3 send NIL block_1 with a QC to last_proposal.
        // G <- p1 <- p2 <- p3 <- NIL1
        for node in &nodes[1..] {
            // the magic num_proposal comes from 3 proposal because we sleep for 1 millis before propose next
            node.time_service
                .update_auto_advance_limit(Duration::from_millis(2250 + num_proposal))
                .await;
        }
        playground
            .wait_for_messages(
                // all-to-all broadcast except nodes 0's messages are dropped and self messages don't count
                (num_nodes - 1) * (num_nodes - 1),
                NetworkPlayground::timeout_votes_only::<TestPayload>,
            )
            .await;
        // Wait for new round start.
        tokio::time::delay_for(Duration::from_secs(1)).await;

        for node in &nodes[1..] {
            node.time_service
                .update_auto_advance_limit(Duration::from_millis(1000))
                .await;
        }
        playground
            .wait_for_messages(
                // all-to-all broadcast except nodes 0's messages are dropped and self messages don't count
                (num_nodes - 1) * (num_nodes - 1),
                NetworkPlayground::timeout_votes_only::<TestPayload>,
            )
            .await;
        // We can't guarantee the timing of the last timeout processing, the only thing we can
        // look at is that HQC round is at least 3.
        assert!(
            nodes[2]
                .smr
                .block_store()
                .unwrap()
                .highest_quorum_cert()
                .certified_block()
                .round()
                >= 3,
        );

        assert!(nodes[2].smr.block_store().unwrap().root().round() >= 1)
    });
}

#[test]
/// Test secondary proposal processing
fn secondary_proposers() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());

    let num_nodes = 4;
    let nodes = SMRNode::start_num_nodes(num_nodes, &mut playground, MultipleOrderedProposers);
    let primary_node = 2;
    timed_block_on(&mut runtime, async {
        // Node 0 is disconnected.
        for node in &nodes {
            playground.drop_message_for(&nodes[primary_node].smr.author(), node.smr.author());
        }

        // wait for secondary proposal sent to all
        playground
            .wait_for_messages(
                num_nodes - 1,
                NetworkPlayground::proposals_only::<TestPayload>,
            )
            .await;
        // Wait for the proposal received.
        tokio::time::delay_for(Duration::from_secs(1)).await;

        for node in &nodes {
            node.time_service
                .update_auto_advance_limit(Duration::from_secs(1))
                .await;
        }

        // Run a system until node 0 is a designated primary proposer. In this round the
        // secondary proposal should be voted for and attached to the timeout message.
        let timeout_votes = playground
            .wait_for_messages(
                (num_nodes - 1) * (num_nodes - 1),
                NetworkPlayground::timeout_votes_only::<TestPayload>,
            )
            .await;
        let mut secondary_proposal_ids = vec![];
        for msg in timeout_votes {
            let vote_msg = match msg.1 {
                ConsensusMsg::VoteMsg(vote_msg) => vote_msg,
                _ => panic!("Unexpected message found"),
            };
            assert!(vote_msg.vote().is_timeout());
            secondary_proposal_ids.push(vote_msg.vote().vote_data().proposed().id());
        }
        assert_eq!(
            secondary_proposal_ids.len(),
            (num_nodes - 1) * (num_nodes - 1)
        );
        let secondary_proposal_id = secondary_proposal_ids[0];
        for id in secondary_proposal_ids {
            assert_eq!(secondary_proposal_id, id);
        }
    });
}
