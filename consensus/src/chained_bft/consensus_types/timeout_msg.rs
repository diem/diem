// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Round},
    consensus_types::{
        sync_info::SyncInfo, timeout_msg::PacemakerTimeoutCertificateVerificationError::*,
    },
    safety::vote_msg::{VoteMsg, VoteMsgVerificationError},
};
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer, SimpleSerializer};
use crypto::{
    hash::{CryptoHash, CryptoHasher, PacemakerTimeoutHasher, TimeoutMsgHasher},
    HashValue,
};
use network;
use nextgen_crypto::*;
use proto_conv::{FromProto, IntoProto};
use protobuf::RepeatedField;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, convert::TryFrom, fmt, iter::FromIterator};
use types::{
    account_address::AccountAddress,
    validator_signer::ValidatorSigner,
    validator_verifier::{ValidatorVerifier, VerifyError},
};

// Internal use only. Contains all the fields in PaceMakerTimeout that contributes to the
// computation of its hash.
struct PacemakerTimeoutSerializer {
    round: Round,
    author: Author,
}

impl CanonicalSerialize for PacemakerTimeoutSerializer {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> failure::Result<()> {
        serializer.encode_u64(self.round)?;
        serializer.encode_struct(&self.author)?;
        Ok(())
    }
}

impl CryptoHash for PacemakerTimeoutSerializer {
    type Hasher = PacemakerTimeoutHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).expect("Should serialize."));
        state.finish()
    }
}

/// TimeoutMsg verification errors.
#[derive(Debug, Fail, PartialEq)]
pub enum TimeoutVerificationError {
    /// The vote message carried by the timeout doesn't pass verification.
    #[fail(display = "VoteMsgError: {}", _0)]
    VoteMsgError(VoteMsgVerificationError),
    /// The signature doesn't pass verification
    #[fail(display = "SigVerifyError: {}", _0)]
    SigVerifyError(VerifyError),
}

/// This message will be broadcast by a pacemaker as part of TimeoutMsg when its local
/// timeout for a round is reached.  Once f+1 PacemakerTimeout structs
/// from unique authors is gathered it forms a TimeoutCertificate.  A TimeoutCertificate is
/// a proof that will cause a replica to advance to the minimum round in the TimeoutCertificate.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct PacemakerTimeout<Sig> {
    round: Round,
    author: Author,
    signature: Sig,
    vote: Option<VoteMsg<Sig>>,
}

impl<Sig: Signature> PacemakerTimeout<Sig> {
    /// Creates new PacemakerTimeout
    pub fn new(
        round: Round,
        validator_signer: &ValidatorSigner<Sig::SigningKeyMaterial>,
        vote: Option<VoteMsg<Sig>>,
    ) -> Self {
        let author = validator_signer.author();
        let digest = PacemakerTimeoutSerializer { round, author }.hash();
        let signature = validator_signer
            .sign_message(digest)
            .expect("Failed to sign PacemakerTimeout");
        PacemakerTimeout {
            round,
            author,
            signature,
            vote,
        }
    }

    fn pacemaker_timeout_digest(author: AccountAddress, round: Round) -> HashValue {
        PacemakerTimeoutSerializer { round, author }.hash()
    }

    /// Calculates digest for this struct
    pub fn digest(&self) -> HashValue {
        Self::pacemaker_timeout_digest(self.author, self.round)
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn vote_msg(&self) -> Option<&VoteMsg<Sig>> {
        self.vote.as_ref()
    }

    /// Verifies that this message has valid signature
    pub fn verify(
        &self,
        validator: &ValidatorVerifier<Sig::VerifyingKeyMaterial>,
    ) -> Result<(), TimeoutVerificationError> {
        validator
            .verify_signature(self.author, self.digest(), &self.signature)
            .map_err(TimeoutVerificationError::SigVerifyError)?;
        if let Some(vote) = self.vote.as_ref() {
            vote.verify(validator)
                .map_err(TimeoutVerificationError::VoteMsgError)?;
        }
        Ok(())
    }

    /// Returns the author of the timeout
    pub fn author(&self) -> Author {
        self.author
    }

    /// Returns the signature of the author for this timeout
    pub fn signature(&self) -> &Sig {
        &self.signature
    }
}

impl<Sig: Signature> IntoProto for PacemakerTimeout<Sig> {
    type ProtoType = network::proto::PacemakerTimeout;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_round(self.round);
        proto.set_author(self.author.into());
        proto.set_signature((&self.signature.to_bytes()[..]).into());
        if let Some(vote) = self.vote {
            proto.set_vote(vote.into_proto());
        }
        proto
    }
}

