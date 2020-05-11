// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_manager::EpochManager,
    network::NetworkTask,
    network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
    network_tests::NetworkPlayground,
    test_utils::{
        consensus_runtime, timed_block_on, MockStateComputer, MockStorage, MockTransactionManager,
        TestPayload,
    },
    util::mock_time_service::SimulatedTimeService,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::block::Block;
use futures::channel::mpsc;
use libra_config::{
    config::{
        ConsensusProposerType::{self, RotatingProposer},
        NodeConfig,
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
use network::peer_manager::{
    conn_notifs_channel, ConnectionRequestSender, PeerManagerRequestSender,
};
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use tokio::runtime::{Builder, Runtime};

/// Auxiliary struct that is preparing SMR for the test
struct SMRNode {
    config: NodeConfig,
    smr_id: usize,
    runtime: Runtime,
    commit_cb_receiver: mpsc::UnboundedReceiver<LedgerInfoWithSignatures>,
    storage: Arc<MockStorage<TestPayload>>,
    state_sync: mpsc::UnboundedReceiver<Vec<usize>>,
    shared_mempool: MockSharedMempool,
}

impl SMRNode {
    fn start(
        playground: &mut NetworkPlayground,
        mut config: NodeConfig,
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
        let (_, conn_notifs_channel) = conn_notifs_channel::new();
        let network_sender = ConsensusNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
            conn_mgr_reqs_tx,
        );
        let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_notifs_channel);
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
        let (self_sender, self_receiver) = channel::new_test(1_024);
        let (timeout_sender, timeout_receiver) = channel::new_test(1_024);

        let runtime = Builder::new()
            .thread_name(format!("node-{}", smr_id))
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
        let time_service = Arc::new(SimulatedTimeService::new());

        let epoch_mgr = EpochManager::new(
            &mut config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage.clone(),
        );
        let (network_task, network_receiver) = NetworkTask::new(network_events, self_receiver);

        runtime.spawn(network_task.start());
        runtime.spawn(epoch_mgr.start(timeout_receiver, network_receiver, reconfig_events));
        Self {
            config,
            smr_id,
            runtime,
            commit_cb_receiver,
            storage,
            state_sync,
            shared_mempool,
        }
    }

    fn start_num_nodes(
        num_nodes: usize,
        playground: &mut NetworkPlayground,
        proposer_type: ConsensusProposerType,
    ) -> Vec<Self> {
        let ValidatorSwarm { nodes } = generator::validator_swarm_for_testing(num_nodes);
        let validator_set = ValidatorSet::new(
            nodes
                .iter()
                .map(|config| {
                    ValidatorInfo::new_with_test_network_keys(
                        config.validator_network.as_ref().unwrap().peer_id, // account address
                        config
                            .test
                            .as_ref()
                            .unwrap()
                            .consensus_keypair
                            .as_ref()
                            .unwrap()
                            .public_key(), // consensus pubkey
                        1,                                                  // voting power
                    )
                })
                .collect(),
        );

        let mut smr_nodes = vec![];
        for (smr_id, mut config) in nodes.into_iter().enumerate() {
            let (_, storage) = MockStorage::start_for_testing(validator_set.clone());

            let waypoint = Waypoint::new_epoch_boundary(&storage.get_ledger_info())
                .expect("Unable to produce waypoint with the provided LedgerInfo");
            config.base.waypoint = Some(waypoint);
            config.consensus.proposer_type = proposer_type;
            // Use in memory storage for testing
            // node_config.consensus.safety_rules = SafetyRulesConfig::default();

            smr_nodes.push(Self::start(playground, config, smr_id, storage));
        }
        smr_nodes
    }
}

#[test]
fn basic_start_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let nodes = SMRNode::start_num_nodes(4, &mut playground, RotatingProposer);
    let genesis = Block::<TestPayload>::make_genesis_block_from_ledger_info(
        &nodes[0].storage.get_ledger_info(),
    );
    timed_block_on(&mut runtime, async {
        let msg = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only::<TestPayload>)
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
