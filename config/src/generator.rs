// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Diem ndoes without the
//! genesis.blob.

use crate::{
    config::{
        DiscoveryMethod, NetworkConfig, NodeConfig, SeedAddresses, TestConfig, HANDSHAKE_VERSION,
    },
    network_id::NetworkId,
};
use diem_network_address::NetworkAddress;
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

    for index in 0..count {
        let mut node = NodeConfig::random_with_template(index as u32, template, &mut rng);
        if randomize_ports {
            node.randomize_ports();
        }

        // For a validator node, any of its validator peers are considered an upstream peer
        let network = node.validator_network.as_mut().unwrap();
        network.discovery_method = DiscoveryMethod::Onchain;
        network.mutual_authentication = true;
        network.network_id = NetworkId::Validator;

        nodes.push(node);
    }

    // set the first validator as every validators' initial configured seed peer.
    let seed_config = &nodes[0].validator_network.as_ref().unwrap();
    let seed_addrs = build_seed_addrs(&seed_config, seed_config.listen_address.clone());
    for node in &mut nodes {
        let network = node.validator_network.as_mut().unwrap();
        network.seed_addrs = seed_addrs.clone();
    }

    ValidatorSwarm { nodes }
}

pub fn validator_swarm_for_testing(nodes: usize) -> ValidatorSwarm {
    let mut config = NodeConfig::default();
    config.test = Some(TestConfig::open_module());
    validator_swarm(&NodeConfig::default(), nodes, [1u8; 32], true)
}

/// Convenience function that builds a `SeedAddresses` containing a single peer
/// with a fully formatted `NetworkAddress` containing its network identity pubkey
/// and handshake protocol version.
pub fn build_seed_addrs(
    seed_config: &NetworkConfig,
    seed_base_addr: NetworkAddress,
) -> SeedAddresses {
    let seed_pubkey = diem_crypto::PrivateKey::public_key(&seed_config.identity_key());
    let seed_addr = seed_base_addr.append_prod_protos(seed_pubkey, HANDSHAKE_VERSION);

    let mut seed_addrs = SeedAddresses::default();
    seed_addrs.insert(seed_config.peer_id(), vec![seed_addr]);
    seed_addrs
}
