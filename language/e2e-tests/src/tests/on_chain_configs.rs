// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
    transaction_status_eq,
};
use compiled_stdlib::transaction_scripts::StdlibScript;
use libra_types::{
    account_config::LBR_NAME,
    on_chain_config::LibraVersion,
    transaction::{TransactionArgument, TransactionStatus},
    vm_status::KeptVMStatus,
};
use libra_vm::LibraVM;
use transaction_builder::encode_update_dual_attestation_limit_script;

#[test]
fn initial_libra_version() {
    let mut executor = FakeExecutor::from_genesis_file();
    let vm = LibraVM::new(executor.get_state_view());

    assert_eq!(
        vm.internals().libra_version().unwrap(),
        LibraVersion { major: 1 }
    );

    let account = Account::new_genesis_account(libra_types::on_chain_config::config_address());
    let txn = account.create_signed_txn_with_args(
        StdlibScript::UpdateLibraVersion.compiled_bytes().into_vec(),
        vec![],
        vec![TransactionArgument::U64(2)],
        1,
        TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    );
    executor.new_block();
    executor.execute_and_apply(txn);

    let new_vm = LibraVM::new(executor.get_state_view());
    assert_eq!(
        new_vm.internals().libra_version().unwrap(),
        LibraVersion { major: 2 }
    );
}

#[test]
fn drop_txn_after_reconfiguration() {
    let mut executor = FakeExecutor::from_genesis_file();
    let vm = LibraVM::new(executor.get_state_view());

    assert_eq!(
        vm.internals().libra_version().unwrap(),
        LibraVersion { major: 1 }
    );

    let account = Account::new_genesis_account(libra_types::on_chain_config::config_address());
    let txn = account.create_signed_txn_with_args(
        StdlibScript::UpdateLibraVersion.compiled_bytes().into_vec(),
        vec![],
        vec![TransactionArgument::U64(2)],
        1,
        TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    );
    executor.new_block();

    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    let txn2 = peer_to_peer_txn(&sender.account(), &receiver.account(), 11, 1000);

    let mut output = executor.execute_block(vec![txn, txn2]).unwrap();
    assert_eq!(output.pop().unwrap().status(), &TransactionStatus::Retry)
}

#[test]
fn updated_limit_allows_txn() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let blessed = Account::new_blessed_tc();
    // create and publish a sender with 5_000_000 coins and a receiver with 0 coins
    let sender = AccountData::new(5_000_000, 10);
    let receiver = AccountData::new(0, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    // Execute updated dual attestation limit
    let new_micro_lbr_limit = 1_000_011;
    let output = executor.execute_and_apply(blessed.signed_script_txn(
        encode_update_dual_attestation_limit_script(3, new_micro_lbr_limit),
        0,
    ));
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    // higher transaction works with higher limit
    let transfer_amount = 1_000_010;
    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);
    let output = executor.execute_and_apply(txn);
    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    ));
    let sender_balance = executor
        .read_balance_resource(sender.account(), account::lbr_currency_code())
        .expect("sender balance must exist");
    let receiver_balance = executor
        .read_balance_resource(receiver.account(), account::lbr_currency_code())
        .expect("receiver balcne must exist");

    assert_eq!(3_999_990, sender_balance.coin());
    assert_eq!(1_000_010, receiver_balance.coin());
}
