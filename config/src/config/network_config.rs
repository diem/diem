// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{BaseConfig, PersistableConfig, RoleType},
    keys::{self, KeyPair},
    utils,
};
use failure::{self, anyhow, ensure, Result};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
    Uniform, ValidKey,
};
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, path::PathBuf, string::ToString, sync::Arc};

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub peer_id: PeerId,
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: Multiaddr,
    // The address that this node advertises to other nodes for the discovery protocol.
    pub advertised_address: Multiaddr,
    pub discovery_interval_ms: u64,
    pub connectivity_check_interval_ms: u64,
    // Flag to toggle if Noise is used for encryption and authentication.
    pub enable_encryption_and_authentication: bool,
    // If the network is permissioned, only trusted peers are allowed to connect. Otherwise, any
    // node can connect. If this flag is set to true, the `enable_encryption_and_authentication`
    // must also be set to true.
    pub is_permissioned: bool,
    // network_keypairs contains the node's network keypairs.
    // it is filled later on from network_keypairs_file.
    #[serde(skip)]
    pub network_keypairs: NetworkKeyPairs,
    pub network_keypairs_file: PathBuf,
    // network peers are the nodes allowed to connect when the network is started in permissioned
    // mode.
    #[serde(skip)]
    pub network_peers: NetworkPeersConfig,
    pub network_peers_file: PathBuf,
    // seed_peers act as seed nodes for the discovery protocol.
    #[serde(skip)]
    pub seed_peers: SeedPeersConfig,
    pub seed_peers_file: PathBuf,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let keypair = NetworkKeyPairs::default();
        let peer_id = PeerId::try_from(keypair.identity_keys.public().to_bytes()).unwrap();
        let peers = Self::default_peers(&keypair, &peer_id);

        Self {
            peer_id,
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse::<Multiaddr>().unwrap(),
            advertised_address: "/ip4/127.0.0.1/tcp/6180".parse::<Multiaddr>().unwrap(),
            discovery_interval_ms: 1000,
            connectivity_check_interval_ms: 5000,
            enable_encryption_and_authentication: true,
            is_permissioned: true,
            network_keypairs_file: PathBuf::new(),
            network_keypairs: keypair,
            network_peers_file: PathBuf::new(),
            network_peers: peers,
            seed_peers_file: PathBuf::new(),
            seed_peers: SeedPeersConfig::default(),
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl NetworkConfig {
    /// This clones the underlying data except for the keypair so that this config can be used as a
    /// template for another config.
    pub fn clone_for_template(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            listen_address: self.listen_address.clone(),
            advertised_address: self.advertised_address.clone(),
            discovery_interval_ms: self.discovery_interval_ms,
            connectivity_check_interval_ms: self.connectivity_check_interval_ms,
            enable_encryption_and_authentication: self.enable_encryption_and_authentication,
            is_permissioned: self.is_permissioned,
            network_keypairs_file: self.network_keypairs_file.clone(),
            network_keypairs: NetworkKeyPairs::default(),
            network_peers_file: self.network_peers_file.clone(),
            network_peers: self.network_peers.clone(),
            seed_peers_file: self.seed_peers_file.clone(),
            seed_peers: self.seed_peers.clone(),
            base: self.base.clone(),
        }
    }

    pub fn prepare(&mut self, base: Arc<BaseConfig>) {
        self.base = base;
    }

    pub fn load(&mut self, network_role: RoleType) -> Result<()> {
        if !self.network_peers_file.as_os_str().is_empty() {
            self.network_peers = NetworkPeersConfig::load_config(self.network_peers_file());
        }
        if !self.network_keypairs_file.as_os_str().is_empty() {
            self.network_keypairs = NetworkKeyPairs::load_config(self.network_keypairs_file());
        }
        if !self.seed_peers_file.as_os_str().is_empty() {
            self.seed_peers = SeedPeersConfig::load_config(self.seed_peers_file());
        }
        if self.advertised_address.to_string().is_empty() {
            self.advertised_address =
                utils::get_local_ip().ok_or_else(|| anyhow!("No local IP"))?;
        }
        if self.listen_address.to_string().is_empty() {
            self.listen_address = utils::get_local_ip().ok_or_else(|| anyhow!("No local IP"))?;
        }
        // If PeerId is not set, it is derived from NetworkIdentityKey.
        if self.peer_id == PeerId::default() {
            self.peer_id =
                PeerId::try_from(self.network_keypairs.identity_keys.public().to_bytes()).unwrap();
        }

        if !network_role.is_validator() {
            ensure!(
                self.peer_id ==
                PeerId::try_from(
                        self.network_keypairs
                        .identity_keys
                        .public()
                        .to_bytes()
                )?,
                "For non-validator roles, the peer_id should be derived from the network identity key.",
            );
        }
        Ok(())
    }

    pub fn save(&mut self) {
        if self.is_permissioned {
            if self.network_keypairs_file.as_os_str().is_empty() {
                let file_name = format!("{}.network.keys.toml", self.peer_id.to_string());
                self.network_keypairs_file = PathBuf::from(file_name);
            }
            self.network_keypairs
                .save_config(self.network_keypairs_file());
        }

        if self.network_peers_file.as_os_str().is_empty() {
            let file_name = format!("{}.network_peers.config.toml", self.peer_id.to_string());
            self.network_peers_file = PathBuf::from(file_name);
        }
        self.network_peers.save_config(self.network_peers_file());

        if self.seed_peers_file.as_os_str().is_empty() {
            let file_name = format!("{}.seed_peers.toml", self.peer_id.to_string());
            self.seed_peers_file = PathBuf::from(file_name);
        }
        self.seed_peers.save_config(self.seed_peers_file());
    }

    pub fn random(&mut self, rng: &mut StdRng) {
        self.random_with_peer_id(rng, None);
    }

    pub fn random_with_peer_id(&mut self, rng: &mut StdRng, peer_id: Option<PeerId>) {
        let signing_key = Ed25519PrivateKey::generate_for_testing(rng);
        let identity_key = X25519StaticPrivateKey::generate_for_testing(rng);
        let network_keypairs = NetworkKeyPairs::load(signing_key, identity_key);
        self.peer_id = if let Some(peer_id) = peer_id {
            peer_id
        } else {
            PeerId::try_from(network_keypairs.identity_keys.public().to_bytes()).unwrap()
        };
        self.network_keypairs = network_keypairs;
        self.network_peers = Self::default_peers(&self.network_keypairs, &self.peer_id);
    }

    pub fn set_default_peer_id(&mut self) {
        self.peer_id =
            PeerId::try_from(self.network_keypairs.identity_keys.public().to_bytes()).unwrap();
    }

    pub fn network_peers_file(&self) -> PathBuf {
        self.base.full_path(&self.network_peers_file)
    }

    pub fn network_keypairs_file(&self) -> PathBuf {
        self.base.full_path(&self.network_keypairs_file)
    }

    pub fn seed_peers_file(&self) -> PathBuf {
        self.base.full_path(&self.seed_peers_file)
    }

    fn default_peers(keypair: &NetworkKeyPairs, peer_id: &PeerId) -> NetworkPeersConfig {
        let mut peers = NetworkPeersConfig::default();
        peers.peers.insert(
            peer_id.clone(),
            NetworkPeerInfo {
                network_identity_pubkey: keypair.identity_keys.public().clone(),
                network_signing_pubkey: keypair.signing_keys.public().clone(),
            },
        );
        peers
    }
}

