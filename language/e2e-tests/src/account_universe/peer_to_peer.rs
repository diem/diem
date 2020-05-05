// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{lbr_currency_code, Account, AccountData, AccountTypeSpecifier},
    account_universe::{
        txn_one_account_result, AUTransactionGen, AccountPair, AccountPairGen, AccountUniverse,
    },
    common_transactions::peer_to_peer_txn,
    gas_costs,
};
use libra_proptest_helpers::Index;
use libra_types::{
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

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
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
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
        let to_deduct = self.amount + sender.peer_to_peer_gas_cost();
        let mut gas_cost = 0;
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

                status = TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED));
                gas_cost = sender.peer_to_peer_gas_cost();
            }
            (true, true, false) => {
                // Enough gas to pass validation and to do the transfer, but not enough to succeed
                // in the epilogue. The transaction will be run and gas will be deducted from the
                // sender, but no other changes will happen.
                sender.sequence_number += 1;
                gas_cost = sender.peer_to_peer_gas_cost();
                sender.balance -= gas_cost;
                // 6 means the balance was insufficient while trying to deduct gas costs in the
                // epilogue.
                // TODO: define these values in a central location
                status =
                    TransactionStatus::Keep(VMStatus::new(StatusCode::ABORTED).with_sub_status(6));
            }
            (true, false, _) => {
                // Enough to pass validation but not to do the transfer. The transaction will be run
                // and gas will be deducted from the sender, but no other changes will happen.
                sender.sequence_number += 1;
                gas_cost = sender.peer_to_peer_too_low_gas_cost();
                sender.balance -= gas_cost;
                // 10 means the balance was insufficient while trying to transfer.
                status =
                    TransactionStatus::Keep(VMStatus::new(StatusCode::ABORTED).with_sub_status(10));
            }
            (false, _, _) => {
                // Not enough gas to pass validation. Nothing will happen.
                status = TransactionStatus::Discard(VMStatus::new(
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
                ));
            }
        }

        (txn, (status, gas_cost))
    }
}

impl AUTransactionGen for P2PNewReceiverGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        // Create a new, nonexistent account for the receiver.
        let txn = peer_to_peer_txn(
            sender.account(),
            &self.receiver,
            sender.sequence_number,
            self.amount,
        );

        let mut gas_cost = sender.peer_to_peer_new_receiver_gas_cost();
        let low_balance_gas_cost = sender.peer_to_peer_new_receiver_too_low_gas_cost();

        let (status, is_success) =
            txn_one_account_result(sender, self.amount, gas_cost, low_balance_gas_cost);
        if is_success {
            sender.event_counter_created = true;
            universe.add_account(AccountData::with_account(
                self.receiver.clone(),
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
