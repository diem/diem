// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error, ValidatorConfig};
use anyhow::{ensure, Result};
use libra_config::{
    config::{NetworkPeersConfig, NodeConfig, RoleType, SeedPeersConfig, UpstreamPeersConfig},
    utils,
};
use libra_crypto::ed25519;
use libra_types::transaction::Transaction;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

pub struct FullNodeConfig {
    advertised: Multiaddr,
    bootstrap: Multiaddr,
    full_node_index: usize,
    full_node_seed: [u8; 32],
    full_nodes: usize,
    genesis: Option<Transaction>,
    listen: Multiaddr,
    enable_remote_authentication: bool,
    template: NodeConfig,
    validator_config: ValidatorConfig,
}

const DEFAULT_SEED: [u8; 32] = [15u8; 32];
const DEFAULT_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/7180";
const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/7180";

impl Default for FullNodeConfig {
    fn default() -> Self {
        let mut template = NodeConfig::default();
        template.base.role = RoleType::FullNode;

        Self {
            advertised: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            bootstrap: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            full_node_index: 0,
            full_node_seed: DEFAULT_SEED,
            full_nodes: 1,
            genesis: None,
            listen: DEFAULT_LISTEN.parse::<Multiaddr>().unwrap(),
            enable_remote_authentication: true,
            template,
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

    pub fn genesis(&mut self, genesis: Transaction) -> &mut Self {
        self.genesis = Some(genesis);
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

    pub fn enable_remote_authentication(&mut self) -> &mut Self {
        self.enable_remote_authentication = true;
        self
    }

    pub fn public(&mut self) -> &mut Self {
        self.enable_remote_authentication = false;
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
        let (mut configs, _) = self.build_internal(false)?;
        let validator_config = configs.last().ok_or(Error::NoConfigs)?;
        let seed_peers = self.build_seed_peers(&validator_config)?;
        let mut config = configs.swap_remove(self.full_node_index);
        let network = &mut config
            .full_node_networks
            .last_mut()
            .ok_or(Error::MissingFullNodeNetwork)?;
        network.advertised_address = self.advertised.clone();
        network.listen_address = self.listen.clone();
        network.seed_peers = seed_peers;

        Ok(config)
    }

    pub fn extend_validator(&self, config: &mut NodeConfig) -> Result<()> {
        let (mut configs, _) = self.build_internal(false)?;
        let mut new_net = configs.swap_remove(configs.len() - 1);
        let seed_peers = self.build_seed_peers(&new_net)?;
        let mut network = new_net.full_node_networks.swap_remove(0);
        network.advertised_address = self.advertised.clone();
        network.listen_address = self.listen.clone();
        network.seed_peers = seed_peers;
        config.full_node_networks.push(network);
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
    ) -> Result<(Vec<NodeConfig>, ed25519::SigningKey)> {
        ensure!(self.full_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.full_node_index < self.full_nodes,
            Error::IndexError {
                index: self.full_node_index,
                nodes: self.full_nodes
            }
        );

        // Don't randomize ports. Until we better separate genesis generation,
        // we don't want to accidentally randomize initial discovery set addresses.
        let randomize_service_ports = false;
        let randomize_libranet_ports = false;
        let (validator_configs, faucet_key) = self
            .validator_config
            .build_common(randomize_service_ports, randomize_libranet_ports)?;
        let validator_config = validator_configs.first().ok_or(Error::NoConfigs)?;

        let mut rng = StdRng::from_seed(self.full_node_seed);
        let mut configs = Vec::new();
        let mut network_peers = NetworkPeersConfig {
            peers: HashMap::new(),
        };

        // @TODO The last one is the upstream peer, note at some point we'll have to support taking
        // in a genesis instead at which point we may not have an upstream peer config
        let actual_nodes = self.full_nodes + 1;
        for _index in 0..actual_nodes {
            let mut config = NodeConfig::random_with_template(&self.template, &mut rng);
            if randomize_ports {
                config.randomize_ports();
            }

            config.execution.genesis = if let Some(genesis) = self.genesis.as_ref() {
                Some(genesis.clone())
            } else {
                validator_config.execution.genesis.clone()
            };

            let network = config
                .full_node_networks
                .get_mut(0)
                .ok_or(Error::MissingFullNodeNetwork)?;
            network.listen_address = utils::get_available_port_in_multiaddr(true);
            network.advertised_address = network.listen_address.clone();
            network.enable_remote_authentication = self.enable_remote_authentication;

            network_peers.peers.insert(
                network.peer_id,
                network
                    .network_keypairs
                    .as_ref()
                    .ok_or(Error::MissingNetworkKeyPairs)?
                    .as_peer_info(),
            );

            configs.push(config);
        }

        let validator_full_node_config = configs.last().ok_or(Error::NoConfigs)?;
        let validator_full_node_network = validator_full_node_config
            .full_node_networks
            .last()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let upstream_peers = UpstreamPeersConfig {
            upstream_peers: vec![validator_full_node_network.peer_id],
        };

        let seed_peers = self.build_seed_peers(&validator_full_node_config)?;
        for config in configs.iter_mut() {
            config.state_sync.upstream_peers = upstream_peers.clone();
            let network = config
                .full_node_networks
                .last_mut()
                .ok_or(Error::MissingFullNodeNetwork)?;
            network.network_peers = network_peers.clone();
            network.seed_peers = seed_peers.clone();
        }

        Ok((configs, faucet_key))
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
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, ed25519::SigningKey)> {
        let (mut configs, faucet_key) = self.build_internal(true)?;
        configs.swap_remove(configs.len() - 1);
        Ok((configs, faucet_key))
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
        assert_ne!(&network.peer_id, seed_peer_id);
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
        assert_eq!(
            config_one.full_node_networks[0],
            config_two.full_node_networks[0]
        );
        assert_eq!(config_two.full_node_networks.len(), 2);
    }
}
