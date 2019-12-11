// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error, ValidatorConfig};
use anyhow::{ensure, Result};
use libra_config::{
    config::{NodeConfig, SeedPeersConfig},
    generator,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use parity_multiaddr::Multiaddr;
use std::collections::HashMap;

pub struct FullNodeConfig {
    advertised: Multiaddr,
    bootstrap: Multiaddr,
    full_node_index: usize,
    full_node_seed: [u8; 32],
    full_nodes: usize,
    listen: Multiaddr,
    permissioned: bool,
    template: NodeConfig,
    validator_config: ValidatorConfig,
}

const DEFAULT_SEED: [u8; 32] = [15u8; 32];
const DEFAULT_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/7180";
const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/7180";

impl Default for FullNodeConfig {
    fn default() -> Self {
        Self {
            advertised: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            bootstrap: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            full_node_index: 0,
            full_node_seed: DEFAULT_SEED,
            full_nodes: 1,
            listen: DEFAULT_LISTEN.parse::<Multiaddr>().unwrap(),
            permissioned: true,
            template: NodeConfig::default(),
            validator_config: ValidatorConfig::new(),
        }
    }
}

impl FullNodeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advertised(&mut self, advertised: Multiaddr) -> &mut Self {
        self.advertised = advertised;
        self
    }

    pub fn bootstrap(&mut self, bootstrap: Multiaddr) -> &mut Self {
        self.bootstrap = bootstrap;
        self
    }

    pub fn full_node_index(&mut self, full_node_index: usize) -> &mut Self {
        self.full_node_index = full_node_index;
        self
    }

    pub fn full_nodes(&mut self, full_nodes: usize) -> &mut Self {
        self.full_nodes = full_nodes;
        self
    }

    pub fn full_node_seed(&mut self, full_node_seed: [u8; 32]) -> &mut Self {
        self.full_node_seed = full_node_seed;
        self
    }

    pub fn listen(&mut self, listen: Multiaddr) -> &mut Self {
        self.listen = listen;
        self
    }

    pub fn nodes(&mut self, nodes: usize) -> &mut Self {
        self.validator_config.nodes(nodes);
        self
    }

    pub fn permissioned(&mut self) -> &mut Self {
        self.permissioned = true;
        self
    }

    pub fn public(&mut self) -> &mut Self {
        self.permissioned = false;
        self
    }

    pub fn seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.validator_config.seed(seed);
        self
    }

    pub fn template(&mut self, mut template: NodeConfig) -> &mut Self {
        template.base.role = RoleType::FullNode;
        self.template = template;
        self
    }

    pub fn build(&self) -> Result<NodeConfig> {
        ensure!(self.full_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.full_node_index < self.full_nodes,
            Error::IndexError {
                index: self.full_node_index,
                nodes: self.full_nodes
            }
        );

        let mut validator_config = self.validator_config.build()?;
        let mut configs = self.build_internal(false, &mut validator_config)?;

        let mut config = configs.swap_remove(self.full_node_index);
        let network = &mut config
            .full_node_networks
            .last_mut()
            .ok_or(Error::MissingFullNodeNetwork)?;
        network.advertised_address = self.advertised.clone();
        network.listen_address = self.listen.clone();

        Ok(config)
    }

    pub fn extend_validator(&self, config: &mut NodeConfig) -> Result<()> {
        self.build_internal(false, config)?;
        let seed_peers = self.build_seed_peers(&config)?;
        let network = config
            .full_node_networks
            .last_mut()
            .ok_or(Error::MissingFullNodeNetwork)?;
        network.advertised_address = self.advertised.clone();
        network.listen_address = self.listen.clone();
        network.seed_peers = seed_peers;
        Ok(())
    }

    pub fn extend(&self, config: &mut NodeConfig) -> Result<()> {
        let mut full_node_config = self.build()?;
        let new_net = full_node_config.full_node_networks.swap_remove(0);
        for net in &config.full_node_networks {
            ensure!(new_net.peer_id != net.peer_id, "Network already exists");
        }
        config.full_node_networks.push(new_net);
        Ok(())
    }

    fn build_internal(
        &self,
        randomize_ports: bool,
        validator_config: &mut NodeConfig,
    ) -> Result<Vec<NodeConfig>> {
        ensure!(self.full_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.full_node_index < self.full_nodes,
            Error::IndexError {
                index: self.full_node_index,
                nodes: self.full_nodes
            }
        );

        let mut configs = generator::full_node_swarm(
            validator_config,
            self.template.clone_for_template(),
            self.full_nodes,
            true,
            true,
            Some(self.full_node_seed),
            self.permissioned,
            randomize_ports,
        )?;

        let seed_peers = self.build_seed_peers(&validator_config)?;
        for config in configs.iter_mut() {
            let network = config
                .full_node_networks
                .last_mut()
                .ok_or(Error::MissingFullNodeNetwork)?;
            network.seed_peers = seed_peers.clone();
        }

        Ok(configs)
    }

    fn build_seed_peers(&self, config: &NodeConfig) -> Result<SeedPeersConfig> {
        let network = config
            .full_node_networks
            .last()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let mut seed_peers = HashMap::new();
        seed_peers.insert(network.peer_id, vec![self.bootstrap.clone()]);
        Ok(SeedPeersConfig { seed_peers })
    }
}

