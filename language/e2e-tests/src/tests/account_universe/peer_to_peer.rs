// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{
        default_num_accounts, default_num_transactions, log_balance_strategy, AUTransactionGen,
        AccountUniverseGen, P2PNewReceiverGen, P2PTransferGen,
    },
    tests::account_universe::{run_and_assert_gas_cost_stability, run_and_assert_universe},
};
use proptest::{collection::vec, prelude::*};
use std::sync::Arc;

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Need a minimum of two accounts to send p2p transactions over.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn p2p_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(2),
        transfers in vec(any_with::<P2PTransferGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers)?;
    }

    #[test]
    fn p2p_high_balance(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            1_000_000u64..10_000_000,
        ),
        transfers in vec(any_with::<P2PTransferGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Test with balances small enough to possibly trigger failures.
    #[test]
    fn p2p_low_balance(
        universe in AccountUniverseGen::strategy(2..default_num_accounts(), 0u64..100_000),
        transfers in vec(any_with::<P2PTransferGen>((1, 50_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    // Need a minimum of one account to send p2p transactions to other accounts.
    // Set balances high enough that transactions will always succeed.
    #[test]
    fn p2p_new_receiver_gas_cost_stability(
        universe in AccountUniverseGen::success_strategy(1),
        transfers in vec(any_with::<P2PNewReceiverGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_gas_cost_stability(universe, transfers)?;
    }

    /// Test that p2p transfers can be done to new accounts.
    #[test]
    fn p2p_new_receiver_high_balance(
        universe in AccountUniverseGen::strategy(
            1..default_num_accounts(),
            1_000_000u64..10_000_000,
        ),
        transfers in vec(any_with::<P2PNewReceiverGen>((1, 10_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Test with balances small enough to possibly trigger failures.
    #[test]
    fn p2p_new_receiver_low_balance(
        universe in AccountUniverseGen::strategy(1..default_num_accounts(), 0u64..100_000),
        transfers in vec(any_with::<P2PNewReceiverGen>((1, 50_000)), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }

    /// Mixed tests with all the different kinds of peer to peer transactions and a large
    /// variety of balances.
    #[test]
    fn p2p_mixed(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        transfers in vec(p2p_strategy(1, 1_000_000), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transfers)?;
    }
}

pub(super) fn p2p_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        3 => any_with::<P2PTransferGen>((min, max)).prop_map(P2PTransferGen::arced),
        1 => any_with::<P2PNewReceiverGen>((min, max)).prop_map(P2PNewReceiverGen::arced),
    ]
}
