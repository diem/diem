// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, executor::FakeExecutor, gas_costs::TXN_RESERVED};
use libra_types::{on_chain_config::LibraVersion, transaction::TransactionArgument};
use libra_vm::LibraVM;
use stdlib::transaction_scripts::StdlibScript;

#[test]
fn initial_libra_version() {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut vm = LibraVM::new();
    vm.load_configs(executor.get_state_view());

    assert_eq!(
        vm.internals().libra_version().unwrap(),
        LibraVersion { major: 1 }
    );

    let account = Account::new_genesis_account(libra_types::on_chain_config::config_address());
    let txn = account.create_signed_txn_with_args(
        StdlibScript::UpdateLibraVersion.compiled_bytes().into_vec(),
        vec![],
        vec![TransactionArgument::U64(2)],
        0,
        TXN_RESERVED,
        0,
    );
    executor.new_block();
    executor.execute_and_apply(txn);

    vm.load_configs(executor.get_state_view());
    assert_eq!(
        vm.internals().libra_version().unwrap(),
        LibraVersion { major: 2 }
    );
}
