// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_environment::TestEnvironment, workspace_builder};
use libra_config::config::{Identity, KeyManagerConfig, NodeConfig};
use libra_global_constants::CONSENSUS_KEY;
use libra_key_manager::libra_interface::{JsonRpcLibraInterface, LibraInterface};
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use std::{
    convert::TryInto,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

const KEY_MANAGER_BIN: &str = "libra-key-manager";

#[test]
#[ignore]
fn test_key_manager_consensus_rotation() {
    // Create and launch a local validator swarm of 2 nodes.
    let mut env = TestEnvironment::new(2);
    env.validator_swarm.launch();

    // Create a node config for the key manager by extracting the first node config in the swarm.
    // TODO(joshlind): see if we can refactor TestEnvironment to clean this up.
    let node_config_path = env.validator_swarm.config.config_files.get(0).unwrap();
    let node_config = NodeConfig::load(&node_config_path).unwrap();
    let json_rpc_endpoint = format!("http://127.0.0.1:{}", node_config.rpc.address.port());

    let mut key_manager_config = KeyManagerConfig::default();
    key_manager_config.json_rpc_endpoint = json_rpc_endpoint.clone();
    key_manager_config.rotation_period_secs = 10;
    key_manager_config.sleep_period_secs = 10;

    // Load validator's on disk storage and update key manager secure backend in config
    let mut storage: Storage = if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        let storage_backend = storage_identity.backend.clone();
        key_manager_config.secure_backend = storage_backend.clone();
        (&storage_backend).try_into().unwrap()
    } else {
        panic!("Couldn't load identity from storage");
    };

    // Save the key manager config to disk
    let key_manager_config_path = node_config_path.with_file_name("key_manager.yaml");
    key_manager_config.save(&key_manager_config_path).unwrap();

    // Create a json-rpc connection to the blockchain and verify storage matches the on-chain state.
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_endpoint);
    let account = node_config.validator_network.unwrap().peer_id();
    let current_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&current_consensus, validator_info.consensus_public_key());

    // Spawn the key manager and sleep until a rotation occurs.
    // TODO(joshlind): support a dedicated key manager log (instead of just printing on failure).
    let mut command = Command::new(workspace_builder::get_bin(KEY_MANAGER_BIN));
    command
        .current_dir(workspace_builder::workspace_root())
        .arg(key_manager_config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let key_manager = command.spawn().unwrap();
    sleep(Duration::from_secs(20));

    // Submit a reconfiguration so that the key rotation will be performed on-chain
    // let libra = get_libra_interface(&node_config);
    // let time_service = RealTimeService::new();
    // let libra_root_key = env.libra_root_key.0;
    // submit_new_reconfig(&libra, &time_service, &libra_root_key).unwrap();
    // sleep(Duration::from_secs(2));

    // Verify the consensus key has been rotated in secure storage and on-chain.
    let rotated_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&rotated_consensus, validator_info.consensus_public_key());
    assert_ne!(current_consensus, rotated_consensus);

    // Cause a failure (i.e., wipe storage) and verify the key manager exits with an error status.
    storage.reset_and_clear().unwrap();
    let output = key_manager.wait_with_output().unwrap();
    if output.status.success() {
        panic!(
            "Key manager did not return an error as expected! Printing key manager output: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
