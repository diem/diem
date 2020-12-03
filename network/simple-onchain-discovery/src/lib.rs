// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::diem_channel::{self, Receiver};
use diem_config::{config::RoleType, network_id::NetworkContext};
use diem_crypto::x25519;
use diem_logger::prelude::*;
use diem_metrics::{
    register_histogram, register_int_counter_vec, DurationHistogram, IntCounterVec,
};
use diem_network_address::NetworkAddress;
use diem_network_address_encryption::{Encryptor, Error as EncryptorError};
use diem_types::on_chain_config::{OnChainConfigPayload, ValidatorSet, ON_CHAIN_CONFIG_REGISTRY};
use futures::{sink::SinkExt, StreamExt};
use network::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    counters::inc_by_with_context,
    logging::NetworkSchema,
};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
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

pub static DISCOVERY_COUNTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_simple_onchain_discovery_counts",
        "Histogram of busy time of spent in event processing loop",
        &["role_type", "network_id", "peer_id", "metric"]
    )
    .unwrap()
});

/// Listener which converts published  updates from the OnChainConfig to ConnectivityRequests
/// for the ConnectivityManager.
pub struct ConfigurationChangeListener {
    network_context: Arc<NetworkContext>,
    encryptor: Encryptor,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
}

pub fn gen_simple_discovery_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all("network", ON_CHAIN_CONFIG_REGISTRY.to_vec(), vec![])
}

/// Extracts a set of ConnectivityRequests from a ValidatorSet which are appropriate for a network with type role.
fn extract_updates(
    network_context: Arc<NetworkContext>,
    encryptor: &Encryptor,
    node_set: ValidatorSet,
) -> Vec<ConnectivityRequest> {
    let role = network_context.role();

    // Decode addresses while ignoring bad addresses
    let new_peer_addrs: HashMap<_, _> = node_set
        .into_iter()
        .map(|info| {
            let peer_id = *info.account_address();
            let config = info.into_config();

            let addrs = match role {
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
            }
            .map_err(|err| {
                inc_by_with_context(&DISCOVERY_COUNTS, &network_context, "read_failure", 1);

                warn!(
                    NetworkSchema::new(&network_context),
                    "OnChainDiscovery: Failed to parse any network address: peer: {}, err: {}",
                    peer_id,
                    err
                )
            })
            .unwrap_or_default();

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
        network_context: Arc<NetworkContext>,
        encryptor: Encryptor,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        Self {
            network_context,
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

        let updates = extract_updates(self.network_context.clone(), &self.encryptor, node_set);

        inc_by_with_context(
            &DISCOVERY_COUNTS,
            &self.network_context,
            "new_nodes",
            updates.len() as u64,
        );
        info!(
            NetworkSchema::new(&self.network_context),
            "Update {} Network about new Node IDs",
            self.network_context.role()
        );

        for update in updates {
            match self.conn_mgr_reqs_tx.send(update).await {
                Ok(()) => (),
                Err(e) => {
                    inc_by_with_context(
                        &DISCOVERY_COUNTS,
                        &self.network_context,
                        "send_failure",
                        1,
                    );
                    warn!(
                        NetworkSchema::new(&self.network_context),
                        "Failed to send update to ConnectivityManager {}", e
                    )
                }
            }
        }
    }

    /// Starts the listener to wait on reconfiguration events.  Creates an infinite loop.
    pub async fn start(mut self) {
        info!(
            NetworkSchema::new(&self.network_context),
            "{} Starting OnChain Discovery actor", self.network_context
        );
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
