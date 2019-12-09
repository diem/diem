// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
use thiserror::Error;
use vm_genesis;

const DEFAULT_SEED: [u8; 32] = [13u8; 32];
const DEFAULT_IPV4_ADVERTISED: &str = "/ip4/127.0.0.1/tcp/6180";
const DEFAULT_IPV4_LISTEN: &str = "/ip4/0.0.0.0/tcp/6180";
const DEFAULT_IPV6_ADVERTISED: &str = "/ip6/::1/tcp/6180";
const DEFAULT_IPV6_LISTEN: &str = "/ip6/::/tcp/6180";

pub struct DynamicConfigBuilder {
    advertised: Multiaddr,
    bootstrap: Multiaddr,
    index: usize,
    ipv4: bool,
    listen: Multiaddr,
    nodes: usize,
    seed: [u8; 32],
    template: NodeConfig,
}

impl Default for DynamicConfigBuilder {
    fn default() -> Self {
        Self {
            advertised: DEFAULT_IPV4_ADVERTISED.parse::<Multiaddr>().unwrap(),
            bootstrap: DEFAULT_IPV4_ADVERTISED.parse::<Multiaddr>().unwrap(),
            index: 0,
            ipv4: true,
            listen: DEFAULT_IPV4_LISTEN.parse::<Multiaddr>().unwrap(),
            nodes: 1,
            seed: DEFAULT_SEED,
            template: NodeConfig::default(),
        }
    }
}

impl DynamicConfigBuilder {
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

    pub fn ipv4(&mut self, ipv4: bool) -> &mut Self {
        self.ipv4 = ipv4;
        if self.ipv4 {
            if self.advertised.to_string() == DEFAULT_IPV6_ADVERTISED {
                self.advertised = DEFAULT_IPV4_ADVERTISED.parse::<Multiaddr>().unwrap();
            }
            if self.bootstrap.to_string() == DEFAULT_IPV6_ADVERTISED {
                self.bootstrap = DEFAULT_IPV4_ADVERTISED.parse::<Multiaddr>().unwrap();
            }
            if self.listen.to_string() == DEFAULT_IPV6_ADVERTISED {
                self.listen = DEFAULT_IPV4_LISTEN.parse::<Multiaddr>().unwrap();
            }
        } else {
            if self.advertised.to_string() == DEFAULT_IPV4_ADVERTISED {
                self.advertised = DEFAULT_IPV6_ADVERTISED.parse::<Multiaddr>().unwrap();
            }
            if self.bootstrap.to_string() == DEFAULT_IPV4_ADVERTISED {
                self.bootstrap = DEFAULT_IPV6_ADVERTISED.parse::<Multiaddr>().unwrap();
            }
            if self.listen.to_string() == DEFAULT_IPV4_ADVERTISED {
                self.listen = DEFAULT_IPV6_LISTEN.parse::<Multiaddr>().unwrap();
            }
        }
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
            .ok_or(DynamicConfigBuilderError::MissingValidatorNetwork)?;
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
            .ok_or(DynamicConfigBuilderError::MissingValidatorNetwork)?;
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

        let (faucet_key, mut rng) = init_from_seed(self.seed);
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
            DynamicConfigBuilderError::MissingConfigs {
                found: configs.len()
            }
        );
        Ok((configs, faucet_key))
    }

    fn validate(&self) -> Result<()> {
        ensure!(self.nodes > 0, DynamicConfigBuilderError::NonZeroNetwork);
        ensure!(
            self.index < self.nodes,
            DynamicConfigBuilderError::IndexError {
                index: self.index,
                nodes: self.nodes
            }
        );
        Ok(())
    }
}

// This function is used by cluster test, if updated please make sure it is still working
pub fn init_from_seed(seed: [u8; 32]) -> (Ed25519PrivateKey, StdRng) {
    let mut rng = StdRng::from_seed(seed);
    (Ed25519PrivateKey::generate_for_testing(&mut rng), rng)
}

#[derive(Error, Debug)]
enum DynamicConfigBuilderError {
    #[error("index out of range: {} >= {}", index, nodes)]
    IndexError { index: usize, nodes: usize },
    #[error("Missing configs only found {}", found)]
    MissingConfigs { found: usize },
    #[error("Config does not contain a validator network")]
    MissingValidatorNetwork,
    #[error("network size should be at least 1")]
    NonZeroNetwork,
}
