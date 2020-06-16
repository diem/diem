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

pub struct OnchainDiscoveryBuilderConfig {
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    network_tx: OnchainDiscoveryNetworkSender,
    discovery_events: OnchainDiscoveryNetworkEvents,
    network_context: Arc<NetworkContext>,
    libra_db: Arc<dyn DbReader>,
    waypoint: Waypoint,
    outbound_rpc_timeout_secs: u64,
    max_concurrent_inbound_queries: usize,
}

impl OnchainDiscoveryBuilderConfig {
    pub fn new(
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        network_tx: OnchainDiscoveryNetworkSender,
        discovery_events: OnchainDiscoveryNetworkEvents,
        network_context: Arc<NetworkContext>,
        libra_db: Arc<dyn DbReader>,
        waypoint: Waypoint,
        outbound_rpc_timeout_secs: u64,
        max_concurrent_inbound_queries: usize,
    ) -> Self {
        Self {
            conn_mgr_reqs_tx,
            network_tx,
            discovery_events,
            network_context,
            libra_db,
            waypoint,
            outbound_rpc_timeout_secs,
            max_concurrent_inbound_queries,
        }
    }
}

impl OnchainDiscoveryBuilder {
    /// Setup OnchainDiscovery to work with the provided tx and rx channels.  Returns a tuple
    /// (OnChainDiscoveryService, OnChainDiscovery) which must be started by the caller.
    pub fn build(config: OnchainDiscoveryBuilderConfig, executor: &Handle) -> Self {
        let (peer_mgr_notifs_rx, conn_notifs_rx) = (
            config.discovery_events.peer_mgr_notifs_rx,
            config.discovery_events.connection_notifs_rx,
        );

        let onchain_discovery_service = OnchainDiscoveryService::new(
            executor.clone(),
            peer_mgr_notifs_rx,
            Arc::clone(&config.libra_db),
            config.max_concurrent_inbound_queries,
        );

        let network_context = config.network_context;
        let waypoint = config.waypoint;
        let network_tx = config.network_tx;
        let conn_mgr_reqs_tx = config.conn_mgr_reqs_tx;
        let libra_db = config.libra_db;
        let ticker_secs = config.outbound_rpc_timeout_secs;

        let onchain_discovery = executor.enter(move || {
            let peer_query_ticker = interval(Duration::from_secs(ticker_secs)).fuse();
            let storage_query_ticker = interval(Duration::from_secs(ticker_secs)).fuse();
            let outbound_rpc_timeout = Duration::from_secs(ticker_secs);

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
