// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{
        compare_balances,
        diem_swarm_utils::{insert_waypoint, load_node_config, save_node_config},
        setup_swarm_and_client_proxy,
    },
    workspace_builder,
};
use diem_config::config::NodeConfig;
use diem_crypto::HashValue;
use diem_types::waypoint::Waypoint;
use std::{fs, path::PathBuf};

#[test]
fn test_basic_state_synchronization() {
    // - Start a swarm of 4 nodes (3 nodes forming a QC).
    // - Kill one node and continue submitting transactions to the others.
    // - Restart the node
    // - Wait for all the nodes to catch up
    // - Verify that the restarted node has synced up with the submitted transactions.

    // we set a smaller chunk limit (=5) here to properly test multi-chunk state sync
    let mut env = SmokeTestEnvironment::new_with_chunk_limit(4, 5);
    env.validator_swarm.launch();
    let mut client_1 = env.get_validator_client(1, None);

    client_1.create_next_account(false).unwrap();
    client_1.create_next_account(false).unwrap();
    client_1
        .mint_coins(&["mb", "0", "100", "XUS"], true)
        .unwrap();
    client_1
        .mint_coins(&["mb", "1", "10", "XUS"], true)
        .unwrap();
    client_1
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_1.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test single chunk sync, chunk_size = 5
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_1.get_balances(&["b", "1"]).unwrap(),
    ));
    client_1
        .transfer_coins(&["tb", "0", "1", "1", "XUS"], true)
        .unwrap();

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_2 = env.get_validator_client(node_to_restart, None);
    client_2.set_accounts(client_1.copy_all_accounts());
    assert!(compare_balances(
        vec![(89.0, "XUS".to_string())],
        client_2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "XUS".to_string())],
        client_2.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test multiple chunk sync
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(89.0, "XUS".to_string())],
        client_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "XUS".to_string())],
        client_1.get_balances(&["b", "1"]).unwrap(),
    ));
    for _ in 0..10 {
        client_1
            .transfer_coins(&["tb", "0", "1", "1", "XUS"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    client_2.set_accounts(client_1.copy_all_accounts());
    assert!(compare_balances(
        vec![(79.0, "XUS".to_string())],
        client_2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(31.0, "XUS".to_string())],
        client_2.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state() {
    let (mut env, mut client_1) = setup_swarm_and_client_proxy(4, 1);
    client_1.create_next_account(false).unwrap();
    client_1.create_next_account(false).unwrap();
    client_1
        .mint_coins(&["mb", "0", "100", "XUS"], true)
        .unwrap();
    client_1
        .mint_coins(&["mb", "1", "10", "XUS"], true)
        .unwrap();
    client_1
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_1.get_balances(&["b", "1"]).unwrap(),
    ));
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let (node_config, _) = load_node_config(&env.validator_swarm, peer_to_stop);
    // TODO Remove hardcoded path to state db
    let state_db_path = node_config.storage.dir().join("diemdb");
    // Verify that state_db_path exists and
    // we are not deleting a non-existent directory
    assert!(state_db_path.as_path().exists());
    // Delete the state db to simulate state db lagging
    // behind consensus db and forcing a state sync
    // during a node startup
    fs::remove_dir_all(state_db_path).unwrap();
    assert!(env.validator_swarm.add_node(peer_to_stop).is_ok());
    // create the client for the restarted node
    let accounts = client_1.copy_all_accounts();
    let mut client_0 = env.get_validator_client(0, None);
    let sender_address = accounts[0].address;
    client_0.set_accounts(accounts);
    client_0.wait_for_transaction(sender_address, 0).unwrap();
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_1
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();
    client_0.wait_for_transaction(sender_address, 1).unwrap();
    assert!(compare_balances(
        vec![(80.0, "XUS".to_string())],
        client_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "XUS".to_string())],
        client_0.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state_with_empty_consensus_db() {
    let (mut env, mut client_1) = setup_swarm_and_client_proxy(4, 1);
    client_1.create_next_account(false).unwrap();
    client_1.create_next_account(false).unwrap();
    client_1
        .mint_coins(&["mb", "0", "100", "XUS"], true)
        .unwrap();
    client_1
        .mint_coins(&["mb", "1", "10", "XUS"], true)
        .unwrap();
    client_1
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_1.get_balances(&["b", "1"]).unwrap(),
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
    let accounts = client_1.copy_all_accounts();
    let mut client_0 = env.get_validator_client(0, None);
    let sender_address = accounts[0].address;
    client_0.set_accounts(accounts);
    client_0.wait_for_transaction(sender_address, 0).unwrap();
    assert!(compare_balances(
        vec![(90.0, "XUS".to_string())],
        client_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "XUS".to_string())],
        client_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_1
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();
    client_0.wait_for_transaction(sender_address, 1).unwrap();
    assert!(compare_balances(
        vec![(80.0, "XUS".to_string())],
        client_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "XUS".to_string())],
        client_0.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_state_sync_multichunk_epoch() {
    let mut env = SmokeTestEnvironment::new_with_chunk_limit(4, 5);
    env.validator_swarm.launch();
    let mut client = env.get_validator_client(0, None);
    // we bring this validator back up with waypoint s.t. the waypoint sync spans multiple epochs,
    // and each epoch spanning multiple chunks
    env.validator_swarm.kill_node(3);
    client.create_next_account(false).unwrap();

    client
        .mint_coins(&["mintb", "0", "10", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));

    // submit more transactions to make the current epoch (=1) span > 1 chunk (= 5 versions)
    for _ in 0..7 {
        client
            .mint_coins(&["mintb", "0", "10", "XUS"], true)
            .unwrap();
    }

    let script_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/test_script.move");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();
    let script_params = &["compile", "0", unwrapped_script_path, unwrapped_stdlib_dir];
    let mut script_compiled_paths = client.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };

    // Initially publishing option was set to CustomScript, this transaction should be executed.
    client
        .execute_script(&["execute", "0", &script_compiled_path[..], "10", "0x0"])
        .unwrap();

    // Bump epoch by trigger a reconfig by modifying allow list for multiple epochs
    for curr_epoch in 1..=2 {
        // bumps epoch from curr_epoch -> curr_epoch + 1
        let hash = hex::encode(&HashValue::random().to_vec());
        client
            .add_to_script_allow_list(&["add_to_script_allow_list", hash.as_str()], true)
            .unwrap();
        assert_eq!(
            client
                .latest_epoch_change_li()
                .unwrap()
                .ledger_info()
                .epoch(),
            curr_epoch
        );
    }

    // bring back dead validator with waypoint
    let waypoint_epoch_2 =
        Waypoint::new_epoch_boundary(client.latest_epoch_change_li().unwrap().ledger_info())
            .unwrap();
    let (mut node_config, _) = load_node_config(&env.validator_swarm, 3);
    node_config.execution.genesis = None;
    node_config.execution.genesis_file_location = PathBuf::from("");
    insert_waypoint(&mut node_config, waypoint_epoch_2);
    save_node_config(&mut node_config, &env.validator_swarm, 3);
    env.validator_swarm.add_node(3).unwrap();

    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());
}
