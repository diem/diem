// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::workspace_builder;
use cli::client_proxy::ClientProxy;
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_genesis_tool::config_builder::FullnodeType;
use libra_key_manager::libra_interface::JsonRpcLibraInterface;
use libra_operational_tool::test_helper::OperationalTool;
use libra_swarm::swarm::LibraSwarm;
use libra_temppath::TempPath;
use libra_types::{chain_id::ChainId, waypoint::Waypoint};
use std::path::PathBuf;

pub struct SmokeTestEnvironment {
    pub validator_swarm: LibraSwarm,
    pub vfn_swarm: Option<LibraSwarm>,
    pub public_fn_swarm: Option<LibraSwarm>,
    libra_root_key: (Ed25519PrivateKey, String),
    mnemonic_file: TempPath,
}

impl SmokeTestEnvironment {
    /// Returns a JSON RPC based Libra Interface pointing to a validator node at the given
    /// node index.
    pub fn get_json_rpc_libra_interface(&self, node_index: usize) -> JsonRpcLibraInterface {
        let json_rpc_endpoint = format!(
            "http://127.0.0.1:{}",
            self.validator_swarm.get_client_port(node_index)
        );
        JsonRpcLibraInterface::new(json_rpc_endpoint)
    }

    /// Returns an operational tool pointing to a validator node at the given node index.
    pub fn get_op_tool(&self, node_index: usize) -> OperationalTool {
        OperationalTool::new(
            format!(
                "http://127.0.0.1:{}",
                self.validator_swarm.get_client_port(node_index)
            ),
            ChainId::test(),
        )
    }

    /// Returns a client pointing to a public full node at the given node index.
    pub fn get_pfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        get_client_proxy(
            self.libra_root_key.1.clone(),
            self.mnemonic_file.path().to_path_buf(),
            self.public_fn_swarm
                .as_ref()
                .expect("Public fn swarm is not initialized"),
            node_index,
            waypoint,
        )
    }

    /// Returns a client pointing to a validator node at the given node index.
    pub fn get_validator_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        get_client_proxy(
            self.libra_root_key.1.clone(),
            self.mnemonic_file.path().to_path_buf(),
            &self.validator_swarm,
            node_index,
            waypoint,
        )
    }

    /// Returns a client pointing to a validator full node at the given node index.
    pub fn get_vfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        get_client_proxy(
            self.libra_root_key.1.clone(),
            self.mnemonic_file.path().to_path_buf(),
            self.vfn_swarm
                .as_ref()
                .expect("Vfn swarm is not initialized"),
            node_index,
            waypoint,
        )
    }

    /// Loads the node config for the validator at the specified index.
    pub fn load_node_config(&self, node_index: usize) -> NodeConfig {
        let node_config_path = self
            .validator_swarm
            .config
            .config_files
            .get(node_index)
            .unwrap();
        NodeConfig::load(&node_config_path).unwrap()
    }

    pub fn new(num_validators: usize) -> Self {
        Self::new_with_chunk_limit(num_validators, 10)
    }

    pub fn new_with_chunk_limit(num_validators: usize, chunk_limit: u64) -> Self {
        ::libra_logger::Logger::new().init();
        let mut template = NodeConfig::default_for_validator();
        template.state_sync.chunk_limit = chunk_limit;

        let validator_swarm = LibraSwarm::configure_validator_swarm(
            &workspace_builder::get_libra_node_with_failpoints(),
            num_validators,
            None,
            Some(template),
        )
        .unwrap();

        let mnemonic_file = libra_temppath::TempPath::new();
        mnemonic_file
            .create_as_file()
            .expect("could not create temporary mnemonic_file_path");

        let key = generate_key::load_key(&validator_swarm.config.libra_root_key_path);
        let key_path = validator_swarm
            .config
            .libra_root_key_path
            .to_str()
            .expect("Unable to read faucet path")
            .to_string();

        Self {
            validator_swarm,
            vfn_swarm: None,
            public_fn_swarm: None,
            libra_root_key: (key, key_path),
            mnemonic_file,
        }
    }

    /// Saves the node config for the validator at the specified index.
    pub fn save_node_config(&self, mut node_config: NodeConfig, node_index: usize) {
        let node_config_path = self
            .validator_swarm
            .config
            .config_files
            .get(node_index)
            .unwrap();
        node_config.save(node_config_path).unwrap();
    }

    /// Configures a specified number of public full nodes
    pub fn setup_pfn_swarm(&mut self, num_nodes: usize) {
        self.public_fn_swarm = Some(
            LibraSwarm::configure_fn_swarm(
                &workspace_builder::get_libra_node_with_failpoints(),
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::PublicFullnode(num_nodes),
            )
            .unwrap(),
        )
    }

    /// Configures a specified number of validator full nodes
    pub fn setup_vfn_swarm(&mut self) {
        self.vfn_swarm = Some(
            LibraSwarm::configure_fn_swarm(
                &workspace_builder::get_libra_node_with_failpoints(),
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::ValidatorFullnode,
            )
            .unwrap(),
        );
    }
}

fn get_client_proxy(
    libra_root_key_path: String,
    mnemonic_file_path: PathBuf,
    swarm: &LibraSwarm,
    node_index: usize,
    waypoint: Option<Waypoint>,
) -> ClientProxy {
    let port = swarm.get_client_port(node_index);

    let mnemonic_file_path = mnemonic_file_path
        .canonicalize()
        .expect("Unable to get canonical path of mnemonic_file_path")
        .to_str()
        .unwrap()
        .to_string();

    ClientProxy::new(
        ChainId::test(),
        &format!("http://localhost:{}/v1", port),
        &libra_root_key_path,
        &libra_root_key_path,
        &libra_root_key_path,
        false,
        /* faucet server */ None,
        Some(mnemonic_file_path),
        waypoint.unwrap_or_else(|| swarm.config.waypoint),
    )
    .unwrap()
}
