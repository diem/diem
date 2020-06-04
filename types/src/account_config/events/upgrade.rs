// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

/// Struct that represents a UpgradeEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeEvent {
    write_set: Vec<u8>,
}

impl UpgradeEvent {
    /// Get the applied writeset.
    pub fn write_set(&self) -> &[u8] {
        &self.write_set
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for UpgradeEvent {
    const MODULE_NAME: &'static str = "LibraWriteSetManager";
    const STRUCT_NAME: &'static str = "UpgradeEvent";
}
