// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{Error, Result},
    task::{
        ExecutionStatus, ExecutorTask, ReadWriteSetInferencer, Transaction as TransactionType,
        TransactionOutput,
    },
};
use anyhow::Result as AResult;
use mvhashmap::MVHashMapView;
use proptest::{
    arbitrary::Arbitrary, collection::vec, prelude::*, proptest, sample::Index, strategy::Strategy,
};
use proptest_derive::Arbitrary;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

///////////////////////////////////////////////////////////////////////////
// Generation of transactions
///////////////////////////////////////////////////////////////////////////

#[derive(Arbitrary, Debug, Clone)]
#[proptest(params = "TransactionGenParams")]
pub struct TransactionGen<V: Arbitrary + Debug + 'static + Clone> {
    #[proptest(
        strategy = "vec((any::<Index>(), value_strategy(params.write_keep_rate)), 1..params.possible_write_size)"
    )]
    keys_modified: Vec<(Index, Option<V>)>,
    #[proptest(strategy = "vec(any::<Index>(), 1..params.read_size)")]
    keys_read: Vec<Index>,
}

#[derive(Clone, Copy)]
pub struct TransactionGenParams {
    pub possible_write_size: usize,
    pub read_size: usize,
    pub write_keep_rate: f64,
}

/// A naive transaction that could be used to test the correctness and throughput of the system.
#[derive(Debug, Clone)]
pub enum Transaction<K, V> {
    Write {
        /// Write to some keys with value provided.
        actual_writes: Vec<(K, V)>,
        /// Skipp writing to some keys. This is used to simulate over approximation of the inferencer.
        skipped_writes: Vec<K>,
        /// Read from some keys.
        reads: Vec<K>,
    },
    /// Skip the execution of trailing transactions.
    SkipRest,
    /// Abort the execution.
    Abort,
}

impl Default for TransactionGenParams {
    fn default() -> Self {
        TransactionGenParams {
            possible_write_size: 10,
            write_keep_rate: 0.5,
            read_size: 10,
        }
    }
}

fn value_strategy<V: Arbitrary + Clone + 'static>(
    keep_rate: f64,
) -> impl Strategy<Value = Option<V>> {
    let value_strategy = any::<V>();
    proptest::option::weighted(keep_rate, value_strategy)
}

impl<V: Arbitrary + Debug + Clone> TransactionGen<V> {
    pub fn materialize<K: Clone + Eq + Ord>(self, universe: &[K]) -> Transaction<K, V> {
        let mut keys_modified = BTreeSet::new();
        let mut actual_writes = vec![];
        let mut skipped_writes = vec![];
        for (idx, value) in self.keys_modified.into_iter() {
            let key = universe[idx.index(universe.len())].clone();
            if !keys_modified.contains(&key) {
                keys_modified.insert(key.clone());
                match value {
                    None => skipped_writes.push(key),
                    Some(v) => actual_writes.push((key, v)),
                };
            }
        }
        Transaction::Write {
            actual_writes,
            skipped_writes,
            reads: self
                .keys_read
                .into_iter()
                .map(|k| universe[k.index(universe.len())].clone())
                .collect(),
        }
    }
}

impl<K, V> TransactionType for Transaction<K, V>
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + 'static,
{
    type Key = K;
    type Value = V;
}

///////////////////////////////////////////////////////////////////////////
// Naive inferencer implementation.
///////////////////////////////////////////////////////////////////////////

pub struct Inferencer<K, V>(PhantomData<(K, V)>);

impl<K, V> Inferencer<K, V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K, V> ReadWriteSetInferencer for Inferencer<K, V>
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + 'static,
{
    type T = Transaction<K, V>;

    fn infer_writes(&self, txn: &Self::T) -> AResult<Vec<K>> {
        match txn {
            Transaction::Write {
                actual_writes,
                skipped_writes,
                reads: _,
            } => {
                let mut writes = actual_writes
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect::<Vec<_>>();
                writes.append(&mut skipped_writes.clone());
                Ok(writes)
            }
            Transaction::SkipRest | Transaction::Abort => Ok(vec![]),
        }
    }

    fn infer_reads(&self, txn: &Self::T) -> AResult<Vec<K>> {
        match txn {
            Transaction::Write {
                actual_writes: _,
                skipped_writes: _,
                reads,
            } => Ok(reads.clone()),
            Transaction::SkipRest | Transaction::Abort => Ok(vec![]),
        }
    }
}

pub struct ImpreciseInferencer<K, V>(PhantomData<(K, V)>);

