// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountData, compile::compile_script_with_address, executor::FakeExecutor};
use bytecode_verifier::VerifiedModule;
use compiler::Compiler;
use libra_types::{
    account_config::LBR_NAME,
    transaction::{Module, SignedTransaction, Transaction, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn move_from_across_blocks() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // remove resource
    let rem_txn = remove_resource_txn(&sender, 14, vec![module.clone()]);
    executor.execute_and_apply(rem_txn);

    // remove resource fails given it was removed already
    let rem_txn = remove_resource_txn(&sender, 15, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());

    // borrow resource fail given it was removed
    let borrow_txn = borrow_resource_txn(&sender, 16, vec![module.clone()]);
    let output = executor.execute_transaction(borrow_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());

    // publish resource again
    let add_txn = add_resource_txn(&sender, 17, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // create 2 remove resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 18, vec![module.clone()])),
        Transaction::UserTransaction(remove_resource_txn(&sender, 19, vec![module])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert_eq!(
        output[1].status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    for out in output {
        executor.apply_write_set(out.write_set());
    }
}

#[test]
fn borrow_after_move() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // create a remove and a borrow resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 14, vec![module.clone()])),
        Transaction::UserTransaction(borrow_resource_txn(&sender, 15, vec![module])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert_eq!(
        output[1].status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    for out in output {
        executor.apply_write_set(out.write_set());
    }
}

#[test]
fn change_after_move() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // create a remove and a change resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 14, vec![module.clone()])),
        Transaction::UserTransaction(change_resource_txn(&sender, 15, vec![module.clone()])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert_eq!(
        output[1].status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    for out in output {
        executor.apply_write_set(out.write_set());
    }

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 16, vec![module]);
    let output = executor.execute_transaction(borrow_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::MISSING_DATA,
    );
    executor.apply_write_set(output.write_set());
}

fn add_module_txn(sender: &AccountData, seq_num: u64) -> (VerifiedModule, SignedTransaction) {
    let module_code = String::from(
        "
        module M {
            resource T1 { v: u64 }

            public borrow_t1() acquires T1 {
                let t1: &Self.T1;
                t1 = borrow_global<T1>(get_txn_sender());
                return;
            }

            public change_t1(v: u64) acquires T1 {
                let t1: &mut Self.T1;
                t1 = borrow_global_mut<T1>(get_txn_sender());
                *&mut move(t1).v = move(v);
                return;
            }

            public remove_t1() acquires T1 {
                let v: u64;
                T1 { v } = move_from<T1>(get_txn_sender());
                return;
            }

            public publish_t1() {
                move_to_sender<T1>(T1 { v: 3 });
                return;
            }
        }
        ",
    );

    let compiler = Compiler {
        address: *sender.address(),
        ..Compiler::default()
    };
    let module = compiler
        .into_compiled_module("file_name", module_code.as_str())
        .expect("Module compilation failed");
    let mut module_blob = vec![];
    module
        .serialize(&mut module_blob)
        .expect("Module must serialize");
    let verified_module = VerifiedModule::new(module).expect("Module must verify");
    (
        verified_module,
        sender.account().create_signed_txn_impl(
            *sender.address(),
            TransactionPayload::Module(Module::new(module_blob)),
            seq_num,
            100_000,
            1,
            LBR_NAME.to_owned(),
        ),
    )
}

fn add_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<VerifiedModule>,
) -> SignedTransaction {
    let program = format!(
        "
            import 0x{}.M;

            main() {{
                M.publish_t1();
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_address(sender.address(), "file_name", &program, extra_deps);
    sender.account().create_signed_txn_impl(
        *sender.address(),
        module,
        seq_num,
        100_000,
        1,
        LBR_NAME.to_owned(),
    )
}

fn remove_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<VerifiedModule>,
) -> SignedTransaction {
    let program = format!(
        "
            import 0x{}.M;

            main() {{
                M.remove_t1();
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_address(sender.address(), "file_name", &program, extra_deps);
    sender.account().create_signed_txn_impl(
        *sender.address(),
        module,
        seq_num,
        100_000,
        1,
        LBR_NAME.to_owned(),
    )
}

fn borrow_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<VerifiedModule>,
) -> SignedTransaction {
    let program = format!(
        "
            import 0x{}.M;

            main() {{
                M.borrow_t1();
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_address(sender.address(), "file_name", &program, extra_deps);
    sender.account().create_signed_txn_impl(
        *sender.address(),
        module,
        seq_num,
        100_000,
        1,
        LBR_NAME.to_owned(),
    )
}

fn change_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<VerifiedModule>,
) -> SignedTransaction {
    let program = format!(
        "
            import 0x{}.M;

            main() {{
                M.change_t1(20);
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_address(sender.address(), "file_name", &program, extra_deps);
    sender.account().create_signed_txn_impl(
        *sender.address(),
        module,
        seq_num,
        100_000,
        1,
        LBR_NAME.to_owned(),
    )
}
