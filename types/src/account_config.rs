// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    byte_array::ByteArray,
    event::EventHandle,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
};
use anyhow::{bail, Result};
use lazy_static::lazy_static;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryInto};

lazy_static! {
    // LibraCoin
    static ref COIN_MODULE_NAME: Identifier = Identifier::new("LibraCoin").unwrap();
    static ref COIN_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();

    // Account
    static ref ACCOUNT_MODULE_NAME: Identifier = Identifier::new("LibraAccount").unwrap();
    static ref ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();

    // LibraSystem
    static ref LIBRA_SYSTEM_MODULE_NAME: Identifier = Identifier::new("LibraSystem").unwrap();
    static ref BLOCK_METADATA_STRUCT_NAME: Identifier = Identifier::new("BlockMetadata").unwrap();

}

pub fn libra_system_module_name() -> &'static IdentStr {
    &*LIBRA_SYSTEM_MODULE_NAME
}
pub fn block_metadata_module_name() -> &'static IdentStr {
    &*BLOCK_METADATA_STRUCT_NAME
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

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D8")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn channel_global_events_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D9")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn subsidy_address() -> AccountAddress {
    //MINT
    AccountAddress::from_hex_literal("0x6d696e74")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn coin_struct_tag() -> StructTag {
    StructTag {
        module: coin_module_name().to_owned(),
        name: coin_struct_name().to_owned(),
        address: core_code_address(),
        type_params: vec![],
    }
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: account_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn block_metadata_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: libra_system_module_name().to_owned(),
        name: block_metadata_module_name().to_owned(),
        type_params: vec![],
    }
}

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Default, Serialize, Deserialize)]
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

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    pub fn make_from(account_map: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<Self> {
        let ap = account_resource_path();
        match account_map.get(&ap) {
            Some(bytes) => lcs::from_bytes(bytes).map_err(Into::into),
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
/// They have an AccountAddress for the sender or receiver, the amount transferred, and metadata.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccountEvent {
    amount: u64,
    account: AccountAddress,
    metadata: Vec<u8>,
}

impl AccountEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(amount: u64, account: AccountAddress, metadata: Vec<u8>) -> Self {
        Self {
            amount,
            account,
            metadata,
        }
    }

    pub fn try_from(bytes: &[u8]) -> Result<AccountEvent> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the account related to the event
    pub fn account(&self) -> AccountAddress {
        self.account
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
