// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    epoch_manager::EpochManager,
    network::NetworkTask,
    network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
    network_tests::{NetworkPlayground, TwinId},
    test_utils::{MockStateComputer, MockStorage, MockTransactionManager},
    twins::test_cases::TestCase,
    util::time_service::ClockTimeService,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::common::{Author, Payload};
use futures::channel::mpsc;
use libra_config::{
    config::{ConsensusProposerType, NodeConfig, WaypointConfig},
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
pub struct SMRNode {
    pub id: TwinId,
    pub storage: Arc<MockStorage>,
    pub commit_cb_receiver: mpsc::UnboundedReceiver<LedgerInfoWithSignatures>,
    pub timeout_sender: channel::Sender<u64>,
    _runtime: Runtime,
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
            timeout_sender.clone(),
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
            timeout_sender,
            commit_cb_receiver,
            storage,
            _shared_mempool: shared_mempool,
            _state_sync: state_sync,
        }
    }

    /// Starts a given number of nodes and their twins
    pub fn start_num_nodes_with_twins(
        num_nodes: usize,
        num_twins: usize,
        playground: &mut NetworkPlayground,
        test_case: Option<TestCase>,
    ) -> Vec<Self> {
        assert!(num_nodes >= num_twins);
        let ValidatorSwarm {
            nodes: mut node_configs,
        } = generator::validator_swarm_for_testing(num_nodes);

        let validator_set = ValidatorSet::new(
            node_configs
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
        node_configs.sort_by(|n1, n2| author_from_config(n1).cmp(&author_from_config(n2)));

        // We don't add twins to ValidatorSet or round_proposers above
        // because a node with twins should be treated the same at the
        // consensus level
        for i in 0..num_twins {
            let twin = node_configs[i].clone();
            node_configs.push(twin);
        }

        let proposer_configs: Vec<_> = match test_case {
            Some(test_case) => {
                let nodes_id: Vec<_> = node_configs
                    .iter()
                    .enumerate()
                    .map(|(id, config)| TwinId {
                        id,
                        author: author_from_config(config),
                    })
                    .collect();
                let configs = test_case
                    .to_round_proposer_config(&nodes_id)
                    .into_iter()
                    .map(|config| ConsensusProposerType::RoundProposer(config))
                    .collect();
                assert!(playground.split_network_round(&test_case.to_partitions(&nodes_id)));
                configs
            }
            None => node_configs
                .iter()
                .map(|_| ConsensusProposerType::FixedProposer)
                .collect(),
        };

        let mut smr_nodes = vec![];

        for ((smr_id, mut config), proposer_type) in
            node_configs.into_iter().enumerate().zip(proposer_configs)
        {
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
            config.consensus.proposer_type = proposer_type;
            config.consensus.safety_rules.verify_vote_proposal_signature = false;
            // NO AUTO TIMEOUT, it should be manually set by the RoundProposerConfig
            config.consensus.round_initial_timeout_ms = 2_000_000;

            let author = author_from_config(&config);

            let twin_id = TwinId { id: smr_id, author };

            smr_nodes.push(Self::start(playground, config, storage, twin_id));
        }
        smr_nodes
    }
}
