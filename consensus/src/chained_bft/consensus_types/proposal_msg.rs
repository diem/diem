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
use std::fmt;
use types::validator_verifier::ValidatorVerifier;

/// ProposalMsg contains the required information for the proposer election protocol to make its
/// choice (typically depends on round and proposer info).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalMsg<T> {
    pub proposal: Block<T>,
    pub sync_info: SyncInfo,
}

impl<T: Payload> ProposalMsg<T> {
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
        if self.proposal.author().is_none() {
            return Err(format_err!(
                "Proposal {} does not define an author",
                self.proposal
            ));
        }
        self.sync_info
            .highest_ledger_info()
            .verify(validator)
            .map_err(|e| format_err!("{:?}", e))?;

        Ok(())
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

impl<T: Payload> IntoProto for ProposalMsg<T> {
    type ProtoType = ProtoProposal;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_proposed_block(self.proposal.into_proto());
        proto.set_sync_info(self.sync_info.into_proto());
        proto
    }
}

impl<T: Payload> FromProto for ProposalMsg<T> {
    type ProtoType = ProtoProposal;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let proposal = Block::<T>::from_proto(object.take_proposed_block())?;
        let sync_info = SyncInfo::from_proto(object.take_sync_info())?;
        Ok(ProposalMsg {
            proposal,
            sync_info,
        })
    }
}
