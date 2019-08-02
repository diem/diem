// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    account_universe::{
        txn_one_account_result, AUTransactionGen, AccountPair, AccountPairGen, AccountUniverse,
    },
    common_transactions::create_account_txn,
    gas_costs,
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use proptest_helpers::Index;
use types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};

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
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
        let sender = universe.pick(&self.sender).1;

        let txn = create_account_txn(
            sender.account(),
            &self.new_account,
            sender.sequence_number,
            self.amount,
        );

        let (status, is_success) = txn_one_account_result(
            sender,
            self.amount,
            *gas_costs::CREATE_ACCOUNT,
            *gas_costs::CREATE_ACCOUNT_TOO_LOW,
        );
        if is_success {
            universe.add_account(AccountData::with_account(
                self.new_account.clone(),
                self.amount,
                0,
            ));
        }

        (txn, status)
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
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
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
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
        let status = if enough_max_gas {
            sender.sequence_number += 1;
            sender.balance -= *gas_costs::CREATE_EXISTING_ACCOUNT;
            TransactionStatus::Keep(VMStatus::Execution(
                ExecutionStatus::CannotWriteExistingResource,
            ))
        } else {
            // Not enough gas to get past the prologue.
            TransactionStatus::Discard(VMStatus::Validation(
                VMValidationStatus::InsufficientBalanceForTransactionFee,
            ))
        };

        (txn, status)
    }
}
