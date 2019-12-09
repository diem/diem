// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use anyhow::{ensure, Result};
use libra_config::{
    config::{ConsensusPeersConfig, NodeConfig, SeedPeersConfig},
    generator,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::transaction::Transaction;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use vm_genesis;

const DEFAULT_SEED: [u8; 32] = [13u8; 32];
const DEFAULT_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/6180";
const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/6180";

pub struct ValidatorConfig {
    advertised: Multiaddr,
    bootstrap: Multiaddr,
    index: usize,
    ipv4: bool,
    listen: Multiaddr,
    nodes: usize,
    seed: [u8; 32],
    template: NodeConfig,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            advertised: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            bootstrap: DEFAULT_ADVERTISED.parse::<Multiaddr>().unwrap(),
            index: 0,
            ipv4: true,
            listen: DEFAULT_LISTEN.parse::<Multiaddr>().unwrap(),
            nodes: 1,
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

    pub fn seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.seed = seed;
        self
    }

    pub fn template(&mut self, template: NodeConfig) -> &mut Self {
        self.template = template;
        self
    }

    pub fn build(&self) -> Result<NodeConfig> {
        let (mut configs, faucet_key) = self.build_common()?;
        let first_config = &configs[0];
        let validator_network = first_config
            .validator_network
            .as_ref()
            .ok_or(Error::MissingValidatorNetwork)?;
        let first_peer_id = validator_network.peer_id;
        let consensus_peers = &first_config.consensus.consensus_peers;

        let genesis = Some(Transaction::UserTransaction(
            vm_genesis::encode_genesis_transaction_with_validator(
                &faucet_key,
                faucet_key.public_key(),
                consensus_peers.get_validator_set(&validator_network.network_peers),
            )
            .into_inner(),
        ));

        let config = &mut configs[self.index];
        config.execution.genesis = genesis;
        let validator_network = config
            .validator_network
            .as_mut()
            .ok_or(Error::MissingValidatorNetwork)?;
        validator_network.listen_address = self.listen.clone();
        validator_network.advertised_address = self.advertised.clone();
        let mut seed_peers = HashMap::new();
        seed_peers.insert(first_peer_id, vec![self.bootstrap.clone()]);
        validator_network.seed_peers = SeedPeersConfig { seed_peers };

        Ok(configs.swap_remove(self.index))
    }

    pub fn build_faucet_client(&self) -> Result<(ConsensusPeersConfig, Ed25519PrivateKey)> {
        let (configs, faucet_key) = self.build_common()?;
        Ok((configs[0].consensus.consensus_peers.clone(), faucet_key))
    }

    fn build_common(&self) -> Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        self.validate()?;

        let mut rng = StdRng::from_seed(self.seed);
        let faucet_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
        let config_seed: [u8; 32] = rng.gen();

        let configs = generator::validator_swarm(
            self.template.clone_for_template(),
            self.nodes,
            true,
            self.ipv4,
            Some(config_seed),
            false,
        )?;

        ensure!(
            configs.len() == self.nodes,
            Error::MissingConfigs {
                found: configs.len()
            }
        );
        Ok((configs, faucet_key))
    }

    fn validate(&self) -> Result<()> {
        ensure!(self.nodes > 0, Error::NonZeroNetwork);
        ensure!(
            self.index < self.nodes,
            Error::IndexError {
                index: self.index,
                nodes: self.nodes
            }
        );
        Ok(())
    }
}
