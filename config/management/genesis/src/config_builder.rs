// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{layout::Layout, storage_helper::StorageHelper, swarm_config::BuildSwarm};
use diem_config::{
    config::{
        DiscoveryMethod, Identity, NodeConfig, OnDiskStorageConfig, SafetyRulesService,
        SecureBackend, SeedAddresses, WaypointConfig, HANDSHAKE_VERSION,
    },
    network_id::NetworkId,
};
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_management::constants::{COMMON_NS, LAYOUT};
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_temppath::TempPath;
use diem_types::{chain_id::ChainId, waypoint::Waypoint};
use std::path::{Path, PathBuf};

const DIEM_ROOT_NS: &str = "diem_root";
const DIEM_ROOT_SHARED_NS: &str = "diem_root_shared";
const OPERATOR_NS: &str = "_operator";
const OPERATOR_SHARED_NS: &str = "_operator_shared";
const OWNER_NS: &str = "_owner";
const OWNER_SHARED_NS: &str = "_owner_shared";

pub struct ValidatorBuilder<T: AsRef<Path>> {
    storage_helper: StorageHelper,
    num_validators: usize,
    randomize_first_validator_ports: bool,
    swarm_path: T,
    template: NodeConfig,
}

impl<T: AsRef<Path>> ValidatorBuilder<T> {
    pub fn new(num_validators: usize, template: NodeConfig, swarm_path: T) -> Self {
        Self {
            storage_helper: StorageHelper::new(),
            num_validators,
            randomize_first_validator_ports: true,
            swarm_path,
            template,
        }
    }

    pub fn randomize_first_validator_ports(mut self, value: bool) -> Self {
        self.randomize_first_validator_ports = value;
        self
    }

    fn secure_backend(&self, ns: &str, usage: &str) -> SecureBackend {
        let original = self.storage_helper.path();
        let dst_base = self.swarm_path.as_ref();
        let mut dst = dst_base.to_path_buf();
        dst.push(format!("{}_{}", usage, ns));
        std::fs::copy(original, &dst).unwrap();

        let mut storage_config = OnDiskStorageConfig::default();
        storage_config.path = dst;
        storage_config.set_data_dir(PathBuf::from(""));
        storage_config.namespace = Some(ns.into());
        SecureBackend::OnDiskStorage(storage_config)
    }

    /// Association uploads the validator layout to shared storage.
    fn create_layout(&self) {
        let mut layout = Layout::default();
        layout.diem_root = DIEM_ROOT_SHARED_NS.into();
        layout.treasury_compliance = DIEM_ROOT_SHARED_NS.into();
        layout.owners = (0..self.num_validators)
            .map(|i| (i.to_string() + OWNER_SHARED_NS))
            .collect();
        layout.operators = (0..self.num_validators)
            .map(|i| (i.to_string() + OPERATOR_SHARED_NS))
            .collect();

        let mut common_storage = self.storage_helper.storage(COMMON_NS.into());
        let layout_value = layout.to_toml().unwrap();
        common_storage.set(LAYOUT, layout_value).unwrap();
    }

    /// Root initializes diem root and treasury root keys.
    fn create_root(&self) {
        self.storage_helper
            .initialize_by_idx(DIEM_ROOT_NS.into(), 0);
        self.storage_helper
            .diem_root_key(DIEM_ROOT_NS, DIEM_ROOT_SHARED_NS)
            .unwrap();
        self.storage_helper
            .treasury_compliance_key(DIEM_ROOT_NS, DIEM_ROOT_SHARED_NS)
            .unwrap();
    }

    /// Generate owner key locally and upload to shared storage.
    fn initialize_validator_owner(&self, index: usize) {
        let local_ns = index.to_string() + OWNER_NS;
        let remote_ns = index.to_string() + OWNER_SHARED_NS;

        self.storage_helper
            .initialize_by_idx(local_ns.clone(), 1 + index);
        let _ = self
            .storage_helper
            .owner_key(&local_ns, &remote_ns)
            .unwrap();
    }

    /// Generate operator key locally and upload to shared storage.
    fn initialize_validator_operator(&self, index: usize) {
        let local_ns = index.to_string() + OPERATOR_NS;
        let remote_ns = index.to_string() + OPERATOR_SHARED_NS;

        self.storage_helper
            .initialize_by_idx(local_ns.clone(), self.num_validators + 1 + index);
        let _ = self
            .storage_helper
            .operator_key(&local_ns, &remote_ns)
            .unwrap();
    }

    /// Sets the operator for the owner by uploading a set-operator transaction to shared storage.
    /// Note, we assume that owner i chooses operator i to operate the validator.
    fn set_validator_operator(&self, index: usize) {
        let remote_ns = index.to_string() + OWNER_SHARED_NS;

        let operator_name = index.to_string() + OPERATOR_SHARED_NS;
        let _ = self.storage_helper.set_operator(&operator_name, &remote_ns);
    }

