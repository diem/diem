// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    connectivity_manager::{ConnectivityManager, ConnectivityRequest},
    peer_manager::{conn_notifs_channel, ConnectionRequestSender},
};
use futures::stream::StreamExt;
use futures_util::stream::Fuse;
use libra_config::network_id::NetworkContext;
use libra_crypto::x25519;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::{
    runtime::Handle,
    time::{interval, Interval},
};
use tokio_retry::strategy::ExponentialBackoff;

pub type ConnectivityManagerService = ConnectivityManager<Fuse<Interval>, ExponentialBackoff>;

/// The configuration fields for ConnectivityManager
struct ConnectivityManagerBuilderConfig {
    network_context: Arc<NetworkContext>,
    eligible: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
    seed_peers: HashMap<PeerId, Vec<NetworkAddress>>,
    connectivity_check_interval_ms: u64,
    backoff_base: u64,
    max_connection_delay_ms: u64,
    connection_reqs_tx: ConnectionRequestSender,
    connection_notifs_rx: conn_notifs_channel::Receiver,
    requests_rx: channel::Receiver<ConnectivityRequest>,
    connection_limit: Option<usize>,
}

pub struct ConnectivityManagerBuilder {
    config: Option<ConnectivityManagerBuilderConfig>,
    connectivity_manager: Option<ConnectivityManagerService>,
}

impl ConnectivityManagerBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        eligible: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
        seed_peers: HashMap<PeerId, Vec<NetworkAddress>>,
        connectivity_check_interval_ms: u64,
        backoff_base: u64,
        max_connection_delay_ms: u64,
        connection_reqs_tx: ConnectionRequestSender,
        connection_notifs_rx: conn_notifs_channel::Receiver,
        requests_rx: channel::Receiver<ConnectivityRequest>,
        connection_limit: Option<usize>,
    ) -> Self {
        Self {
            config: Some(ConnectivityManagerBuilderConfig {
                network_context,
                eligible,
                seed_peers,
                connectivity_check_interval_ms,
                backoff_base,
                max_connection_delay_ms,
                connection_reqs_tx,
                connection_notifs_rx,
                requests_rx,
                connection_limit,
            }),
            connectivity_manager: None,
        }
    }

    pub fn build(&mut self, executor: &Handle) {
        let config = self
            .config
            .take()
            .expect("Config must exist in order to build");

        self.connectivity_manager = Some({
            executor.enter(|| {
                ConnectivityManager::new(
                    config.network_context,
                    config.eligible,
                    config.seed_peers,
                    interval(Duration::from_millis(config.connectivity_check_interval_ms)).fuse(),
                    config.connection_reqs_tx,
                    config.connection_notifs_rx,
                    config.requests_rx,
                    ExponentialBackoff::from_millis(config.backoff_base).factor(1000),
                    config.max_connection_delay_ms,
                    config.connection_limit,
                )
            })
        });
    }

    pub fn start(&mut self, executor: &Handle) {
        if let Some(conn_mgr) = self.connectivity_manager.take() {
            executor.spawn(conn_mgr.start());
        }
    }
}
