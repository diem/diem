// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    proto::types::SignedTransaction as ProtoSignedTransaction,
    transaction::{
        ChannelScriptBody, ChannelWriteSetBody, RawTransaction, SignedTransaction,
        TransactionPayload,
    },
};
use chrono::Utc;
use failure::prelude::*;
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHash, CryptoHasher, TestOnlyHash, TestOnlyHasher},
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
        signatures.extend_from_slice(&signed_txn.signature().to_bytes());
    }
    signatures.test_only_hash()
}

pub fn create_unsigned_txn(
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawTransaction {
    RawTransaction::new(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        std::time::Duration::new((Utc::now().timestamp() + txn_expiration) as u64, 0),
    )
}

pub fn create_unsigned_payload_txn(
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawTransaction {
    RawTransaction::new_payload_txn(
        sender_address,
        sender_sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        std::time::Duration::new((Utc::now().timestamp() + txn_expiration) as u64, 0),
    )
}

pub trait TransactionSigner {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction>;
}

pub trait ChannelPayloadSigner {
    fn sign_script_payload(&self, channel_payload: &ChannelScriptBody) -> Result<Ed25519Signature> {
        self.sign_bytes(SimpleSerializer::serialize(channel_payload)?)
    }

    fn sign_write_set_payload(
        &self,
        channel_payload: &ChannelWriteSetBody,
    ) -> Result<Ed25519Signature> {
        self.sign_bytes(SimpleSerializer::serialize(channel_payload)?)
    }

    fn sign_bytes(&self, bytes: Vec<u8>) -> Result<Ed25519Signature>;
}

/// Craft a transaction request.
pub fn create_user_txn<T: TransactionSigner + ?Sized>(
    signer: &T,
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_txn(
        payload,
        sender_address,
        sender_sequence_number,
        max_gas_amount,
        gas_unit_price,
        txn_expiration,
    );
    signer.sign_txn(raw_txn)
}

/// Craft a transaction request.
pub fn create_signed_payload_txn<T: TransactionSigner + ?Sized>(
    signer: &T,
    payload: TransactionPayload,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_payload_txn(
        payload,
        sender_address,
        sender_sequence_number,
        max_gas_amount,
        gas_unit_price,
        txn_expiration,
    );
    signer.sign_txn(raw_txn)
}

impl TransactionSigner for KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    fn sign_txn(&self, raw_txn: RawTransaction) -> failure::prelude::Result<SignedTransaction> {
        let signature = self.private_key.sign_message(&raw_txn.hash());
        Ok(SignedTransaction::new(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}

impl ChannelPayloadSigner for KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    fn sign_bytes(&self, bytes: Vec<u8>) -> Result<Ed25519Signature> {
        //TODO(jole) use a special hasher.
        let mut hasher = TestOnlyHasher::default();
        hasher.write(bytes.as_slice());
        let hash = hasher.finish();
        Ok(self.private_key.sign_message(&hash))
    }
}
