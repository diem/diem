// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Payload, Round},
    consensus_types::{block::Block, quorum_cert::QuorumCert},
    liveness::new_round_msg::PacemakerTimeoutCertificate,
};
use failure::Result;
use futures::Future;
use network::proto::Proposal as ProtoProposal;
use proto_conv::{FromProto, IntoProto};
use rmp_serde::{from_slice, to_vec_named};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt, pin::Pin};
use types::validator_verifier::ValidatorVerifier;

/// ProposerInfo is a general trait that can include various proposer characteristics
/// relevant to a specific protocol implementation. The author is the only common thing for now.
pub trait ProposerInfo:
    Send + Sync + Clone + Copy + fmt::Debug + DeserializeOwned + Serialize + 'static
{
    fn get_author(&self) -> Author;
}

/// Trivial ProposerInfo implementation.
impl ProposerInfo for Author {
    fn get_author(&self) -> Author {
        *self
    }
}

/// ProposalInfo contains the required information for the proposer election protocol to make its
/// choice (typically depends on round and proposer info).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalInfo<T, P> {
    pub proposal: Block<T>,
    pub proposer_info: P,
    pub timeout_certificate: Option<PacemakerTimeoutCertificate>,
    // use to notify about last committed block and the receiver could decide to start
    // a synchronization if it's behind
    pub highest_ledger_info: QuorumCert,
}

impl<T: Payload, P: ProposerInfo> ProposalInfo<T, P> {
    pub fn verify(&self, validator: &ValidatorVerifier) -> Result<()> {
        self.proposal
            .verify(validator)
            .map_err(|e| format_err!("{:?}", e))?;
        if let Some(tc) = &self.timeout_certificate {
            tc.verify(validator).map_err(|e| format_err!("{:?}", e))?;
        }
        if self.proposal.author() != self.proposer_info.get_author() {
            return Err(format_err!("Proposal for {} has mismatching author of block and proposer info: block={}, proposer={}", self.proposal,
            self.proposal.author(), self.proposer_info.get_author()));
        }
        self.highest_ledger_info
            .verify(validator)
            .map_err(|e| format_err!("{:?}", e))?;

        Ok(())
    }
}

impl<T, P> fmt::Display for ProposalInfo<T, P>
where
    P: ProposerInfo,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[block {} from {}]",
            self.proposal,
            self.proposer_info.get_author().short_str()
        )
    }
}

/// ProposerElection incorporates the logic of choosing a leader among multiple candidates.
/// We are open to a possibility for having multiple proposers per round, the ultimate choice
/// of a proposal is exposed by the election protocol via the stream of proposals.
pub trait ProposerElection<T, P> {
    /// If a given author is a valid candidate for being a proposer, generate the info,
    /// otherwise return None.
    /// Note that this function is synchronous.
    fn is_valid_proposer(&self, author: P, round: Round) -> Option<P>;

    /// Return all the possible valid proposers for a given round (this information can be
    /// used by e.g., voters for choosing the destinations for sending their votes to).
    fn get_valid_proposers(&self, round: Round) -> Vec<P>;

    /// Notify proposer election about a new proposal. The function doesn't return any information:
    /// proposer election is going to notify the client about the chosen proposal via a dedicated
    /// channel (to be passed in constructor).
    fn process_proposal(
        &self,
        proposal: ProposalInfo<T, P>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<T: Payload, P: ProposerInfo> IntoProto for ProposalInfo<T, P> {
    type ProtoType = ProtoProposal;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        let (block, proposer, hli) = (self.proposal, self.proposer_info, self.highest_ledger_info);
        proto.set_proposed_block(block.into_proto());
        proto.set_proposer(
            to_vec_named(&proposer)
                .expect("fail to serialize proposer info")
                .into(),
        );
        if let Some(tc) = self.timeout_certificate {
            proto.set_timeout_quorum_cert(tc.into_proto());
        }
        proto.set_highest_ledger_info(hli.into_proto());
        proto
    }
}

impl<T: Payload, P: ProposerInfo> FromProto for ProposalInfo<T, P> {
    type ProtoType = ProtoProposal;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let proposal = Block::<T>::from_proto(object.take_proposed_block())?;
        let proposer_info = from_slice(object.get_proposer())?;
        let highest_ledger_info = QuorumCert::from_proto(object.take_highest_ledger_info())?;
        let timeout_certificate = if let Some(tc) = object.timeout_quorum_cert.into_option() {
            Some(PacemakerTimeoutCertificate::from_proto(tc)?)
        } else {
            None
        };
        Ok(ProposalInfo {
            proposal,
            proposer_info,
            timeout_certificate,
            highest_ledger_info,
        })
    }
}
