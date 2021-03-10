// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{Error, SecureBackend},
    keys::ConfigKey,
    network_id::NetworkId,
    utils,
};
use diem_crypto::{x25519, Uniform};
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_types::{
    network_address::NetworkAddress, transaction::authenticator::AuthenticationKey, PeerId,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    string::ToString,
};

// TODO: We could possibly move these constants somewhere else, but since they are defaults for the
//   configurations of the system, we'll leave it here for now.
/// Current supported protocol negotiation handshake version. See
/// [`network::protocols::wire::v1`](../../network/protocols/wire/handshake/v1/index.html).
pub const HANDSHAKE_VERSION: u8 = 0;
pub const NETWORK_CHANNEL_SIZE: usize = 1024;
pub const PING_INTERVAL_MS: u64 = 1000;
pub const PING_TIMEOUT_MS: u64 = 10_000;
pub const PING_FAILURES_TOLERATED: u64 = 5;
pub const CONNECTIVITY_CHECK_INTERVAL_MS: u64 = 5000;
pub const MAX_CONCURRENT_NETWORK_REQS: usize = 100;
pub const MAX_CONNECTION_DELAY_MS: u64 = 60_000; /* 1 minute */
pub const MAX_FULLNODE_OUTBOUND_CONNECTIONS: usize = 3;
pub const MAX_INBOUND_CONNECTIONS: usize = 100;
pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024; /* 8 MiB */
pub const CONNECTION_BACKOFF_BASE: u64 = 2;
pub const IP_BYTE_BUCKET_RATE: usize = 102400 /* 100 KiB */;
pub const IP_BYTE_BUCKET_SIZE: usize = IP_BYTE_BUCKET_RATE;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    // Maximum backoff delay for connecting outbound to peers
    pub max_connection_delay_ms: u64,
    // Base for outbound connection backoff
    pub connection_backoff_base: u64,
    // Rate to check connectivity to connected peers
    pub connectivity_check_interval_ms: u64,
    // Size of all network channels
    pub network_channel_size: usize,
    // Maximum number of concurrent network requests
    pub max_concurrent_network_reqs: usize,
    // Choose a protocol to discover and dial out to other peers on this network.
    // `DiscoveryMethod::None` disables discovery and dialing out (unless you have
    // seed peers configured).
    pub discovery_method: DiscoveryMethod,
    pub identity: Identity,
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: NetworkAddress,
    // Select this to enforce that both peers should authenticate each other, otherwise
    // authentication only occurs for outgoing connections.
    pub mutual_authentication: bool,
    // Used to store network address encryption keys for validator nodes
    pub network_address_key_backend: Option<SecureBackend>,
    pub network_id: NetworkId,
    // Addresses of initial peers to connect to. In a mutual_authentication network,
    // we will extract the public keys from these addresses to set our initial
    // trusted peers set.  TODO: Replace usage in configs with `seeds` this is for backwards compatibility
    pub seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    // The initial peers to connect to prior to onchain discovery
    pub seeds: PeerSet,
    // The maximum size of an inbound or outbound request frame
    pub max_frame_size: usize,
    // Enables proxy protocol on incoming connections to get original source addresses
    pub enable_proxy_protocol: bool,
    // Interval to send healthcheck pings to peers
    pub ping_interval_ms: u64,
    // Timeout until a healthcheck ping is rejected
    pub ping_timeout_ms: u64,
    // Number of failed healthcheck pings until a peer is marked unhealthy
    pub ping_failures_tolerated: u64,
    // Maximum number of outbound connections, limited by ConnectivityManager
    pub max_outbound_connections: usize,
    // Maximum number of outbound connections, limited by PeerManager
    pub max_inbound_connections: usize,
    // Inbound rate limiting configuration, if not specified, no rate limiting
    pub inbound_rate_limit_config: Option<RateLimitConfig>,
    // Outbound rate limiting configuration, if not specified, no rate limiting
    pub outbound_rate_limit_config: Option<RateLimitConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig::network_with_id(NetworkId::default())
    }
}

