// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Libra ndoes without the
//! genesis.blob.

use crate::{
    config::{
        ConsensusPeerInfo, ConsensusPeersConfig, NetworkPeersConfig, NodeConfig,
        OnDiskStorageConfig, SafetyRulesBackend, SeedPeersConfig, VMPublishingOption,
    },
    utils,
};
use rand::{rngs::StdRng, SeedableRng};

pub fn validator_swarm(
    template: &NodeConfig,
    nodes: usize,
    seed: [u8; 32],
    randomize_ports: bool,
) -> Vec<NodeConfig> {
    let mut rng = StdRng::from_seed(seed);
    let mut network_peers = NetworkPeersConfig::default();
    let mut consensus_peers = ConsensusPeersConfig::default();
    let mut configs = Vec::new();

    for _index in 0..nodes {
        let mut config = NodeConfig::random_with_template(template, &mut rng);
        if randomize_ports {
            config.randomize_ports();
        }

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.default = true;
        config.consensus.safety_rules.backend = SafetyRulesBackend::OnDiskStorage(storage_config);

        let network = config.validator_network.as_mut().unwrap();
        network.listen_address = utils::get_available_port_in_multiaddr(true);
        network.advertised_address = network.listen_address.clone();

        network_peers
            .peers
            .insert(network.peer_id, network.network_keypairs.as_peer_info());

        consensus_peers.peers.insert(
            network.peer_id,
            ConsensusPeerInfo {
                consensus_pubkey: config.consensus.consensus_keypair.public().clone(),
            },
        );

        configs.push(config);
    }

    let mut seed_peers = SeedPeersConfig::default();
    let network = configs[0].validator_network.as_ref().unwrap();
    seed_peers
        .seed_peers
        .insert(network.peer_id, vec![network.listen_address.clone()]);

    for config in &mut configs {
        config.consensus.consensus_peers = consensus_peers.clone();
        let network = config.validator_network.as_mut().unwrap();
        network.network_peers = network_peers.clone();
        network.seed_peers = seed_peers.clone();
    }

    configs
}

pub fn validator_swarm_for_testing(nodes: usize) -> Vec<NodeConfig> {
    let mut config = NodeConfig::default();
    config.vm_config.publishing_options = VMPublishingOption::Open;
    validator_swarm(&NodeConfig::default(), nodes, [1u8; 32], true)
}
