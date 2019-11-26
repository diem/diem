// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating a random set of Libra ndoes without the
//! genesis.blob.
use crate::{
    config::{
        NetworkConfig, NodeConfig, OnDiskStorageConfig, RoleType, SafetyRulesBackend,
        SeedPeersConfig, VMPublishingOption,
    },
    trusted_peers::{
        ConsensusPeerInfo, ConsensusPeersConfig, NetworkPeerInfo, NetworkPeersConfig,
        UpstreamPeersConfig,
    },
    utils,
};
use failure::Result;
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::HashMap, path::PathBuf};

/// Produces a new set of FullNodes that connect to the specified upstream peer. The resulting
/// configs copy all relevant data from this peer including the data used for generating the
/// ValidatorVerifier. It will also copy the genesis.blob if it is Some.
pub fn full_node_swarm(
    upstream_peer: &mut NodeConfig,
    mut template: NodeConfig,
    num_nodes: usize,
    prune_seed_peers: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
    is_permissioned: bool,
) -> Result<Vec<NodeConfig>> {
    init(&mut template, is_ipv4);
    let seed = key_seed.unwrap_or([0u8; 32]);
    let mut rng = StdRng::from_seed(seed);

    let mut upstream_network = if let Some(network) = template.full_node_networks.get(0) {
        network.clone_for_template()
    } else {
        NetworkConfig::default()
    };
    upstream_network.is_permissioned = is_permissioned;
    upstream_network.random(&mut rng);

    upstream_network.listen_address = utils::get_available_port_in_multiaddr(is_ipv4);
    upstream_network.advertised_address = upstream_network.listen_address.clone();

    let mut network_peers = NetworkPeersConfig {
        peers: HashMap::new(),
    };
    let mut seed_peers = SeedPeersConfig {
        seed_peers: HashMap::new(),
    };
    add_peer(&upstream_network, &mut network_peers, &mut seed_peers);

    template.set_role(RoleType::FullNode)?;
    template.consensus.consensus_peers = upstream_peer.consensus.consensus_peers.clone();
    template.execution.genesis = upstream_peer.execution.genesis.clone();
    template.state_sync.upstream_peers = UpstreamPeersConfig {
        upstream_peers: vec![upstream_network.peer_id],
    };

    let mut configs = Vec::new();

    for _index in 0..num_nodes {
        let mut config = NodeConfig::random_with_template(&template, &mut rng);
        config.randomize_ports();
        let network = &mut config.full_node_networks[0];
        network.listen_address = utils::get_available_port_in_multiaddr(is_ipv4);
        network.advertised_address = network.listen_address.clone();
        add_peer(&network, &mut network_peers, &mut seed_peers);
        configs.push(config);
    }

    if prune_seed_peers {
        seed_peers.seed_peers = seed_peers.seed_peers.into_iter().take(1).collect();
    }

    for config in &mut configs {
        config.full_node_networks[0].network_peers = network_peers.clone();
        config.full_node_networks[0].seed_peers = seed_peers.clone();
    }

    // Modify upstream peer config to add the new network config.
    upstream_network.base = upstream_peer.base.clone();
    upstream_network.network_peers = network_peers;
    upstream_network.seed_peers = seed_peers;
    upstream_peer.full_node_networks.push(upstream_network);

    Ok(configs)
}

/// Generate a set of validator nodes. This does not include the genesis.blob to eliminate crate
/// dependency issues.
pub fn validator_swarm(
    mut template: NodeConfig,
    num_nodes: usize,
    prune_seed_peers: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
) -> Result<Vec<NodeConfig>> {
    init(&mut template, is_ipv4);
    let seed = key_seed.unwrap_or([1u8; 32]);
    let mut rng = StdRng::from_seed(seed);

    let mut network_peers = NetworkPeersConfig {
        peers: HashMap::new(),
    };
    let mut seed_peers = SeedPeersConfig {
        seed_peers: HashMap::new(),
    };
    let mut consensus_peers = HashMap::new();
    let mut configs = Vec::new();

    for _index in 0..num_nodes {
        let mut config = NodeConfig::random_with_template(&template, &mut rng);
        config.randomize_ports();

        config.consensus.safety_rules.backend =
            SafetyRulesBackend::OnDiskStorage(OnDiskStorageConfig {
                default: true,
                path: PathBuf::from("safety_rules.toml"),
                base: config.base.clone(),
            });

        let network = &mut config.validator_network.as_mut().unwrap();
        network.listen_address = utils::get_available_port_in_multiaddr(is_ipv4);
        network.advertised_address = network.listen_address.clone();
        add_peer(network, &mut network_peers, &mut seed_peers);
        // @TODO the validator scripts expect this to be in this format. This restriction
        // should not be moved upstream into those scripts.
        config.consensus.consensus_keypair_file =
            PathBuf::from(format!("{}.consensus.keys.toml", network.peer_id));
        consensus_peers.insert(
            network.peer_id,
            ConsensusPeerInfo {
                consensus_pubkey: config.consensus.consensus_keypair.public().clone(),
            },
        );

        configs.push(config);
    }

    let consensus_peers_config = ConsensusPeersConfig {
        peers: consensus_peers,
    };

    if prune_seed_peers {
        seed_peers.seed_peers = seed_peers.seed_peers.into_iter().take(1).collect();
    }

    for config in &mut configs {
        config.consensus.consensus_peers = consensus_peers_config.clone();
        let network = config.validator_network.as_mut().unwrap();
        network.network_peers = network_peers.clone();
        network.seed_peers = seed_peers.clone();
    }

    Ok(configs)
}

pub fn validator_swarm_for_testing(num_nodes: usize) -> Result<Vec<NodeConfig>> {
    validator_swarm(NodeConfig::default(), num_nodes, true, true, None)
}

fn add_peer(
    peer: &NetworkConfig,
    network_peers: &mut NetworkPeersConfig,
    seed_peers: &mut SeedPeersConfig,
) {
    seed_peers
        .seed_peers
        .insert(peer.peer_id, vec![peer.listen_address.clone()]);
    network_peers.peers.insert(
        peer.peer_id,
        NetworkPeerInfo {
            network_identity_pubkey: peer.network_keypairs.identity_keys.public().clone(),
            network_signing_pubkey: peer.network_keypairs.signing_keys.public().clone(),
        },
    );
}

fn init(template: &mut NodeConfig, is_ipv4: bool) {
    let listen_address = if is_ipv4 { "0.0.0.0" } else { "::1" };
    let listen_address = listen_address.to_string();
    template.admission_control.address = listen_address.clone();
    template.debug_interface.address = listen_address;
    template.vm_config.publishing_options = VMPublishingOption::Open;
}
