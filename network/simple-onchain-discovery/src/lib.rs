// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::diem_channel::{self, Receiver};
use diem_config::{
    config::{Peer, PeerRole},
    network_id::NetworkContext,
};
use diem_crypto::x25519::PublicKey;
use diem_logger::prelude::*;
use diem_metrics::{
    register_histogram, register_int_counter_vec, register_int_gauge_vec, DurationHistogram,
    IntCounterVec, IntGaugeVec,
};
use diem_network_address_encryption::{Encryptor, Error as EncryptorError};
use diem_types::on_chain_config::{OnChainConfigPayload, ValidatorSet, ON_CHAIN_CONFIG_REGISTRY};
use futures::{sink::SinkExt, StreamExt};
use network::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    counters::inc_by_with_context,
    logging::NetworkSchema,
};
use once_cell::sync::Lazy;
use short_hex_str::AsShortHexStr;
use std::{collections::HashSet, sync::Arc};
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

pub static NETWORK_KEY_MISMATCH: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_network_key_mismatch",
        "Gauge of whether the network key mismatches onchain state",
        &["role_type", "network_id", "peer_id"]
    )
    .unwrap()
});

/// Listener which converts published  updates from the OnChainConfig to ConnectivityRequests
/// for the ConnectivityManager.
pub struct ConfigurationChangeListener {
    network_context: Arc<NetworkContext>,
    expected_pubkey: PublicKey,
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
    let is_validator = network_context.network_id().is_validator_network();

    // Decode addresses while ignoring bad addresses
    let discovered_peers = node_set
        .into_iter()
        .map(|info| {
            let peer_id = *info.account_address();
            let config = info.into_config();

            let addrs = if is_validator {
                let result = encryptor.decrypt(&config.validator_network_addresses, peer_id);
                if let Err(EncryptorError::StorageError(_)) = result {
                    panic!(
                        "Unable to initialize validator network addresses: {:?}",
                        result
                    );
                }
                result.map_err(anyhow::Error::from)
            } else {
                config
                    .fullnode_network_addresses()
                    .map_err(anyhow::Error::from)
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

            let peer_role = if is_validator {
                PeerRole::Validator
            } else {
                PeerRole::ValidatorFullNode
            };
            (peer_id, Peer::from_addrs(peer_role, addrs))
        })
        .collect();

    vec![ConnectivityRequest::UpdateDiscoveredPeers(
        DiscoverySource::OnChain,
        discovered_peers,
    )]
}

impl ConfigurationChangeListener {
    /// Creates a new ConfigurationChangeListener
    pub fn new(
        network_context: Arc<NetworkContext>,
        expected_pubkey: PublicKey,
        encryptor: Encryptor,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        Self {
            network_context,
            expected_pubkey,
            encryptor,
            conn_mgr_reqs_tx,
            reconfig_events,
        }
    }

    async fn next_reconfig_event(&mut self) -> Option<OnChainConfigPayload> {
        let _idle_timer = EVENT_PROCESSING_LOOP_IDLE_DURATION_S.start_timer();
        self.reconfig_events.next().await
    }

    fn find_key_mismatches(&self, onchain_keys: Option<&HashSet<PublicKey>>) {
        let mismatch = onchain_keys.map_or(0, |pubkeys| {
            if !pubkeys.contains(&self.expected_pubkey) {
                error!(
                    NetworkSchema::new(&self.network_context),
                    "Onchain pubkey {:?} differs from local pubkey {}",
                    pubkeys,
                    self.expected_pubkey
                );
                1
            } else {
                0
            }
        });

        NETWORK_KEY_MISMATCH
            .with_label_values(&[
                self.network_context.role().as_str(),
                self.network_context.network_id().as_str(),
                self.network_context.peer_id().short_str().as_str(),
            ])
            .set(mismatch);
    }

    /// Processes a received OnChainConfigPayload. Depending on role (Validator or FullNode), parses
    /// the appropriate configuration changes and passes it to the ConnectionManager channel.
    async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let _process_timer = EVENT_PROCESSING_LOOP_BUSY_DURATION_S.start_timer();

        let node_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let updates = extract_updates(self.network_context.clone(), &self.encryptor, node_set);

        // Ensure that the public key matches what's onchain for this peer
        for request in &updates {
            if let ConnectivityRequest::UpdateDiscoveredPeers(_, peer_updates) = request {
                self.find_key_mismatches(
                    peer_updates
                        .get(&self.network_context.peer_id())
                        .map(|peer| &peer.keys),
                )
            }
        }

