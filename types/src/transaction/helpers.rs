// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    language_storage::TypeTag,
    proto::types::SignedTransaction as ProtoSignedTransaction,
    transaction::{RawTransaction, SignedTransaction, TransactionPayload},
};
use anyhow::Result;
use chrono::Utc;
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHash, TestOnlyHash},
    test_utils::KeyPair,
    traits::SigningKey,
    HashValue,
};

/// Used to get the digest of a set of signed transactions.  This is used by a validator
/// to sign a block and to verify the signatures of other validators on a block
pub fn get_signed_transactions_digest(signed_txns: &[ProtoSignedTransaction]) -> HashValue {
    let mut signatures = vec![];
    for transaction in signed_txns {
        let signed_txn: SignedTransaction = lcs::from_bytes(&transaction.txn_bytes)
            .expect("Unable to deserialize SignedTransaction");
        signatures.extend_from_slice(&signed_txn.authenticator().signature_bytes());
    }
    signatures.test_only_hash()
}

pub fn create_unsigned_txn(
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_specifier: TypeTag,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawTransaction {
    RawTransaction::new(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        gas_specifier,
        std::time::Duration::new((Utc::now().timestamp() + txn_expiration) as u64, 0),
    )
}

pub trait TransactionSigner {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction>;
}

/// Craft a transaction request.
pub fn create_user_txn<T: TransactionSigner + ?Sized>(
    signer: &T,
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_specifier: TypeTag,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_txn(
        payload,
        sender_address,
        sender_sequence_number,
        max_gas_amount,
        gas_unit_price,
        gas_specifier,
        txn_expiration,
    );
    signer.sign_txn(raw_txn)
}

impl TransactionSigner for KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction> {
        let signature = self.private_key.sign_message(&raw_txn.hash());
        Ok(SignedTransaction::new(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}
