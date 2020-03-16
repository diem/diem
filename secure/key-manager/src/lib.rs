// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey,
};
use libra_transaction_scripts;
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, Script, Transaction, TransactionArgument},
};
use std::time::{Duration, SystemTime};

#[cfg(test)]
mod tests;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;
const TXN_EXPIRATION: u64 = 100;

fn expiration_time(until: u64) -> Duration {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::time::Duration::from_secs(now + until)
}

pub fn build_transaction(
    sender: AccountAddress,
    seq_id: u64,
    signing_key: &Ed25519PrivateKey,
    new_key: &Ed25519PublicKey,
) -> Transaction {
    let script = Script::new(
        libra_transaction_scripts::ROTATE_CONSENSUS_PUBKEY_TXN.clone(),
        vec![TransactionArgument::U8Vector(new_key.to_bytes().to_vec())],
    );
    let raw_txn = RawTransaction::new_script(
        sender,
        seq_id,
        script,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        expiration_time(TXN_EXPIRATION),
    );
    let signed_txn = raw_txn.sign(signing_key, signing_key.public_key()).unwrap();
    Transaction::UserTransaction(signed_txn.into_inner())
}
