// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    keys::{ConsensusKeyPair, NetworkKeyPairs},
    seed_peers::SeedPeersConfigHelpers,
    trusted_peers::{ConfigHelpers, ConsensusPrivateKey, NetworkPrivateKeys},
    utils::{deserialize_whitelist, get_available_port, serialize_whitelist},
};
use failure::prelude::*;
use libra_tools::tempdir::TempPath;
use libra_types::{
    transaction::{SignedTransaction, Transaction, SCRIPT_HASH_LENGTH},
    PeerId,
};
use prost::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    convert::TryFrom,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    string::ToString,
};
use toml;

mod admission_control_config;
pub use admission_control_config::*;
mod consensus_config;
pub use consensus_config::*;
mod debug_interface_config;
pub use debug_interface_config::*;
mod execution_config;
pub use execution_config::*;
mod metrics_config;
pub use metrics_config::*;
mod network_config;
pub use network_config::*;
mod state_sync_config;
pub use state_sync_config::*;
mod storage_config;
pub use storage_config::*;
mod safety_rules_config;
pub use safety_rules_config::*;

#[cfg(test)]
#[path = "unit_tests/config_test.rs"]
mod config_test;

// path is relative to this file location
static CONFIG_TEMPLATE: &[u8] = include_bytes!("../data/configs/node.config.toml");

/// Config pulls in configuration information from the config file.
/// This is used to set up the nodes and configure various parameters.
/// The config file is broken up into sections for each module
/// so that only that module can be passed around
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
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
    pub networks: Vec<NetworkConfig>,
    #[serde(default)]
    pub consensus: ConsensusConfig,
    #[serde(default)]
    pub mempool: MempoolConfig,
    #[serde(default)]
    pub state_sync: StateSyncConfig,
    #[serde(default)]
    pub logger: LoggerConfig,
    #[serde(default)]
    pub vm_config: VMConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BaseConfig {
    pub data_dir_path: PathBuf,
    #[serde(skip)]
    temp_data_dir: Option<TempPath>,
}

impl Default for BaseConfig {
    fn default() -> BaseConfig {
        BaseConfig {
            data_dir_path: PathBuf::from("."),
            temp_data_dir: None,
        }
    }
}

