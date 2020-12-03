// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    connectivity_manager::{ConnectivityManager, ConnectivityRequest},
    counters,
    peer_manager::{conn_notifs_channel, ConnectionRequestSender},
};
use diem_config::network_id::NetworkContext;
use diem_crypto::x25519;
use diem_infallible::RwLock;
use diem_network_address::NetworkAddress;
use diem_types::PeerId;
use futures::stream::StreamExt;
use futures_util::stream::Fuse;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
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
    eligible: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
    seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
    connectivity_check_interval_ms: u64,
    backoff_base: u64,
    max_connection_delay_ms: u64,
    connection_reqs_tx: ConnectionRequestSender,
    connection_notifs_rx: conn_notifs_channel::Receiver,
    requests_rx: channel::Receiver<ConnectivityRequest>,
    outbound_connection_limit: Option<usize>,
}

#[derive(Debug, PartialEq, PartialOrd)]
enum State {
    CREATED,
    BUILT,
    STARTED,
}

pub struct ConnectivityManagerBuilder {
    config: Option<ConnectivityManagerBuilderConfig>,
    connectivity_manager: Option<ConnectivityManagerService>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    state: State,
}

impl ConnectivityManagerBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        eligible: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
        seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
        seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
        connectivity_check_interval_ms: u64,
        backoff_base: u64,
        max_connection_delay_ms: u64,
        channel_size: usize,
        connection_reqs_tx: ConnectionRequestSender,
        connection_notifs_rx: conn_notifs_channel::Receiver,
        outbound_connection_limit: Option<usize>,
    ) -> Self {
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
            channel_size,
            &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
        );
        Self {
            config: Some(ConnectivityManagerBuilderConfig {
                network_context,
                eligible,
                seed_addrs,
                seed_pubkeys,
                connectivity_check_interval_ms,
                backoff_base,
                max_connection_delay_ms,
                connection_reqs_tx,
                connection_notifs_rx,
                requests_rx: conn_mgr_reqs_rx,
                outbound_connection_limit,
            }),
            connectivity_manager: None,
            conn_mgr_reqs_tx,
            state: State::CREATED,
        }
    }

    pub fn conn_mgr_reqs_tx(&self) -> channel::Sender<ConnectivityRequest> {
        self.conn_mgr_reqs_tx.clone()
    }

    pub fn build(&mut self, executor: &Handle) {
        assert_eq!(self.state, State::CREATED);
        self.state = State::BUILT;
        let config = self
            .config
            .take()
            .expect("Config must exist in order to build");

        self.connectivity_manager = Some({
            executor.enter(|| {
                ConnectivityManager::new(
                    config.network_context,
                    config.eligible,
                    config.seed_addrs,
                    config.seed_pubkeys,
                    interval(Duration::from_millis(config.connectivity_check_interval_ms)).fuse(),
                    config.connection_reqs_tx,
                    config.connection_notifs_rx,
                    config.requests_rx,
                    ExponentialBackoff::from_millis(config.backoff_base).factor(1000),
                    config.max_connection_delay_ms,
                    config.outbound_connection_limit,
                )
            })
        });
    }

    pub fn start(&mut self, executor: &Handle) {
        assert_eq!(self.state, State::BUILT);
        self.state = State::STARTED;
        let conn_mgr = self
            .connectivity_manager
            .take()
            .expect("Service Must be present");
        executor.spawn(conn_mgr.start());
    }
}
