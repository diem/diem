// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error, ValidatorConfig};
use anyhow::{ensure, Result};
use libra_config::{
    config::{DiscoveryMethod, NetworkConfig, NodeConfig, RoleType, SeedAddresses, SeedPublicKeys},
    generator,
    network_id::NetworkId,
    utils,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_network_address::NetworkAddress;
use libra_types::{chain_id::ChainId, transaction::Transaction};
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

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

    pub fn chain_id(&mut self, chain_id: ChainId) -> &mut Self {
        self.validator_config.chain_id = chain_id;
        self
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
        // Build up the VFN so we can get the seed peer configuration
        let mut rng = StdRng::from_seed(self.full_node_seed);
        let validator_config = self.validator_config()?;
        let vfn_config =
            self.build_validator_fullnode_config(&mut rng, false, &validator_config)?;

        // Build up seed peer information
        let seed_config = single_network(
            &vfn_config,
            NetworkId::vfn_network(),
            Error::MissingValidatorFullNodeNetwork,
        )?;
        let seed_pubkeys = Self::config_to_pubkeys(seed_config);
        let seed_addrs = generator::build_seed_addrs(&seed_config, self.bootstrap.clone());

        let full_node_config = self.build_fullnode_config(
            &mut rng,
            false,
            &validator_config,
            &seed_pubkeys,
            &seed_addrs,
        )?;
        let mut public_network = (*networks_with_id(&full_node_config, NetworkId::Public)
            .first()
            .unwrap())
        .clone();

        // TODO: Do we need this?
        public_network.listen_address = self.listen_address.clone();

        Ok(full_node_config)
    }

    // This is to add a VFN network to a validator.
    pub fn extend_validator(&self, config: &mut NodeConfig) -> Result<()> {
        let full_node_config = self.build()?;
        let vfn_network = (*networks_with_id(&full_node_config, NetworkId::vfn_network())
            .first()
            .unwrap())
        .clone();
        for net in &config.full_node_networks {
            let new_net_id = vfn_network.peer_id();
            let net_id = net.peer_id();
            ensure!(new_net_id != net_id, "Network already exists");
        }
        config.full_node_networks.push(vfn_network);
        Ok(())
    }

    // Add a public network to a full node
    // TODO: This doesn't make sense anymore since the network Id would be repeated, the interface should take more info
    pub fn extend(&self, config: &mut NodeConfig) -> Result<()> {
        let full_node_config = self.build()?;
        let mut new_public_network = (*networks_with_id(&full_node_config, NetworkId::Public)
            .first()
            .unwrap())
        .clone();

        // TODO: this seems like it shouldn't matter to be exactly the same
        new_public_network.listen_address = self.listen_address.clone();
        for net in &config.full_node_networks {
            let new_net_id = new_public_network.peer_id();
            let net_id = net.peer_id();
            ensure!(new_net_id != net_id, "Network already exists");
        }
        config.full_node_networks.push(new_public_network);
        Ok(())
    }

    /// Builds a swarm of nodes based on the number of full nodes
    /// The last node will always be the ValidatorFullNode, and the rest are public
    fn build_swarm_internal(&self, randomize_ports: bool) -> Result<Vec<NodeConfig>> {
        ensure!(self.num_full_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.full_node_index < self.num_full_nodes,
            Error::IndexError {
                index: self.full_node_index,
                nodes: self.num_full_nodes
            }
        );

        let mut rng = StdRng::from_seed(self.full_node_seed);
        let validator_config = self.validator_config()?;

        let mut configs = Vec::new();

        // Now let's create our VFN
        let vfn_config =
            self.build_validator_fullnode_config(&mut rng, randomize_ports, &validator_config)?;

        let vfn_network = single_network(
            &vfn_config,
            NetworkId::vfn_network(),
            Error::MissingValidatorFullNodeNetwork,
        )?;

        // Build seed configuration from our upstream (VFN) peer
        let seed_pubkeys = Self::config_to_pubkeys(vfn_network);
        let seed_addrs = generator::build_seed_addrs(&vfn_network, self.bootstrap.clone());

        // @TODO The last one is the upstream peer, note at some point we'll have to support taking
        // in a genesis instead at which point we may not have an upstream peer config
        // Let's create all of our public full nodes
        for _ in 0..self.num_full_nodes {
            let config = self.build_fullnode_config(
                &mut rng,
                randomize_ports,
                &validator_config,
                &seed_pubkeys,
                &seed_addrs,
            )?;
            configs.push(config)
        }

        Ok(configs)
    }

    fn build_fullnode_config(
        &self,
        rng: &mut StdRng,
        randomize_ports: bool,
        validator_config: &NodeConfig,
        seed_pubkeys: &SeedPublicKeys,
        seed_addrs: &SeedAddresses,
    ) -> Result<NodeConfig> {
        let mut config = NodeConfig::random_with_template(&self.template, rng);

        config.execution.genesis = if let Some(genesis) = self.genesis.as_ref() {
            Some(genesis.clone())
        } else {
            validator_config.execution.genesis.clone()
        };
        config.base.waypoint = validator_config.base.waypoint.clone();

        ensure!(
            config.full_node_networks.len() == 1,
            "Expected 1 full node networks"
        );
        let mut vfn_network = config
            .full_node_networks
            .first_mut()
            .ok_or(Error::MissingFullNodeNetwork)?;
        let mut public_network = (*vfn_network).clone();

        // Configure VFN network
        vfn_network.network_id = NetworkId::vfn_network();
        vfn_network.listen_address = utils::get_available_port_in_multiaddr(true);
        vfn_network.discovery_method = DiscoveryMethod::gossip(self.advertised_address.clone());
        vfn_network.mutual_authentication = self.mutual_authentication;

        // Configure public network
        public_network.network_id = NetworkId::Public;
        public_network.seed_pubkeys = seed_pubkeys.clone();
        public_network.seed_addrs = seed_addrs.clone();

        config.full_node_networks.push(public_network);
        ensure!(
            config.full_node_networks.len() == 2,
            "Expected 2 full node networks"
        );

        // Upstream is going to be VFN
        config.upstream.networks.push(NetworkId::vfn_network());

        // Randomize ports at the end so they don't conflict
        if randomize_ports {
            config.randomize_ports();
        }
        Ok(config)
    }

    fn build_validator_fullnode_config(
        &self,
        rng: &mut StdRng,
        randomize_ports: bool,
        validator_config: &NodeConfig,
    ) -> Result<NodeConfig> {
        let mut config = self.build_fullnode_config(
            rng,
            randomize_ports,
            validator_config,
            &HashMap::default(),
            &HashMap::default(),
        )?;

        // Upstream networks includes public this case
        // Order is (VFN, Public)
        config.upstream.networks.push(NetworkId::Public);

        // Add public fallback network
        let public_network =
            single_network(&config, NetworkId::Public, Error::MissingFullNodeNetwork)?;
        let fallback_network = public_network.clone();
        config.full_node_networks.push(fallback_network);

        // Randomize ports with the new networks
        if randomize_ports {
            config.randomize_ports();
        }
        Ok(config)
    }

    fn config_to_pubkeys(network: &NetworkConfig) -> SeedPublicKeys {
        let pubkey = network.identity_key().public_key();
        let pubkey_set: HashSet<_> = [pubkey].iter().copied().collect();
        let mut seed_pubkeys = SeedPublicKeys::new();
        seed_pubkeys.insert(network.peer_id(), pubkey_set);
        seed_pubkeys
    }

    fn validator_config(&self) -> Result<NodeConfig, Error> {
        // Don't randomize ports. Until we better separate genesis generation,
        // we don't want to accidentally randomize initial discovery set addresses.
        let randomize_validator_ports = false;
        let (validator_configs, _) = self
            .validator_config
            .build_common(randomize_validator_ports)
            .unwrap();
        validator_configs.first().cloned().ok_or(Error::NoConfigs)
    }

    fn libra_root_key(&self) -> Result<Ed25519PrivateKey, Error> {
        let randomize_validator_ports = false;
        let (_, libra_root_key) = self
            .validator_config
            .build_common(randomize_validator_ports)
            .unwrap();
        Ok(libra_root_key)
    }
}

