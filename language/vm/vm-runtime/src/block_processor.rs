// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    data_cache::BlockDataCache,
    process_txn::{
        execute::ExecutedTransaction,
        validate::{ValidatedTransaction, ValidationMode},
    },
    runtime::VMRuntime,
    txn_executor::TransactionExecutor,
};
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    transaction::{SignatureCheckedTransaction, SignedTransaction, TransactionOutput},
    vm_error::{sub_status, StatusCode, VMStatus},
};
use rayon::prelude::*;
use vm::{gas_schedule::CostTable, transaction_metadata::TransactionMetadata};

/// Executes a block of `UserTransaction`.
pub fn execute_user_transaction_block(
    txn_block: Vec<SignedTransaction>,
    runtime: &VMRuntime,
    data_cache: &mut BlockDataCache,
    state_view: &dyn StateView,
) -> Result<Vec<TransactionOutput>, VMStatus> {
    trace!("[VM] Execute block, transaction count: {}", txn_block.len());
    report_block_count(txn_block.len());

    let mode = if data_cache.is_genesis() {
        // The genesis transaction must be in a block of its own.
        if txn_block.len() != 1 {
            return Err(VMStatus::new(StatusCode::REJECTED_WRITE_SET));
        } else {
            ValidationMode::Genesis
        }
    } else {
        ValidationMode::Executing
    };

    let mut result = vec![];

    // If we fail to load the gas schedule, then we fail to process the block.
    let gas_schedule = match runtime.load_gas_schedule(data_cache, state_view) {
        // TODO/XXX: This is a hack to get around not having proper writesets yet. Once that gets
        // in remove this line.
        Err(_) if data_cache.is_genesis() => CostTable::zero(),
        Err(_) => {
            return Err(VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND))
        }
        Ok(cost_table) => cost_table,
    };

    let signature_verified_block: Vec<Result<SignatureCheckedTransaction, VMStatus>> = txn_block
        .into_par_iter()
        .map(|txn| {
            txn.check_signature()
                .map_err(|_| VMStatus::new(StatusCode::INVALID_SIGNATURE))
        })
        .collect();

    for transaction in signature_verified_block {
        record_stats! {time_hist | TXN_TOTAL_TIME_TAKEN | {
                let output = match transaction {
                    Ok(t) => transaction_flow(
                        t,
                        runtime,
                        &gas_schedule,
                        data_cache,
                        state_view,
                        mode,
                    ),
                    Err(vm_status) => ExecutedTransaction::discard_error_output(vm_status),
                };
                report_execution_status(output.status());
                data_cache.push_write_set(&output.write_set());

                // `result` is initially empty, a single element is pushed per loop iteration and
                // the number of iterations is bound to the max size of `signature_verified_block`
                assume!(result.len() < usize::max_value());
                result.push(output);

            }
        }
    }
    trace!("[VM] Execute block finished");
    Ok(result)
}

/// Process a transaction and emit a TransactionOutput.
///
/// A successful execution will have `TransactionStatus::Keep` in the TransactionOutput and a
/// non-empty writeset. There are two possibilities for a failed transaction. If a verification or
/// runtime error occurs, the TransactionOutput will have `TransactionStatus::Keep` and a writeset
/// that only contains the charged gas of this transaction. If a validation or `InvariantViolation`
/// error occurs, the TransactionOutput will have `TransactionStatus::Discard` and an empty
/// writeset.
///
/// Note that this function DO HAVE side effect. If a transaction tries to publish some module,
/// and this transaction is executed successfully, this function will update `module_cache` to
/// include those newly published modules. This function will also update the `script_cache` to
/// cache this `txn`
fn transaction_flow(
    txn: SignatureCheckedTransaction,
    runtime: &VMRuntime,
    gas_schedule: &CostTable,
    data_cache: &BlockDataCache,
    state_view: &dyn StateView,
    mode: ValidationMode,
) -> TransactionOutput {
    let validated_txn = record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
        match ValidatedTransaction::new(
            txn,
            runtime,
            state_view,
            gas_schedule,
            data_cache,
            mode,
        ) {
            Ok(validated_txn) => validated_txn,
            Err(vm_status) => {
                return ExecutedTransaction::discard_error_output(vm_status);
            }
        }
    }};

    let txn_data = TransactionMetadata::new(validated_txn.as_inner());
    let verified_txn = record_stats! {time_hist | TXN_VERIFICATION_TIME_TAKEN | {
         match validated_txn.verify(&txn_data) {
            Ok(verified_txn) => verified_txn,
            Err(vm_status) => {
                return ExecutedTransaction::discard_error_output(vm_status);
            }
        }
    }};

    let mut txn_executor = TransactionExecutor::new(gas_schedule, data_cache, txn_data);
    let executed_txn = record_stats! {time_hist | TXN_EXECUTION_TIME_TAKEN | {
        verified_txn.execute(runtime, state_view, &mut txn_executor)
    }};

    // On success, publish the modules into the cache so that future transactions can refer to them
    // directly.
    executed_txn.into_output()
}
