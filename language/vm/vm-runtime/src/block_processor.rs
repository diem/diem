// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::{module_cache::ModuleCache, script_cache::ScriptCache},
    counters::*,
    data_cache::BlockDataCache,
    gas_meter::load_gas_schedule,
    process_txn::{execute::ExecutedTransaction, validate::ValidationMode, ProcessTransaction},
};
use libra_config::config::VMPublishingOption;
use libra_logger::prelude::*;
use libra_types::{
    transaction::{
        SignatureCheckedTransaction, SignedTransaction, TransactionOutput, TransactionStatus,
    },
    vm_error::{sub_status, StatusCode, VMStatus},
};
use rayon::prelude::*;
use vm::gas_schedule::CostTable;
use vm_cache_map::Arena;

pub fn execute_user_transaction_block<'alloc, P>(
    txn_block: Vec<SignedTransaction>,
    module_cache: P,
    script_cache: &ScriptCache<'alloc>,
    data_cache: &mut BlockDataCache<'_>,
    publishing_option: &VMPublishingOption,
) -> Result<Vec<TransactionOutput>, VMStatus>
where
    P: ModuleCache<'alloc>,
{
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
    let gas_schedule = match load_gas_schedule(&module_cache, &data_cache) {
        // TODO/XXX: This is a hack to get around not having proper writesets yet. Once that gets
        // in remove this line.
        Err(_) if data_view.is_genesis() => CostTable::zero(),
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
                        &module_cache,
                        script_cache,
                        data_cache,
                        mode,
                        publishing_option,
                        &gas_schedule,
                    ),
                    Err(vm_status) => ExecutedTransaction::discard_error_output(vm_status),
                };
                report_execution_status(output.status());
                data_cache.push_write_set(&output.write_set());

                // `result` is initally empty, a single element is pushed per loop iteration and
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
fn transaction_flow<'alloc, P>(
    txn: SignatureCheckedTransaction,
    module_cache: P,
    script_cache: &ScriptCache<'alloc>,
    data_cache: &BlockDataCache<'_>,
    mode: ValidationMode,
    publishing_option: &VMPublishingOption,
    gas_schedule: &CostTable,
) -> TransactionOutput
where
    P: ModuleCache<'alloc>,
{
    let arena = Arena::new();
    let process_txn = ProcessTransaction::new(txn, gas_schedule, &module_cache, data_cache, &arena);

    let validated_txn = record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
    match process_txn.validate(mode, publishing_option) {
        Ok(validated_txn) => validated_txn,
        Err(vm_status) => {
            return ExecutedTransaction::discard_error_output(vm_status);
        }
    }
    }
    };

    let verified_txn = record_stats! {time_hist | TXN_VERIFICATION_TIME_TAKEN | {
     match validated_txn.verify(script_cache) {
        Ok(verified_txn) => verified_txn,
        Err(vm_status) => {
            return ExecutedTransaction::discard_error_output(vm_status);
        }
    }
    }
    };

    let executed_txn = record_stats! {time_hist | TXN_EXECUTION_TIME_TAKEN | {
        verified_txn.execute()
        }
    };

    // On success, publish the modules into the cache so that future transactions can refer to them
    // directly.
    let output = executed_txn.into_output();
    match output.status() {
        TransactionStatus::Keep(status) if status.major_status == StatusCode::EXECUTED => {
            module_cache.reclaim_cached_module(arena.into_vec());
        }
        _ => (),
    };
    output
}
