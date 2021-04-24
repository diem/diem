// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::constants::{
        diem_root_address, type_tag_for_currency_code, CORE_CODE_ADDRESS, DIEM_MODULE_IDENTIFIER,
    },
    event::EventHandle,
};
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    language_storage::{ResourceKey, StructTag},
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyInfoResource {
    total_value: u128,
    preburn_value: u64,
    to_xdx_exchange_rate: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: Identifier,
    can_mint: bool,
    mint_events: EventHandle,
    burn_events: EventHandle,
    preburn_events: EventHandle,
    cancel_burn_events: EventHandle,
    exchange_rate_update_events: EventHandle,
}

impl MoveStructType for CurrencyInfoResource {
    const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CurrencyInfo");
}

impl MoveResource for CurrencyInfoResource {}

impl CurrencyInfoResource {
    pub fn new(
        total_value: u128,
        preburn_value: u64,
        to_xdx_exchange_rate: u64,
        is_synthetic: bool,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: Identifier,
        can_mint: bool,
        mint_events: EventHandle,
        burn_events: EventHandle,
        preburn_events: EventHandle,
        cancel_burn_events: EventHandle,
        exchange_rate_update_events: EventHandle,
    ) -> Self {
        Self {
            total_value,
            preburn_value,
            to_xdx_exchange_rate,
            is_synthetic,
            scaling_factor,
            fractional_part,
            currency_code,
            can_mint,
            mint_events,
            burn_events,
            preburn_events,
            cancel_burn_events,
            exchange_rate_update_events,
        }
    }

    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    pub fn scaling_factor(&self) -> u64 {
        self.scaling_factor
    }

    pub fn total_value(&self) -> u128 {
        self.total_value
    }

    pub fn preburn_value(&self) -> u64 {
        self.preburn_value
    }

    pub fn fractional_part(&self) -> u64 {
        self.fractional_part
    }

    pub fn exchange_rate(&self) -> f32 {
        // Exchange rates are represented as 32|32 fixed-point numbers on-chain, so we divide by the scaling
        // factor (2^32) of the number to arrive at the floating point representation of the number.
        (self.to_xdx_exchange_rate as f32) / 2f32.powf(32f32)
    }

    pub fn convert_to_xdx(&self, amount: u64) -> u64 {
        (self.exchange_rate() * (amount as f32)) as u64
    }

    pub fn struct_tag_for(currency_code: Identifier) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: CurrencyInfoResource::module_identifier(),
            name: CurrencyInfoResource::struct_identifier(),
            type_params: vec![type_tag_for_currency_code(currency_code)],
        }
    }

    pub fn resource_path_for(currency_code: Identifier) -> AccessPath {
        let resource_key = ResourceKey::new(
            diem_root_address(),
            CurrencyInfoResource::struct_tag_for(currency_code),
        );
        AccessPath::resource_access_path(resource_key)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn mint_events(&self) -> &EventHandle {
        &self.mint_events
    }

    pub fn burn_events(&self) -> &EventHandle {
        &self.burn_events
    }

    pub fn preburn_events(&self) -> &EventHandle {
        &self.preburn_events
    }

    pub fn cancel_burn_events(&self) -> &EventHandle {
        &self.cancel_burn_events
    }

    pub fn exchange_rate_update_events(&self) -> &EventHandle {
        &self.exchange_rate_update_events
    }
}