impl BuildSwarm for FullNodeConfig {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        let configs = self.build_swarm_internal(true)?;
        let libra_root_key = self.libra_root_key()?;

        Ok((configs, libra_root_key))
    }
}

fn single_network(
    config: &NodeConfig,
    network_id: NetworkId,
    not_found: Error,
) -> Result<&NetworkConfig> {
    let found_networks = networks_with_id(config, network_id);
    ensure!(found_networks.len() == 1, not_found);
    Ok(*found_networks.first().unwrap())
}

fn networks_with_id(config: &NodeConfig, network_id: NetworkId) -> Vec<&NetworkConfig> {
    config
        .full_node_networks
        .iter()
        .filter(|network| network.network_id == network_id)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn verify_correctness() {
        // @TODO eventually if we do not have a validator config, the first peer in this network
        // should be the bootstrap
        let config = FullNodeConfig::new().build().unwrap();
        let vfn_networks: Vec<_> = networks_with_id(&config, NetworkId::vfn_network());

        // TODO: VFN networks?
        for network in vfn_networks.iter() {
            network.verify_seed_addrs().unwrap();
            let (seed_peer_id, seed_addrs) = network.seed_addrs.iter().next().unwrap();
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
    }

    #[test]
    fn verify_upstream_config() {
        let mut validator_config = ValidatorConfig::new().build().unwrap();
        FullNodeConfig::new()
            .extend_validator(&mut validator_config)
            .unwrap();

        let fnc = FullNodeConfig::new().build().unwrap();
        assert!(fnc.upstream.networks.contains(&NetworkId::vfn_network()));
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
        assert_ne!(
            config_extended.full_node_networks,
            config_orig.full_node_networks
        );
        assert_eq!(
            config_extended.full_node_networks[0].seed_pubkeys,
            config_full.full_node_networks[0].seed_pubkeys,
        );
    }

    #[test]
    #[ignore]
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
        assert_eq!(config_two.full_node_networks.len(), 3);
    }
}
