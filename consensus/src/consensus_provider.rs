// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    epoch_manager::EpochManager,
    network::NetworkTask,
    network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
    persistent_liveness_storage::StorageWriteProxy,
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
    util::time_service::ClockTimeService,
};
use channel::libra_channel;
use execution_correctness::ExecutionCorrectnessManager;
use futures::channel::mpsc;
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_mempool::ConsensusRequest;
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction};
use state_synchronizer::StateSyncClient;
use std::{boxed::Box, sync::Arc};
use storage_interface::DbReader;
use tokio::runtime::{self, Runtime};

/// Helper function to start consensus based on configuration and return the runtime
pub fn start_consensus(
    node_config: &mut NodeConfig,
    network_sender: ConsensusNetworkSender<Vec<SignedTransaction>>,
    network_events: ConsensusNetworkEvents<Vec<SignedTransaction>>,
    state_sync_client: Arc<StateSyncClient>,
    consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>,
    libra_db: Arc<dyn DbReader>,
    reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
) -> Runtime {
    let runtime = runtime::Builder::new()
        .thread_name("consensus-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime!");
    let storage = Arc::new(StorageWriteProxy::new(node_config, libra_db));
    let txn_manager = Box::new(MempoolProxy::new(consensus_to_mempool_sender));
    let execution_correctness_manager = ExecutionCorrectnessManager::new(node_config);
    let state_computer = Arc::new(ExecutionProxy::new(
        execution_correctness_manager.client(),
        state_sync_client,
    ));
    let time_service = Arc::new(ClockTimeService::new(runtime.handle().clone()));

    let (timeout_sender, timeout_receiver) = channel::new(1_024, &counters::PENDING_ROUND_TIMEOUTS);
    let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);

    let epoch_mgr = EpochManager::new(
        node_config,
        time_service,
        self_sender,
        network_sender,
        timeout_sender,
        txn_manager,
        state_computer,
        storage,
    );

    let (network_task, network_receiver) = NetworkTask::new(network_events, self_receiver);

    runtime.spawn(network_task.start());
    runtime.spawn(epoch_mgr.start(timeout_receiver, network_receiver, reconfig_events));

    debug!("Consensus started.");
    runtime
}
