// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{layout::Layout, storage_helper::StorageHelper};
use config_builder::{BuildSwarm, SwarmConfig};
use libra_config::config::{
    DiscoveryMethod, NetworkConfig, NetworkKeyPairs, NodeConfig, OnDiskStorageConfig, RoleType,
    SecureBackend,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, x25519, Uniform};
use libra_secure_storage::Value;
use libra_swarm::swarm::{LibraSwarm, LibraSwarmDir};
use libra_temppath::TempPath;
use libra_types::account_address::AccountAddress;
use std::{convert::TryInto, path::PathBuf};

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
    let mut configs = Vec::new();
    for i in 0..num_validators {
        let storage = helper.storage(i.to_string());
        let ns = i.to_string();
        let ns_shared = ns.clone() + shared;
        helper.initialize(ns.clone());

        let mut config = NodeConfig::default();

        let mut network = NetworkConfig::default();
        network.discovery_method = DiscoveryMethod::Onchain;
        config.validator_network = Some(network);

        let mut network = NetworkConfig::default();
        network.discovery_method = DiscoveryMethod::None;
        config.full_node_networks = vec![network];
        config.randomize_ports();

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        let identity_key = storage
            .export_private_key(libra_global_constants::VALIDATOR_NETWORK_KEY)
            .unwrap();
        let identity_key: x25519::PrivateKey = identity_key.to_bytes().as_ref().try_into().unwrap();
        // This is a deprecated field
        let signing_key = Ed25519PrivateKey::generate_for_testing();
        validator_network.network_keypairs = Some(NetworkKeyPairs::load(identity_key, signing_key));

        let fullnode_network = &mut config.full_node_networks[0];
        let identity_key = storage
            .export_private_key(libra_global_constants::FULLNODE_NETWORK_KEY)
            .unwrap();
        let identity_key = identity_key.to_bytes().as_ref().try_into().unwrap();
        // This is a deprecated field
        let signing_key = Ed25519PrivateKey::generate_for_testing();
        fullnode_network.network_keypairs = Some(NetworkKeyPairs::load(identity_key, signing_key));
        let fullnode_network_address = fullnode_network.listen_address.clone();

        configs.push(config);

        helper.operator_key(&ns, &ns_shared).unwrap();
        helper
            .validator_config(
                AccountAddress::random(),
                validator_network_address,
                fullnode_network_address,
                &ns,
                &ns_shared,
            )
            .unwrap();
    }

    // Step 4) Produce genesis and introduce into node configs
    let genesis = helper.genesis().unwrap();

    // Step 6) Introduce waypoint and genesis into the configs
    let temppath = TempPath::new();
    temppath.create_as_dir().unwrap();
    let swarm_path = temppath.path().to_path_buf();

    for (i, mut config) in configs.iter_mut().enumerate() {
        let ns = i.to_string();
        let waypoint = helper.create_waypoint(&ns).unwrap();

        let mut to = swarm_path.clone();
        to.push("secure_store_for_".to_string() + &ns);
        std::fs::copy(helper.path_string(), &to).unwrap();

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.path = to;
        storage_config.set_data_dir(PathBuf::from(""));
        storage_config.namespace = Some(ns);
        config.consensus.safety_rules.backend = SecureBackend::OnDiskStorage(storage_config);

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

    println!("{:?}", swarm_path);
    let mut swarm = LibraSwarm {
        dir: LibraSwarmDir::Temporary(temppath),
        nodes: std::collections::HashMap::new(),
        config: SwarmConfig::build(&management_builder, &swarm_path).unwrap(),
    };

    // Step 7) Launch and exit!
    swarm.launch_attempt(RoleType::Validator, false).unwrap();
}
