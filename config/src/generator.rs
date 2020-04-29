// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Libra ndoes without the
//! genesis.blob.

use crate::{
    config::{
        NodeConfig, OnDiskStorageConfig, SafetyRulesBackend, SafetyRulesService, SeedPeersConfig,
        TestConfig,
    },
    utils,
};
use rand::{rngs::StdRng, SeedableRng};

pub struct ValidatorSwarm {
    pub nodes: Vec<NodeConfig>,
}

pub fn validator_swarm(
    template: &NodeConfig,
    count: usize,
    seed: [u8; 32],
    randomize_service_ports: bool,
    randomize_libranet_ports: bool,
) -> ValidatorSwarm {
    let mut rng = StdRng::from_seed(seed);
    let mut nodes = Vec::new();

    for _index in 0..count {
        let mut node = NodeConfig::random_with_template(template, &mut rng);
        if randomize_service_ports {
            node.randomize_ports();
        }

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.default = true;
        node.consensus.safety_rules.service = SafetyRulesService::Thread;
        node.consensus.safety_rules.backend = SafetyRulesBackend::OnDiskStorage(storage_config);

        let network = node.validator_network.as_mut().unwrap();
        if randomize_libranet_ports {
            network.listen_address = utils::get_available_port_in_multiaddr(true);
            network.advertised_address = network.listen_address.clone();
        }

        // For a validator node, any of its validator peers are considered an upstream peer
        node.upstream.primary_networks.push(network.peer_id);

        nodes.push(node);
    }

    let mut seed_peers = SeedPeersConfig::default();
    let network = nodes[0].validator_network.as_ref().unwrap();
    seed_peers
        .seed_peers
        .insert(network.peer_id, vec![network.listen_address.clone()]);

    for node in &mut nodes {
        let network = node.validator_network.as_mut().unwrap();
        network.seed_peers = seed_peers.clone();
    }

    ValidatorSwarm { nodes }
}

pub fn validator_swarm_for_testing(nodes: usize) -> ValidatorSwarm {
    let mut config = NodeConfig::default();
    config.test = Some(TestConfig::open_module());
    validator_swarm(&NodeConfig::default(), nodes, [1u8; 32], true, true)
}
