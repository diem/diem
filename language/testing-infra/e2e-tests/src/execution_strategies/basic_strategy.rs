// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    execution_strategies::types::{Block, Executor, ExecutorResult, PartitionStrategy},
    executor::FakeExecutor,
};
use diem_types::{transaction::SignedTransaction, vm_status::VMStatus};

#[derive(Debug, Clone)]
pub struct BasicStrategy;

impl PartitionStrategy for BasicStrategy {
    type Txn = SignedTransaction;
    fn partition(&mut self, block: Block<Self::Txn>) -> Vec<Block<SignedTransaction>> {
        vec![block]
    }
}

#[derive(Debug)]
pub struct BasicExecutor {
    executor: FakeExecutor,
    strategy: BasicStrategy,
}

impl Default for BasicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicExecutor {
    pub fn new() -> Self {
        Self {
            executor: FakeExecutor::from_genesis_file(),
            strategy: BasicStrategy,
        }
    }
}

impl Executor for BasicExecutor {
    type Txn = <BasicStrategy as PartitionStrategy>::Txn;
    type BlockResult = VMStatus;
    fn execute_block(&mut self, txns: Block<Self::Txn>) -> ExecutorResult<Self::BlockResult> {
        let mut block = self.strategy.partition(txns);
        let outputs = self.executor.execute_block(block.remove(0))?;
        for output in &outputs {
            self.executor.apply_write_set(output.write_set())
        }
        Ok(outputs)
    }
}
