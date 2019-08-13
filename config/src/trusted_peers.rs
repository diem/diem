// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use nextgen_crypto::{
    ed25519::{compat, *},
    traits::ValidKeyStringExt,
    x25519::{self, X25519StaticPrivateKey, X25519StaticPublicKey},
};
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use types::account_address::AccountAddress;

#[cfg(test)]
#[path = "unit_tests/trusted_peers_test.rs"]
mod trusted_peers_test;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustedPeer {
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "ns")]
    network_signing_pubkey: Ed25519PublicKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "ni")]
    network_identity_pubkey: X25519StaticPublicKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    #[serde(rename = "c")]
    consensus_pubkey: Ed25519PublicKey,
}

pub struct TrustedPeerPrivateKeys {
    network_signing_private_key: Ed25519PrivateKey,
    network_identity_private_key: X25519StaticPrivateKey,
    consensus_private_key: Ed25519PrivateKey,
}

impl TrustedPeerPrivateKeys {
    pub fn get_key_triplet(self) -> (Ed25519PrivateKey, X25519StaticPrivateKey, Ed25519PrivateKey) {
        (
            self.network_signing_private_key,
            self.network_identity_private_key,
            self.consensus_private_key,
        )
    }
}

impl TrustedPeer {
    pub fn get_network_signing_public(&self) -> &Ed25519PublicKey {
        &self.network_signing_pubkey
    }
    pub fn get_network_identity_public(&self) -> &X25519StaticPublicKey {
        &self.network_identity_pubkey
    }
    pub fn get_consensus_public(&self) -> &Ed25519PublicKey {
        &self.consensus_pubkey
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

pub fn serialize_opt_key<S, K>(opt_key: &Option<K>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize + ValidKeyStringExt,
{
    opt_key
        .as_ref()
        .map_or(Ok("".to_string()), |key| {
            key.to_encoded_string()
                .map_err(<S::Error as serde::ser::Error>::custom)
        })
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

pub fn deserialize_opt_key<'de, D, K>(deserializer: D) -> Result<Option<K>, D::Error>
where
    D: Deserializer<'de>,
    K: ValidKeyStringExt + DeserializeOwned + 'static,
{
    let encoded_key: String = Deserialize::deserialize(deserializer)?;

    ValidKeyStringExt::from_encoded_string(&encoded_key)
        .map_err(<D::Error as serde::de::Error>::custom)
        .map(Some)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustedPeersConfig {
    #[serde(flatten)]
    pub peers: HashMap<String, TrustedPeer>,
}

impl TrustedPeersConfig {
    pub fn load_config<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut file = File::open(path)
            .unwrap_or_else(|_| panic!("Cannot open Trusted Peers Config file {:?}", path));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Error reading Trusted Peers Config file {:?}", path));

        Self::parse(&contents)
    }

    pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");

        let mut file = File::create(output_file).expect("Error opening file");

        file.write_all(&contents).expect("Error writing file");
    }

    pub fn get_public_keys(&self, peer_id: &str) -> TrustedPeer {
        self.peers
            .get(peer_id)
            .unwrap_or_else(|| panic!("Missing keys for {}", peer_id))
            .clone()
    }

    pub fn get_consensus_keys(&self, peer_id: &str) -> Ed25519PublicKey {
        self.get_public_keys(peer_id).consensus_pubkey
    }

    pub fn get_network_signing_keys(&self, peer_id: &str) -> Ed25519PublicKey {
        self.get_public_keys(peer_id).network_signing_pubkey
    }

    pub fn get_network_identity_keys(&self, peer_id: &str) -> X25519StaticPublicKey {
        self.get_public_keys(peer_id).network_identity_pubkey
    }

    /// Returns a map of AccountAddress to its PublicKey for consensus.
    pub fn get_trusted_consensus_peers(&self) -> HashMap<AccountAddress, Ed25519PublicKey> {
        let mut res = HashMap::new();
        for (account, keys) in &self.peers {
            res.insert(
                AccountAddress::try_from(account.clone()).expect("Failed to parse account addr"),
                keys.consensus_pubkey.clone(),
            );
        }
        res
    }

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

    fn parse(config_string: &str) -> Self {
        toml::from_str(config_string).expect("Unable to parse Config")
    }
}

impl Default for TrustedPeersConfig {
    fn default() -> TrustedPeersConfig {
        Self {
            peers: HashMap::new(),
        }
    }
}

pub struct TrustedPeersConfigHelpers {}

impl TrustedPeersConfigHelpers {
    /// Creates a new TrustedPeersConfig with the given number of peers,
    /// as well as a hashmap of all the test validator nodes' private keys.
    pub fn get_test_config(
        number_of_peers: usize,
        seed: Option<[u8; 32]>,
    ) -> (HashMap<String, TrustedPeerPrivateKeys>, TrustedPeersConfig) {
        let mut peers = HashMap::new();
        let mut peers_private_keys = HashMap::new();
        // deterministically derive keypairs from a seeded-rng
        let seed = if let Some(seed) = seed {
            seed
        } else {
            [0u8; 32]
        };

        let mut fast_rng = StdRng::from_seed(seed);
        for _ in 0..number_of_peers {
            let (private0, public0) = compat::generate_keypair(&mut fast_rng);
            let (private1, public1) = x25519::compat::generate_keypair(&mut fast_rng);
            let (private2, public2) = compat::generate_keypair(&mut fast_rng);
            // save the public_key in peers hashmap
            let peer = TrustedPeer {
                network_signing_pubkey: public0,
                network_identity_pubkey: public1,
                consensus_pubkey: public2,
            };
            let peer_id = AccountAddress::from_public_key(&peer.consensus_pubkey);
            peers.insert(peer_id.to_string(), peer);
            // save the private keys in a different hashmap
            let private_keys = TrustedPeerPrivateKeys {
                network_signing_private_key: private0,
                network_identity_private_key: private1,
                consensus_private_key: private2,
            };
            peers_private_keys.insert(peer_id.to_string(), private_keys);
        }
        (peers_private_keys, TrustedPeersConfig { peers })
    }
}
