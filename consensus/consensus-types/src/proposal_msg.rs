// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::Block,
    common::{Author, Payload, Round},
    sync_info::SyncInfo,
};
use failure::prelude::*;
use libra_types::crypto_proxies::ValidatorVerifier;
use std::convert::{TryFrom, TryInto};
use std::fmt;

/// ProposalMsg contains the required information for the proposer election protocol to make its
/// choice (typically depends on round and proposer info).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalMsg<T> {
    proposal: Block<T>,
    sync_info: SyncInfo,
}

/// A ProposalMsg is only accessible after verifying the signatures of a ProposalUncheckedSignatures
/// via the `validate_signatures` function.
pub struct ProposalUncheckedSignatures<T>(ProposalMsg<T>);

impl<T: Payload> TryFrom<network::proto::Proposal> for ProposalUncheckedSignatures<T> {
    type Error = failure::Error;

    fn try_from(proto: network::proto::Proposal) -> failure::Result<Self> {
        let proposal = proto
            .proposed_block
            .ok_or_else(|| format_err!("Missing proposed_block"))?
            .try_into()?;
        let sync_info = proto
            .sync_info
            .ok_or_else(|| format_err!("Missing sync_info"))?
            .try_into()?;
        Ok(ProposalUncheckedSignatures(ProposalMsg::new(
            proposal, sync_info,
        )))
    }
}

impl<T: Payload> TryFrom<network::proto::ConsensusMsg> for ProposalUncheckedSignatures<T> {
    type Error = failure::Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> failure::Result<Self> {
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
    /// block and the QC, the timeout certificate signatures and the highest_ledger_info signatures.
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
        // verify the QC signatures of highest_ledger_info
        self.0
            .sync_info
            .highest_ledger_info()
            .verify(validator)
            .map_err(|e| format_err!("{:?}", e))?;
        // return proposal
        Ok(self.0)
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
            .with_context(|e| format!("Fail to verify ProposalMsg's block: {:}", e))?;
        ensure!(
            self.proposal.round() > 0,
            "Proposal for {} has an incorrect round of 0",
            self.proposal,
        );
        ensure!(
            self.proposal.parent_id() == self.sync_info.highest_quorum_cert().certified_block_id(),
            "Proposal HQC in SyncInfo certifies {}, but block parent id is {}",
            self.sync_info.highest_quorum_cert().certified_block_id(),
            self.proposal.parent_id(),
        );
        let previous_round = self.proposal.round() - 1;
        let highest_certified_round = std::cmp::max(
            self.proposal.quorum_cert().certified_block_round(),
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

impl<T: Payload> From<ProposalMsg<T>> for network::proto::Proposal {
    fn from(proposal: ProposalMsg<T>) -> Self {
        Self {
            proposed_block: Some(proposal.proposal.into()),
            sync_info: Some(proposal.sync_info.into()),
        }
    }
}
