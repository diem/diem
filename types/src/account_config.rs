// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::ModuleId;
use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    byte_array::ByteArray,
    event::EventHandle,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use anyhow::Result;
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

// LibraCoin
static COIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("LibraCoin").unwrap());
static COIN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

/// The ModuleId for the LibraCoin module.
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, COIN_MODULE_NAME.clone()));

// Account
static ACCOUNT_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraAccount").unwrap());
static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

/// The ModuleId for the Account module.
pub static ACCOUNT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, ACCOUNT_MODULE_NAME.clone()));

// Payment Events
static SENT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("SentPaymentEvent").unwrap());
static RECEIVED_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ReceivedPaymentEvent").unwrap());

pub fn coin_module_name() -> &'static IdentStr {
    &*COIN_MODULE_NAME
}

pub fn coin_struct_name() -> &'static IdentStr {
    &*COIN_STRUCT_NAME
}

pub fn account_module_name() -> &'static IdentStr {
    &*ACCOUNT_MODULE_NAME
}

pub fn account_struct_name() -> &'static IdentStr {
    &*ACCOUNT_STRUCT_NAME
}

pub fn sent_event_name() -> &'static IdentStr {
    &*SENT_EVENT_NAME
}

pub fn received_event_name() -> &'static IdentStr {
    &*RECEIVED_EVENT_NAME
}

pub const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::DEFAULT;

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D8")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn discovery_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xD15C0")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: account_module_name().to_owned(),
        name: account_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn sent_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: account_module_name().to_owned(),
        name: sent_event_name().to_owned(),
        type_params: vec![],
    }
}

pub fn received_payment_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: account_module_name().to_owned(),
        name: received_event_name().to_owned(),
        type_params: vec![],
    }
}

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountResource {
    authentication_key: ByteArray,
    balance: u64,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    received_events: EventHandle,
    sent_events: EventHandle,
    sequence_number: u64,
    event_generator: u64,
}

impl AccountResource {
    /// Constructs an Account resource.
    pub fn new(
        balance: u64,
        sequence_number: u64,
        authentication_key: ByteArray,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
        sent_events: EventHandle,
        received_events: EventHandle,
        event_generator: u64,
    ) -> Self {
        AccountResource {
            balance,
            sequence_number,
            authentication_key,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events,
            received_events,
            event_generator,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the balance field for the given AccountResource
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &ByteArray {
        &self.authentication_key
    }

    /// Return the sent_events handle for the given AccountResource
    pub fn sent_events(&self) -> &EventHandle {
        &self.sent_events
    }

    /// Return the received_events handle for the given AccountResource
    pub fn received_events(&self) -> &EventHandle {
        &self.received_events
    }

    /// Return the delegated_key_rotation_capability field for the given AccountResource
    pub fn delegated_key_rotation_capability(&self) -> bool {
        self.delegated_key_rotation_capability
    }

    /// Return the delegated_withdrawal_capability field for the given AccountResource
    pub fn delegated_withdrawal_capability(&self) -> bool {
        self.delegated_withdrawal_capability
    }
}

/// Path to the Account resource.
/// It can be used to create an AccessPath for an Account resource.
pub static ACCOUNT_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&account_struct_tag(), &Accesses::empty()));

/// The path to the sent event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_SENT_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = ACCOUNT_RESOURCE_PATH.to_vec();
    path.extend_from_slice(b"/sent_events_count/");
    path
});

/// Returns the path to the received event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_RECEIVED_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = ACCOUNT_RESOURCE_PATH.to_vec();
    path.extend_from_slice(b"/received_events_count/");
    path
});

/// Struct that represents a SentPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct SentPaymentEvent {
    amount: u64,
    receiver: AccountAddress,
    metadata: Vec<u8>,
}

impl SentPaymentEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(amount: u64, receiver: AccountAddress, metadata: Vec<u8>) -> Self {
        Self {
            amount,
            receiver,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the sender of this transaction event.
    pub fn receiver(&self) -> AccountAddress {
        self.receiver
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }
}

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedPaymentEvent {
    amount: u64,
    sender: AccountAddress,
    metadata: Vec<u8>,
}

impl ReceivedPaymentEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(amount: u64, sender: AccountAddress, metadata: Vec<u8>) -> Self {
        Self {
            amount,
            sender,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the receiver of this transaction event.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }
}
