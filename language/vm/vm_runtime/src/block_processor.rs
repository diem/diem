// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::{
        module_adapter::ModuleFetcherImpl,
        module_cache::{BlockModuleCache, ModuleCache, VMModuleCache},
        script_cache::ScriptCache,
    },
    counters,
    data_cache::BlockDataCache,
    process_txn::{execute::ExecutedTransaction, validate::ValidationMode, ProcessTransaction},
};
use config::config::VMPublishingOption;
use logger::prelude::*;
use state_view::StateView;
use types::{
    transaction::{SignedTransaction, TransactionOutput, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
    write_set::WriteSet,
};
use vm_cache_map::Arena;

pub fn execute_block<'alloc>(
    txn_block: Vec<SignedTransaction>,
    code_cache: &VMModuleCache<'alloc>,
    script_cache: &ScriptCache<'alloc>,
    data_view: &dyn StateView,
    publishing_option: &VMPublishingOption,
) -> Vec<TransactionOutput> {
    trace!("[VM] Execute block, transaction count: {}", txn_block.len());

    let mode = if data_view.is_genesis() {
        // The genesis transaction must be in a block of its own.
        if txn_block.len() != 1 {
            // XXX Need a way to return that an entire block failed.
            return txn_block
                .iter()
                .map(|_| {
                    TransactionOutput::new(
                        WriteSet::default(),
                        vec![],
                        0,
                        TransactionStatus::from(VMStatus::Validation(
                            VMValidationStatus::RejectedWriteSet,
                        )),
                    )
                })
                .collect();
        } else {
            ValidationMode::Genesis
        }
    } else {
        ValidationMode::Executing
    };

    let module_cache = BlockModuleCache::new(code_cache, ModuleFetcherImpl::new(data_view));
    let mut data_cache = BlockDataCache::new(data_view);
    let mut result = vec![];
    for txn in txn_block.into_iter() {
        let output = transaction_flow(
            txn,
            &module_cache,
            script_cache,
            &data_cache,
            mode,
            publishing_option,
        );
        data_cache.push_write_set(&output.write_set());
        result.push(output);
    }
    trace!("[VM] Execute block finished");
    result
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
    txn: SignedTransaction,
    module_cache: P,
    script_cache: &ScriptCache<'alloc>,
    data_cache: &BlockDataCache<'_>,
    mode: ValidationMode,
    publishing_option: &VMPublishingOption,
) -> TransactionOutput
where
    P: ModuleCache<'alloc>,
{
    let arena = Arena::new();
    let process_txn = ProcessTransaction::new(txn, &module_cache, data_cache, &arena);

    let validated_txn = match process_txn.validate(mode, publishing_option) {
        Ok(validated_txn) => validated_txn,
        Err(vm_status) => {
            counters::FAILED_TRANSACTION.inc();
            return ExecutedTransaction::discard_error_output(vm_status);
        }
    };
    let verified_txn = match validated_txn.verify() {
        Ok(verified_txn) => verified_txn,
        Err(vm_status) => {
            counters::FAILED_TRANSACTION.inc();
            return ExecutedTransaction::discard_error_output(vm_status);
        }
    };
    let executed_txn = verified_txn.execute(script_cache);

    // On success, publish the modules into the cache so that future transactions can refer to them
    // directly.
    let output = executed_txn.into_output();
    match output.status() {
        TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)) => {
            module_cache.reclaim_cached_module(arena.into_vec());
        }
        _ => counters::FAILED_TRANSACTION.inc(),
    };
    output
}
