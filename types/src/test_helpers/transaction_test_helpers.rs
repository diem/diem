// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_config::XUS_NAME,
    chain_id::ChainId,
    transaction::{
        authenticator::AccountAuthenticator, Module, RawTransaction, RawTransactionWithData,
        Script, SignatureCheckedTransaction, SignedTransaction, TransactionPayload,
    },
    write_set::WriteSet,
};
use diem_crypto::{ed25519::*, traits::*};

const MAX_GAS_AMOUNT: u64 = 1_000_000;
const TEST_GAS_PRICE: u64 = 0;

static EMPTY_SCRIPT: &[u8] = include_bytes!("empty_script.mv");

// Create an expiration time 'seconds' after now
fn expiration_time(seconds: u64) -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("System time is before the UNIX_EPOCH")
        .as_secs()
        + seconds
}

// Test helper for transaction creation
pub fn get_test_signed_module_publishing_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    module: Module,
) -> SignedTransaction {
    let expiration_time = expiration_time(10);
    let raw_txn = RawTransaction::new_module(
        sender,
        sequence_number,
        module,
        MAX_GAS_AMOUNT,
        TEST_GAS_PRICE,
        XUS_NAME.to_owned(),
        expiration_time,
        ChainId::test(),
    );

    let signature = private_key.sign(&raw_txn);

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for transaction creation
pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    payload: Option<TransactionPayload>,
    expiration_timestamp_secs: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    let raw_txn = RawTransaction::new(
        sender,
        sequence_number,
        payload.unwrap_or_else(|| {
            TransactionPayload::Script(Script::new(EMPTY_SCRIPT.to_vec(), vec![], vec![]))
        }),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        gas_currency_code,
        expiration_timestamp_secs,
        ChainId::test(),
    );

    let signature = private_key.sign(&raw_txn);

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for creating transactions for which the signature hasn't been checked.
pub fn get_test_unchecked_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    script: Option<Script>,
    expiration_time: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    get_test_unchecked_transaction_(
        sender,
        sequence_number,
        private_key,
        public_key,
        script,
        expiration_time,
        gas_unit_price,
        gas_currency_code,
        max_gas_amount,
        ChainId::test(),
    )
}

// Test helper for creating transactions for which the signature hasn't been checked.
fn get_test_unchecked_transaction_(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    script: Option<Script>,
    expiration_timestamp_secs: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    max_gas_amount: Option<u64>,
    chain_id: ChainId,
) -> SignedTransaction {
    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        script.unwrap_or_else(|| Script::new(EMPTY_SCRIPT.to_vec(), vec![], Vec::new())),
        max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
        gas_unit_price,
        gas_currency_code,
        expiration_timestamp_secs,
        chain_id,
    );

    let signature = private_key.sign(&raw_txn);

    SignedTransaction::new(raw_txn, public_key, signature)
}

// Test helper for transaction creation. Short version for get_test_signed_transaction
// Omits some fields
pub fn get_test_signed_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    payload: Option<TransactionPayload>,
) -> SignedTransaction {
    let expiration_time = expiration_time(10);
    get_test_signed_transaction(
        sender,
        sequence_number,
        private_key,
        public_key,
        payload,
        expiration_time,
        TEST_GAS_PRICE,
        XUS_NAME.to_owned(),
        None,
    )
}

pub fn get_test_unchecked_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    script: Option<Script>,
) -> SignedTransaction {
    let expiration_time = expiration_time(10);
    get_test_unchecked_transaction(
        sender,
        sequence_number,
        private_key,
        public_key,
        script,
        expiration_time,
        TEST_GAS_PRICE,
        XUS_NAME.to_owned(),
        None,
    )
}

pub fn get_test_unchecked_multi_agent_txn(
    sender: AccountAddress,
    secondary_signers: Vec<AccountAddress>,
    sequence_number: u64,
    sender_private_key: &Ed25519PrivateKey,
    sender_public_key: Ed25519PublicKey,
    secondary_private_keys: Vec<&Ed25519PrivateKey>,
    secondary_public_keys: Vec<Ed25519PublicKey>,
    script: Option<Script>,
) -> SignedTransaction {
    let expiration_time = expiration_time(10);
    let raw_txn = RawTransaction::new(
        sender,
        sequence_number,
        TransactionPayload::Script(
            script.unwrap_or_else(|| Script::new(EMPTY_SCRIPT.to_vec(), vec![], Vec::new())),
        ),
        MAX_GAS_AMOUNT,
        TEST_GAS_PRICE,
        XUS_NAME.to_owned(),
        expiration_time,
        ChainId::test(),
    );
    let message =
        RawTransactionWithData::new_multi_agent(raw_txn.clone(), secondary_signers.clone());

    let sender_signature = sender_private_key.sign(&message);
    let sender_authenticator = AccountAuthenticator::ed25519(sender_public_key, sender_signature);

    let mut secondary_authenticators = vec![];
    for i in 0..secondary_public_keys.len() {
        let signature = secondary_private_keys[i].sign(&message);
        secondary_authenticators.push(AccountAuthenticator::ed25519(
            secondary_public_keys[i].clone(),
            signature,
        ));
    }

    SignedTransaction::new_multi_agent(
        raw_txn,
        sender_authenticator,
        secondary_signers,
        secondary_authenticators,
    )
}

pub fn get_test_txn_with_chain_id(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    chain_id: ChainId,
) -> SignedTransaction {
    let expiration_time = expiration_time(10);
    get_test_unchecked_transaction_(
        sender,
        sequence_number,
        private_key,
        public_key,
        None,
        expiration_time,
        TEST_GAS_PRICE,
        XUS_NAME.to_owned(),
        None,
        chain_id,
    )
}

pub fn get_write_set_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    write_set: Option<WriteSet>,
) -> SignatureCheckedTransaction {
    let write_set = write_set.unwrap_or_default();
    RawTransaction::new_write_set(sender, sequence_number, write_set, ChainId::test())
        .sign(private_key, public_key)
        .unwrap()
}
