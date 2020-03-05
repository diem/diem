// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Libra ndoes without the
//! genesis.blob.

use crate::config::SafetyRulesService;
use crate::{
    config::{
        NodeConfig, OnDiskStorageConfig, SafetyRulesBackend, SeedPeersConfig, VMPublishingOption,
    },
    utils,
};
use libra_types::crypto_proxies::{ValidatorPublicKeys, ValidatorSet};
use rand::{rngs::StdRng, SeedableRng};

pub struct ValidatorSwarm {
    pub nodes: Vec<NodeConfig>,
    pub validator_set: ValidatorSet,
}

pub fn validator_swarm(
    template: &NodeConfig,
    count: usize,
    seed: [u8; 32],
    randomize_ports: bool,
) -> ValidatorSwarm {
    let mut rng = StdRng::from_seed(seed);
    let mut validator_keys = Vec::new();
    let mut nodes = Vec::new();

    for _index in 0..count {
        let mut node = NodeConfig::random_with_template(template, &mut rng);
        if randomize_ports {
            node.randomize_ports();
        }

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.default = true;
        node.consensus.safety_rules.service = SafetyRulesService::Thread;
        node.consensus.safety_rules.backend = SafetyRulesBackend::OnDiskStorage(storage_config);

        let network = node.validator_network.as_mut().unwrap();
        network.listen_address = utils::get_available_port_in_multiaddr(true);
        network.advertised_address = network.listen_address.clone();

        let test = node.test.as_ref().unwrap();
        let consensus_pubkey = test.consensus_keypair.as_ref().unwrap().public().clone();
        let network_keypairs = network
            .network_keypairs
            .as_ref()
            .expect("Network keypairs are not defined");

        validator_keys.push(ValidatorPublicKeys::new(
            network.peer_id,
            consensus_pubkey,
            1, // @TODO: Add support for dynamic weights
            network_keypairs.signing_keys.public().clone(),
            network_keypairs.identity_keys.public().clone(),
        ));

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

    validator_keys.sort_by(|k1, k2| k1.account_address().cmp(k2.account_address()));
    ValidatorSwarm {
        nodes,
        validator_set: ValidatorSet::new(validator_keys),
    }
}

pub fn validator_swarm_for_testing(nodes: usize) -> ValidatorSwarm {
    let mut config = NodeConfig::default();
    config.vm_config.publishing_options = VMPublishingOption::Open;
    validator_swarm(&NodeConfig::default(), nodes, [1u8; 32], true)
}
