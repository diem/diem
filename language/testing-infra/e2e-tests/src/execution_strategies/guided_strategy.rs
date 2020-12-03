// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{
    execution_strategies::types::{Block, Executor, ExecutorResult, PartitionStrategy},
    executor::FakeExecutor,
};
use diem_types::{transaction::SignedTransaction, vm_status::VMStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotatedTransaction {
    Block,
    Txn(Box<SignedTransaction>),
}

#[derive(Debug, Clone)]
pub struct PartitionedGuidedStrategy;

impl PartitionStrategy for PartitionedGuidedStrategy {
    type Txn = AnnotatedTransaction;
    fn partition(&mut self, block: Block<Self::Txn>) -> Vec<Block<SignedTransaction>> {
        block
            .split(|atxn| atxn == &AnnotatedTransaction::Block)
            .map(move |block| {
                block
                    .iter()
                    .map(move |atxn| match atxn {
                        AnnotatedTransaction::Block => unreachable!(),
                        AnnotatedTransaction::Txn(txn) => *txn.clone(),
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct UnPartitionedGuidedStrategy;

impl PartitionStrategy for UnPartitionedGuidedStrategy {
    type Txn = AnnotatedTransaction;
    fn partition(&mut self, block: Block<Self::Txn>) -> Vec<Block<SignedTransaction>> {
        vec![block
            .into_iter()
            .filter_map(|atxn| match atxn {
                AnnotatedTransaction::Block => None,
                AnnotatedTransaction::Txn(txn) => Some(*txn),
            })
            .collect()]
    }
}

pub struct GuidedExecutor<Strategy: PartitionStrategy> {
    strategy: Strategy,
    executor: FakeExecutor,
}

impl<Strategy: PartitionStrategy> GuidedExecutor<Strategy> {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            executor: FakeExecutor::from_genesis_file(),
        }
    }
}

impl<Strategy: PartitionStrategy> Executor for GuidedExecutor<Strategy> {
    type Txn = Strategy::Txn;
    type BlockResult = VMStatus;
    fn execute_block(&mut self, block: Block<Self::Txn>) -> ExecutorResult<Self::BlockResult> {
        let mut outputs = vec![];
        for block in self.strategy.partition(block).into_iter() {
            outputs.extend(
                self.executor
                    .execute_block(block)?
                    .into_iter()
                    .map(|output| {
                        self.executor.apply_write_set(output.write_set());
                        output
                    }),
            )
        }
        Ok(outputs)
    }
}
