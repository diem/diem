// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    connectivity_manager::{ConnectivityManager, ConnectivityRequest},
    counters,
    peer_manager::{conn_notifs_channel, ConnectionRequestSender},
};
use diem_config::{config::PeerSet, network_id::NetworkContext};
use diem_infallible::RwLock;
use diem_time_service::TimeService;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Handle;
use tokio_retry::strategy::ExponentialBackoff;

pub type ConnectivityManagerService = ConnectivityManager<ExponentialBackoff>;

pub struct ConnectivityManagerBuilder {
    connectivity_manager: Option<ConnectivityManagerService>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

impl ConnectivityManagerBuilder {
    pub fn create(
        network_context: Arc<NetworkContext>,
        time_service: TimeService,
        eligible: Arc<RwLock<PeerSet>>,
        seeds: PeerSet,
        connectivity_check_interval_ms: u64,
        backoff_base: u64,
        max_connection_delay_ms: u64,
        channel_size: usize,
        connection_reqs_tx: ConnectionRequestSender,
        connection_notifs_rx: conn_notifs_channel::Receiver,
        outbound_connection_limit: Option<usize>,
        mutual_authentication: bool,
    ) -> Self {
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
            channel_size,
            &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
        );

        Self {
            conn_mgr_reqs_tx,
            connectivity_manager: Some(ConnectivityManager::new(
                network_context,
                time_service,
                eligible,
                seeds,
                connection_reqs_tx,
                connection_notifs_rx,
                conn_mgr_reqs_rx,
                Duration::from_millis(connectivity_check_interval_ms),
                ExponentialBackoff::from_millis(backoff_base).factor(1000),
                Duration::from_millis(max_connection_delay_ms),
                outbound_connection_limit,
                mutual_authentication,
            )),
        }
    }

    pub fn conn_mgr_reqs_tx(&self) -> channel::Sender<ConnectivityRequest> {
        self.conn_mgr_reqs_tx.clone()
    }

    pub fn start(&mut self, executor: &Handle) {
        let conn_mgr = self
            .connectivity_manager
            .take()
            .expect("Service Must be present");
        executor.spawn(conn_mgr.start());
    }
}
