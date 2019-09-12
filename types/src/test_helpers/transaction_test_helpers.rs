// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    proto::transaction::SignedTransaction as ProtoSignedTransaction,
    transaction::{Module, RawTransaction, Script, SignatureCheckedTransaction, SignedTransaction},
    write_set::WriteSet,
};
use crypto::{ed25519::*, hash::CryptoHash, traits::*};
use proto_conv::IntoProto;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static PLACEHOLDER_SCRIPT: &[u8] = include_bytes!("fixtures/scripts/placeholder_script.mvbin");

const MAX_GAS_AMOUNT: u64 = 140_000;
const MAX_GAS_PRICE: u64 = 1;

// Test helper for transaction creation
pub fn get_test_signed_module_publishing_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    module: Module,
) -> ProtoSignedTransaction {
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 10;
    let raw_txn = RawTransaction::new_module(
        sender,
        sequence_number,
        module,
        MAX_GAS_AMOUNT,
        MAX_GAS_PRICE,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.hash());

    SignedTransaction::new(raw_txn, public_key, signature).into_proto()
}

// Test helper for transaction creation
pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
    expiration_time: u64,
    gas_unit_price: u64,
    max_gas_amount: Option<u64>,
) -> ProtoSignedTransaction {
    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        program.unwrap_or_else(placeholder_script),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.hash());

    SignedTransaction::new(raw_txn, public_key, signature).into_proto()
}

// Test helper for creating transactions for which the signature hasn't been checked.
pub fn get_test_unchecked_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
    expiration_time: u64,
    gas_unit_price: u64,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        program.unwrap_or_else(placeholder_script),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.hash());

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for transaction creation. Short version for get_test_signed_transaction
// Omits some fields
pub fn get_test_signed_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
) -> ProtoSignedTransaction {
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 10; // 10 seconds from now.
    get_test_signed_transaction(
        sender,
        sequence_number,
        private_key,
        public_key,
        program,
        expiration_time,
        MAX_GAS_PRICE,
        None,
    )
}

pub fn get_test_unchecked_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
) -> SignedTransaction {
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 10; // 10 seconds from now.
    get_test_unchecked_transaction(
        sender,
        sequence_number,
        private_key,
        public_key,
        program,
        expiration_time,
        MAX_GAS_PRICE,
        None,
    )
}

pub fn placeholder_script() -> Script {
    Script::new(PLACEHOLDER_SCRIPT.to_vec(), vec![])
}

pub fn get_write_set_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    write_set: Option<WriteSet>,
) -> SignatureCheckedTransaction {
    let write_set = write_set.unwrap_or_default();
    RawTransaction::new_write_set(sender, sequence_number, write_set)
        .sign(&private_key, public_key)
        .unwrap()
}