// This is separated to another config so that it can be written to its own file
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SeedPeersConfig {
    // All peers config. Key:a unique peer id, will be PK in future, Value: peer discovery info
    pub seed_peers: HashMap<PeerId, Vec<Multiaddr>>,
}

// Leveraged to store the network keypairs together on disk separate from this config
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NetworkKeyPairs {
    pub signing_keys: KeyPair<Ed25519PrivateKey>,
    pub identity_keys: KeyPair<X25519StaticPrivateKey>,
}

impl NetworkKeyPairs {
    // used in testing to fill the structure with test keypairs
    pub fn load(
        signing_private_key: Ed25519PrivateKey,
        identity_private_key: X25519StaticPrivateKey,
    ) -> Self {
        Self {
            signing_keys: KeyPair::load(signing_private_key),
            identity_keys: KeyPair::load(identity_private_key),
        }
    }
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct NetworkPeersConfig {
    #[serde(flatten)]
    #[serde(serialize_with = "utils::serialize_ordered_map")]
    pub peers: HashMap<PeerId, NetworkPeerInfo>,
}

impl std::fmt::Debug for NetworkPeersConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} keys>", self.peers.len())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NetworkPeerInfo {
    #[serde(serialize_with = "keys::serialize_key")]
    #[serde(deserialize_with = "keys::deserialize_key")]
    #[serde(rename = "ns")]
    pub network_signing_pubkey: Ed25519PublicKey,
    #[serde(serialize_with = "keys::serialize_key")]
    #[serde(deserialize_with = "keys::deserialize_key")]
    #[serde(rename = "ni")]
    pub network_identity_pubkey: X25519StaticPublicKey,
}