impl BuildSwarm for FullNodeConfig {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        let mut validator_config = self.validator_config.build()?;
        let (_, faucet_key) = self.validator_config.build_faucet_client()?;
        Ok((
            self.build_internal(true, &mut validator_config)?,
            faucet_key,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_correctness() {
        // @TODO eventually if we do not have a validator config, the first peer in this network
        // should be the bootstrap
        let config = FullNodeConfig::new().build().unwrap();
        let network = &config.full_node_networks[0];
        let (seed_peer_id, seed_peer_ips) = network.seed_peers.seed_peers.iter().next().unwrap();
        assert!(&network.peer_id != seed_peer_id);
        // This is true because  we didn't update the DEFAULT_ADVERTISED
        assert_eq!(network.advertised_address, seed_peer_ips[0]);
        assert_eq!(
            network.advertised_address,
            DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap()
        );
        assert_eq!(
            network.listen_address,
            DEFAULT_LISTEN.parse::<Multiaddr>().unwrap()
        );
        assert!(config.execution.genesis.is_some());
        assert_eq!(config.consensus.consensus_peers.peers.len(), 1);
    }

    #[test]
    fn verify_state_sync() {
        let mut validator_config = ValidatorConfig::new().build().unwrap();
        FullNodeConfig::new()
            .extend_validator(&mut validator_config)
            .unwrap();
        let val_fn = &validator_config.full_node_networks[0];

        let fnc = FullNodeConfig::new().build().unwrap();
        assert_eq!(
            val_fn.peer_id,
            fnc.state_sync.upstream_peers.upstream_peers[0]
        );
    }

    #[test]
    fn verify_validator_append() {
        let config_orig = ValidatorConfig::new().build().unwrap();
        let mut config_extended = ValidatorConfig::new().build().unwrap();
        FullNodeConfig::new()
            .extend_validator(&mut config_extended)
            .unwrap();
        let config_full = FullNodeConfig::new().build().unwrap();
        assert_eq!(config_extended.consensus, config_orig.consensus);
        assert_eq!(
            config_extended.validator_network,
            config_orig.validator_network
        );
        assert!(config_extended.full_node_networks != config_orig.full_node_networks);
        assert_eq!(
            config_extended.full_node_networks[0].network_peers,
            config_full.full_node_networks[0].network_peers
        );
    }

    #[test]
    fn verify_full_node_append() {
        let config_one = FullNodeConfig::new().build().unwrap();
        let mut config_two = FullNodeConfig::new().build().unwrap();
        let mut fnc = FullNodeConfig::new();
        fnc.full_node_seed([33u8; 32]);
        fnc.extend(&mut config_two).unwrap();
        assert_eq!(config_one.consensus, config_two.consensus);
        assert_eq!(
            config_one.full_node_networks[0],
            config_two.full_node_networks[0]
        );
        assert_eq!(config_two.full_node_networks.len(), 2);
    }
}
