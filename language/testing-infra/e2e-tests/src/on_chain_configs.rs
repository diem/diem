// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, executor::FakeExecutor};
use diem_framework_releases::legacy::transaction_scripts::LegacyStdlibScript;
use diem_types::{
    on_chain_config::DiemVersion,
    transaction::{Script, TransactionArgument},
};
use diem_vm::{DiemVM, DiemVM_};

pub fn set_diem_version(executor: &mut FakeExecutor, version: DiemVersion) {
    let account = Account::new_genesis_account(diem_types::on_chain_config::config_address());
    let txn = account
        .transaction()
        .script(Script::new(
            LegacyStdlibScript::UpdateDiemVersion
                .compiled_bytes()
                .into_vec(),
            vec![],
            vec![
                TransactionArgument::U64(0),
                TransactionArgument::U64(version.major),
            ],
        ))
        .sequence_number(1)
        .sign();
    executor.new_block();
    executor.execute_and_apply(txn);

    let new_vm = DiemVM_::new(executor.get_state_view());
    assert_eq!(new_vm.internals().diem_version().unwrap(), version);
}
