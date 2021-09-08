// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::{DISCOVERY_COUNTS, EVENT_PROCESSING_LOOP_BUSY_DURATION_S, NETWORK_KEY_MISMATCH},
    DiscoveryError,
};
use diem_config::{
    config::{Peer, PeerRole, PeerSet},
    network_id::NetworkContext,
};
use diem_crypto::x25519;
use diem_logger::prelude::*;
use diem_network_address_encryption::{Encryptor, Error as EncryptorError};
use diem_secure_storage::Storage;
use diem_types::on_chain_config::{OnChainConfigPayload, ValidatorSet};
use event_notifications::ReconfigNotificationListener;
use futures::Stream;
use network::{counters::inc_by_with_context, logging::NetworkSchema};
use short_hex_str::AsShortHexStr;
use std::{
    collections::HashSet,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct ValidatorSetStream {
    pub(crate) network_context: Arc<NetworkContext>,
    expected_pubkey: x25519::PublicKey,
    encryptor: Encryptor<Storage>,
    reconfig_events: ReconfigNotificationListener,
}

impl ValidatorSetStream {
    pub(crate) fn new(
        network_context: Arc<NetworkContext>,
        expected_pubkey: x25519::PublicKey,
        encryptor: Encryptor<Storage>,
        reconfig_events: ReconfigNotificationListener,
    ) -> Self {
        Self {
            network_context,
            expected_pubkey,
            encryptor,
            reconfig_events,
        }
    }

    fn find_key_mismatches(&self, onchain_keys: Option<&HashSet<x25519::PublicKey>>) {
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

    fn extract_updates(&mut self, payload: OnChainConfigPayload) -> PeerSet {
        let _process_timer = EVENT_PROCESSING_LOOP_BUSY_DURATION_S.start_timer();

        let node_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let peer_set =
            extract_validator_set_updates(self.network_context.clone(), &self.encryptor, node_set);
        // Ensure that the public key matches what's onchain for this peer
        self.find_key_mismatches(
            peer_set
                .get(&self.network_context.peer_id())
                .map(|peer| &peer.keys),
        );

        inc_by_with_context(
            &DISCOVERY_COUNTS,
            &self.network_context,
            "new_nodes",
            peer_set.len() as u64,
        );

        peer_set
    }
}

impl Stream for ValidatorSetStream {
    type Item = Result<PeerSet, DiscoveryError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.reconfig_events)
            .poll_next(cx)
            .map(|maybe_notification| {
                maybe_notification
                    .map(|notification| Ok(self.extract_updates(notification.on_chain_configs)))
            })
    }
}

/// Extracts a set of ConnectivityRequests from a ValidatorSet which are appropriate for a network with type role.
fn extract_validator_set_updates(
    network_context: Arc<NetworkContext>,
    encryptor: &Encryptor<Storage>,
    node_set: ValidatorSet,
) -> PeerSet {
    let is_validator = network_context.network_id().is_validator_network();

    // Decode addresses while ignoring bad addresses
    node_set
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
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiscoveryChangeListener;
    use channel::{diem_channel, message_queues::QueueStyle};
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
    use event_notifications::ReconfigNotification;
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
        let (mut reconfig_sender, reconfig_events) = diem_channel::new(QueueStyle::LIFO, 1, None);
        let reconfig_listener = ReconfigNotificationListener {
            notification_receiver: reconfig_events,
        };
        let network_context = NetworkContext::mock_with_peer_id(peer_id);
        let listener = DiscoveryChangeListener::validator_set(
            network_context.clone(),
            conn_mgr_reqs_tx,
            pubkey,
            Encryptor::for_testing(),
            reconfig_listener,
        );

        // Build up and send an update with a different pubkey
        send_pubkey_update(
            peer_id,
            consensus_pubkey,
            different_pubkey,
            &mut reconfig_sender,
        );

        let listener_future = async move {
            // Run the test, ensuring we actually stop after a couple seconds in case it fails to fail
            timeout_at(
                tokio::time::Instant::from(Instant::now() + Duration::from_secs(1)),
                Box::pin(listener).run(),
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
        pubkey: x25519::PublicKey,
        reconfig_tx: &mut channel::diem_channel::Sender<(), ReconfigNotification>,
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
        reconfig_tx
            .push(
                (),
                ReconfigNotification {
                    version: 1,
                    on_chain_configs: payload,
                },
            )
            .unwrap();
    }

    fn test_pubkey(seed: [u8; 32]) -> x25519::PublicKey {
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let private_key = PrivateKey::generate(&mut rng);
        private_key.public_key()
    }
}
