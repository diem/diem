// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    scripts_and_modules::{compile_program, enable_open_publishing},
    smoke_test_environment::new_local_swarm,
    test_utils::{
        assert_balance, create_and_fund_account, diem_swarm_utils::insert_waypoint, transfer_coins,
    },
};
use diem_types::{
    account_address::AccountAddress,
    epoch_change::EpochChangeProof,
    transaction::{Script, TransactionArgument, TransactionPayload},
    waypoint::Waypoint,
};
use forge::{NodeExt, Swarm, SwarmExt};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[test]
fn test_basic_state_synchronization() {
    // - Start a swarm of 4 nodes (3 nodes forming a QC).
    // - Kill one node and continue submitting transactions to the others.
    // - Restart the node
    // - Wait for all the nodes to catch up
    // - Verify that the restarted node has synced up with the submitted transactions.

    // we set a smaller chunk limit (=5) here to properly test multi-chunk state sync
    let mut swarm = new_local_swarm(4);
    for validator in swarm.validators_mut() {
        let mut config = validator.config().clone();
        config.state_sync.chunk_limit = 5;
        config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
    }
    swarm.launch().unwrap(); // Make sure all nodes are healthy and live
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    let client_1 = swarm
        .validator(validator_peer_ids[1])
        .unwrap()
        .json_rpc_client();
    let transaction_factory = swarm.chain_info().transaction_factory();

    let mut account_0 = create_and_fund_account(&mut swarm, 100);
    let account_1 = create_and_fund_account(&mut swarm, 10);
    transfer_coins(
        &client_1,
        &transaction_factory,
        &mut account_0,
        &account_1,
        10,
    );
    assert_balance(&client_1, &account_0, 90);
    assert_balance(&client_1, &account_1, 20);

    // Stop a node
    let node_to_restart = validator_peer_ids[0];
    swarm.validator_mut(node_to_restart).unwrap().stop();

    // Do a transfer and ensure it still executes
    transfer_coins(
        &client_1,
        &transaction_factory,
        &mut account_0,
        &account_1,
        1,
    );
    assert_balance(&client_1, &account_0, 89);
    assert_balance(&client_1, &account_1, 21);

    // Restart killed node and wait for all nodes to catchup
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .start()
        .unwrap();
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    // Connect to the newly recovered node and verify its state
    let client_0 = swarm.validator(node_to_restart).unwrap().json_rpc_client();
    assert_balance(&client_0, &account_0, 89);
    assert_balance(&client_0, &account_1, 21);

    // Test multiple chunk sync
    swarm.validator_mut(node_to_restart).unwrap().stop();

    for _ in 0..10 {
        transfer_coins(
            &client_1,
            &transaction_factory,
            &mut account_0,
            &account_1,
            1,
        );
    }

    assert_balance(&client_1, &account_0, 79);
    assert_balance(&client_1, &account_1, 31);

    // Restart killed node and wait for all nodes to catchup
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .start()
        .unwrap();
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();

    assert_balance(&client_0, &account_0, 79);
    assert_balance(&client_0, &account_1, 31);
}

#[test]
fn test_startup_sync_state() {
    let mut swarm = new_local_swarm(4);
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();
    let client_1 = swarm
        .validator(validator_peer_ids[1])
        .unwrap()
        .json_rpc_client();
    let transaction_factory = swarm.chain_info().transaction_factory();

    let mut account_0 = create_and_fund_account(&mut swarm, 100);
    let account_1 = create_and_fund_account(&mut swarm, 10);
    let txn = transfer_coins(
        &client_1,
        &transaction_factory,
        &mut account_0,
        &account_1,
        10,
    );
    assert_balance(&client_1, &account_0, 90);
    assert_balance(&client_1, &account_1, 20);

    // Stop a node
    let node_to_restart = validator_peer_ids[0];
    let node_config = swarm.validator(node_to_restart).unwrap().config().clone();
    swarm.validator_mut(node_to_restart).unwrap().stop();
    // TODO Remove hardcoded path to state db
    let state_db_path = node_config.storage.dir().join("diemdb");
    // Verify that state_db_path exists and
    // we are not deleting a non-existent directory
    assert!(state_db_path.as_path().exists());
    // Delete the state db to simulate state db lagging
    // behind consensus db and forcing a state sync
    // during a node startup
    fs::remove_dir_all(state_db_path).unwrap();
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .start()
        .unwrap();
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();

    let client_0 = swarm.validator(node_to_restart).unwrap().json_rpc_client();
    // Wait for the txn to by synced to the restarted node
    client_0
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();
    assert_balance(&client_0, &account_0, 90);
    assert_balance(&client_0, &account_1, 20);

    let txn = transfer_coins(
        &client_1,
        &transaction_factory,
        &mut account_0,
        &account_1,
        10,
    );
    client_0
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();

    assert_balance(&client_0, &account_0, 80);
    assert_balance(&client_0, &account_1, 30);
}

