// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::libra_channel::{self, Receiver};
use futures::{sink::SinkExt, StreamExt};
use libra_config::config::RoleType;
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::{register_histogram, DurationHistogram};
use libra_network_address::NetworkAddress;
use libra_network_address_encryption::{Encryptor, Error as EncryptorError};
use libra_types::on_chain_config::{OnChainConfigPayload, ValidatorSet, ON_CHAIN_CONFIG_REGISTRY};
use network::connectivity_manager::{ConnectivityRequest, DiscoverySource};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};
use subscription_service::ReconfigSubscription;

pub mod builder;

/// Histogram of idle time of spent in event processing loop
pub static EVENT_PROCESSING_LOOP_IDLE_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "simple_onchain_discovery_event_processing_loop_idle_duration_s",
            "Histogram of idle time of spent in event processing loop"
        )
        .unwrap(),
    )
});

/// Histogram of busy time of spent in event processing loop
pub static EVENT_PROCESSING_LOOP_BUSY_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "simple_onchain_discovery_event_processing_loop_busy_duration_s",
            "Histogram of busy time of spent in event processing loop"
        )
        .unwrap(),
    )
});

/// Listener which converts published  updates from the OnChainConfig to ConnectivityRequests
/// for the ConnectivityManager.
pub struct ConfigurationChangeListener {
    role: RoleType,
    encryptor: Encryptor,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
}

pub fn gen_simple_discovery_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all(ON_CHAIN_CONFIG_REGISTRY.to_vec(), vec![])
}

/// Extracts a set of ConnectivityRequests from a ValidatorSet which are appropriate for a network with type role.
fn extract_updates(
    role: RoleType,
    encryptor: &Encryptor,
    node_set: ValidatorSet,
) -> Vec<ConnectivityRequest> {
    // Decode addresses while ignoring bad addresses
    let new_peer_addrs: HashMap<_, _> = node_set
        .into_iter()
        .map(|info| {
            let peer_id = *info.account_address();
            let config = info.into_config();

            let addrs_res = match role {
                RoleType::Validator => {
                    let result = encryptor.decrypt(&config.validator_network_addresses, peer_id);
                    if let Err(EncryptorError::StorageError(_)) = result {
                        panic!(format!(
                            "Unable to initialize validator network addresses: {:?}",
                            result
                        ));
                    }
                    result.map_err(anyhow::Error::from)
                }
                RoleType::FullNode => config
                    .fullnode_network_addresses()
                    .map_err(anyhow::Error::from),
            };

            let addrs = match addrs_res {
                Ok(addrs) => addrs,
                Err(err) => {
                    warn!(
                        "Failed to parse any network address: peer: {}, err: {}",
                        peer_id, err
                    );
                    Vec::new()
                }
            };

            (peer_id, addrs)
        })
        .collect();

    // Retrieve public keys from addresses
    let new_peer_pubkeys: HashMap<_, _> = new_peer_addrs
        .iter()
        .map(|(peer_id, addrs)| {
            let pubkeys: HashSet<x25519::PublicKey> = addrs
                .iter()
                .filter_map(NetworkAddress::find_noise_proto)
                .collect();
            (*peer_id, pubkeys)
        })
        .collect();

    vec![
        ConnectivityRequest::UpdateAddresses(DiscoverySource::OnChain, new_peer_addrs),
        ConnectivityRequest::UpdateEligibleNodes(DiscoverySource::OnChain, new_peer_pubkeys),
    ]
}

impl ConfigurationChangeListener {
    /// Creates a new ConfigurationListener
    pub fn new(
        role: RoleType,
        encryptor: Encryptor,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        Self {
            role,
            encryptor,
            conn_mgr_reqs_tx,
            reconfig_events,
        }
    }

    /// Processes a received OnChainConfigPayload.  Depending on role (Validator or FullNode), parses
    /// the appropriate configuration changes and passes it to the ConnectionManager channel.
    async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let node_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let updates = extract_updates(self.role, &self.encryptor, node_set);

        info!("Update {} Network about new Node IDs", self.role);

        for update in updates {
            match self.conn_mgr_reqs_tx.send(update).await {
                Ok(()) => (),
                Err(e) => warn!("Failed to send update to ConnectivityManager {}", e),
            }
        }
    }

    /// Starts the listener to wait on reconfiguration events.  Creates an infinite loop.
    pub async fn start(mut self) {
        loop {
            let start_idle_time = Instant::now();
            let payload = self.reconfig_events.select_next_some().await;
            let idle_duration = start_idle_time.elapsed();
            let start_process_time = Instant::now();
            self.process_payload(payload).await;
            let process_duration = start_process_time.elapsed();

            EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
            EVENT_PROCESSING_LOOP_BUSY_DURATION_S.observe_duration(process_duration);
        }
    }
}
