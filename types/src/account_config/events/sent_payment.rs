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

/// Struct that represents a SentPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct SentPaymentEvent {
    amount: u64,
    currency_code: Identifier,
    receiver: AccountAddress,
    metadata: Vec<u8>,
}

impl SentPaymentEvent {
    // TODO: should only be used for diem client testing and be removed eventually
    pub fn new(
        amount: u64,
        currency_code: Identifier,
        receiver: AccountAddress,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            amount,
            currency_code,
            receiver,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
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
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    /// Return the currency currency_code symbol that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }
}

impl MoveStructType for SentPaymentEvent {
    const MODULE_NAME: &'static IdentStr = ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("SentPaymentEvent");
}
