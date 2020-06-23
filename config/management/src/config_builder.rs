// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, layout::Layout, storage_helper::StorageHelper};
use config_builder::BuildSwarm;
use libra_config::config::{
    Identity, NodeConfig, OnDiskStorageConfig, SafetyRulesService, SecureBackend, SeedPeersConfig,
    WaypointConfig, HANDSHAKE_VERSION,
};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_temppath::TempPath;
use libra_types::account_address;
use std::path::{Path, PathBuf};

const SHARED: &str = "_shared";
const ASSOCIATION: &str = "association";
const ASSOCIATION_SHARED: &str = "association_shared";

pub struct ValidatorBuilder<T: AsRef<Path>> {
    storage_helper: StorageHelper,
    num_validators: usize,
    template: NodeConfig,
    swarm_path: T,
}

impl<T: AsRef<Path>> ValidatorBuilder<T> {
    pub fn new(num_validators: usize, template: NodeConfig, swarm_path: T) -> Self {
        Self {
            storage_helper: StorageHelper::new(),
            num_validators,
            template,
            swarm_path,
        }
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
        layout.association = vec![ASSOCIATION_SHARED.into()];
        layout.operators = (0..self.num_validators)
            .map(|v| (v.to_string() + SHARED))
            .collect();

        let mut common_storage = self
            .storage_helper
            .storage(crate::constants::COMMON_NS.into());
        let layout_value = Value::String(layout.to_toml().unwrap());
        common_storage
            .set(crate::constants::LAYOUT, layout_value)
            .unwrap();
    }

    /// Association initializes its account and key.
    fn create_association(&self) {
        self.storage_helper.initialize(ASSOCIATION.into());
        self.storage_helper
            .association_key(ASSOCIATION, ASSOCIATION_SHARED)
            .unwrap();
    }

    /// Operators upload their keys to shared storage
    /// Operators initialize their validator endpoint and full node endpoint
    /// Operators upload their validator_config to shared storage
    fn initialize_validator_config(&self, index: usize) -> NodeConfig {
        let ns = index.to_string();
        let ns_shared = ns.clone() + SHARED;
        self.storage_helper.initialize(ns.clone());

        let operator_key = self.storage_helper.operator_key(&ns, &ns_shared).unwrap();

        let validator_account = account_address::from_public_key(&operator_key);
        let mut config = self.template.clone_for_template();
        config.randomize_ports();

        let validator_network = config.validator_network.as_mut().unwrap();
        let validator_network_address = validator_network.listen_address.clone();
        validator_network.identity = Identity::from_storage(
            libra_global_constants::VALIDATOR_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            self.secure_backend(&ns, "validator"),
        );

        let fullnode_network = &mut config.full_node_networks[0];
        let fullnode_network_address = fullnode_network.listen_address.clone();
        fullnode_network.identity = Identity::from_storage(
            libra_global_constants::FULLNODE_NETWORK_KEY.into(),
            libra_global_constants::OPERATOR_ACCOUNT.into(),
            self.secure_backend(&ns, "full_node"),
        );

        self.storage_helper
            .validator_config(
                validator_account,
                validator_network_address,
                fullnode_network_address,
                &ns,
                &ns_shared,
            )
            .unwrap();
        config
    }

    /// Operators generate genesis from shared storage and verify against waypoint.
    /// Insert the genesis/waypoint into local config.
    fn finish_validator_config(&self, index: usize, config: &mut NodeConfig) {
        let ns = index.to_string();
        let genesis_path = TempPath::new();
        genesis_path.create_as_file().unwrap();
        let genesis = self.storage_helper.genesis(genesis_path.path()).unwrap();
        self.storage_helper
            .insert_waypoint(&ns, constants::COMMON_NS)
            .unwrap();
        let output = self
            .storage_helper
            .verify_genesis(&ns, genesis_path.path())
            .unwrap();
        assert_eq!(output.split("match").count(), 5);

        config.consensus.safety_rules.service = SafetyRulesService::Thread;
        config.consensus.safety_rules.backend = self.secure_backend(&ns, "safety-rules");
        config.execution.backend = self.secure_backend(&ns, "execution");

        let backend = self.secure_backend(&ns, "waypoint");
        config.base.waypoint = WaypointConfig::FromStorage(backend);
        config.execution.genesis = Some(genesis);
        config.execution.genesis_file_location = PathBuf::from("");
    }
}

impl<T: AsRef<Path>> BuildSwarm for ValidatorBuilder<T> {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        self.create_layout();
        self.create_association();
        let faucet_key = self
            .storage_helper
            .storage(ASSOCIATION.into())
            .export_private_key(libra_global_constants::ASSOCIATION_KEY)
            .unwrap();
        let mut configs: Vec<_> = (0..self.num_validators)
            .map(|i| self.initialize_validator_config(i))
            .collect();
        self.storage_helper
            .create_waypoint(constants::COMMON_NS)
            .unwrap();
        for (i, config) in configs.iter_mut().enumerate() {
            self.finish_validator_config(i, config);
        }
        Ok((configs, faucet_key))
    }
}

pub struct ValidatorFullnodeBuilder {
    validator_config_path: Vec<PathBuf>,
    faucet_key_path: PathBuf,
    template: NodeConfig,
}

impl ValidatorFullnodeBuilder {
    pub fn new(
        validator_config_path: Vec<PathBuf>,
        faucet_key_path: PathBuf,
        template: NodeConfig,
    ) -> Self {
        Self {
            validator_config_path,
            faucet_key_path,
            template,
        }
    }

    fn attach_validator_full_node(&self, validator_config: &mut NodeConfig) -> NodeConfig {
        // Create two vfns, we'll pass one to the validator later
        let mut full_node_config = self.template.clone_for_template();
        full_node_config.randomize_ports();

        // The FN's first network is the external, public network, it needs to swap listen addresses
        // with the validator's VFN and to copy it's key access:
        let pfn = &mut full_node_config.full_node_networks[0];
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
        let mut seed_peers = SeedPeersConfig::default();
        seed_peers.insert(v_vfn_id, vec![v_vfn_network_address]);

        let fn_vfn = &mut full_node_config.full_node_networks[1];
        fn_vfn.seed_peers = seed_peers;

        full_node_config.base.waypoint = validator_config.base.waypoint.clone();
        full_node_config.execution.genesis = validator_config.execution.genesis.clone();
        full_node_config.execution.genesis_file_location = PathBuf::from("");
        full_node_config
    }
}

impl BuildSwarm for ValidatorFullnodeBuilder {
    fn build_swarm(&self) -> anyhow::Result<(Vec<NodeConfig>, Ed25519PrivateKey)> {
        let mut configs = vec![];
        for path in &self.validator_config_path {
            let mut validator_config = NodeConfig::load(path)?;
            let fullnode_config = self.attach_validator_full_node(&mut validator_config);
            validator_config.save(path)?;
            configs.push(fullnode_config);
        }
        let faucet_key = generate_key::load_key(&self.faucet_key_path);
        Ok((configs, faucet_key))
    }
}
