// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountData, executor::FakeExecutor};
use libra_types::vm_error::{StatusCode, VMStatus};
use libra_vm::LibraVM;
use move_vm_state::data_cache::BlockDataCache;
use vm::{
    gas_schedule::{GasAlgebra, GasPrice, GasUnits},
    transaction_metadata::TransactionMetadata,
};

#[test]
fn failed_transaction_cleanup_test() {
    let mut fake_executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000, 10);
    fake_executor.add_account_data(&sender);

    let mut libra_vm = LibraVM::new(fake_executor.config());
    let mut data_cache = BlockDataCache::new(fake_executor.get_state_view());
    libra_vm.load_configs(fake_executor.get_state_view());

    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = *sender.address();
    txn_data.max_gas_amount = GasUnits::new(100_000);
    txn_data.gas_unit_price = GasPrice::new(2);

    let gas_left = GasUnits::new(10_000);

    // TYPE_MISMATCH should be kept and charged.
    let out1 = libra_vm.failed_transaction_cleanup(
        VMStatus::new(StatusCode::TYPE_MISMATCH),
        gas_left,
        &txn_data,
        &mut data_cache,
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
        gas_left,
        &txn_data,
        &mut data_cache,
    );
    assert!(out2.write_set().is_empty());
    assert!(out2.gas_used() == 0);
    assert!(out2.status().is_discarded());
    assert_eq!(
        out2.status().vm_status().major_status,
        StatusCode::OUT_OF_BOUNDS_INDEX
    );
}
