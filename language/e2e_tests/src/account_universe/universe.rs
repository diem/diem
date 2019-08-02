// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Logic for account universes. This is not in the parent module to enforce privacy.

use crate::{
    account::AccountData,
    account_universe::{default_num_accounts, default_num_transactions, AccountCurrent},
    executor::FakeExecutor,
};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
};
use proptest_derive::Arbitrary;
use proptest_helpers::{pick_slice_idxs, Index};

/// A set of accounts which can be used to construct an initial state.
///
/// For more, see the [`account_universe` module documentation][self].
#[derive(Clone, Debug)]
pub struct AccountUniverseGen {
    accounts: Vec<AccountData>,
}

/// A set of accounts that has been set up and can now be used to conduct transactions on.
///
/// For more, see the [`account_universe` module documentation][self].
#[derive(Clone, Debug)]
pub struct AccountUniverse {
    accounts: Vec<AccountCurrent>,
    /// Whether to ignore any new accounts that transactions add to the universe.
    ignore_new_accounts: bool,
}

/// Allows pairs of accounts to be uniformly randomly selected from an account universe.
#[derive(Arbitrary, Clone, Debug)]
pub struct AccountPairGen {
    pair: [Index; 2],
    // The pick_slice_idx method used by this struct returns values in order, so use this flag
    // to determine whether to reverse it.
    reverse: bool,
}

impl AccountUniverseGen {
    /// Returns a [`Strategy`] that generates a universe of accounts with pre-populated initial
    /// balances.
    pub fn strategy(
        num_accounts: impl Into<SizeRange>,
        balance_strategy: impl Strategy<Value = u64>,
    ) -> impl Strategy<Value = Self> {
        // Pick a sequence number in a smaller range so that valid transactions can be generated.
        // XXX should we also test edge cases around large sequence numbers?
        // Note that using a function as a strategy directly means that shrinking will not occur,
        // but that should be fine because there's nothing to really shrink within accounts anyway.
        vec(AccountData::strategy(balance_strategy), num_accounts)
            .prop_map(|accounts| Self { accounts })
    }

    /// Returns a [`Strategy`] that generates a universe of accounts that's guaranteed to succeed,
    /// assuming that any transfers out of accounts will be 100_000 or below.
    pub fn success_strategy(min_accounts: usize) -> impl Strategy<Value = Self> {
        // Set the minimum balance to be 5x possible transfers out to handle potential gas cost
        // issues.
        let min_balance = (100_000 * (default_num_transactions()) * 5) as u64;
        let max_balance = min_balance * 10;
        Self::strategy(
            min_accounts..default_num_accounts(),
            min_balance..max_balance,
        )
    }

    /// Returns an [`AccountUniverse`] with the initial state generated in this universe.
    pub fn setup(self, executor: &mut FakeExecutor) -> AccountUniverse {
        for account_data in &self.accounts {
            executor.add_account_data(account_data);
        }

        AccountUniverse::new(self.accounts, false)
    }

    /// Returns an [`AccountUniverse`] with the initial state generated in this universe, and
    /// configures the universe to run tests in gas-cost-stability mode.
    ///
    /// The stability mode causes new accounts to be dropped, since those accounts will usually
    /// not be funded enough.
    pub fn setup_gas_cost_stability(self, executor: &mut FakeExecutor) -> AccountUniverse {
        for account_data in &self.accounts {
            executor.add_account_data(account_data);
        }

        AccountUniverse::new(self.accounts, true)
    }
}

impl AccountUniverse {
    fn new(accounts: Vec<AccountData>, ignore_new_accounts: bool) -> Self {
        let accounts = accounts.into_iter().map(AccountCurrent::new).collect();
        Self {
            accounts,
            ignore_new_accounts,
        }
    }

    /// Returns the number of accounts currently in this universe.
    ///
    /// Some transactions might cause new accounts to be created. The return value of this method
    /// will include those new accounts.
    pub fn num_accounts(&self) -> usize {
        self.accounts.len()
    }

