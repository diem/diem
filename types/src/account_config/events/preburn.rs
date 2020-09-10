// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, account_config::LIBRA_MODULE_NAME};
use anyhow::Result;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a PreburnEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct PreburnEvent {
    amount: u64,
    currency_code: Identifier,
    preburn_address: AccountAddress,
}

impl PreburnEvent {
    /// Get the amount preburned
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Return the code for the currency that was preburned
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    /// Return the address whose Preburn now holds the funds
    pub fn preburn_address(&self) -> AccountAddress {
        self.preburn_address
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for PreburnEvent {
    const MODULE_NAME: &'static str = LIBRA_MODULE_NAME;
    const STRUCT_NAME: &'static str = "PreburnEvent";
}
