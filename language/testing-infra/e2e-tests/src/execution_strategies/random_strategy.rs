// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{
    execution_strategies::types::{Block, Executor, ExecutorResult, PartitionStrategy},
    executor::FakeExecutor,
};
use diem_types::{transaction::SignedTransaction, vm_status::VMStatus};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};

#[derive(Debug, Clone)]
pub struct RandomizedStrategy {
    gen: StdRng,
}

impl RandomizedStrategy {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            gen: StdRng::from_seed(seed),
        }
    }

    pub fn from_os_rng() -> Self {
        let mut seed_rng = OsRng;
        let seed: [u8; 32] = seed_rng.gen();
        Self::from_seed(seed)
    }
}

impl PartitionStrategy for RandomizedStrategy {
    type Txn = SignedTransaction;
    fn partition(&mut self, mut block: Block<Self::Txn>) -> Vec<Block<SignedTransaction>> {
        let mut blocks = vec![];
        while !block.is_empty() {
            let block_size = self.gen.gen_range(0..block.len());
            let new_block: Vec<_> = block.drain(0..block_size + 1).collect();
            blocks.push(new_block);
        }
        blocks
    }
}

#[derive(Debug)]
pub struct RandomExecutor {
    strategy: RandomizedStrategy,
    executor: FakeExecutor,
}

impl RandomExecutor {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            executor: FakeExecutor::from_genesis_file(),
            strategy: RandomizedStrategy::from_seed(seed),
        }
    }

    pub fn from_os_rng() -> Self {
        RandomExecutor::from_seed(OsRng.gen::<[u8; 32]>())
    }
}

impl Executor for RandomExecutor {
    type Txn = SignedTransaction;
    type BlockResult = VMStatus;
    fn execute_block(&mut self, block: Block<Self::Txn>) -> ExecutorResult<Self::BlockResult> {
        let blocks = self.strategy.partition(block);
        let mut results = vec![];
        for block in blocks.into_iter() {
            results.extend(
                self.executor
                    .execute_block(block)?
                    .into_iter()
                    .map(|output| {
                        self.executor.apply_write_set(output.write_set());
                        output
                    }),
            )
        }
        Ok(results)
    }
}
