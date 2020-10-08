// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::libra_swarm_utils::{get_json_rpc_libra_interface, load_node_config},
    workspace_builder,
};
use libra_config::config::{Identity, KeyManagerConfig};
use libra_global_constants::CONSENSUS_KEY;
use libra_key_manager::libra_interface::LibraInterface;
use libra_secure_storage::{CryptoStorage, Storage};
use std::{convert::TryInto, process::Command, thread::sleep, time::Duration};

const KEY_MANAGER_BIN: &str = "libra-key-manager";

#[test]
#[ignore]
fn test_key_manager_consensus_rotation() {
    // Create and launch a local validator swarm of 2 nodes.
    let mut env = SmokeTestEnvironment::new(2);
    env.validator_swarm.launch();

    // Create a node config for the key manager by extracting the first node config in the swarm.
    let (node_config, config_path) = load_node_config(&env.validator_swarm, 0);
    let mut key_manager_config = KeyManagerConfig::default();
    key_manager_config.json_rpc_endpoint =
        format!("http://127.0.0.1:{}", node_config.rpc.address.port());
    key_manager_config.rotation_period_secs = 10;
    key_manager_config.sleep_period_secs = 1000; // Large sleep period to force a single rotation

    // Load validator's on disk storage and update key manager secure backend in config
    let storage: Storage = if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        let storage_backend = storage_identity.backend.clone();
        key_manager_config.secure_backend = storage_backend.clone();
        (&storage_backend).try_into().unwrap()
    } else {
        panic!("Couldn't load identity from storage");
    };

    // Save the key manager config to disk
    let key_manager_config_path = config_path.with_file_name("key_manager.yaml");
    key_manager_config.save(&key_manager_config_path).unwrap();

    // Create a json-rpc connection to the blockchain and verify storage matches the on-chain state.
    let libra_interface = get_json_rpc_libra_interface(&env.validator_swarm, 0);
    let account = node_config.validator_network.unwrap().peer_id();
    let current_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&current_consensus, validator_info.consensus_public_key());

    // Spawn the key manager and sleep until a rotation occurs.
    let mut command = Command::new(workspace_builder::get_bin(KEY_MANAGER_BIN));
    command
        .current_dir(workspace_builder::workspace_root())
        .arg(key_manager_config_path);

    let mut key_manager = command.spawn().unwrap();
    sleep(Duration::from_secs(20));

    // Verify the consensus key has been rotated in secure storage and on-chain.
    let rotated_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&rotated_consensus, validator_info.consensus_public_key());
    assert_ne!(current_consensus, rotated_consensus);

    // Kill the key manager process
    key_manager.kill().unwrap();
}
