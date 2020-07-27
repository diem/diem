// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{account::Account, executor::FakeExecutor, keygen::KeyGen};
use libra_types::{account_config, vm_status::KeptVMStatus};
use move_vm_types::values::Value;
use transaction_builder::*;

#[test]
fn valid_creator_already_vasp() {
    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_, cpubkey) = keygen.generate_keypair();

    let libra_root = Account::new_libra_root();

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                *account.address(),
                account.auth_key_prefix(),
                vec![],
                vec![],
                cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(1)
            .sign(),
    );

    let err = executor
        .try_exec(
            "VASP",
            "publish_parent_vasp_credential",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(*account.address()),
                Value::transaction_argument_signer_reference(*libra_root.address()),
            ],
            libra_root.address(),
        )
        .unwrap_err();
    assert_eq!(err.sub_status(), Some(7));
}

#[test]
fn max_child_accounts_for_vasp() {
    let max_num_child_accounts = 256;

    let mut executor = FakeExecutor::from_genesis_file();

    let account = Account::new();

    let mut keygen = KeyGen::from_seed([9u8; 32]);
    let (_, cpubkey) = keygen.generate_keypair();

    let libra_root = Account::new_libra_root();

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                *account.address(),
                account.auth_key_prefix(),
                vec![],
                vec![],
                cpubkey.to_bytes().to_vec(),
                true,
            ))
            .sequence_number(1)
            .sign(),
    );

    for i in 0..max_num_child_accounts {
        let child = Account::new();
        executor.execute_and_apply(
            account
                .transaction()
                .script(encode_create_child_vasp_account_script(
                    account_config::coin1_tag(),
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
                account_config::coin1_tag(),
                *child.address(),
                child.auth_key_prefix(),
                false,
                0,
            ))
            .sequence_number(max_num_child_accounts)
            .sign(),
    );

    assert!(matches!(
        output.status().status(),
        Ok(KeptVMStatus::MoveAbort(_, 8)) // ETOO_MANY_CHILDREN
    ));
}
