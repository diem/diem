// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_utils::{
    compare_balances,
    libra_swarm_utils::{get_op_tool, load_libra_root_storage},
    setup_swarm_and_client_proxy,
};
use cli::client_proxy::ClientProxy;
use debug_interface::NodeDebugClient;
use libra_trace::trace::trace_node;
use libra_types::{
    account_address::AccountAddress, account_config::testnet_dd_account_address,
    ledger_info::LedgerInfo, waypoint::Waypoint,
};

#[test]
fn smoke_test_multi_node() {
    let (_swarm, client_proxy) = setup_swarm_and_client_proxy(4, 0);
    test_smoke_script(client_proxy);
}

#[test]
fn smoke_test_single_node() {
    let (_swarm, client_proxy) = setup_swarm_and_client_proxy(1, 0);
    test_smoke_script(client_proxy);
}

// Test if we commit not only user transactions but also block metadata transactions,
// assert committed version > # of user transactions
#[test]
fn smoke_test_single_node_block_metadata() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    // just need an address to get the latest version
    let address = AccountAddress::from_hex_literal("0xA550C18").unwrap();
    // this script does 4 transactions
    test_smoke_script(swarm.get_validator_client(0, None));
    let (_account, version) = client_proxy
        .get_latest_account(&["q", &address.to_string()])
        .unwrap();
    assert!(
        version > 4,
        "BlockMetadata txn not produced, current version: {}",
        version
    );
}

#[test]
fn test_basic_fault_tolerance() {
    // A configuration with 4 validators should tolerate single node failure.
    let (mut env, client_proxy) = setup_swarm_and_client_proxy(4, 1);
    // kill the first validator
    env.validator_swarm.kill_node(0);
    // run the script for the smoke test by submitting requests to the second validator
    test_smoke_script(client_proxy);
}

#[test]
fn test_basic_restartability() {
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
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
    let peer_to_restart = 0;
    // restart node
    env.validator_swarm.kill_node(peer_to_restart);
    assert!(env.validator_swarm.add_node(peer_to_restart).is_ok());
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_client_waypoints() {
    let (env, mut client_proxy) = setup_swarm_and_client_proxy(4, 1);
    // Make sure some txns are committed
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();

    // Create the waypoint for the initial epoch
    let genesis_li = client_proxy
        .latest_epoch_change_li()
        .expect("Failed to retrieve genesis LedgerInfo");
    assert_eq!(genesis_li.ledger_info().epoch(), 0);
    let genesis_waypoint = Waypoint::new_epoch_boundary(genesis_li.ledger_info())
        .expect("Failed to generate waypoint from genesis LI");

    // Start another client with the genesis waypoint and make sure it successfully connects
    let mut client_with_waypoint = env.get_validator_client(0, Some(genesis_waypoint));
    client_with_waypoint.test_trusted_connection().unwrap();
    assert_eq!(
        client_with_waypoint.latest_epoch_change_li().unwrap(),
        genesis_li
    );

    // Start next epoch

    // This ugly blob is to remove a validator, we can do better...
    let peer_id = env
        .validator_swarm
        .get_validator(0)
        .unwrap()
        .validator_peer_id()
        .unwrap();
    let op_tool = get_op_tool(&env.validator_swarm, 1);
    let libra_root = load_libra_root_storage(&env.validator_swarm, 0);
    let context = op_tool.remove_validator(peer_id, &libra_root).unwrap();
    client_proxy
        .wait_for_transaction(context.address, context.sequence_number + 1)
        .unwrap();
    // end ugly blob
    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    let epoch_1_li = client_proxy
        .latest_epoch_change_li()
        .expect("Failed to retrieve end of epoch 1 LedgerInfo");

    assert_eq!(epoch_1_li.ledger_info().epoch(), 1);
    let epoch_1_waypoint = Waypoint::new_epoch_boundary(epoch_1_li.ledger_info())
        .expect("Failed to generate waypoint from end of epoch 1");

    // Start a client with the waypoint for end of epoch 1 and make sure it successfully connects
    client_with_waypoint = env.get_validator_client(1, Some(epoch_1_waypoint));
    client_with_waypoint.test_trusted_connection().unwrap();
    assert_eq!(
        client_with_waypoint.latest_epoch_change_li().unwrap(),
        epoch_1_li
    );

    // Verify that a client with the wrong waypoint is not going to be able to connect to the chain.
    let bad_li = LedgerInfo::mock_genesis(None);
    let bad_waypoint = Waypoint::new_epoch_boundary(&bad_li).unwrap();
    let mut client_with_bad_waypoint = env.get_validator_client(1, Some(bad_waypoint));
    assert!(client_with_bad_waypoint.test_trusted_connection().is_err());
}

#[test]
fn test_concurrent_transfers_single_node() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "10", "Coin1"], true)
        .unwrap();
    for _ in 0..20 {
        client_proxy
            .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
            .unwrap();
    }
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(79.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(31.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_trace() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    let port = swarm.validator_swarm.get_validators_debug_ports()[0];
    let mut debug_client = NodeDebugClient::new("localhost", port);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
        .unwrap();
    let events = debug_client.get_events().expect("Failed to get events");
    let txn_node = format!("txn::{}::{}", testnet_dd_account_address(), 1);
    println!("Tracing {}", txn_node);
    trace_node(&events[..], &txn_node);
}

/// This helper function creates 3 new accounts, mints funds, transfers funds
/// between the accounts and verifies that these operations succeed.
fn test_smoke_script(mut client: ClientProxy) {
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));

    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "1", "1", "Coin1"], true)
        .unwrap();
    client
        .transfer_coins(&["tb", "0", "1", "3", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(7.0, "Coin1".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(4.0, "Coin1".to_string())],
        client.get_balances(&["b", "1"]).unwrap(),
    ));

    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "2", "15", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(15.0, "Coin1".to_string())],
        client.get_balances(&["b", "2"]).unwrap(),
    ));
}
