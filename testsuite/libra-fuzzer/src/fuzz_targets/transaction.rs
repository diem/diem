// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use language_e2e_tests::account_universe::{
    all_transactions_strategy, log_balance_strategy, run_and_assert_universe, AUTransactionGen,
    AccountUniverseGen,
};
use libra_proptest_helpers::ValueGenerator;
use libra_types::transaction::SignedTransaction;
use proptest::{collection::vec, prelude::*};

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
        "SignedTransaction (LCS deserializer)"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let value = gen.generate(any_with::<SignedTransaction>(()));
        Some(lcs::to_bytes(&value).expect("serialization should work"))
    }

    fn fuzz(&self, data: &[u8]) {
        let _: Result<SignedTransaction, _> = lcs::from_bytes(&data);
    }
}
