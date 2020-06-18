// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{RoleType, SecureBackend},
    keys::KeyPair,
    network_id::NetworkId,
    utils,
};
use anyhow::{anyhow, ensure, Result};
use libra_crypto::{x25519, Uniform};
use libra_network_address::NetworkAddress;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use libra_types::{transaction::authenticator::AuthenticationKey, PeerId};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    string::ToString,
};

/// Current supported protocol negotiation handshake version.
///
/// See [`perform_handshake`] in `network/src/transport.rs`
// TODO(philiphayes): ideally this constant lives somewhere in network/ ...
// might need to extract into a separate network_constants crate or something.
pub const HANDSHAKE_VERSION: u8 = 0;

pub type NetworkPeersConfig = HashMap<PeerId, x25519::PublicKey>;
pub type SeedPeersConfig = HashMap<PeerId, Vec<NetworkAddress>>;

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: NetworkAddress,
    pub connectivity_check_interval_ms: u64,
    // Select this to enforce that both peers should authenticate each other, otherwise
    // authentication only occurs for outgoing connections.
    pub mutual_authentication: bool,
    // Leveraged by mutual_authentication for incoming peers that may not have a well-defined
    // network address.
    pub network_peers: NetworkPeersConfig,
    // Initial set of peers to connect to
    pub seed_peers: SeedPeersConfig,
    // Enable this network to use either gossip discovery or onchain discovery.
    pub discovery_method: DiscoveryMethod,
    pub identity: Identity,
    pub network_id: NetworkId,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig::network_with_id(NetworkId::default())
    }
}

impl NetworkConfig {
    pub fn network_with_id(network_id: NetworkId) -> NetworkConfig {
        let mut config = Self {
            network_id,
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            connectivity_check_interval_ms: 5000,
            mutual_authentication: false,
            discovery_method: DiscoveryMethod::None,
            identity: Identity::None,
            network_peers: HashMap::default(),
            seed_peers: HashMap::default(),
        };
        config.prepare_identity();
        config
    }
}

impl NetworkConfig {
    /// This clones the underlying data except for the key so that this config can be used as a
    /// template for another config.
    pub fn clone_for_template(&self) -> Self {
        Self {
            network_id: self.network_id.clone(),
            listen_address: self.listen_address.clone(),
            connectivity_check_interval_ms: self.connectivity_check_interval_ms,
            mutual_authentication: self.mutual_authentication,
            discovery_method: self.discovery_method.clone(),
            identity: Identity::None,
            network_peers: self.network_peers.clone(),
            seed_peers: self.seed_peers.clone(),
        }
    }

    pub fn identity_key(&mut self) -> x25519::PrivateKey {
        let key = match &mut self.identity {
            Identity::FromConfig(config) => config.keypair.take_private(),
            Identity::FromStorage(config) => {
                let storage: Storage = (&config.backend).into();
                let key = storage
                    .export_private_key(&config.key_name)
                    .expect("Unable to read key");
                let key = x25519::PrivateKey::from_ed25519_private_bytes(&key.to_bytes())
                    .expect("Unable to convert key");
                Some(key)
            }
            Identity::None => None,
        };
        key.expect("identity key should be present")
    }

    pub fn load(&mut self, network_role: RoleType) -> Result<()> {
        if self.listen_address.to_string().is_empty() {
            self.listen_address = utils::get_local_ip().ok_or_else(|| anyhow!("No local IP"))?;
        }

        if network_role.is_validator() {
            ensure!(
                self.network_peers.is_empty(),
                "Validators should not define network_peers"
            );
        }

        self.prepare_identity();
        Ok(())
    }

    pub fn peer_id(&self) -> PeerId {
        match &self.identity {
            Identity::FromConfig(config) => Some(config.peer_id),
            Identity::FromStorage(config) => {
                let storage: Storage = (&config.backend).into();
                let peer_id = storage
                    .get(&config.peer_id_name)
                    .expect("Unable to read peer id")
                    .value
                    .string()
                    .expect("Expected string for peer id");
                Some(peer_id.try_into().expect("Unable to parse peer id"))
            }
            Identity::None => None,
        }
        .expect("peer id should be present")
    }

