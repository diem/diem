// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::{account::Account, gas_costs};
use libra_types::transaction::{SignedTransaction, TransactionArgument};
use stdlib::transaction_scripts::{
    ADD_VALIDATOR_TXN, CREATE_ACCOUNT_TXN, MINT_TXN, PEER_TO_PEER_TRANSFER_TXN,
    REGISTER_VALIDATOR_TXN, ROTATE_AUTHENTICATION_KEY_TXN,
};

/// Returns a transaction to add a new validator
pub fn add_validator_txn(
    sender: &Account,
    new_validator: &Account,
    seq_num: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_validator.address()));

    sender.create_signed_txn_with_args(
        ADD_VALIDATOR_TXN.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
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
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U64(initial_amount));

    sender.create_signed_txn_with_args(
        CREATE_ACCOUNT_TXN.clone(),
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
    args.push(TransactionArgument::U8Vector(receiver.auth_key_prefix()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        PEER_TO_PEER_TRANSFER_TXN.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,                       // this is a default for gas
    )
}

/// Returns a transaction to register the sender as a candidate validator
pub fn register_validator_txn(
    sender: &Account,
    consensus_pubkey: Vec<u8>,
    validator_network_signing_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
    seq_num: u64,
) -> SignedTransaction {
    let args = vec![
        TransactionArgument::U8Vector(consensus_pubkey),
        TransactionArgument::U8Vector(validator_network_signing_pubkey),
        TransactionArgument::U8Vector(validator_network_identity_pubkey),
        TransactionArgument::U8Vector(validator_network_address),
        TransactionArgument::U8Vector(fullnodes_network_identity_pubkey),
        TransactionArgument::U8Vector(fullnodes_network_address),
    ];
    sender.create_signed_txn_with_args(
        REGISTER_VALIDATOR_TXN.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_key_txn(sender: &Account, new_key_hash: Vec<u8>, seq_num: u64) -> SignedTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    sender.create_signed_txn_with_args(
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
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
    args.push(TransactionArgument::U8Vector(receiver.auth_key_prefix()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        MINT_TXN.clone(),
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,                       // this is a default for gas
    )
}
