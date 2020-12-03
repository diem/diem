// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Logic for account universes. This is not in the parent module to enforce privacy.

use crate::{
    account::AccountData,
    account_universe::{default_num_accounts, default_num_transactions, AccountCurrent},
    executor::FakeExecutor,
};
use diem_proptest_helpers::{pick_slice_idxs, Index};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
};
use proptest_derive::Arbitrary;

/// A set of accounts which can be used to construct an initial state.
///
/// For more, see the [`account_universe` module documentation][self].
#[derive(Clone, Debug)]
pub struct AccountUniverseGen {
    accounts: Vec<AccountData>,
    pick_style: AccountPickStyle,
}

/// A set of accounts that has been set up and can now be used to conduct transactions on.
///
/// For more, see the [`account_universe` module documentation][self].
#[derive(Clone, Debug)]
pub struct AccountUniverse {
    accounts: Vec<AccountCurrent>,
    picker: AccountPicker,
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

/// Determines the sampling algorithm used to pick accounts from the universe.
#[derive(Clone, Debug)]
pub enum AccountPickStyle {
    /// An account may be picked as many times as possible.
    Unlimited,
    /// An account may only be picked these many times.
    Limited(usize),
    // TODO: Scaled(usize)
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
        vec(AccountData::strategy(balance_strategy), num_accounts).prop_map(|accounts| Self {
            accounts,
            pick_style: AccountPickStyle::Unlimited,
        })
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

    /// Sets the pick style used by this account universe.
    pub fn set_pick_style(&mut self, pick_style: AccountPickStyle) -> &mut Self {
        self.pick_style = pick_style;
        self
    }

    /// Returns the number of accounts in this account universe.
    pub fn num_accounts(&self) -> usize {
        self.accounts.len()
    }

    /// Returns an [`AccountUniverse`] with the initial state generated in this universe.
    pub fn setup(self, executor: &mut FakeExecutor) -> AccountUniverse {
        for account_data in &self.accounts {
            executor.add_account_data(account_data);
        }

        AccountUniverse::new(self.accounts, self.pick_style, false)
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

        AccountUniverse::new(self.accounts, self.pick_style, true)
    }
}

impl AccountUniverse {
    fn new(
        accounts: Vec<AccountData>,
        pick_style: AccountPickStyle,
        ignore_new_accounts: bool,
    ) -> Self {
        let accounts: Vec<_> = accounts.into_iter().map(AccountCurrent::new).collect();
        let picker = AccountPicker::new(pick_style, accounts.len());

        Self {
            accounts,
            picker,
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
    pub fn pick(&mut self, index: Index) -> (usize, &mut AccountCurrent) {
        let idx = self.picker.pick(index);
        (idx, &mut self.accounts[idx])
    }
}

#[derive(Clone, Debug)]
enum AccountPicker {
    Unlimited(usize),
    // Vector of (index, times remaining).
    Limited(Vec<(usize, usize)>),
    // TODO: Scaled(RepeatVec<usize>)
}

impl AccountPicker {
    fn new(pick_style: AccountPickStyle, num_accounts: usize) -> Self {
        match pick_style {
            AccountPickStyle::Unlimited => AccountPicker::Unlimited(num_accounts),
            AccountPickStyle::Limited(limit) => {
                let remaining = (0..num_accounts).map(|idx| (idx, limit)).collect();
                AccountPicker::Limited(remaining)
            }
        }
    }

    fn pick(&mut self, index: Index) -> usize {
        match self {
            AccountPicker::Unlimited(num_accounts) => index.index(*num_accounts),
            AccountPicker::Limited(remaining) => {
                let remaining_idx = index.index(remaining.len());
                Self::pick_limited(remaining, remaining_idx)
            }
        }
    }

    fn pick_pair(&mut self, indexes: &[Index; 2]) -> [usize; 2] {
        match self {
            AccountPicker::Unlimited(num_accounts) => Self::pick_pair_impl(*num_accounts, indexes),
            AccountPicker::Limited(remaining) => {
                let [remaining_idx_1, remaining_idx_2] =
                    Self::pick_pair_impl(remaining.len(), indexes);
                // Use the later index first to avoid invalidating indexes.
                let account_idx_2 = Self::pick_limited(remaining, remaining_idx_2);
                let account_idx_1 = Self::pick_limited(remaining, remaining_idx_1);

                [account_idx_1, account_idx_2]
            }
        }
    }

    fn pick_pair_impl(max: usize, indexes: &[Index; 2]) -> [usize; 2] {
        let idxs = pick_slice_idxs(max, indexes);
        assert_eq!(idxs.len(), 2);
        let idxs = [idxs[0], idxs[1]];
        assert!(
            idxs[0] < idxs[1],
            "pick_slice_idxs should return sorted order"
        );
        idxs
    }

    fn pick_limited(remaining: &mut Vec<(usize, usize)>, remaining_idx: usize) -> usize {
        let (account_idx, times_remaining) = {
            let (account_idx, times_remaining) = &mut remaining[remaining_idx];
            *times_remaining -= 1;
            (*account_idx, *times_remaining)
        };

        if times_remaining == 0 {
            // Remove the account from further consideration.
            remaining.remove(remaining_idx);
        }

        account_idx
    }
}

impl AccountPairGen {
    /// Picks two accounts uniformly randomly from this universe and returns mutable references to
    /// them.
    pub fn pick<'a>(&self, universe: &'a mut AccountUniverse) -> AccountPair<'a> {
        let [low_idx, high_idx] = universe.picker.pick_pair(&self.pair);
        // Need to use `split_at_mut` because you can't have multiple mutable references to items
        // from a single slice at any given time.
        let (head, tail) = universe.accounts.split_at_mut(low_idx + 1);
        let (low_account, high_account) = (&mut head[low_idx], &mut tail[high_idx - low_idx - 1]);

        if self.reverse {
            AccountPair {
                idx_1: high_idx,
                idx_2: low_idx,
                account_1: high_account,
                account_2: low_account,
            }
        } else {
            AccountPair {
                idx_1: low_idx,
                idx_2: high_idx,
                account_1: low_account,
                account_2: high_account,
            }
        }
    }
}

/// Mutable references to a pair of distinct accounts picked from a universe.
pub struct AccountPair<'a> {
    /// The index of the first account picked.
    pub idx_1: usize,
    /// The index of the second account picked.
    pub idx_2: usize,
    /// A mutable reference to the first account picked.
    pub account_1: &'a mut AccountCurrent,
    /// A mutable reference to the second account picked.
    pub account_2: &'a mut AccountCurrent,
}
