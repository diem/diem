// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    coordinator::SyncCoordinator,
    counters,
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSynchronizerEvents, StateSynchronizerSender},
};
use anyhow::{format_err, Result};
use diem_config::{
    config::{NodeConfig, RoleType, StateSyncConfig, UpstreamConfig},
    network_id::NodeNetworkId,
};
use diem_mempool::CommitResponse;
use diem_types::{
    contract_event::ContractEvent, epoch_change::Verifier, epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures, transaction::Transaction, waypoint::Waypoint,
};
use executor_types::{ChunkExecutor, ExecutedTrees};
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};
use std::{
    boxed::Box,
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::{
    runtime::{Builder, Runtime},
    time::timeout,
};

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

/// A synchronization request to sync to a specified target ledger info.
pub struct SyncRequest {
    pub callback: oneshot::Sender<Result<()>>,
    pub target: LedgerInfoWithSignatures,
    pub last_progress_tst: SystemTime,
}

/// A commit notification to notify state sync of new commits.
pub struct CommitNotification {
    pub callback: oneshot::Sender<Result<CommitResponse>>,
    pub committed_transactions: Vec<Transaction>,
    pub reconfiguration_events: Vec<ContractEvent>,
}

/// Messages used by the StateSyncClient for communication with the SyncCoordinator.
pub enum CoordinatorMessage {
    SyncRequest(Box<SyncRequest>), // Initiate a new sync request for a given target.
    CommitNotification(Box<CommitNotification>), // Notify state sync about committed transactions.
    GetSyncState(oneshot::Sender<SynchronizationState>), // Return the local synchronization state.
    WaitForInitialization(oneshot::Sender<Result<()>>), // Wait until state sync is initialized to the waypoint.
}

/// A client used for communicating with a SyncCoordinator.
pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncClient {
    /// Timeout for the StateSyncClient to receive an ack when executing commit().
    const COMMIT_TIMEOUT_SECS: u64 = 5;

    pub fn new(coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>) -> Self {
        Self { coordinator_sender }
    }

    /// Sync node's state to target ledger info (LI).
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    pub fn sync_to(&self, target: LedgerInfoWithSignatures) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        let request = SyncRequest {
            callback: cb_sender,
            target,
            last_progress_tst: SystemTime::now(),
        };

        async move {
            sender
                .send(CoordinatorMessage::SyncRequest(Box::new(request)))
                .await?;
            cb_receiver.await?
        }
    }

    /// Notifies state sync about newly committed transactions.
    pub fn commit(
        &self,
        committed_txns: Vec<Transaction>,
        reconfig_events: Vec<ContractEvent>,
    ) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        let notification = CommitNotification {
            callback: cb_sender,
            committed_transactions: committed_txns,
            reconfiguration_events: reconfig_events,
        };

        async move {
            sender
                .send(CoordinatorMessage::CommitNotification(Box::new(
                    notification,
                )))
                .await?;

            match timeout(
                Duration::from_secs(StateSyncClient::COMMIT_TIMEOUT_SECS),
                cb_receiver,
            )
            .await
            {
                Err(_) => {
                    counters::COMMIT_FLOW_FAIL
                        .with_label_values(&[counters::STATE_SYNC_LABEL])
                        .inc();
                    Err(format_err!(
                        "[State Sync Client] Timeout: failed to receive commit() ack in time!"
                    ))
                }
                // TODO(joshlind): clean up the use of CommitResponse.. having a string.is_empty()
                // to check the presence of an error isn't great :(
                Ok(response) => {
                    let CommitResponse { msg } = response??;
                    if msg != "" {
                        Err(format_err!(
                            "[State Sync Client] Failed: commit() returned an error: {:?}",
                            msg
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    /// Returns information about the state sync internal state. This should only
    /// be used by tests.
    #[cfg(test)]
    pub fn get_state(&self) -> impl Future<Output = Result<SynchronizationState>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::GetSyncState(cb_sender))
                .await?;
            cb_receiver.await.map_err(|error| error.into())
        }
    }

    /// Waits until state sync is caught up with the waypoint specified in the local config.
    pub fn wait_until_initialized(&self) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::WaitForInitialization(cb_sender))
                .await?;
            cb_receiver.await?
        }
    }
}
