// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use libra_types::block_info::BlockInfo;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// VoteData keeps the information about the block, and its parent.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, CryptoHasher, LCSCryptoHash)]
pub struct VoteData {
    /// Contains all the block information needed for voting for the proposed round.
    proposed: BlockInfo,
    /// Contains all the block information for the parent for the proposed round.
    parent: BlockInfo,
}

impl Display for VoteData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "VoteData: [block id: {}, epoch: {}, round: {:02}, parent_block_id: {}, parent_block_round: {:02}]",
            self.proposed().id(), self.proposed().epoch(), self.proposed().round(), self.parent().id(), self.parent().round(),
        )
    }
}

impl VoteData {
    pub fn new(proposed: BlockInfo, parent: BlockInfo) -> Self {
        Self { proposed, parent }
    }

    /// Contains all the block information needed for voting for the proposed round.
    pub fn parent(&self) -> &BlockInfo {
        &self.parent
    }

    /// Contains all the block information for the parent for the proposed round.
    pub fn proposed(&self) -> &BlockInfo {
        &self.proposed
    }

    /// Returns true if the vote is on a NIL block
    pub fn is_nil(&self) -> bool {
        self.proposed.timestamp_usecs() == self.parent.timestamp_usecs()
    }

    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.parent.epoch() == self.proposed.epoch(),
            "Parent and proposed epochs do not match",
        );
        ensure!(
            self.parent.round() < self.proposed.round(),
            "Proposed round is less than parent round",
        );
        if self.is_nil() {
            ensure!(
                self.parent.executed_state_id() == self.proposed.executed_state_id(),
                "NIL vote with non-matching executed_state_id",
            );
            ensure!(
                self.parent.version() == self.proposed.version(),
                "NIL vote with non-matching version",
            );
        } else {
            ensure!(
                self.parent.timestamp_usecs() < self.proposed.timestamp_usecs(),
                "Proposed happened before parent",
            );
            ensure!(
                self.parent.version() < self.proposed.version(),
                "Proposed version is less than parent version",
            );
        }

        Ok(())
    }
}
