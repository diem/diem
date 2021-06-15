// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    epoch_manager::EpochManager,
    network::NetworkTask,
    network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
    persistent_liveness_storage::StorageWriteProxy,
    state_computer::ExecutionProxy,
    experimental::{ordering_state_computer::OrderingStateComputer, execution_phase::ExecutionPhase, commit_phase::CommitPhase},
    state_replication::StateComputer,
    txn_manager::MempoolProxy,
    util::time_service::ClockTimeService,
};
use channel::diem_channel;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use diem_config::config::NodeConfig;
use diem_logger::prelude::*;
use diem_mempool::ConsensusRequest;
use diem_metrics::IntGauge;
use diem_types::on_chain_config::OnChainConfigPayload;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use execution_correctness::ExecutionCorrectnessManager;
use futures::channel::mpsc;
use state_sync::client::StateSyncClient;
use std::sync::Arc;
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
    let txn_manager = Arc::new(MempoolProxy::new(
        consensus_to_mempool_sender,
        node_config.consensus.mempool_poll_count,
        node_config.consensus.mempool_txn_pull_timeout_ms,
        node_config.consensus.mempool_executed_txn_timeout_ms,
    ));

    let execution_correctness_manager = ExecutionCorrectnessManager::new(node_config);

    let state_computer: Arc<dyn StateComputer> = if node_config.consensus.decoupled {
        let guage_e = IntGauge::new("D_EXE_CHANNEL_COUNTER", "counter for the decoupling execution channel").unwrap();
        let guage_c = IntGauge::new("D_COM_CHANNEL_COUNTER", "counter for the decoupling committing channel").unwrap();
        let (sender_exec, receiver_exec) = channel::new::<(Vec<Block>, LedgerInfoWithSignatures)>(
            node_config.consensus.channel_size,
            &guage_e
        );
        let (sender_comm, receiver_comm) = channel::new::<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>(
            node_config.consensus.channel_size,
            &guage_c
        );

        let execution_phase = ExecutionPhase::new(
            receiver_exec,
            execution_correctness_manager.client(),
            sender_comm,
        );
        let commit_phase = CommitPhase::new(
            receiver_comm,
            execution_correctness_manager.client(),
            state_sync_client,
        );

        runtime.spawn(commit_phase.start());
        runtime.spawn(execution_phase.start());

        Arc::new(OrderingStateComputer::new(
            sender_exec,
        ))
    } else {
        Arc::new(ExecutionProxy::new(
            execution_correctness_manager.client(),
            state_sync_client,
        ))
    };

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
        reconfig_events,
    );

    let (network_task, network_receiver) = NetworkTask::new(network_events, self_receiver);

    runtime.spawn(network_task.start());
    runtime.spawn(epoch_mgr.start(timeout_receiver, network_receiver));



    debug!("Consensus started.");
    runtime
}
