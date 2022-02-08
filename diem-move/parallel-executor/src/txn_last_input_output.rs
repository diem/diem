// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    errors::Error,
    scheduler::Version,
    task::{ExecutionStatus, Transaction, TransactionOutput},
};
use arc_swap::ArcSwapOption;
use crossbeam::utils::CachePadded;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

type TxnInput<K> = Vec<ReadDescriptor<K>>;
type TxnOutput<T, E> = RwLock<Option<ExecutionStatus<T, Error<E>>>>;

#[derive(Clone)]
pub struct ReadDescriptor<K> {
    access_path: K,

    // Stores a version = (txn_idx, incarnation) if the read of access_path was from
    // shared multi-version data-structure, None if read from storage.
    version: Option<Version>,
}

impl<K> ReadDescriptor<K> {
    pub fn new(access_path: K, version: Option<Version>) -> Self {
        Self {
            access_path,
            version,
        }
    }

    pub fn path(&self) -> &K {
        &self.access_path
    }

    pub fn validate(&self, new_version: Option<Version>) -> bool {
        self.version == new_version
    }
}

pub struct TxnLastInputOutput<K, T, E> {
    inputs: Vec<CachePadded<ArcSwapOption<TxnInput<K>>>>, // txn_idx -> input.

    outputs: Vec<CachePadded<ArcSwapOption<TxnOutput<T, E>>>>, // txn_idx -> output.
}

impl<K, T: TransactionOutput, E: Send + Clone> TxnLastInputOutput<K, T, E> {
    pub fn new(num_txns: usize) -> Self {
        Self {
            inputs: (0..num_txns)
                .map(|_| CachePadded::new(ArcSwapOption::empty()))
                .collect(),
            outputs: (0..num_txns)
                .map(|_| CachePadded::new(ArcSwapOption::empty()))
                .collect(),
        }
    }

    pub fn record(
        &self,
        txn_idx: usize,
        input: Vec<ReadDescriptor<K>>,
        output: ExecutionStatus<T, Error<E>>,
    ) {
        self.inputs[txn_idx].store(Some(Arc::new(input)));
        self.outputs[txn_idx].store(Some(Arc::new(RwLock::new(Some(output)))));
    }

    // executed() must hold. Returns an Arc to the inner read set to avoid clone
    // and maintain lifetime.
    pub fn read_set(&self, txn_idx: usize) -> Arc<Vec<ReadDescriptor<K>>> {
        self.inputs[txn_idx].load_full().unwrap()
    }

    pub fn write_set(
        &self,
        txn_idx: usize,
    ) -> HashSet<<<T as TransactionOutput>::T as Transaction>::Key> {
        let txn_output = self.outputs[txn_idx].load_full();

        if txn_output.is_none() {
            return HashSet::new();
        }

        // TODO: Get rid of smart pointer clone, use load() and values maybe.
        let output = txn_output.as_ref().unwrap().read().unwrap();
        let t = match output.as_ref().unwrap() {
            ExecutionStatus::Success(t) => t,
            ExecutionStatus::SkipRest(t) => t,
            ExecutionStatus::Abort(_) => return HashSet::new(),
        };

        t.get_writes().iter().map(|(k, _)| k.clone()).collect()
    }

    // Must be executed after parallel execution is done, only grab outputs.
    pub fn take_output(&self, txn_idx: usize) -> ExecutionStatus<T, Error<E>> {
        self.outputs[txn_idx]
            .load()
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .take()
            .unwrap()
    }
}
