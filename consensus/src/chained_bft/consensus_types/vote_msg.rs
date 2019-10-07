// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::common;
use crate::chained_bft::{common::Author, consensus_types::vote_data::VoteData};
use crypto::hash::CryptoHash;
use failure::ResultExt;
use libra_types::{
    crypto_proxies::{Signature, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
};
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
    /// The data of the vote
    vote_data: VoteData,
    /// The identity of the voter.
    author: Author,
    /// LedgerInfo of a block that is going to be committed in case this vote gathers QC.
    ledger_info: LedgerInfo,
    /// Signature of the LedgerInfo
    signature: Signature,
    /// The round signatures can be aggregated into a timeout certificate if present.
    round_signature: Option<Signature>,
}

impl Display for VoteMsg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Vote: [vote data: {}, author: {}, {}]",
            self.vote_data,
            self.author.short_str(),
            self.ledger_info
        )
    }
}

impl VoteMsg {
    /// Generates a new VoteMsg corresponding to the "fast-vote" path without the round signatures
    /// that can be aggregated into a timeout certificate
    pub fn new(
        vote_data: VoteData,
        author: Author,
        mut ledger_info_placeholder: LedgerInfo,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        ledger_info_placeholder.set_consensus_data_hash(vote_data.hash());
        let li_sig = validator_signer
            .sign_message(ledger_info_placeholder.hash())
            .expect("Failed to sign LedgerInfo");
        Self {
            vote_data,
            author,
            ledger_info: ledger_info_placeholder,
            signature: li_sig.into(),
            round_signature: None,
        }
    }

    /// Generates a round signature, which can then be used for aggregating a timeout certificate.
    /// Typically called for generating vote messages that are sent upon timeouts.
    pub fn add_round_signature(&mut self, validator_signer: &ValidatorSigner) {
        if self.round_signature.is_some() {
            return; // round signature is already set
        }
        self.round_signature.replace(
            validator_signer
                .sign_message(common::round_hash(self.vote_data().block_round()))
                .expect("Failed to sign round")
                .into(),
        );
    }

    pub fn vote_data(&self) -> &VoteData {
        &self.vote_data
    }

    /// Return the author of the vote
    pub fn author(&self) -> Author {
        self.author
    }

    /// Return the LedgerInfo associated with this vote
    pub fn ledger_info(&self) -> &LedgerInfo {
        &self.ledger_info
    }

    /// Return the signature of the vote
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Returns the signature for the vote_data.block_round() that can be aggregated for
    /// TimeoutCertificate.
    pub fn round_signature(&self) -> Option<&Signature> {
        self.round_signature.as_ref()
    }

    /// Verifies that the consensus data hash of LedgerInfo corresponds to the vote info,
    /// and then verifies the signature.
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        ensure!(
            self.ledger_info.consensus_data_hash() == self.vote_data.hash(),
            "Vote's hash mismatch with LedgerInfo"
        );
        self.signature()
            .verify(validator, self.author(), self.ledger_info.hash())
            .with_context(|e| format!("Fail to verify VoteMsg: {:?}", e))?;
        if let Some(round_signature) = &self.round_signature {
            round_signature
                .verify(
                    validator,
                    self.author(),
                    common::round_hash(self.vote_data().block_round()),
                )
                .with_context(|e| format!("Fail to verify VoteMsg: {:?}", e))?;
        }
        Ok(())
    }
}

impl TryFrom<network::proto::Vote> for VoteMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::Vote) -> failure::Result<Self> {
        let vote_data = proto
            .vote_data
            .ok_or_else(|| format_err!("Missing vote_data"))?
            .try_into()?;
        let author = Author::try_from(proto.author)?;
        let ledger_info = proto
            .ledger_info
            .ok_or_else(|| format_err!("Missing ledger_info"))?
            .try_into()?;
        let signature = Signature::try_from(&proto.signature)?;
        let round_signature = if proto.round_signature.is_empty() {
            None
        } else {
            Some(Signature::try_from(&proto.round_signature)?)
        };
        Ok(Self {
            vote_data,
            author,
            ledger_info,
            signature,
            round_signature,
        })
    }
}

impl TryFrom<network::proto::ConsensusMsg> for VoteMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> failure::Result<Self> {
        match proto.message {
            Some(network::proto::ConsensusMsg_oneof::Vote(vote)) => vote.try_into(),
            _ => bail!("Missing vote"),
        }
    }
}

impl From<VoteMsg> for network::proto::Vote {
    fn from(vote: VoteMsg) -> Self {
        Self {
            vote_data: Some(vote.vote_data.into()),
            author: vote.author.into(),
            ledger_info: Some(vote.ledger_info.into()),
            signature: vote.signature.to_bytes(),
            round_signature: vote
                .round_signature
                .map(|sig| sig.to_bytes())
                .unwrap_or_else(Vec::new),
        }
    }
}
