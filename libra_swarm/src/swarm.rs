// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    output_tee::{OutputTee, OutputTeeGuard},
    utils,
};
use config::config::NodeConfig;
use config_builder::swarm_config::{SwarmConfig, SwarmConfigBuilder};
use crypto::signing::KeyPair;
use debug_interface::NodeDebugClient;
use failure::prelude::*;
use logger::prelude::*;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tempfile::TempDir;
use tools::output_capture::OutputCapture;

const LIBRA_NODE_BIN: &str = "libra_node";

pub struct LibraNode {
    node: Child,
    debug_client: NodeDebugClient,
    peer_id: String,
    log: PathBuf,
    output_tee_guard: Option<OutputTeeGuard>,
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
        if let Some(output_tee_guard) = self.output_tee_guard.take() {
            output_tee_guard.join();
        }
    }
}

impl LibraNode {
    pub fn launch(
        config: &NodeConfig,
        config_path: &Path,
        logdir: &Path,
        disable_logging: bool,
        tee_logs: bool,
    ) -> Result<Self> {
        let peer_id = config.base.peer_id.clone();
        let log = logdir.join(format!("{}.log", peer_id));
        let log_file = File::create(&log)?;
        let mut node_command = Command::new(utils::get_bin(LIBRA_NODE_BIN));
        node_command
            .current_dir(utils::workspace_root())
            .arg("-f")
            .arg(config_path)
            .args(&["-p", &peer_id]);
        if env::var("RUST_LOG").is_err() {
            // Only set our RUST_LOG if its not present in environment
            node_command.env("RUST_LOG", "debug");
        }
        if disable_logging {
            node_command.arg("-d");
        }

        if tee_logs {
            node_command.stdout(Stdio::piped()).stderr(Stdio::piped());
        } else {
            node_command
                .stdout(log_file.try_clone()?)
                .stderr(log_file.try_clone()?);
        };

        let mut node = node_command
            .spawn()
            .context("Error launching node process")?;

        let output_tee_guard = if tee_logs {
            let prefix = format!("[{}] ", &peer_id[..8]);
            let capture = OutputCapture::grab();
            let tee = OutputTee::new(
                capture,
                Box::new(log_file),
                Box::new(node.stdout.take().expect("Can't get child stdout")),
                Box::new(node.stderr.take().expect("Can't get child stderr")),
                prefix,
            );
            Some(tee.start())
        } else {
            None
        };

        let debug_client = NodeDebugClient::new(
            "localhost",
            config.debug_interface.admission_control_node_debug_port,
        );
        Ok(Self {
            node,
            debug_client,
            peer_id,
            log,
            output_tee_guard,
        })
    }

    pub fn get_log_contents(&self) -> Result<String> {
        let mut log = File::open(&self.log)?;
        let mut contents = String::new();
        log.read_to_string(&mut contents)?;

        Ok(contents)
    }

    fn get_metric(&self, metric_name: &str) -> Option<i64> {
        match self.debug_client.get_node_metric(metric_name) {
            Err(e) => {
                debug!(
                    "error getting {} for node: {}; error: {}",
                    metric_name, self.peer_id, e
                );
                None
            }
            Ok(maybeval) => {
                if maybeval.is_none() {
                    debug!("Node: {} did not report {}", self.peer_id, metric_name);
                }
                maybeval
            }
        }
    }

    pub fn check_connectivity(&self, expected_peers: i64) -> bool {
        if let Some(num_connected_peers) = self.get_metric("network_gauge{op=connected_peers}") {
            if num_connected_peers != expected_peers {
                debug!(
                    "Node '{}' Expected peers: {}, found peers: {}",
                    self.peer_id, expected_peers, num_connected_peers
                );
                return false;
            } else {
                return true;
            }
        }
        false
    }

    pub fn health_check(&mut self) -> HealthStatus {
        debug!("Health check on node '{}'", self.peer_id);

        // check if the process has terminated
        match self.node.try_wait() {
            // This would mean the child process has crashed
            Ok(Some(status)) => {
                debug!("Node '{}' crashed with: {}", self.peer_id, status);
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
                debug!("Node '{}' is healthy", self.peer_id);
                HealthStatus::Healthy
            }
            Err(e) => {
                debug!("Error querying metrics for node '{}'", self.peer_id);
                HealthStatus::RpcFailure(e)
            }
        }
    }
}

pub enum HealthStatus {
    Healthy,
    Crashed(::std::process::ExitStatus),
    RpcFailure(failure::Error),
}

/// Struct holding instances and information of Libra Swarm
pub struct LibraSwarm {
    dir: Option<TempDir>,
    nodes: HashMap<u16, LibraNode>,
    config: SwarmConfig,
    tee_logs: bool,
}

impl LibraSwarm {
    pub fn launch_swarm(
        num_nodes: usize,
        disable_logging: bool,
        faucet_account_keypair: KeyPair,
        tee_logs: bool,
    ) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let logs_dir_path = &dir.path().join("logs");
        println!(
            "Base directory containing logs and configs: {:?}",
            dir.path()
        );
        std::fs::create_dir(&logs_dir_path).unwrap();
        let base = utils::workspace_root().join("config/data/configs/node.config.toml");
        let mut config_builder = SwarmConfigBuilder::new();
        config_builder
            .with_ipv4()
            .with_nodes(num_nodes)
            .with_base(base)
            .with_output_dir(&dir)
            .with_faucet_keypair(faucet_account_keypair)
            .randomize_ports();
        let config = config_builder.build().unwrap();

