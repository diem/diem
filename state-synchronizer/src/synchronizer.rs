// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    coordinator::{CoordinatorMessage, EpochRetrievalRequest, SyncCoordinator, SyncRequest},
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSynchronizerEvents, StateSynchronizerSender},
    SynchronizerState,
};
use anyhow::{format_err, Result};
use executor::Executor;
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};
use libra_config::config::{NodeConfig, RoleType, StateSyncConfig};
use libra_mempool::{CommitNotification, CommitResponse};
use libra_types::{
    contract_event::ContractEvent, event_subscription::EventSubscription,
    ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
    validator_change::ValidatorChangeProof, waypoint::Waypoint,
};
use libra_vm::LibraVM;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    runtime::{Builder, Runtime},
    time::timeout,
};

pub struct StateSynchronizer {
    _runtime: Runtime,
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn bootstrap(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
        executor: Arc<Mutex<Executor<LibraVM>>>,
        config: &NodeConfig,
        reconfig_event_subscriptions: Vec<Box<dyn EventSubscription>>,
    ) -> Self {
        let executor_proxy = ExecutorProxy::new(executor, config);
        Self::bootstrap_with_executor_proxy(
            network,
            state_sync_to_mempool_sender,
            config.base.role,
            config.base.waypoint,
            &config.state_sync,
            executor_proxy,
            reconfig_event_subscriptions,
        )
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
        role: RoleType,
        waypoint: Option<Waypoint>,
        state_sync_config: &StateSyncConfig,
        executor_proxy: E,
        reconfig_event_subscriptions: Vec<Box<dyn EventSubscription>>,
    ) -> Self {
        let mut runtime = Builder::new()
            .thread_name("state-sync-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[state synchronizer] failed to create runtime");

        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();

        let initial_state = runtime
            .block_on(executor_proxy.get_local_storage_state())
            .expect("[state sync] Start failure: cannot sync with storage.");
        let coordinator = SyncCoordinator::new(
            coordinator_receiver,
            state_sync_to_mempool_sender,
            role,
            waypoint,
            state_sync_config.clone(),
            executor_proxy,
            initial_state,
            reconfig_event_subscriptions,
        );
        runtime.spawn(coordinator.start(network));

        Self {
            _runtime: runtime,
            coordinator_sender,
        }
    }

    pub fn create_client(&self) -> Arc<StateSyncClient> {
        Arc::new(StateSyncClient {
            coordinator_sender: self.coordinator_sender.clone(),
        })
    }

    /// The function returns a future that is fulfilled when the state synchronizer is
    /// caught up with the waypoint specified in the local config.
    pub async fn wait_until_initialized(&self) -> Result<()> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        sender
            .send(CoordinatorMessage::WaitInitialize(cb_sender))
            .await?;
        cb_receiver.await?
    }
}

pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncClient {
    /// Sync validator's state to target.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    /// It is up to state synchronizer to decide about the specific criteria for the failure
    /// (e.g., lack of progress with all of the peer validators).
    pub fn sync_to(&self, target: LedgerInfoWithSignatures) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        let (callback, cb_receiver) = oneshot::channel();
        let request = SyncRequest { callback, target };
        async move {
            sender.send(CoordinatorMessage::Request(request)).await?;
            cb_receiver.await?
        }
    }

    /// Notifies state synchronizer about new version
    pub fn commit(
        &self,
        // *successfully* committed transactions
        committed_txns: Vec<Transaction>,
        reconfig_events: Vec<ContractEvent>,
    ) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        async move {
            let (callback, callback_rcv) = oneshot::channel();
            sender
                .send(CoordinatorMessage::Commit(
                    committed_txns,
                    reconfig_events,
                    callback,
                ))
                .await?;

            match timeout(Duration::from_secs(1), callback_rcv).await {
                Err(_) => {
                    Err(format_err!("[state sync client] failed to receive commit ACK from state synchronizer on time"))
                }
                Ok(resp) => {
                    let CommitResponse { msg } = resp??;
                    if msg != "" {
                        Err(format_err!("[state sync client] commit failed: {:?}", msg))
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    /// Returns information about StateSynchronizer internal state
    pub fn get_state(&self) -> impl Future<Output = Result<SynchronizerState>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender.send(CoordinatorMessage::GetState(cb_sender)).await?;
            let info = cb_receiver.await?;
            Ok(info)
        }
    }

    pub fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> impl Future<Output = Result<ValidatorChangeProof>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        let request = EpochRetrievalRequest {
            start_epoch,
            end_epoch,
            callback: cb_sender,
        };
        async move {
            sender
                .send(CoordinatorMessage::GetEpochProof(request))
                .await?;
            cb_receiver.await?
        }
    }
}
