// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    account_universe::{
        txn_one_account_result, AUTransactionGen, AccountPair, AccountPairGen, AccountUniverse,
    },
    common_transactions::peer_to_peer_txn,
    gas_costs,
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use proptest_helpers::Index;
use types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};

/// Represents a peer-to-peer transaction performed in the account universe.
///
/// The parameters are the minimum and maximum balances to transfer.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct P2PTransferGen {
    sender_receiver: AccountPairGen,
    #[proptest(strategy = "params.0 ..= params.1")]
    amount: u64,
}

/// Represents a peer-to-peer transaction performed in the account universe to a new receiver.
///
/// The parameters are the minimum and maximum balances to transfer.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(params = "(u64, u64)")]
pub struct P2PNewReceiverGen {
    sender: Index,
    receiver: Account,
    #[proptest(strategy = "params.0 ..= params.1")]
    amount: u64,
}

impl AUTransactionGen for P2PTransferGen {
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
        let AccountPair {
            account_1: sender,
            account_2: receiver,
            ..
        } = self.sender_receiver.pick(universe);

        let txn = peer_to_peer_txn(
            sender.account(),
            receiver.account(),
            sender.sequence_number,
            self.amount,
        );

        // Now figure out whether the transaction will actually work. (The transactions set the
        // gas cost to 1 microlibra.)
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
        // This means that we'll get through the main part of the transaction.
        let enough_to_transfer = sender.balance >= self.amount;
        let to_deduct = self.amount + *gas_costs::PEER_TO_PEER;
        // This means that we'll get through the entire transaction, including the epilogue
        // (where gas costs are deducted).
        let enough_to_succeed = sender.balance >= to_deduct;

        // Expect a failure if the amount is greater than the current balance.
        // XXX return the failure somehow?
        let status;
        match (enough_max_gas, enough_to_transfer, enough_to_succeed) {
            (true, true, true) => {
                // Success!
                sender.sequence_number += 1;
                sender.sent_events_count += 1;
                sender.balance -= to_deduct;

                receiver.balance += self.amount;
                receiver.received_events_count += 1;

                status = TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed));
            }
            (true, true, false) => {
                // Enough gas to pass validation and to do the transfer, but not enough to succeed
                // in the epilogue. The transaction will be run and gas will be deducted from the
                // sender, but no other changes will happen.
                sender.sequence_number += 1;
                sender.balance -= *gas_costs::PEER_TO_PEER;
                // 6 means the balance was insufficient while trying to deduct gas costs in the
                // epilogue.
                // TODO: define these values in a central location
                status = TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Aborted(6)));
            }
            (true, false, _) => {
                // Enough to pass validation but not to do the transfer. The transaction will be run
                // and gas will be deducted from the sender, but no other changes will happen.
                sender.sequence_number += 1;
                sender.balance -= *gas_costs::PEER_TO_PEER_TOO_LOW;
                // 10 means the balance was insufficient while trying to transfer.
                status = TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Aborted(10)));
            }
            (false, _, _) => {
                // Not enough gas to pass validation. Nothing will happen.
                status = TransactionStatus::Discard(VMStatus::Validation(
                    VMValidationStatus::InsufficientBalanceForTransactionFee,
                ));
            }
        }

        (txn, status)
    }
}

impl AUTransactionGen for P2PNewReceiverGen {
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
        let sender = universe.pick(&self.sender).1;

        // Create a new, nonexistent account for the receiver.
        let txn = peer_to_peer_txn(
            sender.account(),
            &self.receiver,
            sender.sequence_number,
            self.amount,
        );

        let (status, is_success) = txn_one_account_result(
            sender,
            self.amount,
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER,
            *gas_costs::PEER_TO_PEER_NEW_RECEIVER_TOO_LOW,
        );
        if is_success {
            universe.add_account(AccountData::with_account(
                self.receiver.clone(),
                self.amount,
                0,
            ));
        }

        (txn, status)
    }
}
