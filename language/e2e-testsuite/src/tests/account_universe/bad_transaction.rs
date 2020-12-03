// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::account_universe::{
    default_num_transactions, run_and_assert_gas_cost_stability, AccountUniverseGen,
    InsufficientBalanceGen, InvalidAuthkeyGen, SequenceNumberMismatchGen,
};
use proptest::{collection::vec, prelude::*};

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn bad_sequence(
        universe in AccountUniverseGen::success_strategy(2),
        txns in vec(any_with::<SequenceNumberMismatchGen>((0, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, txns)?;
    }

    #[test]
    fn bad_auth_key(
        universe in AccountUniverseGen::success_strategy(2),
        txns in vec(any_with::<InvalidAuthkeyGen>(()), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, txns)?;
    }

    #[test]
    fn insufficient_balance(
        universe in AccountUniverseGen::success_strategy(2),
        txns in vec(any_with::<InsufficientBalanceGen>((1, 10_001)), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, txns)?;
    }
}
