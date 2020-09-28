// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::{
    account::{self, Account, AccountData},
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
};
use libra_types::vm_status::{KeptVMStatus, StatusCode, VMStatus};
use libra_vm::{
    data_cache::StateViewCache, logger::NoLogLogger, transaction_metadata::TransactionMetadata,
    LibraVM,
};
use move_core_types::gas_schedule::{GasAlgebra, GasPrice, GasUnits};
use move_vm_types::gas_schedule::zero_cost_schedule;

#[test]
fn failed_transaction_cleanup_test() {
    let mut fake_executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    fake_executor.add_account_data(&sender);

    let logger = NoLogLogger;
    let libra_vm = LibraVM::new(fake_executor.get_state_view());
    let data_cache = StateViewCache::new(fake_executor.get_state_view());

    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = *sender.address();
    txn_data.max_gas_amount = GasUnits::new(100_000);
    txn_data.gas_unit_price = GasPrice::new(0);
    txn_data.sequence_number = 10;

    let gas_left = GasUnits::new(10_000);
    let gas_schedule = zero_cost_schedule();

    // TYPE_MISMATCH should be kept and charged.
    let out1 = libra_vm.failed_transaction_cleanup(
        VMStatus::Error(StatusCode::TYPE_MISMATCH),
        &gas_schedule,
        gas_left,
        &txn_data,
        &data_cache,
        &account::lbr_currency_code(),
        &logger,
    );
    assert!(!out1.write_set().is_empty());
    assert_eq!(out1.gas_used(), 90_000);
    assert!(!out1.status().is_discarded());
    assert_eq!(
        out1.status().status(),
        // StatusCode::TYPE_MISMATCH
        Ok(KeptVMStatus::MiscellaneousError)
    );

    // Invariant violations should be discarded and not charged.
    let out2 = libra_vm.failed_transaction_cleanup(
        VMStatus::Error(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR),
        &gas_schedule,
        gas_left,
        &txn_data,
        &data_cache,
        &account::lbr_currency_code(),
        &logger,
    );
    assert!(out2.write_set().is_empty());
    assert!(out2.gas_used() == 0);
    assert!(out2.status().is_discarded());
    assert_eq!(
        out2.status().status(),
        Err(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
    );
}

#[test]
fn non_existent_sender() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sequence_number = 0;
    let sender = Account::new();
    let receiver = AccountData::new(100_000, sequence_number);
    executor.add_account_data(&receiver);

    let transfer_amount = 10;
    let txn = peer_to_peer_txn(
        &sender,
        receiver.account(),
        sequence_number,
        transfer_amount,
    );

    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status().status(),
        Err(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST),
    );
}
