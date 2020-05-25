// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{select, sink::SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::time::Instant;

use channel::libra_channel;
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_metrics::{register_histogram, DurationHistogram};
use libra_types::{
    on_chain_config::{OnChainConfigPayload, ValidatorSet},
    PeerId,
};
use network::{
    common::NetworkPublicKeys,
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
};

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

fn extract_validator_updates(
    validator_set: ValidatorSet,
) -> (
    Vec<ConnectivityRequest::UpdateAddresses>,
    ConnectivityRequest::UpdateEligibleNodes,
) {
    let validator_keys = validator_set.payload().to_vec();
    let nodes = ConnectivityRequest::UpdateEligibleNodes(
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
    );
    let addresses: Vec<ConnectivityRequest::UpdateAddresses()> = validator_keys
        .into_iter()
        .map(|validator| {
            ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                validator.account_address().clone(),
                validator.config().validator_network_address(),
            )
        })
        .collect();
    (addresses, nodes)
}

fn extract_full_node_updates(
    full_node_set: ValidatorSet,
) -> (
    Vec<ConnectivityRequest::UpdateAddresses>,
    ConnectivityRequest::UpdateEligibleNodes,
) {
    let full_node_infos = full_node_set.payload().to_vec();
    let nodes = ConnectivityRequest::UpdateEligibleNodes(
        full_node_infos
            .into_iter()
            .map(|full_node| {
                (
                    // TODO:  This seems like it is wrong and should be something else.
                    *full_node.account_address().clone(),
                    NetworkPublicKeys {
                        identity_public_key: full_node.config().full_node_network_identity_public_key().clone(),
                        // TODO:  there are not separate keys for the full_node.  Are they therefore the same?
                        signing_public_key: full_node.config().full_node_network_identity_public_key().clone(),
                    },
                )
            })
            .collect(),
    );
    let addresses: Vec<ConnectivityRequest::UpdateAddresses()> = full_node_infos
        .into_iter()
        .map(|full_node| {
            ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                // TODO:  again, this seems wrong for the account address
                full_node.account_address().clone(),
                full_node.config().full_node_network_address().clone(),
            )
        })
        .collect();
    (addresses, nodes)
}

impl ConfigurationChangeListener {
    pub fn new(conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>, role: RoleType) -> Self {
        Self {
            conn_mgr_reqs_tx,
            role,
        }
    }

    //the actual procssing of the onchainconfig payload event
    async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let node_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let (addresses, nodes) = match self.role {
            RoleType::Validator => extract_validator_updates(node_set),
            Roletype::FullNode => extract_full_node_updates(node_set),
        };

        info!(format!("Update {} Network about new Node IDs", self.role));
        self.conn_mgr_reqs_tx
            .send(nodes)
            .await
            .expect("Unable to update network's eligible peers");
        for address in addresses {
            self.conn_mgr_reqs_tex.send(address).await.expect(format!(
                "Unable to update network addresses for peer {}",
                address
            ));
        }
    }

    // Issues:  this appears to be the only thing pushed by statesync.
    // 1)  it does not currently have information about fullnode networks
    // 2) I dont think it has any information about ip addresses

    pub async fn start(
        mut self,
        mut reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) {
        loop {
            let pre_select_instant = Instant::now();
            let _idle_duration;
            _idle_duration = pre_select_instant.elapsed();
            self.process_payload(payload).await;

            //  EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
        }
    }
}
