// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use crate::util::gen_genesis_transaction;
use config::{
    config::{
        BaseConfig, ConsensusConfig, NetworkConfig, NodeConfig, NodeConfigHelpers,
        PersistableConfig, RoleType, VMPublishingOption,
    },
    keys::{ConsensusKeyPair, NetworkKeyPairs},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{
        ConfigHelpers, ConsensusPeersConfig, NetworkPeerPrivateKeys, NetworkPeersConfig,
    },
};
use crypto::{ed25519::*, test_utils::KeyPair};
use failure::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct SwarmConfig {
    pub configs: Vec<(PathBuf, NodeConfig)>,
    pub seed_peers: (PathBuf, SeedPeersConfig),
    pub network_peers: (PathBuf, NetworkPeersConfig),
    pub consensus_peers: (PathBuf, ConsensusPeersConfig),
}

impl SwarmConfig {
    pub fn new(
        mut template: NodeConfig,
        num_nodes: usize,
        role: RoleType,
        faucet_key: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
        prune_seed_peers_for_discovery: bool,
        is_ipv4: bool,
        key_seed: Option<[u8; 32]>,
        output_dir: &Path,
    ) -> Result<Self> {
        let (mut consensus_private_keys, mut consensus_peers_config, consensus_peers_file) =
            SwarmConfig::building_consensus_peers_config(
                &mut template,
                num_nodes,
                &output_dir,
                key_seed,
            );
        let (mut network_private_keys, network_peers_config, network_peers_file) =
            SwarmConfig::building_network_peers_config(
                &mut template,
                &output_dir,
                &mut consensus_peers_config,
                key_seed,
            );

        let (seed_peers_file, mut seed_peers_config) =
            SwarmConfig::building_seed_peers_config(&template, &network_peers_config, is_ipv4);

        gen_genesis_transaction(
            &output_dir.join(&template.execution.genesis_file_location),
            &faucet_key,
            &consensus_peers_config,
            &network_peers_config,
        )?;

        let configs = SwarmConfig::building_each_node_config(
            &template,
            &output_dir,
            role,
            prune_seed_peers_for_discovery,
            &seed_peers_file,
            &mut seed_peers_config,
            &mut consensus_private_keys,
            &mut network_private_keys,
        );

        Ok(Self {
            configs,
            seed_peers: (output_dir.join(seed_peers_file), seed_peers_config),
            network_peers: (output_dir.join(network_peers_file), network_peers_config),
            consensus_peers: (
                output_dir.join(consensus_peers_file),
                consensus_peers_config,
            ),
        })
    }

    fn building_consensus_peers_config(
        template: &mut NodeConfig,
        num_nodes: usize,
        output_dir: &Path,
        key_seed: Option<[u8; 32]>,
    ) -> (
        HashMap<String, Ed25519PrivateKey>,
        ConsensusPeersConfig,
        PathBuf,
    ) {
        template.base.data_dir_path = output_dir.into();
        let consensus_peers_file = template.consensus.consensus_peers_file.clone();
        let (consensus_private_keys, consensus_peers_config) =
            ConfigHelpers::get_test_consensus_config(num_nodes, key_seed);

        consensus_peers_config.save_config(&output_dir.join(&consensus_peers_file));

        (
            consensus_private_keys,
            consensus_peers_config,
            consensus_peers_file,
        )
    }

    fn building_network_peers_config(
        template: &mut NodeConfig,
        output_dir: &Path,
        consensus_peers_config: &mut ConsensusPeersConfig,
        key_seed: Option<[u8; 32]>,
    ) -> (
        HashMap<String, NetworkPeerPrivateKeys>,
        NetworkPeersConfig,
        PathBuf,
    ) {
        let network_peers_file = template.networks.get(0).unwrap().network_peers_file.clone();
        let (network_private_keys, network_peers_config) =
            ConfigHelpers::get_test_network_peers_config(&consensus_peers_config, key_seed);
        network_peers_config.save_config(&output_dir.join(&network_peers_file));

        (
            network_private_keys,
            network_peers_config,
            network_peers_file,
        )
    }

