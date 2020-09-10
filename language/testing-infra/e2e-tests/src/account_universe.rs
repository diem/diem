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

mod bad_transaction;
mod create_account;
mod peer_to_peer;
mod rotate_key;
mod universe;
pub use bad_transaction::*;
pub use create_account::*;
pub use peer_to_peer::*;
pub use rotate_key::*;
pub use universe::*;

use crate::{
    account::{self, lbr_currency_code, Account, AccountData},
    executor::FakeExecutor,
    gas_costs, transaction_status_eq,
};
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use libra_types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_status::{known_locations, KeptVMStatus, StatusCode},
};
use once_cell::sync::Lazy;
use proptest::{prelude::*, strategy::Union};
use std::{fmt, sync::Arc};

static UNIVERSE_SIZE: Lazy<usize> = Lazy::new(|| {
    use std::{env, process::abort};

    match env::var("UNIVERSE_SIZE") {
        Ok(s) => match s.parse::<usize>() {
            Ok(val) => val,
            Err(err) => {
                println!("Could not parse universe size, aborting: {:?}", err);
                // Abort because Lazy with panics causes poisoning and isn't very
                // helpful overall.
                abort();
            }
        },
        Err(env::VarError::NotPresent) => 20,
        Err(err) => {
            println!(
                "Could not read universe size from the environment, aborting: {:?}",
                err
            );
            abort();
        }
    }
});

/// The number of accounts to run universe-based proptests with. Set with the `UNIVERSE_SIZE`
/// environment variable.
///
/// Larger values will provide greater testing but will take longer to run and shrink. Release mode
/// is recommended for values above 100.
#[inline]
// REVIEW: this was changed from pub(crate) to pub in order to separate the actual tests from
// the e2e tests library. Take a closer look and decide what we really want to do:
//   1. pub(crate) -> pub
//   2. Move this into e2e-testsuite
//   3. Move certain tests back (because they are unit tests)
pub fn default_num_accounts() -> usize {
    *UNIVERSE_SIZE
}

/// The number of transactions to run universe-based proptests with. Set with the `UNIVERSE_SIZE`
/// environment variable (this function will return twice that).
///
/// Larger values will provide greater testing but will take longer to run and shrink. Release mode
/// is recommended for values above 100.
#[inline]
// REVIEW: same as above.
pub fn default_num_transactions() -> usize {
    *UNIVERSE_SIZE * 2
}

/// Represents any sort of transaction that can be done in an account universe.
pub trait AUTransactionGen: fmt::Debug {
    /// Applies this transaction onto the universe, updating balances within the universe as
    /// necessary. Returns a signed transaction that can be run on the VM and the expected values:
    /// the transaction status and the gas used.
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64));

    /// Creates an arced version of this transaction, suitable for dynamic dispatch.
    fn arced(self) -> Arc<dyn AUTransactionGen>
    where
        Self: 'static + Sized,
    {
        Arc::new(self)
    }
}

impl AUTransactionGen for Arc<dyn AUTransactionGen> {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        (**self).apply(universe)
    }
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
    // creation of event counter affects gas usage in create account. This tracks it
    event_counter_created: bool,
}

impl AccountCurrent {
    fn new(initial_data: AccountData) -> Self {
        let balance = initial_data.balance(&lbr_currency_code());
        let sequence_number = initial_data.sequence_number();
        let sent_events_count = initial_data.sent_events_count();
        let received_events_count = initial_data.received_events_count();
        Self {
            initial_data,
            balance,
            sequence_number,
            sent_events_count,
            received_events_count,
            event_counter_created: false,
        }
    }

    /// Returns the underlying account.
    pub fn account(&self) -> &Account {
        &self.initial_data.account()
    }

