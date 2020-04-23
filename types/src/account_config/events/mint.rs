// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_config::LIBRA_MODULE_NAME, move_resource::MoveResource};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
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
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for MintEvent {
    const MODULE_NAME: &'static str = LIBRA_MODULE_NAME;
    const STRUCT_NAME: &'static str = "MintEvent";
}
