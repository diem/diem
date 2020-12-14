// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_proptest_helpers::ValueGenerator;
use diem_types::transaction::SignedTransaction;
use language_e2e_tests::account_universe::{
    all_transactions_strategy, log_balance_strategy, run_and_assert_universe, AUTransactionGen,
    AccountUniverseGen,
};
use once_cell::sync::Lazy;
use proptest::{
    collection::vec,
    prelude::*,
    strategy::{Strategy, ValueTree},
    test_runner::{self, RngAlgorithm, TestRunner},
};

#[derive(Clone, Debug, Default)]
pub struct LanguageTransactionExecution;

impl FuzzTargetImpl for LanguageTransactionExecution {
    fn description(&self) -> &'static str {
        "Language execute randomly generated transactions"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(run_and_assert_universe_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let (universe, txns) = fuzz_data_to_value(data, run_and_assert_universe_input());
        run_and_assert_universe(universe, txns).unwrap();
    }
}

prop_compose! {
    fn run_and_assert_universe_input()(
        universe in AccountUniverseGen::strategy(2..20, log_balance_strategy(10_000_000)),
        txns in vec(all_transactions_strategy(1, 1_000_000), 1..40)
    ) -> (AccountUniverseGen, Vec<impl AUTransactionGen + Clone>) {
        (universe, txns)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SignedTransactionTarget;

impl FuzzTargetImpl for SignedTransactionTarget {
    fn description(&self) -> &'static str {
        "SignedTransaction (BCS deserializer)"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let value = gen.generate(any_with::<SignedTransaction>(()));
        Some(bcs::to_bytes(&value).expect("serialization should work"))
    }

    fn fuzz(&self, data: &[u8]) {
        let _: Result<SignedTransaction, _> = bcs::from_bytes(&data);
    }
}

/// This fuzzer ensures that we cannot mutate the serialization of a test transaction/
/// To do this, it generates a single transaction via a seeded proptest and serializes it,
/// The fuzzer then mutates this serialized transaction in hope of deserializing it to the same transaction.
#[derive(Clone, Debug, Default)]
pub struct MutatedSignedTransaction;

static SIGNED_TXN: Lazy<SignedTransaction> = Lazy::new(|| {
    let seed = [0u8; 32];
    let recorder_rng = test_runner::TestRng::from_seed(RngAlgorithm::ChaCha, &seed);
    let mut runner = TestRunner::new_with_rng(test_runner::Config::default(), recorder_rng);
    SignedTransaction::arbitrary()
        .new_tree(&mut runner)
        .expect("creating a new value should succeed")
        .current()
});

static SERIALIZED_SIGNED_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| bcs::to_bytes(&SIGNED_TXN.clone()).expect("serialization should work"));

impl FuzzTargetImpl for MutatedSignedTransaction {
    fn description(&self) -> &'static str {
        "SignedTransaction (BCS serialized -> mutation -> deserializer)"
    }

    /// We always return the same serialized signed transaction for corpus generation,
    /// as we only fuzz the serialization of a unique transaction.
    /// This is quite a limited fuzzer, refer to TwoSignedTransactions for a different approach.
    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(SERIALIZED_SIGNED_TXN.clone())
    }

    fn fuzz(&self, data: &[u8]) {
        // it is possible that the fuzzer sends the same serialization,
        // which is not useful as we know that it'll lead to the same signed transaction
        if data == SERIALIZED_SIGNED_TXN.as_slice() {
            return;
        }

        if let Ok(signed_txn) = bcs::from_bytes::<SignedTransaction>(&data) {
            assert_ne!(*SIGNED_TXN, signed_txn);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TwoSignedTransactions;

impl FuzzTargetImpl for TwoSignedTransactions {
    fn description(&self) -> &'static str {
        "Two different SignedTransactions serialization -> different SignedTransactions"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let txn = gen.generate(any_with::<SignedTransaction>(()));
        let mut serialized_txn = bcs::to_bytes(&txn).expect("serialization should work");
        // return [serialized_txn | serialized_txn]
        serialized_txn.extend_from_slice(&serialized_txn.clone());
        Some(serialized_txn)
    }

    fn fuzz(&self, data: &[u8]) {
        if data.is_empty() || data.len() % 2 != 0 {
            // can't cut data in two equal parts
            return;
        }

        // data is two similar serialized transactions [txn1 | txn2]
        let (txn1, txn2) = data.split_at(data.len() / 2);

        // ensure the parts are different
        if txn1 == txn2 {
            return;
        }

        // ensure the deserialization is different
        if let Ok(txn1) = bcs::from_bytes::<SignedTransaction>(txn1) {
            if let Ok(txn2) = bcs::from_bytes::<SignedTransaction>(txn2) {
                assert_ne!(txn1, txn2);
            }
        }
    }
}
