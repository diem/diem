// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::coordinator::EpochRetrievalRequest;
use crate::{
    coordinator::{CoordinatorMessage, SyncCoordinator, SyncRequest},
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    SynchronizerState,
};
use executor::Executor;
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};
use libra_config::config::{NodeConfig, RoleType, StateSyncConfig};
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use libra_types::crypto_proxies::ValidatorChangeEventWithProof;
use network::validator_network::{StateSynchronizerEvents, StateSynchronizerSender};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use vm_runtime::MoveVM;

pub struct StateSynchronizer {
    _runtime: Runtime,
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn bootstrap(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        executor: Arc<Executor<MoveVM>>,
        config: &NodeConfig,
    ) -> Self {
        let executor_proxy = ExecutorProxy::new(executor, config);
        Self::bootstrap_with_executor_proxy(
            network,
            config.base.role,
            &config.state_sync,
            executor_proxy,
        )
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        role: RoleType,
        state_sync_config: &StateSyncConfig,
        executor_proxy: E,
    ) -> Self {
        let runtime = Builder::new()
            .thread_name("state-sync-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[state synchronizer] failed to create runtime");
        let executor = runtime.handle();

        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();

        let coordinator = SyncCoordinator::new(
            coordinator_receiver,
            role,
            state_sync_config.clone(),
            executor_proxy,
        );
        executor.spawn(coordinator.start(network));

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
    pub fn commit(&self) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        async move {
            sender.send(CoordinatorMessage::Commit).await?;
            Ok(())
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
    ) -> impl Future<Output = Result<ValidatorChangeEventWithProof>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        let request = EpochRetrievalRequest {
            start_epoch,
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
