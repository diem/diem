// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Libra ndoes without the
//! genesis.blob.

use crate::config::{NetworkConfig, NodeConfig, SeedPeersConfig, TestConfig, HANDSHAKE_VERSION};
use libra_network_address::NetworkAddress;
use rand::{rngs::StdRng, SeedableRng};

pub struct ValidatorSwarm {
    pub nodes: Vec<NodeConfig>,
}

pub fn validator_swarm(
    template: &NodeConfig,
    count: usize,
    seed: [u8; 32],
    randomize_ports: bool,
) -> ValidatorSwarm {
    let mut rng = StdRng::from_seed(seed);
    let mut nodes = Vec::new();

    for _index in 0..count {
        let mut node = NodeConfig::random_with_template(template, &mut rng);
        if randomize_ports {
            node.randomize_ports();
        }

        // For a validator node, any of its validator peers are considered an upstream peer
        let network = node.validator_network.as_mut().unwrap();
        node.upstream.primary_networks.push(network.peer_id);

        nodes.push(node);
    }

    // set the first validator as every validators' initial configured seed peer.
    let seed_config = &nodes[0].validator_network.as_ref().unwrap();
    let seed_peers = build_seed_peers(&seed_config, seed_config.advertised_address.clone());
    for node in &mut nodes {
        let network = node.validator_network.as_mut().unwrap();
        network.seed_peers = seed_peers.clone();
    }

    ValidatorSwarm { nodes }
}

pub fn validator_swarm_for_testing(nodes: usize) -> ValidatorSwarm {
    let mut config = NodeConfig::default();
    config.test = Some(TestConfig::open_module());
    validator_swarm(&NodeConfig::default(), nodes, [1u8; 32], true)
}

/// Convenience function that builds a `SeedPeersConfig` containing a single peer
/// with a fully formatted `NetworkAddress` containing its network identity pubkey
/// and handshake protocol version.
pub fn build_seed_peers(
    seed_config: &NetworkConfig,
    seed_base_addr: NetworkAddress,
) -> SeedPeersConfig {
    let seed_pubkey = seed_config
        .network_keypairs
        .as_ref()
        .expect("Expect network keypairs")
        .identity_keypair
        .public_key();
    let seed_addr = seed_base_addr.append_prod_protos(seed_pubkey, HANDSHAKE_VERSION);

    let mut seed_peers = SeedPeersConfig::default();
    seed_peers
        .seed_peers
        .insert(seed_config.peer_id, vec![seed_addr]);
    seed_peers
        .verify_libranet_addrs()
        .expect("Expect LibraNet addresses");
    seed_peers
}