impl<K, V> ImpreciseInferencer<K, V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K, V> ReadWriteSetInferencer for ImpreciseInferencer<K, V>
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + 'static,
{
    type T = Transaction<K, V>;

    fn infer_writes(&self, txn: &Self::T) -> AResult<Vec<K>> {
        match txn {
            Transaction::Write {
                actual_writes,
                skipped_writes,
                reads: _,
            } => {
                let mut writes = actual_writes
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect::<Vec<_>>();
                writes.append(&mut skipped_writes.clone());
                Ok(writes)
            }
            Transaction::SkipRest | Transaction::Abort => Ok(vec![]),
        }
    }

    fn infer_reads(&self, txn: &Self::T) -> AResult<Vec<K>> {
        match txn {
            Transaction::Write {
                actual_writes: _,
                skipped_writes: _,
                reads,
            } => {
                let mut results = reads.clone();
                // Drop one read entry to simulate imprecise read estimation
                results.pop();
                Ok(results)
            }
            Transaction::SkipRest | Transaction::Abort => Ok(vec![]),
        }
    }
}

///////////////////////////////////////////////////////////////////////////
// Naive transaction executor implementation.
///////////////////////////////////////////////////////////////////////////

pub struct Task<K, V>(PhantomData<(K, V)>);

impl<K, V> Task<K, V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K, V> ExecutorTask for Task<K, V>
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + 'static,
{
    type T = Transaction<K, V>;
    type Output = Output<K, V>;
    type Error = usize;
    type Argument = ();

    fn init(_argument: Self::Argument) -> Self {
        Self::new()
    }

    fn execute_transaction(
        &self,
        view: MVHashMapView<K, V>,
        txn: &Self::T,
    ) -> ExecutionStatus<Self::Output, Self::Error> {
        match txn {
            Transaction::Write {
                reads,
                actual_writes,
                skipped_writes: _,
            } => {
                // Reads
                let mut reads_result = vec![];
                for k in reads.iter() {
                    reads_result.push(match view.read(k) {
                        Ok(v) => Some(v.clone()),
                        Err(None) => None,
                        Err(Some(v)) => return ExecutionStatus::Retry(v),
                    })
                }
                ExecutionStatus::Success(Output(actual_writes.clone(), reads_result))
            }
            Transaction::SkipRest => ExecutionStatus::SkipRest(Output(vec![], vec![])),
            Transaction::Abort => ExecutionStatus::Abort(view.version()),
        }
    }
}

pub struct Output<K, V>(Vec<(K, V)>, Vec<Option<V>>);

impl<K, V> TransactionOutput for Output<K, V>
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + 'static,
{
    type T = Transaction<K, V>;

    fn get_writes(&self) -> Vec<(K, V)> {
        self.0.clone()
    }

    fn skip_output() -> Self {
        Self(vec![], vec![])
    }
}

///////////////////////////////////////////////////////////////////////////
// Sequential Baseline implementation.
///////////////////////////////////////////////////////////////////////////

/// Sequential baseline of execution result for dummy transaction.
pub enum ExpectedOutput<V> {
    Aborted(usize),
    SkipRest(usize, Vec<Vec<Option<V>>>),
    Success(Vec<Vec<Option<V>>>),
}

impl<V: Clone + Eq> ExpectedOutput<V> {
    pub fn generate_baseline<K: Hash + Clone + Eq>(txns: &[Transaction<K, V>]) -> Self {
        let mut current_world = HashMap::new();
        let mut result_vec = vec![];
        for (idx, txn) in txns.iter().enumerate() {
            match txn {
                Transaction::Abort => return Self::Aborted(idx),
                Transaction::Write {
                    reads,
                    actual_writes,
                    skipped_writes: _,
                } => {
                    let mut result = vec![];
                    for k in reads.iter() {
                        result.push(current_world.get(k).cloned());
                    }
                    for (k, v) in actual_writes.iter() {
                        current_world.insert(k.clone(), v.clone());
                    }
                    result_vec.push(result)
                }
                Transaction::SkipRest => return Self::SkipRest(idx, result_vec),
            }
        }
        Self::Success(result_vec)
    }

    pub fn check_output<K>(&self, results: &Result<Vec<Output<K, V>>, usize>) -> bool {
        match (self, results) {
            (Self::Aborted(i), Err(Error::UserError(idx))) => i == idx,
            (Self::SkipRest(skip_at, expected_results), Ok(results)) => {
                results
                    .iter()
                    .take(*skip_at)
                    .zip(expected_results.iter())
                    .all(|(Output(_, result), expected_results)| expected_results == result)
                    && results
                        .iter()
                        .skip(*skip_at)
                        .all(|Output(_, result)| result.is_empty())
            }
            (Self::Success(expected_results), Ok(results)) => expected_results
                .iter()
                .zip(results.iter())
                .all(|(expected_result, Output(_, result))| expected_result == result),
            _ => false,
        }
    }
}
