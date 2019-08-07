// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{deserialize_whitelist, get_local_ip, serialize_whitelist};
use parity_multiaddr::{Multiaddr, Protocol};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    string::ToString,
};

use logger::LoggerType;
use nextgen_crypto::{
    ed25519::*,
    test_utils::TEST_SEED,
    x25519::{self, X25519StaticPrivateKey, X25519StaticPublicKey},
};
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use toml;

use failure::prelude::*;
use proto_conv::FromProtoBytes;
use types::transaction::{SignedTransaction, SCRIPT_HASH_LENGTH};

use crate::{
    config::ConsensusProposerType::{FixedProposer, RotatingProposer},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{
        deserialize_key, deserialize_legacy_key, deserialize_opt_key, serialize_key,
        serialize_legacy_key, serialize_opt_key, TrustedPeerPrivateKeys, TrustedPeersConfig,
        TrustedPeersConfigHelpers,
    },
    utils::get_available_port,
};

#[cfg(test)]
#[path = "unit_tests/config_test.rs"]
mod config_test;

pub const DISPOSABLE_DIR_MARKER: &str = "<USE_TEMP_DIR>";

// path is relative to this file location
static CONFIG_TEMPLATE: &[u8] = include_bytes!("../data/configs/node.config.toml");

/// Config pulls in configuration information from the config file.
/// This is used to set up the nodes and configure various parameters.
/// The config file is broken up into sections for each module
/// so that only that module can be passed around
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Clone))]
pub struct NodeConfig {
    //TODO Add configuration for multiple chain's in a future diff
    #[serde(default)]
    pub base: BaseConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub admission_control: AdmissionControlConfig,
    #[serde(default)]
    pub debug_interface: DebugInterfaceConfig,

    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub consensus: ConsensusConfig,
    #[serde(default)]
    pub mempool: MempoolConfig,
    #[serde(default)]
    pub state_sync: StateSyncConfig,
    #[serde(default)]
    pub log_collector: LoggerConfig,
    #[serde(default)]
    pub vm_config: VMConfig,

    #[serde(default)]
    pub secret_service: SecretServiceConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BaseConfig {
    pub peer_id: String,
    pub role: String,
    // peer_keypairs contains all the node's private keys,
    // it is filled later on from a different file
    #[serde(skip)]
    pub peer_keypairs: KeyPairs,
    // peer_keypairs_file contains the configuration file containing all the node's private keys.
    pub peer_keypairs_file: PathBuf,
    pub data_dir_path: PathBuf,
    #[serde(skip)]
    temp_data_dir: Option<TempDir>,
    //TODO move set of trusted peers into genesis file
    pub trusted_peers_file: String,
    #[serde(skip)]
    pub trusted_peers: TrustedPeersConfig,

    // Size of chunks to request when performing restart sync to catchup
    pub node_sync_batch_size: u64,

    // Number of retries per chunk download
    pub node_sync_retries: usize,

    // Buffer size for sync_channel used for node syncing (number of elements that it can
    // hold before it blocks on sends)
    pub node_sync_channel_buffer_size: u64,

    // chan_size of slog async drain for node logging.
    pub node_async_log_chan_size: usize,
}

impl Default for BaseConfig {
    fn default() -> BaseConfig {
        BaseConfig {
            peer_id: "".to_string(),
            role: "validator".to_string(),
            peer_keypairs_file: PathBuf::from("peer_keypairs.config.toml"),
            peer_keypairs: KeyPairs::default(),
            data_dir_path: PathBuf::from("<USE_TEMP_DIR>"),
            temp_data_dir: None,
            trusted_peers_file: "trusted_peers.config.toml".to_string(),
            trusted_peers: TrustedPeersConfig::default(),
            node_sync_batch_size: 1000,
            node_sync_retries: 7,
            node_sync_channel_buffer_size: 10,
            node_async_log_chan_size: 256,
        }
    }
}

// KeyPairs is used to store all of a node's private keys.
// It is filled via a config file at the moment.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Clone))]
pub struct KeyPairs {
    #[serde(serialize_with = "serialize_opt_key")]
    #[serde(deserialize_with = "deserialize_opt_key")]
    network_signing_private_key: Option<Ed25519PrivateKey>,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    network_signing_public_key: Ed25519PublicKey,

