// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_types::{account_address::AccountAddress, language_storage::TypeTag},
    types::{
        account_config::{xdx_type_tag, xus_tag, XDX_NAME, XUS_NAME},
        chain_id::ChainId,
        transaction::{authenticator::AuthenticationKey, RawTransaction, TransactionPayload},
    },
};
use std::fmt;

pub use diem_transaction_builder::stdlib;

pub struct TransactionBuilder {
    sender: Option<AccountAddress>,
    sequence_number: Option<u64>,
    payload: TransactionPayload,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
}

impl TransactionBuilder {
    pub fn sender(mut self, sender: AccountAddress) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn sequence_number(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    pub fn max_gas_amount(mut self, max_gas_amount: u64) -> Self {
        self.max_gas_amount = max_gas_amount;
        self
    }

    pub fn gas_unit_price(mut self, gas_unit_price: u64) -> Self {
        self.gas_unit_price = gas_unit_price;
        self
    }

    pub fn gas_currency_code<T: Into<String>>(mut self, gas_currency_code: T) -> Self {
        self.gas_currency_code = gas_currency_code.into();
        self
    }

    pub fn chain_id(mut self, chain_id: ChainId) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn expiration_timestamp_secs(mut self, expiration_timestamp_secs: u64) -> Self {
        self.expiration_timestamp_secs = expiration_timestamp_secs;
        self
    }

    pub fn build(self) -> RawTransaction {
        RawTransaction::new(
            self.sender.expect("sender must have been set"),
            self.sequence_number
                .expect("sequence number must have been set"),
            self.payload,
            self.max_gas_amount,
            self.gas_unit_price,
            self.gas_currency_code,
            self.expiration_timestamp_secs,
            self.chain_id,
        )
    }
}

pub struct TransactionFactory {
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: Currency,
    transaction_expiration_time: u64,
    chain_id: ChainId,
}

impl TransactionFactory {
    pub fn new(chain_id: ChainId) -> Self {
        Self {
            max_gas_amount: 1_000_000,
            gas_unit_price: 0,
            gas_currency_code: Currency::XUS,
            transaction_expiration_time: 100,
            chain_id,
        }
    }

    pub fn peer_to_peer(
        &self,
        currency: Currency,
        payee: AccountAddress,
        amount: u64,
    ) -> TransactionBuilder {
        self.peer_to_peer_with_metadata(currency, payee, amount, Vec::new(), Vec::new())
    }

    pub fn peer_to_peer_with_metadata(
        &self,
        currency: Currency,
        payee: AccountAddress,
        amount: u64,
        metadata: Vec<u8>,
        metadata_signature: Vec<u8>,
    ) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_peer_to_peer_with_metadata_script(
                currency.type_tag(),
                payee,
                amount,
                metadata,
                metadata_signature,
            ),
        ))
    }

    pub fn add_currency_to_account(&self, currency: Currency) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_add_currency_to_account_script(currency.type_tag()),
        ))
    }

    pub fn add_recovery_rotation_capability(
        &self,
        recovery_address: AccountAddress,
    ) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_add_recovery_rotation_capability_script(recovery_address),
        ))
    }

    pub fn create_child_vasp_account(
        &self,
        coin_type: Currency,
        child_auth_key: AuthenticationKey,
        add_all_currencies: bool,
        child_initial_balance: u64,
    ) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_create_child_vasp_account_script(
                coin_type.type_tag(),
                child_auth_key.derived_address(),
                child_auth_key.prefix().to_vec(),
                add_all_currencies,
                child_initial_balance,
            ),
        ))
    }

    pub fn create_recovery_address(&self) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_create_recovery_address_script(),
        ))
    }

    pub fn rotate_authentication_key(&self, new_key: AuthenticationKey) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_rotate_authentication_key_script(new_key.to_vec()),
        ))
    }

    pub fn rotate_authentication_key_with_recovery_address(
        &self,
        recovery_address: AccountAddress,
        to_recover: AccountAddress,
        new_key: AuthenticationKey,
    ) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_rotate_authentication_key_with_recovery_address_script(
                recovery_address,
                to_recover,
                new_key.to_vec(),
            ),
        ))
    }

    pub fn rotate_dual_attestation_info(
        &self,
        new_url: Vec<u8>,
        new_key: Vec<u8>,
    ) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_rotate_dual_attestation_info_script(new_url, new_key),
        ))
    }

    pub fn publish_shared_ed25519_public_key(&self, public_key: Vec<u8>) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_publish_shared_ed25519_public_key_script(public_key),
        ))
    }

    pub fn publish_rotate_ed25519_public_key(&self, public_key: Vec<u8>) -> TransactionBuilder {
        self.transaction_builder(TransactionPayload::Script(
            stdlib::encode_rotate_shared_ed25519_public_key_script(public_key),
        ))
    }

    //
    // Internal Helpers
    //

    fn transaction_builder(&self, payload: TransactionPayload) -> TransactionBuilder {
        TransactionBuilder {
            sender: None,
            sequence_number: None,
            payload,
            max_gas_amount: self.max_gas_amount,
            gas_unit_price: self.gas_unit_price,
            gas_currency_code: self.gas_currency_code.into(),
            expiration_timestamp_secs: self.expiration_timestamp(),
            chain_id: self.chain_id,
        }
    }

    fn expiration_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + self.transaction_expiration_time
    }
}

pub struct DualAttestationMessage {
    message: Box<[u8]>,
}

impl DualAttestationMessage {
    pub fn new<M: Into<Vec<u8>>>(metadata: M, reciever: AccountAddress, amount: u64) -> Self {
        let mut message = metadata.into();
        bcs::serialize_into(&mut message, &reciever).unwrap();
        bcs::serialize_into(&mut message, &amount).unwrap();
        message.extend(b"@@$$DIEM_ATTEST$$@@");

        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &[u8] {
        &self.message
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    XDX,
    XUS,
}

impl Currency {
    pub fn as_str(&self) -> &str {
        match self {
            Currency::XDX => XDX_NAME,
            Currency::XUS => XUS_NAME,
        }
    }

    pub fn type_tag(&self) -> TypeTag {
        match self {
            Currency::XDX => xdx_type_tag(),
            Currency::XUS => xus_tag(),
        }
    }
}

impl PartialEq<str> for Currency {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Currency> for str {
    fn eq(&self, other: &Currency) -> bool {
        other.as_str().eq(self)
    }
}

impl PartialEq<String> for Currency {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Currency> for String {
    fn eq(&self, other: &Currency) -> bool {
        other.as_str().eq(self)
    }
}

impl From<Currency> for String {
    fn from(currency: Currency) -> Self {
        currency.as_str().to_owned()
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
