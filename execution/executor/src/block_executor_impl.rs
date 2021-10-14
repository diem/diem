// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::logging::{LogEntry, LogSchema};
use anyhow::format_err;
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_state_view::StateViewId;
use diem_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        Transaction, TransactionOutput, TransactionStatus, TransactionToCommit, Version,
    },
};
use diem_vm::VMExecutor;
use executor_types::{BlockExecutor, Error, StateComputeResult};
use fail::fail_point;

use crate::{
    metrics::{
        DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS, DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS,
        DIEM_EXECUTOR_SAVE_TRANSACTIONS_SECONDS, DIEM_EXECUTOR_TRANSACTIONS_SAVED,
        DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS,
    },
    types::ProcessedVMOutput,
    Executor,
};
use diem_types::protocol_spec::ProtocolSpec;

impl<PS, V> BlockExecutor for Executor<PS, V>
where
    PS: ProtocolSpec,
    V: VMExecutor,
{
    fn committed_block_id(&self) -> anyhow::Result<HashValue, Error> {
        Ok(Self::committed_block_id(self))
    }

    fn reset(&self) -> anyhow::Result<(), Error> {
        self.reset_cache()
    }

    fn execute_block(
        &self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> anyhow::Result<StateComputeResult, Error> {
        let (block_id, mut transactions) = block;
        let read_lock = self.cache.read();

        // Reconfiguration rule - if a block is a child of pending reconfiguration, it needs to be empty
        // So we roll over the executed state until it's committed and we start new epoch.
        let (output, state_compute_result) = if parent_block_id != read_lock.committed_block_id()
            && read_lock
                .get_block(&parent_block_id)?
                .lock()
                .output()
                .has_reconfiguration()
        {
            let parent = read_lock.get_block(&parent_block_id)?;
            drop(read_lock);
            let parent_block = parent.lock();
            let parent_output = parent_block.output();

            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "reconfig_descendant_block_received"
            );

            let output = ProcessedVMOutput::new(
                vec![],
                parent_output.executed_trees().clone(),
                parent_output.epoch_state().clone(),
            );

            let parent_accu = parent_output.executed_trees().txn_accumulator();
            let state_compute_result = output.compute_result(
                parent_accu.frozen_subtree_roots().clone(),
                parent_accu.num_leaves(),
            );

            // Reset the reconfiguration suffix transactions to empty list.
            transactions = vec![];

            (output, state_compute_result)
        } else {
            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "execute_block"
            );

            let _timer = DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS.start_timer();

            let parent_block_executed_trees =
                Self::get_executed_trees_from_lock(&read_lock, parent_block_id)?;

            // Hold a ref to the current base smt, so that all in-mem state in between the
            // currently committed version and the end version of the parent block won't go away
            // during execution.
            let _base_smt = read_lock.committed_trees().state_tree().clone();

            let state_view = self.get_executed_state_view_from_lock(
                &read_lock,
                StateViewId::BlockExecution { block_id },
                &parent_block_executed_trees,
            );
            drop(read_lock);

            let vm_outputs = {
                let _timer = DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.start_timer();
                fail_point!("executor::vm_execute_block", |_| {
                    Err(Error::from(anyhow::anyhow!(
                        "Injected error in vm_execute_block"
                    )))
                });
                V::execute_block(transactions.clone(), &state_view).map_err(anyhow::Error::from)?
            };

            let status: Vec<_> = vm_outputs
                .iter()
                .map(TransactionOutput::status)
                .cloned()
                .collect();
            if !status.is_empty() {
                trace!("Execution status: {:?}", status);
            }

            let (account_to_state, account_to_proof) = state_view.into();
            let output = Self::process_vm_outputs(
                account_to_state,
                account_to_proof,
                &transactions,
                vm_outputs,
                &parent_block_executed_trees,
            )
            .map_err(|err| format_err!("Failed to execute block: {}", err))?;

            let parent_accu = parent_block_executed_trees.txn_accumulator();

            let state_compute_result = output.compute_result(
                parent_accu.frozen_subtree_roots().clone(),
                parent_accu.num_leaves(),
            );
            (output, state_compute_result)
        };

        // Add the output to the speculation_output_tree
        self.cache
            .write()
            .add_block(parent_block_id, (block_id, transactions, output))?;

        Ok(state_compute_result)
    }

    fn commit_blocks(
        &self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> anyhow::Result<(), Error> {
        let _timer = DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS.start_timer();
        self.do_commit_blocks(block_ids, &ledger_info_with_sigs)?;

        self.cache
            .write()
            .prune(ledger_info_with_sigs.ledger_info())?;

        // Now that the blocks are persisted successfully, we can reply to consensus
        Ok(())
    }
}

