// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{lbr_currency_code, Account, AccountData, AccountTypeSpecifier},
    account_universe::{
        txn_one_account_result, AUTransactionGen, AccountPair, AccountPairGen, AccountUniverse,
    },
    common_transactions::create_account_txn,
    gas_costs,
};
use libra_proptest_helpers::Index;
use libra_types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

/// Represents a create-account transaction performed in the account universe.
///
/// The parameters are the minimum and maximum balances to transfer.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct CreateAccountGen {
    sender: Index,
    new_account: Account,
    #[proptest(strategy = "params.0 ..= params.1")]
    amount: u64,
}

impl AUTransactionGen for CreateAccountGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        let txn = create_account_txn(
            sender.account(),
            &self.new_account,
            sender.sequence_number,
            self.amount,
        );

        let mut gas_cost = sender.create_account_gas_cost();
        let low_balance_gas_cost = sender.create_account_low_balance_gas_cost();

        let (status, is_success) =
            txn_one_account_result(sender, self.amount, gas_cost, low_balance_gas_cost);
        if is_success {
            sender.event_counter_created = true;
            universe.add_account(AccountData::with_account(
                self.new_account.clone(),
                self.amount,
                lbr_currency_code(),
                0,
                AccountTypeSpecifier::default(),
            ));
        } else {
            gas_cost = 0;
        }

        (txn, (status, gas_cost))
    }
}

/// Represents a create-account transaction in the account universe where the destination already
/// exists.
///
/// The parameters are the minimum and maximum balances to transfer.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct CreateExistingAccountGen {
    sender_receiver: AccountPairGen,
    #[proptest(strategy = "params.0 ..= params.1")]
    amount: u64,
}

impl AUTransactionGen for CreateExistingAccountGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let AccountPair {
            account_1: sender,
            account_2: receiver,
            ..
        } = self.sender_receiver.pick(universe);

        let txn = create_account_txn(
            sender.account(),
            receiver.account(),
            sender.sequence_number,
            self.amount,
        );

        // This transaction should never work, but it will fail differently if there's not enough
        // gas to reserve.
        let mut gas_cost = 0;
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
        let status = if enough_max_gas {
            sender.sequence_number += 1;
            gas_cost = sender.create_existing_account_gas_cost();
            sender.balance -= gas_cost;
            TransactionStatus::Keep(VMStatus::new(StatusCode::CANNOT_WRITE_EXISTING_RESOURCE))
        } else {
            // Not enough gas to get past the prologue.
            TransactionStatus::Discard(VMStatus::new(
                StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
            ))
        };

        (txn, (status, gas_cost))
    }
}
