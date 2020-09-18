// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config_builder::BuildSwarm;
use libra_config::config::NodeConfig;
use libra_genesis_tool::config_builder::ValidatorBuilder;

pub struct Node {
    pub config: NodeConfig,
    pub root_key: libra_crypto::ed25519::Ed25519PrivateKey,
    node: libra_node::main_node::LibraHandle,
    _temp_dir: libra_temppath::TempPath,
}

impl Drop for Node {
    fn drop(&mut self) {
        self.node.shutdown();
    }
}

impl Node {
    pub fn start() -> Result<Self> {
        let temp_dir = libra_temppath::TempPath::new();
        temp_dir
            .create_as_dir()
            .expect("unable to create temporary config dir");
        let node_dir = temp_dir.path();

        let config_path = node_dir.join("node.yaml");

        let builder = ValidatorBuilder::new(1, NodeConfig::default_for_validator(), node_dir);
        let (mut configs, root_key) = builder.build_swarm()?;
        let mut config = configs.pop().unwrap();
        config.set_data_dir(node_dir.to_path_buf());
        config.save(&config_path)?;

        let node = libra_node::main_node::setup_environment(&config, None);

        Ok(Self {
            root_key,
            config,
            node,
            _temp_dir: temp_dir,
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
