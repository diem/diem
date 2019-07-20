// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Gas costs for common transactions.

use crate::{
    account::{Account, AccountData},
    common_transactions::{create_account_txn, peer_to_peer_txn, rotate_key_txn},
    executor::FakeExecutor,
};
use lazy_static::lazy_static;
use types::{account_address::AccountAddress, transaction::SignedTransaction};

/// The gas each transaction is configured to reserve. If the gas available in the account,
/// converted to microlibra, falls below this threshold, transactions are expected to fail with
/// an insufficient balance.
pub const TXN_RESERVED: u64 = 100_000;

lazy_static! {
    /// The gas cost of a create-account transaction.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref CREATE_ACCOUNT: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        let receiver = Account::new();

        let txn = create_account_txn(sender.account(), &receiver, 10, 20_000);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a create-account transaction where the sender has an insufficient balance.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref CREATE_ACCOUNT_TOO_LOW: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        // The gas amount is the minimum that needs to be reserved, so use a value that's
        // clearly higher than that.
        let balance = TXN_RESERVED + 10_000;
        let sender = AccountData::new(balance, 10);
        executor.add_account_data(&sender);
        let receiver = Account::new();

        let txn = create_account_txn(sender.account(), &receiver, 10, balance + 1);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a create-account transaction where the receiver already exists.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref CREATE_EXISTING_ACCOUNT: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        let sender = AccountData::new(1_000_000, 10);
        let receiver = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        let txn = create_account_txn(sender.account(), receiver.account(), 10, 20_000);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a peer-to-peer transaction.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref PEER_TO_PEER: u64 = {
        // Compute gas used by running a placeholder transaction.
        let mut executor = FakeExecutor::from_genesis_file();
        let sender = AccountData::new(1_000_000, 10);
        let receiver = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, 20_000);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a peer-to-peer transaction with an insufficient balance.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref PEER_TO_PEER_TOO_LOW: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        // The gas amount is the minimum that needs to be reserved, so use a value that's clearly
        // higher than that.
        let balance = TXN_RESERVED + 10_000;
        let sender = AccountData::new(balance, 10);
        let receiver = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, balance + 1);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a peer-to-peer transaction that creates a new account.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref PEER_TO_PEER_NEW_RECEIVER: u64 = {
        // Compute gas used by running a placeholder transaction.
        let mut executor = FakeExecutor::from_genesis_file();
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        let receiver = Account::new();

        let txn = peer_to_peer_txn(sender.account(), &receiver, 10, 20_000);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a peer-to-peer transaction that tries to create a new account, but fails
    /// because of an insufficient balance.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref PEER_TO_PEER_NEW_RECEIVER_TOO_LOW: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        // The gas amount is the minimum that needs to be reserved, so use a value that's
        // clearly higher than that.
        let balance = TXN_RESERVED + 10_000;
        let sender = AccountData::new(balance, 10);
        executor.add_account_data(&sender);
        let receiver = Account::new();

        let txn = peer_to_peer_txn(sender.account(), &receiver, 10, balance + 1);
        compute_gas_used(txn, &mut executor)
    };

    /// The gas cost of a rotate-key transaction.
    ///
    /// All such transactions are expected to cost the same gas.
    pub static ref ROTATE_KEY: u64 = {
        let mut executor = FakeExecutor::from_genesis_file();
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);
        let (_privkey, pubkey) = crypto::signing::generate_keypair();
        let new_key_hash = AccountAddress::from(pubkey);

         let txn = rotate_key_txn(sender.account(), new_key_hash, 10);
         compute_gas_used(txn, &mut executor)
    };
}

fn compute_gas_used(txn: SignedTransaction, executor: &mut FakeExecutor) -> u64 {
    let output = &executor.execute_block(vec![txn])[0];
    output.gas_used()
}
