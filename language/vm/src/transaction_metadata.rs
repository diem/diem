// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::{signing::generate_genesis_keypair, PublicKey};
use types::{account_address::AccountAddress, transaction::SignedTransaction};

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub public_key: PublicKey,
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub transaction_size: u64,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedTransaction) -> Self {
        Self {
            sender: txn.sender(),
            public_key: txn.public_key(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: txn.max_gas_amount(),
            gas_unit_price: txn.gas_unit_price(),
            transaction_size: txn.raw_txn_bytes_len() as u64,
        }
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> u64 {
        self.gas_unit_price
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender.to_owned()
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn transaction_size(&self) -> u64 {
        self.transaction_size
    }
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        let (_, public_key) = generate_genesis_keypair();
        TransactionMetadata {
            sender: AccountAddress::default(),
            public_key,
            sequence_number: 0,
            max_gas_amount: 100_000_000,
            gas_unit_price: 0,
            transaction_size: 0,
        }
    }
}
