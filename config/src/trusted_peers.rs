// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{compat, *},
    traits::{ValidKey, ValidKeyStringExt},
    x25519::{self, X25519StaticPrivateKey, X25519StaticPublicKey},
};
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{ValidatorInfo, ValidatorVerifier},
    validator_public_keys::ValidatorPublicKeys,
    validator_set::ValidatorSet,
    PeerId,
};
use mirai_annotations::postcondition;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    fmt,
    hash::BuildHasher,
    str::FromStr,
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

pub struct NetworkPrivateKeys {
    pub network_signing_private_key: Ed25519PrivateKey,
    pub network_identity_private_key: X25519StaticPrivateKey,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NetworkPeersConfig {
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_ordered_map")]
    pub peers: HashMap<String, NetworkPeerInfo>,
}

impl fmt::Debug for NetworkPeersConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} keys>", self.peers.len())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusPeerInfo {
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "c")]
    pub consensus_pubkey: Ed25519PublicKey,
}

pub struct ConsensusPrivateKey {
    pub consensus_private_key: Ed25519PrivateKey,
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
                    // TODO: Add support for dynamic voting weights in config
                    1,
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

    pub fn get_validator_verifier(&self) -> ValidatorVerifier {
        ValidatorVerifier::new(
            self.peers
                .iter()
                .map(|(peer_id_str, peer_info)| {
                    (
                        PeerId::from_str(peer_id_str).unwrap_or_else(|_| {
                            panic!(
                                "Failed to deserialize PeerId: {} from consensus peers config: ",
                                peer_id_str
                            )
                        }),
                        // TODO: Add support for dynamic voting weights in config
                        ValidatorInfo::new(peer_info.consensus_pubkey.clone(), 1),
                    )
                })
                .collect(),
        )
    }
}

// TODO: move to mod utils.
pub struct ConfigHelpers {}

// TODO: Update comment.
/// Creates a new TrustedPeersConfig with the given number of peers,
/// as well as a hashmap of all the test validator nodes' private keys.
impl ConfigHelpers {
    pub fn gen_validator_nodes(
        num_peers: usize,
        seed: Option<[u8; 32]>,
    ) -> (
        HashMap<AccountAddress, (ConsensusPrivateKey, NetworkPrivateKeys)>,
        ConsensusPeersConfig,
        NetworkPeersConfig,
    ) {
        let mut consensus_peers = HashMap::new();
        let mut network_peers = HashMap::new();
        let mut consensus_private_keys = BTreeMap::new();
        let mut peers_private_keys = HashMap::new();
        // Deterministically derive keypairs from a seeded-rng
        let seed = seed.unwrap_or([0u8; 32]);
        let mut fast_rng = StdRng::from_seed(seed);
        for _ in 0..num_peers {
            let _ = compat::generate_keypair(&mut fast_rng);
            let _ = x25519::compat::generate_keypair(&mut fast_rng);
            let (private2, public2) = compat::generate_keypair(&mut fast_rng);
            // Generate peer id from consensus public key.
            let peer_id = AccountAddress::from_public_key(&public2);
            consensus_peers.insert(
                peer_id.to_string(),
                ConsensusPeerInfo {
                    consensus_pubkey: public2,
                },
            );
            consensus_private_keys.insert(
                peer_id,
                ConsensusPrivateKey {
                    consensus_private_key: private2,
                },
            );
        }
        let mut fast_rng = StdRng::from_seed(seed);
        for (peer_id, consensus_private_key) in consensus_private_keys.into_iter() {
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let (private1, public1) = x25519::compat::generate_keypair(&mut fast_rng);
            let _ = compat::generate_keypair(&mut fast_rng);
            network_peers.insert(
                peer_id.to_string(),
                NetworkPeerInfo {
                    network_signing_pubkey: public0,
                    network_identity_pubkey: public1,
                },
            );
            // save the private keys in a different hashmap
            peers_private_keys.insert(
                peer_id,
                (
                    consensus_private_key,
                    NetworkPrivateKeys {
                        network_identity_private_key: private1,
                        network_signing_private_key: private0,
                    },
                ),
            );
        }
        postcondition!(peers_private_keys.len() == num_peers);
        (
            peers_private_keys,
            ConsensusPeersConfig {
                peers: consensus_peers,
            },
            NetworkPeersConfig {
                peers: network_peers,
            },
        )
    }

    pub fn gen_full_nodes(
        num_peers: usize,
        seed: Option<[u8; 32]>,
    ) -> (
        HashMap<AccountAddress, NetworkPrivateKeys>,
        NetworkPeersConfig,
    ) {
        let mut network_peers = HashMap::new();
        let mut peers_private_keys = HashMap::new();
        // Deterministically derive keypairs from a seeded-rng
        let seed = seed.unwrap_or([1u8; 32]);
        let mut fast_rng = StdRng::from_seed(seed);
        for _ in 0..num_peers {
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let (private1, public1) = x25519::compat::generate_keypair(&mut fast_rng);
            // Generate peer id from network identity key.
            let peer_id = AccountAddress::try_from(public1.to_bytes()).unwrap();
            network_peers.insert(
                peer_id.to_string(),
                NetworkPeerInfo {
                    network_signing_pubkey: public0,
                    network_identity_pubkey: public1,
                },
            );
            // save the private keys in a different hashmap
            peers_private_keys.insert(
                peer_id,
                NetworkPrivateKeys {
                    network_identity_private_key: private1,
                    network_signing_private_key: private0,
                },
            );
        }
        postcondition!(peers_private_keys.len() == num_peers);
        (
            peers_private_keys,
            NetworkPeersConfig {
                peers: network_peers,
            },
        )
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
