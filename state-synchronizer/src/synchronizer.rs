// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    coordinator::SyncCoordinator,
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSynchronizerEvents, StateSynchronizerSender},
    state_sync_client::CoordinatorMessage,
    state_sync_client::StateSyncClient,
};
use anyhow::Result;
use executor_types::ChunkExecutor;
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use libra_config::config::{NodeConfig, RoleType, StateSyncConfig, UpstreamConfig};
use libra_mempool::CommitNotification;
use libra_types::{waypoint::Waypoint, PeerId};
use std::{boxed::Box, collections::HashMap, sync::Arc};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::runtime::{Builder, Runtime};

/// StateSynchronizer is in charge of taking requests from network clients (other full nodes),
/// as well as internal clients (consensus)
pub struct StateSynchronizer {
    _runtime: Runtime,
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn bootstrap(
        network: Vec<(PeerId, StateSynchronizerSender, StateSynchronizerEvents)>,
        state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
        storage: Arc<dyn DbReader>,
        executor: Box<dyn ChunkExecutor>,
        config: &NodeConfig,
        waypoint: Waypoint,
        reconfig_event_subscriptions: Vec<ReconfigSubscription>,
    ) -> Self {
        let runtime = Builder::new()
            .thread_name("state-sync-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[state synchronizer] failed to create runtime");

        let executor_proxy = ExecutorProxy::new(storage, executor, reconfig_event_subscriptions);
        Self::bootstrap_with_executor_proxy(
            runtime,
            network,
            state_sync_to_mempool_sender,
            config.base.role,
            Some(waypoint),
            &config.state_sync,
            config.upstream.clone(),
            executor_proxy,
        )
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        runtime: Runtime,
        network: Vec<(PeerId, StateSynchronizerSender, StateSynchronizerEvents)>,
        state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
        role: RoleType,
        waypoint: Option<Waypoint>,
        state_sync_config: &StateSyncConfig,
        upstream_config: UpstreamConfig,
        executor_proxy: E,
    ) -> Self {
        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();

        let initial_state = executor_proxy
            .get_local_storage_state()
            .expect("[state sync] Start failure: cannot sync with storage.");

        let network_senders: HashMap<_, _> = network
            .iter()
            .map(|(network_id, sender, _events)| (*network_id, sender.clone()))
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
        );
        runtime.spawn(coordinator.start(network));

        Self {
            _runtime: runtime,
            coordinator_sender,
        }
    }

    pub fn create_client(&self) -> Arc<StateSyncClient> {
        Arc::new(StateSyncClient::new(self.coordinator_sender.clone()))
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
