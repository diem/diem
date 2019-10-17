// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{sync_info::SyncInfo, vote::Vote};
use failure::{bail, format_err};
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

/// VoteMsg is the struct that is ultimately sent by the voter in response for
/// receiving a proposal.
/// VoteMsg carries the `LedgerInfo` of a block that is going to be committed in case this vote
/// is gathers QuorumCertificate (see the detailed explanation in the comments of `LedgerInfo`).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct VoteMsg {
    /// The container for the vote (VoteData, LedgerInfo, Signature)
    vote: Vote,
    /// Sync info carries information about highest QC, TC and LedgerInfo
    sync_info: SyncInfo,
}

impl Display for VoteMsg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VoteMsg: [{}]", self.vote,)
    }
}

impl VoteMsg {
    pub fn new(vote: Vote, sync_info: SyncInfo) -> Self {
        Self { vote, sync_info }
    }

    /// Container for actual voting material
    pub fn vote(&self) -> &Vote {
        &self.vote
    }

    /// SyncInfo of the given vote message
    pub fn sync_info(&self) -> &SyncInfo {
        &self.sync_info
    }
}

impl TryFrom<network::proto::VoteMsg> for VoteMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::VoteMsg) -> failure::Result<Self> {
        let vote = proto
            .vote
            .ok_or_else(|| format_err!("Missing vote"))?
            .try_into()?;
        let sync_info = proto
            .sync_info
            .ok_or_else(|| format_err!("Missing sync_info"))?
            .try_into()?;
        Ok(Self { vote, sync_info })
    }
}

impl TryFrom<network::proto::ConsensusMsg> for VoteMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> failure::Result<Self> {
        match proto.message {
            Some(network::proto::ConsensusMsg_oneof::VoteMsg(vote_msg)) => vote_msg.try_into(),
            _ => bail!("Missing vote"),
        }
    }
}

impl From<VoteMsg> for network::proto::VoteMsg {
    fn from(vote_msg: VoteMsg) -> Self {
        Self {
            vote: Some(vote_msg.vote.into()),
            sync_info: Some(vote_msg.sync_info.into()),
        }
    }
}
