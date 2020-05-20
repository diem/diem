// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use config_builder::{FullNodeConfig, SwarmConfig, ValidatorConfig};
use debug_interface::NodeDebugClient;
use libra_config::config::{NodeConfig, RoleType};
use libra_logger::prelude::*;
use libra_temppath::TempPath;
use libra_types::account_address::AccountAddress;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};
use thiserror::Error;

const LIBRA_NODE_BIN: &str = "libra-node";

pub struct LibraNode {
    node: Child,
    node_id: String,
    validator_peer_id: Option<AccountAddress>,
    role: RoleType,
    debug_client: NodeDebugClient,
    port: u16,
    log: PathBuf,
}

impl Drop for LibraNode {
    // When the LibraNode struct goes out of scope we need to kill the child process
    fn drop(&mut self) {
        // check if the process has already been terminated
        match self.node.try_wait() {
            // The child process has already terminated, perhaps due to a crash
            Ok(Some(_)) => {}

            // The node is still running so we need to attempt to kill it
            _ => {
                if let Err(e) = self.node.kill() {
                    panic!("LibraNode process could not be killed: '{}'", e);
                }
            }
        }
    }
}

impl LibraNode {
    /// Prior to using LibraSwarm this should be run. LibraSwarm will acquire ephemeral networking
    /// ports that are only reserved in a safe state for a brief period of time.
    /// workspace_builder::get_bin actually compiles all of the Libra code base which can take
    /// substantially longer time than the networking ports are reserved. Calling prior to
    /// reserving those ports will reduce the liklihood of issues.
    pub fn prepare() {
        Command::new(workspace_builder::get_bin(LIBRA_NODE_BIN));
    }

    pub fn launch(
        node_id: String,
        role: RoleType,
        config_path: &Path,
        log_path: PathBuf,
        struct_log_path: PathBuf,
        disable_logging: bool,
    ) -> Result<Self> {
        let config = NodeConfig::load(&config_path)
            .unwrap_or_else(|_| panic!("Failed to load NodeConfig from file: {:?}", config_path));
        let log_file = File::create(&log_path)?;
        let validator_peer_id = match role {
            RoleType::Validator => Some(config.validator_network.unwrap().peer_id),
            RoleType::FullNode => None,
        };
        let mut node_command = Command::new(workspace_builder::get_bin(LIBRA_NODE_BIN));
        node_command
            .current_dir(workspace_builder::workspace_root())
            .arg("-f")
            .arg(config_path);
        if env::var("RUST_LOG").is_err() {
            // Only set our RUST_LOG if its not present in environment
            node_command.env("RUST_LOG", "debug");
        }
        if disable_logging {
            node_command.arg("-d");
        } else {
            node_command.env("STRUCT_LOG_FILE", struct_log_path);
        }
        node_command
            .stdout(log_file.try_clone()?)
            .stderr(log_file.try_clone()?);
        let node = node_command
            .spawn()
            .context("Error launching node process")?;
        let debug_client = NodeDebugClient::new(
            "localhost",
            config.debug_interface.admission_control_node_debug_port,
        );
        Ok(Self {
            node,
            node_id,
            validator_peer_id,
            role,
            debug_client,
            port: config.rpc.address.port(),
            log: log_path,
        })
    }

