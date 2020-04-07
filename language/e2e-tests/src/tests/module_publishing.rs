// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountData, assert_prologue_parity, assert_status_eq,
    compile::compile_module_with_address, executor::FakeExecutor, transaction_status_eq,
};
use libra_types::{
    account_config::lbr_type_tag,
    on_chain_config::VMPublishingOption,
    transaction::TransactionStatus,
    vm_error::{StatusCode, StatusType, VMStatus},
};

// A module with an address different from the sender's address should be rejected
#[test]
fn bad_module_address() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let account1 = AccountData::new(1_000_000, 10);
    let account2 = AccountData::new(1_000_000, 10);

    executor.add_account_data(&account1);
    executor.add_account_data(&account2);

    let program = String::from(
        "
        module M {

        }
        ",
    );

    // compile with account 1's address
    let compiled_module = compile_module_with_address(account1.address(), "file_name", &program);
    // send with account 2's address
    let txn = account2.account().create_signed_txn_impl(
        *account2.address(),
        compiled_module,
        10,
        100_000,
        1,
        lbr_type_tag(),
    );

    // TODO: This is not verified for now.
    // verify and fail because the addresses don't match
    // let vm_status = executor.verify_transaction(txn.clone()).status().unwrap();
    // assert!(vm_status.is(StatusType::Verification));
    // assert!(vm_status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    let status = match output.status() {
        TransactionStatus::Keep(status) => {
            assert!(status.is(StatusType::Verification));
            status
        }
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
    assert!(status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);
}

// Publishing a module named M under the same address twice should be rejected
#[test]
fn duplicate_module() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    let sequence_number = 2;
    let account = AccountData::new(1_000_000, sequence_number);
    executor.add_account_data(&account);

    let program = String::from(
        "
        module M {

        }
        ",
    );
    let compiled_module = compile_module_with_address(account.address(), "file_name", &program);

    let txn1 = account.account().create_signed_txn_impl(
        *account.address(),
        compiled_module.clone(),
        sequence_number,
        100_000,
        1,
        lbr_type_tag(),
    );

    let txn2 = account.account().create_signed_txn_impl(
        *account.address(),
        compiled_module,
        sequence_number + 1,
        100_000,
        1,
        lbr_type_tag(),
    );

    let output1 = executor.execute_transaction(txn1);
    executor.apply_write_set(output1.write_set());
    // first tx should succeed
    assert!(transaction_status_eq(
        &output1.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
    ));

    // second one should fail because it tries to re-publish a module named M
    let output2 = executor.execute_transaction(txn2);
    assert!(transaction_status_eq(
        &output2.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::DUPLICATE_MODULE_NAME)),
    ));
}

#[test]
pub fn test_publishing_no_modules_non_whitelist_script() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script = compile_module_with_address(sender.address(), "file_name", &program);
    let txn = sender.account().create_signed_txn_impl(
        *sender.address(),
        random_script,
        10,
        100_000,
        1,
        lbr_type_tag(),
    );

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        VMStatus::new(StatusCode::UNKNOWN_MODULE)
    );
}

#[test]
pub fn test_publishing_allow_modules() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script = compile_module_with_address(sender.address(), "file_name", &program);
    let txn = sender.account().create_signed_txn_impl(
        *sender.address(),
        random_script,
        10,
        100_000,
        1,
        lbr_type_tag(),
    );
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
}
