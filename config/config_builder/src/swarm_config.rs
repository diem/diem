// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use crate::util::gen_genesis_transaction;
use config::{
    config::{BaseConfig, KeyPairs, NodeConfig, NodeConfigHelpers, VMPublishingOption},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{TrustedPeersConfig, TrustedPeersConfigHelpers},
};
use failure::prelude::*;
use nextgen_crypto::{ed25519::*, test_utils::KeyPair};
use std::path::{Path, PathBuf};

/// Topology indicates the shape of the validator network
/// Currently does not handle full nodes, the launch_swarm will only use num_nodes value
#[derive(Debug, Clone)]
pub struct LibraSwarmTopology {
    // TODO: make it more general to support various network shapes
    data: Vec<usize>,
}

impl LibraSwarmTopology {
    pub fn create_validator_network(num_validator_nodes: usize) -> Self {
        Self {
            data: vec![num_validator_nodes],
        }
    }

    pub fn create_uniform_network(num_validator_nodes: usize, num_full_nodes: usize) -> Self {
        Self {
            data: vec![num_validator_nodes, num_full_nodes],
        }
    }

    pub fn num_validators(&self) -> usize {
        if !self.data.is_empty() {
            return self.data[0];
        }
        0
    }

    pub fn num_full_nodes(&self) -> usize {
        if self.data.len() > 1 {
            return self.data[1];
        }
        0
    }
}

pub struct SwarmConfig {
    configs: Vec<(PathBuf, NodeConfig)>,
    seed_peers: (PathBuf, SeedPeersConfig),
    trusted_peers: (PathBuf, TrustedPeersConfig),
}

impl SwarmConfig {
    //TODO convert this to use the Builder paradigm
    pub fn new(
        mut template: NodeConfig,
        topology: &LibraSwarmTopology,
        faucet_key: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
        prune_seed_peers_for_discovery: bool,
        is_ipv4: bool,
        key_seed: Option<[u8; 32]>,
        output_dir: &Path,
        static_ports: bool,
    ) -> Result<Self> {
        // Generate trusted peer configs + their private keys.
        template.base.data_dir_path = output_dir.into();
        let (mut peers_private_keys, trusted_peers_config) =
            TrustedPeersConfigHelpers::get_test_config(topology.num_validators(), key_seed);
        let trusted_peers_file = template.base.trusted_peers_file.clone();
        let seed_peers_file = template.network.seed_peers_file.clone();
        trusted_peers_config.save_config(&output_dir.join(&trusted_peers_file));
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
            let key_file_name = format!("{}.node.keys.toml", node_id.clone());

            let base_config = BaseConfig::new(
                node_id.clone(),
                template.base.role.clone(),
                KeyPairs::default(),
                key_file_name.into(),
                template.base.data_dir_path.clone(),
                trusted_peers_file.clone(),
                template.base.trusted_peers.clone(),
                template.base.node_sync_retries,
                template.base.node_sync_channel_buffer_size,
                template.base.node_async_log_chan_size,
            );
            let mut config = NodeConfig {
                base: base_config,
                metrics: template.metrics.clone(),
                execution: template.execution.clone(),
                admission_control: template.admission_control.clone(),
                debug_interface: template.debug_interface.clone(),
                storage: template.storage.clone(),
                network: template.network.clone(),
                consensus: template.consensus.clone(),
                mempool: template.mempool.clone(),
                state_sync: template.state_sync.clone(),
                log_collector: template.log_collector.clone(),
                vm_config: template.vm_config.clone(),
                secret_service: template.secret_service.clone(),
            };

            config.base.peer_id = node_id.clone();
            // serialize keypairs on independent {node}.node.keys.toml file
            // this is because the peer_keypairs field is skipped during (de)serialization
            let private_keys = peers_private_keys
                .remove_entry(node_id.as_str())
                .expect(
                    &format!(
                        "Seed peer {} not present in peer private keys, aborting",
                        node_id.as_str()
                    )[..],
                )
                .1;
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
        seed_peers_config.save_config(&output_dir.join(&seed_peers_file));
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
            seed_peers: (output_dir.join(seed_peers_file), seed_peers_config),
            trusted_peers: (output_dir.join(trusted_peers_file), trusted_peers_config),
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
    topology: LibraSwarmTopology,
    template_path: PathBuf,
    static_ports: bool,
    output_dir: PathBuf,
    force_discovery: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
    faucet_account_keypair_filepath: Option<PathBuf>,
    faucet_account_keypair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
}
impl Default for SwarmConfigBuilder {
    fn default() -> Self {
        SwarmConfigBuilder {
            topology: LibraSwarmTopology::create_validator_network(1),
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

    pub fn with_faucet_keypair(
        &mut self,
        keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    ) -> &mut Self {
        self.faucet_account_keypair = Some(keypair);
        self
    }

    pub fn with_topology(&mut self, topology: LibraSwarmTopology) -> &mut Self {
        self.topology = topology;
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

    pub fn build(&mut self) -> Result<SwarmConfig> {
        // verify required fields
        let faucet_key_path = self.faucet_account_keypair_filepath.clone();
        let faucet_key = self.faucet_account_keypair.take().unwrap_or_else(|| {
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
            &self.topology,
            faucet_key,
            self.force_discovery,
            self.is_ipv4,
            self.key_seed,
            &self.output_dir,
            self.static_ports,
        )
    }
}
