// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::OnchainDiscovery,
    network_interface::{OnchainDiscoveryNetworkEvents, OnchainDiscoveryNetworkSender},
    service::OnchainDiscoveryService,
};
use futures::stream::{Fuse, StreamExt};
use libra_config::network_id::NetworkContext;
use libra_types::waypoint::Waypoint;
use network::connectivity_manager::ConnectivityRequest;
use std::{sync::Arc, time::Duration};
use storage_interface::DbReader;
use tokio::{
    runtime::Handle,
    time::{interval, Interval},
};

pub struct OnchainDiscoveryBuilder {
    onchain_discovery_service: OnchainDiscoveryService,
    onchain_discovery: OnchainDiscovery<Fuse<Interval>>,
    started: bool,
}

impl OnchainDiscoveryBuilder {
    /// Setup OnchainDiscovery to work with the provided tx and rx channels.  Returns a tuple
    /// (OnChainDiscoveryService, OnChainDiscovery) which must be started by the caller.
    pub fn build(
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        network_tx: OnchainDiscoveryNetworkSender,
        discovery_events: OnchainDiscoveryNetworkEvents,
        network_context: Arc<NetworkContext>,
        libra_db: Arc<dyn DbReader>,
        waypoint: Waypoint,
        executor: &Handle,
    ) -> Self {
        let outbound_rpc_timeout = Duration::from_secs(30);
        let max_concurrent_inbound_queries = 8;
        let (peer_mgr_notifs_rx, conn_notifs_rx) = (
            discovery_events.peer_mgr_notifs_rx,
            discovery_events.connection_notifs_rx,
        );

        let onchain_discovery_service = OnchainDiscoveryService::new(
            executor.clone(),
            peer_mgr_notifs_rx,
            Arc::clone(&libra_db),
            max_concurrent_inbound_queries,
        );

        let onchain_discovery = executor.enter(move || {
            let peer_query_ticker = interval(Duration::from_secs(30)).fuse();
            let storage_query_ticker = interval(Duration::from_secs(30)).fuse();

            OnchainDiscovery::new(
                network_context,
                waypoint,
                network_tx,
                conn_mgr_reqs_tx,
                conn_notifs_rx,
                libra_db,
                peer_query_ticker,
                storage_query_ticker,
                outbound_rpc_timeout,
            )
        });
        Self {
            onchain_discovery_service,
            onchain_discovery,
            started: false,
        }
    }

    /// Starts the provided onchain_discovery_service and onchain_discovery.  Should be called at
    /// most once.
    pub fn start(mut self, executor: &Handle) {
        assert!(!self.started);
        self.started = true;
        executor.spawn(self.onchain_discovery_service.start());
        executor.spawn(self.onchain_discovery.start());
    }
}
