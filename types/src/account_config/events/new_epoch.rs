// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::event::EventKey;
use anyhow::Result;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

/// Struct that represents a NewEpochEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewEpochEvent {
    epoch: u64,
}

impl NewEpochEvent {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn event_key() -> EventKey {
        crate::on_chain_config::new_epoch_event_key()
    }
}

impl MoveResource for NewEpochEvent {
    const MODULE_NAME: &'static str = "LibraConfig";
    const STRUCT_NAME: &'static str = "NewBlockEvent";
}
