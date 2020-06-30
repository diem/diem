// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error, ValidatorConfig};
use anyhow::{ensure, Result};
use libra_config::{
    config::{DiscoveryMethod, NetworkPeersConfig, NodeConfig, RoleType},
    generator,
    network_id::NetworkId,
    utils,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_network_address::NetworkAddress;
use libra_types::transaction::Transaction;
use rand::{rngs::StdRng, SeedableRng};
use std::str::FromStr;

pub struct FullNodeConfig {
    pub advertised_address: NetworkAddress,
    pub bootstrap: NetworkAddress,
    pub full_node_index: usize,
    pub full_node_seed: [u8; 32],
    pub num_full_nodes: usize,
    pub genesis: Option<Transaction>,
    pub listen_address: NetworkAddress,
    pub mutual_authentication: bool,
    template: NodeConfig,
    validator_config: ValidatorConfig,
}

const DEFAULT_SEED: [u8; 32] = [15u8; 32];
const DEFAULT_ADVERTISED_ADDRESS: &str = "/ip4/127.0.0.1/tcp/7180";
const DEFAULT_LISTEN_ADDRESS: &str = "/ip4/0.0.0.0/tcp/7180";

impl Default for FullNodeConfig {
    fn default() -> Self {
        let mut template = NodeConfig::default();
        template.base.role = RoleType::FullNode;

        Self {
            advertised_address: NetworkAddress::from_str(DEFAULT_ADVERTISED_ADDRESS).unwrap(),
            bootstrap: NetworkAddress::from_str(DEFAULT_ADVERTISED_ADDRESS).unwrap(),
            full_node_index: 0,
            full_node_seed: DEFAULT_SEED,
            num_full_nodes: 1,
            genesis: None,
            listen_address: NetworkAddress::from_str(DEFAULT_LISTEN_ADDRESS).unwrap(),
            mutual_authentication: true,
            template,
            validator_config: ValidatorConfig::new(),
        }
    }
}

impl FullNodeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_validator_nodes(&mut self, nodes: usize) -> &mut Self {
        self.validator_config.num_nodes = nodes;
        self
    }

    pub fn validator_seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.validator_config.seed = seed;
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
        let seed_config = &validator_config
            .full_node_networks
            .last()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let seed_peers = generator::build_seed_peers(&seed_config, self.bootstrap.clone());

        let mut config = configs.swap_remove(self.full_node_index);
        let network = &mut config
            .full_node_networks
            .last_mut()
            .ok_or(Error::MissingFullNodeNetwork)?;
        network.discovery_method = DiscoveryMethod::gossip(self.advertised_address.clone());
        network.listen_address = self.listen_address.clone();
        network.seed_peers = seed_peers;

        Ok(config)
    }

    pub fn extend_validator(&self, config: &mut NodeConfig) -> Result<()> {
        let (mut configs, _) = self.build_internal(false)?;

        let mut new_net = configs.swap_remove(configs.len() - 1);
        let seed_config = &new_net
            .full_node_networks
            .last()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let seed_peers = generator::build_seed_peers(&seed_config, self.bootstrap.clone());

        let mut network = new_net.full_node_networks.swap_remove(0);
        network.discovery_method = DiscoveryMethod::gossip(self.advertised_address.clone());
        network.listen_address = self.listen_address.clone();
        network.seed_peers = seed_peers;
        config.full_node_networks.push(network);
        Ok(())
    }

    pub fn extend(&self, config: &mut NodeConfig) -> Result<()> {
        let mut full_node_config = self.build()?;
        let new_net = full_node_config.full_node_networks.swap_remove(0);
        for net in &config.full_node_networks {
            let new_net_id = new_net.peer_id();
            let net_id = net.peer_id();
            ensure!(new_net_id != net_id, "Network already exists");
        }
        config.full_node_networks.push(new_net);
        Ok(())
    }

    fn build_internal(
        &self,
        randomize_ports: bool,
    ) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        ensure!(self.num_full_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.full_node_index < self.num_full_nodes,
            Error::IndexError {
                index: self.full_node_index,
                nodes: self.num_full_nodes
            }
        );

        // Don't randomize ports. Until we better separate genesis generation,
        // we don't want to accidentally randomize initial discovery set addresses.
        let randomize_validator_ports = false;
        let (validator_configs, faucet_key) = self
            .validator_config
            .build_common(randomize_validator_ports)?;
        let validator_config = validator_configs.first().ok_or(Error::NoConfigs)?;

        let mut rng = StdRng::from_seed(self.full_node_seed);
        let mut configs = Vec::new();
        let mut network_peers = NetworkPeersConfig::default();

        // @TODO The last one is the upstream peer, note at some point we'll have to support taking
        // in a genesis instead at which point we may not have an upstream peer config
        let actual_nodes = self.num_full_nodes + 1;
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
            config.base.waypoint = validator_config.base.waypoint.clone();

            let network = config
                .full_node_networks
                .get_mut(0)
                .ok_or(Error::MissingFullNodeNetwork)?;
            network.network_id = NetworkId::vfn_network();
            network.listen_address = utils::get_available_port_in_multiaddr(true);
            network.discovery_method = DiscoveryMethod::gossip(self.advertised_address.clone());
            network.mutual_authentication = self.mutual_authentication;

            network_peers.insert(
                network.peer_id(),
                network
                    .identity
                    .public_key_from_config()
                    .ok_or(Error::MissingNetworkKeyPairs)?,
            );

            configs.push(config);
        }

        let validator_full_node_config = configs.last().ok_or(Error::NoConfigs)?;
        let validator_full_node_network = validator_full_node_config
            .full_node_networks
            .last()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let seed_peers =
            generator::build_seed_peers(&validator_full_node_network, self.bootstrap.clone());
        for (idx, config) in configs.iter_mut().enumerate() {
            let network = config
                .full_node_networks
                .last_mut()
                .ok_or(Error::MissingFullNodeNetwork)?;
            network.network_id = NetworkId::Public;
            network.network_peers = network_peers.clone();
            network.seed_peers = seed_peers.clone();
            if idx < actual_nodes - 1 {
                config.upstream.networks.push(network.network_id.clone());
            }
        }

        Ok((configs, faucet_key))
    }
}

