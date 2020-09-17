// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use config_builder::BuildSwarm;
use libra_config::config::NodeConfig;
use libra_genesis_tool::config_builder::ValidatorBuilder;
use std::process::{Child, Command};

pub struct Node {
    node: Child,
    pub config: NodeConfig,
    pub root_key: libra_crypto::ed25519::Ed25519PrivateKey,
}

impl Drop for Node {
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

impl Node {
    pub fn start(node_dir: &std::path::Path) -> Result<Self> {
        let config_path = node_dir.join("node.yaml");
        let struct_log_path = node_dir.join("struct.log");
        let log_path = node_dir.join("log");

        // TODO: test with default_for_validator_full_node?
        let builder = ValidatorBuilder::new(1, NodeConfig::default_for_validator(), node_dir);
        let (mut configs, root_key) = builder.build_swarm()?;
        configs[0].set_data_dir(node_dir.to_path_buf());
        configs[0].save(&config_path)?;

        let mut cmd = Command::new(workspace_builder::get_libra_node_with_failpoints());
        cmd.current_dir(workspace_builder::workspace_root())
            .arg("-f")
            .arg(config_path);
        cmd.env("STRUCT_LOG_FILE", struct_log_path);

        let log_file = std::fs::File::create(&log_path)?;
        cmd.stdout(log_file.try_clone()?)
            .stderr(log_file.try_clone()?);

        let node = cmd.spawn().context("Error launching node process")?;

        Ok(Self {
            node,
            root_key,
            config: configs.pop().unwrap(),
        })
    }

    pub fn port(&self) -> u16 {
        self.config.rpc.address.port()
    }

    pub fn url(&self) -> String {
        format!("http://localhost:{}/v1", self.port())
    }

    pub fn healthy_url(&self) -> String {
        format!("http://localhost:{}/-/healthy", self.port())
    }

    pub fn wait_for_jsonrpc_connectivity(&self) {
        let num_attempts = 60;
        for _ in 0..num_attempts {
            if self.check_jsonrpc_connectivity() {
                return;
            }
            ::std::thread::sleep(::std::time::Duration::from_millis(500));
        }

        panic!("wait for jsonrpc connectivity timeout");
    }

    pub fn check_jsonrpc_connectivity(&self) -> bool {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(&self.healthy_url()).send();
        if let Ok(ret) = resp {
            if let reqwest::StatusCode::OK = ret.status() {
                return true;
            }
        }
        false
    }
}
