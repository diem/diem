// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{PersistableConfig, RoleType, RootPath},
    keys::KeyPair,
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
    pub enable_noise: bool,
    // If the network uses remote authentication, only trusted peers are allowed to connect.
    // Otherwise, any node can connect. If this flag is set to true, `enable_noise` must
    // also be set to true.
    pub enable_remote_authentication: bool,
    // network peers are the nodes allowed to connect when the network is started in authenticated
    // mode.
    #[serde(skip)]
    pub network_peers: NetworkPeersConfig,
    pub network_peers_file: PathBuf,
    // seed_peers act as seed nodes for the discovery protocol.
    #[serde(skip)]
    pub seed_peers: SeedPeersConfig,
    pub seed_peers_file: PathBuf,
    pub network_keypairs: Option<NetworkKeyPairs>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            peer_id: PeerId::default(),
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse::<Multiaddr>().unwrap(),
            advertised_address: "/ip4/127.0.0.1/tcp/6180".parse::<Multiaddr>().unwrap(),
            discovery_interval_ms: 1000,
            connectivity_check_interval_ms: 5000,
            enable_noise: true,
            enable_remote_authentication: true,
            network_keypairs: None,
            network_peers_file: PathBuf::new(),
            network_peers: NetworkPeersConfig::default(),
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
            enable_noise: self.enable_noise,
            enable_remote_authentication: self.enable_remote_authentication,
            network_keypairs: None,
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

        if self.enable_remote_authentication {
            ensure!(
                self.enable_noise,
                "For a node to enforce remote authentication, noise must be enabled.",
            );
        }

        if network_role.is_validator() {
            ensure!(
                self.network_peers_file.as_os_str().is_empty(),
                "Validators should not define network_peers_file"
            );
            ensure!(
                self.network_peers.peers.is_empty(),
                "Validators should not define network_peers"
            );
        }

        // TODO(joshlind): investigate the implications of removing these checks.
        if let Some(network_keypairs) = &self.network_keypairs {
            let identity_key = network_keypairs.identity_keys.public();
            let peer_id = PeerId::try_from(identity_key.to_bytes()).unwrap();

            // If PeerId is not set, derive the PeerId from identity_key.
            if self.peer_id == PeerId::default() {
                self.peer_id = peer_id;
            }
            // Full nodes with remote authentication must derive PeerId from identity_key.
            if !network_role.is_validator() && self.enable_remote_authentication {
                ensure!(
                    self.peer_id == peer_id,
                    "For full-nodes that use remote authentication, the peer_id must be derived from the identity key.",
                );
            }
        }
        Ok(())
    }

    fn default_path(&self, config_path: &str) -> String {
        format!("{}.{}", self.peer_id.to_string(), config_path)
    }

    pub fn save(&mut self, root_dir: &RootPath) -> Result<()> {
        if self.network_peers != NetworkPeersConfig::default() {
            if self.network_peers_file.as_os_str().is_empty() {
                let file_name = self.default_path(NETWORK_PEERS_DEFAULT);
                self.network_peers_file = PathBuf::from(file_name);
            }
            let path = root_dir.full_path(&self.network_peers_file);
            self.network_peers.save_config(&path)?;
        }

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
        self.network_keypairs = Some(network_keypairs);
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
    #[serde(rename = "ns")]
    pub signing_public_key: Ed25519PublicKey,
    #[serde(rename = "ni")]
    pub identity_public_key: X25519StaticPublicKey,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::RoleType;
    use libra_temppath::TempPath;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_with_defaults() {
        // Assert default exists
        let (mut config, path) = generate_config();
        assert_eq!(config.network_peers, NetworkPeersConfig::default());
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.network_keypairs, None);
        assert_eq!(config.peer_id, PeerId::default());
        assert_eq!(config.seed_peers, SeedPeersConfig::default());
        assert_eq!(config.seed_peers_file, PathBuf::new());

        // Assert default loading doesn't affect paths and defaults remain in place
        let root_dir = RootPath::new_path(path.path());
        config.load(&root_dir, RoleType::FullNode).unwrap();
        assert_eq!(config.network_peers, NetworkPeersConfig::default());
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.network_keypairs, None);
        assert_eq!(config.peer_id, PeerId::default());
        assert_eq!(config.seed_peers_file, PathBuf::new());
        assert_eq!(config.seed_peers, SeedPeersConfig::default());

        // Assert saving updates paths
        config.save(&root_dir).unwrap();
        assert_eq!(config.seed_peers, SeedPeersConfig::default());
        assert_eq!(
            config.seed_peers_file,
            PathBuf::from(config.default_path(SEED_PEERS_DEFAULT))
        );

        // Assert paths and values are not set (i.e., no defaults apply)
        assert_eq!(config.network_keypairs, None);
        assert_eq!(config.network_peers, NetworkPeersConfig::default());
        assert_eq!(config.network_peers_file, PathBuf::new());
    }

    #[test]
    fn test_with_random() {
        let (mut config, path) = generate_config();
        config.network_peers = NetworkPeersConfig::default();
        let mut rng = StdRng::from_seed([5u8; 32]);
        config.random(&mut rng);
        // This is default (empty) otherwise
        config.seed_peers.seed_peers.insert(config.peer_id, vec![]);

        let keypairs = config.network_keypairs.clone();
        let peers = config.network_peers.clone();
        let seed_peers = config.seed_peers.clone();

        // Assert empty paths
        assert_eq!(config.network_peers_file, PathBuf::new());
        assert_eq!(config.seed_peers_file, PathBuf::new());

        // Assert saving updates paths
        let root_dir = RootPath::new_path(path.path());
        config.save(&root_dir).unwrap();
        assert_eq!(config.network_keypairs, keypairs);
        assert_eq!(config.network_peers, peers);
        assert_eq!(config.network_peers_file, PathBuf::new(),);
        assert_eq!(config.seed_peers, seed_peers);
        assert_eq!(
            config.seed_peers_file,
            PathBuf::from(config.default_path(SEED_PEERS_DEFAULT))
        );

        // Assert a fresh load correctly populates the config
        let mut new_config = NetworkConfig::default();
        new_config.peer_id = config.peer_id;
        // First that paths are empty
        assert_eq!(new_config.network_peers_file, PathBuf::new());
        assert_eq!(new_config.seed_peers_file, PathBuf::new());
        // Loading populates things correctly
        let result = new_config.load(&root_dir, RoleType::Validator);
        result.unwrap();
        assert_eq!(config.network_keypairs, keypairs);
        assert_eq!(config.network_peers, peers);
        assert_eq!(config.network_peers_file, PathBuf::new(),);
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

        let default_peer_id = PeerId::default();
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
        let mut config = NetworkConfig::default();
        config.network_peers = NetworkPeersConfig::default();
        (config, temp_dir)
    }
}