impl NetworkConfig {
    pub fn network_with_id(network_id: NetworkId) -> NetworkConfig {
        let mut config = Self {
            discovery_method: DiscoveryMethod::None,
            identity: Identity::None,
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            mutual_authentication: false,
            network_address_key_backend: None,
            network_id,
            seed_addrs: HashMap::new(),
            seeds: PeerSet::default(),
            max_frame_size: MAX_FRAME_SIZE,
            enable_proxy_protocol: false,
            max_connection_delay_ms: MAX_CONNECTION_DELAY_MS,
            connectivity_check_interval_ms: CONNECTIVITY_CHECK_INTERVAL_MS,
            network_channel_size: NETWORK_CHANNEL_SIZE,
            max_concurrent_network_reqs: MAX_CONCURRENT_NETWORK_REQS,
            connection_backoff_base: CONNECTION_BACKOFF_BASE,
            ping_interval_ms: PING_INTERVAL_MS,
            ping_timeout_ms: PING_TIMEOUT_MS,
            ping_failures_tolerated: PING_FAILURES_TOLERATED,
            max_outbound_connections: MAX_FULLNODE_OUTBOUND_CONNECTIONS,
            max_inbound_connections: MAX_INBOUND_CONNECTIONS,
            inbound_rate_limit_config: None,
            outbound_rate_limit_config: None,
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

    pub fn identity_from_storage(&self) -> IdentityFromStorage {
        if let Identity::FromStorage(identity) = self.identity.clone() {
            identity
        } else {
            panic!("Invalid identity found, expected a storage identity.");
        }
    }

    pub fn encryptor(&self) -> Encryptor {
        if let Some(backend) = self.network_address_key_backend.as_ref() {
            let storage = backend.into();
            Encryptor::new(storage)
        } else {
            Encryptor::for_testing()
        }
    }

    /// Per convenience, so that NetworkId isn't needed to be specified for `validator_networks`
    pub fn load_validator_network(&mut self) -> Result<(), Error> {
        self.network_id = NetworkId::Validator;
        self.load()
    }

    pub fn load_fullnode_network(&mut self) -> Result<(), Error> {
        if self.network_id.is_validator_network() {
            return Err(Error::InvariantViolation(format!(
                "Set {} network for a non-validator network",
                self.network_id
            )));
        }
        self.load()
    }

    fn load(&mut self) -> Result<(), Error> {
        if self.listen_address.to_string().is_empty() {
            self.listen_address = utils::get_local_ip()
                .ok_or_else(|| Error::InvariantViolation("No local IP".to_string()))?;
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
                    .get::<PeerId>(&config.peer_id_name)
                    .expect("Unable to read peer id")
                    .value;
                Some(peer_id)
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
                let peer_id =
                    diem_types::account_address::from_identity_public_key(key.public_key());
                self.identity = Identity::from_config(key, peer_id);
            }
            Identity::FromConfig(config) => {
                let peer_id =
                    diem_types::account_address::from_identity_public_key(config.key.public_key());
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

    fn verify_address(peer_id: &PeerId, addr: &NetworkAddress) -> Result<(), Error> {
        crate::config::invariant(
            addr.is_diemnet_addr(),
            format!(
                "Unexpected seed peer address format: peer_id: {}, addr: '{}'",
                peer_id.short_str(),
                addr,
            ),
        )
    }

    // Verifies both the `seed_addrs` and `seeds` before they're merged
    pub fn verify_seeds(&self) -> Result<(), Error> {
        for (peer_id, addrs) in self.seed_addrs.iter() {
            for addr in addrs {
                Self::verify_address(peer_id, addr)?;
            }
        }

        for (peer_id, seed) in self.seeds.iter() {
            for addr in seed.addresses.iter() {
                Self::verify_address(peer_id, addr)?;
            }

            // Require there to be a pubkey somewhere, either in the address (assumed by `is_diemnet_addr`)
            crate::config::invariant(
                !seed.keys.is_empty() || !seed.addresses.is_empty(),
                format!("Seed peer {} has no pubkeys", peer_id.short_str()),
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    Onchain,
    None,
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
    #[serde(flatten)]
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

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RateLimitConfig {
    /// Maximum number of bytes/s for an IP
    pub ip_byte_bucket_rate: usize,
    /// Maximum burst of bytes for an IP
    pub ip_byte_bucket_size: usize,
    /// Initial amount of tokens initially in the bucket
    pub initial_bucket_fill_percentage: u8,
    /// Allow for disabling the throttles
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            ip_byte_bucket_rate: IP_BYTE_BUCKET_RATE,
            ip_byte_bucket_size: IP_BYTE_BUCKET_SIZE,
            initial_bucket_fill_percentage: 25,
            enabled: true,
        }
    }
}

pub type PeerSet = HashMap<PeerId, Peer>;

// TODO: Combine with RoleType?
/// Represents the Role that a peer plays in the network ecosystem rather than the type of node.
/// Determines how nodes are connected to other nodes, and how discovery views them.
///
/// Rules for upstream nodes via Peer Role:
///
/// Validator -> Always upstream if not Validator else P2P
/// PreferredUpstream -> Always upstream, overriding any other discovery
/// ValidatorFullNode -> Always upstream for incoming connections (including other ValidatorFullNodes)
/// Upstream -> Upstream, if no ValidatorFullNode or PreferredUpstream.  Useful for initial seed discovery
/// Downstream -> Downstream, defining a controlled downstream that I always want to connect
/// Known -> A known peer, but it has no particular role assigned to it
/// Unknown -> Undiscovered peer, likely due to a non-mutually authenticated connection always downstream
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PeerRole {
    Validator = 0,
    PreferredUpstream,
    ValidatorFullNode,
    Upstream,
    Downstream,
    Known,
    Unknown,
}

impl Default for PeerRole {
    /// Default to least trusted
    fn default() -> Self {
        PeerRole::Unknown
    }
}

/// Represents a single seed configuration for a seed peer
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Peer {
    pub addresses: Vec<NetworkAddress>,
    pub keys: HashSet<x25519::PublicKey>,
    pub role: PeerRole,
}

impl Peer {
    /// Combines `Vec<NetworkAddress>` keys with the `HashSet` given
    pub fn new(
        addresses: Vec<NetworkAddress>,
        mut keys: HashSet<x25519::PublicKey>,
        role: PeerRole,
    ) -> Peer {
        let addr_keys = addresses
            .iter()
            .filter_map(NetworkAddress::find_noise_proto);
        keys.extend(addr_keys);
        Peer {
            addresses,
            keys,
            role,
        }
    }

    /// Combines two `Peer`.  Note: Does not merge duplicate addresses
    /// TODO: Instead of rejecting, maybe pick one of the roles?
    pub fn extend(&mut self, other: Peer) -> Result<(), Error> {
        crate::config::invariant(
            self.role != other.role,
            format!(
                "Roles don't match self {:?} vs other {:?}",
                self.role, other.role
            ),
        )?;
        self.addresses.extend(other.addresses);
        self.keys.extend(other.keys);
        Ok(())
    }

    pub fn from_addrs(role: PeerRole, addresses: Vec<NetworkAddress>) -> Peer {
        let keys: HashSet<x25519::PublicKey> = addresses
            .iter()
            .filter_map(NetworkAddress::find_noise_proto)
            .collect();
        Peer::new(addresses, keys, role)
    }
}
