// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_types::{
    account_config, transaction::authenticator::AuthenticationKey, vm_status::KeptVMStatus,
};
use language_e2e_tests::{account::Account, current_function_name, executor::FakeExecutor};
use move_core_types::vm_status::VMStatus;
use move_vm_types::values::Value;
use transaction_builder::*;

#[test]
fn valid_creator_already_vasp() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let account = executor.create_raw_account();

    let treasury_compliance = Account::new_blessed_tc();

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
            .sequence_number(0)
            .sign(),
    );

    let err = executor
        .try_exec(
            "VASP",
            "publish_parent_vasp_credential",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(*account.address()),
                Value::transaction_argument_signer_reference(*treasury_compliance.address()),
            ],
            treasury_compliance.address(),
        )
        .unwrap_err();
    if let VMStatus::MoveAbort(_, code) = err {
        assert_eq!(code, 6);
    } else {
        panic!("expected MoveAbort")
    }
}

#[test]
fn max_child_accounts_for_vasp_recovery_address() {
    let max_num_child_accounts = 256;

    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let account = executor.create_raw_account();

    let treasury_compliance = Account::new_blessed_tc();

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
            .sequence_number(0)
            .sign(),
    );

    let mut accounts = Vec::new();

    // Create the maximum number of allowed child accounts
    for i in 0..max_num_child_accounts + 1 {
        let child = executor.create_raw_account();
        accounts.push(child.clone());
        executor.execute_and_apply(
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
    for account in &accounts {
        executor.execute_and_apply(
            account
                .transaction()
                .script(encode_add_recovery_rotation_capability_script(
                    *recovery_account.address(),
                ))
                .sequence_number(0)
                .sign(),
        );
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

    // Now rotate all of the account keys
    for account in &accounts {
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
        executor.execute_and_apply(
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
                .sign(),
        );
    }
}
