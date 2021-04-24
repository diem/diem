// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::DIEM_MODULE_IDENTIFIER;
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a MintEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct MintEvent {
    amount: u64,
    currency_code: Identifier,
}

impl MintEvent {
    /// Get the amount minted
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Return the code for the currency that was minted
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for MintEvent {
    const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MintEvent");
}
