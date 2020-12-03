// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils::diem_swarm_utils::get_client_proxy, workspace_builder};
use cli::client_proxy::ClientProxy;
use diem_config::config::NodeConfig;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_genesis_tool::config_builder::FullnodeType;
use diem_swarm::swarm::DiemSwarm;
use diem_temppath::TempPath;
use diem_types::waypoint::Waypoint;

pub struct SmokeTestEnvironment {
    pub validator_swarm: DiemSwarm,
    pub vfn_swarm: Option<DiemSwarm>,
    pub pfn_swarm: Option<DiemSwarm>,
    diem_root_key: (Ed25519PrivateKey, String),
    mnemonic_file: TempPath,
}

impl SmokeTestEnvironment {
    pub fn new_with_chunk_limit(num_validators: usize, chunk_limit: u64) -> Self {
        ::diem_logger::Logger::new().init();
        let mut template = NodeConfig::default_for_validator();
        template.state_sync.chunk_limit = chunk_limit;

        let validator_swarm = DiemSwarm::configure_validator_swarm(
            &workspace_builder::get_diem_node_with_failpoints(),
            num_validators,
            None,
            Some(template),
        )
        .unwrap();

        let mnemonic_file = diem_temppath::TempPath::new();
        mnemonic_file
            .create_as_file()
            .expect("could not create temporary mnemonic_file_path");

        let key = generate_key::load_key(&validator_swarm.config.diem_root_key_path);
        let key_path = validator_swarm
            .config
            .diem_root_key_path
            .to_str()
            .expect("Unable to read faucet path")
            .to_string();

        Self {
            validator_swarm,
            vfn_swarm: None,
            pfn_swarm: None,
            diem_root_key: (key, key_path),
            mnemonic_file,
        }
    }
    pub fn new(num_validators: usize) -> Self {
        Self::new_with_chunk_limit(num_validators, 10)
    }

    pub fn setup_vfn_swarm(&mut self) {
        self.vfn_swarm = Some(
            DiemSwarm::configure_fn_swarm(
                &workspace_builder::get_diem_node_with_failpoints(),
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::ValidatorFullnode,
            )
            .unwrap(),
        );
    }

    pub fn setup_pfn_swarm(&mut self, num_nodes: usize) {
        self.pfn_swarm = Some(
            DiemSwarm::configure_fn_swarm(
                &workspace_builder::get_diem_node_with_failpoints(),
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::PublicFullnode(num_nodes),
            )
            .unwrap(),
        );
    }

    pub fn get_validator_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        get_client_proxy(
            &self.validator_swarm,
            node_index,
            &self.diem_root_key.1,
            self.mnemonic_file.path().to_path_buf(),
            waypoint,
        )
    }

    pub fn get_vfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        get_client_proxy(
            self.vfn_swarm
                .as_ref()
                .expect("Vfn swarm is not initialized"),
            node_index,
            &self.diem_root_key.1,
            self.mnemonic_file.path().to_path_buf(),
            waypoint,
        )
    }

    pub fn get_pfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        get_client_proxy(
            self.pfn_swarm
                .as_ref()
                .expect("Public fn swarm is not initialized"),
            node_index,
            &self.diem_root_key.1,
            self.mnemonic_file.path().to_path_buf(),
            waypoint,
        )
    }
}
