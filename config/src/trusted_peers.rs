// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::{
    signing,
    utils::{encode_to_string, from_encoded_string},
    x25519::{self, X25519PrivateKey, X25519PublicKey},
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
    network_signing_pubkey: signing::PublicKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    network_identity_pubkey: X25519PublicKey,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    consensus_pubkey: signing::PublicKey,
}

pub struct TrustedPeerPrivateKeys {
    network_signing_private_key: signing::PrivateKey,
    network_identity_private_key: X25519PrivateKey,
    consensus_private_key: signing::PrivateKey,
}

impl TrustedPeerPrivateKeys {
    pub fn get_network_signing_private(&self) -> signing::PrivateKey {
        self.network_signing_private_key.clone()
    }
    pub fn get_network_identity_private(&self) -> X25519PrivateKey {
        self.network_identity_private_key.clone()
    }
    pub fn get_consensus_private(&self) -> signing::PrivateKey {
        self.consensus_private_key.clone()
    }
}

impl TrustedPeer {
    pub fn get_network_signing_public(&self) -> signing::PublicKey {
        self.network_signing_pubkey
    }
    pub fn get_network_identity_public(&self) -> X25519PublicKey {
        self.network_identity_pubkey
    }
    pub fn get_consensus_public(&self) -> signing::PublicKey {
        self.consensus_pubkey
    }
}

pub fn serialize_key<S, K>(key: &K, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize,
{
    serializer.serialize_str(&encode_to_string(key))
}

pub fn deserialize_key<'de, D, K>(deserializer: D) -> Result<K, D::Error>
where
    D: Deserializer<'de>,
    K: DeserializeOwned + 'static,
{
    let encoded_key: String = Deserialize::deserialize(deserializer)?;

    Ok(from_encoded_string(encoded_key))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustedPeersConfig {
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

    pub fn get_consensus_keys(&self, peer_id: &str) -> signing::PublicKey {
        self.get_public_keys(peer_id).consensus_pubkey
    }

    pub fn get_network_signing_keys(&self, peer_id: &str) -> signing::PublicKey {
        self.get_public_keys(peer_id).network_signing_pubkey
    }

    pub fn get_network_identity_keys(&self, peer_id: &str) -> X25519PublicKey {
        self.get_public_keys(peer_id).network_identity_pubkey
    }

    /// Returns a map of AccountAddress to its PublicKey for consensus.
    pub fn get_trusted_consensus_peers(&self) -> HashMap<AccountAddress, signing::PublicKey> {
        let mut res = HashMap::new();
        for (account, keys) in &self.peers {
            res.insert(
                AccountAddress::try_from(account.clone()).expect("Failed to parse account addr"),
                keys.consensus_pubkey,
            );
        }
        res
    }

    /// Returns a map of AccountAddress to a pair of PublicKeys for network peering. The first
    /// PublicKey is the one used for signing, whereas the second is to determine eligible members
    /// of the network.
    pub fn get_trusted_network_peers(
        &self,
    ) -> HashMap<AccountAddress, (signing::PublicKey, X25519PublicKey)> {
        self.peers
            .iter()
            .map(|(account, keys)| {
                (
                    AccountAddress::try_from(account.clone())
                        .expect("Failed to parse account addr"),
                    (keys.network_signing_pubkey, keys.network_identity_pubkey),
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
            let (private0, public0) = signing::generate_keypair_for_testing(&mut fast_rng);
            let (private1, public1) = x25519::generate_keypair_for_testing(&mut fast_rng);
            let (private2, public2) = signing::generate_keypair_for_testing(&mut fast_rng);
            // save the public_key in peers hashmap
            let peer = TrustedPeer {
                network_signing_pubkey: public0,
                network_identity_pubkey: public1,
                consensus_pubkey: public2,
            };
            let peer_id = AccountAddress::from(peer.consensus_pubkey);
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
