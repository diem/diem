// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_types::{account_config, vm_status::KeptVMStatus};
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
fn max_child_accounts_for_vasp() {
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

    for i in 0..max_num_child_accounts {
        let child = executor.create_raw_account();
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

    let child = executor.create_raw_account();
    let output = executor.execute_transaction(
        account
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
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
