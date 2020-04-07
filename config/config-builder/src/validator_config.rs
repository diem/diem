// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error};
use anyhow::{ensure, Result};
use libra_config::{
    config::{
        ConsensusType, NodeConfig, RemoteService, SafetyRulesBackend, SafetyRulesService,
        SeedPeersConfig, VaultConfig,
    },
    generator,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::validator_set::ValidatorSet;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{collections::HashMap, net::SocketAddr};

const DEFAULT_SEED: [u8; 32] = [13u8; 32];
const DEFAULT_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/6180";
const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/6180";

pub struct ValidatorConfig {
    advertised: Multiaddr,
    bootstrap: Multiaddr,
    index: usize,
    listen: Multiaddr,
    nodes: usize,
    nodes_in_genesis: Option<usize>,
    safety_rules_addr: Option<SocketAddr>,
    safety_rules_backend: Option<String>,
    safety_rules_host: Option<String>,
    safety_rules_namespace: Option<String>,
    safety_rules_token: Option<String>,
    seed: [u8; 32],
    template: NodeConfig,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            advertised: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            bootstrap: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            index: 0,
            listen: DEFAULT_LISTEN.parse::<Multiaddr>().unwrap(),
            nodes: 1,
            nodes_in_genesis: None,
            safety_rules_addr: None,
            safety_rules_backend: None,
            safety_rules_host: None,
            safety_rules_namespace: None,
            safety_rules_token: None,
            seed: DEFAULT_SEED,
            template: NodeConfig::default(),
        }
    }
}

impl ValidatorConfig {
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

    pub fn index(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    pub fn listen(&mut self, listen: Multiaddr) -> &mut Self {
        self.listen = listen;
        self
    }

    pub fn nodes(&mut self, nodes: usize) -> &mut Self {
        self.nodes = nodes;
        self
    }

    pub fn nodes_in_genesis(&mut self, nodes_in_genesis: Option<usize>) -> &mut Self {
        self.nodes_in_genesis = nodes_in_genesis;
        self
    }

    pub fn safety_rules_addr(&mut self, safety_rules_addr: Option<SocketAddr>) -> &mut Self {
        self.safety_rules_addr = safety_rules_addr;
        self
    }

    pub fn safety_rules_backend(&mut self, safety_rules_backend: Option<String>) -> &mut Self {
        self.safety_rules_backend = safety_rules_backend;
        self
    }

    pub fn safety_rules_host(&mut self, safety_rules_host: Option<String>) -> &mut Self {
        self.safety_rules_host = safety_rules_host;
        self
    }

    pub fn safety_rules_namespace(&mut self, safety_rules_namespace: Option<String>) -> &mut Self {
        self.safety_rules_namespace = safety_rules_namespace;
        self
    }

    pub fn safety_rules_token(&mut self, safety_rules_token: Option<String>) -> &mut Self {
        self.safety_rules_token = safety_rules_token;
        self
    }

    pub fn seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.seed = seed;
        self
    }

    pub fn template(&mut self, template: NodeConfig) -> &mut Self {
        self.template = template;
        self
    }

    pub fn build(&self) -> Result<NodeConfig> {
        let mut configs = self.build_set()?;
        let first_peer_id = configs[0]
            .validator_network
            .as_ref()
            .ok_or(Error::MissingValidatorNetwork)?
            .peer_id;
        let mut config = configs.swap_remove(self.index);

        let validator_network = config
            .validator_network
            .as_mut()
            .ok_or(Error::MissingValidatorNetwork)?;
        validator_network.listen_address = self.listen.clone();
        validator_network.advertised_address = self.advertised.clone();

        let mut seed_peers = HashMap::new();
        seed_peers.insert(first_peer_id, vec![self.bootstrap.clone()]);
        let seed_peers_config = SeedPeersConfig { seed_peers };
        validator_network.seed_peers = seed_peers_config;

        self.build_safety_rules(&mut config)?;

        Ok(config)
    }

