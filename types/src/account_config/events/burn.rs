// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::LIBRA_MODULE_NAME, move_resource::MoveResource,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use serde::{Deserialize, Serialize};

/// Struct that represents a BurnEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct BurnEvent {
    amount: u64,
    currency_code: Identifier,
    preburn_address: AccountAddress,
}

impl BurnEvent {
    /// Get the amount burned
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Return the code for the currency that was burned
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    /// Return the address whose Preburn resource formerly held the burned funds
    pub fn preburn_address(&self) -> AccountAddress {
        self.preburn_address
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for BurnEvent {
    const MODULE_NAME: &'static str = LIBRA_MODULE_NAME;
    const STRUCT_NAME: &'static str = "BurnEvent";
}
