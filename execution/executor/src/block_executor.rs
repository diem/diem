// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::logging::{LogEntry, LogSchema};
use anyhow::Result;
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_state_view::StateViewId;
use diem_types::{ledger_info::LedgerInfoWithSignatures, transaction::Transaction};
use diem_vm::VMExecutor;
use executor_types::{BlockExecutorTrait, Error, StateComputeResult};
use fail::fail_point;
use std::marker::PhantomData;

use crate::{
    components::{block_tree::BlockTree, chunk_output::ChunkOutput},
    metrics::{
        DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS, DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS,
        DIEM_EXECUTOR_SAVE_TRANSACTIONS_SECONDS, DIEM_EXECUTOR_TRANSACTIONS_SAVED,
        DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS,
    },
};
use storage_interface::DbReaderWriter;

pub struct BlockExecutor<V> {
    pub db: DbReaderWriter,
    block_tree: BlockTree,
    phantom: PhantomData<V>,
}

impl<V> BlockExecutor<V>
where
    V: VMExecutor,
{
    pub fn new(db: DbReaderWriter) -> Self {
        let block_tree = BlockTree::new(&db.reader).expect("Block tree failed to init.");
        Self {
            db,
            block_tree,
            phantom: PhantomData,
        }
    }
}

impl<V> BlockExecutorTrait for BlockExecutor<V>
where
    V: VMExecutor,
{
    fn committed_block_id(&self) -> HashValue {
        self.block_tree.root_block().id
    }

    fn reset(&self) -> Result<(), Error> {
        Ok(self.block_tree.reset(&self.db.reader)?)
    }

    fn execute_block(
        &self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let (block_id, transactions) = block;
        let committed_block = self.block_tree.root_block();
        let mut block_vec = self
            .block_tree
            .get_blocks_opt(&[block_id, parent_block_id])?;
        let parent_block = block_vec
            .pop()
            .expect("Must exist.")
            .ok_or(Error::BlockNotFound(parent_block_id))?;
        let parent_output = &parent_block.output;
        let parent_view = &parent_output.result_view;
        let parent_accumulator = parent_view.txn_accumulator();

        if let Some(b) = block_vec.pop().expect("Must exist") {
            // this is a retry
            return Ok(b.output.as_state_compute_result(parent_accumulator));
        }

        let output = if parent_block_id != committed_block.id && parent_output.has_reconfiguration()
        {
            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "reconfig_descendant_block_received"
            );
            parent_output.reconfig_suffix()
        } else {
            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "execute_block"
            );
            let _timer = DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS.start_timer();
            let state_view = parent_view.state_view(
                &committed_block.output.result_view,
                StateViewId::BlockExecution { block_id },
                self.db.reader.clone(),
            );

            let chunk_output = {
                let _timer = DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.start_timer();
                fail_point!("executor::vm_execute_block", |_| {
                    Err(Error::from(anyhow::anyhow!(
                        "Injected error in vm_execute_block"
                    )))
                });
                ChunkOutput::by_transaction_execution::<V>(transactions, state_view)?
            };
            chunk_output.trace_log_transaction_status();

            let (output, _, _) = chunk_output.apply_to_ledger(parent_view)?;
            output
        };

        output.ensure_ends_with_state_checkpoint()?;
        let block = self
            .block_tree
            .add_block(parent_block_id, block_id, output)?;
        Ok(block.output.as_state_compute_result(parent_accumulator))
    }

    fn commit_blocks(
        &self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        let _timer = DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS.start_timer();
        let committed_block = self.block_tree.root_block();
        if committed_block.num_persisted_transactions()
            == ledger_info_with_sigs.ledger_info().version() + 1
        {
            // a retry
            return Ok(());
        }

        let block_id_to_commit = ledger_info_with_sigs.ledger_info().consensus_block_id();
        info!(
            LogSchema::new(LogEntry::BlockExecutor).block_id(block_id_to_commit),
            "commit_block"
        );

        let blocks = self.block_tree.get_blocks(&block_ids)?;
        let txns_to_commit: Vec<_> = blocks
            .into_iter()
            .map(|block| block.output.transactions_to_commit())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        let first_version = committed_block
            .output
            .result_view
            .txn_accumulator()
            .num_leaves();
        let to_commit = txns_to_commit.len();
        let target_version = ledger_info_with_sigs.ledger_info().version();
        if first_version + txns_to_commit.len() as u64 != target_version + 1 {
            return Err(Error::BadNumTxnsToCommit {
                first_version,
                to_commit,
                target_version,
            });
        }

        {
            let _timer = DIEM_EXECUTOR_SAVE_TRANSACTIONS_SECONDS.start_timer();
            DIEM_EXECUTOR_TRANSACTIONS_SAVED.observe(to_commit as f64);

            fail_point!("executor::commit_blocks", |_| {
                Err(anyhow::anyhow!("Injected error in commit_blocks.").into())
            });

            self.db.writer.save_transactions(
                &txns_to_commit,
                first_version,
                Some(&ledger_info_with_sigs),
            )?;
            self.block_tree
                .prune(ledger_info_with_sigs.ledger_info())
                .expect("Failure pruning block tree.");
        }
        Ok(())
    }
}
