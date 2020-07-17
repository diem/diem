// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{Error, RoleType, SecureBackend},
    keys::ConfigKey,
    network_id::NetworkId,
    utils,
};
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
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    string::ToString,
};

/// Current supported protocol negotiation handshake version.
///
/// See [`perform_handshake`] in `network/src/transport.rs`
// TODO(philiphayes): ideally these constants live somewhere in network/ ...
// might need to extract into a separate network_constants crate or something.
pub const HANDSHAKE_VERSION: u8 = 0;
pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024;

pub type SeedPublicKeys = HashMap<PeerId, HashSet<x25519::PublicKey>>;
pub type SeedAddresses = HashMap<PeerId, Vec<NetworkAddress>>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub connectivity_check_interval_ms: u64,
    // Enable this network to use either gossip discovery or onchain discovery.
    pub discovery_method: DiscoveryMethod,
    pub identity: Identity,
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: NetworkAddress,
    // Select this to enforce that both peers should authenticate each other, otherwise
    // authentication only occurs for outgoing connections.
    pub mutual_authentication: bool,
    pub network_id: NetworkId,
    // Addresses of initial peers to connect to. In a mutual_authentication network,
    // we will extract the public keys from these addresses to set our initial
    // trusted peers set.
    pub seed_addrs: SeedAddresses,
    // Backup for public keys of peers that we'll accept connections from in a
    // mutual_authentication network. This config field is intended as a fallback
    // in case some peers don't have well defined addresses.
    pub seed_pubkeys: SeedPublicKeys,
    pub max_frame_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig::network_with_id(NetworkId::default())
    }
}

impl NetworkConfig {
    pub fn network_with_id(network_id: NetworkId) -> NetworkConfig {
        let mut config = Self {
            connectivity_check_interval_ms: 5000,
            discovery_method: DiscoveryMethod::None,
            identity: Identity::None,
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            mutual_authentication: false,
            network_id,
            seed_pubkeys: HashMap::default(),
            seed_addrs: HashMap::default(),
            max_frame_size: MAX_FRAME_SIZE,
        };
        config.prepare_identity();
        config
    }
}

impl NetworkConfig {
    pub fn identity_key(&self) -> x25519::PrivateKey {
        let key = match &self.identity {
            Identity::FromConfig(config) => Some(config.key.clone().key),
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

    pub fn load(&mut self, role: RoleType) -> Result<(), Error> {
        if self.listen_address.to_string().is_empty() {
            self.listen_address = utils::get_local_ip()
                .ok_or_else(|| Error::InvariantViolation("No local IP".to_string()))?;
        }

        if role == RoleType::Validator {
            self.network_id = NetworkId::Validator;
        } else if self.network_id == NetworkId::Validator {
            return Err(Error::InvariantViolation(
                "Set NetworkId::Validator network for a non-validator network".to_string(),
            ));
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
                let pubkey = config.key.public_key();
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
    pub fn verify_seed_addrs(&self) -> Result<(), Error> {
        for (peer_id, addrs) in self.seed_addrs.iter() {
            for addr in addrs {
                crate::config::invariant(
                    addr.is_libranet_addr(),
                    format!(
                        "Unexpected seed peer address format: peer_id: {}, addr: '{}'",
                        peer_id.short_str(),
                        addr,
                    ),
                )?;
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
#[serde(deny_unknown_fields)]
pub struct GossipConfig {
    // The address that this node advertises to other nodes for the discovery protocol.
    pub advertised_address: NetworkAddress,
    pub discovery_interval_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Identity {
    FromConfig(IdentityFromConfig),
    FromStorage(IdentityFromStorage),
    None,
}

impl Identity {
    pub fn from_config(key: x25519::PrivateKey, peer_id: PeerId) -> Self {
        let key = ConfigKey::new(key);
        Identity::FromConfig(IdentityFromConfig { key, peer_id })
    }

    pub fn from_storage(key_name: String, peer_id_name: String, backend: SecureBackend) -> Self {
        Identity::FromStorage(IdentityFromStorage {
            backend,
            key_name,
            peer_id_name,
        })
    }
}

/// The identity is stored within the config.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityFromConfig {
    pub key: ConfigKey<x25519::PrivateKey>,
    pub peer_id: PeerId,
}

/// This represents an identity in a secure-storage as defined in NodeConfig::secure.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityFromStorage {
    pub backend: SecureBackend,
    pub key_name: String,
    pub peer_id_name: String,
}
