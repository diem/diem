// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_config::{constants::ACCOUNT_MODULE_NAME, resources::AccountResource},
    move_resource::MoveResource,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Returns the path to the received event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_RECEIVED_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = AccountResource::resource_path();
    path.extend_from_slice(b"/received_events_count/");
    path
});

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedPaymentEvent {
    amount: u64,
    currency_code: Identifier,
    sender: AccountAddress,
    metadata: Vec<u8>,
}

impl ReceivedPaymentEvent {
    // TODO: should only be used for libra client testing and be removed eventually
    pub fn new(
        amount: u64,
        currency_code: Identifier,
        sender: AccountAddress,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            amount,
            currency_code,
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

    /// Return the currency code that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }
}

impl MoveResource for ReceivedPaymentEvent {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "ReceivedPaymentEvent";
}
