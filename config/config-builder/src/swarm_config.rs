// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use crate::util::gen_genesis_transaction_bytes;
use failure::prelude::*;
use libra_config::{
    config::{
        BaseConfig, ConsensusConfig, NetworkConfig, NodeConfig, NodeConfigHelpers,
        PersistableConfig, RoleType, VMPublishingOption,
    },
    keys::{ConsensusKeyPair, NetworkKeyPairs},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{
        ConfigHelpers, ConsensusPeersConfig, ConsensusPrivateKey, NetworkPeersConfig,
        NetworkPrivateKeys, UpstreamPeersConfig,
    },
    utils::get_available_port,
};
use libra_crypto::{ed25519::*, test_utils::KeyPair};
use libra_logger::prelude::*;
use libra_types::PeerId;
use parity_multiaddr::{Multiaddr, Protocol};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct SwarmConfig {
    pub configs: Vec<PathBuf>,
}

impl SwarmConfig {
    pub fn new_full_node_swarm(
        mut template: NodeConfig,
        num_nodes: usize,
        prune_seed_peers_for_discovery: bool,
        is_ipv4: bool,
        key_seed: Option<[u8; 32]>,
        output_dir: &Path,
        is_permissioned: bool,
        upstream_config_dir: PathBuf,
    ) -> Result<Self> {
        // Load upstream peer config from file.
        let mut upstream_peer_config =
            NodeConfig::load(&upstream_config_dir.join("node.config.toml"))?;
        // Generate new network config for upstream peer (permissioned if so).
        let (mut upstream_private_keys, upstream_network_peers_config) =
            ConfigHelpers::gen_full_nodes(1, Some([2u8; 32]));
        let upstream_peer_id = *upstream_private_keys.keys().nth(0).unwrap();
        let upstream_private_keys = upstream_private_keys
            .remove_entry(&upstream_peer_id)
            .unwrap()
            .1;
        let upstream_network_keypairs = NetworkKeyPairs::load(
            upstream_private_keys.network_signing_private_key,
            upstream_private_keys.network_identity_private_key,
        );
        let template_network = template.networks.get(0).unwrap();
        // Generate upstream peer address.
        let upstream_full_node_address = {
            let mut addr = Multiaddr::empty();
            if is_ipv4 {
                addr.push(Protocol::Ip4("0.0.0.0".parse().unwrap()));
            } else {
                addr.push(Protocol::Ip6("::1".parse().unwrap()));
            }
            addr.push(Protocol::Tcp(get_available_port()));
            addr
        };
        // Save new network keys for upstream peer.
        let upstream_network_keys_file_name =
            format!("{}.network.keys.toml", upstream_peer_id.to_string());
        upstream_network_keypairs
            .save_config(&upstream_config_dir.join(&upstream_network_keys_file_name));
        // Create network config for upstream node.
        let mut upstream_full_node_config = NetworkConfig {
            peer_id: upstream_peer_id.to_string(),
            role: "full_node".to_string(),
            network_keypairs_file: upstream_network_keys_file_name.into(),
            network_peers_file: template_network.network_peers_file.clone(),
            seed_peers_file: template_network.seed_peers_file.clone(),
            listen_address: upstream_full_node_address.clone(),
            advertised_address: upstream_full_node_address.clone(),
            discovery_interval_ms: template_network.discovery_interval_ms,
            connectivity_check_interval_ms: template_network.connectivity_check_interval_ms,
            enable_encryption_and_authentication: template_network
                .enable_encryption_and_authentication,
            is_permissioned,
            // Dummy values - will be loaded from corresponding files.
            network_keypairs: NetworkKeyPairs::default(),
            network_peers: template_network.network_peers.clone(),
            seed_peers: template_network.seed_peers.clone(),
        };
        let (mut private_keys, mut network_peers_config) =
            ConfigHelpers::gen_full_nodes(num_nodes, key_seed);
        // Add upstream peer to NetworkPeersConfig.
        network_peers_config
            .peers
            .extend(upstream_network_peers_config.peers.into_iter());
        // Update network peers config in upstream peer config if permissioned.
        if is_permissioned {
            let upstream_network_peers_file_name =
                format!("{}.network_peers.config.toml", upstream_peer_id.to_string());
            network_peers_config
                .save_config(&upstream_config_dir.join(&upstream_network_peers_file_name));
            upstream_full_node_config.network_peers_file = upstream_network_peers_file_name.into();
        }
        // Modify upstream peer config to add the new network config.
        upstream_peer_config
            .networks
            .push(upstream_full_node_config);
        // Write contents of upstream config to file.
        upstream_peer_config.save_config(&upstream_config_dir.join("node.config.toml"));
        // Add upstream peer to StateSync::UpstreamPeersConfig.
        template.state_sync.upstream_peers = UpstreamPeersConfig {
            upstream_peers: vec![upstream_peer_id.to_string()],
        };
        // Setup seed peers config.
        let mut seed_peers_config = SeedPeersConfigHelpers::get_test_config_with_ipver(
            &network_peers_config,
            None,
            is_ipv4,
        );
        // Extract peer addresses for full nodes from seed peer config.
        let peer_addresses: BTreeMap<_, _> = seed_peers_config
            .seed_peers
            .clone()
            .into_iter()
            .filter(|(peer_id, _)| *peer_id != upstream_peer_id.to_string())
            .collect();
        // Prune seed peers config to a single node if needed.
        if prune_seed_peers_for_discovery {
            seed_peers_config.seed_peers = seed_peers_config
                .seed_peers
                .clone()
                .into_iter()
                .take(1)
                .collect();
        }
        // Add upstream peer to SeedPeersConfig of full nodes.
        seed_peers_config.seed_peers.insert(
            upstream_peer_id.to_string(),
            vec![upstream_full_node_address.clone()],
        );
        // Load contents of consensus peers file from upstream node.
        let consensus_peers_config = ConsensusPeersConfig::load_config(
            &upstream_config_dir.join(&upstream_peer_config.consensus.consensus_peers_file),
        );
        // NOTE: Need to restart upstream node with new configuration.
        let mut configs = Vec::new();
        // Generate configs for all nodes.
        for (index, (node_id, addrs)) in peer_addresses.iter().enumerate() {
            let node_dir = output_dir.join(format!("{}", index));
            std::fs::create_dir_all(&node_dir).expect("unable to create config dir");
            // Copy contents of genesis file from upstream node.
            let genesis_transaction_file_target =
                node_dir.join(&template.execution.genesis_file_location);
            let genesis_transaction_file_src =
                upstream_config_dir.join(&upstream_peer_config.execution.genesis_file_location);
            fs::copy(
                genesis_transaction_file_src.clone(),
                genesis_transaction_file_target.clone(),
            )
            .or_else(|err| {
                error!(
                    "Failed to copy {:?} to {:?}: {}",
                    genesis_transaction_file_src, genesis_transaction_file_target, err
                );
                Err(err)
            })?;
            // Remove network private keys for this peer.
            let peer_id = PeerId::from_str(&node_id).unwrap();
            warn!("Looking for peer id for peer: {}", node_id);
            let NetworkPrivateKeys {
                network_signing_private_key,
                network_identity_private_key,
            } = private_keys
                .remove_entry(&peer_id)
                .unwrap_or_else(|| panic!("Key not found for peer: {}", node_id))
                .1;
            let network_keypairs =
                NetworkKeyPairs::load(network_signing_private_key, network_identity_private_key);
            let full_node_config = Self::get_config_by_role(
                &template,
                RoleType::FullNode,
                &node_id,
                &network_keypairs,
                &ConsensusKeyPair::load(None),
                &seed_peers_config,
                &network_peers_config,
                &consensus_peers_config,
                &node_dir,
                &addrs,
            );
            let config_file = node_dir.join("node.config.toml");
            full_node_config.save_config(&config_file);
            configs.push(config_file);
        }
        Ok(Self { configs })
    }