#[test]
fn test_state_sync_multichunk_epoch() {
    let mut swarm = new_local_swarm(4);
    for validator in swarm.validators_mut() {
        let mut config = validator.config().clone();
        config.state_sync.chunk_limit = 5;
        config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
    }
    swarm.launch().unwrap(); // Make sure all nodes are healthy and live
    let validator_peer_ids = swarm.validators().map(|v| v.peer_id()).collect::<Vec<_>>();

    let client_0 = swarm
        .validator(validator_peer_ids[0])
        .unwrap()
        .json_rpc_client();
    let transaction_factory = swarm.chain_info().transaction_factory();
    enable_open_publishing(
        &client_0,
        &transaction_factory,
        swarm.chain_info().root_account,
    )
    .unwrap();

    let mut account_0 = create_and_fund_account(&mut swarm, 100);
    let account_1 = create_and_fund_account(&mut swarm, 10);
    assert_balance(&client_0, &account_0, 100);
    assert_balance(&client_0, &account_1, 10);

    // we bring this validator back up with waypoint s.t. the waypoint sync spans multiple epochs,
    // and each epoch spanning multiple chunks
    let node_to_restart = validator_peer_ids[3];
    swarm.validator_mut(node_to_restart).unwrap().stop();

    // submit more transactions to make the current epoch (=1) span > 1 chunk (= 5 versions)
    for _ in 0..7 {
        transfer_coins(
            &client_0,
            &transaction_factory,
            &mut account_0,
            &account_1,
            10,
        );
    }

    let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("testsuite/smoke-test/src/dev_modules/test_script.move")
        .canonicalize()
        .unwrap();
    let move_stdlib_dir = move_stdlib::move_stdlib_modules_full_path();
    let diem_core_framework_dir = diem_framework::diem_core_modules_full_path();
    let diem_payment_framework_dir = diem_framework::diem_payment_modules_full_path();
    let dependencies = &[
        move_stdlib_dir.as_str(),
        diem_core_framework_dir.as_str(),
        diem_payment_framework_dir.as_str(),
    ];
    let compiled_script = compile_program(script_path.to_str().unwrap(), dependencies).unwrap();

    let txn = account_0.sign_with_transaction_builder(transaction_factory.payload(
        TransactionPayload::Script(Script::new(
            compiled_script,
            vec![],
            vec![
                TransactionArgument::U64(10),
                TransactionArgument::Address(AccountAddress::from_hex_literal("0x0").unwrap()),
            ],
        )),
    ));
    client_0.submit(&txn).unwrap();
    client_0
        .wait_for_signed_transaction(&txn, None, None)
        .unwrap();

    // Bump epoch by trigger a reconfig for multiple epochs
    for curr_epoch in 2..=3 {
        // bumps epoch from curr_epoch -> curr_epoch + 1
        enable_open_publishing(
            &client_0,
            &transaction_factory,
            swarm.chain_info().root_account,
        )
        .unwrap();

        let epoch_change_proof: EpochChangeProof = bcs::from_bytes(
            client_0
                .get_state_proof(0)
                .unwrap()
                .into_inner()
                .epoch_change_proof
                .inner(),
        )
        .unwrap();

        let ledger_info = epoch_change_proof
            .ledger_info_with_sigs
            .last()
            .unwrap()
            .ledger_info();

        assert_eq!(ledger_info.epoch(), curr_epoch);
    }

    // bring back dead validator with waypoint
    let epoch_change_proof: EpochChangeProof = bcs::from_bytes(
        client_0
            .get_state_proof(0)
            .unwrap()
            .into_inner()
            .epoch_change_proof
            .inner(),
    )
    .unwrap();
    let waypoint_epoch_2 = Waypoint::new_epoch_boundary(
        epoch_change_proof
            .ledger_info_with_sigs
            .last()
            .unwrap()
            .ledger_info(),
    )
    .unwrap();

    let node_config_path = swarm.validator(node_to_restart).unwrap().config_path();
    let mut node_config = swarm.validator(node_to_restart).unwrap().config().clone();
    node_config.execution.genesis = None;
    node_config.execution.genesis_file_location = PathBuf::from("");
    insert_waypoint(&mut node_config, waypoint_epoch_2);
    node_config.save(node_config_path).unwrap();

    // Restart killed node and wait for all nodes to catchup
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .start()
        .unwrap();
    swarm
        .validator_mut(node_to_restart)
        .unwrap()
        .wait_until_healthy(Instant::now() + Duration::from_secs(10))
        .unwrap();
    swarm
        .wait_for_all_nodes_to_catchup(Instant::now() + Duration::from_secs(60))
        .unwrap();
}
