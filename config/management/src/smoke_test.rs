// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::{COMMON_NS, LAYOUT},
    layout::Layout,
    storage_helper::StorageHelper,
};
use config_builder::{BuildSwarm, SwarmConfig};
use libra_config::config::{
    Identity, NodeConfig, OnDiskStorageConfig, RoleType, SecureBackend, SeedPeersConfig,
    WaypointConfig, HANDSHAKE_VERSION,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_global_constants::ASSOCIATION_KEY;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_swarm::swarm::{LibraNode, LibraSwarm, LibraSwarmDir};
use libra_temppath::TempPath;
use libra_types::account_address;
use std::path::{Path, PathBuf};

struct ManagementBuilder {
    configs: Vec<NodeConfig>,
    faucet_key: Ed25519PrivateKey,
}

impl BuildSwarm for ManagementBuilder {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        Ok((self.configs.clone(), self.faucet_key.clone()))
    }
}

#[test]
fn smoke_test() {
    let num_validators = 5;
    let shared_ns = "_shared";
    let association_ns = "association";
    let association_shared_ns = "assocation_shared";
    LibraNode::prepare();

    // Step 1) Prepare the layout
    let mut layout = Layout::default();
    layout.association = vec![association_shared_ns.to_string()];
    layout.operators = (0..num_validators)
        .map(|v| (v.to_string() + shared_ns))
        .collect();

    let storage_helper = StorageHelper::new();
    let mut common_storage = storage_helper.storage(COMMON_NS.into());
    common_storage
        .set(LAYOUT, Value::String(layout.to_toml().unwrap()))
        .unwrap();

    // Step 2) Set association key
    storage_helper.initialize(association_ns.into());
    storage_helper
        .association_key(&association_ns, &association_shared_ns)
        .unwrap();

    // Step 3) Prepare validators
    let swarm_path = create_temp_path(false);

    let mut configs = Vec::new();
    for validator_index in 0..num_validators {
        let local_ns = validator_index.to_string();
        let shared_ns = local_ns.clone() + shared_ns;
        storage_helper.initialize(local_ns.clone());

        let operator_key = storage_helper.operator_key(&local_ns, &shared_ns).unwrap();
        storage_helper.operator_key(&local_ns, &shared_ns).unwrap();

        let node_config = create_validator_node_config(&storage_helper, &swarm_path, &local_ns);
        configs.push(node_config.clone());

        storage_helper
            .validator_config(
                account_address::from_public_key(&operator_key),
                node_config
                    .validator_network
                    .unwrap()
                    .listen_address
                    .clone(),
                node_config.full_node_networks[0].listen_address.clone(),
                &local_ns,
                &shared_ns,
            )
            .unwrap();
    }

    // Step 4) Produce genesis and introduce into node configs
    let genesis_path = create_temp_path(true);
    let genesis = storage_helper.genesis(genesis_path.path()).unwrap();

    // Save the waypoint into shared secure storage so that validators can perform insert_waypoint
    let waypoint = storage_helper.create_waypoint(COMMON_NS).unwrap();

    // Step 5) Introduce waypoint and genesis into the configs and verify along the way
    for (index, mut config) in configs.iter_mut().enumerate() {
        let local_ns = index.to_string();
        storage_helper
            .insert_waypoint(&local_ns, COMMON_NS)
            .unwrap();
        let output = storage_helper
            .verify_genesis(&local_ns, genesis_path.path())
            .unwrap();
        // 4 matches = 5 splits
        assert_eq!(output.split("match").count(), 5);

        config.consensus.safety_rules.backend = secure_backend(
            storage_helper.path(),
            &swarm_path,
            &local_ns,
            "safety-rules",
        );
        config.execution.backend =
            secure_backend(storage_helper.path(), &swarm_path, &local_ns, "execution");

        if index == 0 {
            // This is unfortunate due to the way SwarmConfig works
            config.base.waypoint = WaypointConfig::FromConfig(waypoint);
        } else {
            let backend = secure_backend(storage_helper.path(), &swarm_path, &local_ns, "waypoint");
            config.base.waypoint = WaypointConfig::FromStorage(backend);
        }
        config.execution.genesis = Some(genesis.clone());
        config.execution.genesis_file_location = PathBuf::from("");
    }

    // Step 6) Prepare ecosystem
    let full_node_config = attach_validator_full_node(&mut configs[0]);

    // Step 7) Build configuration for Swarm
    let faucet_key = storage_helper
        .storage(association_ns.into())
        .export_private_key(ASSOCIATION_KEY)
        .unwrap();
    let management_builder = ManagementBuilder {
        configs,
        faucet_key: faucet_key.clone(),
    };

    let mut swarm = create_libra_swarm(swarm_path, management_builder);

    // Step 8) Launch validators
    swarm.launch_attempt(RoleType::Validator, false).unwrap();

    // Step 9) Launch ecosystem
    let management_builder = ManagementBuilder {
        configs: vec![full_node_config],
        faucet_key,
    };

    let swarm_path = create_temp_path(false);
    let mut swarm = create_libra_swarm(swarm_path, management_builder);
    swarm.launch_attempt(RoleType::FullNode, false).unwrap();

    assert!(check_connectivity(&mut swarm, 1));
}

// Creates and returns a temporary path. If as_file is set, creates the path as a file. Otherwise,
// a directory path is returned.
fn create_temp_path(as_file: bool) -> TempPath {
    let temp_path = TempPath::new();
    if as_file {
        temp_path.create_as_file().unwrap();
    } else {
        temp_path.create_as_dir().unwrap();
    }
    temp_path
}

fn create_libra_swarm(output_dir: TempPath, management_builder: ManagementBuilder) -> LibraSwarm {
    let path_buf = output_dir.path().to_path_buf();

    LibraSwarm {
        dir: LibraSwarmDir::Temporary(output_dir),
        nodes: std::collections::HashMap::new(),
        config: SwarmConfig::build(&management_builder, &path_buf).unwrap(),
    }
}

fn create_validator_node_config(
    storage_helper: &StorageHelper,
    swarm_path: &TempPath,
    local_ns: &str,
) -> NodeConfig {
    let mut config = NodeConfig::default_for_validator();
    config.randomize_ports();

    let validator_network = config.validator_network.as_mut().unwrap();
    validator_network.identity = Identity::from_storage(
        libra_global_constants::VALIDATOR_NETWORK_KEY.into(),
        libra_global_constants::OPERATOR_ACCOUNT.into(),
        secure_backend(storage_helper.path(), &swarm_path, &local_ns, "validator"),
    );

    let fullnode_network = &mut config.full_node_networks[0];
    fullnode_network.identity = Identity::from_storage(
        libra_global_constants::FULLNODE_NETWORK_KEY.into(),
        libra_global_constants::OPERATOR_ACCOUNT.into(),
        secure_backend(storage_helper.path(), &swarm_path, &local_ns, "full_node"),
    );

    config
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

fn secure_backend(original: &Path, dst_base: &TempPath, ns: &str, usage: &str) -> SecureBackend {
    let mut dst = dst_base.path().to_path_buf();
    dst.push(format!("{}_{}", usage, ns));
    std::fs::copy(original, &dst).unwrap();

    let mut storage_config = OnDiskStorageConfig::default();
    storage_config.path = dst;
    storage_config.set_data_dir(PathBuf::from(""));
    storage_config.namespace = Some(ns.into());
    SecureBackend::OnDiskStorage(storage_config)
}
