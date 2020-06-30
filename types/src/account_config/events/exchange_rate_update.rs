// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::LIBRA_MODULE_NAME;
use anyhow::Result;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a ToLBRExchangeRateUpdateEvent
#[derive(Debug, Serialize, Deserialize)]
pub struct ToLBRExchangeRateUpdateEvent {
    currency_code: Identifier,
    new_to_lbr_exchange_rate: u64,
}

impl ToLBRExchangeRateUpdateEvent {
    /// Exchange rates are represented as 32|32 fixed-point numbers on-chain. So we divide by the scaling
    /// factor (2^32) of the number to arrive at the floating point representation of the number.
    /// The exchange rate returned is the on-chain rate to two decimal places rounded up (e.g. 1.3333
    /// would be rounded to 1.34).
    pub fn new_to_lbr_exchange_rate(&self) -> f32 {
        let unrounded = (self.new_to_lbr_exchange_rate as f32) / 2f32.powf(32f32);
        (unrounded * 100.0).round() / 100.0
    }

    /// Return the code for the currency whose exchange rate was updated
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for ToLBRExchangeRateUpdateEvent {
    const MODULE_NAME: &'static str = LIBRA_MODULE_NAME;
    const STRUCT_NAME: &'static str = "ToLBRExchangeRateUpdateEvent";
}
