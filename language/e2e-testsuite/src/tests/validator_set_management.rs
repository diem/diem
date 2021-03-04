// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::{self, StdLibOptions};
use diem_types::{
    on_chain_config::{new_epoch_event_key, VMPublishingOption},
    transaction::{TransactionOutput, TransactionStatus, WriteSetPayload},
    vm_status::KeptVMStatus,
};
use language_e2e_tests::{account::Account, current_function_name, executor::FakeExecutor};
use transaction_builder::*;

fn assert_aborted_with(output: TransactionOutput, error_code: u64) {
    assert!(matches!(
        output.status().status(),
        Ok(KeptVMStatus::MoveAbort(_, code)) if code == error_code
    ));
}

fn try_add_validator(executor: &mut FakeExecutor) -> TransactionOutput {
    let diem_root_account = Account::new_diem_root();
    let validator_account = executor.create_raw_account();
    let operator_account = executor.create_raw_account();

    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_account_script(
                0,
                *validator_account.address(),
                validator_account.auth_key_prefix(),
                b"validator_0".to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_operator_account_script(
                0,
                *operator_account.address(),
                operator_account.auth_key_prefix(),
                b"operator_0".to_vec(),
            ))
            .sequence_number(2)
            .sign(),
    );
    // validator sets operator
    executor.execute_and_apply(
        validator_account
            .transaction()
            .script(encode_set_validator_operator_script(
                b"operator_0".to_vec(),
                *operator_account.address(),
            ))
            .sequence_number(0)
            .sign(),
    );
    executor.new_block();

    executor.execute_and_apply(
        operator_account
            .transaction()
            .script(encode_register_validator_config_script(
                *validator_account.address(),
                [
                    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9,
                    0x64, 0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02,
                    0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
                ]
                .to_vec(),
                vec![254; 32],
                vec![253; 32],
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.execute_transaction(
        diem_root_account
            .transaction()
            .script(encode_add_validator_and_reconfigure_script(
                2,
                b"validator_0".to_vec(),
                *validator_account.address(),
            ))
            .sequence_number(3)
            .sign(),
    )
}

#[test]
fn validator_add() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let output = try_add_validator(&mut executor);
    executor.apply_write_set(output.write_set());

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}

#[test]
fn validator_add_max_number() {
    let mut executor = FakeExecutor::custom_genesis(
        compiled_stdlib::stdlib_modules(StdLibOptions::Compiled)
            .bytes_opt
            .unwrap(),
        Some(256),
        VMPublishingOption::open(),
    );

    executor.set_golden_file(current_function_name!());

    let output = try_add_validator(&mut executor);

    assert_aborted_with(output, 1800);
}

#[test]
fn validator_rotate_key_and_reconfigure() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let diem_root_account = Account::new_diem_root();
    let validator_account = executor.create_raw_account();
    let validator_operator = executor.create_raw_account();

    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_account_script(
                0,
                *validator_account.address(),
                validator_account.auth_key_prefix(),
                b"validator_0".to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );

    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_operator_account_script(
                0,
                *validator_operator.address(),
                validator_operator.auth_key_prefix(),
                b"bobby".to_vec(),
            ))
            .sequence_number(2)
            .sign(),
    );
    // validator_0 sets operator
    executor.execute_and_apply(
        validator_account
            .transaction()
            .script(encode_set_validator_operator_script(
                b"bobby".to_vec(),
                *validator_operator.address(),
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.new_block();

    let output = executor.execute_and_apply(
        validator_operator
            .transaction()
            .script(encode_register_validator_config_script(
                *validator_account.address(),
                [
                    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9,
                    0x64, 0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02,
                    0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
                ]
                .to_vec(),
                vec![254; 32],
                vec![253; 32],
            ))
            .sequence_number(0)
            .sign(),
    );
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    let output = executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_add_validator_and_reconfigure_script(
                2,
                b"validator_0".to_vec(),
                *validator_account.address(),
            ))
            .sequence_number(3)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));

    executor.new_block_with_timestamp(300000010);

    let output = executor.execute_and_apply(
        validator_operator
            .transaction()
            .script(encode_set_validator_config_and_reconfigure_script(
                *validator_account.address(),
                [
                    0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a, 0x92, 0xb7, 0x0a, 0xa7, 0x4d,
                    0x1b, 0x7e, 0xbc, 0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c, 0xc0, 0xcd,
                    0x55, 0xf1, 0x2a, 0xf4, 0x66, 0x0c,
                ]
                .to_vec(),
                vec![254; 32],
                vec![253; 32],
            ))
            .sequence_number(1)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}

#[test]
fn validator_set_operator_set_key_reconfigure() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let diem_root_account = Account::new_diem_root();
    let validator_account = executor.create_raw_account();
    let operator_account_0 = executor.create_raw_account();
    let operator_account_1 = executor.create_raw_account();

    // Create operator 0
    let output = executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_operator_account_script(
                0,
                *operator_account_0.address(),
                operator_account_0.auth_key_prefix(),
                b"operator_0".to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    // Create operator 1
    let output = executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_operator_account_script(
                0,
                *operator_account_1.address(),
                operator_account_1.auth_key_prefix(),
                b"operator_1".to_vec(),
            ))
            .sequence_number(2)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    // Create validator 0
    let output = executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_account_script(
                0,
                *validator_account.address(),
                validator_account.auth_key_prefix(),
                b"validator_0".to_vec(),
            ))
            .sequence_number(3)
            .sign(),
    );
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    executor.new_block();

    // DR sets operator 1 for validator 0
    let admin_script = encode_set_validator_operator_with_nonce_admin_script(
        0,
        b"operator_1".to_vec(),
        *operator_account_1.address(),
    );
    let txn = diem_root_account
        .transaction()
        .write_set(WriteSetPayload::Script {
            script: admin_script,
            execute_as: *validator_account.address(),
        })
        .sequence_number(4)
        .sign();
    executor.new_block();
    let output = executor.execute_transaction(txn);

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    // Validator then sets operator 0
    let output = executor.execute_and_apply(
        validator_account
            .transaction()
            .script(encode_set_validator_operator_script(
                b"operator_0".to_vec(),
                *operator_account_0.address(),
            ))
            .sequence_number(0)
            .sign(),
    );
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    let output = executor.execute_and_apply(
        operator_account_0
            .transaction()
            .script(encode_register_validator_config_script(
                *validator_account.address(),
                [
                    0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a, 0x92, 0xb7, 0x0a, 0xa7, 0x4d,
                    0x1b, 0x7e, 0xbc, 0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c, 0xc0, 0xcd,
                    0x55, 0xf1, 0x2a, 0xf4, 0x66, 0x0c,
                ]
                .to_vec(),
                vec![254; 32],
                vec![253; 32],
            ))
            .sequence_number(0)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    let output = executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_add_validator_and_reconfigure_script(
                3,
                b"validator_0".to_vec(),
                *validator_account.address(),
            ))
            .sequence_number(4)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));

    executor.new_block_with_timestamp(300000010);

    let output = executor.execute_and_apply(
        operator_account_0
            .transaction()
            .script(encode_set_validator_config_and_reconfigure_script(
                *validator_account.address(),
                [
                    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9,
                    0x64, 0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02,
                    0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
                ]
                .to_vec(),
                vec![254; 32],
                vec![253; 32],
            ))
            .sequence_number(1)
            .sign(),
    );

    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    assert!(output
        .events()
        .iter()
        .any(|e| e.key() == &new_epoch_event_key()));
}
