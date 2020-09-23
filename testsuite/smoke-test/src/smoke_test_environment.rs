// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::workspace_builder;
use cli::client_proxy::ClientProxy;
use debug_interface::NodeDebugClient;
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_genesis_tool::config_builder::FullnodeType;
use libra_operational_tool::test_helper::OperationalTool;
use libra_swarm::swarm::{LibraNode, LibraSwarm};
use libra_temppath::TempPath;
use libra_types::{chain_id::ChainId, waypoint::Waypoint};

pub struct SmokeTestEnvironment {
    pub validator_swarm: LibraSwarm,
    pub vfn_swarm: Option<LibraSwarm>,
    pub public_fn_swarm: Option<LibraSwarm>,
    libra_root_key: (Ed25519PrivateKey, String),
    mnemonic_file: TempPath,
}

impl SmokeTestEnvironment {
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
    pub fn new(num_validators: usize) -> Self {
        Self::new_with_chunk_limit(num_validators, 10)
    }

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

    pub fn setup_public_fn_swarm(&mut self, num_nodes: usize) {
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

    fn get_json_rpc_client(&self, port: u16, waypoint: Option<Waypoint>) -> ClientProxy {
        let mnemonic_file_path = self
            .mnemonic_file
            .path()
            .to_path_buf()
            .canonicalize()
            .expect("Unable to get canonical path of mnemonic_file_path")
            .to_str()
            .unwrap()
            .to_string();

        ClientProxy::new(
            ChainId::test(),
            &format!("http://localhost:{}/v1", port),
            &self.libra_root_key.1,
            &self.libra_root_key.1,
            &self.libra_root_key.1,
            false,
            /* faucet server */ None,
            Some(mnemonic_file_path),
            waypoint.unwrap_or_else(|| self.validator_swarm.config.waypoint),
        )
        .unwrap()
    }

    fn get_client(
        &self,
        swarm: &LibraSwarm,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        let port = swarm.get_client_port(node_index);
        self.get_json_rpc_client(port, waypoint)
    }

    pub fn get_validator_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        self.get_client(&self.validator_swarm, node_index, waypoint)
    }

    pub fn get_validator_debug_interface_client(&self, node_index: usize) -> NodeDebugClient {
        let port = self.validator_swarm.get_validators_debug_ports()[node_index];
        NodeDebugClient::new("localhost", port)
    }

    #[allow(dead_code)]
    fn get_validator_debug_interface_clients(&self) -> Vec<NodeDebugClient> {
        self.validator_swarm
            .get_validators_debug_ports()
            .iter()
            .map(|port| NodeDebugClient::new("localhost", *port))
            .collect()
    }

    pub fn get_vfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        self.get_client(
            self.vfn_swarm
                .as_ref()
                .expect("Vfn swarm is not initialized"),
            node_index,
            waypoint,
        )
    }

    pub fn get_public_fn_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        self.get_client(
            self.public_fn_swarm
                .as_ref()
                .expect("Public fn swarm is not initialized"),
            node_index,
            waypoint,
        )
    }

    pub fn get_validator(&self, node_index: usize) -> Option<&LibraNode> {
        self.validator_swarm.get_validator(node_index)
    }

    pub fn get_op_tool(&self, node_index: usize) -> OperationalTool {
        OperationalTool::new(
            format!(
                "http://127.0.0.1:{}",
                self.validator_swarm.get_client_port(node_index)
            ),
            ChainId::test(),
        )
    }
}
