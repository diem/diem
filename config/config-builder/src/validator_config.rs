// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error};
use anyhow::{ensure, format_err, Result};
use executor::db_bootstrapper;
use libra_config::{
    config::{
        ConsensusType, NodeConfig, RemoteService, SafetyRulesService, SecureBackend, Token,
        VaultConfig,
    },
    generator,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_network_address::NetworkAddress;
use libra_temppath::TempPath;
use libra_types::waypoint::Waypoint;
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{net::SocketAddr, str::FromStr};
use storage_interface::DbReaderWriter;

const DEFAULT_SEED: [u8; 32] = [13u8; 32];
const DEFAULT_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/6180";
const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/6180";

pub struct ValidatorConfig {
    advertised: NetworkAddress,
    build_waypoint: bool,
    bootstrap: NetworkAddress,
    index: usize,
    listen: NetworkAddress,
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
            advertised: NetworkAddress::from_str(DEFAULT_ADVERTISED).unwrap(),
            bootstrap: NetworkAddress::from_str(DEFAULT_ADVERTISED).unwrap(),
            build_waypoint: true,
            index: 0,
            listen: NetworkAddress::from_str(DEFAULT_LISTEN).unwrap(),
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

    pub fn advertised(&mut self, advertised: NetworkAddress) -> &mut Self {
        self.advertised = advertised;
        self
    }

    pub fn bootstrap(&mut self, bootstrap: NetworkAddress) -> &mut Self {
        self.bootstrap = bootstrap;
        self
    }

    pub fn build_waypoint(&mut self, enabled: bool) -> &mut Self {
        self.build_waypoint = enabled;
        self
    }

    pub fn validator_index(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    pub fn listen(&mut self, listen: NetworkAddress) -> &mut Self {
        self.listen = listen;
        self
    }

    pub fn validators(&mut self, nodes: usize) -> &mut Self {
        self.nodes = nodes;
        self
    }

    pub fn validators_in_genesis(&mut self, nodes_in_genesis: Option<usize>) -> &mut Self {
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

        // Extract and format first node's advertised address for the cluster
        // bootstrap.

        let seed_config = configs[0]
            .validator_network
            .as_ref()
            .ok_or(Error::MissingValidatorNetwork)?;
        let seed_peers = seed_config.build_seed_peers(self.bootstrap.clone())?;

        let mut config = configs.swap_remove(self.index);
        let validator_network = config
            .validator_network
            .as_mut()
            .ok_or(Error::MissingValidatorNetwork)?;
        validator_network.listen_address = self.listen.clone();
        validator_network.advertised_address = self.advertised.clone();
        validator_network.seed_peers = seed_peers;

        self.build_safety_rules(&mut config)?;

        Ok(config)
    }

    pub fn build_set(&self) -> Result<Vec<NodeConfig>> {
        let (configs, _) = self.build_common(false)?;
        Ok(configs)
    }

    pub fn build_faucet_client(&self) -> Result<(Ed25519PrivateKey, Waypoint)> {
        let (configs, faucet_key) = self.build_common(false)?;
        Ok((
            faucet_key,
            configs[0]
                .base
                .waypoint
                .ok_or_else(|| format_err!("Waypoint not generated"))?,
        ))
    }

    pub fn build_common(
        &self,
        randomize_ports: bool,
    ) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        ensure!(self.nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.index < self.nodes,
            Error::IndexError {
                index: self.index,
                nodes: self.nodes
            }
        );

        let (faucet_key, config_seed) = self.build_faucet_key();
        let generator::ValidatorSwarm { mut nodes, .. } =
            generator::validator_swarm(&self.template, self.nodes, config_seed, randomize_ports);

        ensure!(
            nodes.len() == self.nodes,
            Error::MissingConfigs { found: nodes.len() }
        );

        // Optionally choose a limited subset of generated validators to be
        // present at genesis time.
        let nodes_in_genesis = self.nodes_in_genesis.unwrap_or(self.nodes);

        let validators = vm_genesis::validator_registrations(&nodes[..nodes_in_genesis]);

        let genesis = vm_genesis::encode_genesis_transaction_with_validator(
            faucet_key.public_key(),
            &validators,
            self.template
                .test
                .as_ref()
                .and_then(|config| config.publishing_option.clone()),
        );

        let waypoint = if self.build_waypoint {
            let path = TempPath::new();
            let db_rw = DbReaderWriter::new(LibraDB::open(
                &path, false, /* readonly */
                None,  /* pruner */
            )?);
            Some(
                db_bootstrapper::bootstrap_db_if_empty::<LibraVM>(&db_rw, &genesis)?
                    .ok_or_else(|| format_err!("Failed to bootstrap empty DB."))?,
            )
        } else {
            None
        };

        let genesis = Some(genesis);

        for node in &mut nodes {
            node.base.waypoint = waypoint;
            node.execution.genesis = genesis.clone();
        }

        Ok((nodes, faucet_key))
    }

    pub fn build_faucet_key(&self) -> (Ed25519PrivateKey, [u8; 32]) {
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
                "in-memory" => SecureBackend::InMemoryStorage,
                "on-disk" => safety_rules_config.backend.clone(),
                "vault" => SecureBackend::Vault(VaultConfig {
                    namespace: self.safety_rules_namespace.clone(),
                    server: self
                        .safety_rules_host
                        .as_ref()
                        .ok_or_else(|| Error::MissingSafetyRulesHost)?
                        .clone(),
                    ca_certificate: None,
                    token: Token::new_config(
                        self.safety_rules_token
                            .as_ref()
                            .ok_or_else(|| Error::MissingSafetyRulesToken)?
                            .clone(),
                    ),
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
        let config = validator_config
            .validators(2)
            .validator_index(1)
            .build()
            .unwrap();
        let network = config.validator_network.as_ref().unwrap();

        network.seed_peers.verify_libranet_addrs().unwrap();
        let (seed_peer_id, seed_addrs) = network.seed_peers.seed_peers.iter().next().unwrap();
        assert_eq!(seed_addrs.len(), 1);
        assert_ne!(&network.peer_id, seed_peer_id);
        assert_ne!(&network.advertised_address, &seed_addrs[0]);

        assert_eq!(
            network.advertised_address,
            NetworkAddress::from_str(DEFAULT_ADVERTISED).unwrap()
        );
        assert_eq!(
            network.listen_address,
            NetworkAddress::from_str(DEFAULT_LISTEN).unwrap()
        );

        assert!(config.execution.genesis.is_some());
    }

    #[test]
    fn verify_same_genesis() {
        let config1 = ValidatorConfig::new()
            .validators(10)
            .validator_index(1)
            .build()
            .unwrap();
        let config2 = ValidatorConfig::new()
            .validators(13)
            .validator_index(12)
            .validators_in_genesis(Some(10))
            .build()
            .unwrap();

        assert_eq!(config1.execution.genesis, config2.execution.genesis);
    }
}
