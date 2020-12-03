// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO: all of these tests rely on a symmetric account creation mechanism; that is, an account of
// type T that can create another account of type T. This does not exist in the current system, but
// will exist once we introduce unhosted wallets. Will bring these back once we enable unhosted
// wallets
/*proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Need a minimum of one account for create_account.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn create_account_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(1),
        transfers in vec(any_with::<CreateAccountGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers)?;
    }

    #[test]
    fn create_account_high_balance(
        universe in AccountUniverseGen::strategy(1..default_num_accounts(), 1_000_000u64..10_000_000),
        transfers in vec(any_with::<CreateAccountGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Test with balances small enough to possibly trigger failures.
    #[test]
    fn create_account_low_balance(
        universe in AccountUniverseGen::strategy(1..default_num_accounts(), 0u64..100_000),
        transfers in vec(any_with::<CreateAccountGen>((1, 50_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    // Need a minimum of two accounts for create account with existing receiver.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn create_existing_account_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(2),
        transfers in vec(
            any_with::<CreateExistingAccountGen>((1, 10_000)),
            0..default_num_transactions(),
        ),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers)?;
    }

    #[test]
    fn create_existing_account(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        transfers in vec(
            any_with::<CreateExistingAccountGen>((1, 1_000_000)),
            0..default_num_transactions(),
        ),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Mixed tests with the different kinds of create-account transactions and a large variety
    /// of balances.
    #[test]
    fn create_account_mixed(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        transfers in vec(create_account_strategy(1, 1_000_000), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }
}*/
