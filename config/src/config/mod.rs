// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use libra_types::PeerId;
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

mod admission_control_config;
pub use admission_control_config::*;
mod rpc_config;
pub use rpc_config::*;
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
mod secure_config;
pub use secure_config::*;
mod state_sync_config;
pub use state_sync_config::*;
mod storage_config;
pub use storage_config::*;
mod safety_rules_config;
pub use safety_rules_config::*;
mod upstream_config;
pub use upstream_config::*;
mod test_config;
use libra_types::waypoint::Waypoint;
pub use test_config::*;

/// Config pulls in configuration information from the config file.
/// This is used to set up the nodes and configure various parameters.
/// The config file is broken up into sections for each module
/// so that only that module can be passed around
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone, PartialEq))]
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub admission_control: AdmissionControlConfig,
    #[serde(default)]
    pub rpc: RpcConfig,
    #[serde(default)]
    pub base: BaseConfig,
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
    pub secure: SecureConfig,
    #[serde(default)]
    pub state_sync: StateSyncConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub test: Option<TestConfig>,
    #[serde(default)]
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub validator_network: Option<NetworkConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BaseConfig {
    data_dir: PathBuf,
    pub role: RoleType,
    pub waypoint: Option<Waypoint>,
}

impl Default for BaseConfig {
    fn default() -> BaseConfig {
        BaseConfig {
            data_dir: PathBuf::from("/opt/libra/data/commmon"),
            role: RoleType::Validator,
            waypoint: None,
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
    pub fn is_validator(self) -> bool {
        self == RoleType::Validator
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RoleType::Validator => "validator",
            RoleType::FullNode => "full_node",
        }
    }
}

impl FromStr for RoleType {
    type Err = ParseRoleError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "validator" => Ok(RoleType::Validator),
            "full_node" => Ok(RoleType::FullNode),
            _ => Err(ParseRoleError(s.to_string())),
        }
    }
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Error)]
#[error("Invalid node role: {0}")]
pub struct ParseRoleError(String);

impl NodeConfig {
    pub fn data_dir(&self) -> &PathBuf {
        &self.base.data_dir
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.base.data_dir = data_dir.clone();
        self.consensus.set_data_dir(data_dir.clone());
        self.metrics.set_data_dir(data_dir.clone());
        self.storage.set_data_dir(data_dir);
    }

    /// This clones the underlying data except for the keypair so that this config can be used as a
    /// template for another config.
    pub fn clone_for_template(&self) -> Self {
        Self {
            admission_control: self.admission_control.clone(),
            rpc: self.rpc.clone(),
            base: self.base.clone(),
            consensus: self.consensus.clone(),
            debug_interface: self.debug_interface.clone(),
            execution: self.execution.clone(),
            full_node_networks: self
                .full_node_networks
                .iter()
                .map(|c| c.clone_for_template())
                .collect(),
            logger: self.logger.clone(),
            metrics: self.metrics.clone(),
            mempool: self.mempool.clone(),
            state_sync: self.state_sync.clone(),
            secure: self.secure.clone(),
            storage: self.storage.clone(),
            test: None,
            upstream: self.upstream.clone(),
            validator_network: self
                .validator_network
                .as_ref()
                .map(|n| n.clone_for_template()),
        }
    }

