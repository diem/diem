// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_config::constants::ACCOUNT_MODULE_NAME, event::EventHandle};
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountResource {
    authentication_key: Vec<u8>,
    delegated_key_rotation_capability: bool,
    delegated_withdrawal_capability: bool,
    received_events: EventHandle,
    sent_events: EventHandle,
    sequence_number: u64,
    is_frozen: bool,
}

impl AccountResource {
    /// Constructs an Account resource.
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        delegated_key_rotation_capability: bool,
        delegated_withdrawal_capability: bool,
        sent_events: EventHandle,
        received_events: EventHandle,
        is_frozen: bool,
    ) -> Self {
        AccountResource {
            sequence_number,
            authentication_key,
            delegated_key_rotation_capability,
            delegated_withdrawal_capability,
            sent_events,
            received_events,
            is_frozen,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
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

    /// Return the the is_frozen flag for the given AccountResource
    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }
}

impl MoveResource for AccountResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "T";
}
