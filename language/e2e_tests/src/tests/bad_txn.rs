// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountData, executor::FakeExecutor};
use types::{
    transaction::{Module, Script, SignedTransaction, TransactionPayload, TransactionStatus},
    vm_error::{VMStatus, VMValidationStatus},
};

#[test]
fn module_script_disabled() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();

    // create and publish a sender
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let module = TransactionPayload::Module(Module::new(vec![]));
    let mod_txn = sender.account().create_signed_txn(module, 10, 0, 0);
    let mod_txns: Vec<SignedTransaction> = vec![mod_txn];
    let mod_output = executor.execute_block(mod_txns);
    assert_eq!(
        mod_output[0].status(),
        &TransactionStatus::Discard(VMStatus::Validation(VMValidationStatus::UnknownModule)),
    );

    let script = TransactionPayload::Script(Script::new(vec![], vec![]));
    let script_txn = sender.account().create_signed_txn(script, 10, 0, 0);
    let script_txns: Vec<SignedTransaction> = vec![script_txn];
    let script_output = executor.execute_block(script_txns);
    assert_eq!(
        script_output[0].status(),
        &TransactionStatus::Discard(VMStatus::Validation(VMValidationStatus::UnknownScript)),
    );
}