        inc_by_with_context(
            &DISCOVERY_COUNTS,
            &self.network_context,
            "new_nodes",
            updates.len() as u64,
        );
        info!(
            NetworkSchema::new(&self.network_context),
            "Update {} Network about new Node IDs",
            self.network_context.network_id()
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

    /// Starts the listener to wait on reconfiguration events.
    pub async fn start(mut self) {
        info!(
            NetworkSchema::new(&self.network_context),
            "{} Starting OnChain Discovery actor", self.network_context
        );

        while let Some(payload) = self.next_reconfig_event().await {
            self.process_payload(payload).await;
        }

        warn!(
            NetworkSchema::new(&self.network_context),
            "{} OnChain Discovery actor terminated", self.network_context,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diem_config::config::HANDSHAKE_VERSION;
    use diem_crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
        x25519::PrivateKey,
        PrivateKey as PK, Uniform,
    };
    use diem_types::{
        network_address::NetworkAddress, on_chain_config::OnChainConfig,
        validator_config::ValidatorConfig, validator_info::ValidatorInfo, PeerId,
    };
    use futures::executor::block_on;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{collections::HashMap, time::Instant};
    use tokio::{
        runtime::Runtime,
        time::{timeout_at, Duration},
    };

    #[test]
    fn metric_if_key_mismatch() {
        diem_logger::DiemLogger::init_for_testing();
        let runtime = Runtime::new().unwrap();
        let consensus_private_key = Ed25519PrivateKey::generate_for_testing();
        let consensus_pubkey = consensus_private_key.public_key();
        let pubkey = test_pubkey([0u8; 32]);
        let different_pubkey = test_pubkey([1u8; 32]);
        let peer_id = diem_types::account_address::from_identity_public_key(pubkey);

        // Build up the Reconfig Listener
        let (conn_mgr_reqs_tx, _rx) = channel::new_test(1);
        let (mut reconfig_tx, reconfig_rx) = gen_simple_discovery_reconfig_subscription();
        let network_context = NetworkContext::mock_with_peer_id(peer_id);
        let listener = ConfigurationChangeListener::new(
            network_context.clone(),
            pubkey,
            Encryptor::for_testing(),
            conn_mgr_reqs_tx,
            reconfig_rx,
        );

        // Build up and send an update with a different pubkey
        send_pubkey_update(
            peer_id,
            consensus_pubkey,
            different_pubkey,
            &mut reconfig_tx,
        );

        let listener_future = async move {
            // Run the test, ensuring we actually stop after a couple seconds in case it fails to fail
            timeout_at(
                tokio::time::Instant::from(Instant::now() + Duration::from_secs(1)),
                listener.start(),
            )
            .await
            .expect_err("Expect timeout");
        };

        // Ensure the metric is updated
        check_network_key_mismatch_metric(0, &network_context);
        block_on(runtime.spawn(listener_future)).unwrap();
        check_network_key_mismatch_metric(1, &network_context);
    }

    fn check_network_key_mismatch_metric(expected: i64, network_context: &NetworkContext) {
        assert_eq!(
            expected,
            NETWORK_KEY_MISMATCH
                .get_metric_with_label_values(&[
                    network_context.role().as_str(),
                    network_context.network_id().as_str(),
                    network_context.peer_id().short_str().as_str()
                ])
                .unwrap()
                .get()
        )
    }

    fn send_pubkey_update(
        peer_id: PeerId,
        consensus_pubkey: Ed25519PublicKey,
        pubkey: PublicKey,
        reconfig_tx: &mut ReconfigSubscription,
    ) {
        let validator_address =
            NetworkAddress::mock().append_prod_protos(pubkey, HANDSHAKE_VERSION);
        let addresses = vec![validator_address];
        let encryptor = Encryptor::for_testing();
        let encrypted_addresses = encryptor.encrypt(&addresses, peer_id, 0).unwrap();
        let encoded_addresses = bcs::to_bytes(&addresses).unwrap();
        let validator = ValidatorInfo::new(
            peer_id,
            0,
            ValidatorConfig::new(consensus_pubkey, encrypted_addresses, encoded_addresses),
        );
        let validator_set = ValidatorSet::new(vec![validator]);
        let mut configs = HashMap::new();
        configs.insert(
            ValidatorSet::CONFIG_ID,
            bcs::to_bytes(&validator_set).unwrap(),
        );
        let payload = OnChainConfigPayload::new(1, Arc::new(configs));
        reconfig_tx.publish(payload).unwrap();
    }

    fn test_pubkey(seed: [u8; 32]) -> PublicKey {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let private_key = PrivateKey::generate(&mut rng);
        private_key.public_key()
    }
}