    /// Operators upload their validator_config to shared storage.
    fn initialize_validator_config(&self, index: usize) -> NodeConfig {
        let local_ns = index.to_string() + OPERATOR_NS;
        let remote_ns = index.to_string() + OPERATOR_SHARED_NS;

        let mut config = self.template.clone();
        if index > 0 || self.randomize_first_validator_ports {
            config.randomize_ports();
        }

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        let fullnode_network = &mut config.full_node_networks[0];
        let fullnode_network_address = fullnode_network.listen_address.clone();

        self.storage_helper
            .validator_config(
                &(index.to_string() + OWNER_SHARED_NS),
                validator_network_address,
                fullnode_network_address,
                ChainId::test(),
                &local_ns,
                &remote_ns,
            )
            .unwrap();

        let validator_identity = validator_network.identity_from_storage();
        validator_network.identity = Identity::from_storage(
            validator_identity.key_name,
            validator_identity.peer_id_name,
            self.secure_backend(&local_ns, "validator"),
        );
        validator_network.network_address_key_backend =
            Some(self.secure_backend(&local_ns, "validator"));

        let fullnode_identity = fullnode_network.identity_from_storage();
        fullnode_network.identity = Identity::from_storage(
            fullnode_identity.key_name,
            fullnode_identity.peer_id_name,
            self.secure_backend(&local_ns, "full_node"),
        );

        config
    }

    /// Operators generate genesis from shared storage and verify against waypoint.
    /// Insert the genesis/waypoint into local config.
    fn finish_validator_config(&self, index: usize, config: &mut NodeConfig, waypoint: Waypoint) {
        let local_ns = index.to_string() + OPERATOR_NS;

        let genesis_path = TempPath::new();
        genesis_path.create_as_file().unwrap();
        let genesis = self
            .storage_helper
            .genesis(ChainId::test(), genesis_path.path())
            .unwrap();

        self.storage_helper
            .insert_waypoint(&local_ns, waypoint)
            .unwrap();

        let output = self
            .storage_helper
            .verify_genesis(&local_ns, genesis_path.path())
            .unwrap();
        assert_eq!(output.split("match").count(), 5, "Failed to verify genesis");

        config.consensus.safety_rules.service = SafetyRulesService::Thread;
        config.consensus.safety_rules.backend = self.secure_backend(&local_ns, "safety-rules");
        config.execution.backend = self.secure_backend(&local_ns, "execution");

        let backend = self.secure_backend(&local_ns, "safety-rules");
        config.base.waypoint = WaypointConfig::FromStorage(backend);
        config.execution.genesis = Some(genesis);
        config.execution.genesis_file_location = PathBuf::from("");
    }
}

impl<T: AsRef<Path>> BuildSwarm for ValidatorBuilder<T> {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        self.create_layout();
        self.create_root();
        let diem_root_key = self
            .storage_helper
            .storage(DIEM_ROOT_NS.into())
            .export_private_key(diem_global_constants::DIEM_ROOT_KEY)
            .unwrap();

        // Upload both owner and operator keys to shared storage
        for index in 0..self.num_validators {
            self.initialize_validator_owner(index);
            self.initialize_validator_operator(index);
        }

        // Set the operator for each owner and the validator config for each operator
        let mut configs = vec![];
        for index in 0..self.num_validators {
            let _ = self.set_validator_operator(index);
            let config = self.initialize_validator_config(index);
            configs.push(config);
        }

        let waypoint = self
            .storage_helper
            .create_waypoint(ChainId::test())
            .unwrap();
        // Create genesis and waypoint
        for (i, config) in configs.iter_mut().enumerate() {
            self.finish_validator_config(i, config, waypoint);
        }

        Ok((configs, diem_root_key))
    }
}

#[derive(Debug)]
pub enum FullnodeType {
    ValidatorFullnode,
    PublicFullnode(usize),
}

pub struct FullnodeBuilder {
    validator_config_path: Vec<PathBuf>,
    diem_root_key_path: PathBuf,
    template: NodeConfig,
    build_type: FullnodeType,
}

impl FullnodeBuilder {
    pub fn new(
        validator_config_path: Vec<PathBuf>,
        diem_root_key_path: PathBuf,
        template: NodeConfig,
        build_type: FullnodeType,
    ) -> Self {
        Self {
            validator_config_path,
            diem_root_key_path,
            template,
            build_type,
        }
    }

