// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_builder::ValidatorBuilder;
use diem_config::config::NodeConfig;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_temppath::TempPath;
use rand::{rngs::StdRng, SeedableRng};

pub fn bars_test_config() -> (NodeConfig, Ed25519PrivateKey) {
    test_config_impl(diem_experimental_framework_releases::current_module_blobs().to_vec())
}

pub fn test_config() -> (NodeConfig, Ed25519PrivateKey) {
    test_config_impl(diem_framework_releases::current_module_blobs().to_vec())
}

fn test_config_impl(module_blobs: Vec<Vec<u8>>) -> (NodeConfig, Ed25519PrivateKey) {
    let path = TempPath::new();
    path.create_as_dir().unwrap();
    let (root_keys, _genesis, _genesis_waypoint, validators) =
        ValidatorBuilder::new(path.path(), module_blobs)
            .template(NodeConfig::default_for_validator())
            .build(StdRng::from_seed([0; 32]))
            .unwrap();
    let mut configs = validators.into_iter().map(|v| v.config).collect::<Vec<_>>();
    let key = root_keys.root_key;

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
