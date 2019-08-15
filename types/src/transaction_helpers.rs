// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    proto::transaction::SignedTransaction as ProtoSignedTransaction,
    transaction::{Program, RawTransaction, RawTransactionBytes, SignedTransaction},
};
use chrono::Utc;
use crypto::{
    hash::{CryptoHash, TestOnlyHash},
    HashValue,
};
use failure::prelude::*;
use nextgen_crypto::{ed25519::*, test_utils::KeyPair, traits::SigningKey};
use proto_conv::IntoProto;
use protobuf::Message;

/// Used to get the digest of a set of signed transactions.  This is used by a validator
/// to sign a block and to verify the signatures of other validators on a block
pub fn get_signed_transactions_digest(signed_txns: &[ProtoSignedTransaction]) -> HashValue {
    let mut signatures = vec![];
    for transaction in signed_txns {
        signatures.extend_from_slice(&transaction.sender_signature);
    }
    signatures.test_only_hash()
}

pub fn create_unsigned_txn(
    program: Program,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> RawTransaction {
    RawTransaction::new(
        sender_address,
        sender_sequence_number,
        program,
        max_gas_amount,
        gas_unit_price,
        std::time::Duration::new((Utc::now().timestamp() + txn_expiration) as u64, 0),
    )
}

pub trait TransactionSigner {
    fn sign_txn(&self, raw_txn: RawTransaction) -> Result<SignedTransaction>;
}

/// Craft a transaction request.
pub fn create_signed_txn<T: TransactionSigner + ?Sized>(
    signer: &T,
    program: Program,
    sender_address: AccountAddress,
    sender_sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    txn_expiration: i64, // for compatibility with UTC's timestamp.
) -> Result<SignedTransaction> {
    let raw_txn = create_unsigned_txn(
        program,
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
        let bytes = raw_txn.clone().into_proto().write_to_bytes()?;
        let hash = RawTransactionBytes(&bytes).hash();
        let signature = self.private_key.sign_message(&hash);
        Ok(SignedTransaction::craft_signed_transaction_for_client(
            raw_txn,
            self.public_key.clone(),
            signature,
        ))
    }
}