    /// Reads the config file and returns the configuration object in addition to doing some
    /// post-processing of the config
    /// Paths used in the config are either absolute or relative to the config location
    pub fn load<P: AsRef<Path>>(input_path: P) -> Result<Self> {
        let mut config = Self::load_config(&input_path)?;
        if config.base.role.is_validator() {
            ensure!(
                config.validator_network.is_some(),
                "Missing a validator network config for a validator node"
            );
        } else {
            ensure!(
                config.validator_network.is_none(),
                "Provided a validator network config for a full_node node"
            );
        }

        let input_dir = RootPath::new(input_path);
        config.execution.load(&input_dir)?;
        if let Some(network) = &mut config.validator_network {
            network.load(&input_dir, RoleType::Validator)?;
        }
        for network in &mut config.full_node_networks {
            network.load(&input_dir, RoleType::FullNode)?;
        }
        config.set_data_dir(config.data_dir().clone());
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&mut self, output_path: P) -> Result<()> {
        let output_dir = RootPath::new(&output_path);
        self.execution.save(&output_dir)?;
        if let Some(network) = &mut self.validator_network {
            network.save(&output_dir)?;
        }
        for network in &mut self.full_node_networks {
            network.save(&output_dir)?;
        }
        // This must be last as calling save on subconfigs may change their fields
        self.save_config(&output_path)?;
        Ok(())
    }

    pub fn randomize_ports(&mut self) {
        self.admission_control.randomize_ports();
        self.debug_interface.randomize_ports();
        self.storage.randomize_ports();
        self.rpc.randomize_ports();
    }

    pub fn random() -> Self {
        let mut rng = StdRng::from_seed([0u8; 32]);
        Self::random_with_rng(&mut rng)
    }

    pub fn random_with_rng(rng: &mut StdRng) -> Self {
        let mut config = NodeConfig::default();
        config.random_internal(rng);
        config
    }

    pub fn random_with_template(template: &Self, rng: &mut StdRng) -> Self {
        let mut config = template.clone_for_template();
        config.random_internal(rng);
        config
    }

    fn random_internal(&mut self, rng: &mut StdRng) {
        let mut test = TestConfig::new_with_temp_dir();

        if self.base.role == RoleType::Validator {
            test.initialize_storage = true;
            test.random_account_key(rng);
            let peer_id =
                PeerId::from_public_key(&test.account_keypair.as_ref().unwrap().public_key());

            if self.validator_network.is_none() {
                self.validator_network = Some(NetworkConfig::default());
            }

            let validator_network = self.validator_network.as_mut().unwrap();
            validator_network.random_with_peer_id(rng, Some(peer_id));
            test.random_consensus_key(rng);
        } else {
            self.validator_network = None;
            if self.full_node_networks.is_empty() {
                self.full_node_networks.push(NetworkConfig::default());
            }
            for network in &mut self.full_node_networks {
                network.random(rng);
            }
        }
        self.set_data_dir(test.temp_dir().unwrap().to_path_buf());
        self.test = Some(test);
    }
}

pub trait PersistableConfig: Serialize + DeserializeOwned {
    fn load_config<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::parse(&contents)
    }

    fn save_config<P: AsRef<Path>>(&self, output_file: P) -> Result<()> {
        let contents = toml::to_vec(&self)?;
        let mut file = File::create(output_file)?;
        file.write_all(&contents)?;
        // @TODO This causes a major perf regression that needs to be evaluated before enabling
        // file.sync_all()?;
        Ok(())
    }

    fn parse(serialized: &str) -> Result<Self> {
        Ok(toml::from_str(&serialized)?)
    }
}

impl<T: ?Sized> PersistableConfig for T where T: Serialize + DeserializeOwned {}

#[derive(Debug)]
pub struct RootPath {
    root_path: PathBuf,
}