    pub fn validator_peer_id(&self) -> Option<AccountAddress> {
        self.validator_peer_id
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn get_log_contents(&self) -> Result<String> {
        let mut log = File::open(&self.log)?;
        let mut contents = String::new();
        log.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn get_metric(&mut self, metric_name: &str) -> Option<i64> {
        match self.debug_client.get_node_metric(metric_name) {
            Err(e) => {
                println!(
                    "error getting {} for node: {}; error: {}",
                    metric_name, self.node_id, e
                );
                None
            }
            Ok(maybeval) => {
                if maybeval.is_none() {
                    println!("Node: {} did not report {}", self.node_id, metric_name);
                }
                maybeval
            }
        }
    }

    pub fn check_connectivity(&mut self, expected_peers: i64) -> bool {
        let connected_peers = format!(
            "libra_network_peers{{role_type={},state=connected}}",
            self.role.to_string()
        );
        if let Some(num_connected_peers) = self.get_metric(&connected_peers) {
            if num_connected_peers < expected_peers {
                println!(
                    "Node '{}' Expected peers: {}, found peers: {}",
                    self.node_id, expected_peers, num_connected_peers
                );
                return false;
            } else {
                return true;
            }
        }
        false
    }

    pub fn health_check(&mut self) -> HealthStatus {
        println!("Health check on node '{}'", self.node_id);

        // check if the process has terminated
        match self.node.try_wait() {
            // This would mean the child process has crashed
            Ok(Some(status)) => {
                println!("Node '{}' crashed with: {}", self.node_id, status);
                return HealthStatus::Crashed(status);
            }

            // This is the case where the node is still running
            Ok(None) => {}

            // Some other unknown error
            Err(e) => {
                panic!("error attempting to query Node: {}", e);
            }
        }

        match self.debug_client.get_node_metrics() {
            Ok(_) => {
                println!("Node '{}' is healthy", self.node_id);
                HealthStatus::Healthy
            }
            Err(e) => {
                println!("Error querying metrics for node '{}'", self.node_id);
                HealthStatus::RpcFailure(e)
            }
        }
    }
}

pub enum HealthStatus {
    Healthy,
    Crashed(::std::process::ExitStatus),
    RpcFailure(anyhow::Error),
}

/// A wrapper that unifies PathBuf and TempPath.
#[derive(Debug)]
pub enum LibraSwarmDir {
    Persistent(PathBuf),
    Temporary(TempPath),
}

impl AsRef<Path> for LibraSwarmDir {
    fn as_ref(&self) -> &Path {
        match self {
            LibraSwarmDir::Persistent(path_buf) => path_buf.as_path(),
            LibraSwarmDir::Temporary(temp_dir) => temp_dir.path(),
        }
    }
}

/// Struct holding instances and information of Libra Swarm
pub struct LibraSwarm {
    // Output log, LibraNodes' config file, libradb etc, into this dir.
    pub dir: LibraSwarmDir,
    // Maps the node id of a node to the LibraNode struct
    pub nodes: HashMap<String, LibraNode>,
    pub config: SwarmConfig,
}

#[derive(Debug, Error)]
pub enum SwarmLaunchFailure {
    /// Timeout while waiting for nodes to start
    #[error("Node launch check timeout")]
    LaunchTimeout,
    /// Node return status indicates a crash
    #[error("Node crash")]
    NodeCrash,
    /// Timeout while waiting for the nodes to report that they're all interconnected
    #[error("Node connectivity check timeout")]
    ConnectivityTimeout,
    #[error("IO Error")]
    IoError(#[from] io::Error),
}

impl LibraSwarm {
    pub fn launch_swarm(
        num_nodes: usize,
        role: RoleType,
        disable_logging: bool,
        config_dir: Option<String>,
        template: Option<NodeConfig>,
        upstream_config_dir: Option<String>,
    ) -> Self {
        let num_launch_attempts = 5;
        for i in 0..num_launch_attempts {
            info!("Launch swarm attempt: {} of {}", i, num_launch_attempts);

            if let Ok(mut swarm) = Self::configure_swarm(
                num_nodes,
                role,
                config_dir.clone(),
                Some(template.as_ref().cloned().unwrap_or_default()),
                upstream_config_dir.clone(),
            ) {
                if let Err(e) = swarm.launch_attempt(role, disable_logging) {
                    error!("Error launching swarm: {}", e);
                } else {
                    return swarm;
                }
            }
        }
        panic!("Max out {} attempts to launch swarm", num_launch_attempts);
    }

    /// Either create a persistent directory for swarm or return a temporary one.
    /// If specified persistent directory already exists,
    /// assumably due to previous launch failure, it will be removed.
    /// The directory for the last failed attempt won't be removed.
    fn setup_config_dir(config_dir: &Option<String>) -> LibraSwarmDir {
        if let Some(dir_str) = config_dir {
            let path_buf = PathBuf::from_str(&dir_str).expect("unable to create config dir");
            if path_buf.exists() {
                std::fs::remove_dir_all(dir_str).expect("unable to delete previous config dir");
            }
            std::fs::create_dir_all(dir_str).expect("unable to create config dir");
            LibraSwarmDir::Persistent(path_buf)
        } else {
            let temp_dir = TempPath::new();
            temp_dir
                .create_as_dir()
                .expect("unable to create temporary config dir");
            LibraSwarmDir::Temporary(temp_dir)
        }
    }

    pub fn configure_swarm(
        num_nodes: usize,
        role: RoleType,
        config_dir: Option<String>,
        template: Option<NodeConfig>,
        upstream_config_dir: Option<String>,
    ) -> Result<LibraSwarm> {
        LibraNode::prepare();

        let swarm_config_dir = Self::setup_config_dir(&config_dir);
        info!("logs at {:?}", swarm_config_dir);

        let mut node_config = if let Some(template) = template {
            template
        } else {
            NodeConfig::default()
        };
        node_config.base.role = role;

        let config_path = &swarm_config_dir.as_ref().to_path_buf();
        let config = if role.is_validator() {
            let mut validator_builder = ValidatorConfig::new();
            validator_builder
                .template(node_config)
                .validators(num_nodes);
            SwarmConfig::build(&validator_builder, config_path)?
        } else {
            let upstream_config_dir = upstream_config_dir.expect("No upstream node for full nodes");
            let upstream_config_file = PathBuf::from(upstream_config_dir).join("node.config.toml");
            let mut validator_config = NodeConfig::load(&upstream_config_file)?;
            let genesis = validator_config.execution.genesis.as_ref();
            let mut full_node_builder = FullNodeConfig::new();
            full_node_builder
                .full_nodes(num_nodes)
                .genesis(genesis.expect("Missing genesis from validator").clone())
                .template(node_config);
            full_node_builder.extend_validator(&mut validator_config)?;
            validator_config.save(&upstream_config_file)?;
            full_node_builder.bootstrap(
                validator_config.full_node_networks[0]
                    .advertised_address
                    .clone(),
            );
            SwarmConfig::build(&full_node_builder, config_path)?
        };

        Ok(Self {
            dir: swarm_config_dir,
            nodes: HashMap::new(),
            config,
        })
    }

    pub fn launch_attempt(
        &mut self,
        role: RoleType,
        disable_logging: bool,
    ) -> Result<(), SwarmLaunchFailure> {
        let logs_dir_path = self.dir.as_ref().join("logs");
        std::fs::create_dir(&logs_dir_path)?;
        // For each config launch a node
        for (index, path) in self.config.config_files.iter().enumerate() {
            // Use index as node id.
            let node_id = format!("{}", index);
            let node = LibraNode::launch(
                node_id.clone(),
                role,
                &path,
                logs_dir_path.join(format!("{}.log", index)),
                logs_dir_path.join(format!("{}.struct.log", index)),
                disable_logging,
            )
            .unwrap();
            self.nodes.insert(node_id, node);
        }
        self.wait_for_startup()?;
        self.wait_for_connectivity()?;
        info!("Successfully launched Swarm");
        Ok(())
    }

    fn wait_for_connectivity(&mut self) -> Result<(), SwarmLaunchFailure> {
        // Early return if we're only launching a single node
        if self.nodes.len() == 1 {
            return Ok(());
        }

        let num_attempts = 60;

        for i in 0..num_attempts {
            println!("Wait for connectivity attempt: {}", i);

            let len = self.nodes.len();
            if self
                .nodes
                .values_mut()
                .all(|node| node.check_connectivity(len as i64 - 1))
            {
                return Ok(());
            }
            // TODO check full node connectivity for full nodes

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        Err(SwarmLaunchFailure::ConnectivityTimeout)
    }

    fn wait_for_startup(&mut self) -> Result<(), SwarmLaunchFailure> {
        let num_attempts = 120;
        let mut done = vec![false; self.nodes.len()];
        for i in 0..num_attempts {
            println!("Wait for startup attempt: {} of {}", i, num_attempts);
            for (node, done) in self.nodes.values_mut().zip(done.iter_mut()) {
                if *done {
                    continue;
                }
                match node.health_check() {
                    HealthStatus::Healthy => *done = true,
                    HealthStatus::RpcFailure(_) => continue,
                    HealthStatus::Crashed(status) => {
                        error!(
                            "Libra node '{}' has crashed with status '{}'. Log output: '''{}'''",
                            node.node_id,
                            status,
                            node.get_log_contents().unwrap()
                        );
                        return Err(SwarmLaunchFailure::NodeCrash);
                    }
                }
            }

            // Check if all the nodes have been successfully launched
            if done.iter().all(|status| *status) {
                return Ok(());
            }

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        Err(SwarmLaunchFailure::LaunchTimeout)
    }

    /// This function first checks the last committed round of all the nodes, picks the max
    /// value and then waits for all the nodes to catch up to that round.
    /// Once done, we can guarantee that all the txns committed before the invocation of this
    /// function are now available at all the nodes.
    pub fn wait_for_all_nodes_to_catchup(&mut self) -> bool {
        let num_attempts = 60;
        let last_committed_round_str = "libra_consensus_last_committed_round{}";
        let mut done = vec![false; self.nodes.len()];

        let mut last_committed_round = 0;
        // First, try to retrieve the max value across all the committed rounds
        println!("Calculating max committed round across the validators.");
        for node in self.nodes.values_mut() {
            match node.get_metric(last_committed_round_str) {
                Some(val) => {
                    println!("\tNode {} last committed round = {}", node.node_id, val);
                    last_committed_round = last_committed_round.max(val);
                }
                None => {
                    println!(
                        "\tNode {} last committed round unknown, assuming 0.",
                        node.node_id
                    );
                }
            }
        }

        // Now wait for all the nodes to catch up to the max.
        for i in 0..num_attempts {
            println!(
                "Wait for catchup, target_commit_round = {}, attempt: {} of {}",
                last_committed_round,
                i + 1,
                num_attempts
            );

            for (node, done) in self.nodes.values_mut().zip(done.iter_mut()) {
                if *done {
                    continue;
                }

                if let Some(val) = node.get_metric(last_committed_round_str) {
                    if val >= last_committed_round {
                        println!(
                            "\tNode {} is caught up with last committed round {}",
                            node.node_id, val
                        );
                        *done = true;
                    } else {
                        println!(
                            "\tNode {} is not caught up yet with last committed round {}",
                            node.node_id, val
                        );
                    }
                } else {
                    println!(
                        "\tNode {} last committed round unknown, assuming 0.",
                        node.node_id
                    );
                }
            }

            // Check if all the nodes have been successfully caught up
            if done.iter().all(|status| *status) {
                return true;
            }

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        false
    }

    /// A specific public JSON RPC port of a validator or a full node.
    pub fn get_client_port(&self, index: usize) -> u16 {
        let node_id = format!("{}", index);
        self.nodes.get(&node_id).map(|node| node.port()).unwrap()
    }

    /// Vector with the peer ids of the validators in the swarm.
    pub fn get_validators_ids(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Vector with the debug ports of all the validators in the swarm.
    pub fn get_validators_debug_ports(&self) -> Vec<u16> {
        self.config
            .config_files
            .iter()
            .map(|path| {
                let config = NodeConfig::load(&path).unwrap();
                config.debug_interface.admission_control_node_debug_port
            })
            .collect()
    }

    pub fn get_validator(&self, idx: usize) -> Option<&LibraNode> {
        let node_id = format!("{}", idx);
        self.nodes.get(&node_id)
    }

    pub fn kill_node(&mut self, idx: usize) {
        let node_id = format!("{}", idx);
        self.nodes.remove(&node_id);
    }

    pub fn add_node(
        &mut self,
        idx: usize,
        role: RoleType,
        disable_logging: bool,
    ) -> Result<(), SwarmLaunchFailure> {
        // First take the configs out to not keep immutable borrow on self when calling
        // `launch_node`.
        let path = self
            .config
            .config_files
            .get(idx)
            .unwrap_or_else(|| panic!("Node at index {} not found", idx));
        let log_file_path = self.dir.as_ref().join("logs").join(format!("{}.log", idx));
        let struct_log_file_path = self
            .dir
            .as_ref()
            .join("logs")
            .join(format!("{}.struct.log", idx));
        let node_id = format!("{}", idx);
        let mut node = LibraNode::launch(
            node_id.clone(),
            role,
            path,
            log_file_path,
            struct_log_file_path,
            disable_logging,
        )
        .unwrap();
        for _ in 0..60 {
            if let HealthStatus::Healthy = node.health_check() {
                self.nodes.insert(node_id, node);
                return self.wait_for_connectivity();
            }
            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }
        Err(SwarmLaunchFailure::LaunchTimeout)
    }
}

impl Drop for LibraSwarm {
    fn drop(&mut self) {
        // If panicking, we don't want to gc the swarm directory.
        if std::thread::panicking() {
            // let dir = self.dir;
            if let LibraSwarmDir::Temporary(temp_dir) = &mut self.dir {
                temp_dir.persist();
                let log_path = temp_dir.path();
                println!("logs located at {:?}", log_path);

                // Dump logs for each validator to stdout when `LIBRA_DUMP_LOGS`
                // environment variable is set
                if env::var_os("LIBRA_DUMP_LOGS").is_some() {
                    for (peer_id, node) in &mut self.nodes {
                        // Skip dumping logs for healthy nodes
                        if let HealthStatus::Healthy = node.health_check() {
                            continue;
                        }

                        // Grab the contents of the node's logs and skip if we were unable to
                        // grab its logs
                        let log_contents = match node.get_log_contents() {
                            Ok(contents) => contents,
                            Err(_) => continue,
                        };

                        println!();
                        println!();
                        println!("{:=^80}", "");
                        println!("Validator {}", peer_id);
                        println!();
                        println!();
                        println!("{}", log_contents);
                    }
                }
            }
        }
    }
}
