// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use compiler::Compiler;
use diem_types::{
    account_config,
    on_chain_config::DiemVersion,
    transaction::{Script, TransactionStatus, WriteSetPayload},
    vm_status::KeptVMStatus,
};
use diem_vm::DiemVM;
use diem_writeset_generator::build_changeset;
use language_e2e_tests::{
    account::Account, compile::compile_module_with_address, executor::FakeExecutor,
};

#[test]
fn build_upgrade_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    executor.new_block();

    // create a transaction trying to publish a new module.
    let genesis_account = Account::new_diem_root();

    let program = String::from(
        "
        module M {
            public magic(): u64 { return 42; }
        }
        ",
    );

    let module =
        compile_module_with_address(&account_config::CORE_CODE_ADDRESS, "file_name", &program).0;

    let change_set = build_changeset(
        executor.get_state_view(),
        |session| {
            session.set_diem_version(11);
        },
        &[module.clone()],
    );

    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(change_set))
        .sequence_number(1)
        .sign();

    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(executor.verify_transaction(writeset_txn).status().is_none());

    executor.apply_write_set(output.write_set());

    let new_vm = DiemVM::new(executor.get_state_view());
    assert_eq!(
        new_vm.internals().diem_version().unwrap(),
        DiemVersion { major: 11 }
    );

    let script_body = {
        let code = r#"
import 0x1.M;

main(lr_account: &signer) {
  assert(M.magic() == 42, 100);
  return;
}
"#;

        let compiler = Compiler {
            address: account_config::CORE_CODE_ADDRESS,
            extra_deps: vec![module],
            ..Compiler::default()
        };
        compiler
            .into_script_blob("file_name", code)
            .expect("Failed to compile")
    };

    let txn = genesis_account
        .transaction()
        .script(Script::new(script_body, vec![], vec![]))
        .sequence_number(2)
        .sign();

    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}
