// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::libra_channel::{self, Receiver};
use futures::{sink::SinkExt, StreamExt};
use libra_canonical_serialization as lcs;
use libra_config::config::RoleType;
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::{register_histogram, DurationHistogram};
use libra_network_address::NetworkAddress;
use libra_types::{
    on_chain_config::{OnChainConfigPayload, ValidatorSet, ON_CHAIN_CONFIG_REGISTRY},
    validator_config::ValidatorConfig,
};
use network::connectivity_manager::{ConnectivityRequest, DiscoverySource};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, time::Instant};
use subscription_service::ReconfigSubscription;

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
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    role: RoleType,
}

pub fn gen_simple_discovery_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all(ON_CHAIN_CONFIG_REGISTRY.to_vec(), vec![])
}

/// Extract the network_address from the provided config, depending on role.
fn network_address(role: RoleType, config: &ValidatorConfig) -> Result<NetworkAddress, lcs::Error> {
    match role {
        RoleType::Validator => NetworkAddress::try_from(&config.validator_network_address),
        RoleType::FullNode => NetworkAddress::try_from(&config.full_node_network_address),
    }
}

/// Extracts the public key from the provided config, depending on role.
fn public_key(role: RoleType, config: &ValidatorConfig) -> x25519::PublicKey {
    match role {
        RoleType::Validator => config.validator_network_identity_public_key,
        RoleType::FullNode => config.full_node_network_identity_public_key,
    }
}

/// Extracts a set of ConnectivityRequests from a ValidatorSet which are appropriate for a network with type role.
fn extract_updates(role: RoleType, node_set: ValidatorSet) -> Vec<ConnectivityRequest> {
    let node_list = node_set.payload().to_vec();

    let mut updates = Vec::new();

    // Collect the set of address updates.
    let address_map = node_list
        .iter()
        .flat_map(|node| match network_address(role, node.config()) {
            Ok(addr) => Some((*node.account_address(), vec![addr])),
            Err(e) => {
                warn!("Cannot parse network address {}", e);
                None
            }
        })
        .collect();

    let update_address_req =
        ConnectivityRequest::UpdateAddresses(DiscoverySource::OnChain, address_map);

    updates.push(update_address_req);

    // Collect the set of EligibleNodes
    updates.push(ConnectivityRequest::UpdateEligibleNodes(
        node_list
            .into_iter()
            .map(|node| (*node.account_address(), public_key(role, node.config())))
            .collect(),
    ));

    updates
}

impl ConfigurationChangeListener {
    /// Creates a new ConfigurationListener
    pub fn new(conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>, role: RoleType) -> Self {
        Self {
            conn_mgr_reqs_tx,
            role,
        }
    }

    /// Processes a received OnChainConfigPayload.  Depending on role (Validator or FullNode), parses
    /// the appropriate configuration changes and passes it to the ConnectionManager channel.
    async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let node_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let updates = match self.role {
            RoleType::Validator => extract_updates(self.role, node_set),
            RoleType::FullNode => extract_updates(self.role, node_set),
        };

        info!(
            "Update {} Network about new Node IDs",
            self.role.to_string()
        );

        for update in updates {
            match self.conn_mgr_reqs_tx.send(update).await {
                Ok(()) => (),
                Err(e) => warn!("Failed to send update to ConnectivityManager {}", e),
            }
        }
    }

    /// Starts the listener to wait on reconfiguration events.  Creates an infinite loop.
    pub async fn start(
        mut self,
        mut reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) {
        loop {
            let start_idle_time = Instant::now();
            let payload = reconfig_events.select_next_some().await;
            let idle_duration = start_idle_time.elapsed();
            let start_process_time = Instant::now();
            self.process_payload(payload).await;
            let process_duration = start_process_time.elapsed();

            EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
            EVENT_PROCESSING_LOOP_BUSY_DURATION_S.observe_duration(process_duration);
        }
    }
}
