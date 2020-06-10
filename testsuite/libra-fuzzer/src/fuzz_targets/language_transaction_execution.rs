// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use language_e2e_tests::account_universe::{
    all_transactions_strategy, log_balance_strategy, run_and_assert_universe, AccountUniverseGen,
};
use libra_proptest_helpers::ValueGenerator;
use proptest::{collection::vec, test_runner};
use rand::RngCore;

#[derive(Clone, Debug, Default)]
pub struct LanguageTransactionExecution;

impl FuzzTargetImpl for LanguageTransactionExecution {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "Language execute randomly generated transactions"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let mut output = vec![0u8; 4096];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut output);
        Some(output)
    }

    fn fuzz(&self, data: &[u8]) {
        let passthrough_rng =
            test_runner::TestRng::from_seed(test_runner::RngAlgorithm::PassThrough, &data);

        let mut generator = ValueGenerator::new_with_rng(passthrough_rng);
        let txn_strategy = vec(all_transactions_strategy(0, 1_000_000), 1..40);

        let txns = generator.generate(txn_strategy);

        let universe_strategy =
            AccountUniverseGen::strategy(2..20, log_balance_strategy(10_000_000));

        let universe = generator.generate(universe_strategy);

        run_and_assert_universe(universe, txns).unwrap();
    }
}
