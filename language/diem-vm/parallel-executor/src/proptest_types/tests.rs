// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::ParallelTransactionExecutor,
    proptest_types::types::{
        ExpectedOutput, ImpreciseInferencer, Inferencer, Task, Transaction, TransactionGen,
    },
};
use proptest::{collection::vec, prelude::*, sample::Index, strategy::Strategy};
use std::{fmt::Debug, hash::Hash};

fn run_transactions<K, V>(
    key_universe: Vec<K>,
    transaction_gens: Vec<TransactionGen<V>>,
    abort_transactions: Vec<Index>,
    skip_rest_transactions: Vec<Index>,
    imprecise_read: bool,
) -> bool
where
    K: Hash + Clone + Debug + Eq + Send + Sync + PartialOrd + Ord + 'static,
    V: Clone + Eq + Send + Sync + Arbitrary + 'static,
{
    let mut transactions: Vec<_> = transaction_gens
        .into_iter()
        .map(|txn_gen| txn_gen.materialize(&key_universe))
        .collect();

    let length = transactions.len();

    for i in abort_transactions {
        *transactions.get_mut(i.index(length)).unwrap() = Transaction::Abort;
    }

    for i in skip_rest_transactions {
        *transactions.get_mut(i.index(length)).unwrap() = Transaction::SkipRest;
    }

    let baseline = ExpectedOutput::generate_baseline(&transactions);

    let output = if imprecise_read {
        ParallelTransactionExecutor::<Transaction<K, V>, Task<K, V>, ImpreciseInferencer<K, V>>::new(
            ImpreciseInferencer::new(),
        )
            .execute_transactions_parallel((), transactions)
    } else {
        ParallelTransactionExecutor::<Transaction<K, V>, Task<K, V>, Inferencer<K, V>>::new(
            Inferencer::new(),
        )
        .execute_transactions_parallel((), transactions)
    };

    baseline.check_output(&output)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn no_early_termination(
        universe in vec(any::<[u8; 32]>(), 100),
        transaction_gen in vec(any::<TransactionGen<[u8;32]>>(), 5000).no_shrink(),
        abort_transactions in vec(any::<Index>(), 0),
        skip_rest_transactions in vec(any::<Index>(), 0),
    ) {
        prop_assert!(run_transactions(universe, transaction_gen, abort_transactions, skip_rest_transactions, false));
    }

    #[test]
    fn abort_only(
        universe in vec(any::<[u8; 32]>(), 100),
        transaction_gen in vec(any::<TransactionGen<[u8;32]>>(), 5000).no_shrink(),
        abort_transactions in vec(any::<Index>(), 5),
        skip_rest_transactions in vec(any::<Index>(), 0),
    ) {
        prop_assert!(run_transactions(universe, transaction_gen, abort_transactions, skip_rest_transactions, false));
    }

    #[test]
    fn skip_rest_only(
        universe in vec(any::<[u8; 32]>(), 100),
        transaction_gen in vec(any::<TransactionGen<[u8;32]>>(), 5000).no_shrink(),
        abort_transactions in vec(any::<Index>(), 0),
        skip_rest_transactions in vec(any::<Index>(), 5),
    ) {
        prop_assert!(run_transactions(universe, transaction_gen, abort_transactions, skip_rest_transactions, false));
    }


    #[test]
    fn mixed_transactions(
        universe in vec(any::<[u8; 32]>(), 100),
        transaction_gen in vec(any::<TransactionGen<[u8;32]>>(), 5000).no_shrink(),
        abort_transactions in vec(any::<Index>(), 5),
        skip_rest_transactions in vec(any::<Index>(), 5),
    ) {
        prop_assert!(run_transactions(universe, transaction_gen, abort_transactions, skip_rest_transactions, false));
    }

    #[test]
    fn imprecise_read_estimation(
        universe in vec(any::<[u8; 32]>(), 100),
        transaction_gen in vec(any::<TransactionGen<[u8;32]>>(), 3000).no_shrink(),
        abort_transactions in vec(any::<Index>(), 5),
        skip_rest_transactions in vec(any::<Index>(), 5),
    ) {
        prop_assert!(run_transactions(universe, transaction_gen, abort_transactions, skip_rest_transactions, true));
    }
}
