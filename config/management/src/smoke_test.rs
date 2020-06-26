// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, layout::Layout, storage_helper::StorageHelper};
use config_builder::{BuildSwarm, SwarmConfig};
use libra_config::config::{
    Identity, NodeConfig, OnDiskStorageConfig, RoleType, SecureBackend, SeedPeersConfig,
    WaypointConfig, HANDSHAKE_VERSION,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_swarm::swarm::{LibraNode, LibraSwarm, LibraSwarmDir};
use libra_temppath::TempPath;
use libra_types::account_address;
use libra_types::waypoint::Waypoint;
use std::path::{Path, PathBuf};

const SHARED: &str = "_shared";
const ASSOCIATION: &str = "association";
const ASSOCIATION_SHARED: &str = "association_shared";

struct ManagementBuilder {
    storage_helper: StorageHelper,
    num_validators: usize,
    swarm_path: TempPath,
}

impl ManagementBuilder {
    pub fn new(num_validators: usize) -> Self {
        let swarm_path = TempPath::new();
        swarm_path.create_as_dir().unwrap();
        Self {
            storage_helper: StorageHelper::new(),
            num_validators,
            swarm_path,
        }
    }

    fn secure_backend(&self, ns: &str, usage: &str) -> SecureBackend {
        let original = self.storage_helper.path();
        let dst_base = self.swarm_path.path();
        let mut dst = dst_base.to_path_buf();
        dst.push(format!("{}_{}", usage, ns));
        std::fs::copy(original, &dst).unwrap();

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.path = dst;
        storage_config.set_data_dir(PathBuf::from(""));
        storage_config.namespace = Some(ns.into());
        SecureBackend::OnDiskStorage(storage_config)
    }

    /// Association uploads the validator layout to shared storage.
    fn create_layout(&self) {
        let mut layout = Layout::default();
        layout.association = vec![ASSOCIATION_SHARED.into()];
        layout.operators = (0..self.num_validators)
            .map(|v| (v.to_string() + SHARED))
            .collect();

        let mut common_storage = self
            .storage_helper
            .storage(crate::constants::COMMON_NS.into());
        let layout_value = Value::String(layout.to_toml().unwrap());
        common_storage
            .set(crate::constants::LAYOUT, layout_value)
            .unwrap();
    }

    /// Association initializes its account and key.
    fn create_association(&self) {
        self.storage_helper.initialize(ASSOCIATION.into());
        self.storage_helper
            .association_key(ASSOCIATION, ASSOCIATION_SHARED)
            .unwrap();
    }

    /// Operators upload their keys to shared storage
    /// Operators initialize their validator endpoint and full node endpoint
    /// Operators upload their validator_config to shared storage
    fn initialize_validator_config(&self, index: usize) -> NodeConfig {
        let ns = index.to_string();
        let ns_shared = ns.clone() + SHARED;
        self.storage_helper.initialize(ns.clone());

        let operator_key = self.storage_helper.operator_key(&ns, &ns_shared).unwrap();

        let validator_account = account_address::from_public_key(&operator_key);
        let mut config = NodeConfig::default_for_validator();
        config.randomize_ports();

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        validator_network.identity = Identity::from_storage(
            libra_global_constants::VALIDATOR_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            self.secure_backend(&ns, "validator"),
        );

        let fullnode_network = &mut config.full_node_networks[0];
        let fullnode_network_address = fullnode_network.listen_address.clone();
        fullnode_network.identity = Identity::from_storage(
            libra_global_constants::FULLNODE_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            self.secure_backend(&ns, "full_node"),
        );

        self.storage_helper
            .validator_config(
                validator_account,
                validator_network_address,
                fullnode_network_address,
                &ns,
                &ns_shared,
            )
            .unwrap();
        config
    }

    /// Operators generate genesis from shared storage and verify against waypoint.
    /// Insert the genesis/waypoint into local config.
    fn finish_validator_config(&self, index: usize, config: &mut NodeConfig, waypoint: Waypoint) {
        let ns = index.to_string();
        let genesis_path = TempPath::new();
        genesis_path.create_as_file().unwrap();
        let genesis = self.storage_helper.genesis(genesis_path.path()).unwrap();
        self.storage_helper
            .insert_waypoint(&ns, constants::COMMON_NS)
            .unwrap();
        let output = self
            .storage_helper
            .verify_genesis(&ns, genesis_path.path())
            .unwrap();
        assert_eq!(output.split("match").count(), self.num_validators);

        config.consensus.safety_rules.backend = self.secure_backend(&ns, "safety-rules");
        config.execution.backend = self.secure_backend(&ns, "execution");

        if index == 0 {
            // This is unfortunate due to the way SwarmConfig works
            config.base.waypoint = WaypointConfig::FromConfig(waypoint);
        } else {
            let backend = self.secure_backend(&ns, "waypoint");
            config.base.waypoint = WaypointConfig::FromStorage(backend);
        }
        config.execution.genesis = Some(genesis.clone());
        config.execution.genesis_file_location = PathBuf::from("");
    }

    fn create_swarm(self) -> LibraSwarm {
        let config = SwarmConfig::build(&self, &self.swarm_path.path().to_path_buf()).unwrap();
        LibraSwarm {
            dir: LibraSwarmDir::Temporary(self.swarm_path),
            nodes: std::collections::HashMap::new(),
            config,
        }
    }
}

impl BuildSwarm for ManagementBuilder {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        self.create_layout();
        self.create_association();
        let faucet_key = self
            .storage_helper
            .storage(ASSOCIATION.into())
            .export_private_key(libra_global_constants::ASSOCIATION_KEY)
            .unwrap();
        let mut configs: Vec<_> = (0..self.num_validators)
            .map(|i| self.initialize_validator_config(i))
            .collect();
        let waypoint = self
            .storage_helper
            .create_waypoint(constants::COMMON_NS)
            .unwrap();
        configs
            .iter_mut()
            .enumerate()
            .for_each(|(i, config)| self.finish_validator_config(i, config, waypoint));
        Ok((configs, faucet_key))
    }
}

