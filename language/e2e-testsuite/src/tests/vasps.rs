// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_transaction_builder::stdlib::*;
use diem_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, TransactionStatus},
    vm_status::KeptVMStatus,
};
use language_e2e_tests::{test_with_different_versions, versioning::CURRENT_RELEASE_VERSIONS};
use move_core_types::{
    value::{serialize_values, MoveValue},
    vm_status::VMStatus,
};

#[test]
fn valid_creator_already_vasp() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let account = executor.create_raw_account();

        let treasury_compliance = test_env.tc_account;

        executor.execute_and_apply(
            treasury_compliance
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                    account_config::xus_tag(),
                    0,
                    *account.address(),
                    account.auth_key_prefix(),
                    vec![],
                    true,
                ))
                .sequence_number(test_env.tc_sequence_number)
                .sign(),
        );

        let err = executor
            .try_exec(
                "VASP",
                "publish_parent_vasp_credential",
                vec![],
                serialize_values(&vec![
                    MoveValue::Signer(*account.address()),
                    MoveValue::Signer(*treasury_compliance.address()),
                ]),
            )
            .unwrap_err();
        if let VMStatus::MoveAbort(_, code) = err {
            assert_eq!(code, 6);
        } else {
            panic!("expected MoveAbort")
        }
    }
    }
}

#[test]
fn max_child_accounts_for_vasp_recovery_address() {
    let max_num_child_accounts = 256;

    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let account = executor.create_raw_account();

        let treasury_compliance = test_env.tc_account;

        executor.execute_and_apply(
            treasury_compliance
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                    account_config::xus_tag(),
                    0,
                    *account.address(),
                    account.auth_key_prefix(),
                    vec![],
                    true,
                ))
                .sequence_number(test_env.tc_sequence_number)
                .sign(),
        );

        let mut accounts = Vec::new();

        // Batch the transactions to reduce the runtime of the test (cuts ~100 seconds)
        let mut block = Vec::new();
        // Create the maximum number of allowed child accounts
        for i in 0..max_num_child_accounts + 1 {
            let child = executor.create_raw_account();
            accounts.push(child.clone());
            block.push(
                account
                    .transaction()
                    .script(encode_create_child_vasp_account_script(
                        account_config::xus_tag(),
                        *child.address(),
                        child.auth_key_prefix(),
                        false,
                        0,
                    ))
                    .sequence_number(i)
                    .sign(),
            );
        }

        let output = executor.execute_block(block).unwrap();
        for output in output {
            executor.apply_write_set(output.write_set())
        }

        // Now setup the recovery addresses
        let recovery_account = accounts.remove(0);
        let one_account_too_many = accounts.remove(0);
        executor.execute_and_apply(
            recovery_account
                .transaction()
                .script(encode_create_recovery_address_script())
                .sequence_number(0)
                .sign(),
        );

        // Batch the transactions to reduce the runtime of the test (cuts ~ another 100 seconds)
        let block = accounts
            .iter()
            .map(|account| {
                account
                    .transaction()
                    .script(encode_add_recovery_rotation_capability_script(
                        *recovery_account.address(),
                    ))
                    .sequence_number(0)
                    .sign()
            })
            .collect();

        let outputs = executor.execute_block(block).unwrap();

        for output in outputs {
            executor.apply_write_set(output.write_set())
        }

        // Make sure that we can't add any more
        let output = executor.execute_transaction(
            one_account_too_many
                .transaction()
                .script(encode_add_recovery_rotation_capability_script(
                    *recovery_account.address(),
                ))
                .sequence_number(0)
                .sign(),
        );

        if let Ok(KeptVMStatus::MoveAbort(_, code)) = output.status().status() {
            assert_eq!(code, 1544);
        } else {
            panic!("expected MoveAbort")
        }

        // Batch again, cuts about ~100 seconds more as well
        let block = accounts
            .iter()
            .map(|account| {
                let privkey = Ed25519PrivateKey::generate_for_testing();
                let pubkey = privkey.public_key();
                let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
                account
                    .transaction()
                    .script(
                        encode_rotate_authentication_key_with_recovery_address_script(
                            *recovery_account.address(),
                            *account.address(),
                            new_key_hash,
                        ),
                    )
                    .sequence_number(1)
                    .sign()
            })
            .collect();

        let outputs = executor.execute_block(block).unwrap();

        // Now make sure all the rotations were executed
        for output in outputs {
            assert!(*output.status() == TransactionStatus::Keep(KeptVMStatus::Executed));
        }
    }
    }
}
