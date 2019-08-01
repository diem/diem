// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Payload},
    consensus_types::{block::Block, sync_info::SyncInfo},
};
use failure::prelude::*;
use network::proto::Proposal as ProtoProposal;
use nextgen_crypto::ed25519::*;
use proto_conv::{FromProto, IntoProto};
use rmp_serde::{from_slice, to_vec_named};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;
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
    pub sync_info: SyncInfo,
}

impl<T: Payload, P: ProposerInfo> ProposalInfo<T, P> {
    pub fn verify(&self, validator: &ValidatorVerifier<Ed25519PublicKey>) -> Result<()> {
        if self.proposal.is_nil_block() {
            return Err(format_err!("Proposal {} for a NIL block", self.proposal));
        }
        self.proposal
            .verify(validator)
            .map_err(|e| format_err!("{:?}", e))?;
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
        if let Some(tc) = self.sync_info.highest_timeout_certificate() {
            let previous_round = self.proposal.round() - 1;
            tc.verify(validator).map_err(|e| format_err!("{:?}", e))?;
            ensure!(
                tc.round() == previous_round,
                "Proposal for {} has a timeout certificate with an incorrect round={}",
                self.proposal,
                tc.round(),
            );
            ensure!(
                self.proposal.quorum_cert().certified_block_round() != tc.round(),
                "Proposal for {} has a timeout certificate and a quorum certificate that have the same round",
                self.proposal,
            );
        } else {
            ensure!(
                self.proposal.quorum_cert().certified_block_round() == previous_round,
                "Proposal for {} has a timeout certificate with an incorrect round={}",
                self.proposal,
                self.proposal.quorum_cert().certified_block_round(),
            );
        }
        match self.proposal.author() {
            Some(author) => {
                if author != self.proposer_info.get_author() {
                    return Err(format_err!(
                        "Proposal {} mismatch author of block and proposer info: block={}, proposer={}",
                        self.proposal,
                        author,
                        self.proposer_info.get_author()));
                }
            }
            None => {
                return Err(format_err!(
                    "Proposal {} does not define an author",
                    self.proposal
                ));
            }
        }
        self.sync_info
            .highest_ledger_info()
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

impl<T: Payload, P: ProposerInfo> IntoProto for ProposalInfo<T, P> {
    type ProtoType = ProtoProposal;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_proposed_block(self.proposal.into_proto());
        proto.set_proposer(
            to_vec_named(&self.proposer_info)
                .expect("fail to serialize proposer info")
                .into(),
        );
        proto.set_sync_info(self.sync_info.into_proto());
        proto
    }
}

impl<T: Payload, P: ProposerInfo> FromProto for ProposalInfo<T, P> {
    type ProtoType = ProtoProposal;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let proposal = Block::<T>::from_proto(object.take_proposed_block())?;
        let proposer_info = from_slice(object.get_proposer())?;
        let sync_info = SyncInfo::from_proto(object.take_sync_info())?;
        Ok(ProposalInfo {
            proposal,
            proposer_info,
            sync_info,
        })
    }
}
