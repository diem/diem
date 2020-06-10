// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{
        default_num_transactions, AUTransactionGen, AccountUniverseGen, InsufficientBalanceGen,
        InvalidAuthkeyGen, SequenceNumberMismatchGen,
    },
    tests::account_universe::run_and_assert_gas_cost_stability,
};
use proptest::{collection::vec, prelude::*};
use std::sync::Arc;

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

pub(super) fn bad_txn_strategy() -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        1 => any_with::<SequenceNumberMismatchGen>((0, 10_000)).prop_map(SequenceNumberMismatchGen::arced),
        1 => any_with::<InvalidAuthkeyGen>(()).prop_map(InvalidAuthkeyGen::arced),
        1 => any_with::<InsufficientBalanceGen>((1, 20_000)).prop_map(InsufficientBalanceGen::arced),
    ]
}
