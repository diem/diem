// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::process_txn::verify::VerifiedTransaction;
use crate::runtime::VMRuntime;
use crate::txn_executor::TransactionExecutor;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    transaction::{TransactionOutput, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};

/// Represents a transaction that has been executed.
pub struct ExecutedTransaction {
    output: TransactionOutput,
}

impl ExecutedTransaction {
    /// Creates a new instance by executing this transaction.
    pub fn new(
        verified_txn: VerifiedTransaction,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        executor: &mut TransactionExecutor,
    ) -> Self {
        let output = execute(verified_txn, runtime, state_view, executor);
        Self { output }
    }

    /// Returns the `TransactionOutput` for this transaction.
    pub fn into_output(self) -> TransactionOutput {
        self.output
    }
}

fn execute(
    verified_txn: VerifiedTransaction,
    runtime: &VMRuntime,
    state_view: &dyn StateView,
    executor: &mut TransactionExecutor,
) -> TransactionOutput {
    match verified_txn
        .into_inner()
        .into_raw_transaction()
        .into_payload()
    {
        TransactionPayload::Program => {
            ExecutedTransaction::discard_error_output(VMStatus::new(StatusCode::MALFORMED))
        }
        // WriteSet transaction. Just proceed and use the writeset as output.
        TransactionPayload::WriteSet(write_set_payload) => {
            let (write_set, events) = write_set_payload.into_inner();
            TransactionOutput::new(
                write_set,
                events,
                0,
                VMStatus::new(StatusCode::EXECUTED).into(),
            )
        }
        TransactionPayload::Module(module) => {
            let module = module.into_inner();
            let module_id = match executor.publish_module(&module, runtime, state_view) {
                Ok(module_id) => module_id,
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error publishing module: {:?}", err);
                        return ExecutedTransaction::discard_error_output(err);
                    }
                    _ => {
                        warn!("[VM] User error publishing module: {:?}", err);
                        return executor.failed_transaction_cleanup(runtime, state_view, Err(err));
                    }
                },
            };

            executor.transaction_cleanup(runtime, state_view, vec![(module_id, module)])
            // TODO: should a module be published eagerly? this is only going to the WriteSet now
        }
        TransactionPayload::Script(script) => {
            let (main, args) = script.into_inner();
            match executor.execute_script(runtime, state_view, main, args) {
                Ok(_) => executor.transaction_cleanup(runtime, state_view, vec![]),
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error running script: {:?}", err);
                        ExecutedTransaction::discard_error_output(err)
                    }
                    _ => {
                        warn!("[VM] User error running script: {:?}", err);
                        executor.failed_transaction_cleanup(runtime, state_view, Err(err))
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