    fn building_seed_peers_config(
        template: &NodeConfig,
        network_peers_config: &NetworkPeersConfig,
        is_ipv4: bool,
    ) -> (PathBuf, SeedPeersConfig) {
        let seed_peers_file = template.networks.get(0).unwrap().seed_peers_file.clone();
        let seed_peers_config = SeedPeersConfigHelpers::get_test_config_with_ipver(
            &network_peers_config,
            None,
            is_ipv4,
        );

        (seed_peers_file, seed_peers_config)
    }

    fn building_each_node_config(
        template: &NodeConfig,
        output_dir: &Path,
        role: RoleType,
        prune_seed_peers_for_discovery: bool,
        seed_peers_file: &PathBuf,
        seed_peers_config: &mut SeedPeersConfig,
        consensus_private_keys: &mut HashMap<String, Ed25519PrivateKey>,
        network_private_keys: &mut HashMap<String, NetworkPeerPrivateKeys>,
    ) -> (Vec<(PathBuf, NodeConfig)>) {
        let mut configs = Vec::new();
        // Generate configs for all nodes.
        for (node_id, addrs) in &seed_peers_config.seed_peers {
            // Serialize keypairs on independent {node}.node.keys.toml file. This is because the
            // network_keypairs and consensus_keypair fields are skipped during
            // (de)serialization.
            let consensus_private_key = consensus_private_keys.remove_entry(node_id).unwrap().1;
            let consensus_keypair = ConsensusKeyPair::load(Some(consensus_private_key));
            let NetworkPeerPrivateKeys {
                network_signing_private_key,
                network_identity_private_key,
            } = network_private_keys.remove_entry(node_id).unwrap().1;
            let network_keypairs =
                NetworkKeyPairs::load(network_signing_private_key, network_identity_private_key);
            let mut validator_config = Self::get_config_by_role(
                &template,
                role,
                &node_id,
                &network_keypairs,
                &consensus_keypair,
                &output_dir,
                &template.storage.dir,
            );
            // If listen address is different from advertised address, we need to set it
            // appropriately below.
            validator_config.networks.get_mut(0).unwrap().listen_address = addrs[0].clone();
            validator_config
                .networks
                .get_mut(0)
                .unwrap()
                .advertised_address = addrs[0].clone();
            configs.push(validator_config);
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
                let file_name = format!("{}.node.config.toml", Self::get_alias(&config));
                let config_file = output_dir.join(file_name);
                (config_file, config)
            })
            .collect::<Vec<(PathBuf, NodeConfig)>>();
        for (path, node_config) in &configs {
            node_config.save_config(&path);
        }

        (configs)
    }

    fn get_config_by_role(
        template: &NodeConfig,
        role: RoleType,
        node_id: &str,
        network_keypairs: &NetworkKeyPairs,
        // TODO(abhayb): make Optional.
        consenus_keypair: &ConsensusKeyPair,
        output_dir: &Path,
        dir: &PathBuf,
    ) -> NodeConfig {
        let network_keys_file_name = format!("{}.node.network.keys.toml", node_id.to_string());
        network_keypairs.save_config(&output_dir.join(&network_keys_file_name));
        let consensus_keys_file_name = format!("{}.node.consensus.keys.toml", node_id.to_string());
        consenus_keypair.save_config(&output_dir.join(&consensus_keys_file_name));

        let role_string = match role {
            RoleType::Validator => "validator".to_string(),
            RoleType::FullNode => "full_node".to_string(),
        };

        let base_config = BaseConfig::new(
            template.base.data_dir_path.clone(),
            template.base.node_sync_retries,
            template.base.node_sync_channel_buffer_size,
            template.base.node_async_log_chan_size,
        );
        let template_network = template.networks.get(0).unwrap();
        let network_config = NetworkConfig {
            peer_id: node_id.to_string(),
            role: role_string,
            network_keypairs: NetworkKeyPairs::default(),
            network_keypairs_file: network_keys_file_name.into(),
            network_peers: template_network.network_peers.clone(),
            network_peers_file: template_network.network_peers_file.clone(),
            seed_peers: template_network.seed_peers.clone(),
            seed_peers_file: template_network.seed_peers_file.clone(),
            listen_address: template_network.listen_address.clone(),
            advertised_address: template_network.advertised_address.clone(),
            discovery_interval_ms: template_network.discovery_interval_ms,
            connectivity_check_interval_ms: template_network.connectivity_check_interval_ms,
            enable_encryption_and_authentication: template_network
                .enable_encryption_and_authentication,
            is_permissioned: template_network.is_permissioned,
        };
        let consensus_config = ConsensusConfig {
            max_block_size: template.consensus.max_block_size,
            proposer_type: template.consensus.proposer_type.clone(),
            contiguous_rounds: template.consensus.contiguous_rounds,
            max_pruned_blocks_in_mem: template.consensus.max_pruned_blocks_in_mem,
            pacemaker_initial_timeout_ms: template.consensus.pacemaker_initial_timeout_ms,
            consensus_keypair: ConsensusKeyPair::default(),
            consensus_keypair_file: consensus_keys_file_name.into(),
            consensus_peers: template.consensus.consensus_peers.clone(),
            consensus_peers_file: template.consensus.consensus_peers_file.clone(),
        };
        let mut config = NodeConfig {
            base: base_config,
            networks: vec![network_config],
            consensus: consensus_config,
            metrics: template.metrics.clone(),
            execution: template.execution.clone(),
            admission_control: template.admission_control.clone(),
            debug_interface: template.debug_interface.clone(),
            storage: template.storage.clone(),
            mempool: template.mempool.clone(),
            state_sync: template.state_sync.clone(),
            log_collector: template.log_collector.clone(),
            vm_config: template.vm_config.clone(),
            secret_service: template.secret_service.clone(),
        };
        NodeConfigHelpers::randomize_config_ports(&mut config);
        let alias = Self::get_alias(&config);
        config.storage.dir = dir.join(alias).join("db");
        config.vm_config.publishing_options = VMPublishingOption::Open;
        config
    }

    pub fn get_alias(config: &NodeConfig) -> String {
        let network = config.networks.get(0).unwrap();
        match (&network.role).into() {
            RoleType::Validator => format!("validator_{}", network.peer_id),
            RoleType::FullNode => format!(
                "full_node_{}_{}",
                network.peer_id, config.admission_control.admission_control_service_port
            ),
        }
    }
}

