// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasPrice, GasUnits};
use crypto::ed25519::{compat, Ed25519PublicKey};
use libra_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, TransactionPayload},
};

pub struct ChannelMetadata {
    pub receiver: AccountAddress,
    pub channel_sequence_number: u64,
    pub receiver_public_key: Ed25519PublicKey,
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
                receiver: channel_payload.receiver(),
                channel_sequence_number: channel_payload.channel_sequence_number(),
                receiver_public_key: channel_payload.receiver_public_key.clone(),
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

    pub fn receiver(&self) -> Option<AccountAddress> {
        self.channel_metadata
            .as_ref()
            .map(|metadata| metadata.receiver)
    }

    pub fn channel_metadata(&self) -> Option<&ChannelMetadata> {
        self.channel_metadata.as_ref()
    }

    pub fn is_channel_txn(&self) -> bool {
        match self.channel_metadata {
            Some(_) => true,
            None => false,
        }
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
