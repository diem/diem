// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_info::BlockInfo;
use failure::prelude::format_err;
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher, VoteDataHasher},
    HashValue,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

/// VoteData keeps the information about the block, and its parent.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
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
            "VoteData: [block id: {}, round: {:02}, parent_block_id: {}, parent_block_round: {:02}]",
            self.proposed().id(), self.proposed().round(), self.parent().id(), self.parent().round(),
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

impl TryFrom<network::proto::VoteData> for VoteData {
    type Error = failure::Error;

    fn try_from(proto: network::proto::VoteData) -> failure::Result<Self> {
        let proposed = proto
            .proposed
            .ok_or_else(|| format_err!("Missing proposed BlockInfo"))?
            .try_into()?;

        let parent = proto
            .parent
            .ok_or_else(|| format_err!("Missing parent BlockInfo"))?
            .try_into()?;

        Ok(VoteData { proposed, parent })
    }
}

impl From<VoteData> for network::proto::VoteData {
    fn from(vote: VoteData) -> Self {
        Self {
            proposed: Some(vote.proposed.into()),
            parent: Some(vote.parent.into()),
        }
    }
}
