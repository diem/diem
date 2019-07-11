// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use crate::util::gen_genesis_transaction;
use config::{
    config::{KeyPairs, NodeConfig, NodeConfigHelpers, VMPublishingOption},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{TrustedPeersConfig, TrustedPeersConfigHelpers},
};
use crypto::signing::KeyPair;
use failure::prelude::*;
use std::path::{Path, PathBuf};

pub struct SwarmConfig {
    configs: Vec<(PathBuf, NodeConfig)>,
    seed_peers: (PathBuf, SeedPeersConfig),
    trusted_peers: (PathBuf, TrustedPeersConfig),
}

impl SwarmConfig {
    //TODO convert this to use the Builder paradigm
    pub fn new(
        mut template: NodeConfig,
        num_nodes: usize,
        faucet_key: KeyPair,
        prune_seed_peers_for_discovery: bool,
        is_ipv4: bool,
        key_seed: Option<[u8; 32]>,
        output_dir: &Path,
        static_ports: bool,
    ) -> Result<Self> {
        // Generate trusted peer configs + their private keys.
        template.base.data_dir_path = output_dir.into();
        let (peers_private_keys, trusted_peers_config) =
            TrustedPeersConfigHelpers::get_test_config(num_nodes, key_seed);
        trusted_peers_config.save_config(&output_dir.join(&template.base.trusted_peers_file));
        let mut seed_peers_config = SeedPeersConfigHelpers::get_test_config_with_ipver(
            &trusted_peers_config,
            None,
            is_ipv4,
        );

        gen_genesis_transaction(
            &output_dir.join(&template.execution.genesis_file_location),
            &faucet_key,
            &trusted_peers_config,
        )?;

        let mut configs = Vec::new();
        // Generate configs for all nodes.
        for (node_id, addrs) in &seed_peers_config.seed_peers {
            let mut config = template.clone();
            config.base.peer_id = node_id.clone();
            // serialize keypairs on independent {node}.node.keys.toml file
            // this is because the peer_keypairs field is skipped during (de)serialization
            let private_keys = peers_private_keys.get(node_id.as_str()).unwrap();
            let peer_keypairs = KeyPairs::load(private_keys);
            let key_file_name = format!("{}.node.keys.toml", config.base.peer_id);

            config.base.peer_keypairs_file = key_file_name.into();
            peer_keypairs.save_config(&output_dir.join(&config.base.peer_keypairs_file));
            if !static_ports {
                NodeConfigHelpers::randomize_config_ports(&mut config);
            }

            // create subdirectory for storage: <node_id>/db, unless provided directly
            config.storage.dir = config.storage.dir.join(node_id).join("db");

            // If listen address is different from advertised address, we need to set it
            // appropriately below.
            config.network.listen_address = addrs[0].clone();
            config.network.advertised_address = addrs[0].clone();

            config.vm_config.publishing_options = VMPublishingOption::Open;
            configs.push(config);
        }
        if prune_seed_peers_for_discovery {
            seed_peers_config.seed_peers = seed_peers_config
                .seed_peers
                .clone()
                .into_iter()
                .take(1)
                .collect();
        }
        seed_peers_config.save_config(&output_dir.join(&template.network.seed_peers_file));
        let configs = configs
            .into_iter()
            .map(|config| {
                let file_name = format!("{}.node.config.toml", config.base.peer_id);
                let config_file = output_dir.join(file_name);
                (config_file, config)
            })
            .collect::<Vec<(PathBuf, NodeConfig)>>();

        for (path, node_config) in &configs {
            node_config.save_config(&path);
        }

        Ok(Self {
            configs,
            seed_peers: (
                output_dir.join(template.network.seed_peers_file),
                seed_peers_config,
            ),
            trusted_peers: (
                output_dir.join(template.base.trusted_peers_file),
                trusted_peers_config,
            ),
        })
    }

    pub fn get_configs(&self) -> &[(PathBuf, NodeConfig)] {
        &self.configs
    }

    pub fn get_seed_peers_config(&self) -> &(PathBuf, SeedPeersConfig) {
        &self.seed_peers
    }

    pub fn get_trusted_peers_config(&self) -> &(PathBuf, TrustedPeersConfig) {
        &self.trusted_peers
    }
}

pub struct SwarmConfigBuilder {
    node_count: usize,
    template_path: PathBuf,
    static_ports: bool,
    output_dir: PathBuf,
    force_discovery: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
    faucet_account_keypair_filepath: Option<PathBuf>,
    faucet_account_keypair: Option<KeyPair>,
}
impl Default for SwarmConfigBuilder {
    fn default() -> Self {
        SwarmConfigBuilder {
            node_count: 1,
            template_path: "config/data/configs/node.config.toml".into(),
            static_ports: false,
            output_dir: "configs".into(),
            force_discovery: false,
            is_ipv4: false,
            key_seed: None,
            faucet_account_keypair_filepath: None,
            faucet_account_keypair: None,
        }
    }
}

impl SwarmConfigBuilder {
    pub fn new() -> SwarmConfigBuilder {
        SwarmConfigBuilder::default()
    }

    pub fn randomize_ports(&mut self) -> &mut Self {
        self.static_ports = false;
        self
    }

    pub fn static_ports(&mut self) -> &mut Self {
        self.static_ports = true;
        self
    }

    pub fn with_base<P: AsRef<Path>>(&mut self, base_template_path: P) -> &mut Self {
        self.template_path = base_template_path.as_ref().to_path_buf();
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

    pub fn with_faucet_keypair(&mut self, keypair: KeyPair) -> &mut Self {
        self.faucet_account_keypair = Some(keypair);
        self
    }

    pub fn with_nodes(&mut self, n: usize) -> &mut Self {
        self.node_count = n;
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

    pub fn build(&self) -> Result<SwarmConfig> {
        // verify required fields
        let faucet_key_path = self.faucet_account_keypair_filepath.clone();
        let faucet_key_option = self.faucet_account_keypair.clone();
        let faucet_key = faucet_key_option.unwrap_or_else(|| {
            generate_keypair::load_key_from_file(
                faucet_key_path.expect("Must provide faucet key file"),
            )
            .expect("Faucet account key is required to generate config")
        });

        // generate all things needed for generation
        if !self.output_dir.is_dir() {
            if !self.output_dir.exists() {
                // generate if doesn't exist
                std::fs::create_dir(&self.output_dir).expect("Failed to create output dir");
            }
            assert!(
                !self.output_dir.is_file(),
                "Output-dir is a file, expecting a directory"
            );
        }

        // read template
        let mut template = NodeConfig::load_template(&self.template_path)?;
        // update everything in the template and then generate swarm config
        let listen_address = if self.is_ipv4 { "0.0.0.0" } else { "::1" };
        let listen_address = listen_address.to_string();
        template.admission_control.address = listen_address.clone();
        template.debug_interface.address = listen_address;

        template.execution.genesis_file_location = "genesis.blob".to_string();

        // Set and generate trusted peers config file
        if template.base.trusted_peers_file.is_empty() {
            template.base.trusted_peers_file = "trusted_peers.config.toml".to_string();
        };
        // Set seed peers file and config. Config is populated in the loop below
        if template.network.seed_peers_file.is_empty() {
            template.network.seed_peers_file = "seed_peers.config.toml".to_string();
        };

        SwarmConfig::new(
            template,
            self.node_count,
            faucet_key,
            self.force_discovery,
            self.is_ipv4,
            self.key_seed,
            &self.output_dir,
            self.static_ports,
        )
    }
}
