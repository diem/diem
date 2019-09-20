// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    byte_array::ByteArray,
    event::EventHandle,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleDeserializer,
};
use failure::prelude::*;
use lazy_static::lazy_static;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use std::{collections::BTreeMap, convert::TryInto};

lazy_static! {
    // LibraCoin
    static ref COIN_MODULE_NAME: Identifier = Identifier::new("LibraCoin").unwrap();
    static ref COIN_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();

    // Account
    static ref ACCOUNT_MODULE_NAME: Identifier = Identifier::new("LibraAccount").unwrap();
    static ref ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
}

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

pub fn core_code_address() -> AccountAddress {
    AccountAddress::default()
}

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D8")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: account_struct_name().to_owned(),
        type_params: vec![],
    }
}

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Default)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct AccountResource {
    balance: u64,
    sequence_number: u64,
    authentication_key: ByteArray,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    sent_events: EventHandle,
    received_events: EventHandle,
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
    ) -> Self {
        AccountResource {
            balance,
            sequence_number,
            authentication_key,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events,
            received_events,
        }
    }

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    pub fn make_from(account_map: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<Self> {
        let ap = account_resource_path();
        match account_map.get(&ap) {
            Some(bytes) => SimpleDeserializer::deserialize(bytes),
            None => bail!("No data for {:?}", ap),
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

    pub fn get_event_handle_by_query_path(&self, query_path: &[u8]) -> Result<&EventHandle> {
        if *ACCOUNT_RECEIVED_EVENT_PATH == query_path {
            Ok(&self.received_events)
        } else if *ACCOUNT_SENT_EVENT_PATH == query_path {
            Ok(&self.sent_events)
        } else {
            bail!("Unrecognized query path: {:?}", query_path);
        }
    }
}

impl CanonicalSerialize for AccountResource {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        // TODO(drussi): the order in which these fields are serialized depends on some
        // implementation details in the VM.
        serializer
            .encode_struct(&self.authentication_key)?
            .encode_u64(self.balance)?
            .encode_bool(self.delegated_key_rotation_capability)?
            .encode_bool(self.delegated_withdrawal_capability)?
            .encode_struct(&self.received_events)?
            .encode_struct(&self.sent_events)?
            .encode_u64(self.sequence_number)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccountResource {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let authentication_key = deserializer.decode_struct()?;
        let balance = deserializer.decode_u64()?;
        let delegated_key_rotation_capability = deserializer.decode_bool()?;
        let delegated_withdrawal_capability = deserializer.decode_bool()?;
        let received_events = deserializer.decode_struct()?;
        let sent_events = deserializer.decode_struct()?;
        let sequence_number = deserializer.decode_u64()?;

        Ok(AccountResource {
            balance,
            sequence_number,
            authentication_key,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events,
            received_events,
        })
    }
}

pub fn get_account_resource_or_default(
    account_state: &Option<AccountStateBlob>,
) -> Result<AccountResource> {
    match account_state {
        Some(blob) => {
            let account_btree = blob.try_into()?;
            AccountResource::make_from(&account_btree)
        }
        None => Ok(AccountResource::default()),
    }
}

/// Return the path to the Account resource. It can be used to create an AccessPath for an
/// Account resource.
pub fn account_resource_path() -> Vec<u8> {
    AccessPath::resource_access_vec(&account_struct_tag(), &Accesses::empty())
}

lazy_static! {
    /// The path to the sent event counter for an Account resource.
    /// It can be used to query the event DB for the given event.
    pub static ref ACCOUNT_SENT_EVENT_PATH: Vec<u8> = {
        let mut path = account_resource_path();
        path.extend_from_slice(b"/sent_events_count/");
        path
    };

    /// Returns the path to the received event counter for an Account resource.
    /// It can be used to query the event DB for the given event.
    pub static ref ACCOUNT_RECEIVED_EVENT_PATH: Vec<u8> = {
        let mut path = account_resource_path();
        path.extend_from_slice(b"/received_events_count/");
        path
    };
}

/// Generic struct that represents an Account event.
/// Both SentPaymentEvent and ReceivedPaymentEvent are representable with this struct.
/// They have an AccountAddress for the sender or receiver and the amount transferred.
#[derive(Debug, Default)]
pub struct AccountEvent {
    account: AccountAddress,
    amount: u64,
}

impl AccountEvent {
    pub fn try_from(bytes: &[u8]) -> Result<AccountEvent> {
        let mut deserializer = SimpleDeserializer::new(bytes);
        let amount = deserializer.decode_u64()?;
        let account = deserializer.decode_struct()?;
        Ok(Self { account, amount })
    }

    /// Get the account related to the event
    pub fn account(&self) -> AccountAddress {
        self.account
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }
}
