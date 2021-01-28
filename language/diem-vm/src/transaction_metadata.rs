// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey};
use diem_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    transaction::{
        authenticator::AuthenticationKeyPreimage, SignedTransaction, TransactionPayload,
    },
};
use move_core_types::gas_schedule::{
    AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits,
};
use std::convert::TryFrom;

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub authentication_key_preimage: Vec<u8>,
    pub secondary_signers: Vec<AccountAddress>,
    pub secondary_authentication_key_preimages: Vec<Vec<u8>>,
    pub sequence_number: u64,
    pub max_gas_amount: GasUnits<GasCarrier>,
    pub gas_unit_price: GasPrice<GasCarrier>,
    pub transaction_size: AbstractMemorySize<GasCarrier>,
    pub expiration_timestamp_secs: u64,
    pub chain_id: ChainId,
    pub script_hash: Vec<u8>,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedTransaction) -> Self {
        Self {
            sender: txn.sender(),
            authentication_key_preimage: txn
                .authenticator()
                .sender()
                .authentication_key_preimage()
                .into_vec(),
            secondary_signers: txn.authenticator().secondary_signer_addreses(),
            secondary_authentication_key_preimages: txn
                .authenticator()
                .secondary_signers()
                .iter()
                .map(|account_auth| account_auth.authentication_key_preimage().into_vec())
                .collect(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: GasUnits::new(txn.max_gas_amount()),
            gas_unit_price: GasPrice::new(txn.gas_unit_price()),
            transaction_size: AbstractMemorySize::new(txn.raw_txn_bytes_len() as u64),
            expiration_timestamp_secs: txn.expiration_timestamp_secs(),
            chain_id: txn.chain_id(),
            script_hash: match txn.payload() {
                TransactionPayload::Script(s) => HashValue::sha3_256_of(s.code()).to_vec(),
                TransactionPayload::ScriptFunction(_) => vec![],
                TransactionPayload::Module(_) => vec![],
                TransactionPayload::WriteSet(_) => vec![],
            },
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

    pub fn secondary_signers(&self) -> Vec<AccountAddress> {
        self.secondary_signers.to_owned()
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

    pub fn expiration_timestamp_secs(&self) -> u64 {
        self.expiration_timestamp_secs
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn is_multi_agent(&self) -> bool {
        !self.secondary_signers.is_empty()
    }
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        let mut buf = [0u8; Ed25519PrivateKey::LENGTH];
        buf[Ed25519PrivateKey::LENGTH - 1] = 1;
        let public_key = Ed25519PrivateKey::try_from(&buf[..]).unwrap().public_key();
        TransactionMetadata {
            sender: AccountAddress::ZERO,
            authentication_key_preimage: AuthenticationKeyPreimage::ed25519(&public_key).into_vec(),
            secondary_signers: vec![],
            secondary_authentication_key_preimages: vec![],
            sequence_number: 0,
            max_gas_amount: GasUnits::new(100_000_000),
            gas_unit_price: GasPrice::new(0),
            transaction_size: AbstractMemorySize::new(0),
            expiration_timestamp_secs: 0,
            chain_id: ChainId::test(),
            script_hash: vec![],
        }
    }
}
