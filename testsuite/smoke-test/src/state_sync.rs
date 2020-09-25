// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{compare_balances, setup_swarm_and_client_proxy},
};
use libra_config::config::NodeConfig;
use std::fs;

#[test]
fn test_basic_state_synchronization() {
    // - Start a swarm of 5 nodes (3 nodes forming a QC).
    // - Kill one node and continue submitting transactions to the others.
    // - Restart the node
    // - Wait for all the nodes to catch up
    // - Verify that the restarted node has synced up with the submitted transactions.

    // we set a smaller chunk limit (=5) here to properly test multi-chunk state sync
    let mut env = SmokeTestEnvironment::new_with_chunk_limit(5, 5);
    env.validator_swarm.launch();
    let mut client_proxy = env.get_validator_client(1, None);

    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test single chunk sync, chunk_size = 5
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(89.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test multiple chunk sync
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(89.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    for _ in 0..10 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(79.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(31.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap(),
    ));
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let node_config = NodeConfig::load(
        env.validator_swarm
            .config
            .config_files
            .get(peer_to_stop)
            .unwrap(),
    )
    .unwrap();
    // TODO Remove hardcoded path to state db
    let state_db_path = node_config.storage.dir().join("libradb");
    // Verify that state_db_path exists and
    // we are not deleting a non-existent directory
    assert!(state_db_path.as_path().exists());
    // Delete the state db to simulate state db lagging
    // behind consensus db and forcing a state sync
    // during a node startup
    fs::remove_dir_all(state_db_path).unwrap();
    assert!(env.validator_swarm.add_node(peer_to_stop).is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_client(0, None);
    let sender_address = accounts[0].address;
    client_proxy_0.set_accounts(accounts);
    client_proxy_0
        .wait_for_transaction(sender_address, 1)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state_with_empty_consensus_db() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap(),
    ));
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let node_config = NodeConfig::load(
        env.validator_swarm
            .config
            .config_files
            .get(peer_to_stop)
            .unwrap(),
    )
    .unwrap();
    let consensus_db_path = node_config.storage.dir().join("consensusdb");
    // Verify that consensus db exists and
    // we are not deleting a non-existent directory
    assert!(consensus_db_path.as_path().exists());
    // Delete the consensus db to simulate consensus db is nuked
    fs::remove_dir_all(consensus_db_path).unwrap();
    assert!(env.validator_swarm.add_node(peer_to_stop).is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_client(0, None);
    let sender_address = accounts[0].address;
    client_proxy_0.set_accounts(accounts);
    client_proxy_0
        .wait_for_transaction(sender_address, 1)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
}
