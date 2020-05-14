// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{RawTransaction, SignedTransaction, TransactionPayload},
};
use anyhow::Result;
use chrono::Utc;
use libra_crypto::{ed25519::*, hash::CryptoHash, test_utils::KeyPair, traits::SigningKey};

pub fn create_unsigned_txn(
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawTransaction {
    RawTransaction::new(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        gas_currency_code,
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
    gas_currency_code: String,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_txn(
        payload,
        sender_address,
        sender_sequence_number,
        max_gas_amount,
        gas_unit_price,
        gas_currency_code,
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