    pub fn build_set(&self) -> Result<Vec<NodeConfig>> {
        let (configs, _) = self.build_common(false)?;
        Ok(configs)
    }

    pub fn build_faucet_client(&self) -> Ed25519PrivateKey {
        let (faucet_key, _) = self.build_faucet();
        faucet_key
    }

    fn build_common(&self, randomize_ports: bool) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        ensure!(self.nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.index < self.nodes,
            Error::IndexError {
                index: self.index,
                nodes: self.nodes
            }
        );

        let (faucet_key, config_seed) = self.build_faucet();
        let mut validator_swarm =
            generator::validator_swarm(&self.template, self.nodes, config_seed, randomize_ports);

        ensure!(
            validator_swarm.nodes.len() == self.nodes,
            Error::MissingConfigs {
                found: validator_swarm.nodes.len()
            }
        );
        let nodes_in_genesis = self.nodes_in_genesis.unwrap_or(self.nodes);

        let validator_set = ValidatorSet::new(
            validator_swarm
                .validator_set
                .payload()
                .iter()
                .cloned()
                .take(nodes_in_genesis)
                .collect(),
        );
        let discovery_set = vm_genesis::make_placeholder_discovery_set(&validator_set);

        let genesis = Some(vm_genesis::encode_genesis_transaction_with_validator(
            &faucet_key,
            faucet_key.public_key(),
            &validator_swarm.nodes,
            validator_set,
            discovery_set,
            self.template
                .test
                .as_ref()
                .and_then(|config| config.publishing_option.clone()),
        ));

        for node in &mut validator_swarm.nodes {
            node.execution.genesis = genesis.clone();
        }

        Ok((validator_swarm.nodes, faucet_key))
    }

    fn build_faucet(&self) -> (Ed25519PrivateKey, [u8; 32]) {
        let mut faucet_rng = StdRng::from_seed(self.seed);
        let faucet_key = Ed25519PrivateKey::generate(&mut faucet_rng);
        let config_seed: [u8; 32] = faucet_rng.gen();
        (faucet_key, config_seed)
    }

    fn build_safety_rules(&self, config: &mut NodeConfig) -> Result<()> {
        let safety_rules_config = &mut config.consensus.safety_rules;
        if let Some(server_address) = self.safety_rules_addr {
            safety_rules_config.service = SafetyRulesService::Process(RemoteService {
                server_address,
                consensus_type: ConsensusType::SignedTransactions,
            })
        }

        if let Some(backend) = &self.safety_rules_backend {
            safety_rules_config.backend = match backend.as_str() {
                "in-memory" => SafetyRulesBackend::InMemoryStorage,
                "on-disk" => safety_rules_config.backend.clone(),
                "vault" => SafetyRulesBackend::Vault(VaultConfig {
                    default: true,
                    namespace: self.safety_rules_namespace.clone(),
                    server: self
                        .safety_rules_host
                        .as_ref()
                        .ok_or_else(|| Error::MissingSafetyRulesHost)?
                        .clone(),
                    token: self
                        .safety_rules_token
                        .as_ref()
                        .ok_or_else(|| Error::MissingSafetyRulesToken)?
                        .clone(),
                }),
                _ => return Err(Error::InvalidSafetyRulesBackend(backend.to_string()).into()),
            };
        }

        Ok(())
    }
}

impl BuildSwarm for ValidatorConfig {
    fn build_swarm(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        self.build_common(true)
    }
}

pub fn test_config() -> (NodeConfig, Ed25519PrivateKey) {
    let validator_config = ValidatorConfig::new();
    let (mut configs, key) = validator_config.build_swarm().unwrap();
    (configs.swap_remove(0), key)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_correctness() {
        let mut validator_config = ValidatorConfig::new();
        let config = validator_config.nodes(2).index(1).build().unwrap();
        let network = config.validator_network.as_ref().unwrap();
        let (seed_peer_id, seed_peer_ips) = network.seed_peers.seed_peers.iter().next().unwrap();
        assert!(&network.peer_id != seed_peer_id);
        // These equal cause we didn't set
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
}
