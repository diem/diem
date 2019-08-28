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
    convert::TryFrom,
    hash::BuildHasher,
};
use types::account_address::AccountAddress;

#[cfg(test)]
#[path = "unit_tests/trusted_peers_test.rs"]
mod trusted_peers_test;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkPubKeys {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusPeer {
    #[serde(rename = "aa")]
    pub account_address: String,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "c")]
    pub consensus_pubkey: Ed25519PublicKey,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkPeersConfig {
    // TODO: Replace HashMap with Vec<NetworkPubKeys> after migration of network stack to using
    // NetworkIdentityKey as node identifier.
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_ordered_map")]
    pub peers: HashMap<String, NetworkPubKeys>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusPeersConfig {
    #[serde(default)]
    pub peers: Vec<ConsensusPeer>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpstreamPeersConfig {
    /// List of PeerIds serialized as string.
    pub peers: Vec<String>,
}

impl NetworkPeersConfig {
    /// Returns a map of AccountAddress to a pair of PublicKeys for network peering. The first
    /// PublicKey is the one used for signing, whereas the second is to determine eligible members
    /// of the network.
    pub fn get_trusted_network_peers(
        &self,
    ) -> HashMap<AccountAddress, (Ed25519PublicKey, X25519StaticPublicKey)> {
        self.peers
            .iter()
            .map(|(account, keys)| {
                (
                    AccountAddress::try_from(account.clone())
                        .expect("Failed to parse account addr"),
                    (
                        keys.network_signing_pubkey.clone(),
                        keys.network_identity_pubkey.clone(),
                    ),
                )
            })
            .collect()
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
        let mut consensus_peers = Vec::new();
        let mut peers_private_keys = HashMap::new();
        // deterministically derive keypairs from a seeded-rng
        let seed = if let Some(seed) = seed {
            seed
        } else {
            [0u8; 32]
        };
        let mut fast_rng = StdRng::from_seed(seed);
        for _ in 0..number_of_peers {
            // TODO: generate to preserve peer ids.
            let _ = compat::generate_keypair(&mut fast_rng);
            let _ = x25519::compat::generate_keypair(&mut fast_rng);
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let peer_id = AccountAddress::from_public_key(&public0);
            consensus_peers.push(ConsensusPeer {
                account_address: peer_id.to_string(),
                consensus_pubkey: public0,
            });
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
        let mut fast_rng = StdRng::from_seed(seed);
        for validator in &consensus_peers.peers {
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let (private1, public1) = x25519::compat::generate_keypair(&mut fast_rng);
            // save the public_key in peers hashmap
            let peer = NetworkPubKeys {
                network_signing_pubkey: public0,
                network_identity_pubkey: public1,
            };
            let peer_id = validator.account_address.clone();
            network_peers.insert(peer_id.clone(), peer);
            // save the private keys in a different hashmap
            let private_keys = NetworkPeerPrivateKeys {
                network_signing_private_key: private0,
                network_identity_private_key: private1,
            };
            peers_private_keys.insert(peer_id, private_keys);
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
        let mut upstream_peers = Vec::new();
        for peer in network_peers.peers.keys() {
            upstream_peers.push(peer.clone());
        }
        UpstreamPeersConfig {
            peers: upstream_peers,
        }
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

pub fn serialize_ordered_map<S, H>(
    value: &HashMap<String, NetworkPubKeys, H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    H: BuildHasher,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
