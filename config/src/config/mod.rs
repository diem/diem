// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::prelude::*;
use libra_tools::tempdir::TempPath;
use libra_types::{
    transaction::{SignedTransaction, Transaction},
    PeerId,
};
use prost::Message;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
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
mod logger_config;
pub use logger_config::*;
mod metrics_config;
pub use metrics_config::*;
mod mempool_config;
pub use mempool_config::*;
mod network_config;
pub use network_config::*;
mod state_sync_config;
pub use state_sync_config::*;
mod storage_config;
pub use storage_config::*;
mod safety_rules_config;
pub use safety_rules_config::*;
mod vm_config;
pub use vm_config::*;

/// Config pulls in configuration information from the config file.
/// This is used to set up the nodes and configure various parameters.
/// The config file is broken up into sections for each module
/// so that only that module can be passed around
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub admission_control: AdmissionControlConfig,
    #[serde(default)]
    pub base: Arc<BaseConfig>,
    #[serde(default)]
    pub consensus: ConsensusConfig,
    #[serde(default)]
    pub debug_interface: DebugInterfaceConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub full_node_networks: Vec<NetworkConfig>,
    #[serde(default)]
    pub logger: LoggerConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub mempool: MempoolConfig,
    #[serde(default)]
    pub state_sync: StateSyncConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub validator_network: Option<NetworkConfig>,
    #[serde(default)]
    pub vm_config: VMConfig,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BaseConfig {
    pub data_dir: PathBuf,
    pub role: RoleType,
    // Used only to prevent a potentially temporary data_dir from being deleted. This should
    // eventually be moved to be owned by something outside the config.
    #[serde(skip)]
    temp_dir: Option<TempPath>,
}

impl Clone for BaseConfig {
    fn clone(&self) -> Self {
        Self {
            data_dir: self.data_dir.clone(),
            role: self.role,
            temp_dir: None,
        }
    }
}

impl Default for BaseConfig {
    fn default() -> BaseConfig {
        BaseConfig {
            data_dir: PathBuf::from("."),
            role: RoleType::Validator,
            temp_dir: None,
        }
    }
}

impl BaseConfig {
    pub fn new(data_dir: PathBuf, role: RoleType) -> Self {
        BaseConfig {
            data_dir,
            role,
            temp_dir: None,
        }
    }

    /// Returns the full path to a file path. If the file_path is relative, it prepends with the
    /// data_dir. Otherwise it returns the provided full_path.
    pub fn full_path(&self, file_path: &PathBuf) -> PathBuf {
        if file_path.is_relative() {
            self.data_dir.join(file_path)
        } else {
            file_path.clone()
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    Validator,
    FullNode,
}

impl RoleType {
    pub fn is_validator(&self) -> bool {
        *self == RoleType::Validator
    }
}

impl std::str::FromStr for RoleType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "validator" => Ok(RoleType::Validator),
            "full_node" => Ok(RoleType::FullNode),
            _ => bail!("Invalid node role: {}", s),
        }
    }
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoleType::Validator => write!(f, "validator"),
            RoleType::FullNode => write!(f, "full_node"),
        }
    }
}

impl NodeConfig {
    pub fn set_data_dir(&mut self, path: PathBuf) -> Result<()> {
        self.base = Arc::new(BaseConfig::new(path, self.base.role));
        self.prepare()?;
        Ok(())
    }

    pub fn set_role(&mut self, role: RoleType) -> Result<()> {
        self.base = Arc::new(BaseConfig::new(self.base.data_dir.clone(), role));
        self.prepare()?;
        Ok(())
    }

    pub fn set_temp_dir(&mut self, path: TempPath) -> Result<()> {
        self.base = Arc::new(BaseConfig {
            data_dir: path.path().into(),
            role: self.base.role,
            temp_dir: Some(path),
        });
        self.prepare()?;
        Ok(())
    }

    pub fn base(&self) -> &Arc<BaseConfig> {
        &self.base
    }

    fn prepare(&mut self) -> Result<()> {
        if self.base.role.is_validator() {
            ensure!(
                self.validator_network.is_some(),
                "Missing a validator network config for a validator node"
            );
        } else {
            ensure!(
                self.validator_network.is_none(),
                "Provided a validator network config for a full_node node"
            );
        }

        for network in &mut self.full_node_networks {
            network.load(self.base.clone(), RoleType::FullNode)?;
        }
        if let Some(network) = &mut self.validator_network {
            network.load(self.base.clone(), RoleType::Validator)?;
        }
        self.consensus.load(self.base.clone())?;
        self.execution.load(self.base.clone())?;
        self.metrics.load(self.base.clone())?;
        self.storage.load(self.base.clone())?;
        Ok(())
    }

