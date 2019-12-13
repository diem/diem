// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
use libra_types::block_info::BlockInfo;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// VoteData keeps the information about the block, and its parent.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, CryptoHasher)]
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
}

impl CryptoHash for VoteData {
    type Hasher = VoteDataHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(lcs::to_bytes(self).expect("Should serialize.").as_ref());
        state.finish()
    }
}
