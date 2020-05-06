// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::constants::{
        association_address, type_tag_for_currency_code, CORE_CODE_ADDRESS,
    },
    event::EventHandle,
    language_storage::{ResourceKey, StructTag},
    move_resource::MoveResource,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyInfoResource {
    total_value: u128,
    preburn_value: u64,
    to_lbr_exchange_rate: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: Identifier,
    can_mint: bool,
    mint_events: EventHandle,
    burn_events: EventHandle,
    preburn_events: EventHandle,
    cancel_burn_events: EventHandle,
}

impl MoveResource for CurrencyInfoResource {
    const MODULE_NAME: &'static str = "Libra";
    const STRUCT_NAME: &'static str = "CurrencyInfo";
}

impl CurrencyInfoResource {
    pub fn currency_code(&self) -> &IdentStr {
        &self.currency_code
    }

    pub fn scaling_factor(&self) -> u64 {
        self.scaling_factor
    }

    pub fn fractional_part(&self) -> u64 {
        self.fractional_part
    }

    pub fn convert_to_lbr(&self, amount: u64) -> u64 {
        let mut mult = (amount as u128) * (self.to_lbr_exchange_rate as u128);
        mult >>= 32;
        mult as u64
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
            association_address(),
            CurrencyInfoResource::struct_tag_for(currency_code),
        );
        AccessPath::resource_access_path(&resource_key)
    }

    pub fn access_path_for(currency_code: Identifier) -> Vec<u8> {
        AccessPath::resource_access_vec(&CurrencyInfoResource::struct_tag_for(currency_code))
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}
