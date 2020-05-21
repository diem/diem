// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, event::EventKey};
use anyhow::Result;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

/// Struct that represents a NewBlockEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewBlockEvent {
    round: u64,
    proposer: AccountAddress,
    previous_block_votes: Vec<AccountAddress>,
    time_micro_seconds: u64,
}

impl NewBlockEvent {
    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn proposer(&self) -> AccountAddress {
        self.proposer
    }

    pub fn previous_block_votes(&self) -> &[AccountAddress] {
        &self.previous_block_votes
    }

    pub fn proposed_time(&self) -> u64 {
        self.time_micro_seconds
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn event_key() -> EventKey {
        crate::block_metadata::new_block_event_key()
    }
}

impl MoveResource for NewBlockEvent {
    const MODULE_NAME: &'static str = "LibraBlock";
    const STRUCT_NAME: &'static str = "NewBlockEvent";
}
