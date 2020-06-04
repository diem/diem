// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{layout::Layout, storage_helper::StorageHelper};
use config_builder::{BuildSwarm, SwarmConfig};
use libra_config::config::{
    DiscoveryMethod, IdentityKey, NetworkConfig, NodeConfig, OnDiskStorageConfig, RoleType,
    SecureBackend,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_secure_storage::Value;
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
    LibraNode::prepare();
    let helper = StorageHelper::new();
    let num_validators = 5;
    let shared = "_shared";
    let association = "association";
    let association_shared = association.to_string() + shared;

    // Step 1) Prepare the layout
    let mut layout = Layout::default();
    layout.association = vec![association_shared.to_string()];
    layout.operators = (0..num_validators)
        .map(|v| (v.to_string() + shared))
        .collect();

    let mut common_storage = helper.storage(crate::constants::COMMON_NS.into());
    let layout_value = Value::String(layout.to_toml().unwrap());
    common_storage
        .set(crate::constants::LAYOUT, layout_value)
        .unwrap();

    // Step 2) Set association key
    helper.initialize(association.into());
    helper
        .association_key(&association, &association_shared)
        .unwrap();

    // Step 3) Prepare validators
    let temppath = TempPath::new();
    temppath.create_as_dir().unwrap();
    let swarm_path = temppath.path().to_path_buf();

    let mut configs = Vec::new();
    for i in 0..num_validators {
        let ns = i.to_string();
        let ns_shared = ns.clone() + shared;
        helper.initialize(ns.clone());

        let operator_key = helper.operator_key(&ns, &ns_shared).unwrap();

        let validator_account = account_address::from_public_key(&operator_key);
        let mut config = NodeConfig::default();

        let mut network = NetworkConfig::default();
        network.discovery_method = DiscoveryMethod::Onchain;
        network.peer_id = validator_account;
        config.validator_network = Some(network);

        let mut network = NetworkConfig::default();
        network.discovery_method = DiscoveryMethod::None;
        config.full_node_networks = vec![network];
        config.randomize_ports();

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        validator_network.identity_key = IdentityKey::from_storage(
            libra_global_constants::VALIDATOR_NETWORK_KEY.into(),
            secure_backend(helper.path(), &swarm_path, &ns, "full_node"),
        );

        let fullnode_network = &mut config.full_node_networks[0];
        let fullnode_network_address = fullnode_network.listen_address.clone();
        fullnode_network.identity_key = IdentityKey::from_storage(
            libra_global_constants::FULLNODE_NETWORK_KEY.into(),
            secure_backend(helper.path(), &swarm_path, &ns, "full_node"),
        );

        configs.push(config);

        helper.operator_key(&ns, &ns_shared).unwrap();
        helper
            .validator_config(
                validator_account,
                validator_network_address,
                fullnode_network_address,
                &ns,
                &ns_shared,
            )
            .unwrap();
    }

    // Step 4) Produce genesis and introduce into node configs
    let genesis_path = TempPath::new();
    genesis_path.create_as_file().unwrap();
    let genesis = helper.genesis(genesis_path.path()).unwrap();

    // Step 5) Introduce waypoint and genesis into the configs and verify along the way
    for (i, mut config) in configs.iter_mut().enumerate() {
        let ns = i.to_string();
        let waypoint = helper.create_waypoint(&ns).unwrap();
        let output = helper.verify_genesis(&ns, genesis_path.path()).unwrap();
        // 4 matches = 5 splits
        assert_eq!(output.split("match").count(), 5);

        config.consensus.safety_rules.backend =
            secure_backend(helper.path(), &swarm_path, &ns, "safety-rules");

        // TODO: this should be exclusively acquired via secure storage
        config.base.waypoint = Some(waypoint);
        config.execution.genesis = Some(genesis.clone());
    }

    // Step 6) Build configuration for Swarm
    let faucet_key = helper
        .storage(association.into())
        .export_private_key(libra_global_constants::ASSOCIATION_KEY)
        .unwrap();
    let management_builder = ManagementBuilder {
        configs,
        faucet_key,
    };

    let mut swarm = LibraSwarm {
        dir: LibraSwarmDir::Temporary(temppath),
        nodes: std::collections::HashMap::new(),
        config: SwarmConfig::build(&management_builder, &swarm_path).unwrap(),
    };

    // Step 7) Launch and exit!
    swarm.launch_attempt(RoleType::Validator, false).unwrap();
}

fn secure_backend(original: &Path, dst_base: &Path, ns: &str, usage: &str) -> SecureBackend {
    let mut dst = dst_base.to_path_buf();
    dst.push(format!("{}_{}", usage, ns));
    std::fs::copy(original, &dst).unwrap();

    let mut storage_config = OnDiskStorageConfig::default();
    storage_config.path = dst;
    storage_config.set_data_dir(PathBuf::from(""));
    storage_config.namespace = Some(ns.into());
    SecureBackend::OnDiskStorage(storage_config)
}
