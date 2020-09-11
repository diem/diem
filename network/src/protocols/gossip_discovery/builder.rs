// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    connectivity_manager::ConnectivityRequest,
    logging::NetworkSchema,
    protocols::gossip_discovery::{
        GossipDiscovery, GossipDiscoveryNetworkEvents, GossipDiscoveryNetworkSender,
    },
};
use futures::stream::StreamExt;
use futures_util::stream::Fuse;
use libra_config::network_id::NetworkContext;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use std::{sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    time::{interval, Interval},
};

/// Configuration object which describes the production Discovery component.
struct GossipDiscoveryBuilderConfig {
    network_context: Arc<NetworkContext>,
    self_addrs: Vec<NetworkAddress>,
    discovery_interval_ms: u64,
    network_reqs_tx: GossipDiscoveryNetworkSender,
    network_notifs_rx: GossipDiscoveryNetworkEvents,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

impl GossipDiscoveryBuilderConfig {
    pub fn new(
        network_context: Arc<NetworkContext>,
        self_addrs: Vec<NetworkAddress>,
        discovery_interval_ms: u64,
        network_reqs_tx: GossipDiscoveryNetworkSender,
        network_notifs_rx: GossipDiscoveryNetworkEvents,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self {
            network_context,
            self_addrs,
            discovery_interval_ms,
            network_reqs_tx,
            network_notifs_rx,
            conn_mgr_reqs_tx,
        }
    }
}

type GossipDiscoveryService = GossipDiscovery<Fuse<Interval>>;

#[derive(Debug, PartialOrd, PartialEq)]
enum State {
    CREATED,
    BUILT,
    STARTED,
}

pub struct GossipDiscoveryBuilder {
    network_context: Arc<NetworkContext>,
    config: Option<GossipDiscoveryBuilderConfig>,
    discovery: Option<GossipDiscoveryService>,
    state: State,
}

impl GossipDiscoveryBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        self_addrs: Vec<NetworkAddress>,
        discovery_interval_ms: u64,
        network_reqs_tx: GossipDiscoveryNetworkSender,
        network_notifs_rx: GossipDiscoveryNetworkEvents,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        debug!(
            NetworkSchema::new(&network_context),
            "{} Created gossip discovery protocol actor (builder)", network_context
        );
        Self {
            network_context: network_context.clone(),
            config: Some(GossipDiscoveryBuilderConfig::new(
                network_context,
                self_addrs,
                discovery_interval_ms,
                network_reqs_tx,
                network_notifs_rx,
                conn_mgr_reqs_tx,
            )),
            discovery: None,
            state: State::CREATED,
        }
    }

    pub fn build(&mut self, executor: &Handle) -> &mut Self {
        assert_eq!(self.state, State::CREATED);
        self.state = State::BUILT;
        if let Some(config) = self.config.take() {
            self.discovery = Some(executor.enter(|| {
                GossipDiscovery::new(
                    config.network_context,
                    config.self_addrs,
                    interval(Duration::from_millis(config.discovery_interval_ms)).fuse(),
                    config.network_reqs_tx,
                    config.network_notifs_rx,
                    config.conn_mgr_reqs_tx,
                )
            }));
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Built gossip discovery protocol actor (builder)", self.network_context
            );
        };

        self
    }

    pub fn start(&mut self, executor: &Handle) -> &mut Self {
        assert_eq!(self.state, State::BUILT);
        self.state = State::STARTED;
        if let Some(discovery) = self.discovery.take() {
            executor.spawn(discovery.start());
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Started gossip discovery protocol actor (builder)", self.network_context
            );
        }
        self
    }
}
