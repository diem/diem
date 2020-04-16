// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::{account::Account, gas_costs};
use libra_types::{
    account_address::AccountAddress,
    account_config::lbr_type_tag,
    transaction::{RawTransaction, SignedTransaction, TransactionArgument},
};
use stdlib::transaction_scripts::StdlibScript;

/// Returns a transaction to add a new validator
pub fn add_validator_txn(
    sender: &Account,
    new_validator: &Account,
    seq_num: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_validator.address()));

    sender.create_signed_txn_with_args(
        StdlibScript::AddValidator.compiled_bytes().into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED * 2,
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
        StdlibScript::CreateAccount.compiled_bytes().into_vec(),
        vec![],
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
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![lbr_type_tag()],
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
        StdlibScript::RegisterValidator.compiled_bytes().into_vec(),
        vec![],
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
        StdlibScript::RotateAuthenticationKey
            .compiled_bytes()
            .into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn raw_rotate_key_txn(
    sender: AccountAddress,
    new_key_hash: Vec<u8>,
    seq_num: u64,
) -> RawTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    Account::create_raw_txn_with_args(
        sender,
        StdlibScript::RotateAuthenticationKey
            .compiled_bytes()
            .into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        1,
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_consensus_pubkey_txn(
    sender: &Account,
    new_key_hash: Vec<u8>,
    seq_num: u64,
) -> SignedTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    sender.create_signed_txn_with_args(
        StdlibScript::RotateConsensusPubkey
            .compiled_bytes()
            .into_vec(),
        vec![],
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
        StdlibScript::Mint.compiled_bytes().into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,                       // this is a default for gas
    )
}
