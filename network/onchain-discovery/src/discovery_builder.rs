use crate::{
    client::OnchainDiscovery,
    service::OnchainDiscoveryService,
    types::{QueryDiscoverySetRequest, QueryDiscoverySetResponse},
};
use anyhow::{Context as AnyhowContext, Result};
use futures::{
    future::{Future, FutureExt},
    stream::{Fuse, StreamExt},
};
use libra_config::config::RoleType;
use libra_types::{account_config, waypoint::Waypoint, PeerId};
use network::validator_network::network_builder::NetworkBuilder;
use std::{sync::Arc, time::Duration};
use storage_interface::DbReader;
use tokio::{
    runtime::Handle,
    task,
    time::{interval, Interval},
};

pub struct OnchainDiscoveryBuilder {
    onchain_discovery_service: OnchainDiscoveryService,
    onchain_discovery: OnchainDiscovery<Fuse<Interval>>,
    started: bool,
}

impl OnchainDiscoveryBuilder {
    /// Setup OnchainDiscovery to work with the provided NetworkBuilder.  Returns a tuple
    /// (OnChainDiscoveryService, OnChainDiscovery) which must be started by the caller.
    pub fn build(
        network: &mut NetworkBuilder,
        peer_id: PeerId,
        role: RoleType,
        libra_db: Arc<dyn DbReader>,
        waypoint: Waypoint,
        executor: &Handle,
    ) -> Self {
        // TODO:  move this outside of the config_builder
        let (network_tx, peer_mgr_notifs_rx, conn_notifs_rx) =
            crate::network_interface::add_to_network(network);
        let outbound_rpc_timeout = Duration::from_secs(30);
        let max_concurrent_inbound_queries = 8;

        let onchain_discovery_service = OnchainDiscoveryService::new(
            // TODO:  Why are we passing a clone of the executor in?  This seems to imply that we are spawning an ever growing pile of executors in the system...
            executor.clone(),
            peer_mgr_notifs_rx,
            Arc::clone(&libra_db),
            max_concurrent_inbound_queries,
        );

        let onchain_discovery = executor.enter(move || {
            let peer_query_ticker = interval(Duration::from_secs(30)).fuse();
            let storage_query_ticker = interval(Duration::from_secs(30)).fuse();

            OnchainDiscovery::new(
                peer_id,
                role,
                waypoint,
                network_tx,
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

    /// Starts the provided onchain_discovery_service and onchain_discovery.
    pub fn start(self, executor: &Handle) {
        assert!(!self.started);
        executor.spawn(self.onchain_discovery_service.start());
        executor.spawn(self.onchain_discovery.start());
    }
}
