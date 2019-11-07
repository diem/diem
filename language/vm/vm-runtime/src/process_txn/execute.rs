// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::module_cache::ModuleCache,
    process_txn::verify::{VerTxn, VerifiedTransaction, VerifiedTransactionState},
};
use libra_logger::prelude::*;
use libra_types::{
    transaction::{TransactionOutput, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use vm::{
    access::ModuleAccess,
    errors::{vm_error, Location},
};

/// Represents a transaction that has been executed.
pub struct ExecutedTransaction {
    output: TransactionOutput,
}

impl ExecutedTransaction {
    /// Creates a new instance by executing this transaction.
    pub fn new<'alloc, 'txn, P>(verified_txn: VerifiedTransaction<'alloc, 'txn, P>) -> Self
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        let output = execute(verified_txn);
        Self { output }
    }

    /// Returns the `TransactionOutput` for this transaction.
    pub fn into_output(self) -> TransactionOutput {
        self.output
    }
}

fn execute<'alloc, 'txn, P>(
    mut verified_txn: VerifiedTransaction<'alloc, 'txn, P>,
) -> TransactionOutput
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    let txn_state = verified_txn.take_state();

    match verified_txn
        .into_inner()
        .into_raw_transaction()
        .into_payload()
    {
        TransactionPayload::Program => {
            ExecutedTransaction::discard_error_output(VMStatus::new(StatusCode::MALFORMED))
        }
        // WriteSet transaction. Just proceed and use the writeset as output.
        TransactionPayload::WriteSet(write_set) => TransactionOutput::new(
            write_set,
            vec![],
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        ),
        TransactionPayload::Module(module) => {
            let VerifiedTransactionState {
                mut txn_executor,
                verified_txn,
            } = txn_state.expect("module transactions should always have associated state");
            let ver_module = match verified_txn {
                VerTxn::Module(ver_module) => ver_module,
                _ => unreachable!("TransactionPayload::Module expects VerTxn::Module"),
            };
            let module_id = ver_module.self_id();
            // Make sure that there is not already a module with this name published
            // under the transaction sender's account.
            // Note: `exists_module` will fetch the module from either the
            //       global storage or from the local data cache (which means that this module is
            //       published within the same block).
            if txn_executor.exists_module(&module_id) {
                return txn_executor.failed_transaction_cleanup(Err(vm_error(
                    Location::default(),
                    StatusCode::DUPLICATE_MODULE_NAME,
                )));
            };
            let module_bytes = module.into_inner();
            let output = txn_executor.transaction_cleanup(vec![(module_id, module_bytes)]);
            match output.status() {
                TransactionStatus::Keep(status) if status.major_status == StatusCode::EXECUTED => {
                    txn_executor.module_cache().cache_module(*ver_module);
                }
                _ => (),
            };
            output
        }
        TransactionPayload::Script(script) => {
            let VerifiedTransactionState {
                mut txn_executor,
                verified_txn,
            } = txn_state.expect("script-based transactions should always have associated state");
            let main = match verified_txn {
                VerTxn::Script(func) => func,
                _ => unreachable!("TransactionPayload::Script expects VerTxn::Program"),
            };

            let (_, args) = script.into_inner();
            match txn_executor.interpeter_entrypoint(main, args) {
                Ok(_) => txn_executor.transaction_cleanup(vec![]),
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error running script: {:?}", err);
                        ExecutedTransaction::discard_error_output(err)
                    }
                    _ => {
                        warn!("[VM] User error running script: {:?}", err);
                        txn_executor.failed_transaction_cleanup(Err(err))
                    }
                },
            }
        }
    }
}

impl ExecutedTransaction {
    #[inline]
    pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
        // Since this transaction will be discarded, no writeset will be included.
        TransactionOutput::new(
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Discard(err),
        )
    }
}