pub struct SwarmConfigBuilder {
    num_nodes: usize,
    template_path: PathBuf,
    output_dir: PathBuf,
    force_discovery: bool,
    is_ipv4: bool,
    key_seed: Option<[u8; 32]>,
    faucet_account_keypair_filepath: Option<PathBuf>,
    faucet_account_keypair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    role: RoleType,
}
impl Default for SwarmConfigBuilder {
    fn default() -> Self {
        SwarmConfigBuilder {
            num_nodes: 1,
            template_path: "config/data/configs/node.config.toml".into(),
            output_dir: "configs".into(),
            force_discovery: false,
            is_ipv4: false,
            key_seed: None,
            faucet_account_keypair_filepath: None,
            faucet_account_keypair: None,
            role: RoleType::Validator,
        }
    }
}

impl SwarmConfigBuilder {
    pub fn new() -> SwarmConfigBuilder {
        SwarmConfigBuilder::default()
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
        let mut template = NodeConfig::load_config(&self.template_path);
        // update everything in the template and then generate swarm config
        let listen_address = if self.is_ipv4 { "0.0.0.0" } else { "::1" };
        let listen_address = listen_address.to_string();
        template.admission_control.address = listen_address.clone();
        template.debug_interface.address = listen_address;
        template.execution.genesis_file_location = "genesis.blob".to_string();
        // Set and generate network peers config file
        if template
            .networks
            .get(0)
            .unwrap()
            .network_peers_file
            .as_os_str()
            .is_empty()
        {
            template.networks.get_mut(0).unwrap().network_peers_file =
                PathBuf::from("network_peers.config.toml");
        };

        // Set and generate network peers config file
        if template
            .consensus
            .consensus_peers_file
            .as_os_str()
            .is_empty()
        {
            template.consensus.consensus_peers_file = PathBuf::from("consensus_peers.config.toml");
        };

        // Set seed peers file and config. Config is populated in the loop below
        if template
            .networks
            .get(0)
            .unwrap()
            .seed_peers_file
            .as_os_str()
            .is_empty()
        {
            template.networks.get_mut(0).unwrap().seed_peers_file =
                PathBuf::from("seed_peers.config.toml");
        };

        SwarmConfig::new(
            template,
            self.num_nodes,
            self.role,
            faucet_key,
            self.force_discovery,
            self.is_ipv4,
            self.key_seed,
            &self.output_dir,
        )
    }
}