impl<Sig: Signature> FromProto for PacemakerTimeout<Sig> {
    type ProtoType = network::proto::PacemakerTimeout;

    fn from_proto(mut object: Self::ProtoType) -> failure::Result<Self> {
        let round = object.get_round();
        let author = Author::try_from(object.take_author())?;
        let signature = Sig::try_from(object.get_signature())?;
        let vote = if let Some(vote_msg) = object.vote.into_option() {
            Some(VoteMsg::from_proto(vote_msg)?)
        } else {
            None
        };
        Ok(PacemakerTimeout {
            round,
            author,
            signature,
            vote,
        })
    }
}

// Internal use only. Contains all the fields in TimeoutMsg that contributes to the computation of
// its hash.
struct TimeoutMsgSerializer {
    pacemaker_timeout_digest: HashValue,
}

impl CanonicalSerialize for TimeoutMsgSerializer {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> failure::Result<()> {
        serializer.encode_raw_bytes(self.pacemaker_timeout_digest.as_ref())?;
        Ok(())
    }
}

impl CryptoHash for TimeoutMsgSerializer {
    type Hasher = TimeoutMsgHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).expect("Should serialize."));
        state.finish()
    }
}

/// This message will be broadcast by a pacemaker when its local timeout for a round is reached.
/// Once the broadcasts start, retries will continue for every timeout until the round changes.
/// Retries are required since, say if a proposer for a round r was unresponsive, it might not
/// propose if it misses even only one PacemakerTimeoutMsg.
///
/// The expected proposer will wait until n-f such messages are received before proposing to
/// ensure liveness (a next proposal has the highest quorum certificate across all replicas
/// as justification). If the expected proposer has a quorum certificate on round r-1, it need
/// not wait until n-f such messages are received and can make a proposal justified
/// by this quorum certificate.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TimeoutMsg<Sig> {
    sync_info: SyncInfo<Sig>,
    pacemaker_timeout: PacemakerTimeout<Sig>,
    signature: Sig,
}

impl<Sig: Signature> TimeoutMsg<Sig> {
    /// Creates new TimeoutMsg
    pub fn new(
        sync_info: SyncInfo<Sig>,
        pacemaker_timeout: PacemakerTimeout<Sig>,
        validator_signer: &ValidatorSigner<Sig::SigningKeyMaterial>,
    ) -> TimeoutMsg<Sig> {
        let digest = Self::new_round_digest(pacemaker_timeout.digest());
        let signature = validator_signer
            .sign_message(digest)
            .expect("Failed to sign PacemakerTimeoutMsg");
        TimeoutMsg {
            sync_info,
            pacemaker_timeout,
            signature,
        }
    }

    fn new_round_digest(pacemaker_timeout_digest: HashValue) -> HashValue {
        TimeoutMsgSerializer {
            pacemaker_timeout_digest,
        }
        .hash()
    }

    /// SyncInfo of the given timeout message
    pub fn sync_info(&self) -> &SyncInfo<Sig> {
        &self.sync_info
    }

    /// Returns a reference to the included PacemakerTimeout
    pub fn pacemaker_timeout(&self) -> &PacemakerTimeout<Sig> {
        &self.pacemaker_timeout
    }

    /// Verifies that this message has valid signature
    pub fn verify(
        &self,
        validator: &ValidatorVerifier<Sig::VerifyingKeyMaterial>,
    ) -> Result<(), TimeoutVerificationError> {
        self.pacemaker_timeout.verify(validator)
    }

    /// Returns the author of the TimeoutMsg
    pub fn author(&self) -> Author {
        self.pacemaker_timeout.author()
    }

    /// Returns a reference to the signature of the author
    #[allow(dead_code)]
    pub fn signature(&self) -> &Sig {
        &self.signature
    }
}

impl<Sig: Signature> FromProto for TimeoutMsg<Sig>
where
    Sig::SigningKeyMaterial: Genesis,
{
    type ProtoType = network::proto::TimeoutMsg;

    fn from_proto(mut object: network::proto::TimeoutMsg) -> failure::Result<Self> {
        let sync_info = SyncInfo::from_proto(object.take_sync_info())?;
        let pacemaker_timeout = PacemakerTimeout::from_proto(object.take_pacemaker_timeout())?;
        let signature = Sig::try_from(object.get_signature())?;
        Ok(TimeoutMsg {
            sync_info,
            pacemaker_timeout,
            signature,
        })
    }
}

