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
    vm_status::{StatusCode, VMStatus},
};
use libra_vm::LibraVM;
use transaction_builder::encode_update_travel_rule_limit;

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
        1,
        TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    );
    executor.new_block();
    executor.execute_and_apply(txn);

    vm.load_configs(executor.get_state_view());
    assert_eq!(
        vm.internals().libra_version().unwrap(),
        LibraVersion { major: 2 }
    );
}

// Testsupdate_travel_rule_limit.move DualAttestionLimit
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
    let output = executor.execute_and_apply(
        blessed.signed_script_txn(encode_update_travel_rule_limit(1, new_micro_lbr_limit), 0),
    );
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );

    // higher transaction works with higher limit
    let transfer_amount = 1_000_010;
    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);
    let output = executor.execute_and_apply(txn);
    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
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
