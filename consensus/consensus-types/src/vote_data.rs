// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

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

    /// Returns true if the vote is on a proposal block
    pub fn block_type_proposal(&self) -> bool {
        if self.proposed.timestamp_usecs() == self.parent.timestamp_usecs() {
            // NIL or genesis block
            return false;
        }
        // proposals always have a block metadata transaction
        self.proposed.version() >= self.parent.version().saturating_add(1)
    }

    /// Returns true if the vote is on a NIL block
    pub fn block_type_nil(&self) -> bool {
        if self.proposed.timestamp_usecs() != self.parent.timestamp_usecs() {
            // proposal block
            return false;
        }
        // NIL block always have a block metadata transaction
        let (with_blockmetadata_transaction, overflowed) = self.parent.version().overflowing_add(1);
        if overflowed {
            return false;
        }
        self.proposed.version() == with_blockmetadata_transaction
    }

    /// Returns true if the block is a reconfig block
    pub fn block_type_reconfig(&self) -> bool {
        // reconfiguration blocks have no blockmetadata transaction
        self.proposed.version() == self.parent.version()
    }

    /// Well-formedness checks on the VoteData
    pub fn verify(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.parent.epoch() == self.proposed.epoch(),
            "Parent and proposed epochs do not match",
        );
        anyhow::ensure!(
            self.parent.round() < self.proposed.round(),
            "Proposed round is less than parent round",
        );
        anyhow::ensure!(
            self.parent.timestamp_usecs() <= self.proposed.timestamp_usecs(),
            "Proposed happened before parent",
        );
        anyhow::ensure!(
            self.parent.version() <= self.proposed.version(),
            "Proposed version is less than parent version",
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_reconfig_block() {
        unimplemented!();
    }


    #[test]
    fn test_nil_block() {
        unimplemented!();
    }

    #[test]
    fn test_empty_block() {
        unimplemented!();
    }
}