    fn prepare_identity(&mut self) {
        match &mut self.identity {
            Identity::FromStorage(_) => (),
            Identity::None => {
                let mut rng = StdRng::from_seed(OsRng.gen());
                let key = x25519::PrivateKey::generate(&mut rng);
                let peer_id = AuthenticationKey::try_from(key.public_key().as_slice())
                    .unwrap()
                    .derived_address();
                self.identity = Identity::from_config(key, peer_id);
            }
            Identity::FromConfig(config) => {
                let pubkey = config.keypair.public_key();
                let peer_id = AuthenticationKey::try_from(pubkey.as_slice())
                    .unwrap()
                    .derived_address();

                if config.peer_id == PeerId::ZERO {
                    config.peer_id = peer_id;
                }
            }
        };
    }

    pub fn random(&mut self, rng: &mut StdRng) {
        self.random_with_peer_id(rng, None);
    }

    pub fn random_with_peer_id(&mut self, rng: &mut StdRng, peer_id: Option<PeerId>) {
        let identity_key = x25519::PrivateKey::generate(rng);
        let peer_id = if let Some(peer_id) = peer_id {
            peer_id
        } else {
            AuthenticationKey::try_from(identity_key.public_key().as_slice())
                .unwrap()
                .derived_address()
        };
        self.identity = Identity::from_config(identity_key, peer_id);
    }

    /// Check that all seed peer addresses look like canonical LibraNet addresses
    pub fn verify_seed_peer_addrs(&self) -> Result<()> {
        for (peer_id, addrs) in self.seed_peers.iter() {
            for addr in addrs {
                ensure!(
                    addr.is_libranet_addr(),
                    "Unexpected seed peer address format: peer_id: {}, addr: '{}'",
                    peer_id.short_str(),
                    addr,
                );
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    // default until we can deprecate
    Gossip(GossipConfig),
    Onchain,
    None,
}

impl DiscoveryMethod {
    pub fn gossip(advertised_address: NetworkAddress) -> Self {
        DiscoveryMethod::Gossip(GossipConfig {
            advertised_address,
            discovery_interval_ms: 1000,
        })
    }

    pub fn advertised_address(&self) -> NetworkAddress {
        if let DiscoveryMethod::Gossip(config) = self {
            config.advertised_address.clone()
        } else {
            panic!("Invalid discovery method");
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GossipConfig {
    // The address that this node advertises to other nodes for the discovery protocol.
    pub advertised_address: NetworkAddress,
    pub discovery_interval_ms: u64,
}

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Identity {
    FromConfig(IdentityFromConfig),
    FromStorage(IdentityFromStorage),
    None,
}

impl Identity {
    pub fn from_config(key: x25519::PrivateKey, peer_id: PeerId) -> Self {
        let keypair = KeyPair::load(key);
        Identity::FromConfig(IdentityFromConfig { keypair, peer_id })
    }

    pub fn from_storage(key_name: String, peer_id_name: String, backend: SecureBackend) -> Self {
        Identity::FromStorage(IdentityFromStorage {
            key_name,
            peer_id_name,
            backend,
        })
    }

    pub fn public_key_from_config(&self) -> Option<x25519::PublicKey> {
        if let Identity::FromConfig(config) = self {
            Some(config.keypair.public_key())
        } else {
            None
        }
    }
}

/// The identity is stored within the config.
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
pub struct IdentityFromConfig {
    #[serde(rename = "key")]
    pub keypair: KeyPair<x25519::PrivateKey>,
    pub peer_id: PeerId,
}

/// This represents an identity in a secure-storage as defined in NodeConfig::secure.
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone, PartialEq))]
#[derive(Debug, Deserialize, Serialize)]
pub struct IdentityFromStorage {
    pub key_name: String,
    pub peer_id_name: String,
    pub backend: SecureBackend,
}
