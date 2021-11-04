// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![allow(dead_code)]

use consensus_notifications::ConsensusNotificationListener;
use diem_config::{config::NodeConfig, network_id::NetworkId};
use diem_types::{move_resource::MoveStorage, protocol_spec::DpnProto, waypoint::Waypoint};
use event_notifications::{EventNotificationSender, EventSubscriptionService};
use executor_types::ChunkExecutor;
use futures::executor::block_on;
use mempool_notifications::MempoolNotificationSender;
use network::protocols::network::AppConfig;
use state_sync_v1::{
    bootstrapper::StateSyncBootstrapper,
    network::{StateSyncEvents, StateSyncSender},
};
use std::sync::Arc;
use storage_interface::DbReader;

/// A multiplexer allowing multiple versions of state sync to operate
/// concurrently (i.e., state sync v1 and state sync v2).
pub struct StateSyncMultiplexer {
    state_sync_v1: StateSyncBootstrapper,
}

impl StateSyncMultiplexer {
    pub fn new<M: MempoolNotificationSender + 'static>(
        network: Vec<(NetworkId, StateSyncSender, StateSyncEvents)>,
        mempool_notifier: M,
        consensus_listener: ConsensusNotificationListener,
        storage: Arc<dyn DbReader<DpnProto>>,
        executor: Box<dyn ChunkExecutor>,
        node_config: &NodeConfig,
        waypoint: Waypoint,
        mut event_subscription_service: EventSubscriptionService,
    ) -> Self {
        // Notify subscribers of the initial on-chain config values
        match (&*storage).fetch_synced_version() {
            Ok(synced_version) => {
                if let Err(error) =
                    event_subscription_service.notify_initial_configs(synced_version)
                {
                    panic!(
                        "Failed to notify subscribers of initial on-chain configs: {:?}",
                        error
                    )
                }
            }
            Err(error) => panic!("Failed to fetch the initial synced version: {:?}", error),
        }

        // TODO(joshlind): use configs to determine the state sync v1 mode
        let state_sync_v1_read_only = false;

        // Create a state sync v1 bootstrapper
        let state_sync_bootstrapper = StateSyncBootstrapper::bootstrap(
            network,
            mempool_notifier,
            consensus_listener,
            storage,
            executor,
            node_config,
            waypoint,
            event_subscription_service,
            state_sync_v1_read_only,
        );

        Self {
            state_sync_v1: state_sync_bootstrapper,
        }
    }

    pub fn block_until_initialized(&self) {
        let state_sync_v1_client = self.state_sync_v1.create_client();
        block_on(state_sync_v1_client.wait_until_initialized())
            .expect("State sync v1 initialization failure");
    }
}

/// Configuration for the network endpoints to support state sync.
pub fn state_sync_v1_network_config() -> AppConfig {
    state_sync_v1::network::network_endpoint_config()
}
