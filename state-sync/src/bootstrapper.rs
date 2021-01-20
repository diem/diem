// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    client::{CoordinatorMessage, StateSyncClient},
    coordinator::SyncCoordinator,
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
    network::{StateSynchronizerEvents, StateSynchronizerSender},
};
use diem_config::{
    config::{NodeConfig, RoleType, StateSyncConfig, UpstreamConfig},
    network_id::NodeNetworkId,
};
use diem_types::waypoint::Waypoint;
use executor_types::ChunkExecutor;
use futures::channel::mpsc;
use std::{boxed::Box, collections::HashMap, sync::Arc};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::runtime::{Builder, Runtime};

/// Creates and bootstraps new state syncs and creates clients for
/// communicating with those state syncs.
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
