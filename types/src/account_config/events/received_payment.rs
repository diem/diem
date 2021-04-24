// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::constants::ACCOUNT_MODULE_IDENTIFIER,
};
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedPaymentEvent {
    amount: u64,
    currency_code: Identifier,
    sender: AccountAddress,
    metadata: Vec<u8>,
}

impl ReceivedPaymentEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
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
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    /// Return the currency code that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }
}

impl MoveStructType for ReceivedPaymentEvent {
    const MODULE_NAME: &'static IdentStr = ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ReceivedPaymentEvent");
}