    #[serde(serialize_with = "serialize_legacy_key")]
    #[serde(deserialize_with = "deserialize_legacy_key")]
    network_identity_private_key: X25519StaticPrivateKey,
    #[serde(serialize_with = "serialize_legacy_key")]
    #[serde(deserialize_with = "deserialize_legacy_key")]
    network_identity_public_key: X25519StaticPublicKey,

    #[serde(serialize_with = "serialize_opt_key")]
    #[serde(deserialize_with = "deserialize_opt_key")]
    consensus_private_key: Option<Ed25519PrivateKey>,
    #[serde(serialize_with = "serialize_key")]
    #[serde(deserialize_with = "deserialize_key")]
    consensus_public_key: Ed25519PublicKey,
}

// required for serialization
impl Default for KeyPairs {
    fn default() -> Self {
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (net_private_sig, net_public_sig) = compat::generate_keypair(&mut rng);
        let (consensus_private_sig, consensus_public_sig) = compat::generate_keypair(&mut rng);
        let (private_kex, public_kex) = x25519::compat::generate_keypair(&mut rng);
        Self {
            network_signing_private_key: Some(net_private_sig),
            network_signing_public_key: net_public_sig,
            network_identity_private_key: private_kex,
            network_identity_public_key: public_kex,
            consensus_private_key: Some(consensus_private_sig),
            consensus_public_key: consensus_public_sig,
        }
    }
}

impl KeyPairs {
    // used to deserialize keypairs from a configuration file
    pub fn load_config<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut file = File::open(path)
            .unwrap_or_else(|_| panic!("Cannot open KeyPair Config file {:?}", path));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Error reading KeyPair Config file {:?}", path));

        Self::parse(&contents)
    }
    fn parse(config_string: &str) -> Self {
        toml::from_str(config_string).expect("Unable to parse Config")
    }
    // used to serialize keypairs to a configuration file
    pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");

        let mut file = File::create(output_file).expect("Error opening file");

