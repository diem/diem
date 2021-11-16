// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod read_write_set_analyzer;
mod storage_wrapper;
mod vm_wrapper;

use crate::{
    adapter_common::{preprocess_transaction, PreprocessedTransaction},
    data_cache::RemoteStorage,
    diem_vm::DiemVM,
    parallel_executor::{
        read_write_set_analyzer::ReadWriteSetAnalysisWrapper, vm_wrapper::DiemVMWrapper,
    },
};
use diem_logger::prelude::*;
use diem_parallel_executor::{
    errors::Error,
    executor::ParallelTransactionExecutor,
    task::{Transaction as PTransaction, TransactionOutput as PTransactionOutput},
};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    write_set::{WriteOp, WriteSet},
};
use move_core_types::vm_status::{StatusCode, VMStatus};
use rayon::prelude::*;
use read_write_set_dynamic::NormalizedReadWriteSetAnalysis;

impl PTransaction for PreprocessedTransaction {
    type Key = AccessPath;
    type Value = WriteOp;
}

// Wrapper to avoid orphan rule
pub(crate) struct DiemTransactionOutput(TransactionOutput);

impl DiemTransactionOutput {
    pub fn new(output: TransactionOutput) -> Self {
        Self(output)
    }
    pub fn into(self) -> TransactionOutput {
        self.0
    }
}

impl PTransactionOutput for DiemTransactionOutput {
    type T = PreprocessedTransaction;

    fn get_writes(&self) -> Vec<(AccessPath, WriteOp)> {
        self.0.write_set().iter().cloned().collect()
    }

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self {
        Self(TransactionOutput::new(
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Retry,
        ))
    }
}

pub struct ParallelDiemVM();

impl ParallelDiemVM {
    pub fn execute_block<S: StateView>(
        analysis_result: &NormalizedReadWriteSetAnalysis,
        transactions: Vec<Transaction>,
        state_view: &S,
    ) -> Result<(Vec<TransactionOutput>, Option<Error<VMStatus>>), VMStatus> {
        let blockchain_view = RemoteStorage::new(state_view);
        let analyzer = ReadWriteSetAnalysisWrapper::new(analysis_result, &blockchain_view);

        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.

        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction::<DiemVM>(txn.clone()))
            .collect();

        match ParallelTransactionExecutor::<
            PreprocessedTransaction,
            DiemVMWrapper<S>,
            ReadWriteSetAnalysisWrapper<RemoteStorage<S>>,
        >::new(analyzer)
        .execute_transactions_parallel(state_view, signature_verified_block)
        {
            Ok(results) => Ok((
                results
                    .into_iter()
                    .map(DiemTransactionOutput::into)
                    .collect(),
                None,
            )),
            Err(err @ Error::InferencerError) | Err(err @ Error::UnestimatedWrite) => {
                warn!("Parallel Execution Rollback {:?} for txns: {:?}", err, transactions);
                let output = DiemVM::execute_block_and_keep_vm_status(transactions, state_view)?;
                Ok((
                    output
                        .into_iter()
                        .map(|(_vm_status, txn_output)| txn_output)
                        .collect(),
                    Some(err),
                ))
            }
            Err(Error::InvariantViolation) => Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )),
            Err(Error::UserError(err)) => Err(err),
        }
    }
}
