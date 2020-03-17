// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits};
use libra_crypto::ed25519::compat;
use libra_types::{
    account_address::AccountAddress,
    transaction::{authenticator::AuthenticationKeyPreimage, SignedTransaction},
};
use std::time::Duration;

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub authentication_key_preimage: Vec<u8>,
    pub sequence_number: u64,
    pub max_gas_amount: GasUnits<GasCarrier>,
    pub gas_unit_price: GasPrice<GasCarrier>,
    pub transaction_size: AbstractMemorySize<GasCarrier>,
    pub expiration_time: Duration,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedTransaction) -> Self {
        Self {
            sender: txn.sender(),
            authentication_key_preimage: txn
                .authenticator()
                .authentication_key_preimage()
                .into_vec(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: GasUnits::new(txn.max_gas_amount()),
            gas_unit_price: GasPrice::new(txn.gas_unit_price()),
            transaction_size: AbstractMemorySize::new(txn.raw_txn_bytes_len() as u64),
            expiration_time: txn.expiration_time(),
        }
    }

    pub fn max_gas_amount(&self) -> GasUnits<GasCarrier> {
        self.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> GasPrice<GasCarrier> {
        self.gas_unit_price
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender.to_owned()
    }

    pub fn authentication_key_preimage(&self) -> &[u8] {
        &self.authentication_key_preimage
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn transaction_size(&self) -> AbstractMemorySize<GasCarrier> {
        self.transaction_size
    }

    pub fn expiration_time(&self) -> u64 {
        self.expiration_time.as_secs()
    }
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        let (_, public_key) = compat::generate_genesis_keypair();
        TransactionMetadata {
            sender: AccountAddress::default(),
            authentication_key_preimage: AuthenticationKeyPreimage::ed25519(&public_key).into_vec(),
            sequence_number: 0,
            max_gas_amount: GasUnits::new(100_000_000),
            gas_unit_price: GasPrice::new(0),
            transaction_size: AbstractMemorySize::new(0),
            expiration_time: Duration::new(0, 0),
        }
    }
}
