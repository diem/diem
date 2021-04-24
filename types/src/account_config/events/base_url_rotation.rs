// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a BaseUrlRotationEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseUrlRotationEvent {
    new_base_url: Vec<u8>,
    time_rotated_seconds: u64,
}

impl BaseUrlRotationEvent {
    /// Get the new base url
    pub fn new_base_url(&self) -> &[u8] {
        &self.new_base_url
    }

    /// Get the (blockchain) time in seconds when the url was rotated
    pub fn time_rotated_seconds(&self) -> u64 {
        self.time_rotated_seconds
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for BaseUrlRotationEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("DualAttestation");
    const STRUCT_NAME: &'static IdentStr = ident_str!("BaseUrlRotationEvent");
}
