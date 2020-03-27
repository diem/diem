// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{sync_info::SyncInfo, vote::Vote};
use anyhow::ensure;
use libra_crypto::HashValue;
use libra_types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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

    pub fn proposed_block_id(&self) -> HashValue {
        self.vote.vote_data().proposed().id()
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
