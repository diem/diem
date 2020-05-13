// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use libra_vm::{data_cache::StateViewCache, LibraVM};
use move_core_types::gas_schedule::{GasAlgebra, GasPrice, GasUnits};
use move_vm_types::{gas_schedule::zero_cost_schedule, transaction_metadata::TransactionMetadata};

#[test]
fn failed_transaction_cleanup_test() {
    let mut fake_executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    fake_executor.add_account_data(&sender);

    let mut libra_vm = LibraVM::new();
    let mut data_cache = StateViewCache::new(fake_executor.get_state_view());
    libra_vm.load_configs(fake_executor.get_state_view());

    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = *sender.address();
    txn_data.max_gas_amount = GasUnits::new(100_000);
    txn_data.gas_unit_price = GasPrice::new(2);

    let gas_schedule = zero_cost_schedule();
    let gas_left = GasUnits::new(10_000);

    // TYPE_MISMATCH should be kept and charged.
    let out1 = libra_vm.failed_transaction_cleanup(
        VMStatus::new(StatusCode::TYPE_MISMATCH),
        &gas_schedule,
        gas_left,
        &txn_data,
        &mut data_cache,
        &account::lbr_currency_code(),
    );
    assert!(!out1.write_set().is_empty());
    assert!(out1.gas_used() == 180_000);
    assert!(!out1.status().is_discarded());
    assert_eq!(
        out1.status().vm_status().major_status,
        StatusCode::TYPE_MISMATCH
    );

    // OUT_OF_BOUNDS_INDEX should be discarded and not charged.
    let out2 = libra_vm.failed_transaction_cleanup(
        VMStatus::new(StatusCode::OUT_OF_BOUNDS_INDEX),
        &gas_schedule,
        gas_left,
        &txn_data,
        &mut data_cache,
        &account::lbr_currency_code(),
    );
    assert!(out2.write_set().is_empty());
    assert!(out2.gas_used() == 0);
    assert!(out2.status().is_discarded());
    assert_eq!(
        out2.status().vm_status().major_status,
        StatusCode::OUT_OF_BOUNDS_INDEX
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
        output.status().vm_status().major_status,
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST,
    );
}
