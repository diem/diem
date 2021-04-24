// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, account_config::DIEM_MODULE_IDENTIFIER};
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a CancelBurnEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelBurnEvent {
    amount: u64,
    currency_code: Identifier,
    preburn_address: AccountAddress,
}

impl CancelBurnEvent {
    /// Get the amount canceled
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Return the code for the currency that was returned
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    /// Return the address whose Preburn resource formerly held the returned funds
    pub fn preburn_address(&self) -> AccountAddress {
        self.preburn_address
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for CancelBurnEvent {
    const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CancelBurnEvent");
}
