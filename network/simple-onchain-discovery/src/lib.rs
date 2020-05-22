// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::sink::SinkExt;
use futures::{select, StreamExt};
use once_cell::sync::Lazy;
use std::time::Instant;

use channel::libra_channel;
use libra_logger::prelude::*;
use libra_metrics::{register_histogram, DurationHistogram};
use libra_types::on_chain_config::{OnChainConfigPayload, ValidatorSet};
use network::{common::NetworkPublicKeys, connectivity_manager::ConnectivityRequest};

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
}

fn validator_set_to_update_eligible_nodes_request(
    validator_set: ValidatorSet,
) -> ConnectivityRequest {
    let validator_keys = validator_set.payload().to_vec();
    ConnectivityRequest::UpdateEligibleNodes(
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
    )
}

impl ConfigurationChangeListener {
    pub fn new(conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>) -> Self {
        Self { conn_mgr_reqs_tx }
    }

    //the actual procssing of the onchainconfig payload event
    async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let validator_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let update_eligible_nodes_request =
            validator_set_to_update_eligible_nodes_request(validator_set);
        info!("Update Network about new validators");
        self.conn_mgr_reqs_tx
            .send(update_eligible_nodes_request)
            .await
            .expect("Unable to update network's eligible peers");
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
            select! {
                  payload = reconfig_events.select_next_some() => {
                      _idle_duration = pre_select_instant.elapsed();
                      self.process_payload(payload).await
              }

            //  EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
              }
        }
    }
}
