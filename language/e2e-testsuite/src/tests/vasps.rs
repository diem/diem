// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use language_e2e_tests::{account::Account, executor::FakeExecutor};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::{
    account_config, transaction::authenticator::AuthenticationKey, vm_status::KeptVMStatus,
};
use move_core_types::vm_status::VMStatus;
use move_vm_types::values::Value;
use transaction_builder::*;

#[test]
fn valid_creator_already_vasp() {
    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let treasury_compliance = Account::new_blessed_tc();

    executor.execute_and_apply(
        treasury_compliance
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tmp_tag(),
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
fn max_child_accounts_for_vasp() {
    let max_num_child_accounts = 256;

    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let treasury_compliance = Account::new_blessed_tc();

    executor.execute_and_apply(
        treasury_compliance
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tmp_tag(),
                0,
                *account.address(),
                account.auth_key_prefix(),
                vec![],
                true,
            ))
            .sequence_number(0)
            .sign(),
    );

    for i in 0..max_num_child_accounts {
        let child = Account::new();
        executor.execute_and_apply(
            account
                .transaction()
                .script(encode_create_child_vasp_account_script(
                    account_config::coin1_tmp_tag(),
                    *child.address(),
                    child.auth_key_prefix(),
                    false,
                    0,
                ))
                .sequence_number(i)
                .sign(),
        );
    }

    let child = Account::new();
    let output = executor.execute_transaction(
        account
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tmp_tag(),
                *child.address(),
                child.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(max_num_child_accounts)
            .sign(),
    );

    if let Ok(KeptVMStatus::MoveAbort(_, code)) = output.status().status() {
        assert_eq!(code, 264); // ETOO_MANY_CHILDREN
    } else {
        panic!("expected MoveAbort")
    }
}

#[test]
fn max_child_accounts_for_vasp_recovery_address() {
    let max_num_child_accounts = 256;

    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let treasury_compliance = Account::new_blessed_tc();

    executor.execute_and_apply(
        treasury_compliance
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tmp_tag(),
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
    for i in 0..max_num_child_accounts {
        let child = Account::new();
        accounts.push(child.clone());
        executor.execute_and_apply(
            account
                .transaction()
                .script(encode_create_child_vasp_account_script(
                    account_config::coin1_tmp_tag(),
                    *child.address(),
                    child.auth_key_prefix(),
                    false,
                    0,
                ))
                .sequence_number(i)
                .sign(),
        );
    }

    // make sure we've maxed the number out
    let child = Account::new();
    let output = executor.execute_transaction(
        account
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::coin1_tmp_tag(),
                *child.address(),
                child.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(max_num_child_accounts)
            .sign(),
    );

    if let Ok(KeptVMStatus::MoveAbort(_, code)) = output.status().status() {
        assert_eq!(code, 264); // ETOO_MANY_CHILDREN
    } else {
        panic!("expected MoveAbort")
    }

    // Now setup the recovery addresses
    let recovery_account = accounts.remove(0);
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