impl<PS, V> Executor<PS, V>
    where
        PS: ProtocolSpec,
        V: VMExecutor,
{
    fn do_commit_blocks(&self, block_ids: Vec<HashValue>, ledger_info_with_sigs: &LedgerInfoWithSignatures) -> anyhow::Result<(), Error> {
        let block_id_to_commit = ledger_info_with_sigs.ledger_info().consensus_block_id();
        let read_lock = self.cache.read();

        info!(
            LogSchema::new(LogEntry::BlockExecutor).block_id(block_id_to_commit),
            "commit_block"
        );

        let version = ledger_info_with_sigs.ledger_info().version();

        let num_txns_in_li = version
            .checked_add(1)
            .ok_or_else(|| format_err!("version + 1 overflows"))?;
        let num_persistent_txns = read_lock.synced_trees().txn_accumulator().num_leaves();

        if num_txns_in_li < num_persistent_txns {
            return Err(Error::InternalError {
                error: format!(
                    "Try to commit stale transactions with the last version as {}",
                    version
                ),
            });
        }

        if num_txns_in_li == num_persistent_txns {
            return Ok(());
        }

        // All transactions that need to go to storage. In the above example, this means all the
        // transactions in A, B and C whose status == TransactionStatus::Keep.
        // This must be done before calculate potential skipping of transactions in idempotent commit.
        let mut txns_to_keep = vec![];
        let arc_blocks = block_ids
            .iter()
            .map(|id| read_lock.get_block(id))
            .collect::<anyhow::Result<Vec<_>, Error>>()?;
        let blocks = arc_blocks.iter().map(|b| b.lock()).collect::<Vec<_>>();
        for (txn, txn_data) in blocks.iter().flat_map(|block| {
            itertools::zip_eq(block.transactions(), block.output().transaction_data())
        }) {
            if let TransactionStatus::Keep(recorded_status) = txn_data.status() {
                txns_to_keep.push(TransactionToCommit::new(
                    txn.clone(),
                    txn_data.account_blobs().clone(),
                    Some(txn_data.jf_node_hashes().clone()),
                    txn_data.write_set().clone(),
                    txn_data.events().to_vec(),
                    txn_data.gas_used(),
                    recorded_status.clone(),
                ));
            }
        }

        let last_block = blocks
            .last()
            .ok_or_else(|| format_err!("CommittableBlockBatch is empty"))?;

        // Check that the version in ledger info (computed by consensus) matches the version
        // computed by us.
        let num_txns_in_speculative_accumulator = last_block
            .output()
            .executed_trees()
            .txn_accumulator()
            .num_leaves();
        assert_eq!(
            num_txns_in_li, num_txns_in_speculative_accumulator as Version,
            "Number of transactions in ledger info ({}) does not match number of transactions \
             in accumulator ({}).",
            num_txns_in_li, num_txns_in_speculative_accumulator,
        );
        drop(blocks);
        drop(read_lock);

        let num_txns_to_keep = txns_to_keep.len() as u64;

        // Skip txns that are already committed to allow failures in state sync process.
        let first_version_to_keep = num_txns_in_li - num_txns_to_keep;
        assert!(
            first_version_to_keep <= num_persistent_txns,
            "first_version {} in the blocks to commit cannot exceed # of committed txns: {}.",
            first_version_to_keep,
            num_persistent_txns
        );

        let num_txns_to_skip = num_persistent_txns - first_version_to_keep;
        let first_version_to_commit = first_version_to_keep + num_txns_to_skip;

        if num_txns_to_skip != 0 {
            debug!(
                LogSchema::new(LogEntry::BlockExecutor)
                    .latest_synced_version(num_persistent_txns - 1)
                    .first_version_to_keep(first_version_to_keep)
                    .num_txns_to_keep(num_txns_to_keep)
                    .first_version_to_commit(first_version_to_commit),
                "skip_transactions_when_committing"
            );
        }

        // Skip duplicate txns that are already persistent.
        let txns_to_commit = &txns_to_keep[num_txns_to_skip as usize..];
        let num_txns_to_commit = txns_to_commit.len() as u64;
        {
            let _timer = DIEM_EXECUTOR_SAVE_TRANSACTIONS_SECONDS.start_timer();
            DIEM_EXECUTOR_TRANSACTIONS_SAVED.observe(num_txns_to_commit as f64);

            assert_eq!(first_version_to_commit, num_txns_in_li - num_txns_to_commit);
            fail_point!("executor::commit_blocks", |_| {
                Err(Error::from(anyhow::anyhow!(
                    "Injected error in commit_blocks"
                )))
            });

            self.db.writer.save_transactions(
                txns_to_commit,
                first_version_to_commit,
                Some(ledger_info_with_sigs),
            )?;
        }

        Ok(())
    }
}
