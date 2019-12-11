// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{sync_info::SyncInfo, vote::Vote};
use anyhow::ensure;
use libra_types::crypto_proxies::ValidatorVerifier;
use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "fuzzing"))]
use std::convert::TryInto;
use std::{
    convert::TryFrom,
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

    pub fn epoch(&self) -> u64 {
        self.vote.epoch()
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        ensure!(
            self.vote().epoch() == self.sync_info.epoch(),
            "VoteMsg has different epoch"
        );
        // We're not verifying SyncInfo here yet: we are going to verify it only in case we need
        // it. This way we avoid verifying O(n) SyncInfo messages while aggregating the votes
        // (O(n^2) signature verifications).
        self.vote().verify(validator)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl TryFrom<network::proto::ConsensusMsg> for VoteMsg {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> anyhow::Result<Self> {
        match proto.message {
            Some(network::proto::ConsensusMsg_oneof::VoteMsg(vote_msg)) => vote_msg.try_into(),
            _ => anyhow::bail!("Missing vote"),
        }
    }
}

impl TryFrom<network::proto::VoteMsg> for VoteMsg {
    type Error = anyhow::Error;

    fn try_from(proto: network::proto::VoteMsg) -> anyhow::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<VoteMsg> for network::proto::VoteMsg {
    type Error = anyhow::Error;

    fn try_from(vote_msg: VoteMsg) -> anyhow::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&vote_msg)?,
        })
    }
}
