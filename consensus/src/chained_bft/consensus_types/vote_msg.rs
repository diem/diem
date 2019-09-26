// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{common::Author, consensus_types::vote_data::VoteData};
use crypto::hash::CryptoHash;
use failure::{Result as ProtoResult, ResultExt};
use network::proto::Vote as ProtoVote;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
};
use types::{
    crypto_proxies::{Signature, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
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
        }
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
        Ok(())
    }
}

impl IntoProto for VoteMsg {
    type ProtoType = ProtoVote;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_vote_data(self.vote_data.into_proto());
        proto.set_author(self.author.into());
        proto.set_ledger_info(self.ledger_info.into_proto());
        proto.set_signature(bytes::Bytes::from(self.signature.to_bytes()));
        proto
    }
}

impl FromProto for VoteMsg {
    type ProtoType = ProtoVote;

    fn from_proto(mut object: Self::ProtoType) -> ProtoResult<Self> {
        let vote_data = VoteData::from_proto(object.take_vote_data())?;
        let author = Author::try_from(object.take_author())?;
        let ledger_info = LedgerInfo::from_proto(object.take_ledger_info())?;
        let signature = Signature::try_from(object.get_signature())?;
        Ok(VoteMsg {
            vote_data,
            author,
            ledger_info,
            signature,
        })
    }
}
