// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::ACCOUNT_MODULE_IDENTIFIER;
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a AdminEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminTransactionEvent {
    committed_timestamp_secs: u64,
}

impl AdminTransactionEvent {
    /// Get the applied writeset.
    pub fn committed_timestamp_secs(&self) -> u64 {
        self.committed_timestamp_secs
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for AdminTransactionEvent {
    const MODULE_NAME: &'static IdentStr = ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("AdminTransactionEvent");
}