impl BuildSwarm for FullNodeConfig {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
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

        network.verify_seed_peer_addrs().unwrap();
        let (seed_peer_id, seed_addrs) = network.seed_peers.iter().next().unwrap();
        assert_eq!(seed_addrs.len(), 1);
        assert_ne!(&network.peer_id(), seed_peer_id);
        assert_ne!(
            &network.discovery_method.advertised_address(),
            &seed_addrs[0]
        );

        assert_eq!(
            network.discovery_method.advertised_address(),
            NetworkAddress::from_str(DEFAULT_ADVERTISED_ADDRESS).unwrap()
        );
        assert_eq!(
            network.listen_address,
            NetworkAddress::from_str(DEFAULT_LISTEN_ADDRESS).unwrap()
        );

        assert!(config.execution.genesis.is_some());
    }

    #[test]
    fn verify_upstream_config() {
        let mut validator_config = ValidatorConfig::new().build().unwrap();
        FullNodeConfig::new()
            .extend_validator(&mut validator_config)
            .unwrap();

        let fnc = FullNodeConfig::new().build().unwrap();
        let fn_network_id = &fnc.full_node_networks[0].network_id;
        assert!(fnc.upstream.networks.contains(fn_network_id));
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
        fnc.full_node_seed = [33u8; 32];
        fnc.extend(&mut config_two).unwrap();
        assert_eq!(
            config_one.full_node_networks[0],
            config_two.full_node_networks[0]
        );
        assert_eq!(config_two.full_node_networks.len(), 2);
    }
}
