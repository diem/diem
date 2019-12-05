// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use anyhow::{bail, ensure, format_err, Result};
use libra_config::{
    config::{NodeConfig, PersistableConfig, RoleType, VMPublishingOption},
    generator,
};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use libra_types::transaction::Transaction;
use std::path::{Path, PathBuf};
use vm_genesis;

pub struct SwarmConfigBuilder {
    num_nodes: usize,
    template_path: Option<PathBuf>,
    output_dir: PathBuf,
    force_discovery: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
    faucet_account_keypair_filepath: Option<PathBuf>,
    faucet_account_keypair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    role: RoleType,
    upstream_peer_dir: Option<String>,
    is_permissioned: bool,
}

impl Default for SwarmConfigBuilder {
    fn default() -> Self {
        SwarmConfigBuilder {
            num_nodes: 1,
            template_path: None,
            output_dir: "configs".into(),
            force_discovery: false,
            is_ipv4: false,
            key_seed: None,
            faucet_account_keypair_filepath: None,
            faucet_account_keypair: None,
            role: RoleType::Validator,
            upstream_peer_dir: None,
            is_permissioned: true,
        }
    }
}

impl SwarmConfigBuilder {
    pub fn new() -> SwarmConfigBuilder {
        SwarmConfigBuilder::default()
    }

    pub fn with_base<P: AsRef<Path>>(&mut self, base_template_path: P) -> &mut Self {
        self.template_path = Some(base_template_path.as_ref().to_path_buf());
        self
    }

    pub fn with_output_dir<P: AsRef<Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    pub fn with_faucet_keypair_filepath<P: AsRef<Path>>(&mut self, keypair_file: P) -> &mut Self {
        self.faucet_account_keypair_filepath = Some(keypair_file.as_ref().to_path_buf());
        self
    }

    pub fn with_faucet_keypair(
        &mut self,
        keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    ) -> &mut Self {
        self.faucet_account_keypair = Some(keypair);
        self
    }

    pub fn with_num_nodes(&mut self, num_nodes: usize) -> &mut Self {
        self.num_nodes = num_nodes;
        self
    }

    pub fn with_role(&mut self, role: RoleType) -> &mut Self {
        self.role = role;
        self
    }

    pub fn force_discovery(&mut self) -> &mut Self {
        self.force_discovery = true;
        self
    }

    pub fn with_ipv4(&mut self) -> &mut Self {
        self.is_ipv4 = true;
        self
    }

    pub fn with_ipv6(&mut self) -> &mut Self {
        self.is_ipv4 = false;
        self
    }

    pub fn with_key_seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.key_seed = Some(seed);
        self
    }

    pub fn with_upstream_config_dir(&mut self, upstream_peer_dir: Option<String>) -> &mut Self {
        self.upstream_peer_dir = upstream_peer_dir;
        self
    }

    pub fn with_is_permissioned(&mut self, is_permissioned: bool) -> &mut Self {
        self.is_permissioned = is_permissioned;
        self
    }

    pub fn build(&mut self) -> Result<SwarmConfig> {
        ensure!(self.num_nodes > 0, "Cannot build 0 NodeConfigs");

        let faucet_key = if let Some(keypair) = self.faucet_account_keypair.take() {
            keypair
        } else {
            let faucet_key_path = self
                .faucet_account_keypair_filepath
                .as_ref()
                .ok_or_else(|| format_err!("Must provide faucet keys or faucet key file"))?;
            generate_keypair::load_key_from_file(faucet_key_path)?
        };

        if !self.output_dir.is_dir() {
            if !self.output_dir.exists() {
                std::fs::create_dir(&self.output_dir)?;
            }
            ensure!(
                !self.output_dir.is_file(),
                "Output-dir is a file, expecting a directory"
            );
        }

        let mut template = if let Some(template_path) = &&self.template_path {
            NodeConfig::load_config(template_path)?
        } else {
            NodeConfig::default()
        };

        // @TODO this shouldn't be hardcode and instead should be passed in via the template but
        // this interface expects a path not a NodeConfig complicating things...
        template.vm_config.publishing_options = VMPublishingOption::Open;

        let mut configs;
        let genesis;

        if self.role.is_validator() {
            configs = generator::validator_swarm(
                template,
                self.num_nodes,
                self.force_discovery,
                self.is_ipv4,
                self.key_seed,
                true,
            )?;
            let config = configs
                .get(0)
                .ok_or_else(|| format_err!("Missing NodeConfig"))?;
            let consensus_peers = &config.consensus.consensus_peers;
            let network = match config.validator_network.as_ref() {
                Some(network) => network,
                None => bail!("Unable to read validator_network"),
            };

            genesis = Some(Transaction::UserTransaction(
                vm_genesis::encode_genesis_transaction_with_validator(
                    &faucet_key.private_key,
                    faucet_key.public_key.clone(),
                    consensus_peers.get_validator_set(&network.network_peers),
                )
                .into_inner(),
            ));
        } else {
            let upstream_peer_dir = self
                .upstream_peer_dir
                .as_ref()
                .ok_or_else(|| format_err!("Missing upstream_peer_dir"))?;
            let upstream_peer_dir = PathBuf::from(upstream_peer_dir);
            let mut upstream_peer = NodeConfig::load(upstream_peer_dir.join("node.config.toml"))?;
            genesis = upstream_peer.execution.genesis.clone();
            configs = generator::full_node_swarm(
                &mut upstream_peer,
                template,
                self.num_nodes,
                self.force_discovery,
                self.is_ipv4,
                self.key_seed,
                self.is_permissioned,
                true,
            )?;
            upstream_peer.save(&PathBuf::from("node.config.toml"));
        }

        ensure!(
            configs.len() == self.num_nodes,
            format!(
                "Expected {}, found {} NodeConfigs",
                configs.len(),
                self.num_nodes
            )
        );

        let mut config_files = vec![];

        for (index, config) in configs.iter_mut().enumerate() {
            let node_dir = self.output_dir.join(format!("{}", index));
            std::fs::create_dir_all(&node_dir).expect("unable to create config dir");

            config.set_data_dir(node_dir.clone())?;
            config_files.push(node_dir.join("node.config.toml"));
            config.execution.genesis = genesis.clone();
            config.save(&PathBuf::from("node.config.toml"));
        }

        Ok(SwarmConfig { config_files })
    }
}

pub struct SwarmConfig {
    pub config_files: Vec<PathBuf>,
}
