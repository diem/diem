// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use diem_framework_releases::current_modules;
use diem_transaction_builder::stdlib;
use diem_types::{
    account_config,
    transaction::TransactionOutput,
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{
    current_function_name, executor::FakeExecutor, test_with_different_versions,
};

// The CRSN size used throughout the tests
const K: u64 = 10;

fn assert_aborted_with(output: TransactionOutput, error_code: u64) {
    assert!(matches!(
        output.status().status(),
        Ok(KeptVMStatus::MoveAbort(_, code)) if code == error_code
    ));
}

#[test]
fn crsn_versioning_publish() {
    test_with_different_versions! {&[2], |test_env| {
        let mut executor = test_env.executor;
        // set the diem version back
        let sender = executor.create_raw_account_data(1_000_010, 0);
        executor.add_account_data(&sender);
        for module in current_modules() {
            let mut blob = Vec::new();
            module.serialize(&mut blob).unwrap();
            executor.add_module(&module.self_id(), blob);
        }

        // This shouldn't apply
        let txn = sender
            .account()
            .transaction()
            .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
            .sequence_number(0)
            .sign();
        assert_aborted_with(executor.execute_transaction(txn), 769);
    }}
}

#[test]
fn can_opt_in_to_crsn() {
    let mut executor = FakeExecutor::from_genesis_file();
    // set the diem version back
    let sender = executor.create_raw_account_data(1_000_010, 0);
    executor.add_account_data(&sender);
    executor.set_golden_file(current_function_name!());

    // This should apply
    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);

    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
            account_config::xus_tag(),
            *sender.address(),
            100,
            vec![],
            vec![],
        ))
        .sequence_number(1)
        .sign();
    executor.execute_and_apply(txn);
}

#[test]
fn crsns_prevent_replay_window_shift() {
    let mut executor = FakeExecutor::from_genesis_file();
    // set the diem version back
    let sender = executor.create_raw_account_data(1_000_010, 0);
    executor.add_account_data(&sender);
    executor.set_golden_file(current_function_name!());

    // This should apply
    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);

    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
            account_config::xus_tag(),
            *sender.address(),
            100,
            vec![],
            vec![],
        ))
        .sequence_number(1)
        .sign();
    executor.execute_and_apply(txn.clone());
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status().status().unwrap_err(),
        StatusCode::SEQUENCE_NONCE_INVALID
    );
}

#[test]
fn crsns_prevent_replay_no_window_shift() {
    let mut executor = FakeExecutor::from_genesis_file();
    // set the diem version back
    let sender = executor.create_raw_account_data(1_000_010, 0);
    executor.add_account_data(&sender);
    executor.set_golden_file(current_function_name!());

    // This should apply
    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);

    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
            account_config::xus_tag(),
            *sender.address(),
            100,
            vec![],
            vec![],
        ))
        .sequence_number(10)
        .sign();
    executor.execute_and_apply(txn.clone());

    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status().status().unwrap_err(),
        StatusCode::SEQUENCE_NONCE_INVALID
    );
}

#[test]
fn crsns_can_be_executed_out_of_order() {
    let mut executor = FakeExecutor::from_genesis_file();
    // set the diem version back
    let sender = executor.create_raw_account_data(1_000_010, 0);
    executor.add_account_data(&sender);
    executor.set_golden_file(current_function_name!());

    // This should apply
    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);

    let mut txns = Vec::new();

    // worst-case scenario for out-of-order execution
    for i in 0..K {
        let txn = sender
            .account()
            .transaction()
            .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
                account_config::xus_tag(),
                *sender.address(),
                100,
                vec![],
                vec![],
            ))
            .sequence_number(K - i)
            .sign();
        txns.push(txn);
    }

    for output in executor.execute_block_and_keep_vm_status(txns).unwrap() {
        assert_eq!(output.0.status_code(), StatusCode::EXECUTED);
    }
}

#[test]
fn force_expiration_of_crsns() {
    let mut executor = FakeExecutor::from_genesis_file();
    // set the diem version back
    let sender = executor.create_raw_account_data(1_000_010, 0);
    executor.add_account_data(&sender);
    executor.set_golden_file(current_function_name!());

    // This should apply
    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_opt_in_to_crsn_script_function(K))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);

    // worst-case scenario for out-of-order execution
    for i in K / 2..K {
        let txn = sender
            .account()
            .transaction()
            .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
                account_config::xus_tag(),
                *sender.address(),
                100,
                vec![],
                vec![],
            ))
            .sequence_number(i)
            .sign();
        executor.execute_and_apply(txn);
    }

    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
            account_config::xus_tag(),
            *sender.address(),
            100,
            vec![],
            vec![],
        ))
        .sequence_number(K + 1)
        .sign();
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status().status().unwrap_err(),
        StatusCode::SEQUENCE_NONCE_INVALID
    );

    let txn = sender
        .account()
        .transaction()
        .payload(stdlib::encode_force_expire_script_function(2 * K))
        .sequence_number(2)
        .sign();
    executor.execute_and_apply(txn);

    let mut txns = Vec::new();

    // Check that the old range is expired
    for i in 0..2 * K + 1 {
        let txn = sender
            .account()
            .transaction()
            .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
                account_config::xus_tag(),
                *sender.address(),
                100,
                vec![],
                vec![],
            ))
            .sequence_number(i)
            .sign();
        txns.push(txn);
    }

    for output in executor.execute_block_and_keep_vm_status(txns).unwrap() {
        assert_eq!(output.0.status_code(), StatusCode::SEQUENCE_NONCE_INVALID);
    }

    let mut txns = Vec::new();

    // and that the new range works
    for i in 0..K {
        let txn = sender
            .account()
            .transaction()
            .payload(stdlib::encode_peer_to_peer_with_metadata_script_function(
                account_config::xus_tag(),
                *sender.address(),
                100,
                vec![],
                vec![],
            ))
            .sequence_number(3 * K - i)
            .sign();

        txns.push(txn);
    }

    for output in executor.execute_block_and_keep_vm_status(txns).unwrap() {
        assert_eq!(output.0.status_code(), StatusCode::EXECUTED);
    }
}
