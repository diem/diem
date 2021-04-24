// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config, account_config::ACCOUNT_MODULE_IDENTIFIER,
    event::EventKey,
};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountEvent {
    created: AccountAddress,
    role_id: u64,
}

impl CreateAccountEvent {
    pub fn created(&self) -> AccountAddress {
        self.created
    }

    pub fn role_id(&self) -> u64 {
        self.role_id
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn event_key() -> EventKey {
        EventKey::new_from_address(&account_config::diem_root_address(), 0)
    }
}

impl MoveStructType for CreateAccountEvent {
    const MODULE_NAME: &'static IdentStr = ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CreateAccountEvent");
}
