// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    account_config::{self},
    on_chain_config::VMPublishingOption,
    transaction::TransactionStatus,
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{
    account::Account, assert_prologue_parity, compile::compile_module_with_address,
    current_function_name, executor::FakeExecutor, transaction_status_eq,
};

// A module with an address different from the sender's address should be rejected
#[test]
fn bad_module_address() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let account1 = executor.create_raw_account_data(1_000_000, 10);
    let account2 = executor.create_raw_account_data(1_000_000, 10);

    executor.add_account_data(&account1);
    executor.add_account_data(&account2);

    let program = String::from(
        "
        module M {

        }
        ",
    );

    // compile with account 1's address
    let compiled_module = compile_module_with_address(account1.address(), "file_name", &program).1;
    // send with account 2's address
    let txn = account2
        .account()
        .transaction()
        .module(compiled_module)
        .sequence_number(10)
        .gas_unit_price(1)
        .sign();

    // TODO: This is not verified for now.
    // verify and fail because the addresses don't match
    // let vm_status = executor.verify_transaction(txn.clone()).status().unwrap();
    // assert!(vm_status.is(StatusType::Verification));
    // assert!(vm_status.major_status == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);

    // execute and fail for the same reason
    let output = executor.execute_transaction(txn);
    match output.status() {
        TransactionStatus::Keep(status) => {
            assert!(status == &KeptVMStatus::MiscellaneousError);
            // assert!(status.status_code() == StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER);
        }
        vm_status => panic!("Unexpected verification status: {:?}", vm_status),
    };
}

macro_rules! module_republish_test {
    ($name:ident, $prog1:literal, $prog2:literal, $result:ident) => {
        #[test]
        fn $name() {
            let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
            executor.set_golden_file(current_function_name!());

            let sequence_number = 2;
            let account = executor.create_raw_account_data(1_000_000, sequence_number);
            executor.add_account_data(&account);

            let program1 = String::from($prog1);
            let compiled_module1 =
                compile_module_with_address(account.address(), "file_name", &program1).1;

            let txn1 = account
                .account()
                .transaction()
                .module(compiled_module1.clone())
                .sequence_number(sequence_number)
                .sign();

            let program2 = String::from($prog2);
            let compiled_module2 =
                compile_module_with_address(account.address(), "file_name", &program2).1;

            let txn2 = account
                .account()
                .transaction()
                .module(compiled_module2)
                .sequence_number(sequence_number + 1)
                .sign();

            let output1 = executor.execute_transaction(txn1);
            executor.apply_write_set(output1.write_set());
            // first tx should allways succeed
            assert!(transaction_status_eq(
                &output1.status(),
                &TransactionStatus::Keep(KeptVMStatus::Executed),
            ));

            let output2 = executor.execute_transaction(txn2);
            // second tx should yield the expected result
            assert!(transaction_status_eq(
                &output2.status(),
                &TransactionStatus::Keep(KeptVMStatus::$result),
            ));
        }
    };
}

// Publishing a module named M under the same address twice is OK (a module is self-compatible)
module_republish_test!(
    duplicate_module,
    "
    module M {
        struct T { f: u64 }
        public f() { return; }
    }
    ",
    "
    module M {
        struct T { f: u64 }
        public f() { return; }
    }
    ",
    Executed
);

// Republishing a module named M under the same address with a superset of the structs is OK
module_republish_test!(
    layout_compatible_module,
    "
    module M {
    }
    ",
    "
    module M {
        struct T { f: u64 }
    }
    ",
    Executed
);

// Republishing a module named M under the same address with a superset of public functions is OK
module_republish_test!(
    linking_compatible_module,
    "
    module M {
    }
    ",
    "
    module M {
        public f() { return; }
    }
    ",
    Executed
);

