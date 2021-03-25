// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils::diem_swarm_utils::get_client_proxy, workspace_builder};
use cli::client_proxy::ClientProxy;
use diem_config::config::NodeConfig;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_genesis_tool::{config_builder::FullnodeType, swarm_config::SwarmConfig};
use diem_infallible::Mutex;
use diem_swarm::swarm::DiemSwarm;
use diem_temppath::TempPath;
use diem_types::waypoint::Waypoint;
use std::{collections::HashMap, sync::Arc};

/// A way to get us to have multiple full node swarms in the environment
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FullNodeSwarmType {
    Validator,
    Public(&'static str),
}

pub struct SmokeTestEnvironment {
    pub validator_swarm: DiemSwarm,
    /// A list of full node swarms.  Uses an `Arc<Mutex` to not have to deal with mutability
    fn_swarms: HashMap<FullNodeSwarmType, Arc<Mutex<DiemSwarm>>>,
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
            fn_swarms: HashMap::new(),
            diem_root_key: (key, key_path),
            mnemonic_file,
        }
    }
    pub fn new(num_validators: usize) -> Self {
        Self::new_with_chunk_limit(num_validators, 10)
    }

    pub fn setup_vfn_swarm(&mut self) {
        let swarm_key = FullNodeSwarmType::Validator;
        if self.fn_swarms.contains_key(&swarm_key) {
            panic!("Already setup VFN swarm");
        }

        let swarm = DiemSwarm::configure_fn_swarm(
            "ValidatorFullNode",
            &workspace_builder::get_diem_node_with_failpoints(),
            None,
            None,
            &self.validator_swarm.config,
            FullnodeType::ValidatorFullnode,
        )
        .unwrap();
        self.add_fn_swarm(swarm_key, swarm);
    }

    pub fn add_public_fn_swarm(
        &mut self,
        name: &'static str,
        num_nodes: usize,
        template: Option<NodeConfig>,
        upstream_swarm: &SwarmConfig,
    ) {
        // Let's shortcut it so we don't have to wait for any startup time
        let swarm_key = FullNodeSwarmType::Public(name);
        if self.fn_swarms.contains_key(&swarm_key) {
            panic!("Already setup full node {:?} swarm", swarm_key);
        }
        let swarm = DiemSwarm::configure_fn_swarm(
            name,
            &workspace_builder::get_diem_node_with_failpoints(),
            None,
            template,
            upstream_swarm,
            FullnodeType::PublicFullnode(num_nodes),
        )
        .unwrap();

        self.add_fn_swarm(swarm_key, swarm);
    }

    pub fn add_fn_swarm(&mut self, swarm_key: FullNodeSwarmType, fn_swarm: DiemSwarm) {
        if self.fn_swarms.contains_key(&swarm_key) {
            panic!("Already setup full node {:?} swarm", swarm_key);
        }
        self.fn_swarms
            .insert(swarm_key, Arc::new(Mutex::new(fn_swarm)));
    }

    pub fn fn_swarm(&self, fn_type: FullNodeSwarmType) -> Option<Arc<Mutex<DiemSwarm>>> {
        self.fn_swarms.get(&fn_type).cloned()
    }

    pub fn vfn_swarm(&self) -> Arc<Mutex<DiemSwarm>> {
        self.fn_swarm(FullNodeSwarmType::Validator)
            .expect("VFN swarm is not initialized")
    }

    pub fn public_swarm(&self, name: &'static str) -> Arc<Mutex<DiemSwarm>> {
        self.fn_swarm(FullNodeSwarmType::Public(name))
            .expect("Public fn swarm is not initialized")
    }

    fn get_client(
        &self,
        swarm: &DiemSwarm,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        get_client_proxy(
            swarm,
            node_index,
            &self.diem_root_key.1,
            self.mnemonic_file.path().to_path_buf(),
            waypoint,
        )
    }

    pub fn get_fn_client(
        &self,
        fn_key: FullNodeSwarmType,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        let arc = self.fn_swarm(fn_key).unwrap();
        let swarm = arc.lock();
        self.get_client(&swarm, node_index, waypoint)
    }

    pub fn get_validator_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        self.get_client(&self.validator_swarm, node_index, waypoint)
    }

    pub fn get_vfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        self.get_fn_client(FullNodeSwarmType::Validator, node_index, waypoint)
    }

    pub fn get_pfn_client(
        &self,
        name: &'static str,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        self.get_fn_client(FullNodeSwarmType::Public(name), node_index, waypoint)
    }
}