        let mut swarm = Self {
            dir: Some(dir),
            nodes: HashMap::new(),
            config,
            tee_logs,
        };
        // For each config launch a node
        for (path, node_config) in swarm.config.get_configs() {
            let node = LibraNode::launch(
                &node_config,
                &path,
                &logs_dir_path,
                disable_logging,
                tee_logs,
            )
            .unwrap();
            swarm.nodes.insert(
                node_config.admission_control.admission_control_service_port,
                node,
            );
        }

        if !swarm.wait_for_startup() {
            panic!("Error launching swarm");
        }
        if !swarm.wait_for_connectivity() {
            // Verify connectivity
            panic!("Some nodes not connected to each other");
        }
        info!("Successfully launched Swarm");

        swarm
    }

    fn wait_for_connectivity(&self) -> bool {
        // Early return if we're only launching a single node
        if self.nodes.len() == 1 {
            return true;
        }

        let num_attempts = 60;

        for i in 0..num_attempts {
            debug!("Wait for connectivity attempt: {}", i);

            if self
                .nodes
                .values()
                .all(|node| node.check_connectivity(self.nodes.len() as i64 - 1))
            {
                return true;
            }

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        false
    }

    fn wait_for_startup(&mut self) -> bool {
        let num_attempts = 120;
        let mut done = vec![false; self.nodes.len()];

        for i in 0..num_attempts {
            debug!("Wait for startup attempt: {} of {}", i, num_attempts);
            for (node, done) in self.nodes.values_mut().zip(done.iter_mut()) {
                if *done {
                    continue;
                }

                match node.health_check() {
                    HealthStatus::Healthy => *done = true,
                    HealthStatus::RpcFailure(_) => continue,
                    HealthStatus::Crashed(status) => {
                        panic!(
                            "Libra node '{}' has crashed with status '{}'. Log output: '''{}'''",
                            node.peer_id,
                            status,
                            node.get_log_contents().unwrap()
                        );
                    }
                }
            }

            // Check if all the nodes have been successfully launched
            if done.iter().all(|status| *status) {
                return true;
            }

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        false
    }

    /// This function first checks the last committed round of all the nodes, picks the max
    /// value and then waits for all the nodes to catch up to that round.
    /// Once done, we can guarantee that all the txns committed before the invocation of this
    /// function are now available at all the nodes.
    pub fn wait_for_all_nodes_to_catchup(&mut self) -> bool {
        let num_attempts = 60;
        let last_committed_round_str = "consensus{op=committed_blocks_count}";
        let mut done = vec![false; self.nodes.len()];

        let mut last_committed_round = 0;
        // First, try to retrieve the max value across all the committed rounds
        debug!("Calculating max committed round across the validators.");
        for node in self.nodes.values() {
            match node.get_metric(last_committed_round_str) {
                Some(val) => {
                    debug!("\tNode {} last committed round = {}", node.peer_id, val);
                    last_committed_round = last_committed_round.max(val);
                }
                None => {
                    debug!(
                        "\tNode {} last committed round unknown, assuming 0.",
                        node.peer_id
                    );
                }
            }
        }

        // Now wait for all the nodes to catch up to the max.
        for i in 0..num_attempts {
            debug!(
                "Wait for catchup, target_commit_round = {}, attempt: {} of {}",
                last_committed_round, i, num_attempts
            );
            for (node, done) in self.nodes.values_mut().zip(done.iter_mut()) {
                if *done {
                    continue;
                }

                match node.get_metric(last_committed_round_str) {
                    Some(val) => {
                        if val >= last_committed_round {
                            debug!(
                                "\tNode {} is caught up with last committed round {}",
                                node.peer_id, val
                            );
                            *done = true;
                        } else {
                            debug!(
                                "\tNode {} is not caught up yet with last committed round {}",
                                node.peer_id, val
                            );
                        }
                    }
                    None => {
                        debug!(
                            "\tNode {} last committed round unknown, assuming 0.",
                            node.peer_id
                        );
                    }
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

    /// Vector with the public ports of all the validators in the swarm.
    pub fn get_validators_public_ports(&self) -> Vec<u16> {
        self.nodes.keys().cloned().collect()
    }

    /// Vector with the debug ports of all the validators in the swarm.
    pub fn get_validators_debug_ports(&self) -> Vec<u16> {
        self.config
            .get_configs()
            .iter()
            .map(|(_, c)| c.debug_interface.admission_control_node_debug_port)
            .collect()
    }

    pub fn kill_node(&mut self, port: u16) {
        self.nodes.remove(&port);
    }

    pub fn add_node(&mut self, port: u16, disable_logging: bool) -> bool {
        let logs_dir_path = self.dir.as_ref().map(|x| x.path().join("logs")).unwrap();

        for (path, config) in self.config.get_configs() {
            if config.admission_control.admission_control_service_port == port {
                let mut node = LibraNode::launch(
                    &config,
                    &path,
                    &logs_dir_path,
                    disable_logging,
                    self.tee_logs,
                )
                .unwrap();
                for _ in 0..60 {
                    if let HealthStatus::Healthy = node.health_check() {
                        self.nodes.insert(port, node);
                        return self.wait_for_connectivity();
                    }
                    ::std::thread::sleep(::std::time::Duration::from_millis(1000));
                }
            }
        }
        false
    }

    pub fn get_trusted_peers_config_path(&self) -> String {
        let (path, _) = self.config.get_trusted_peers_config();
        path.canonicalize()
            .expect("Unable to get canonical path of trusted peers config file")
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl Drop for LibraSwarm {
    fn drop(&mut self) {
        // If panicking, we don't want to gc the swarm directory.
        if std::thread::panicking() {
            if let Some(dir) = self.dir.take() {
                let logs = dir.into_path();
                println!("logs located: {:?}", logs);
            }
        }
    }
}
