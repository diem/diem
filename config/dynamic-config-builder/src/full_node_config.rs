// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, ValidatorConfig};
use anyhow::{ensure, Result};
use libra_config::{config::NodeConfig, generator};
use parity_multiaddr::Multiaddr;

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

    pub fn template(&mut self, template: NodeConfig) -> &mut Self {
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
        let mut configs = generator::full_node_swarm(
            &mut validator_config,
            self.template.clone_for_template(),
            self.full_nodes,
            true,
            true,
            Some(self.full_node_seed),
            self.permissioned,
            false,
        )?;
        Ok(configs.swap_remove(self.full_node_index))
    }
}
