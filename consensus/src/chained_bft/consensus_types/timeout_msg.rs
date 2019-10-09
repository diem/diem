// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Round},
    consensus_types::{sync_info::SyncInfo, vote_msg::VoteMsg},
};
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer, SimpleSerializer};
use crypto::{
    hash::{CryptoHash, CryptoHasher, PacemakerTimeoutHasher, TimeoutMsgHasher},
    HashValue,
};
use failure::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{Signature, ValidatorSigner, ValidatorVerifier},
};
use mirai_annotations::assumed_postcondition;
use network;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    fmt,
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

/// This message will be broadcast by a pacemaker as part of TimeoutMsg when its local
/// timeout for a round is reached.  Once f+1 PacemakerTimeout structs
/// from unique authors is gathered it forms a TimeoutCertificate.  A TimeoutCertificate is
/// a proof that will cause a replica to advance to the minimum round in the TimeoutCertificate.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct PacemakerTimeout {
    round: Round,
    author: Author,
    signature: Signature,
    vote: Option<VoteMsg>,
}

impl PacemakerTimeout {
    /// Creates new PacemakerTimeout
    pub fn new(round: Round, validator_signer: &ValidatorSigner, vote: Option<VoteMsg>) -> Self {
        let author = validator_signer.author();
        let digest = PacemakerTimeoutSerializer { round, author }.hash();
        let signature = validator_signer
            .sign_message(digest)
            .expect("Failed to sign PacemakerTimeout");
        PacemakerTimeout {
            round,
            author,
            signature: signature.into(),
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

    pub fn vote_msg(&self) -> Option<&VoteMsg> {
        self.vote.as_ref()
    }

    /// Verifies that this message has valid signature
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        self.signature
            .verify(validator, self.author, self.digest())
            .map_err(Error::from)
            .and_then(|_| {
                if let Some(vote) = self.vote.as_ref() {
                    vote.verify(validator)?;
                }
                Ok(())
            })
            .with_context(|e| format!("Fail to verify TimeoutMsg: {:?}", e))?;
        Ok(())
    }

    /// Returns the author of the timeout
    pub fn author(&self) -> Author {
        self.author
    }

    /// Returns the signature of the author for this timeout
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl TryFrom<network::proto::PacemakerTimeout> for PacemakerTimeout {
    type Error = failure::Error;

    fn try_from(proto: network::proto::PacemakerTimeout) -> failure::Result<Self> {
        let round = proto.round;
        let author = Author::try_from(&proto.author[..])?;
        let signature = Signature::try_from(&proto.signature)?;
        let vote = if let Some(vote_msg) = proto.vote {
            Some(VoteMsg::try_from(vote_msg)?)
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

impl From<PacemakerTimeout> for network::proto::PacemakerTimeout {
    fn from(timeout: PacemakerTimeout) -> Self {
        Self {
            round: timeout.round,
            author: timeout.author.to_vec(),
            signature: timeout.signature.to_bytes(),
            vote: timeout.vote.map(Into::into),
        }
    }
}

// Internal use only. Contains all the fields in TimeoutMsg that contributes to the computation of
// its hash.
struct TimeoutMsgSerializer {
    pacemaker_timeout_digest: HashValue,
}

impl CanonicalSerialize for TimeoutMsgSerializer {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> failure::Result<()> {
        serializer.encode_bytes(self.pacemaker_timeout_digest.as_ref())?;
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
pub struct TimeoutMsg {
    sync_info: SyncInfo,
    pacemaker_timeout: PacemakerTimeout,
    signature: Signature,
}

impl TimeoutMsg {
    /// Creates new TimeoutMsg
    pub fn new(
        sync_info: SyncInfo,
        pacemaker_timeout: PacemakerTimeout,
        validator_signer: &ValidatorSigner,
    ) -> TimeoutMsg {
        let digest = Self::new_round_digest(pacemaker_timeout.digest());
        let signature = validator_signer
            .sign_message(digest)
            .expect("Failed to sign PacemakerTimeoutMsg");
        TimeoutMsg {
            sync_info,
            pacemaker_timeout,
            signature: signature.into(),
        }
    }

    fn new_round_digest(pacemaker_timeout_digest: HashValue) -> HashValue {
        TimeoutMsgSerializer {
            pacemaker_timeout_digest,
        }
        .hash()
    }

    /// SyncInfo of the given timeout message
    pub fn sync_info(&self) -> &SyncInfo {
        &self.sync_info
    }

    /// Returns a reference to the included PacemakerTimeout
    pub fn pacemaker_timeout(&self) -> &PacemakerTimeout {
        &self.pacemaker_timeout
    }

    /// Verifies that this message has valid signature
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        self.pacemaker_timeout.verify(validator)
    }

    /// Returns the author of the TimeoutMsg
    pub fn author(&self) -> Author {
        self.pacemaker_timeout.author()
    }

    /// Returns a reference to the signature of the author
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl TryFrom<network::proto::TimeoutMsg> for TimeoutMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::TimeoutMsg) -> failure::Result<Self> {
        let sync_info = proto
            .sync_info
            .ok_or_else(|| format_err!("Missing sync_info"))?
            .try_into()?;
        let pacemaker_timeout = proto
            .pacemaker_timeout
            .ok_or_else(|| format_err!("Missing pacemaker_timeout"))?
            .try_into()?;
        let signature = Signature::try_from(&proto.signature)?;
        Ok(TimeoutMsg {
            sync_info,
            pacemaker_timeout,
            signature,
        })
    }
}

impl From<TimeoutMsg> for network::proto::TimeoutMsg {
    fn from(timeout_msg: TimeoutMsg) -> Self {
        Self {
            sync_info: Some(timeout_msg.sync_info.into()),
            pacemaker_timeout: Some(timeout_msg.pacemaker_timeout.into()),
            signature: timeout_msg.signature.to_bytes(),
        }
    }
}

impl TryFrom<network::proto::ConsensusMsg> for TimeoutMsg {
    type Error = failure::Error;

    fn try_from(proto: network::proto::ConsensusMsg) -> failure::Result<Self> {
        match proto.message {
            Some(network::proto::ConsensusMsg_oneof::TimeoutMsg(timeout_msg)) => {
                timeout_msg.try_into()
            }
            _ => bail!("Missing timeout_msg"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// Proposal can include this timeout certificate as justification for switching to next round
pub struct PacemakerTimeoutCertificate {
    round: Round,
    timeouts: Vec<PacemakerTimeout>,
}

impl fmt::Display for PacemakerTimeoutCertificate {
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

impl PacemakerTimeoutCertificate {
    /// Creates new PacemakerTimeoutCertificate
    pub fn new(round: Round, timeouts: Vec<PacemakerTimeout>) -> PacemakerTimeoutCertificate {
        PacemakerTimeoutCertificate { round, timeouts }
    }

    /// Verifies that timeouts in message actually certify the round
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        let mut min_round: Option<Round> = None;
        let mut unique_authors = HashSet::new();
        for timeout in &self.timeouts {
            timeout
                .signature()
                .verify(validator, timeout.author(), timeout.digest())
                .with_context(|e| format!("Fail to verify TimeoutCert: {:?}", e))?;
            unique_authors.insert(timeout.author());
            let timeout_round = timeout.round();
            min_round = Some(min_round.map_or(timeout_round, move |x| x.min(timeout_round)))
        }
        validator.check_voting_power(unique_authors.iter())?;
        ensure!(
            min_round == Some(self.round),
            "TimeoutCert has inconsistent round {}, expected: {:?}",
            self.round,
            min_round
        );
        Ok(())
    }

    /// Returns the round of the timeout
    pub fn round(&self) -> Round {
        // Round numbers:
        // - are reset to 0 periodically.
        // - do not exceed std::u64::MAX - 2 per the 3 chain safety rule
        // (ConsensusState::commit_rule_for_certified_block)
        assumed_postcondition!(self.round < std::u64::MAX - 1);
        self.round
    }

    /// Returns the timeouts that certify the PacemakerTimeoutCertificate
    pub fn timeouts(&self) -> &Vec<PacemakerTimeout> {
        &self.timeouts
    }
}

impl TryFrom<network::proto::PacemakerTimeoutCertificate> for PacemakerTimeoutCertificate {
    type Error = failure::Error;

    fn try_from(proto: network::proto::PacemakerTimeoutCertificate) -> failure::Result<Self> {
        let timeouts = proto
            .timeouts
            .into_iter()
            .map(PacemakerTimeout::try_from)
            .collect::<failure::Result<Vec<_>>>()?;
        Ok(PacemakerTimeoutCertificate::new(proto.round, timeouts))
    }
}

impl From<PacemakerTimeoutCertificate> for network::proto::PacemakerTimeoutCertificate {
    fn from(timeout: PacemakerTimeoutCertificate) -> Self {
        Self {
            round: timeout.round,
            timeouts: timeout.timeouts.into_iter().map(Into::into).collect(),
        }
    }
}
