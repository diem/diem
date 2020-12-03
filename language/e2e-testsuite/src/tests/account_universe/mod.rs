// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod bad_transaction;
mod create_account;
mod peer_to_peer;
mod rotate_key;

use language_e2e_tests::{
    account_universe::{
        all_transactions_strategy, default_num_accounts, default_num_transactions,
        log_balance_strategy, run_and_assert_universe, AccountCurrent, AccountPairGen,
        AccountPickStyle, AccountUniverseGen,
    },
    executor::FakeExecutor,
};
use proptest::{collection::vec, prelude::*};

proptest! {
    // These tests are pretty slow but quite comprehensive, so run a smaller number of them.
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Ensure that account pair generators return the correct indexes.
    #[test]
    fn account_pair_gen(
        universe in AccountUniverseGen::strategy(2..default_num_accounts(), 0u64..10000),
        pairs in vec(any::<AccountPairGen>(), 0..default_num_transactions()),
    ) {
        let mut executor = FakeExecutor::from_genesis_file();
        let mut universe = universe.setup(&mut executor);

        for pair in pairs {
            let (idx_1, idx_2, account_1, account_2) = {
                let pick = pair.pick(&mut universe);
                (
                    pick.idx_1,
                    pick.idx_2,
                    // Need to convert to raw pointers to avoid holding a mutable reference
                    // (pick_mut below borrows universe mutably, which would conflict.)
                    // This is safe as all we're doing is comparing pointer equality.
                    pick.account_1 as *const AccountCurrent,
                    pick.account_2 as *const AccountCurrent,
                )
            };

            prop_assert_eq!(account_1, &universe.accounts()[idx_1] as *const AccountCurrent);
            prop_assert_eq!(account_2, &universe.accounts()[idx_2] as *const AccountCurrent);
        }
    }

    #[test]
    fn all_transactions(
        universe in AccountUniverseGen::strategy(
            2..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        transactions in vec(all_transactions_strategy(1, 1_000_000), 0..default_num_transactions()),
    ) {
        run_and_assert_universe(universe, transactions)?;
    }

    #[test]
    fn all_transactions_limited(
        mut universe in AccountUniverseGen::strategy(
            4..default_num_accounts(),
            log_balance_strategy(10_000_000),
        ),
        mut transactions in vec(
            all_transactions_strategy(1, 1_000_000),
            0..default_num_transactions(),
        ),
    ) {
        universe.set_pick_style(AccountPickStyle::Limited(4));
        // Each transaction consumes up to 2 slots, and there are (4 * universe.num_accounts())
        // slots. Use only 3/4 of the slots to allow for some tolerance against edge cases. So
        // the maximum number of transactions is (3 * universe.num_accounts()) / 2.
        let max_transactions = (3 * universe.num_accounts()) / 2;
        if transactions.len() >= max_transactions {
            transactions.drain(max_transactions..);
        }

        run_and_assert_universe(universe, transactions)?;
    }
}