    /// Rotates the key in this account.
    pub fn rotate_key(&mut self, privkey: Ed25519PrivateKey, pubkey: Ed25519PublicKey) {
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

    /// Returns the gas cost of a create-account transaction.
    pub fn create_account_gas_cost(&self) -> u64 {
        if self.event_counter_created {
            *gas_costs::CREATE_ACCOUNT_NEXT
        } else {
            *gas_costs::CREATE_ACCOUNT_FIRST
        }
    }

    /// Returns the gas cost of a create-account transaction where the sender has an
    /// insufficient balance.
    pub fn create_account_low_balance_gas_cost(&self) -> u64 {
        if self.event_counter_created {
            *gas_costs::CREATE_ACCOUNT_TOO_LOW_NEXT
        } else {
            *gas_costs::CREATE_ACCOUNT_TOO_LOW_FIRST
        }
    }

    /// Returns the gas cost of a create-account transaction where the account exists already.
    pub fn create_existing_account_gas_cost(&self) -> u64 {
        if self.event_counter_created {
            *gas_costs::CREATE_EXISTING_ACCOUNT_NEXT
        } else {
            *gas_costs::CREATE_EXISTING_ACCOUNT_FIRST
        }
    }

    /// Returns the gas cost of a peer-to-peer transaction.
    pub fn peer_to_peer_gas_cost(&self) -> u64 {
        *gas_costs::PEER_TO_PEER
    }

    /// Returns the gas cost of a peer-to-peer transaction with an insufficient balance.
    pub fn peer_to_peer_too_low_gas_cost(&self) -> u64 {
        *gas_costs::PEER_TO_PEER_TOO_LOW
    }

    /// Returns the gas cost of a create-account transaction where the account exists already.
    pub fn peer_to_peer_new_receiver_gas_cost(&self) -> u64 {
        if self.event_counter_created {
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER_NEXT
        } else {
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER_FIRST
        }
    }

    /// Returns the gas cost of a create-account transaction where the account exists already.
    pub fn peer_to_peer_new_receiver_too_low_gas_cost(&self) -> u64 {
        if self.event_counter_created {
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER_TOO_LOW_NEXT
        } else {
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER_TOO_LOW_FIRST
        }
    }

    /// Returns the gas cost of a peer-to-peer transaction with an insufficient balance.
    pub fn rotate_key_gas_cost(&self) -> u64 {
        *gas_costs::ROTATE_KEY
    }
}

/// Computes the result for running a transfer out of one account. Also updates the account to
/// reflect this transaction.
///
/// The return value is a pair of the expected status and whether the transaction was successful.
pub fn txn_one_account_result(
    sender: &mut AccountCurrent,
    amount: u64,
    gas_price: u64,
    gas_used: u64,
    low_gas_used: u64,
) -> (TransactionStatus, bool) {
    // The transactions set the gas cost to 1 microlibra.
    let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED * gas_price;
    // This means that we'll get through the main part of the transaction.
    let enough_to_transfer = sender.balance >= amount;
    let to_deduct = amount + gas_used * gas_price;
    // This means that we'll get through the entire transaction, including the epilogue
    // (where gas costs are deducted).
    let enough_to_succeed = sender.balance >= to_deduct;

    match (enough_max_gas, enough_to_transfer, enough_to_succeed) {
        (true, true, true) => {
            // Success!
            sender.sequence_number += 1;
            sender.sent_events_count += 1;
            sender.balance -= to_deduct;
            (TransactionStatus::Keep(KeptVMStatus::Executed), true)
        }
        (true, true, false) => {
            // Enough gas to pass validation and to do the transfer, but not enough to succeed
            // in the epilogue. The transaction will be run and gas will be deducted from the
            // sender, but no other changes will happen.
            sender.sequence_number += 1;
            sender.balance -= gas_used * gas_price;
            (
                TransactionStatus::Keep(KeptVMStatus::MoveAbort(
                    known_locations::account_module_abort(),
                    6,
                )),
                false,
            )
        }
        (true, false, _) => {
            // Enough gas to pass validation but not enough to succeed. The transaction will
            // be run and gas will be deducted from the sender, but no other changes will
            // happen.
            sender.sequence_number += 1;
            sender.balance -= low_gas_used * gas_price;
            (
                TransactionStatus::Keep(KeptVMStatus::MoveAbort(
                    known_locations::account_module_abort(),
                    10,
                )),
                false,
            )
        }
        (false, _, _) => {
            // Not enough gas to pass validation. Nothing will happen.
            (
                TransactionStatus::Discard(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE),
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

/// A strategy that returns a random transaction.
pub fn all_transactions_strategy(
    min: u64,
    max: u64,
) -> impl Strategy<Value = Arc<dyn AUTransactionGen + 'static>> {
    prop_oneof![
        // Most transactions should be p2p payments.
        8 => p2p_strategy(min, max),
        // TODO: resurrecte once we have unhosted wallets
        //1 => create_account_strategy(min, max),
        1 => any::<RotateKeyGen>().prop_map(RotateKeyGen::arced),
        1 => bad_txn_strategy(),
    ]
}

/// Run these transactions and make sure that they all cost the same amount of gas.
pub fn run_and_assert_gas_cost_stability(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen + Clone>,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup_gas_cost_stability(&mut executor);
    let (transactions, expected_values): (Vec<_>, Vec<_>) = transaction_gens
        .iter()
        .map(|transaction_gen| transaction_gen.clone().apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions).unwrap();

    for (idx, (output, expected_value)) in outputs.iter().zip(&expected_values).enumerate() {
        prop_assert!(
            transaction_status_eq(output.status(), &expected_value.0),
            "unexpected status for transaction {}",
            idx
        );
        prop_assert_eq!(
            output.gas_used(),
            expected_value.1,
            "transaction at idx {} did not have expected gas cost",
            idx,
        );
    }
    Ok(())
}

/// Run these transactions and verify the expected output.
pub fn run_and_assert_universe(
    universe: AccountUniverseGen,
    transaction_gens: Vec<impl AUTransactionGen + Clone>,
) -> Result<(), TestCaseError> {
    let mut executor = FakeExecutor::from_genesis_file();
    let mut universe = universe.setup(&mut executor);
    let (transactions, expected_values): (Vec<_>, Vec<_>) = transaction_gens
        .iter()
        .map(|transaction_gen| transaction_gen.clone().apply(&mut universe))
        .unzip();
    let outputs = executor.execute_block(transactions).unwrap();

    prop_assert_eq!(outputs.len(), expected_values.len());

    for (idx, (output, expected)) in outputs.iter().zip(&expected_values).enumerate() {
        prop_assert!(
            transaction_status_eq(output.status(), &expected.0),
            "unexpected status for transaction {}",
            idx
        );
        executor.apply_write_set(output.write_set());
    }

    assert_accounts_match(&universe, &executor)
}

/// Verify that the account information in the universe matches the information in the executor.
pub fn assert_accounts_match(
    universe: &AccountUniverse,
    executor: &FakeExecutor,
) -> Result<(), TestCaseError> {
    for (idx, account) in universe.accounts().iter().enumerate() {
        let resource = executor
            .read_account_resource(&account.account())
            .expect("account resource must exist");
        let resource_balance = executor
            .read_balance_resource(account.account(), account::lbr_currency_code())
            .expect("account balance resource must exist");
        let auth_key = account.account().auth_key();
        prop_assert_eq!(
            auth_key.as_slice(),
            resource.authentication_key(),
            "account {} should have correct auth key",
            idx
        );
        prop_assert_eq!(
            account.balance(),
            resource_balance.coin(),
            "account {} should have correct balance",
            idx
        );
        // XXX These two don't work at the moment because the VM doesn't bump up event counts.
        //        prop_assert_eq!(
        //            account.received_events_count(),
        //            AccountResource::read_received_events_count(&resource),
        //            "account {} should have correct received_events_count",
        //            idx
        //        );
        //        prop_assert_eq!(
        //            account.sent_events_count(),
        //            AccountResource::read_sent_events_count(&resource),
        //            "account {} should have correct sent_events_count",
        //            idx
        //        );
        prop_assert_eq!(
            account.sequence_number(),
            resource.sequence_number(),
            "account {} should have correct sequence number",
            idx
        );
    }
    Ok(())
}
