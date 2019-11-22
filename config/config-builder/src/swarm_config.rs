// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience structs and functions for generating configuration for a swarm of libra nodes
use crate::util::genesis_transaction;
use failure::prelude::*;
use libra_config::{
    config::{
        BaseConfig, ConsensusConfig, NetworkConfig, NodeConfig, OnDiskStorageConfig,
        PersistableConfig, RoleType, SafetyRulesBackend, SafetyRulesConfig, VMPublishingOption,
    },
    keys::{ConsensusKeyPair, NetworkKeyPairs},
    seed_peers::{SeedPeersConfig, SeedPeersConfigHelpers},
    trusted_peers::{
        ConfigHelpers, ConsensusPeersConfig, ConsensusPrivateKey, NetworkPeerInfo,
        NetworkPeersConfig, NetworkPrivateKeys, UpstreamPeersConfig,
    },
    utils,
};
use libra_crypto::{ed25519::*, test_utils::KeyPair};
use libra_logger::prelude::*;
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct SwarmConfig {
    pub configs: Vec<PathBuf>,
}

fn add_peer(
    peer: &NetworkConfig,
    network_peers: &mut NetworkPeersConfig,
    seed_peers: &mut SeedPeersConfig,
) {
    seed_peers
        .seed_peers
        .insert(peer.peer_id, vec![peer.listen_address.clone()]);
    network_peers.peers.insert(
        peer.peer_id,
        NetworkPeerInfo {
            network_identity_pubkey: peer.network_keypairs.get_network_identity_public().clone(),
            network_signing_pubkey: peer.network_keypairs.get_network_signing_public().clone(),
        },
    );
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
        let seed = key_seed.unwrap_or([0u8; 32]);
        let mut rng = StdRng::from_seed(seed);

        let mut upstream_network = if let Some(network) = template.full_node_networks.get(0) {
            network.clone_for_template()
        } else {
            NetworkConfig::default()
        };
        upstream_network.is_permissioned = is_permissioned;
        upstream_network.random(&mut rng);

        upstream_network.listen_address = utils::get_available_port_in_multiaddr(is_ipv4);
        upstream_network.advertised_address = upstream_network.listen_address.clone();

        let mut network_peers = NetworkPeersConfig {
            peers: HashMap::new(),
        };
        let mut seed_peers = SeedPeersConfig {
            seed_peers: HashMap::new(),
        };
        add_peer(&upstream_network, &mut network_peers, &mut seed_peers);

        let mut upstream_config = NodeConfig::load(&upstream_config_dir.join("node.config.toml"))?;

        template.set_role(RoleType::FullNode)?;
        template.consensus.consensus_peers = upstream_config.consensus.consensus_peers.clone();
        template.execution.genesis = upstream_config.execution.genesis.clone();
        template.state_sync.upstream_peers = UpstreamPeersConfig {
            upstream_peers: vec![upstream_network.peer_id],
        };

        let mut configs = Vec::new();
        let mut config_files = Vec::new();

        // Generate configs for all nodes.
        for index in 0..num_nodes {
            let node_dir = output_dir.join(format!("{}", index));
            std::fs::create_dir_all(&node_dir).expect("unable to create config dir");

            let mut config = NodeConfig::random_with_template(&template, &mut rng);
            config.randomize_ports();
            config.set_data_dir(node_dir.clone())?;
            let network = &mut config.full_node_networks[0];
            network.listen_address = utils::get_available_port_in_multiaddr(is_ipv4);
            network.advertised_address = network.listen_address.clone();
            add_peer(&network, &mut network_peers, &mut seed_peers);
            configs.push(config);
            config_files.push(node_dir.join("node.config.toml"));
        }

        if prune_seed_peers_for_discovery {
            seed_peers.seed_peers = seed_peers.seed_peers.into_iter().take(1).collect();
        }

        for config in &mut configs {
            config.full_node_networks[0].network_peers = network_peers.clone();
            config.full_node_networks[0].seed_peers = seed_peers.clone();
            config.save(&PathBuf::from("node.config.toml"));
        }

        // Modify upstream peer config to add the new network config.
        upstream_network.base = upstream_config.base.clone();
        upstream_network.network_peers = network_peers;
        upstream_network.seed_peers = seed_peers;
        upstream_config.full_node_networks.push(upstream_network);
        upstream_config.save(&PathBuf::from("node.config.toml"));

        Ok(Self {
            configs: config_files,
        })
    }

    pub fn new_validator_swarm(
        mut template: NodeConfig,
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
        let genesis_transaction =
            genesis_transaction(&faucet_key, &consensus_peers_config, &network_peers_config);
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
        template.execution.genesis_file_location = PathBuf::from("genesis.blob");
        // Generate configs for all nodes.
        for (index, (node_id, addrs)) in peer_addresses.iter().enumerate() {
            let node_dir = output_dir.join(format!("{}", index));
            std::fs::create_dir_all(&node_dir).expect("unable to create config dir");
            debug!("Directory for node {}: {:?}", index, node_dir);
            // Save genesis transaction in file.
            let mut genesis_transaction_file =
                File::create(&node_dir.join(&template.execution.genesis_file_location))?;
            genesis_transaction_file.write_all(&lcs::to_bytes(&genesis_transaction)?)?;
            let (
                ConsensusPrivateKey {
                    consensus_private_key,
                },
                NetworkPrivateKeys {
                    network_signing_private_key,
                    network_identity_private_key,
                },
            ) = private_keys.remove_entry(&node_id).unwrap().1;
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
        node_id: &PeerId,
        network_keypairs: &NetworkKeyPairs,
        consenus_keypair: &ConsensusKeyPair,
        seed_peers_config: &SeedPeersConfig,
        network_peers_config: &NetworkPeersConfig,
        consensus_peers_config: &ConsensusPeersConfig,
        output_dir: &Path,
        addrs: &[Multiaddr],
    ) -> NodeConfig {
        let base_config = Arc::new(BaseConfig::new(output_dir.to_path_buf(), role));
        // Save consensus keys if present.
        let mut consensus_keys_file_name = "".to_string();
        if consenus_keypair.is_present() {
            consensus_keys_file_name = format!("{}.node.consensus.keys.toml", node_id.to_string());
            consenus_keypair.save_config(&output_dir.join(&consensus_keys_file_name));
        }
        // Prepare safety rules
        let mut safety_rules_config = SafetyRulesConfig::default();
        if role == RoleType::Validator {
            safety_rules_config.backend = SafetyRulesBackend::OnDiskStorage(OnDiskStorageConfig {
                default: true,
                path: PathBuf::from(format!("{}.node.safety_rules.toml", node_id.to_string())),
                base: base_config.clone(),
            })
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
        let template_network = NetworkConfig::default();
        let network_config = NetworkConfig {
            peer_id: *node_id,
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
            network_peers: network_peers_config.clone(),
            seed_peers: template_network.seed_peers.clone(),
            base: base_config.clone(),
        };
        let consensus_config = ConsensusConfig {
            max_block_size: template.consensus.max_block_size,
            proposer_type: template.consensus.proposer_type,
            contiguous_rounds: template.consensus.contiguous_rounds,
            max_pruned_blocks_in_mem: template.consensus.max_pruned_blocks_in_mem,
            pacemaker_initial_timeout_ms: template.consensus.pacemaker_initial_timeout_ms,
            consensus_keypair_file: consensus_keys_file_name.into(),
            consensus_peers_file: consensus_peers_file_name.into(),
            // Dummy values - will be loaded from corresponding files.
            consensus_keypair: ConsensusKeyPair::default(),
            consensus_peers: consensus_peers_config.clone(),
            safety_rules: safety_rules_config,
            base: base_config.clone(),
        };

        let mut full_node_networks = vec![];
        let mut validator_network = None;
        if role == RoleType::Validator {
            validator_network = Some(network_config);
        } else {
            full_node_networks = vec![network_config];
        }

        let mut config = NodeConfig {
            base: base_config,
            full_node_networks,
            validator_network,
            consensus: consensus_config,
            metrics: template.metrics.clone(),
            execution: template.execution.clone(),
            admission_control: template.admission_control.clone(),
            debug_interface: template.debug_interface.clone(),
            storage: template.storage.clone(),
            mempool: template.mempool.clone(),
            state_sync: template.state_sync.clone(),
            logger: template.logger.clone(),
            test: None,
            vm_config: template.vm_config.clone(),
        };
        config.randomize_ports();
        config.vm_config.publishing_options = VMPublishingOption::Open;
        config
            .set_data_dir(output_dir.to_path_buf())
            .expect("Unable to set output directory");
        config
    }
}

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
    upstream_config_dir: Option<String>,
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
        let mut template = if let Some(template_path) = self.template_path {
            NodeConfig::load_config(template_path)
        } else {
            NodeConfig::default()
        };

        // update everything in the template and then generate swarm config
        let listen_address = if self.is_ipv4 { "0.0.0.0" } else { "::1" };
        let listen_address = listen_address.to_string();
        template.admission_control.address = listen_address.clone();
        template.debug_interface.address = listen_address;
        template.vm_config.publishing_options = VMPublishingOption::Open;

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