// Republishing a module named M under the same address that breaks data layout should be rejected
module_republish_test!(
    layout_incompatible_module_with_new_field,
    "
    module M {
        struct T { f: u64 }
    }
    ",
    "
    module M {
        struct T { f: u64, g: bool }
    }
    ",
    MiscellaneousError
);

module_republish_test!(
    layout_incompatible_module_with_changed_field,
    "
    module M {
        struct T { f: u64 }
    }
    ",
    "
    module M {
        struct T { f: bool }
    }
    ",
    MiscellaneousError
);

module_republish_test!(
    layout_incompatible_module_with_removed_field,
    "
    module M {
        struct T { f: u64 }
    }
    ",
    "
    module M {
        struct T {}
    }
    ",
    MiscellaneousError
);

module_republish_test!(
    layout_incompatible_module_with_removed_struct,
    "
    module M {
        struct T { f: u64 }
    }
    ",
    "
    module M {
    }
    ",
    MiscellaneousError
);

// Republishing a module named M under the same address that breaks linking should be rejected
module_republish_test!(
    linking_incompatible_module_with_added_param,
    "
    module M {
        public f() { return; }
    }
    ",
    "
    module M {
        public f(_a: u64) { return; }
    }
    ",
    MiscellaneousError
);

module_republish_test!(
    linking_incompatible_module_with_changed_param,
    "
    module M {
        public f(_a: u64) { return; }
    }
    ",
    "
    module M {
        public f(_a: bool) { return; }
    }
    ",
    MiscellaneousError
);

module_republish_test!(
    linking_incompatible_module_with_removed_pub_fn,
    "
    module M {
        public f() { return; }
    }
    ",
    "
    module M {
    }
    ",
    MiscellaneousError
);

#[test]
pub fn test_publishing_no_modules_non_allowlist_script() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_module = compile_module_with_address(sender.address(), "file_name", &program).1;
    let txn = sender
        .account()
        .transaction()
        .module(random_module)
        .sequence_number(10)
        .gas_unit_price(1)
        .sign();

    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_MODULE_PUBLISHER
    );
}

#[test]
pub fn test_publishing_no_modules_non_allowlist_script_proper_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor =
        FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = Account::new_diem_root();

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_module =
        compile_module_with_address(&account_config::CORE_CODE_ADDRESS, "file_name", &program).1;
    let txn = sender
        .transaction()
        .module(random_module)
        .sequence_number(1)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}

#[test]
pub fn test_publishing_no_modules_proper_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = Account::new_diem_root();

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script =
        compile_module_with_address(&account_config::CORE_CODE_ADDRESS, "file_name", &program).1;
    let txn = sender
        .transaction()
        .module(random_script)
        .sequence_number(1)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}

#[test]
pub fn test_publishing_no_modules_core_code_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = Account::new_genesis_account(account_config::CORE_CODE_ADDRESS);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script =
        compile_module_with_address(&account_config::CORE_CODE_ADDRESS, "file_name", &program).1;
    let txn = sender
        .transaction()
        .module(random_script)
        .sequence_number(1)
        .sign();
    // Doesn't work because the core code address doesn't exist
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_MODULE_PUBLISHER
    );
}

#[test]
pub fn test_publishing_no_modules_invalid_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::allowlist_genesis();
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script = compile_module_with_address(sender.address(), "file_name", &program).1;
    let txn = sender
        .account()
        .transaction()
        .module(random_script)
        .sequence_number(10)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(txn.clone()).status(),
        executor.execute_transaction(txn).status(),
        StatusCode::INVALID_MODULE_PUBLISHER
    );
}

#[test]
pub fn test_publishing_allow_modules() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
        }
        ",
    );

    let random_script = compile_module_with_address(sender.address(), "file_name", &program).1;
    let txn = sender
        .account()
        .transaction()
        .module(random_script)
        .sequence_number(10)
        .sign();
    assert_eq!(executor.verify_transaction(txn.clone()).status(), None);
    assert_eq!(
        executor.execute_transaction(txn).status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}
