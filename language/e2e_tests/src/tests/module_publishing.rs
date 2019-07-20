// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::AccountData, assert_prologue_parity, compile::compile_program_with_address,
    executor::FakeExecutor,
};
use assert_matches::assert_matches;
use config::config::VMPublishingOption;
use types::{
    transaction::TransactionStatus,
    vm_error::{
        ExecutionStatus, VMStatus, VMValidationStatus, VMVerificationError, VMVerificationStatus,
    },
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
        modules:
        module M {

        }

        script:
        main() {
          return;
        }
        ",
    );

    // compile with account 1's address
    let compiled_script = compile_program_with_address(account1.address(), &program, vec![]);
    // send with account 2's address
    let txn = account2.account().create_signed_txn_impl(
        *account2.address(),
        compiled_script,
        10,
        100_000,
        1,
    );

    // verify and fail because the addresses don't match
    let vm_status = executor.verify_transaction(txn.clone());
    let status = match vm_status {
        Some(VMStatus::Verification(status)) => status,
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
    match status.as_slice() {
        &[VMVerificationStatus::Module(
            0,
            VMVerificationError::ModuleAddressDoesNotMatchSender(_),
        )] => {}
        err => panic!("Unexpected verification error: {:?}", err),
    };

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    let status = match output.status() {
        TransactionStatus::Discard(VMStatus::Verification(status)) => status,
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
    match status.as_slice() {
        &[VMVerificationStatus::Module(
            0,
            VMVerificationError::ModuleAddressDoesNotMatchSender(_),
        )] => {}
        err => panic!("Unexpected verification error: {:?}", err),
    };
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
        modules:
        module M {

        }

        script:
        main() {
          return;
        }
        ",
    );
    let compiled_script = compile_program_with_address(account.address(), &program, vec![]);

    let txn1 = account.account().create_signed_txn_impl(
        *account.address(),
        compiled_script.clone(),
        sequence_number,
        100_000,
        1,
    );

    let txn2 = account.account().create_signed_txn_impl(
        *account.address(),
        compiled_script,
        sequence_number + 1,
        100_000,
        1,
    );

    let output1 = executor.execute_transaction(txn1);
    executor.apply_write_set(output1.write_set());
    // first tx should succeed
    assert_eq!(
        output1.status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)),
    );

    // second one should fail because it tries to re-publish a module named M
    let output2 = executor.execute_transaction(txn2);
    assert_eq!(
        output2.status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::DuplicateModuleName)),
    );
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
        modules:
        module M {
        }
        script:
        main () {
            return;
        }
        ",
    );

    let random_script = compile_program_with_address(sender.address(), &program, vec![]);
    let txn =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()),
        executor.execute_transaction(txn).status(),
        VMStatus::Validation(VMValidationStatus::UnknownModule)
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
        modules:
        module M {
        }
        script:
        main () {
            return;
        }",
    );

    let random_script = compile_program_with_address(sender.address(), &program, vec![]);;
    let txn =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);
    assert_eq!(executor.verify_transaction(txn.clone()), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
}

#[test]
pub fn test_publishing_with_error() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);

    // create a transaction trying to publish a new module.
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        modules:
        module M {
        }
        script:
        main () {
            assert(false, 42);
            return;
        }",
    );

    let random_script = compile_program_with_address(sender.address(), &program, vec![]);;
    let txn1 =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 10, 100_000, 1);
    let program = String::from(
        "
        modules:
        module M {
        }
        script:
        main () {
            return;
        }",
    );

    let random_script = compile_program_with_address(sender.address(), &program, vec![]);;
    let txn2 =
        sender
            .account()
            .create_signed_txn_impl(*sender.address(), random_script, 11, 100_000, 1);

    assert_eq!(executor.verify_transaction(txn1.clone()), None);
    assert_eq!(executor.verify_transaction(txn2.clone()), None);

    let result = executor.execute_block(vec![txn1, txn2]);
    assert_eq!(
        result[0].status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Aborted(42)))
    );

    assert_eq!(
        result[1].status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
    );
}
