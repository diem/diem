// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    proto::transaction::{
        RawTransaction as ProtoRawTransaction, SignedTransaction as ProtoSignedTransaction,
        SignedTransactionsBlock,
    },
    transaction::{
        Program, RawTransaction, RawTransactionBytes, SignatureCheckedTransaction,
        SignedTransaction,
    },
    transaction_helpers::get_signed_transactions_digest,
    write_set::WriteSet,
};
use crypto::hash::CryptoHash;
use nextgen_crypto::{ed25519::*, traits::*};
use proto_conv::{FromProto, IntoProto};
use protobuf::Message;
use std::time::{SystemTime, UNIX_EPOCH};

static PLACEHOLDER_SCRIPT: &[u8] = include_bytes!("fixtures/scripts/placeholder_script.mvbin");

const MAX_GAS_AMOUNT: u64 = 140_000;
const MAX_GAS_PRICE: u64 = 1;

// Test helper for transaction creation
pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Program>,
    expiration_time: u64,
    gas_unit_price: u64,
    max_gas_amount: Option<u64>,
) -> ProtoSignedTransaction {
    let mut raw_txn = ProtoRawTransaction::new();
    raw_txn.set_sender_account(sender.as_ref().to_vec());
    raw_txn.set_sequence_number(sequence_number);
    raw_txn.set_program(program.unwrap_or_else(placeholder_script).into_proto());
    raw_txn.set_expiration_time(expiration_time);
    raw_txn.set_max_gas_amount(max_gas_amount.unwrap_or(MAX_GAS_AMOUNT));
    raw_txn.set_gas_unit_price(gas_unit_price);

    let bytes = raw_txn.write_to_bytes().unwrap();
    let hash = RawTransactionBytes(&bytes).hash();
    let signature = private_key.sign_message(&hash);

    let mut signed_txn = ProtoSignedTransaction::new();
    signed_txn.set_raw_txn_bytes(bytes);
    signed_txn.set_sender_public_key(public_key.to_bytes().to_vec());
    signed_txn.set_sender_signature(signature.to_bytes().to_vec());
    signed_txn
}

// Test helper for creating transactions for which the signature hasn't been checked.
pub fn get_test_unchecked_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Program>,
    expiration_time: u64,
    gas_unit_price: u64,
    max_gas_amount: Option<u64>,
) -> SignedTransaction {
    let mut raw_txn = ProtoRawTransaction::new();
    raw_txn.set_sender_account(sender.as_ref().to_vec());
    raw_txn.set_sequence_number(sequence_number);
    raw_txn.set_program(program.unwrap_or_else(placeholder_script).into_proto());
    raw_txn.set_expiration_time(expiration_time);
    raw_txn.set_max_gas_amount(max_gas_amount.unwrap_or(MAX_GAS_AMOUNT));
    raw_txn.set_gas_unit_price(gas_unit_price);

    let bytes = raw_txn.write_to_bytes().unwrap();
    let hash = RawTransactionBytes(&bytes).hash();
    let signature = private_key.sign_message(&hash);

    SignedTransaction::craft_signed_transaction_for_client(
        RawTransaction::from_proto(raw_txn).unwrap(),
        public_key,
        signature,
    )
}

// Test helper for transaction creation. Short version for get_test_signed_transaction
// Omits some fields
pub fn get_test_signed_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Program>,
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
    program: Option<Program>,
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

pub fn placeholder_script() -> Program {
    Program::new(PLACEHOLDER_SCRIPT.to_vec(), vec![], vec![])
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

// Test helper for transaction block creation
pub fn create_signed_transactions_block(
    sender: AccountAddress,
    starting_sequence_number: u64,
    num_transactions_in_block: u64,
    priv_key: &Ed25519PrivateKey,
    pub_key: &Ed25519PublicKey,
    validator_priv_key: &Ed25519PrivateKey,
    validator_pub_key: &Ed25519PublicKey,
) -> SignedTransactionsBlock {
    let mut signed_txns_block = SignedTransactionsBlock::new();
    for i in starting_sequence_number..(starting_sequence_number + num_transactions_in_block) {
        // Add some transactions to the block
        signed_txns_block.transactions.push(get_test_signed_txn(
            sender,
            i, /* seq_number */
            priv_key.clone(),
            pub_key.clone(),
            None,
        ));
    }

    let message = get_signed_transactions_digest(&signed_txns_block.transactions);
    let signature = validator_priv_key.sign_message(&message);
    signed_txns_block.set_validator_signature(signature.to_bytes().to_vec());
    signed_txns_block.set_validator_public_key(validator_pub_key.to_bytes().to_vec());

    signed_txns_block
}