    pub fn new_validator_swarm(
        template: NodeConfig,
        num_nodes: usize,
        faucet_key: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
        prune_seed_peers_for_discovery: bool,
        is_ipv4: bool,
        key_seed: Option<[u8; 32]>,
        output_dir: &Path,
    ) -> Result<Self> {
        let (mut private_keys, consensus_peers_config, network_peers_config) =
            ConfigHelpers::gen_validator_nodes(num_nodes, key_seed);
        let mut seed_peers_config = SeedPeersConfigHelpers::get_test_config_with_ipver(
            &network_peers_config,
            None,
            is_ipv4,
        );
        let raw_genesis_transaction = gen_genesis_transaction_bytes(
            &faucet_key,
            &consensus_peers_config,
            &network_peers_config,
        );
        // Extract peer addresses from seed peer config.
        let peer_addresses: BTreeMap<_, _> =
            seed_peers_config.seed_peers.clone().into_iter().collect();
        // Prune seed peers config if needed.
        if prune_seed_peers_for_discovery {
            seed_peers_config.seed_peers = seed_peers_config
                .seed_peers
                .clone()
                .into_iter()
                .take(1)
                .collect();
        }
        let mut configs = Vec::new();
        // Generate configs for all nodes.
        for (index, (node_id, addrs)) in peer_addresses.iter().enumerate() {
            let node_dir = output_dir.join(format!("{}", index));
            std::fs::create_dir_all(&node_dir).expect("unable to create config dir");
            debug!("Directory for node {}: {:?}", index, node_dir);
            // Save genesis transaction in file.
            let mut genesis_transaction_file =
                File::create(&node_dir.join(&template.execution.genesis_file_location))?;
            genesis_transaction_file.write_all(&raw_genesis_transaction)?;
            let peer_id = PeerId::from_str(&node_id).unwrap();
            let (
                ConsensusPrivateKey {
                    consensus_private_key,
                },
                NetworkPrivateKeys {
                    network_signing_private_key,
                    network_identity_private_key,
                },
            ) = private_keys.remove_entry(&peer_id).unwrap().1;
            let consensus_keypair = ConsensusKeyPair::load(Some(consensus_private_key));
            let network_keypairs =
                NetworkKeyPairs::load(network_signing_private_key, network_identity_private_key);
            let validator_config = Self::get_config_by_role(
                &template,
                RoleType::Validator,
                &node_id,
                &network_keypairs,
                &consensus_keypair,
                &seed_peers_config,
                &network_peers_config,
                &consensus_peers_config,
                &node_dir,
                &addrs,
            );
            let config_file = node_dir.join("node.config.toml");
            validator_config.save_config(&config_file);
            configs.push(config_file);
        }
        Ok(Self { configs })
    }

