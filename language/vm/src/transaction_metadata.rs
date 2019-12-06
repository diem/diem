// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits};
use libra_crypto::ed25519::{compat, Ed25519PublicKey, Ed25519Signature};
use libra_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, TransactionPayload},
};

pub struct ChannelMetadata {
    pub channel_address: AccountAddress,
    pub channel_sequence_number: u64,
    pub proposer: AccountAddress,
    pub public_keys: Vec<Ed25519PublicKey>,
    pub signatures: Vec<Option<Ed25519Signature>>,
    pub authorized: bool, //is authorized by participants
}

pub struct TransactionMetadata {
    pub sender: AccountAddress,
    pub public_key: Ed25519PublicKey,
    pub sequence_number: u64,
    pub max_gas_amount: GasUnits<GasCarrier>,
    pub gas_unit_price: GasPrice<GasCarrier>,
    pub transaction_size: AbstractMemorySize<GasCarrier>,
    pub channel_metadata: Option<ChannelMetadata>,
}

impl TransactionMetadata {
    pub fn new(txn: &SignedTransaction) -> Self {
        let channel_metadata = match txn.payload() {
            TransactionPayload::Channel(channel_payload) => Some(ChannelMetadata {
                channel_address: channel_payload.channel_address(),
                channel_sequence_number: channel_payload.channel_sequence_number(),
                proposer: channel_payload.proposer(),
                public_keys: channel_payload.public_keys().to_vec(),
                signatures: channel_payload.signatures().to_vec(),
                authorized: channel_payload.is_authorized(),
            }),
            _ => None,
        };

        Self {
            sender: txn.sender(),
            public_key: txn.public_key(),
            sequence_number: txn.sequence_number(),
            max_gas_amount: GasUnits::new(txn.max_gas_amount()),
            gas_unit_price: GasPrice::new(txn.gas_unit_price()),
            transaction_size: AbstractMemorySize::new(txn.raw_txn_bytes_len() as u64),
            channel_metadata,
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

    pub fn public_key(&self) -> &Ed25519PublicKey {
        &self.public_key
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn transaction_size(&self) -> AbstractMemorySize<GasCarrier> {
        self.transaction_size
    }

    pub fn channel_metadata(&self) -> Option<&ChannelMetadata> {
        self.channel_metadata.as_ref()
    }

    pub fn is_channel_txn(&self) -> bool {
        self.channel_metadata.is_some()
    }
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        let (_, public_key) = compat::generate_genesis_keypair();
        TransactionMetadata {
            sender: AccountAddress::default(),
            public_key,
            sequence_number: 0,
            max_gas_amount: GasUnits::new(100_000_000),
            gas_unit_price: GasPrice::new(0),
            transaction_size: AbstractMemorySize::new(0),
            channel_metadata: None,
        }
    }
}