impl BaseConfig {
    /// Constructs a new BaseConfig with an empty temp directory
    pub fn new(data_dir_path: PathBuf) -> Self {
        BaseConfig {
            data_dir_path,
            temp_data_dir: None,
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Clone for BaseConfig {
    fn clone(&self) -> Self {
        Self {
            data_dir_path: self.data_dir_path.clone(),
            temp_data_dir: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggerConfig {
    // Use async logging
    pub is_async: bool,
    // chan_size of slog async drain for node logging.
    pub chan_size: usize,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            is_async: true,
            chan_size: 256,
        }
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
            capacity: 1_000_000,
            capacity_per_user: 100,
            system_transaction_timeout_secs: 86400,
            address: "localhost".to_string(),
            mempool_service_port: 6182,
            system_transaction_gc_interval_ms: 180_000,
        }
    }
}

impl NodeConfig {
    /// Reads the config file and returns the configuration object in addition to doing some
    /// post-processing of the config
    /// Paths used in the config are either absolute or relative to the config location
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Self::load_config(&path);
        let mut validator_count = 0;
        for network in &mut config.networks {
            // We use provided peer id for validator role. Otherwise peer id is generated using
            // network identity key.
            if network.role == "validator" {
                assert_eq!(
                    validator_count, 0,
                    "At most 1 network config should be for a validator"
                );
                network.load(path.as_ref())?;
                validator_count += 1;
            } else {
                network.load(path.as_ref())?;
            }
        }
        config.consensus.load(path.as_ref())?;
        Ok(config)
    }

    /// Returns true if the node config is for a validator. Otherwise returns false.
    pub fn is_validator(&self) -> bool {
        self.networks
            .iter()
            .any(|network| RoleType::Validator == (&network.role).into())
    }

    /// Returns true if the node config is for a validator. Otherwise returns false.
    pub fn get_role(&self) -> RoleType {
        if self.is_validator() {
            RoleType::Validator
        } else {
            RoleType::FullNode
        }
    }

    /// Returns the validator network config for this node.
    pub fn get_validator_network_config(&self) -> Option<&NetworkConfig> {
        self.networks
            .iter()
            .filter(|network| RoleType::Validator == (&network.role).into())
            .last()
    }

    pub fn get_genesis_transaction_file(&self) -> PathBuf {
        let path = &self.execution.genesis_file_location;
        if path.is_relative() {
            self.base.data_dir_path.join(path)
        } else {
            path.clone()
        }
    }

    pub fn get_genesis_transaction(&self) -> Result<Transaction> {
        let file_path = self.get_genesis_transaction_file();
        let mut file: File = File::open(&file_path).unwrap_or_else(|err| {
            panic!(
                "Failed to open file: {:?}; error: {:?}",
                file_path.clone(),
                err
            );
        });
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        // TODO: update to use `Transaction::WriteSet` variant when ready.
        Ok(Transaction::UserTransaction(SignedTransaction::try_from(
            libra_types::proto::types::SignedTransaction::decode(&buffer)?,
        )?))
    }

    pub fn get_storage_dir(&self) -> PathBuf {
        let path = self.storage.dir.clone();
        if path.is_relative() {
            self.base.data_dir_path.join(path)
        } else {
            path
        }
    }

    pub fn get_metrics_dir(&self) -> Option<PathBuf> {
        let path = self.metrics.dir.as_path();
        if path.as_os_str().is_empty() {
            None
        } else if path.is_relative() {
            Some(self.base.data_dir_path.join(path))
        } else {
            Some(path.to_owned())
        }
    }

    /// Returns true if network_config is for an upstream network
    pub fn is_upstream_network(&self, network_config: &NetworkConfig) -> bool {
        self.state_sync
            .upstream_peers
            .upstream_peers
            .iter()
            .any(|peer_id| network_config.network_peers.peers.contains_key(peer_id))
    }

    pub fn get_upstream_peer_ids(&self) -> Vec<PeerId> {
        self.state_sync
            .upstream_peers
            .upstream_peers
            .iter()
            .map(|peer_id_str| {
                (PeerId::from_str(peer_id_str).unwrap_or_else(|_| {
                    panic!("Failed to parse peer_id from string: {}", peer_id_str)
                }))
            })
            .collect()
    }
}

pub struct NodeConfigHelpers {}

impl NodeConfigHelpers {
    /// Returns a simple test config for single node. It does not have correct network_peers_file,
    /// consensus_peers_file, network_keypairs_file, consensus_keypair_file, and seed_peers_file
    /// set. It is expected that the callee will provide these.
    pub fn get_single_node_test_config(random_ports: bool) -> NodeConfig {
        Self::get_single_node_test_config_publish_options(random_ports, None)
    }

    /// Returns a simple test config for single node. It does not have correct network_peers_file,
    /// consensus_peers_file, network_keypairs_file, consensus_keypair_file, and seed_peers_file
    /// set. It is expected that the callee will provide these.
    /// `publishing_options` is either one of either `Open` or `CustomScripts` only.
    pub fn get_single_node_test_config_publish_options(
        random_ports: bool,
        publishing_options: Option<VMPublishingOption>,
    ) -> NodeConfig {
        let config_string = String::from_utf8_lossy(CONFIG_TEMPLATE);
        let mut config =
            NodeConfig::parse(&config_string).expect("Error parsing single node test config");
        // Create temporary directory for persisting configs.
        let dir = TempPath::new();
        dir.create_as_dir().expect("error creating tempdir");
        config.base.data_dir_path = dir.path().to_owned();
        config.base.temp_data_dir = Some(dir);
        if random_ports {
            NodeConfigHelpers::randomize_config_ports(&mut config);
        }
        if let Some(vm_publishing_option) = publishing_options {
            config.vm_config.publishing_options = vm_publishing_option;
        }
        let (mut private_keys, test_consensus_peers, test_network_peers) =
            ConfigHelpers::gen_validator_nodes(1, None);
        let peer_id = *private_keys.keys().nth(0).unwrap();
        let (
            ConsensusPrivateKey {
                consensus_private_key,
            },
            NetworkPrivateKeys {
                network_signing_private_key,
                network_identity_private_key,
            },
        ) = private_keys.remove_entry(&peer_id).unwrap().1;
        config.consensus.consensus_keypair = ConsensusKeyPair::load(Some(consensus_private_key));
        config.consensus.consensus_peers = test_consensus_peers;
        // Setup node's peer id.
        let mut network = config.networks.get_mut(0).unwrap();
        network.peer_id = peer_id.to_string();
        network.network_keypairs =
            NetworkKeyPairs::load(network_signing_private_key, network_identity_private_key);
        let seed_peers_config = SeedPeersConfigHelpers::get_test_config(&test_network_peers, None);
        network.listen_address = seed_peers_config
            .seed_peers
            .get(&peer_id.to_string())
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        network.advertised_address = network.listen_address.clone();
        network.seed_peers = seed_peers_config;
        network.network_peers = test_network_peers;
        config
    }

    pub fn randomize_config_ports(config: &mut NodeConfig) {
        config.admission_control.admission_control_service_port = get_available_port();
        config.debug_interface.admission_control_node_debug_port = get_available_port();
        config.debug_interface.metrics_server_port = get_available_port();
        config.debug_interface.public_metrics_server_port = get_available_port();
        config.debug_interface.storage_node_debug_port = get_available_port();
        config.execution.port = get_available_port();
        config.mempool.mempool_service_port = get_available_port();
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
    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
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
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn empty_whitelist_FOR_TESTING() -> Self {
        VMConfig {
            publishing_options: VMPublishingOption::Locked(HashSet::new()),
        }
    }
}

pub trait PersistableConfig: Serialize + DeserializeOwned {
    // TODO: Return Result<Self> instead of panic.
    fn load_config<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut file =
            File::open(path).unwrap_or_else(|_| panic!("Cannot open config file {:?}", path));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Error reading config file {:?}", path));
        Self::parse(&contents).expect("Unable to parse config")
    }

    fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");
        let mut file = File::create(output_file).expect("Error opening file");
        file.write_all(&contents).expect("Error writing file");
    }

    fn parse(serialized: &str) -> Result<Self> {
        Ok(toml::from_str(&serialized)?)
    }
}

impl<T: ?Sized> PersistableConfig for T where T: Serialize + DeserializeOwned {}
