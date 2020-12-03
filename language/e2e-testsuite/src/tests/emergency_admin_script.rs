// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::HashValue;
use diem_types::{
    account_config::diem_root_address,
    on_chain_config::new_epoch_event_key,
    transaction::{TransactionPayload, TransactionStatus},
    vm_status::KeptVMStatus,
};
use diem_writeset_generator::{
    encode_custom_script, encode_halt_network_transaction, encode_remove_validators_transaction,
};
use language_e2e_tests::{
    account::Account, common_transactions::peer_to_peer_txn, current_function_name,
    executor::FakeExecutor,
};
use move_core_types::vm_status::StatusCode;
use move_vm_types::values::Value;
use serde_json::json;
use transaction_builder::*;

#[test]
fn validator_batch_remove() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let diem_root_account = Account::new_diem_root();
    let validator_account_0 = executor.create_raw_account();
    let validator_account_1 = executor.create_raw_account();
    let operator_account = executor.create_raw_account();

    // Add validator_0
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_account_script(
                0,
                *validator_account_0.address(),
                validator_account_0.auth_key_prefix(),
                b"validator_0".to_vec(),
            ))
            .sequence_number(1)
            .sign(),
    );
    // Add operator
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_operator_account_script(
                0,
                *operator_account.address(),
                operator_account.auth_key_prefix(),
                b"operator".to_vec(),
            ))
            .sequence_number(2)
            .sign(),
    );
    // validator_0 sets operator
    executor.execute_and_apply(
        validator_account_0
            .transaction()
            .script(encode_set_validator_operator_script(
                b"operator".to_vec(),
                *operator_account.address(),
            ))
            .sequence_number(0)
            .sign(),
    );

    executor.new_block();

    // operator_accounts registers config
    executor.execute_and_apply(
        operator_account
            .transaction()
            .script(encode_register_validator_config_script(
                *validator_account_0.address(),
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

    // diem_root adds validator
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_add_validator_and_reconfigure_script(
                2,
                b"validator_0".to_vec(),
                *validator_account_0.address(),
            ))
            .sequence_number(3)
            .sign(),
    );

    // Add validator_1
    executor.new_block();
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_create_validator_account_script(
                0,
                *validator_account_1.address(),
                validator_account_1.auth_key_prefix(),
                b"validator_1".to_vec(),
            ))
            .sequence_number(4)
            .sign(),
    );
    // validator_1 sets operator
    executor.execute_and_apply(
        validator_account_1
            .transaction()
            .script(encode_set_validator_operator_script(
                b"operator".to_vec(),
                *operator_account.address(),
            ))
            .sequence_number(0)
            .sign(),
    );
    executor.new_block();

    // operator sets the config for validator_account_1
    executor.execute_and_apply(
        operator_account
            .transaction()
            .script(encode_register_validator_config_script(
                *validator_account_1.address(),
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

    // diem_root adds validator
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_add_validator_and_reconfigure_script(
                3,
                b"validator_1".to_vec(),
                *validator_account_1.address(),
            ))
            .sequence_number(5)
            .sign(),
    );

    let txn1 = encode_remove_validators_transaction(vec![
        *validator_account_0.address(),
        *validator_account_1.address(),
    ]);

    let txn2 = encode_custom_script(
        "remove_validators.move",
        &json!({ "addresses": [validator_account_0.address().to_string(), validator_account_1.address().to_string()]}),
    );

    assert_eq!(txn1, txn2);
    // Remove two newly added validators.
    executor.new_block();
    let output = executor
        .execute_transaction_block(vec![txn1])
        .unwrap()
        .pop()
        .unwrap();
    assert!(output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key()));
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    // Make sure both validators are removed from the validator set.
    assert!(executor
        .try_exec(
            "DiemSystem",
            "remove_validator",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(diem_root_address()),
                Value::address(*validator_account_0.address())
            ],
            diem_root_account.address()
        )
        .is_err());
    assert!(executor
        .try_exec(
            "DiemSystem",
            "remove_validator",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(diem_root_address()),
                Value::address(*validator_account_1.address())
            ],
            diem_root_account.address()
        )
        .is_err());
}

#[test]
fn halt_network() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let diem_root_account = Account::new_diem_root();
    let sender = executor.create_raw_account_data(1_000_000, 10);
    let receiver = executor.create_raw_account_data(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    executor.new_block();
    let output = executor
        .execute_transaction_block(vec![encode_halt_network_transaction()])
        .unwrap()
        .pop()
        .unwrap();

    assert!(output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key()));
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, 1);
    let script_hash = match txn.payload() {
        TransactionPayload::Script(s) => HashValue::sha3_256_of(s.code()).to_vec(),
        _ => panic!("Unexpected types of transaction"),
    };
    // Regular transactions like p2p are no longer allowed.
    let output = executor.execute_transaction(txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::UNKNOWN_SCRIPT)
    );

    // DiemRoot can still send transaction
    executor.execute_and_apply(
        diem_root_account
            .transaction()
            .script(encode_add_to_script_allow_list_script(script_hash, 0))
            .sequence_number(1)
            .sign(),
    );
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}
