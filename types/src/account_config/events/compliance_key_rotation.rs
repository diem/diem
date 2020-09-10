// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

/// Struct that represents a ComplianceKeyRotationEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceKeyRotationEvent {
    new_compliance_public_key: Vec<u8>,
    time_rotated_seconds: u64,
}

impl ComplianceKeyRotationEvent {
    /// Get the new compliance public key
    pub fn new_compliance_public_key(&self) -> &[u8] {
        &self.new_compliance_public_key
    }

    /// Get the (blockchain) time in seconds when the url was rotated
    pub fn time_rotated_seconds(&self) -> u64 {
        self.time_rotated_seconds
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for ComplianceKeyRotationEvent {
    const MODULE_NAME: &'static str = "DualAttestation";
    const STRUCT_NAME: &'static str = "ComplianceKeyRotationEvent";
}