impl<Sig: Signature> IntoProto for TimeoutMsg<Sig> {
    type ProtoType = network::proto::TimeoutMsg;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_signature((&self.signature.to_bytes()[..]).into());
        proto.set_sync_info(self.sync_info.into_proto());
        proto.set_pacemaker_timeout(self.pacemaker_timeout.into_proto());
        proto
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// Proposal can include this timeout certificate as justification for switching to next round
pub struct PacemakerTimeoutCertificate<Sig> {
    round: Round,
    timeouts: Vec<PacemakerTimeout<Sig>>,
}

/// PacemakerTimeoutCertificate verification errors.
#[derive(Debug, PartialEq, Fail)]
pub enum PacemakerTimeoutCertificateVerificationError {
    /// Number of signed timeouts is less then required quorum size
    #[fail(display = "NoQuorum")]
    NoQuorum,
    /// Round in message does not match calculated rounds based on signed timeouts
    #[fail(display = "RoundMismatch {}", expected)]
    RoundMismatch { expected: Round },
    /// The signature on one of timeouts doesn't pass verification
    #[fail(display = "SigVerifyError for {}: {}", _0, _1)]
    SigVerifyError(Author, VerifyError),
}

impl<Sig: Signature> fmt::Display for PacemakerTimeoutCertificate<Sig> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimeoutCertificate[round: {}, timeouts:[", self.round)?;
        for (idx, timeout) in self.timeouts.iter().enumerate() {
            write!(f, "<{}>", timeout.round())?;
            if idx != self.timeouts.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl<Sig: Signature> PacemakerTimeoutCertificate<Sig> {
    /// Creates new PacemakerTimeoutCertificate
    pub fn new(
        round: Round,
        timeouts: Vec<PacemakerTimeout<Sig>>,
    ) -> PacemakerTimeoutCertificate<Sig> {
        PacemakerTimeoutCertificate { round, timeouts }
    }

    /// Verifies that timeouts in message actually certify the round
    pub fn verify(
        &self,
        validator: &ValidatorVerifier<Sig::VerifyingKeyMaterial>,
    ) -> Result<(), PacemakerTimeoutCertificateVerificationError> {
        let mut min_round: Option<Round> = None;
        let mut unique_authors = HashSet::new();
        for timeout in &self.timeouts {
            if let Err(e) =
                validator.verify_signature(timeout.author(), timeout.digest(), timeout.signature())
            {
                return Err(SigVerifyError(timeout.author(), e));
            }
            unique_authors.insert(timeout.author());
            let timeout_round = timeout.round();
            min_round = Some(min_round.map_or(timeout_round, move |x| x.min(timeout_round)))
        }
        if unique_authors.len() < validator.quorum_size() {
            return Err(NoQuorum);
        }
        if min_round == Some(self.round) {
            Ok(())
        } else {
            Err(RoundMismatch {
                expected: min_round.unwrap_or(0),
            })
        }
    }

    /// Returns the round of the timeout
    pub fn round(&self) -> Round {
        self.round
    }

    /// Returns the timeouts that certify the PacemakerTimeoutCertificate
    #[allow(dead_code)]
    pub fn timeouts(&self) -> &Vec<PacemakerTimeout<Sig>> {
        &self.timeouts
    }
}

impl<Sig: Signature> IntoProto for PacemakerTimeoutCertificate<Sig> {
    type ProtoType = network::proto::PacemakerTimeoutCertificate;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_timeouts(RepeatedField::from_iter(
            self.timeouts.into_iter().map(PacemakerTimeout::into_proto),
        ));
        proto.set_round(self.round);
        proto
    }
}

impl<Sig: Signature> FromProto for PacemakerTimeoutCertificate<Sig> {
    type ProtoType = network::proto::PacemakerTimeoutCertificate;

    fn from_proto(mut object: Self::ProtoType) -> failure::Result<Self> {
        let timeouts = object
            .take_timeouts()
            .into_iter()
            .map(PacemakerTimeout::from_proto)
            .collect::<failure::Result<Vec<_>>>()?;
        Ok(PacemakerTimeoutCertificate::new(
            object.get_round(),
            timeouts,
        ))
    }
}