impl RootPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root_path = if let Some(parent) = path.as_ref().parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from("")
        };

        Self { root_path }
    }

    /// This function assumes that the path is already a directory
    pub fn new_path<P: AsRef<Path>>(path: P) -> Self {
        let root_path = path.as_ref().to_path_buf();
        Self { root_path }
    }

    /// This adds a full path when loading / storing if one is not specified
    pub fn full_path(&self, file_path: &PathBuf) -> PathBuf {
        if file_path.is_relative() {
            self.root_path.join(file_path)
        } else {
            file_path.clone()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DEFAULT: &str = "src/config/test_data/single.node.config.toml";
    const RANDOM_DEFAULT: &str = "src/config/test_data/random.default.node.config.toml";
    const RANDOM_COMPLETE: &str = "src/config/test_data/random.complete.node.config.toml";

    #[test]
    fn verify_default_config() {
        // This test likely failed because there was a breaking change in the NodeConfig. It may be
        // desirable to reverse the change or to change the test config and potentially documentation.
        let mut actual = NodeConfig::random();
        let mut expected = NodeConfig::load(DEFAULT).expect("Unable to load config");

        // These are randomly generated, so let's force them to be the same, perhaps we can use a
        // random seed so that these can be made uniform...
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

        expected.set_data_dir(actual.data_dir().clone());
        compare_configs(&actual, &expected);
    }

    #[test]
    fn verify_random_complete_config() {
        let mut rng = StdRng::from_seed([255u8; 32]);
        let mut expected = NodeConfig::random_with_rng(&mut rng);

        // Update paths after save
        let root_dir = RootPath::new(expected.test.as_ref().unwrap().temp_dir().unwrap());
        let path = root_dir.full_path(&PathBuf::from("node.config.toml"));
        expected.save(&path).expect("Unable to save config");

        let actual = NodeConfig::load(RANDOM_COMPLETE).expect("Unable to load config");
        expected.set_data_dir(actual.data_dir().clone());
        compare_configs(&actual, &expected);
    }

    #[test]
    fn verify_random_default_config() {
        let mut rng = StdRng::from_seed([255u8; 32]);
        let mut expected = NodeConfig::random_with_rng(&mut rng);

        // Update paths after save
        let root_dir = RootPath::new(expected.test.as_ref().unwrap().temp_dir().unwrap());
        let path = root_dir.full_path(&PathBuf::from("node.config.toml"));
        expected.save(&path).expect("Unable to save config");

        let actual = NodeConfig::load(RANDOM_DEFAULT).expect("Unable to load config");
        expected.set_data_dir(actual.data_dir().clone());
        compare_configs(&actual, &expected);
    }

    fn compare_configs(actual: &NodeConfig, expected: &NodeConfig) {
        // This is broken down first into smaller evaluations to improve identifying what is broken.
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
        assert_eq!(actual.test, expected.test);
        assert_eq!(actual.validator_network, expected.validator_network);
        assert_eq!(actual, expected);
    }

    #[test]
    fn verify_all_configs() {
        let _ = vec![
            // This contains all the default fields written to disk, it verifies that the default
            // is consistent and can be loaded without failure
            DEFAULT,
            // This config leverages default fields but uses the same PeerId and secondary files as
            // the random.complete.node.config.toml. It verifies the assumptions about loading
            // files even if the paths aren't present
            RANDOM_DEFAULT,
            // This config explicitly writes all the default values for a random peer to disk and
            // verifies that it correctly loads. It shares the same PeerId as
            // random.default.node.config.toml
            RANDOM_COMPLETE,
        ]
        .iter()
        .map(|path| {
            NodeConfig::load(PathBuf::from(path)).unwrap_or_else(|_| panic!("Error in {}", path))
        })
        .collect::<Vec<_>>();
    }

    #[test]
    fn verify_role_type_conversion() {
        // Verify relationship between RoleType and as_string() is reflexive
        let validator = RoleType::Validator;
        let full_node = RoleType::FullNode;
        let converted_validator = RoleType::from_str(validator.as_str()).unwrap();
        let converted_full_node = RoleType::from_str(full_node.as_str()).unwrap();
        assert_eq!(converted_validator, validator);
        assert_eq!(converted_full_node, full_node);
    }

    #[test]
    // TODO(joshlind): once the 'matches' crate becomes stable, clean this test up!
    fn verify_parse_role_error_on_invalid_role() {
        let invalid_role_type = "this is not a valid role type";
        match RoleType::from_str(invalid_role_type) {
            Err(ParseRoleError(_)) => { /* the expected error was thrown! */ }
            _ => panic!("A ParseRoleError should have been thrown on the invalid role type!"),
        }
    }
}
