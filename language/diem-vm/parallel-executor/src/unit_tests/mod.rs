// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::ParallelTransactionExecutor,
    proptest_types::types::{ExpectedOutput, Inferencer, Task, Transaction},
};
use rand::random;
use std::{fmt::Debug, hash::Hash};

fn run_and_assert<K, V>(transactions: Vec<Transaction<K, V>>)
where
    K: PartialOrd + Send + Sync + Clone + Hash + Eq + 'static,
    V: Send + Sync + Debug + Clone + Eq + 'static,
{
    let baseline = ExpectedOutput::generate_baseline(&transactions);

    let output =
        ParallelTransactionExecutor::<Transaction<K, V>, Task<K, V>, Inferencer<K, V>>::new(
            Inferencer::new(),
        )
        .execute_transactions_parallel((), transactions);

    assert!(baseline.check_output(&output))
}

const TOTAL_KEY_NUM: u64 = 50;
const WRITES_PER_KEY: u64 = 100;

#[test]
fn cycle_transactions() {
    let mut transactions = vec![];
    // For every key in `TOTAL_KEY_NUM`, generate a series transaction that will assign a value to
    // this key.
    for _ in 0..TOTAL_KEY_NUM {
        let key = random::<[u8; 32]>();
        for _ in 0..WRITES_PER_KEY {
            transactions.push(Transaction::Write {
                reads: vec![key],
                actual_writes: vec![(key, random::<u64>())],
                skipped_writes: vec![],
            })
        }
    }
    run_and_assert(transactions)
}

const NUM_BLOCKS: u64 = 10;
const TXN_PER_BLOCK: u64 = 100;

#[test]
fn one_reads_all_barrier() {
    let mut transactions = vec![];
    let keys: Vec<_> = (0..TXN_PER_BLOCK).map(|_| random::<[u8; 32]>()).collect();
    for _ in 0..NUM_BLOCKS {
        for key in &keys {
            transactions.push(Transaction::Write {
                reads: vec![*key],
                actual_writes: vec![(*key, random::<u64>())],
                skipped_writes: vec![],
            })
        }
        // One transaction reading the write results of every prior transactions in the block.
        transactions.push(Transaction::Write {
            reads: keys.clone(),
            actual_writes: vec![],
            skipped_writes: vec![],
        })
    }
    run_and_assert(transactions)
}

#[test]
fn one_writes_all_barrier() {
    let mut transactions = vec![];
    let keys: Vec<_> = (0..TXN_PER_BLOCK).map(|_| random::<[u8; 32]>()).collect();
    for _ in 0..NUM_BLOCKS {
        for key in &keys {
            transactions.push(Transaction::Write {
                reads: vec![*key],
                actual_writes: vec![(*key, random::<u64>())],
                skipped_writes: vec![],
            })
        }
        // One transaction writing to the write results of every prior transactions in the block.
        transactions.push(Transaction::Write {
            reads: keys.clone(),
            actual_writes: keys
                .iter()
                .map(|key| (*key, random::<u64>()))
                .collect::<Vec<_>>(),
            skipped_writes: vec![],
        })
    }
    run_and_assert(transactions)
}

#[test]
fn early_aborts() {
    let mut transactions = vec![];
    let keys: Vec<_> = (0..TXN_PER_BLOCK).map(|_| random::<[u8; 32]>()).collect();

    for _ in 0..NUM_BLOCKS {
        for key in &keys {
            transactions.push(Transaction::Write {
                reads: vec![*key],
                actual_writes: vec![(*key, random::<u64>())],
                skipped_writes: vec![],
            })
        }
        // One transaction that triggers an abort
        transactions.push(Transaction::Abort)
    }
    run_and_assert(transactions)
}

#[test]
fn early_skips() {
    let mut transactions = vec![];
    let keys: Vec<_> = (0..TXN_PER_BLOCK).map(|_| random::<[u8; 32]>()).collect();

    for _ in 0..NUM_BLOCKS {
        for key in &keys {
            transactions.push(Transaction::Write {
                reads: vec![*key],
                actual_writes: vec![(*key, random::<u64>())],
                skipped_writes: vec![],
            })
        }
        // One transaction that triggers an abort
        transactions.push(Transaction::SkipRest)
    }
    run_and_assert(transactions)
}
