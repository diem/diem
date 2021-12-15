// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::components::apply_chunk_output::ApplyChunkOutput;
use anyhow::Result;
use diem_logger::trace;
use diem_types::transaction::{Transaction, TransactionOutput};
use diem_vm::VMExecutor;
use executor_types::{ExecutedChunk, ExecutedTrees};
use fail::fail_point;
use storage_interface::state_view::{StateCache, VerifiedStateView};

pub struct ChunkOutput {
    /// Input transactions.
    pub transactions: Vec<Transaction>,
    /// Raw VM output.
    pub transaction_outputs: Vec<TransactionOutput>,
    /// Carries the frozen base state view, so all in-mem nodes involved won't drop before the
    /// execution result is processed; as well as al the accounts touched during execution, together
    /// with their proofs.
    pub state_cache: StateCache,
}

impl ChunkOutput {
    pub fn by_transaction_execution<V: VMExecutor>(
        transactions: Vec<Transaction>,
        state_view: VerifiedStateView,
    ) -> Result<Self> {
        let transaction_outputs = V::execute_block(transactions.clone(), &state_view)?;

        Ok(Self {
            transactions,
            transaction_outputs,
            state_cache: state_view.into_state_cache(),
        })
    }

    pub fn by_transaction_output(
        transactions_and_outputs: Vec<(Transaction, TransactionOutput)>,
        state_view: VerifiedStateView,
    ) -> Result<Self> {
        let (transactions, transaction_outputs): (Vec<_>, Vec<_>) =
            transactions_and_outputs.into_iter().unzip();

        // prime the state cache by fetching all touched accounts
        let write_sets: Vec<_> = transaction_outputs.iter().map(|o| o.write_set()).collect();
        state_view.prime_cache_by_write_set(&write_sets)?;

        Ok(Self {
            transactions,
            transaction_outputs,
            state_cache: state_view.into_state_cache(),
        })
    }

    pub fn apply_to_ledger(
        self,
        base_view: &ExecutedTrees,
    ) -> Result<(ExecutedChunk, Vec<Transaction>, Vec<Transaction>)> {
        fail_point!("executor::vm_execute_chunk", |_| {
            Err(anyhow::anyhow!("Injected error in apply_to_ledger."))
        });
        ApplyChunkOutput::apply(self, base_view)
    }

    pub fn trace_log_transaction_status(&self) {
        let status: Vec<_> = self
            .transaction_outputs
            .iter()
            .map(TransactionOutput::status)
            .cloned()
            .collect();

        if !status.is_empty() {
            trace!("Execution status: {:?}", status);
        }
    }
}
