// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::Block,
    common::{Author, Payload, Round},
    sync_info::SyncInfo,
};
use anyhow::{bail, ensure, format_err, Context, Error, Result};
use libra_types::crypto_proxies::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;

/// ProposalMsg contains the required information for the proposer election protocol to make its
/// choice (typically depends on round and proposer info).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProposalMsg<T> {
    #[serde(bound(deserialize = "Block<T>: Deserialize<'de>"))]
    proposal: Block<T>,
    sync_info: SyncInfo,
}

/// A ProposalMsg is only accessible after verifying the signatures of a ProposalUncheckedSignatures
/// via the `validate_signatures` function.
pub struct ProposalUncheckedSignatures<T>(ProposalMsg<T>);

impl<T: Payload> TryFrom<network::proto::Proposal> for ProposalUncheckedSignatures<T> {
    type Error = Error;

    fn try_from(proto: network::proto::Proposal) -> Result<Self> {
        Ok(ProposalUncheckedSignatures(lcs::from_bytes(&proto.bytes)?))
    }
}

impl<T: Payload> TryFrom<network::proto::ConsensusMsg> for ProposalUncheckedSignatures<T> {
    type Error = Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> Result<Self> {
        match proto.message {
            Some(network::proto::ConsensusMsg_oneof::Proposal(proposal)) => proposal.try_into(),
            _ => bail!("Missing proposal"),
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl<T: Payload> From<ProposalUncheckedSignatures<T>> for ProposalMsg<T> {
    fn from(proposal: ProposalUncheckedSignatures<T>) -> Self {
        proposal.0
    }
}

impl<T: Payload> ProposalUncheckedSignatures<T> {
    /// Validates the signatures of the proposal. This includes the leader's signature over the
    /// block and the QC, the timeout certificate signatures.
    pub fn validate_signatures(self, validator: &ValidatorVerifier) -> Result<ProposalMsg<T>> {
        // verify block leader's signature and QC
        self.0
            .proposal
            .validate_signatures(validator)
            .map_err(|e| format_err!("{:?}", e))?;
        // if there is a timeout certificate, verify its signatures
        if let Some(tc) = self.0.sync_info.highest_timeout_certificate() {
            tc.verify(validator).map_err(|e| format_err!("{:?}", e))?;
        }
        // Note that we postpone the verification of SyncInfo until it's being used.
        // return proposal
        Ok(self.0)
    }

    pub fn epoch(&self) -> u64 {
        self.0.proposal.epoch()
    }
}

impl<T: Payload> ProposalMsg<T> {
    /// Creates a new proposal.
    pub fn new(proposal: Block<T>, sync_info: SyncInfo) -> Self {
        Self {
            proposal,
            sync_info,
        }
    }

    /// Verifies that the ProposalMsg is well-formed.
    pub fn verify_well_formed(self) -> Result<Self> {
        ensure!(
            !self.proposal.is_nil_block(),
            "Proposal {} for a NIL block",
            self.proposal
        );
        self.proposal
            .verify_well_formed()
            .context("Fail to verify ProposalMsg's block")?;
        ensure!(
            self.proposal.round() > 0,
            "Proposal for {} has an incorrect round of 0",
            self.proposal,
        );
        ensure!(
            self.proposal.epoch() == self.sync_info.epoch(),
            "ProposalMsg has different epoch number from SyncInfo"
        );
        ensure!(
            self.proposal.parent_id()
                == self.sync_info.highest_quorum_cert().certified_block().id(),
            "Proposal HQC in SyncInfo certifies {}, but block parent id is {}",
            self.sync_info.highest_quorum_cert().certified_block().id(),
            self.proposal.parent_id(),
        );
        let previous_round = self.proposal.round() - 1;
        let highest_certified_round = std::cmp::max(
            self.proposal.quorum_cert().certified_block().round(),
            self.sync_info
                .highest_timeout_certificate()
                .map_or(0, |tc| tc.round()),
        );
        ensure!(
            previous_round == highest_certified_round,
            "Proposal {} does not have a certified round {}",
            self.proposal,
            previous_round
        );
        ensure!(
            self.proposal.author().is_some(),
            "Proposal {} does not define an author",
            self.proposal
        );
        Ok(self)
    }

    pub fn proposal(&self) -> &Block<T> {
        &self.proposal
    }

    pub fn take_proposal(self) -> Block<T> {
        self.proposal
    }

    pub fn sync_info(&self) -> &SyncInfo {
        &self.sync_info
    }

    pub fn round(&self) -> Round {
        self.proposal.round()
    }

    pub fn proposer(&self) -> Author {
        self.proposal
            .author()
            .expect("Proposal should be verified having an author")
    }
}

impl<T: Payload> fmt::Display for ProposalMsg<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let author = match self.proposal.author() {
            Some(author) => author.short_str(),
            None => String::from("NIL"),
        };
        write!(f, "[proposal {} from {}]", self.proposal, author,)
    }
}

impl<T: Payload> TryFrom<ProposalMsg<T>> for network::proto::Proposal {
    type Error = Error;

    fn try_from(proposal: ProposalMsg<T>) -> Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&proposal)?,
        })
    }
}
