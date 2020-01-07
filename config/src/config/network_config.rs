// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{PersistableConfig, RoleType, RootPath},
    keys::{self, KeyPair},
    utils,
};
use anyhow::{anyhow, ensure, Result};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
    Uniform, ValidKey,
};
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, path::PathBuf, string::ToString};

const NETWORK_KEYPAIRS_DEFAULT: &str = "network.keys.toml";
const NETWORK_PEERS_DEFAULT: &str = "network_peers.config.toml";
const SEED_PEERS_DEFAULT: &str = "seed_peers.toml";

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
        }
    }

    pub fn load(&mut self, root_dir: &RootPath, network_role: RoleType) -> Result<()> {
        if !self.network_peers_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.network_peers_file);
            self.network_peers = NetworkPeersConfig::load_config(&path)?;
        }
        if !self.network_keypairs_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.network_keypairs_file);
            self.network_keypairs = NetworkKeyPairs::load_config(&path)?;
        }
        if !self.seed_peers_file.as_os_str().is_empty() {
            let path = root_dir.full_path(&self.seed_peers_file);
            self.seed_peers = SeedPeersConfig::load_config(&path)?;
        }
        if self.advertised_address.to_string().is_empty() {
            self.advertised_address =
                utils::get_local_ip().ok_or_else(|| anyhow!("No local IP"))?;
        }
        if self.listen_address.to_string().is_empty() {
            self.listen_address = utils::get_local_ip().ok_or_else(|| anyhow!("No local IP"))?;
        }

        let keypair = NetworkKeyPairs::default();
        let default_peer_id = PeerId::try_from(keypair.identity_keys.public().to_bytes()).unwrap();
        let identity_key = self.network_keypairs.identity_keys.public();
        let key_peer_id = PeerId::try_from(identity_key.to_bytes()).unwrap();

        // If PeerId is not set, it is derived from NetworkIdentityKey.
        if self.peer_id == default_peer_id {
            self.peer_id = key_peer_id;
        }

        ensure!(
            network_role.is_validator() || !self.is_permissioned || self.peer_id == key_peer_id,
            "For permissioned, full-node networks, the peer_id should be derived from the identity key.",
        );
        Ok(())
    }

    fn default_path(&self, config_path: &str) -> String {
        format!("{}.{}", self.peer_id.to_string(), config_path)
    }

    pub fn save(&mut self, root_dir: &RootPath) -> Result<()> {
        if self.is_permissioned {
            if self.network_keypairs_file.as_os_str().is_empty() {
                let file_name = self.default_path(NETWORK_KEYPAIRS_DEFAULT);
                self.network_keypairs_file = PathBuf::from(file_name);
            }
            let path = root_dir.full_path(&self.network_keypairs_file);
            self.network_keypairs.save_config(&path)?;
        }

        if self.network_peers_file.as_os_str().is_empty() {
            let file_name = self.default_path(NETWORK_PEERS_DEFAULT);
            self.network_peers_file = PathBuf::from(file_name);
        }
        let path = root_dir.full_path(&self.network_peers_file);
        self.network_peers.save_config(&path)?;

        if self.seed_peers_file.as_os_str().is_empty() {
            let file_name = self.default_path(SEED_PEERS_DEFAULT);
            self.seed_peers_file = PathBuf::from(file_name);
        }
        let path = root_dir.full_path(&self.seed_peers_file);
        self.seed_peers.save_config(&path)?;
        Ok(())
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

    fn default_peers(keypair: &NetworkKeyPairs, peer_id: &PeerId) -> NetworkPeersConfig {
        let mut peers = NetworkPeersConfig::default();
        peers.peers.insert(
            peer_id.clone(),
            NetworkPeerInfo {
                identity_public_key: keypair.identity_keys.public().clone(),
                signing_public_key: keypair.signing_keys.public().clone(),
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

    pub fn as_peer_info(&self) -> NetworkPeerInfo {
        NetworkPeerInfo {
            signing_public_key: self.signing_keys.public().clone(),
            identity_public_key: self.identity_keys.public().clone(),
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
    pub signing_public_key: Ed25519PublicKey,
    #[serde(serialize_with = "keys::serialize_key")]
    #[serde(deserialize_with = "keys::deserialize_key")]
    #[serde(rename = "ni")]
    pub identity_public_key: X25519StaticPublicKey,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::RoleType;
    use libra_tools::tempdir::TempPath;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_with_defaults() {
        let keypair = NetworkKeyPairs::default();
        let peer_id = PeerId::try_from(keypair.identity_keys.public().to_bytes()).unwrap();
        let peers = NetworkConfig::default_peers(&keypair, &peer_id);

        // Assert default exists
        let (mut config, path) = generate_config();
        assert_eq!(config.network_peers, peers);
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.network_keypairs, NetworkKeyPairs::default());
        assert_eq!(config.network_keypairs_file, PathBuf::new());
        assert_eq!(config.seed_peers, SeedPeersConfig::default());
        assert_eq!(config.seed_peers_file, PathBuf::new());

        // Assert default loading doesn't affect paths and defaults remain in place
        let root_dir = RootPath::new_path(path.path());
        config.load(&root_dir, RoleType::Validator).unwrap();
        assert_eq!(config.network_peers, peers);
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.network_keypairs, NetworkKeyPairs::default());
        assert_eq!(config.network_keypairs_file, PathBuf::new());
        assert_eq!(config.seed_peers_file, PathBuf::new());
        assert_eq!(config.seed_peers, SeedPeersConfig::default());

        // Assert saving updates paths
        config.save(&root_dir).unwrap();
        assert_eq!(config.network_peers, peers);
        assert_eq!(
            config.network_keypairs_file,
            PathBuf::from(config.default_path(NETWORK_KEYPAIRS_DEFAULT))
        );
        assert_eq!(
            config.network_peers_file,
            PathBuf::from(config.default_path(NETWORK_PEERS_DEFAULT))
        );
        assert_eq!(config.network_keypairs, NetworkKeyPairs::default());
        assert_eq!(config.seed_peers, SeedPeersConfig::default());
        assert_eq!(
            config.seed_peers_file,
            PathBuf::from(config.default_path(SEED_PEERS_DEFAULT))
        );
    }

    #[test]
    fn test_with_random() {
        let (mut config, path) = generate_config();
        let mut rng = StdRng::from_seed([5u8; 32]);
        config.random(&mut rng);
        // This is default (empty) otherwise
        config.seed_peers.seed_peers.insert(config.peer_id, vec![]);

        let keypairs = config.network_keypairs.clone();
        let peers = config.network_peers.clone();
        let seed_peers = config.seed_peers.clone();

        // Assert empty paths
        assert_eq!(config.network_keypairs_file, PathBuf::new());
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.seed_peers_file, PathBuf::new());

        // Assert saving updates paths
        let root_dir = RootPath::new_path(path.path());
        config.save(&root_dir).unwrap();
        assert_eq!(config.network_keypairs, keypairs);
        assert_eq!(
            config.network_keypairs_file,
            PathBuf::from(config.default_path(NETWORK_KEYPAIRS_DEFAULT))
        );
        assert_eq!(config.network_peers, peers);
        assert_eq!(
            config.network_peers_file,
            PathBuf::from(config.default_path(NETWORK_PEERS_DEFAULT))
        );
        assert_eq!(config.seed_peers, seed_peers);
        assert_eq!(
            config.seed_peers_file,
            PathBuf::from(config.default_path(SEED_PEERS_DEFAULT))
        );

        // Assert a fresh load correctly populates the config
        let mut new_config = NetworkConfig::default();
        new_config.peer_id = config.peer_id;
        // First that paths are empty
        assert_eq!(new_config.network_keypairs_file, PathBuf::new());
        assert_eq!(new_config.network_peers_file, PathBuf::new());
        assert_eq!(new_config.seed_peers_file, PathBuf::new());
        // Loading populates things correctly
        let result = new_config.load(&root_dir, RoleType::Validator);
        assert!(result.is_ok());
        assert_eq!(config.network_keypairs, keypairs);
        assert_eq!(
            config.network_keypairs_file,
            PathBuf::from(config.default_path(NETWORK_KEYPAIRS_DEFAULT))
        );
        assert_eq!(config.network_peers, peers);
        assert_eq!(
            config.network_peers_file,
            PathBuf::from(config.default_path(NETWORK_PEERS_DEFAULT))
        );
        assert_eq!(config.seed_peers, seed_peers);
        assert_eq!(
            config.seed_peers_file,
            PathBuf::from(config.default_path(SEED_PEERS_DEFAULT))
        );
    }

    #[test]
    fn test_default_peer_id() {
        // Generate a random node and verify a distinct peer id
        let (mut config, path) = generate_config();
        let mut rng = StdRng::from_seed([32u8; 32]);
        config.random(&mut rng);
        let root_dir = RootPath::new_path(path.path());

        let keypair = NetworkKeyPairs::default();
        let default_peer_id = PeerId::try_from(keypair.identity_keys.public().to_bytes()).unwrap();
        let actual_peer_id = config.peer_id;
        assert!(actual_peer_id != default_peer_id);

        // Now reset and save
        config.peer_id = default_peer_id;
        config.save(&root_dir).unwrap();

        // Now load and verify the distinct peer id
        assert_eq!(config.peer_id, default_peer_id);
        config.load(&root_dir, RoleType::FullNode).unwrap();
        assert_eq!(config.peer_id, actual_peer_id);
    }

    fn generate_config() -> (NetworkConfig, TempPath) {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        let network_config = NetworkConfig::default();
        (network_config, temp_dir)
    }
}
