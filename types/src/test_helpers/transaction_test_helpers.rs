// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_config::LBR_NAME,
    transaction::{Module, RawTransaction, Script, SignatureCheckedTransaction, SignedTransaction},
    write_set::WriteSet,
};
use libra_crypto::{ed25519, hash::CryptoHash, traits::*};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_GAS_AMOUNT: u64 = 400_000;
const MAX_GAS_PRICE: u64 = 1;

static EMPTY_SCRIPT: &[u8] =
    include_bytes!("../../../language/stdlib/staged/transaction_scripts/empty_script.mv");

// Test helper for transaction creation
pub fn get_test_signed_module_publishing_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    module: Module,
) -> SignedTransaction {
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
        LBR_NAME.to_string(),
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.hash());

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for transaction creation
pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    script: Option<Script>,
    expiration_time: u64,
    gas_unit_price: u64,
    gas_specifier: String,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        script.unwrap_or_else(|| Script::new(EMPTY_SCRIPT.to_vec(), vec![], Vec::new())),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        gas_specifier,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.hash());

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for creating transactions for which the signature hasn't been checked.
pub fn get_test_unchecked_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    script: Option<Script>,
    expiration_time: u64,
    gas_unit_price: u64,
    gas_specifier: String,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        script.unwrap_or_else(|| Script::new(EMPTY_SCRIPT.to_vec(), vec![], Vec::new())),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        gas_specifier,
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
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    script: Option<Script>,
) -> SignedTransaction {
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
        script,
        expiration_time,
        MAX_GAS_PRICE,
        LBR_NAME.to_string(),
        None,
    )
}

pub fn get_test_unchecked_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    script: Option<Script>,
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
        script,
        expiration_time,
        MAX_GAS_PRICE,
        LBR_NAME.to_string(),
        None,
    )
}

pub fn get_write_set_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &ed25519::PrivateKey,
    public_key: ed25519::PublicKey,
    write_set: Option<WriteSet>,
) -> SignatureCheckedTransaction {
    let write_set = write_set.unwrap_or_default();
    RawTransaction::new_write_set(sender, sequence_number, write_set)
        .sign(&private_key, public_key)
        .unwrap()
}
