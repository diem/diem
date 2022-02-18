// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::hash::HashValue;
use diem_types::transaction::{Transaction, Version};
use diem_vm::DiemVM;
use executor::block_executor::BlockExecutor;
use executor_types::BlockExecutorTrait;
use std::{
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

pub struct TransactionExecutor {
    executor: Arc<BlockExecutor<DiemVM>>,
    parent_block_id: HashValue,
    start_time: Instant,
    version: Version,
    // If commit_sender is `None`, we will commit all the execution result immediately in this struct.
    commit_sender:
        Option<mpsc::SyncSender<(HashValue, HashValue, Instant, Instant, Duration, usize)>>,
}

impl TransactionExecutor {
    pub fn new(
        executor: Arc<BlockExecutor<DiemVM>>,
        parent_block_id: HashValue,
        version: Version,
        commit_sender: Option<
            mpsc::SyncSender<(HashValue, HashValue, Instant, Instant, Duration, usize)>,
        >,
    ) -> Self {
        Self {
            executor,
            parent_block_id,
            version,
            start_time: Instant::now(),
            commit_sender,
        }
    }

    pub fn execute_block(&mut self, transactions: Vec<Transaction>) {
        let num_txns = transactions.len();
        self.version += num_txns as Version;

        let execution_start = std::time::Instant::now();

        let block_id = HashValue::random();
        let output = self
            .executor
            .execute_block((block_id, transactions), self.parent_block_id)
            .unwrap();
        assert_eq!(output.version(), self.version, "num_txns: {}", num_txns);

        self.parent_block_id = block_id;

        if let Some(ref commit_sender) = self.commit_sender {
            commit_sender
                .send((
                    block_id,
                    output.root_hash(),
                    self.start_time,
                    execution_start,
                    Instant::now().duration_since(execution_start),
                    num_txns,
                ))
                .unwrap();
        } else {
            let ledger_info_with_sigs = super::transaction_committer::gen_li_with_sigs(
                block_id,
                output.root_hash(),
                self.version,
            );
            self.executor
                .commit_blocks(vec![block_id], ledger_info_with_sigs)
                .unwrap();
        }
    }
}
