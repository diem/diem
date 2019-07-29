// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Round,
        safety::vote_msg::{VoteMsg, VoteMsgVerificationError},
    },
    state_replication::ExecutedState,
};
use crypto::{
    hash::{CryptoHash, ACCUMULATOR_PLACEHOLDER_HASH, GENESIS_BLOCK_ID},
    HashValue,
};
use failure::Result;
use network::proto::QuorumCert as ProtoQuorumCert;
use nextgen_crypto::ed25519::*;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};
use types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct QuorumCert {
    /// The id of a block that is certified by this QuorumCertificate.
    certified_block_id: HashValue,
    /// The execution state of the corresponding block.
    certified_state: ExecutedState,
    /// The round of a certified block.
    certified_block_round: Round,
    /// The signed LedgerInfo of a committed block that carries the data about the certified block.
    signed_ledger_info: LedgerInfoWithSignatures,
    /// The id of the parent block of the certified block
    certified_parent_block_id: HashValue,
    /// The round of the parent block of the certified block
    certified_parent_block_round: Round,
    /// The id of the grandparent block of the certified block
    certified_grandparent_block_id: HashValue,
    /// The round of the grandparent block of the certified block
    certified_grandparent_block_round: Round,
}

impl Display for QuorumCert {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "QuorumCert: [block id: {}, round: {:02}, {}]",
            self.certified_block_id, self.certified_block_round, self.signed_ledger_info
        )
    }
}

#[allow(dead_code)]
impl QuorumCert {
    pub fn new(
        block_id: HashValue,
        state: ExecutedState,
        round: Round,
        signed_ledger_info: LedgerInfoWithSignatures,
        certified_parent_block_id: HashValue,
        certified_parent_block_round: Round,
        certified_grandparent_block_id: HashValue,
        certified_grandparent_block_round: Round,
    ) -> Self {
        QuorumCert {
            certified_block_id: block_id,
            certified_state: state,
            certified_block_round: round,
            signed_ledger_info,
            certified_parent_block_id,
            certified_parent_block_round,
            certified_grandparent_block_id,
            certified_grandparent_block_round,
        }
    }

    pub fn certified_block_id(&self) -> HashValue {
        self.certified_block_id
    }

    pub fn certified_state(&self) -> ExecutedState {
        self.certified_state
    }

    pub fn certified_block_round(&self) -> Round {
        self.certified_block_round
    }

    pub fn ledger_info(&self) -> &LedgerInfoWithSignatures {
        &self.signed_ledger_info
    }

    pub fn certified_parent_block_id(&self) -> HashValue {
        self.certified_parent_block_id
    }

    pub fn certified_parent_block_round(&self) -> Round {
        self.certified_parent_block_round
    }

    pub fn certified_grandparent_block_id(&self) -> HashValue {
        self.certified_grandparent_block_id
    }

    pub fn certified_grandparent_block_round(&self) -> Round {
        self.certified_grandparent_block_round
    }

    pub fn committed_block_id(&self) -> Option<HashValue> {
        let id = self.ledger_info().ledger_info().consensus_block_id();
        if id.is_zero() {
            None
        } else {
            Some(id)
        }
    }

    /// QuorumCert for the genesis block:
    /// - the ID of the block is predetermined by the `GENESIS_BLOCK_ID` constant.
    /// - the accumulator root hash of the LedgerInfo is set to `ACCUMULATOR_PLACEHOLDER_HASH`
    ///   constant.
    /// - the map of signatures is empty because genesis block is implicitly agreed.
    pub fn certificate_for_genesis() -> QuorumCert {
        let genesis_digest = VoteMsg::vote_digest(
            *GENESIS_BLOCK_ID,
            ExecutedState::state_for_genesis(),
            0,
            *GENESIS_BLOCK_ID,
            0,
            *GENESIS_BLOCK_ID,
            0,
        );
        let signer = ValidatorSigner::<Ed25519PrivateKey>::genesis();
        let li = LedgerInfo::new(
            0,
            *ACCUMULATOR_PLACEHOLDER_HASH,
            genesis_digest,
            *GENESIS_BLOCK_ID,
            0,
            0,
        );
        let signature = signer
            .sign_message(li.hash())
            .expect("Fail to sign genesis ledger info");
        let mut signatures = HashMap::new();
        signatures.insert(signer.author(), signature);
        QuorumCert::new(
            *GENESIS_BLOCK_ID,
            ExecutedState::state_for_genesis(),
            0,
            LedgerInfoWithSignatures::new(li, signatures),
            *GENESIS_BLOCK_ID,
            0,
            *GENESIS_BLOCK_ID,
            0,
        )
    }

    pub fn verify(
        &self,
        validator: &ValidatorVerifier<Ed25519PublicKey>,
    ) -> ::std::result::Result<(), VoteMsgVerificationError> {
        let vote_hash = VoteMsg::vote_digest(
            self.certified_block_id,
            self.certified_state,
            self.certified_block_round,
            self.certified_parent_block_id,
            self.certified_parent_block_round,
            self.certified_grandparent_block_id,
            self.certified_grandparent_block_round,
        );
        if self.ledger_info().ledger_info().consensus_data_hash() != vote_hash {
            return Err(VoteMsgVerificationError::ConsensusDataMismatch);
        }
        // Genesis is implicitly agreed upon, it doesn't have real signatures.
        if self.certified_block_round == 0
            && self.certified_block_id == *GENESIS_BLOCK_ID
            && self.certified_state == ExecutedState::state_for_genesis()
        {
            return Ok(());
        }
        self.ledger_info()
            .verify(validator)
            .map_err(VoteMsgVerificationError::SigVerifyError)
    }
}

impl IntoProto for QuorumCert {
    type ProtoType = ProtoQuorumCert;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_block_id(self.certified_block_id.into());
        proto.set_state_id(self.certified_state.state_id.into());
        proto.set_version(self.certified_state.version);
        proto.set_round(self.certified_block_round);
        proto.set_signed_ledger_info(self.signed_ledger_info.into_proto());
        proto.set_parent_block_id(self.certified_parent_block_id.into());
        proto.set_parent_block_round(self.certified_parent_block_round);
        proto.set_grandparent_block_id(self.certified_grandparent_block_id.into());
        proto.set_grandparent_block_round(self.certified_grandparent_block_round);
        proto
    }
}

impl FromProto for QuorumCert {
    type ProtoType = ProtoQuorumCert;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        let certified_block_id = HashValue::from_slice(object.get_block_id())?;
        let state_id = HashValue::from_slice(object.get_state_id())?;
        let version = object.get_version();
        let certified_block_round = object.get_round();
        let signed_ledger_info =
            LedgerInfoWithSignatures::from_proto(object.get_signed_ledger_info().clone())?;
        let certified_parent_block_id = HashValue::from_slice(object.get_parent_block_id())?;
        let certified_parent_block_round = object.get_parent_block_round();
        let certified_grandparent_block_id =
            HashValue::from_slice(object.get_grandparent_block_id())?;
        let certified_grandparent_block_round = object.get_grandparent_block_round();

        Ok(QuorumCert {
            certified_block_id,
            certified_state: ExecutedState { state_id, version },
            certified_block_round,
            signed_ledger_info,
            certified_parent_block_id,
            certified_parent_block_round,
            certified_grandparent_block_id,
            certified_grandparent_block_round,
        })
    }
}
