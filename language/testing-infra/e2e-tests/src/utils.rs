// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{
    account::Account,
    compile,
    executor::{self, FakeExecutor},
};
use transaction_builder::*;
use vm::file_format::CompiledModule;

pub fn close_module_publishing(
    executor: &mut FakeExecutor,
    dr_account: &Account,
    dr_seqno: &mut u64,
) {
    let compiled_script = {
        let script = "
            import 0x1.DiemTransactionPublishingOption;
        main(config: signer) {
            DiemTransactionPublishingOption.set_open_module(&config, false);
            return;
        }
        ";
        compile::compile_script_with_address(dr_account.address(), "file_name", script, vec![])
    };

    let txn = dr_account
        .transaction()
        .script(compiled_script)
        .sequence_number(*dr_seqno)
        .sign();

    executor.execute_and_apply(txn);
    *dr_seqno = dr_seqno.checked_add(1).unwrap();
}

pub fn start_with_released_df() -> (FakeExecutor, Account, Account, Account) {
    let executor = FakeExecutor::from_saved_genesis(executor::RELEASE_1_1_GENESIS);
    let mut dd_account = Account::new_testing_dd();
    let mut dr_account = Account::new_diem_root();
    let mut tc_account = Account::new_blessed_tc();

    dd_account.rotate_key(
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    );
    dr_account.rotate_key(
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    );
    tc_account.rotate_key(
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
        bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    );
    (executor, dr_account, tc_account, dd_account)
}

pub fn upgrade_df(
    executor: &mut FakeExecutor,
    dr_account: &Account,
    dr_seqno: &mut u64,
    update_version_number: Option<u64>,
) {
    close_module_publishing(executor, dr_account, dr_seqno);
    for compiled_module_bytes in
        compiled_stdlib::stdlib_modules(compiled_stdlib::StdLibOptions::Compiled).bytes_vec()
    {
        let compiled_module_id = CompiledModule::deserialize(&compiled_module_bytes)
            .unwrap()
            .self_id();
        executor.add_module(&compiled_module_id, compiled_module_bytes);
    }

    if let Some(version_number) = update_version_number {
        executor.execute_and_apply(
            dr_account
                .transaction()
                .script(encode_update_diem_version_script(0, version_number))
                .sequence_number(*dr_seqno)
                .sign(),
        );
        *dr_seqno = dr_seqno.checked_add(1).unwrap();
    }
}
