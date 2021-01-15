// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    on_chain_config::VMPublishingOption, transaction::TransactionStatus, vm_status::KeptVMStatus,
};
use language_e2e_tests::{
    compile::{
        compile_module_with_address, compile_module_with_address_and_deps,
        compile_script_with_address,
    },
    current_function_name,
    executor::FakeExecutor,
    transaction_status_eq,
};
use move_core_types::vm_status::StatusCode;

#[test]
fn no_script_execution_after_module_update() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
    executor.set_golden_file(current_function_name!());

    let mut seq = 10;

    // prepare an account
    let account = executor.create_raw_account_data(1_000_000, seq);
    executor.add_account_data(&account);

    // publish module A
    let program_module_a = String::from(
        "
        module A {
            public a() { return; }
        }
        ",
    );
    let (compiled_module_a, serialized_module_a) =
        compile_module_with_address(account.address(), "file_name", &program_module_a);

    let txn_a = account
        .account()
        .transaction()
        .module(serialized_module_a)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // execute the script to load module A in code cache
    let program_script = format!(
        "
            import 0x{}.A;
            main(account: &signer) {{ A.a(); return; }}
        ",
        account.address(),
    );

    let serialized_script = compile_script_with_address(
        account.address(),
        "file_name",
        &program_script,
        vec![compiled_module_a],
    );

    let txn_s = account
        .account()
        .transaction()
        .script(serialized_script)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // update module A to A'
    let program_module_a1 = String::from(
        "
        module A {
            public a() { abort 0; }
        }
        ",
    );
    let (compiled_module_a1, serialized_module_a1) =
        compile_module_with_address(account.address(), "file_name", &program_module_a1);
    let txn_a1 = account
        .account()
        .transaction()
        .module(serialized_module_a1)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // re-compile the script
    let serialized_script_recompiled = compile_script_with_address(
        account.address(),
        "file_name",
        &program_script,
        vec![compiled_module_a1],
    );

    let txn_s1 = account
        .account()
        .transaction()
        .script(serialized_script_recompiled)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();

    // execute all transactions in the same block
    let mut outputs = executor
        .execute_block(vec![txn_a, txn_s, txn_a1, txn_s1])
        .expect("The VM should not fail to startup");

    assert_eq!(outputs.len() as u64, 4);

    // Only first 3 txns should pass, txn_s1 should abort because of expired code cache
    assert!(transaction_status_eq(
        outputs.pop().unwrap().status(),
        &TransactionStatus::Discard(StatusCode::CODE_CACHE_EXPIRED),
    ));
    for output in outputs {
        assert!(transaction_status_eq(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed),
        ));
    }
}

#[test]
fn no_module_publish_after_module_update() {
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::open());
    executor.set_golden_file(current_function_name!());

    let mut seq = 10;

    // prepare an account
    let account = executor.create_raw_account_data(1_000_000, seq);
    executor.add_account_data(&account);

    // publish module C
    let program_module_c = String::from(
        "
        module C {
            public c() { return; }
        }
        ",
    );
    let (compiled_module_c, serialized_module_c) =
        compile_module_with_address(account.address(), "file_name", &program_module_c);
    let txn_c = account
        .account()
        .transaction()
        .module(serialized_module_c)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // publish module A
    let program_module_a = String::from(
        "
        module A {
            public a() { return; }
        }
        ",
    );
    let (compiled_module_a, serialized_module_a) =
        compile_module_with_address(account.address(), "file_name", &program_module_a);
    let txn_a = account
        .account()
        .transaction()
        .module(serialized_module_a)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // execute the script to load everything in code cache
    let program_script = format!(
        "
            import 0x{}.A;
            import 0x{}.C;
            main(account: &signer) {{ A.a(); C.c(); return; }}
        ",
        account.address(),
        account.address(),
    );

    let serialized_script = compile_script_with_address(
        account.address(),
        "file_name",
        &program_script,
        vec![compiled_module_a.clone(), compiled_module_c],
    );

    let txn_s = account
        .account()
        .transaction()
        .script(serialized_script)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // update module C'
    let program_module_c1 = format!(
        "
        module C {{
            import 0x{}.A;
            public c() {{ A.a(); return; }}
        }}
        ",
        account.address()
    );
    let (compiled_module_c1, serialized_module_c1) = compile_module_with_address_and_deps(
        account.address(),
        "file_name",
        &program_module_c1,
        vec![compiled_module_a],
    );
    let txn_c1 = account
        .account()
        .transaction()
        .module(serialized_module_c1)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();
    seq += 1;

    // update module A'
    let program_module_a1 = format!(
        "
        module A {{
            import 0x{}.C;
            public a() {{ C.c(); return; }}
        }}
        ",
        account.address(),
    );
    let (_, serialized_module_a1) = compile_module_with_address_and_deps(
        account.address(),
        "file_name",
        &program_module_a1,
        vec![compiled_module_c1],
    );
    let txn_a1 = account
        .account()
        .transaction()
        .module(serialized_module_a1)
        .sequence_number(seq)
        .gas_unit_price(1)
        .sign();

    // execute all transactions in the same block
    let mut outputs = executor
        .execute_block(vec![txn_c, txn_a, txn_s, txn_c1, txn_a1])
        .expect("The VM should not fail to startup");

    // Only first 4 txns should pass, txn_s1 should abort because of expired code cache
    assert!(transaction_status_eq(
        outputs.pop().unwrap().status(),
        &TransactionStatus::Discard(StatusCode::CODE_CACHE_EXPIRED),
    ));
    for output in outputs {
        assert!(transaction_status_eq(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed),
        ));
    }
}
