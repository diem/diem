// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{
        log_balance_strategy, num_accounts, num_transactions, AUTransactionGen, AccountUniverseGen,
        CreateAccountGen, CreateExistingAccountGen,
    },
    gas_costs,
    tests::account_universe::{run_and_assert_gas_cost_stability, run_and_assert_universe},
};
use proptest::{collection::vec, prelude::*};

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Need a minimum of one account for create_account.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn create_account_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(1),
        transfers in vec(any_with::<CreateAccountGen>((1, 10_000)), 0..num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers, *gas_costs::CREATE_ACCOUNT)?;
    }

    #[test]
    fn create_account_high_balance(
        universe in AccountUniverseGen::strategy(1..num_accounts(), 1_000_000u64..10_000_000),
        transfers in vec(any_with::<CreateAccountGen>((1, 10_000)), 0..num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Test with balances small enough to possibly trigger failures.
    #[test]
    fn create_account_low_balance(
        universe in AccountUniverseGen::strategy(1..num_accounts(), 0u64..100_000),
        transfers in vec(any_with::<CreateAccountGen>((1, 50_000)), 0..num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    // Need a minimum of two accounts for create account with existing receiver.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn create_existing_account_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(2),
        transfers in vec(any_with::<CreateExistingAccountGen>((1, 10_000)), 0..num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers, *gas_costs::CREATE_EXISTING_ACCOUNT)?;
    }

    #[test]
    fn create_existing_account(
        universe in AccountUniverseGen::strategy(2..num_accounts(), log_balance_strategy(10_000_000)),
        transfers in vec(any_with::<CreateExistingAccountGen>((1, 1_000_000)), 0..num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Mixed tests with the different kinds of create-account transactions and a large variety
    /// of balances.
    #[test]
    fn create_account_mixed(
        universe in AccountUniverseGen::strategy(2..num_accounts(), log_balance_strategy(10_000_000)),
        transfers in vec(create_account_strategy(1, 1_000_000), 0..num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }
}

pub(super) fn create_account_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Box<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        3 => any_with::<CreateAccountGen>((min, max)).prop_map(CreateAccountGen::boxed),
        1 => any_with::<CreateExistingAccountGen>((min, max)).prop_map(
            CreateExistingAccountGen::boxed,
        ),
    ]
}