    /// Returns the accounts currently in this universe.
    ///
    /// Some transactions might cause new accounts to be created. The return value of this method
    /// will include those new accounts.
    pub fn accounts(&self) -> &[AccountCurrent] {
        &self.accounts
    }

    /// Adds an account to the universe so that future transactions can be made out of this account.
    ///
    /// This is ignored if the universe was configured to be in gas-cost-stability mode.
    pub fn add_account(&mut self, account_data: AccountData) {
        if !self.ignore_new_accounts {
            self.accounts.push(AccountCurrent::new(account_data));
        }
    }

    /// Picks an account using the provided `Index` as a source of randomness.
    pub fn pick_mut(&mut self, index: &Index) -> (usize, &mut AccountCurrent) {
        // TODO: allow alternate picking mechanisms.
        let idx = index.index(self.num_accounts());
        (idx, &mut self.accounts[idx])
    }
}

impl AccountPairGen {
    /// Picks two accounts uniformly randomly from this universe and returns shared references to
    /// them.
    pub fn pick<'a>(&self, universe: &'a AccountUniverse) -> AccountPair<'a> {
        let idxs = pick_slice_idxs(universe.num_accounts(), &self.pair);
        assert_eq!(idxs.len(), 2, "universe should have at least two accounts");
        let (low_idx, high_idx) = (idxs[0], idxs[1]);
        assert_ne!(low_idx, high_idx, "accounts picked must be distinct");

        if self.reverse {
            AccountPair {
                idx_1: high_idx,
                idx_2: low_idx,
                account_1: &universe.accounts[high_idx],
                account_2: &universe.accounts[low_idx],
            }
        } else {
            AccountPair {
                idx_1: low_idx,
                idx_2: high_idx,
                account_1: &universe.accounts[low_idx],
                account_2: &universe.accounts[high_idx],
            }
        }
    }

    /// Picks two accounts uniformly randomly from this universe and returns mutable references to
    /// them.
    pub fn pick_mut<'a>(&self, universe: &'a mut AccountUniverse) -> AccountPairMut<'a> {
        let idxs = pick_slice_idxs(universe.num_accounts(), &self.pair);
        assert_eq!(idxs.len(), 2, "universe should have at least two accounts");
        let (low_idx, high_idx) = (idxs[0], idxs[1]);
        assert_ne!(low_idx, high_idx, "accounts picked must be distinct");
        // Need to use `split_at_mut` because you can't have multiple mutable references to items
        // from a single slice at any given time.
        let (head, tail) = universe.accounts.split_at_mut(low_idx + 1);
        let (low_account, high_account) = (&mut head[low_idx], &mut tail[high_idx - low_idx - 1]);

        if self.reverse {
            AccountPairMut {
                idx_1: high_idx,
                idx_2: low_idx,
                account_1: high_account,
                account_2: low_account,
            }
        } else {
            AccountPairMut {
                idx_1: low_idx,
                idx_2: high_idx,
                account_1: low_account,
                account_2: high_account,
            }
        }
    }
}

/// Shared references to a pair of distinct accounts picked from a universe.
pub struct AccountPair<'a> {
    /// The index of the first account picked.
    pub idx_1: usize,
    /// The index of the second account picked.
    pub idx_2: usize,
    /// A reference to the first account picked.
    pub account_1: &'a AccountCurrent,
    /// A reference to the second account picked.
    pub account_2: &'a AccountCurrent,
}

/// Mutable references to a pair of distinct accounts picked from a universe.
pub struct AccountPairMut<'a> {
    /// The index of the first account picked.
    pub idx_1: usize,
    /// The index of the second account picked.
    pub idx_2: usize,
    /// A mutable reference to the first account picked.
    pub account_1: &'a mut AccountCurrent,
    /// A mutable reference to the second account picked.
    pub account_2: &'a mut AccountCurrent,
}
