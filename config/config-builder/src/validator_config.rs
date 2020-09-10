// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BuildSwarm, Error};
use anyhow::{ensure, format_err, Result};
use executor::db_bootstrapper;
use libra_config::{
    config::{
        DiscoveryMethod, NodeConfig, OnDiskStorageConfig, RemoteService, SafetyRulesService,
        SecureBackend, Token, VaultConfig, WaypointConfig,
    },
    generator,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_network_address::NetworkAddress;
use libra_temppath::TempPath;
use libra_types::{chain_id::ChainId, waypoint::Waypoint};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{net::SocketAddr, str::FromStr};
use storage_interface::DbReaderWriter;

const DEFAULT_SEED: [u8; 32] = [13u8; 32];
const DEFAULT_ADVERTISED_ADDRESS: &str = "/ip4/127.0.0.1/tcp/6180";
const DEFAULT_LISTEN_ADDRESS: &str = "/ip4/0.0.0.0/tcp/6180";

pub struct ValidatorConfig {
    pub advertised_address: NetworkAddress,
    pub build_waypoint: bool,
    pub bootstrap: NetworkAddress,
    pub chain_id: ChainId,
    pub listen_address: NetworkAddress,
    pub node_index: usize,
    pub num_nodes: usize,
    pub num_nodes_in_genesis: Option<usize>,
    pub safety_rules_addr: Option<SocketAddr>,
    pub safety_rules_backend: Option<String>,
    pub safety_rules_host: Option<String>,
    pub safety_rules_namespace: Option<String>,
    pub safety_rules_token: Option<String>,
    pub seed: [u8; 32],
    pub template: NodeConfig,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            advertised_address: NetworkAddress::from_str(DEFAULT_ADVERTISED_ADDRESS).unwrap(),
            bootstrap: NetworkAddress::from_str(DEFAULT_ADVERTISED_ADDRESS).unwrap(),
            build_waypoint: true,
            chain_id: ChainId::test(),
            listen_address: NetworkAddress::from_str(DEFAULT_LISTEN_ADDRESS).unwrap(),
            node_index: 0,
            num_nodes: 1,
            num_nodes_in_genesis: None,
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

    pub fn build(&self) -> Result<NodeConfig> {
        let mut configs = self.build_set()?;

        // Extract and format first node's advertised address to use as the seed
        // peer for bootstrapping other validator nodes.
        let seed_config = &configs[0]
            .validator_network
            .as_ref()
            .ok_or(Error::MissingValidatorNetwork)?;
        let seed_addrs = generator::build_seed_addrs(&seed_config, self.bootstrap.clone());

        // Pull out this specific node from the generated validator configs.
        let mut config = configs.swap_remove(self.node_index);

        let validator_network = config
            .validator_network
            .as_mut()
            .ok_or(Error::MissingValidatorNetwork)?;
        validator_network.listen_address = self.listen_address.clone();
        validator_network.discovery_method =
            DiscoveryMethod::gossip(self.advertised_address.clone());
        validator_network.seed_addrs = seed_addrs;

        self.build_safety_rules(&mut config)?;

        Ok(config)
    }

    pub fn build_set(&self) -> Result<Vec<NodeConfig>> {
        let (configs, _) = self.build_common(false)?;
        Ok(configs)
    }

    pub fn build_faucet_client(&self) -> Result<(Ed25519PrivateKey, Waypoint)> {
        let (configs, libra_root_key) = self.build_common(false)?;
        Ok((
            libra_root_key,
            configs[0]
                .base
                .waypoint
                .waypoint_from_config()
                .ok_or_else(|| format_err!("Waypoint not generated"))?,
        ))
    }

    pub fn build_common(
        &self,
        randomize_ports: bool,
    ) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        ensure!(self.num_nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.node_index < self.num_nodes,
            Error::IndexError {
                index: self.node_index,
                nodes: self.num_nodes
            }
        );

        let (libra_root_key, config_seed) = self.build_libra_root_key();
        let generator::ValidatorSwarm { mut nodes, .. } = generator::validator_swarm(
            &self.template,
            self.num_nodes,
            config_seed,
            randomize_ports,
        );

        ensure!(
            nodes.len() == self.num_nodes,
            Error::MissingConfigs { found: nodes.len() }
        );

        // Optionally choose a limited subset of generated validators to be
        // present at genesis time.
        let nodes_in_genesis = self.num_nodes_in_genesis.unwrap_or(self.num_nodes);

        let operator_assignments = vm_genesis::operator_assignments(&nodes[..nodes_in_genesis]);
        let operator_registrations = vm_genesis::operator_registrations(&nodes[..nodes_in_genesis]);

        let genesis = vm_genesis::encode_genesis_transaction(
            libra_root_key.public_key(),
            libra_root_key.public_key(),
            &operator_assignments,
            &operator_registrations,
            self.template
                .test
                .as_ref()
                .and_then(|config| config.publishing_option.clone()),
            self.chain_id,
        );

        let (waypoint, maybe_waypoint) = if self.build_waypoint {
            let path = TempPath::new();
            let db_rw = DbReaderWriter::new(LibraDB::open(
                &path, false, /* readonly */
                None,  /* pruner */
            )?);
            let waypoint = db_bootstrapper::generate_waypoint::<LibraVM>(&db_rw, &genesis)?;
            (WaypointConfig::FromConfig(waypoint), Some(waypoint))
        } else {
            (WaypointConfig::None, None)
        };

        let genesis = Some(genesis);

        for node in &mut nodes {
            if let Some(test_config) = node.consensus.safety_rules.test.as_mut() {
                test_config.waypoint = maybe_waypoint;
            }

            node.base.waypoint = waypoint.clone();
            node.execution.genesis = genesis.clone();
        }

        Ok((nodes, libra_root_key))
    }

    pub fn build_libra_root_key(&self) -> (Ed25519PrivateKey, [u8; 32]) {
        let mut seeded_rng = StdRng::from_seed(self.seed);
        let libra_root_key = Ed25519PrivateKey::generate(&mut seeded_rng);
        let config_seed: [u8; 32] = seeded_rng.gen();
        (libra_root_key, config_seed)
    }

    fn build_safety_rules(&self, config: &mut NodeConfig) -> Result<()> {
        let safety_rules_config = &mut config.consensus.safety_rules;

        if let Some(server_address) = self.safety_rules_addr {
            safety_rules_config.service = SafetyRulesService::Process(RemoteService {
                server_address: server_address.into(),
            })
        }

        if let Some(backend) = &self.safety_rules_backend {
            safety_rules_config.backend = match backend.as_str() {
                "in-memory" => SecureBackend::InMemoryStorage,
                "on-disk" => SecureBackend::OnDiskStorage(OnDiskStorageConfig::default()),
                "vault" => SecureBackend::Vault(VaultConfig {
                    namespace: self.safety_rules_namespace.clone(),
                    server: self
                        .safety_rules_host
                        .as_ref()
                        .ok_or_else(|| Error::MissingSafetyRulesHost)?
                        .clone(),
                    ca_certificate: None,
                    token: Token::FromConfig(
                        self.safety_rules_token
                            .as_ref()
                            .ok_or_else(|| Error::MissingSafetyRulesToken)?
                            .clone(),
                    ),
                    renew_ttl_secs: None,
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
        validator_config.num_nodes = 2;
        validator_config.node_index = 1;

        let config = validator_config.build().unwrap();
        let network = config.validator_network.as_ref().unwrap();

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

    #[test]
    fn verify_same_genesis() {
        let mut config1 = ValidatorConfig::new();
        config1.num_nodes = 10;
        config1.node_index = 1;
        let config1 = config1.build().unwrap();

        let mut config2 = ValidatorConfig::new();
        config2.num_nodes = 13;
        config2.node_index = 12;
        config2.num_nodes_in_genesis = Some(10);
        let config2 = config2.build().unwrap();

        assert_eq!(config1.execution.genesis, config2.execution.genesis);
    }
}
