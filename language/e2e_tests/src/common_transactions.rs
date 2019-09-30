// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::{account::Account, compile::compile_script, gas_costs};
use lazy_static::lazy_static;
use libra_stdlib::transaction_scripts;
use libra_types::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    transaction::{SignedTransaction, TransactionArgument},
};

lazy_static! {
    /// A serialized transaction to create a new account.
    pub static ref CREATE_ACCOUNT: Vec<u8> = { create_account() };
    /// A serialized transaction to mint new funds.
    pub static ref MINT: Vec<u8> = { mint() };
    /// A serialized transaction to transfer coin from one account to another (possibly new)
    /// one.
    pub static ref PEER_TO_PEER: Vec<u8> = { peer_to_peer() };
    /// A serialized transaction to change the keys for an account.
    pub static ref ROTATE_KEY: Vec<u8> = { rotate_key() };
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn(
    sender: &Account,
    new_account: &Account,
    seq_num: u64,
    initial_amount: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U64(initial_amount));

    sender.create_signed_txn_with_args(
        CREATE_ACCOUNT.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
}

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        PEER_TO_PEER.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,                       // this is a default for gas
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_key_txn(
    sender: &Account,
    new_key_hash: AccountAddress,
    seq_num: u64,
) -> SignedTransaction {
    let args = vec![TransactionArgument::ByteArray(ByteArray::new(
        new_key_hash.to_vec(),
    ))];
    sender.create_signed_txn_with_args(
        ROTATE_KEY.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
}

/// Returns a transaction to mint new funds with the given arguments.
pub fn mint_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        MINT.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,                       // this is a default for gas
    )
}

fn create_account() -> Vec<u8> {
    compile_script(transaction_scripts::create_account())
}

fn mint() -> Vec<u8> {
    compile_script(transaction_scripts::mint())
}

fn peer_to_peer() -> Vec<u8> {
    compile_script(transaction_scripts::peer_to_peer())
}

fn rotate_key() -> Vec<u8> {
    compile_script(transaction_scripts::rotate_key())
}