    fn get_config_by_role(
        template: &NodeConfig,
        role: RoleType,
        node_id: &str,
        network_keypairs: &NetworkKeyPairs,
        consenus_keypair: &ConsensusKeyPair,
        seed_peers_config: &SeedPeersConfig,
        network_peers_config: &NetworkPeersConfig,
        consensus_peers_config: &ConsensusPeersConfig,
        output_dir: &Path,
        addrs: &[Multiaddr],
    ) -> NodeConfig {
        // Save consensus keys if present.
        let mut consensus_keys_file_name = "".to_string();
        if consenus_keypair.is_present() {
            consensus_keys_file_name = format!("{}.node.consensus.keys.toml", node_id.to_string());
            consenus_keypair.save_config(&output_dir.join(&consensus_keys_file_name));
        }
        // Save network keys.
        let network_keys_file_name = format!("{}.node.network.keys.toml", node_id.to_string());
        network_keypairs.save_config(&output_dir.join(&network_keys_file_name));
        // Save seed peers file.
        let seed_peers_file_name = format!("{}.seed_peers.config.toml", node_id);
        seed_peers_config.save_config(&output_dir.join(&seed_peers_file_name));
        // Save network peers file.
        let network_peers_file_name = format!("{}.network_peers.config.toml", node_id);
        network_peers_config.save_config(&output_dir.join(&network_peers_file_name));
        // Save consensus peers file.
        let consensus_peers_file_name = "consensus_peers.config.toml".to_string();
        consensus_peers_config.save_config(&output_dir.join(&consensus_peers_file_name));
        let role_string = match role {
            RoleType::Validator => "validator".to_string(),
            RoleType::FullNode => "full_node".to_string(),
        };
        let base_config = BaseConfig::new(
            output_dir.to_path_buf(),
            template.base.node_sync_retries,
            template.base.node_sync_channel_buffer_size,
            template.base.node_async_log_chan_size,
        );
        let template_network = template.networks.get(0).unwrap();
        let network_config = NetworkConfig {
            peer_id: node_id.to_string(),
            role: role_string,
            network_keypairs_file: network_keys_file_name.into(),
            network_peers_file: network_peers_file_name.into(),
            seed_peers_file: seed_peers_file_name.into(),
            listen_address: addrs[0].clone(),
            advertised_address: addrs[0].clone(),
            discovery_interval_ms: template_network.discovery_interval_ms,
            connectivity_check_interval_ms: template_network.connectivity_check_interval_ms,
            enable_encryption_and_authentication: template_network
                .enable_encryption_and_authentication,
            is_permissioned: template_network.is_permissioned,
            // Dummy values - will be loaded from corresponding files.
            network_keypairs: NetworkKeyPairs::default(),
            network_peers: template_network.network_peers.clone(),
            seed_peers: template_network.seed_peers.clone(),
        };
        let consensus_config = ConsensusConfig {
            max_block_size: template.consensus.max_block_size,
            proposer_type: template.consensus.proposer_type.clone(),
            contiguous_rounds: template.consensus.contiguous_rounds,
            max_pruned_blocks_in_mem: template.consensus.max_pruned_blocks_in_mem,
            pacemaker_initial_timeout_ms: template.consensus.pacemaker_initial_timeout_ms,
            consensus_keypair_file: consensus_keys_file_name.into(),
            consensus_peers_file: consensus_peers_file_name.into(),
            // Dummy values - will be loaded from corresponding files.
            consensus_keypair: ConsensusKeyPair::default(),
            consensus_peers: template.consensus.consensus_peers.clone(),
            consensus_type: template.consensus.consensus_type.clone(),
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
        config.vm_config.publishing_options = VMPublishingOption::Open;
        config
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
    upstream_config_dir: Option<String>,
    is_permissioned: bool,
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
            upstream_config_dir: None,
            is_permissioned: true,
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

    pub fn with_upstream_config_dir(&mut self, upstream_config_dir: Option<String>) -> &mut Self {
        self.upstream_config_dir = upstream_config_dir;
        self
    }

    pub fn with_is_permissioned(&mut self, is_permissioned: bool) -> &mut Self {
        self.is_permissioned = is_permissioned;
        self
    }

    pub fn build(mut self) -> Result<SwarmConfig> {
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
        template.base.data_dir_path = self.output_dir.clone();
        template.admission_control.address = listen_address.clone();
        template.debug_interface.address = listen_address;
        // TODO:
        // [] Use rng instead of seed to prevent duplicate key generation in trusted_peers.rs.
        if self.role == RoleType::Validator {
            SwarmConfig::new_validator_swarm(
                template,
                self.num_nodes,
                faucet_key,
                self.force_discovery,
                self.is_ipv4,
                self.key_seed,
                &self.output_dir,
            )
        } else {
            SwarmConfig::new_full_node_swarm(
                template,
                self.num_nodes,
                self.force_discovery,
                self.is_ipv4,
                self.key_seed,
                &self.output_dir,
                self.is_permissioned,
                PathBuf::from(
                    self.upstream_config_dir
                        .expect("Upstream config path not set"),
                ),
            )
        }
    }
}
