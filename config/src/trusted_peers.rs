// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::{
    ed25519::{compat, *},
    traits::ValidKeyStringExt,
    x25519::{self, X25519StaticPrivateKey, X25519StaticPublicKey},
};
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
    str::FromStr,
};
use types::{
    account_address::AccountAddress, validator_public_keys::ValidatorPublicKeys,
    validator_set::ValidatorSet,
};

#[cfg(test)]
#[path = "unit_tests/trusted_peers_test.rs"]
mod trusted_peers_test;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkPeerInfo {
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "ns")]
    pub network_signing_pubkey: Ed25519PublicKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "ni")]
    pub network_identity_pubkey: X25519StaticPublicKey,
}

pub struct NetworkPeerPrivateKeys {
    pub network_signing_private_key: Ed25519PrivateKey,
    pub network_identity_private_key: X25519StaticPrivateKey,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkPeersConfig {
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_ordered_map")]
    pub peers: HashMap<String, NetworkPeerInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusPeerInfo {
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "c")]
    pub consensus_pubkey: Ed25519PublicKey,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusPeersConfig {
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_ordered_map")]
    pub peers: HashMap<String, ConsensusPeerInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpstreamPeersConfig {
    /// List of PeerIds serialized as string.
    pub upstream_peers: Vec<String>,
}

impl ConsensusPeersConfig {
    /// Return a sorted vector of ValidatorPublicKey's
    pub fn get_validator_set(&self, network_peers_config: &NetworkPeersConfig) -> ValidatorSet {
        let mut keys: Vec<ValidatorPublicKeys> = self
            .peers
            .iter()
            .map(|(peer_id_str, peer_info)| {
                ValidatorPublicKeys::new(
                    AccountAddress::from_str(peer_id_str).expect("[config] invalid peer_id"),
                    peer_info.consensus_pubkey.clone(),
                    network_peers_config
                        .peers
                        .get(peer_id_str)
                        .unwrap()
                        .network_signing_pubkey
                        .clone(),
                    network_peers_config
                        .peers
                        .get(peer_id_str)
                        .unwrap()
                        .network_identity_pubkey
                        .clone(),
                )
            })
            .collect();
        // self.peers is a HashMap, so iterating over it produces a differently ordered vector each
        // time. Sort by account address to produce a canonical ordering
        keys.sort_by(|k1, k2| k1.account_address().cmp(k2.account_address()));
        ValidatorSet::new(keys)
    }
}

// TODO: move to mod utils.
pub struct ConfigHelpers {}

// TODO: Update comment.
/// Creates a new TrustedPeersConfig with the given number of peers,
/// as well as a hashmap of all the test validator nodes' private keys.
impl ConfigHelpers {
    pub fn get_test_consensus_config(
        number_of_peers: usize,
        seed: Option<[u8; 32]>,
    ) -> (HashMap<String, Ed25519PrivateKey>, ConsensusPeersConfig) {
        let mut consensus_peers = HashMap::new();
        let mut peers_private_keys = HashMap::new();
        // deterministically derive keypairs from a seeded-rng
        let seed = if let Some(seed) = seed {
            seed
        } else {
            [0u8; 32]
        };
        let mut fast_rng = StdRng::from_seed(seed);
        for _ in 0..number_of_peers {
            // Generate extra keypairs to preserve peer ids.
            let _ = compat::generate_keypair(&mut fast_rng);
            let _ = x25519::compat::generate_keypair(&mut fast_rng);
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let peer_id = AccountAddress::from_public_key(&public0);
            consensus_peers.insert(
                peer_id.to_string(),
                ConsensusPeerInfo {
                    consensus_pubkey: public0,
                },
            );
            // save the private keys in a different hashmap
            peers_private_keys.insert(peer_id.to_string(), private0);
        }
        (
            peers_private_keys,
            ConsensusPeersConfig {
                peers: consensus_peers,
            },
        )
    }

    pub fn get_test_network_peers_config(
        consensus_peers: &ConsensusPeersConfig,
        seed: Option<[u8; 32]>,
    ) -> (HashMap<String, NetworkPeerPrivateKeys>, NetworkPeersConfig) {
        let mut network_peers = HashMap::new();
        let mut peers_private_keys = HashMap::new();
        // deterministically derive keypairs from a seeded-rng
        let seed = if let Some(seed) = seed {
            seed
        } else {
            [0u8; 32]
        };
        let peers_ordered: BTreeMap<_, _> = consensus_peers.peers.iter().collect();
        let mut fast_rng = StdRng::from_seed(seed);
        for peer_id in peers_ordered.keys().cloned() {
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let (private1, public1) = x25519::compat::generate_keypair(&mut fast_rng);
            // Generate extra keypairs to preserve peer ids.
            let _ = compat::generate_keypair(&mut fast_rng);
            // save the public_key in peers hashmap
            let peer = NetworkPeerInfo {
                network_signing_pubkey: public0,
                network_identity_pubkey: public1,
            };
            network_peers.insert(peer_id.clone(), peer);
            // save the private keys in a different hashmap
            let private_keys = NetworkPeerPrivateKeys {
                network_signing_private_key: private0,
                network_identity_private_key: private1,
            };
            peers_private_keys.insert(peer_id.clone(), private_keys);
        }
        (
            peers_private_keys,
            NetworkPeersConfig {
                peers: network_peers,
            },
        )
    }

    pub fn get_test_upstream_peers_config(
        network_peers: &NetworkPeersConfig,
    ) -> UpstreamPeersConfig {
        let upstream_peers = network_peers.peers.keys().cloned().collect();
        UpstreamPeersConfig { upstream_peers }
    }
}

pub fn serialize_key<S, K>(key: &K, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize + ValidKeyStringExt,
{
    key.to_encoded_string()
        .map_err(<S::Error as serde::ser::Error>::custom)
        .and_then(|str| serializer.serialize_str(&str[..]))
}

pub fn deserialize_key<'de, D, K>(deserializer: D) -> Result<K, D::Error>
where
    D: Deserializer<'de>,
    K: ValidKeyStringExt + DeserializeOwned + 'static,
{
    let encoded_key: String = Deserialize::deserialize(deserializer)?;

    ValidKeyStringExt::from_encoded_string(&encoded_key)
        .map_err(<D::Error as serde::de::Error>::custom)
}

pub fn serialize_ordered_map<S, V, H>(
    value: &HashMap<String, V, H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    H: BuildHasher,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
