// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::libra_swarm_utils::load_backend_storage,
};
use libra_config::config::NodeConfig;
use libra_global_constants::CONSENSUS_KEY;
use libra_key_manager::{
    libra_interface::{JsonRpcLibraInterface, LibraInterface},
    KeyManager,
};
use libra_secure_storage::{CryptoStorage, Storage};
use libra_secure_time::RealTimeService;
use libra_smoke_test_attribute::smoke_test;
use libra_types::chain_id::ChainId;
use std::{convert::TryInto, thread, thread::sleep, time::Duration};

#[smoke_test]
fn test_key_manager_consensus_rotation() {
    // Create and launch a local validator swarm
    let mut env = SmokeTestEnvironment::new(1);
    env.validator_swarm.launch();

    // Fetch the first node config in the swarm
    let node_config_path = env.validator_swarm.config.config_files.get(0).unwrap();
    let node_config = NodeConfig::load(&node_config_path).unwrap();

    // Load validator's on disk storage
    let secure_backend = load_backend_storage(&env.validator_swarm, 0);
    let storage: Storage = (&secure_backend).try_into().unwrap();

    // Create a json-rpc connection to the blockchain and verify storage matches the on-chain state.
    let json_rpc_endpoint = format!(
        "http://127.0.0.1:{}",
        env.validator_swarm.get_client_port(0)
    );
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_endpoint.clone());
    let account = node_config.validator_network.unwrap().peer_id();
    let current_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&current_consensus, validator_info.consensus_public_key());

    // Create the key manager
    let key_manager_storage: Storage = (&secure_backend).try_into().unwrap();
    let mut key_manager = KeyManager::new(
        JsonRpcLibraInterface::new(json_rpc_endpoint),
        key_manager_storage,
        RealTimeService::new(),
        1,
        1000, // Large sleep period to force a single rotation
        1000,
        ChainId::test(),
    );

    // Spawn the key manager and execute a rotation
    let _key_manager_thread = thread::spawn(move || key_manager.execute());

    // Verify the consensus key has been rotated in secure storage and on-chain.
    for _ in 0..10 {
        sleep(Duration::from_secs(6));

        let rotated_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
        let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
        if current_consensus != rotated_consensus
            && validator_info.consensus_public_key() == &rotated_consensus
        {
            return; // The consensus key was successfully rotated
        }
    }

    panic!("The key manager failed to rotate the consensus key!");
}