    /// Reads the config file and returns the configuration object in addition to doing some
    /// post-processing of the config
    /// Paths used in the config are either absolute or relative to the config location
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Self::load_config(&path);
        config.prepare()?;
        Ok(config)
    }

    pub fn get_genesis_transaction(&self) -> Result<Transaction> {
        let file_path = self.execution.genesis_file_location();
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
                    unreachable!("Failed to parse peer_id from string: {}", peer_id_str)
                }))
            })
            .collect()
    }

    pub fn randomize_ports(&mut self) {
        self.admission_control.randomize_ports();
        self.debug_interface.randomize_ports();
        self.execution.randomize_ports();
        self.mempool.randomize_ports();
        self.storage.randomize_ports();
    }

    pub fn random() -> Self {
        let mut rng = StdRng::from_seed([0u8; 32]);
        Self::random_with_rng(&mut rng)
    }

    pub fn random_with_rng(mut rng: &mut StdRng) -> Self {
        let mut config = NodeConfig::default();
        let validator_network = NetworkConfig::random(&mut rng);
        config.consensus = ConsensusConfig::random(&mut rng, validator_network.peer_id);
        config.validator_network = Some(validator_network);

        // Create temporary directory for persisting configs.
        let dir = TempPath::new();
        dir.create_as_dir().expect("error creating tempdir");
        config.set_temp_dir(dir).expect("Error setting temp_dir");

        config
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

#[cfg(test)]
mod test {
    use super::*;

    static EXPECTED_SINGLE_NODE_CONFIG: &[u8] =
        include_bytes!("../../data/configs/single.node.config.toml");

    #[test]
    fn verify_test_config() {
        // This test likely failed because there was a breaking change in the NodeConfig. It may be
        // desirable to reverse the change or to change the test config and potentially documentation.
        let mut actual = NodeConfig::random();
        let mut expected = NodeConfig::parse(&String::from_utf8_lossy(EXPECTED_SINGLE_NODE_CONFIG))
            .expect("Error parsing expected single node config");

        // These are randomly generated, so let's force them to be the same, perhaps we can use a
        // random seed so that these can be made uniform...
        expected
            .set_data_dir(actual.base.data_dir.clone())
            .expect("Unable to set data_dir");
        // Reseting any temp_dir
        actual
            .set_data_dir(actual.base.data_dir.clone())
            .expect("Unable to set data_dir");
        expected.consensus.consensus_keypair = actual.consensus.consensus_keypair.clone();
        expected.consensus.consensus_peers = actual.consensus.consensus_peers.clone();

        let actual_network = actual
            .validator_network
            .as_mut()
            .expect("Missing actual network config");
        let expected_network = expected
            .validator_network
            .as_mut()
            .expect("Missing expected network config");

        expected_network.advertised_address = actual_network.advertised_address.clone();
        expected_network.listen_address = actual_network.listen_address.clone();
        expected_network.network_keypairs = actual_network.network_keypairs.clone();
        expected_network.network_peers = actual_network.network_peers.clone();
        expected_network.seed_peers = actual_network.seed_peers.clone();

        // This is broken down first into smaller evaluations to improve idenitfying what is broken.
        // The output for a broken config leveraging assert at the top level config is not readable.
        assert_eq!(actual.admission_control, expected.admission_control);
        assert_eq!(actual.base, expected.base);
        assert_eq!(actual.consensus, expected.consensus);
        assert_eq!(actual.debug_interface, expected.debug_interface);
        assert_eq!(actual.execution, expected.execution);
        assert_eq!(actual.full_node_networks, expected.full_node_networks);
        assert_eq!(actual.full_node_networks.len(), 0);
        assert_eq!(actual.logger, expected.logger);
        assert_eq!(actual.mempool, expected.mempool);
        assert_eq!(actual.metrics, expected.metrics);
        assert_eq!(actual.state_sync, expected.state_sync);
        assert_eq!(actual.storage, expected.storage);
        assert_eq!(actual.validator_network, expected.validator_network);
        assert_eq!(actual.vm_config, expected.vm_config);
        assert_eq!(actual, expected);
    }

    #[test]
    fn verify_all_configs() {
        let _ = vec![
            PathBuf::from("data/configs/overrides/persistent_data.node.config.override.toml"),
            PathBuf::from("data/configs/single.node.config.toml"),
            PathBuf::from("../terraform/validator-sets/100/fn/node.config.toml"),
            PathBuf::from("../terraform/validator-sets/100/val/node.config.toml"),
            PathBuf::from("../terraform/validator-sets/dev/fn/node.config.toml"),
            PathBuf::from("../terraform/validator-sets/dev/val/node.config.toml"),
        ]
        .iter()
        .map(|path| NodeConfig::load(path).expect("NodeConfig"));
    }
}