        file.write_all(&contents).expect("Error writing file");
    }

    // used in testing to fill the structure with test keypairs
    pub fn load(private_keys: TrustedPeerPrivateKeys) -> Self {
        let (network_signing_private_key, network_identity_private_key, consensus_private_key) =
            private_keys.get_key_triplet();
        let network_signing_public_key = (&network_signing_private_key).into();
        let network_identity_public_key = (&network_identity_private_key).into();
        let consensus_public_key = (&consensus_private_key).into();
        Self {
            network_signing_private_key: Some(network_signing_private_key),
            network_signing_public_key,
            network_identity_private_key,
            network_identity_public_key,
            consensus_private_key: Some(consensus_private_key),
            consensus_public_key,
        }
    }
    // getters for private keys
    /// Beware, this destroys the private key from this NodeConfig
    pub fn take_network_signing_private(&mut self) -> Option<Ed25519PrivateKey> {
        std::mem::replace(&mut self.network_signing_private_key, None)
    }
    pub fn get_network_signing_private(&self) -> &Option<Ed25519PrivateKey> {
        &self.network_signing_private_key
    }
    pub fn get_network_identity_private(&self) -> X25519StaticPrivateKey {
        self.network_identity_private_key.clone()
    }
    pub fn get_consensus_private(&self) -> &Option<Ed25519PrivateKey> {
        &self.consensus_private_key
    }

    /// Beware, this destroys the private key from this NodeConfig
    pub fn take_consensus_private(&mut self) -> Option<Ed25519PrivateKey> {
        std::mem::replace(&mut self.consensus_private_key, None)
    }
    // getters for public keys
    pub fn get_network_signing_public(&self) -> &Ed25519PublicKey {
        &self.network_signing_public_key
    }
    pub fn get_network_identity_public(&self) -> &X25519StaticPublicKey {
        &self.network_identity_public_key
    }
    pub fn get_consensus_public(&self) -> &Ed25519PublicKey {
        &self.consensus_public_key
    }
    // getters for keypairs
    pub fn get_network_identity_keypair(&self) -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
        (
            self.get_network_identity_private(),
            self.get_network_identity_public().clone(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RoleType {
    Validator,
    FullNode,
}

impl BaseConfig {
    /// Constructs a new BaseConfig with an empty temp directory
    pub fn new(
        peer_id: String,
        role: String,
        peer_keypairs: KeyPairs,
        peer_keypairs_file: PathBuf,
        data_dir_path: PathBuf,
        trusted_peers_file: String,
        trusted_peers: TrustedPeersConfig,

        node_sync_batch_size: u64,

        node_sync_retries: usize,

        node_sync_channel_buffer_size: u64,

        node_async_log_chan_size: usize,
    ) -> Self {
        BaseConfig {
            peer_id,
            role,
            peer_keypairs,
            peer_keypairs_file,
            data_dir_path,
            temp_data_dir: None,
            trusted_peers_file,
            trusted_peers,
            node_sync_batch_size,
            node_sync_retries,
            node_sync_channel_buffer_size,
            node_async_log_chan_size,
        }
    }

    pub fn get_role(&self) -> RoleType {
        match self.role.as_str() {
            "validator" => RoleType::Validator,
            "full_node" => RoleType::FullNode,
            &_ => unimplemented!("Invalid node role: {}", self.role),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Clone for BaseConfig {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id.clone(),
            role: self.role.clone(),
            peer_keypairs: self.peer_keypairs.clone(),
            peer_keypairs_file: self.peer_keypairs_file.clone(),
            data_dir_path: self.data_dir_path.clone(),
            temp_data_dir: None,
            trusted_peers_file: self.trusted_peers_file.clone(),
            trusted_peers: self.trusted_peers.clone(),
            node_sync_batch_size: self.node_sync_batch_size,
            node_sync_retries: self.node_sync_retries,
            node_sync_channel_buffer_size: self.node_sync_channel_buffer_size,
            node_async_log_chan_size: self.node_async_log_chan_size,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MetricsConfig {
    pub dir: PathBuf,
    pub collection_interval_ms: u64,
    pub push_server_addr: String,
}

impl Default for MetricsConfig {
    fn default() -> MetricsConfig {
        MetricsConfig {
            dir: PathBuf::from("metrics"),
            collection_interval_ms: 1000,
            push_server_addr: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ExecutionConfig {
    pub address: String,
    pub port: u16,
    // directive to load the testnet genesis block or the default genesis block.
    // There are semantic differences between the 2 genesis related to minting and
    // account creation
    pub testnet_genesis: bool,
    pub genesis_file_location: String,
}

impl Default for ExecutionConfig {
    fn default() -> ExecutionConfig {
        ExecutionConfig {
            address: "localhost".to_string(),
            port: 6183,
            testnet_genesis: false,
            genesis_file_location: "genesis.blob".to_string(),
        }
    }
}

impl ExecutionConfig {
    pub fn get_genesis_transaction(&self) -> Result<SignedTransaction> {
        let mut file = File::open(self.genesis_file_location.clone())?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        SignedTransaction::from_proto_bytes(&buffer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggerConfig {
    pub http_endpoint: Option<String>,
    pub is_async: bool,
    pub chan_size: Option<usize>,
    pub use_std_output: bool,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            http_endpoint: None,
            is_async: true,
            chan_size: None,
            use_std_output: true,
        }
    }
}

impl LoggerConfig {
    pub fn get_log_collector_type(&self) -> Option<LoggerType> {
        // There is priority between different logger. If multiple ones are specified, only
        // the higher one will be returned.
        if self.http_endpoint.is_some() {
            return Some(LoggerType::Http(
                self.http_endpoint
                    .clone()
                    .expect("Http endpoint not available for logger"),
            ));
        } else if self.use_std_output {
            return Some(LoggerType::StdOutput);
        }
        None
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SecretServiceConfig {
    pub address: String,
    pub secret_service_port: u16,
}

impl Default for SecretServiceConfig {
    fn default() -> SecretServiceConfig {
        SecretServiceConfig {
            address: "localhost".to_string(),
            secret_service_port: 6185,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AdmissionControlConfig {
    pub address: String,
    pub admission_control_service_port: u16,
    pub need_to_check_mempool_before_validation: bool,
}

impl Default for AdmissionControlConfig {
    fn default() -> AdmissionControlConfig {
        AdmissionControlConfig {
            address: "0.0.0.0".to_string(),
            admission_control_service_port: 8000,
            need_to_check_mempool_before_validation: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DebugInterfaceConfig {
    pub admission_control_node_debug_port: u16,
    pub secret_service_node_debug_port: u16,
    pub storage_node_debug_port: u16,
    // This has similar use to the core-node-debug-server itself
    pub metrics_server_port: u16,
    pub address: String,
}

impl Default for DebugInterfaceConfig {
    fn default() -> DebugInterfaceConfig {
        DebugInterfaceConfig {
            admission_control_node_debug_port: 6191,
            storage_node_debug_port: 6194,
            secret_service_node_debug_port: 6195,
            metrics_server_port: 9101,
            address: "localhost".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    pub address: String,
    pub port: u16,
    pub dir: PathBuf,
    pub grpc_max_receive_len: Option<i32>,
}

impl StorageConfig {
    pub fn get_dir(&self) -> &Path {
        &self.dir
    }
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        StorageConfig {
            address: "localhost".to_string(),
            port: 6184,
            dir: PathBuf::from("libradb"),
            grpc_max_receive_len: Some(100_000_000),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub seed_peers_file: String,
    #[serde(skip)]
    pub seed_peers: SeedPeersConfig,
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: Multiaddr,
    // The address that this node advertises to other nodes for the discovery protocol.
    pub advertised_address: Multiaddr,
    pub discovery_interval_ms: u64,
    pub connectivity_check_interval_ms: u64,
    pub enable_encryption_and_authentication: bool,
}

impl Default for NetworkConfig {
    fn default() -> NetworkConfig {
        NetworkConfig {
            seed_peers_file: "seed_peers.config.toml".to_string(),
            seed_peers: SeedPeersConfig::default(),
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse::<Multiaddr>().unwrap(),
            advertised_address: "/ip4/127.0.0.1/tcp/6180".parse::<Multiaddr>().unwrap(),
            discovery_interval_ms: 1000,
            connectivity_check_interval_ms: 5000,
            enable_encryption_and_authentication: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsensusConfig {
    max_block_size: u64,
    proposer_type: String,
    contiguous_rounds: u32,
    max_pruned_blocks_in_mem: Option<u64>,
    pacemaker_initial_timeout_ms: Option<u64>,
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        ConsensusConfig {
            max_block_size: 100,
            proposer_type: "rotating_proposer".to_string(),
            contiguous_rounds: 2,
            max_pruned_blocks_in_mem: None,
            pacemaker_initial_timeout_ms: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConsensusProposerType {
    // Choose the smallest PeerId as the proposer
    FixedProposer,
    // Round robin rotation of proposers
    RotatingProposer,
}

impl ConsensusConfig {
    pub fn get_proposer_type(&self) -> ConsensusProposerType {
        match self.proposer_type.as_str() {
            "fixed_proposer" => FixedProposer,
            "rotating_proposer" => RotatingProposer,
            &_ => unimplemented!("Invalid proposer type: {}", self.proposer_type),
        }
    }

    pub fn contiguous_rounds(&self) -> u32 {
        self.contiguous_rounds
    }

    pub fn max_block_size(&self) -> u64 {
        self.max_block_size
    }

    pub fn max_pruned_blocks_in_mem(&self) -> &Option<u64> {
        &self.max_pruned_blocks_in_mem
    }

    pub fn pacemaker_initial_timeout_ms(&self) -> &Option<u64> {
        &self.pacemaker_initial_timeout_ms
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MempoolConfig {
    pub broadcast_transactions: bool,
    pub shared_mempool_tick_interval_ms: u64,
    pub shared_mempool_batch_size: usize,
    pub shared_mempool_max_concurrent_inbound_syncs: usize,
    pub capacity: usize,
    // max number of transactions per user in Mempool
    pub capacity_per_user: usize,
    pub sequence_cache_capacity: usize,
    pub system_transaction_timeout_secs: u64,
    pub system_transaction_gc_interval_ms: u64,
    pub mempool_service_port: u16,
    pub address: String,
}

impl Default for MempoolConfig {
    fn default() -> MempoolConfig {
        MempoolConfig {
            broadcast_transactions: true,
            shared_mempool_tick_interval_ms: 50,
            shared_mempool_batch_size: 100,
            shared_mempool_max_concurrent_inbound_syncs: 100,
            capacity: 10_000_000,
            capacity_per_user: 100,
            sequence_cache_capacity: 1000,
            system_transaction_timeout_secs: 86400,
            address: "localhost".to_string(),
            mempool_service_port: 6182,
            system_transaction_gc_interval_ms: 180_000,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct StateSyncConfig {
    pub address: String,
    pub service_port: u16,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            address: "localhost".to_string(),
            service_port: 55557,
        }
    }
}

impl NodeConfig {
    /// Reads the config file and returns the configuration object
    pub fn load_template<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut file =
            File::open(path).with_context(|_| format!("Cannot open NodeConfig file {:?}", path))?;
        let mut config_string = String::new();
        file.read_to_string(&mut config_string)
            .with_context(|_| format!("Cannot read NodeConfig file {:?}", path))?;

        let config = Self::parse(&config_string)
            .with_context(|_| format!("Cannot parse NodeConfig file {:?}", path))?;

        Ok(config)
    }

    /// Reads the config file and returns the configuration object in addition to doing some
    /// post-processing of the config
    /// Paths used in the config are either absolute or relative to the config location
    pub fn load_config<P: AsRef<Path>>(peer_id: Option<String>, path: P) -> Result<Self> {
        let mut config = Self::load_template(&path)?;
        // Allow peer_id override if set
        if let Some(peer_id) = peer_id {
            config.base.peer_id = peer_id;
        }
        if !config.base.trusted_peers_file.is_empty() {
            config.base.trusted_peers = TrustedPeersConfig::load_config(
                path.as_ref()
                    .with_file_name(&config.base.trusted_peers_file),
            );
        }
        if !config.base.peer_keypairs_file.as_os_str().is_empty() {
            config.base.peer_keypairs = KeyPairs::load_config(
                path.as_ref()
                    .with_file_name(&config.base.peer_keypairs_file),
            );
        }
        if !config.network.seed_peers_file.is_empty() {
            config.network.seed_peers = SeedPeersConfig::load_config(
                path.as_ref()
                    .with_file_name(&config.network.seed_peers_file),
            );
        }
        if config.network.advertised_address.to_string().is_empty() {
            config.network.advertised_address =
                get_local_ip().ok_or_else(|| ::failure::err_msg("No local IP"))?;
        }
        if config.network.listen_address.to_string().is_empty() {
            config.network.listen_address =
                get_local_ip().ok_or_else(|| ::failure::err_msg("No local IP"))?;
        }
        NodeConfigHelpers::update_data_dir_path_if_needed(&mut config, &path)?;
        Ok(config)
    }

    pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");
        let mut file = File::create(output_file).expect("Error opening file");

        file.write_all(&contents).expect("Error writing file");
    }

    /// Parses the config file into a Config object
    pub fn parse(config_string: &str) -> Result<Self> {
        assert!(!config_string.is_empty());
        Ok(toml::from_str(config_string)?)
    }

    /// Returns the peer info for this node
    pub fn own_addrs(&self) -> (String, Vec<Multiaddr>) {
        let own_peer_id = self.base.peer_id.clone();
        let own_addrs = vec![self.network.advertised_address.clone()];
        (own_peer_id, own_addrs)
    }
}

// Given a multiaddr, randomizes its Tcp port if present.
fn randomize_tcp_port(addr: &Multiaddr) -> Multiaddr {
    let mut new_addr = Multiaddr::empty();
    for p in addr.iter() {
        if let Protocol::Tcp(_) = p {
            new_addr.push(Protocol::Tcp(get_available_port()));
        } else {
            new_addr.push(p);
        }
    }
    new_addr
}

fn get_tcp_port(addr: &Multiaddr) -> Option<u16> {
    for p in addr.iter() {
        if let Protocol::Tcp(port) = p {
            return Some(port);
        }
    }
    None
}

pub struct NodeConfigHelpers {}

impl NodeConfigHelpers {
    /// Returns a simple test config for single node. It does not have correct trusted_peers_file,
    /// peer_keypairs_file, and seed_peers_file set and expected that callee will provide these
    pub fn get_single_node_test_config(random_ports: bool) -> NodeConfig {
        Self::get_single_node_test_config_publish_options(random_ports, None)
    }

    /// Returns a simple test config for single node. It does not have correct trusted_peers_file,
    /// peer_keypairs_file, and seed_peers_file set and expected that callee will provide these
    /// `publishing_options` is either one of either `Open` or `CustomScripts` only.
    pub fn get_single_node_test_config_publish_options(
        random_ports: bool,
        publishing_options: Option<VMPublishingOption>,
    ) -> NodeConfig {
        let config_string = String::from_utf8_lossy(CONFIG_TEMPLATE);
        let mut config =
            NodeConfig::parse(&config_string).expect("Error parsing single node test config");
        if random_ports {
            NodeConfigHelpers::randomize_config_ports(&mut config);
        }

        if let Some(vm_publishing_option) = publishing_options {
            config.vm_config.publishing_options = vm_publishing_option;
        }

        let (mut peers_private_keys, trusted_peers_test) =
            TrustedPeersConfigHelpers::get_test_config(1, None);
        let peer_id = trusted_peers_test.peers.keys().collect::<Vec<_>>()[0];
        config.base.peer_id = peer_id.clone();
        // load node's keypairs
        let private_keys = peers_private_keys.remove_entry(peer_id.as_str()).unwrap().1;
        config.base.peer_keypairs = KeyPairs::load(private_keys);
        config.base.trusted_peers = trusted_peers_test;
        config.network.seed_peers = SeedPeersConfigHelpers::get_test_config(
            &config.base.trusted_peers,
            get_tcp_port(&config.network.advertised_address),
        );
        NodeConfigHelpers::update_data_dir_path_if_needed(&mut config, ".")
            .expect("creating tempdir");
        config
    }

    /// Replaces temp marker with the actual path and returns holder to the temp dir.
    fn update_data_dir_path_if_needed<P: AsRef<Path>>(
        config: &mut NodeConfig,
        base_path: P,
    ) -> Result<()> {
        if config.base.data_dir_path == Path::new(DISPOSABLE_DIR_MARKER) {
            let dir = tempfile::tempdir().context("error creating tempdir")?;
            config.base.data_dir_path = dir.path().to_owned();
            config.base.temp_data_dir = Some(dir);
        }

        if !config.metrics.dir.as_os_str().is_empty() {
            // do not set the directory if metrics.dir is empty to honor the check done
            // in setup_metrics
            config.metrics.dir = config.base.data_dir_path.join(&config.metrics.dir);
        }
        config.storage.dir = config.base.data_dir_path.join(config.storage.get_dir());
        if config.execution.genesis_file_location == DISPOSABLE_DIR_MARKER {
            config.execution.genesis_file_location = config
                .base
                .data_dir_path
                .join("genesis.blob")
                .to_str()
                .unwrap()
                .to_string();
        }
        config.execution.genesis_file_location = base_path
            .as_ref()
            .with_file_name(&config.execution.genesis_file_location)
            .to_str()
            .unwrap()
            .to_string();

        Ok(())
    }

    pub fn randomize_config_ports(config: &mut NodeConfig) {
        config.admission_control.admission_control_service_port = get_available_port();
        config.debug_interface.admission_control_node_debug_port = get_available_port();
        config.debug_interface.metrics_server_port = get_available_port();
        config.debug_interface.secret_service_node_debug_port = get_available_port();
        config.debug_interface.storage_node_debug_port = get_available_port();
        config.execution.port = get_available_port();
        config.mempool.mempool_service_port = get_available_port();
        config.network.advertised_address = randomize_tcp_port(&config.network.advertised_address);
        config.network.listen_address = randomize_tcp_port(&config.network.listen_address);
        config.state_sync.service_port = get_available_port();
        config.secret_service.secret_service_port = get_available_port();
        config.storage.port = get_available_port();
    }
}

/// Holds the VM configuration, currently this is only the publishing options for scripts and
/// modules, but in the future this may need to be expanded to hold more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl Default for VMConfig {
    fn default() -> VMConfig {
        VMConfig {
            publishing_options: VMPublishingOption::Open,
        }
    }
}

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "whitelist")]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    #[serde(deserialize_with = "deserialize_whitelist")]
    #[serde(serialize_with = "serialize_whitelist")]
    Locked(HashSet<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl VMPublishingOption {
    pub fn custom_scripts_only(&self) -> bool {
        !self.is_open() && !self.is_locked()
    }

    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            VMPublishingOption::Locked { .. } => true,
            _ => false,
        }
    }

    pub fn get_whitelist_set(&self) -> Option<&HashSet<[u8; SCRIPT_HASH_LENGTH]>> {
        match self {
            VMPublishingOption::Locked(whitelist) => Some(&whitelist),
            _ => None,
        }
    }
}

impl VMConfig {
    /// Creates a new `VMConfig` where the whitelist is empty. This should only be used for testing.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub fn empty_whitelist_FOR_TESTING() -> Self {
        VMConfig {
            publishing_options: VMPublishingOption::Locked(HashSet::new()),
        }
    }

    pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");

        let mut file = File::create(output_file).expect("Error opening file");

        file.write_all(&contents).expect("Error writing file");
    }
}
