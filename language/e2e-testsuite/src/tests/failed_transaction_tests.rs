// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::vm_status::{KeptVMStatus, StatusCode, VMStatus};
use diem_vm::{data_cache::StateViewCache, transaction_metadata::TransactionMetadata, DiemVM};
use language_e2e_tests::{
    account, common_transactions::peer_to_peer_txn, current_function_name, executor::FakeExecutor,
};
use move_core_types::gas_schedule::{GasAlgebra, GasPrice, GasUnits};
use move_vm_runtime::logging::NoContextLog;
use move_vm_types::gas_schedule::zero_cost_schedule;

#[test]
fn failed_transaction_cleanup_test() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    let log_context = NoContextLog::new();
    let diem_vm = DiemVM::new(executor.get_state_view());
    let data_cache = StateViewCache::new(executor.get_state_view());

    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = *sender.address();
    txn_data.max_gas_amount = GasUnits::new(100_000);
    txn_data.gas_unit_price = GasPrice::new(0);
    txn_data.sequence_number = 10;

    let gas_left = GasUnits::new(10_000);
    let gas_schedule = zero_cost_schedule();

    // TYPE_MISMATCH should be kept and charged.
    let out1 = diem_vm.failed_transaction_cleanup(
        VMStatus::Error(StatusCode::TYPE_MISMATCH),
        &gas_schedule,
        gas_left,
        &txn_data,
        &data_cache,
        &account::xus_currency_code(),
        &log_context,
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
    let out2 = diem_vm.failed_transaction_cleanup(
        VMStatus::Error(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR),
        &gas_schedule,
        gas_left,
        &txn_data,
        &data_cache,
        &account::xus_currency_code(),
        &log_context,
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
    executor.set_golden_file(current_function_name!());
    let sequence_number = 0;
    let sender = executor.create_raw_account();
    let receiver = executor.create_raw_account_data(100_000, sequence_number);
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