    fn attach_validator_full_node(&self, validator_config: &mut NodeConfig) -> NodeConfig {
        // Create two vfns, we'll pass one to the validator later
        let mut full_node_config = self.template.clone();
        full_node_config.randomize_ports();

        // The FN's external, public network needs to swap listen addresses
        // with the validator's VFN and to copy it's key access:
        let pfn = &mut full_node_config
            .full_node_networks
            .iter_mut()
            .find(|n| {
                n.network_id == NetworkId::Public && n.discovery_method != DiscoveryMethod::Onchain
            })
            .expect("vfn missing external public network in config");
        let v_vfn = &mut validator_config.full_node_networks[0];
        pfn.identity = v_vfn.identity.clone();
        let temp_listen = v_vfn.listen_address.clone();
        v_vfn.listen_address = pfn.listen_address.clone();
        pfn.listen_address = temp_listen;

        // Now let's prepare the full nodes internal network to communicate with the validators
        // internal network

        let v_vfn_network_address = v_vfn.listen_address.clone();
        let v_vfn_pub_key = v_vfn.identity_key().public_key();
        let v_vfn_network_address =
            v_vfn_network_address.append_prod_protos(v_vfn_pub_key, HANDSHAKE_VERSION);
        let v_vfn_id = v_vfn.peer_id();
        let mut seed_addrs = SeedAddresses::default();
        seed_addrs.insert(v_vfn_id, vec![v_vfn_network_address]);

        let fn_vfn = &mut full_node_config
            .full_node_networks
            .iter_mut()
            .find(|n| matches!(n.network_id, NetworkId::Private(_)))
            .expect("vfn missing vfn full node network in config");
        fn_vfn.seed_addrs = seed_addrs;

        Self::insert_waypoint_and_genesis(&mut full_node_config, &validator_config);
        full_node_config
    }

    fn insert_waypoint_and_genesis(config: &mut NodeConfig, upstream: &NodeConfig) {
        config.base.waypoint = upstream.base.waypoint.clone();
        config.execution.genesis = upstream.execution.genesis.clone();
        config.execution.genesis_file_location = PathBuf::from("");
    }

    fn build_vfn(&self) -> anyhow::Result<Vec<NodeConfig>> {
        let mut configs = vec![];
        for path in &self.validator_config_path {
            let mut validator_config = NodeConfig::load(path)?;
            let fullnode_config = self.attach_validator_full_node(&mut validator_config);
            validator_config.save(path)?;
            configs.push(fullnode_config);
        }
        Ok(configs)
    }

    fn build_public_fn(&self, num_nodes: usize) -> anyhow::Result<Vec<NodeConfig>> {
        let mut configs = vec![];
        let validator_config = NodeConfig::load(
            self.validator_config_path
                .first()
                .ok_or_else(|| anyhow::format_err!("No validator config path"))?,
        )?;
        for _ in 0..num_nodes {
            let mut fullnode_config = self.template.clone();
            fullnode_config.randomize_ports();
            Self::insert_waypoint_and_genesis(&mut fullnode_config, &validator_config);
            configs.push(fullnode_config);
        }
        Ok(configs)
    }
}

impl BuildSwarm for FullnodeBuilder {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        let configs = match self.build_type {
            FullnodeType::ValidatorFullnode => self.build_vfn(),
            FullnodeType::PublicFullnode(num_nodes) => self.build_public_fn(num_nodes),
        }?;
        let diem_root_key_path = generate_key::load_key(&self.diem_root_key_path);
        Ok((configs, diem_root_key_path))
    }
}

pub fn test_config() -> (NodeConfig, Ed25519PrivateKey) {
    let path = TempPath::new();
    path.create_as_dir().unwrap();
    let builder = ValidatorBuilder::new(1, NodeConfig::default_for_validator(), path.path());
    let (mut configs, key) = builder.build_swarm().unwrap();

    let mut config = configs.swap_remove(0);
    config.set_data_dir(path.path().to_path_buf());
    let backend = &config
        .validator_network
        .as_ref()
        .unwrap()
        .identity_from_storage()
        .backend;
    let storage: Storage = std::convert::TryFrom::try_from(backend).unwrap();
    let mut test = diem_config::config::TestConfig::new_with_temp_dir(Some(path));
    test.execution_key(
        storage
            .export_private_key(diem_global_constants::EXECUTION_KEY)
            .unwrap(),
    );
    test.operator_key(
        storage
            .export_private_key(diem_global_constants::OPERATOR_KEY)
            .unwrap(),
    );
    test.owner_key(
        storage
            .export_private_key(diem_global_constants::OWNER_KEY)
            .unwrap(),
    );
    config.test = Some(test);

    let owner_account = storage
        .get(diem_global_constants::OWNER_ACCOUNT)
        .unwrap()
        .value;
    let mut sr_test = diem_config::config::SafetyRulesTestConfig::new(owner_account);
    sr_test.consensus_key(
        storage
            .export_private_key(diem_global_constants::CONSENSUS_KEY)
            .unwrap(),
    );
    sr_test.execution_key(
        storage
            .export_private_key(diem_global_constants::EXECUTION_KEY)
            .unwrap(),
    );
    config.consensus.safety_rules.test = Some(sr_test);

    (config, key)
}
