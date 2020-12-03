// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::execution_strategies::types::{Block, Executor, ExecutorResult};
use diem_types::transaction::TransactionOutput;
use std::{collections::BTreeMap, error::Error, fmt};

#[derive(Debug)]
pub enum MultiResult<E: Error> {
    NonMatchingOutput(TransactionOutput, TransactionOutput),
    OtherResult(E),
}

impl<E: Error> std::fmt::Display for MultiResult<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiResult::OtherResult(err) => std::fmt::Display::fmt(&err, f),
            MultiResult::NonMatchingOutput(output1, output2) => {
                write!(f, "{:?} != {:?}", output1, output2)
            }
        }
    }
}

impl<E: Error> Error for MultiResult<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MultiResult::OtherResult(err) => err.source(),
            MultiResult::NonMatchingOutput(_output1, _output2) => None,
        }
    }
}

pub struct MultiExecutor<TxnType, E: Error> {
    executors: Vec<Box<dyn Executor<Txn = TxnType, BlockResult = E>>>,
}

impl<TxnType, E: Error> Default for MultiExecutor<TxnType, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TxnType, E: Error> MultiExecutor<TxnType, E> {
    pub fn new() -> Self {
        Self {
            executors: Vec::new(),
        }
    }

    pub fn add_executor(
        &mut self,
        executor: impl Executor<Txn = TxnType, BlockResult = E> + 'static,
    ) {
        self.executors.push(Box::new(executor))
    }
}

impl<TxnType: Clone, E: Error> Executor for MultiExecutor<TxnType, E> {
    type BlockResult = MultiResult<E>;
    type Txn = TxnType;
    fn execute_block(&mut self, block: Block<Self::Txn>) -> ExecutorResult<Self::BlockResult> {
        let mut results = BTreeMap::new();
        for executor in self.executors.iter_mut() {
            let block = match executor.execute_block(block.clone()) {
                Err(err) => return Err(MultiResult::OtherResult(err)),
                Ok(block) => block,
            };
            for (index, output) in block.into_iter().enumerate() {
                match results.get(&index) {
                    None => {
                        results.insert(index, output);
                    }
                    Some(previous_output) => {
                        if &output != previous_output {
                            return Err(MultiResult::NonMatchingOutput(
                                output,
                                previous_output.clone(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(results.into_iter().map(|(_, v)| v).collect())
    }
}
