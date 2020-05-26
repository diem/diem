// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::libra_channel;
use futures::{sink::SinkExt, StreamExt};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_metrics::{register_histogram, DurationHistogram};
use libra_network_address::NetworkAddress;
use libra_types::on_chain_config::{OnChainConfigPayload, ValidatorSet};
use network::{
    common::NetworkPublicKeys,
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, time::Instant};

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

pub struct ConfigurationChangeListener {
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    role: RoleType,
}

fn extract_validator_updates(validator_set: ValidatorSet) -> Vec<ConnectivityRequest> {
    let validator_keys = validator_set.payload().to_vec();

    // Collect the set of address updates.
    let updates: Vec<ConnectivityRequest> = validator_keys
        .into_iter()
        .map(|validator| {
            ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                validator.account_address().clone(),
                vec![
                    NetworkAddress::try_from(&validator.config().validator_network_address)
                        .expect("WTF"),
                ],
            )
        })
        .collect();

    // Collect the set of EligibleNodes
    updates.push(ConnectivityRequest::UpdateEligibleNodes(
        validator_keys
            .into_iter()
            .map(|validator| {
                (
                    *validator.account_address(),
                    NetworkPublicKeys {
                        identity_public_key: validator.network_identity_public_key(),
                        signing_public_key: validator.network_signing_public_key().clone(),
                    },
                )
            })
            .collect(),
    ));

    updates
}

fn extract_full_node_updates(full_node_set: ValidatorSet) -> Vec<ConnectivityRequest> {
    let full_node_infos = full_node_set.payload().to_vec();

    // Collect the set of updated addresses.
    let updates: Vec<ConnectivityRequest> = full_node_infos
        .into_iter()
        .map(|full_node| {
            ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                // TODO:  this seems wrong for the account address.  Is it shared with the validator?
                // A:  should be account_address::from_public_key
                full_node.account_address().clone(),
                vec![NetworkAddress::try_from(
                    &full_node.config().full_node_network_address.clone(),
                )
                .expect("WTF")],
            )
        })
        .collect();

    // Collect the EligibleNodes
    updates.push(ConnectivityRequest::UpdateEligibleNodes(
        full_node_infos
            .into_iter()
            .map(|full_node| {
                (
                    // TODO:  This seems like it is wrong and should be something else.  Or is the full_node account address the same as the validator?
                    full_node.account_address().clone(),
                    NetworkPublicKeys {
                        identity_public_key: full_node
                            .config()
                            .full_node_network_identity_public_key
                            .clone(),
                        // FullNodes are currently trusted portions of the validator and use the primary signing key.
                        signing_public_key: full_node
                            .config()
                            .validator_network_signing_public_key
                            .clone(),
                    },
                )
            })
            .collect(),
    ));

    updates
}

impl ConfigurationChangeListener {
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
            RoleType::Validator => extract_validator_updates(node_set),
            RoleType::FullNode => extract_full_node_updates(node_set),
        };

        info!(
            "Update {} Network about new Node IDs",
            self.role.to_string()
        );

        let mut futures = Vec::new();
        for mut update in updates {
            match &update {
                ConnectivityRequest::UpdateAddresses(a, b, c) => {
                    info!(
                        "Updating {} network address for {} to {}",
                        self.role.to_string(),
                        b.to_string(),
                        c[0].to_string()
                    );
                }
                ConnectivityRequest::UpdateEligibleNodes(a) => {
                    info!(
                        "Update {} network about new Node IDs",
                        self.role.to_string()
                    );
                }
                ConnectivityRequest::GetDialQueueSize(a) => {
                    unimplemented!("Not expecting to receive this change notification!");
                }
            };
            futures.push(self.conn_mgr_reqs_tx.send(update));
        }
        join!(futures);
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