#[test]
fn smoke_test() {
    LibraNode::prepare();
    let mut swarm = ManagementBuilder::new(5).create_swarm();
    swarm.launch_attempt(RoleType::Validator, false).unwrap();
}

fn check_connectivity(swarm: &mut LibraSwarm, expected_peers: i64) -> bool {
    for node in swarm.nodes.iter_mut() {
        let mut timed_out = true;
        for i in 0..60 {
            println!("Wait for connectivity attempt: {} {}", node.0, i);
            std::thread::sleep(std::time::Duration::from_secs(1));
            if node.1.check_connectivity(expected_peers) {
                timed_out = false;
                break;
            }
        }
        if timed_out {
            return false;
        }
    }
    true
}

fn attach_validator_full_node(validator_config: &mut NodeConfig) -> NodeConfig {
    // Create two vfns, we'll pass one to the validator later
    let mut full_node_config = NodeConfig::default_for_validator_full_node();
    full_node_config.randomize_ports();

    // The FN's first network is the external, public network, it needs to swap listen addresses
    // with the validator's VFN and to copy it's key access:
    let pfn = &mut full_node_config.full_node_networks[0];
    let v_vfn = &mut validator_config.full_node_networks[0];
    pfn.identity = v_vfn.identity.clone();
    let temp_listen = v_vfn.listen_address.clone();
    v_vfn.listen_address = pfn.listen_address.clone();
    pfn.listen_address = temp_listen;

    // Now let's prepare the full nodes internal network to communicate with the validators
    // internal network

    let v_vfn_network_address = v_vfn.listen_address.clone();
    let v_vfn_pub_key = v_vfn.identity_key().public_key();
    let v_vfn_network_address =
        v_vfn_network_address.append_prod_protos(v_vfn_pub_key, HANDSHAKE_VERSION);
    let v_vfn_id = v_vfn.peer_id();
    let mut seed_peers = SeedPeersConfig::default();
    seed_peers.insert(v_vfn_id, vec![v_vfn_network_address]);

    let fn_vfn = &mut full_node_config.full_node_networks[1];
    fn_vfn.seed_peers = seed_peers;

    full_node_config.base.waypoint = validator_config.base.waypoint.clone();
    full_node_config.execution.genesis = validator_config.execution.genesis.clone();
    full_node_config.execution.genesis_file_location = PathBuf::from("");
    full_node_config
}
