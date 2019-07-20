// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A model to test properties of common Libra transactions.
//!
//! The structs and functions in this module together form a simplified *model* of how common Libra
//! transactions should behave. This model is then used as an *oracle* for property-based tests --
//! the results of executing transactions through the VM should match the results computed using
//! this model.
//!
//! For examples of property-based tests written against this model, see the
//! `tests/account_universe` directory.

// clippy warns on the Arbitrary impl for `AccountPairGen` -- it's how Arbitrary works so ignore it.
#![allow(clippy::unit_arg)]

mod create_account;
mod peer_to_peer;
mod rotate_key;
pub use create_account::*;
pub use peer_to_peer::*;
pub use rotate_key::*;

use crate::{
    account::{Account, AccountData},
    executor::FakeExecutor,
    gas_costs,
};
use crypto::{PrivateKey, PublicKey};
use lazy_static::lazy_static;
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
    strategy::Union,
};
use proptest_derive::Arbitrary;
use proptest_helpers::{pick_slice_idxs, Index};
use std::fmt;
use types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};

lazy_static! {
    static ref UNIVERSE_SIZE: usize = {
        use std::{env, process::abort};

        match env::var("UNIVERSE_SIZE") {
            Ok(s) => match s.parse::<usize>() {
                Ok(val) => val,
                Err(err) => {
                    println!("Could not parse universe size, aborting: {:?}", err);
                    // Abort because lazy_static with panics causes poisoning and isn't very
                    // helpful overall.
                    abort();
                }
            }
            Err(env::VarError::NotPresent) => 20,
            Err(err) => {
                println!("Could not read universe size from the environment, aborting: {:?}", err);
                abort();
            }
        }
    };
}

/// The number of accounts to run universe-based proptests with. Set with the `UNIVERSE_SIZE`
/// environment variable.
///
/// Larger values will provide greater testing but will take longer to run and shrink. Release mode
/// is recommended for values above 100.
#[inline]
pub(crate) fn num_accounts() -> usize {
    *UNIVERSE_SIZE
}

/// The number of transactions to run universe-based proptests with. Set with the `UNIVERSE_SIZE`
/// environment variable (this function will return twice that).
///
/// Larger values will provide greater testing but will take longer to run and shrink. Release mode
/// is recommended for values above 100.
#[inline]
pub(crate) fn num_transactions() -> usize {
    *UNIVERSE_SIZE * 2
}

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

/// Represents any sort of transaction that can be done in an account universe.
pub trait AUTransactionGen: fmt::Debug {
    /// Applies this transaction onto the universe, updating balances within the universe as
    /// necessary. Returns a signed transaction that can be run on the VM and the expected output.
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus);

    /// Creates a boxed version of this transaction, suitable for dynamic dispatch.
    fn boxed(self) -> Box<dyn AUTransactionGen>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}

impl AUTransactionGen for Box<dyn AUTransactionGen> {
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
        (**self).apply(universe)
    }
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
        let min_balance = (100_000 * (num_transactions()) * 5) as u64;
        let max_balance = min_balance * 10;
        Self::strategy(min_accounts..num_accounts(), min_balance..max_balance)
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

/// Represents the current state of account in a universe, possibly after its state has been updated
/// by running transactions against the universe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountCurrent {
    initial_data: AccountData,
    balance: u64,
    sequence_number: u64,
    sent_events_count: u64,
    received_events_count: u64,
}

impl AccountCurrent {
    fn new(initial_data: AccountData) -> Self {
        let balance = initial_data.balance();
        let sequence_number = initial_data.sequence_number();
        let sent_events_count = initial_data.sent_events_count();
        let received_events_count = initial_data.received_events_count();
        Self {
            initial_data,
            balance,
            sequence_number,
            sent_events_count,
            received_events_count,
        }
    }

    /// Returns the underlying account.
    pub fn account(&self) -> &Account {
        &self.initial_data.account()
    }

    /// Rotates the key in this account.
    pub fn rotate_key(&mut self, privkey: PrivateKey, pubkey: PublicKey) {
        self.initial_data.rotate_key(privkey, pubkey);
    }

    /// Returns the current balance for this account, assuming all transactions seen so far are
    /// applied.
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Returns the current sequence number for this account, assuming all transactions seen so far
    /// are applied.
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns the current sent events count for this account, assuming all transactions seen so
    /// far are applied.
    pub fn sent_events_count(&self) -> u64 {
        self.sent_events_count
    }

    /// Returns the current received events count for this account, assuming all transactions seen
    /// so far are applied.
    pub fn received_events_count(&self) -> u64 {
        self.received_events_count
    }
}

/// Computes the result for running a transfer out of one account. Also updates the account to
/// reflect this transaction.
///
/// The return value is a pair of the expected status and whether the transaction was successful.
pub fn txn_one_account_result(
    sender: &mut AccountCurrent,
    amount: u64,
    gas_cost: u64,
    low_gas_cost: u64,
) -> (TransactionStatus, bool) {
    // The transactions set the gas cost to 1 microlibra.
    let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
    // This means that we'll get through the main part of the transaction.
    let enough_to_transfer = sender.balance >= amount;
    let to_deduct = amount + gas_cost;
    // This means that we'll get through the entire transaction, including the epilogue
    // (where gas costs are deducted).
    let enough_to_succeed = sender.balance >= to_deduct;

    match (enough_max_gas, enough_to_transfer, enough_to_succeed) {
        (true, true, true) => {
            // Success!
            sender.sequence_number += 1;
            sender.sent_events_count += 1;
            sender.balance -= to_deduct;
            (
                TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)),
                true,
            )
        }
        (true, true, false) => {
            // Enough gas to pass validation and to do the transfer, but not enough to succeed
            // in the epilogue. The transaction will be run and gas will be deducted from the
            // sender, but no other changes will happen.
            sender.sequence_number += 1;
            sender.balance -= gas_cost;
            (
                TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Aborted(6))),
                false,
            )
        }
        (true, false, _) => {
            // Enough gas to pass validation but not enough to succeed. The transaction will
            // be run and gas will be deducted from the sender, but no other changes will
            // happen.
            sender.sequence_number += 1;
            sender.balance -= low_gas_cost;
            (
                TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Aborted(10))),
                false,
            )
        }
        (false, _, _) => {
            // Not enough gas to pass validation. Nothing will happen.
            (
                TransactionStatus::Discard(VMStatus::Validation(
                    VMValidationStatus::InsufficientBalanceForTransactionFee,
                )),
                false,
            )
        }
    }
}

/// Returns a [`Strategy`] that provides a variety of balances (or transfer amounts) over a roughly
/// logarithmic distribution.
pub fn log_balance_strategy(max_balance: u64) -> impl Strategy<Value = u64> {
    // The logarithmic distribution is modeled by uniformly picking from ranges of powers of 2.
    let minimum = gas_costs::TXN_RESERVED.next_power_of_two();
    assert!(max_balance >= minimum, "minimum to make sense");
    let mut strategies = vec![];
    // Balances below and around the minimum are interesting but don't cover *every* power of 2,
    // just those starting from the minimum.
    let mut lower_bound: u64 = 0;
    let mut upper_bound: u64 = minimum;
    loop {
        strategies.push(lower_bound..upper_bound);
        if upper_bound >= max_balance {
            break;
        }
        lower_bound = upper_bound;
        upper_bound = (upper_bound * 2).min(max_balance);
    }
    Union::new(strategies)
}
