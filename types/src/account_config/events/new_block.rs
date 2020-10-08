// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct NewBlockEvent {
    round: u64,
    participants: Vec<bool>,
    timestamp: u64,
}

impl NewBlockEvent {
    pub fn new(round: u64, participants: Vec<bool>, timestamp: u64) -> Self {
        Self {
            round,
            participants,
            timestamp,
        }
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn proposed_time(&self) -> u64 {
        self.timestamp
    }

    pub fn participants(&self) -> &[bool] {
        &self.participants
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for NewBlockEvent {
    const MODULE_NAME: &'static str = "LibraBlock";
    const STRUCT_NAME: &'static str = "NewBlockEvent";
}
