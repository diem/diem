// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    epoch_manager::EpochManager,
    experimental::{
        commit_phase::CommitPhase, execution_phase::ExecutionPhase,
        order_state_computer::OrderProxy,
    },
    network::NetworkTask,
    network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
    persistent_liveness_storage::StorageWriteProxy,
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
    util::time_service::ClockTimeService,
};
use channel::diem_channel;
use diem_config::config::NodeConfig;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_mempool::ConsensusRequest;
use diem_types::on_chain_config::OnChainConfigPayload;
use execution_correctness::ExecutionCorrectnessManager;
use futures::channel::mpsc;
use state_sync::client::StateSyncClient;
use std::{
    collections::HashSet,
    sync::{atomic::AtomicU64, Arc},
};
use storage_interface::DbReader;
use tokio::runtime::{self, Runtime};

/// Helper function to start consensus based on configuration and return the runtime
pub fn start_consensus(
    node_config: &NodeConfig,
    network_sender: ConsensusNetworkSender,
    network_events: ConsensusNetworkEvents,
    state_sync_client: StateSyncClient,
    consensus_to_mempool_sender: mpsc::Sender<ConsensusRequest>,
    diem_db: Arc<dyn DbReader>,
    reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
) -> Runtime {
    let runtime = runtime::Builder::new_multi_thread()
        .thread_name("consensus")
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime!");
    let storage = Arc::new(StorageWriteProxy::new(node_config, diem_db));
    let pending_txns = Arc::new(Mutex::new(HashSet::new()));
    let txn_manager = Arc::new(MempoolProxy::new(
        consensus_to_mempool_sender,
        node_config.consensus.mempool_poll_count,
        node_config.consensus.mempool_txn_pull_timeout_ms,
        node_config.consensus.mempool_executed_txn_timeout_ms,
        pending_txns.clone(),
    ));
    let execution_correctness_manager = ExecutionCorrectnessManager::new(node_config);
    let state_computer = Arc::new(ExecutionProxy::new(
        execution_correctness_manager.client(),
        state_sync_client,
    ));
    let time_service = Arc::new(ClockTimeService::new(runtime.handle().clone()));

    let (timeout_sender, timeout_receiver) = channel::new(1_024, &counters::PENDING_ROUND_TIMEOUTS);
    let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);

    let back_pressure = Arc::new(AtomicU64::new(0));
    let (execute_tx, execute_rx) = channel::new_test(1000);
    let (commit_tx, commit_rx) = channel::new_test(1000);
    let order_only_computer = Arc::new(OrderProxy::new(execute_tx, pending_txns.clone()));
    let execution_phase = ExecutionPhase::new(
        state_computer.clone(),
        execute_rx,
        commit_tx,
        back_pressure.clone(),
        pending_txns,
    );
    let commit_phase = CommitPhase::new(state_computer.clone(), commit_rx);

    let epoch_mgr = EpochManager::new(
        node_config,
        time_service,
        self_sender,
        network_sender,
        timeout_sender,
        txn_manager,
        order_only_computer,
        storage,
        reconfig_events,
        back_pressure,
    );

    let (network_task, network_receiver) = NetworkTask::new(network_events, self_receiver);

    runtime.spawn(network_task.start());
    runtime.spawn(epoch_mgr.start(timeout_receiver, network_receiver));
    runtime.spawn(execution_phase.start());
    runtime.spawn(commit_phase.start());

    debug!("Consensus started.");
    runtime
}
