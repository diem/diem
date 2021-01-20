// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    client::{CoordinatorMessage, StateSyncClient},
    coordinator::SyncCoordinator,
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSynchronizerEvents, StateSynchronizerSender},
};
use anyhow::Result;
use diem_config::{
    config::{NodeConfig, RoleType, StateSyncConfig, UpstreamConfig},
    network_id::NodeNetworkId,
};
use diem_types::{
    epoch_change::Verifier, epoch_state::EpochState, ledger_info::LedgerInfoWithSignatures,
    waypoint::Waypoint,
};
use executor_types::{ChunkExecutor, ExecutedTrees};
use futures::channel::mpsc;
use std::{boxed::Box, collections::HashMap, sync::Arc};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::runtime::{Builder, Runtime};

/// SynchronizationState contains the following fields:
/// * `committed_ledger_info` holds the latest certified ledger info (committed to storage),
///    i.e., the ledger info for the highest version for which storage has all ledger state.
/// * `synced_trees` holds the latest transaction accumulator and state tree (which may
///    or may not be committed to storage), i.e., some ledger state for the next highest
///    ledger info version is missing.
/// * `trusted_epoch_state` corresponds to the current epoch if the highest committed
///    ledger info (`committed_ledger_info`) is in the middle of the epoch, otherwise, it
///    corresponds to the next epoch if the highest committed ledger info ends the epoch.
///
/// Note: `committed_ledger_info` is used for helping other Diem nodes synchronize (i.e.,
/// it corresponds to the highest version we have a proof for in storage). `synced_trees`
/// is used locally for retrieving missing chunks for the local storage.
#[derive(Clone)]
pub struct SynchronizationState {
    committed_ledger_info: LedgerInfoWithSignatures,
    synced_trees: ExecutedTrees,
    trusted_epoch_state: EpochState,
}

impl SynchronizationState {
    pub fn new(
        committed_ledger_info: LedgerInfoWithSignatures,
        synced_trees: ExecutedTrees,
        current_epoch_state: EpochState,
    ) -> Self {
        let trusted_epoch_state = committed_ledger_info
            .ledger_info()
            .next_epoch_state()
            .cloned()
            .unwrap_or(current_epoch_state);

        SynchronizationState {
            committed_ledger_info,
            synced_trees,
            trusted_epoch_state,
        }
    }

    pub fn committed_epoch(&self) -> u64 {
        self.committed_ledger_info.ledger_info().epoch()
    }

    pub fn committed_ledger_info(&self) -> LedgerInfoWithSignatures {
        self.committed_ledger_info.clone()
    }

    pub fn committed_version(&self) -> u64 {
        self.committed_ledger_info.ledger_info().version()
    }

    /// Returns the highest available version in the local storage, even if it's not
    /// committed (i.e., covered by a ledger info).
    pub fn synced_version(&self) -> u64 {
        self.synced_trees.version().unwrap_or(0)
    }

    pub fn trusted_epoch(&self) -> u64 {
        self.trusted_epoch_state.epoch
    }

    pub fn verify_ledger_info(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()> {
        self.trusted_epoch_state.verify(ledger_info)
    }
}

/// Creates and bootstraps new state sync runtimes and creates clients for
/// communicating with those state sync runtimes.
pub struct StateSyncBootstrapper {
    _runtime: Runtime,
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncBootstrapper {
    pub fn bootstrap(
        network: Vec<(
            NodeNetworkId,
            StateSynchronizerSender,
            StateSynchronizerEvents,
        )>,
        state_sync_to_mempool_sender: mpsc::Sender<diem_mempool::CommitNotification>,
        storage: Arc<dyn DbReader>,
        executor: Box<dyn ChunkExecutor>,
        config: &NodeConfig,
        waypoint: Waypoint,
        reconfig_event_subscriptions: Vec<ReconfigSubscription>,
    ) -> Self {
        let runtime = Builder::new_multi_thread()
            .thread_name("state-sync")
            .enable_all()
            .build()
            .expect("[State Sync] Failed to create runtime!");

        let executor_proxy = ExecutorProxy::new(storage, executor, reconfig_event_subscriptions);
        Self::bootstrap_with_executor_proxy(
            runtime,
            network,
            state_sync_to_mempool_sender,
            config.base.role,
            waypoint,
            &config.state_sync,
            config.upstream.clone(),
            executor_proxy,
        )
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        runtime: Runtime,
        network: Vec<(
            NodeNetworkId,
            StateSynchronizerSender,
            StateSynchronizerEvents,
        )>,
        state_sync_to_mempool_sender: mpsc::Sender<diem_mempool::CommitNotification>,
        role: RoleType,
        waypoint: Waypoint,
        state_sync_config: &StateSyncConfig,
        upstream_config: UpstreamConfig,
        executor_proxy: E,
    ) -> Self {
        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();
        let initial_state = executor_proxy
            .get_local_storage_state()
            .expect("[State Sync] Starting failure: cannot sync with storage!");
        let network_senders: HashMap<_, _> = network
            .iter()
            .map(|(network_id, sender, _events)| (network_id.clone(), sender.clone()))
            .collect();

        let coordinator = SyncCoordinator::new(
            coordinator_receiver,
            state_sync_to_mempool_sender,
            network_senders,
            role,
            waypoint,
            state_sync_config.clone(),
            upstream_config,
            executor_proxy,
            initial_state,
        )
        .expect("[State Sync] Unable to create state sync coordinator!");
        runtime.spawn(coordinator.start(network));

        Self {
            _runtime: runtime,
            coordinator_sender,
        }
    }

    pub fn create_client(&self) -> StateSyncClient {
        StateSyncClient::new(self.coordinator_sender.clone())
    }
}
