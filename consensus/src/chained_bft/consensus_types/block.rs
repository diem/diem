// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::{Author, Height, Round},
        consensus_types::quorum_cert::QuorumCert,
        safety::vote_msg::VoteMsgVerificationError,
    },
    state_replication::ExecutedState,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalSerialize, CanonicalSerializer, SimpleSerializer,
};
use crypto::{
    hash::{BlockHasher, CryptoHash, CryptoHasher, GENESIS_BLOCK_ID},
    HashValue,
};
use failure::Result;
use mirai_annotations::{checked_precondition, checked_precondition_eq};
use network::proto::Block as ProtoBlock;
use nextgen_crypto::ed25519::*;
use proto_conv::{FromProto, IntoProto};
use rmp_serde::{from_slice, to_vec_named};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::{Display, Formatter},
};
use types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};

#[cfg(test)]
#[path = "block_test.rs"]
pub mod block_test;

#[derive(Debug)]
pub enum BlockVerificationError {
    /// The verification of quorum cert of this block failed.
    QCVerificationError(VoteMsgVerificationError),
    /// The signature verification of this block failed.
    SigVerifyError,
}

/// Blocks are managed in a speculative tree, the committed blocks form a chain.
/// Each block must know the id of its parent and keep the QuorurmCertificate to that parent.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Block<T> {
    /// This block's id as a hash value
    id: HashValue,
    /// Parent block id of this block as a hash value (all zeros to indicate the genesis block)
    parent_id: HashValue,
    /// T of the block (e.g. one or more transaction(s)
    payload: T,
    /// The round of a block is an internal monotonically increasing counter used by Consensus
    /// protocol.
    round: Round,
    /// The height of a block is its position in the chain (block height = parent block height + 1)
    height: Height,
    /// The approximate physical time a block is proposed by a proposer.  This timestamp is used
    /// for
    /// * Time-dependent logic in smart contracts (the current time of execution)
    /// * Clients determining if they are relatively up-to-date with respect to the block chain.
    ///
    /// It makes the following guarantees:
    /// 1. Time Monotonicity: Time is monotonically increasing in the block
    ///    chain. (i.e. If H1 < H2, H1.Time < H2.Time).
    /// 2. If a block of transactions B is agreed on with timestamp T, then at least f+1
    ///    honest replicas think that T is in the past.  An honest replica will only vote
    ///    on a block when its own clock >= timestamp T.
    /// 3. If a block of transactions B is agreed on with timestamp T, then at least f+1 honest
    ///    replicas saw the contents of B no later than T + delta for some delta.
    ///    If T = 3:00 PM and delta is 10 minutes, then an honest replica would not have
    ///    voted for B unless its clock was between 3:00 PM to 3:10 PM at the time the
    ///    proposal was received.  After 3:10 PM, an honest replica would no longer vote
    ///    on B, noting it was too far in the past.
    timestamp_usecs: u64,
    /// Contains the quorum certified ancestor and whether the quorum certified ancestor was
    /// voted on successfully
    quorum_cert: QuorumCert,
    /// Author of the block that can be validated by the author's public key and the signature
    author: Author,
    /// Signature that the hash of this block has been authored by the owner of the private key
    signature: Ed25519Signature,
}

impl<T> Display for Block<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "[id: {}, round: {:02}, parent_id: {}]",
            self.id, self.round, self.parent_id
        )
    }
}

impl<T> Block<T>
where
    T: Serialize + Default + CanonicalSerialize,
{
    // Make an empty genesis block
    pub fn make_genesis_block() -> Self {
        let ancestor_id = HashValue::zero();
        let genesis_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::genesis();
        let state = ExecutedState::state_for_genesis();
        // Genesis carries a placeholder quorum certificate to its parent id with LedgerInfo
        // carrying information about version `0`.
        let genesis_quorum_cert = QuorumCert::new(
            ancestor_id,
            state,
            0,
            LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    0,
                    state.state_id,
                    HashValue::zero(),
                    HashValue::zero(),
                    0,
                    0,
                ),
                HashMap::new(),
            ),
            ancestor_id,
            0,
            ancestor_id,
            0,
        );
        let genesis_id = *GENESIS_BLOCK_ID;
        let signature = genesis_validator_signer
            .sign_message(genesis_id)
            .expect("Failed to sign genesis id.");

        Block {
            id: genesis_id,
            payload: T::default(),
            parent_id: HashValue::zero(),
            round: 0,
            height: 0,
            timestamp_usecs: 0, // The beginning of UNIX TIME
            quorum_cert: genesis_quorum_cert,
            author: genesis_validator_signer.author(),
            signature: signature.into(),
        }
    }

    // Create a block directly.  Most users should prefer make_block() as it ensures correct block
    // chaining.  This functionality should typically only be used for testing.
    pub fn new_internal(
        payload: T,
        parent_id: HashValue,
        round: Round,
        height: Height,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        validator_signer: &ValidatorSigner<Ed25519PrivateKey>,
    ) -> Self {
        let block_internal = BlockSerializer {
            parent_id,
            payload: &payload,
            round,
            height,
            timestamp_usecs,
            quorum_cert: &quorum_cert,
            author: validator_signer.author(),
        };

        let id = block_internal.hash();
        let signature = validator_signer
            .sign_message(id)
            .expect("Failed to sign message");

        Block {
            id,
            payload,
            parent_id,
            round,
            height,
            timestamp_usecs,
            quorum_cert,
            author: validator_signer.author(),
            signature,
        }
    }

    pub fn make_block(
        parent_block: &Block<T>,
        payload: T,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        validator_signer: &ValidatorSigner<Ed25519PrivateKey>,
    ) -> Self {
        // A block must carry a QC to its parent.
        checked_precondition_eq!(quorum_cert.certified_block_id(), parent_block.id());
        checked_precondition!(round > parent_block.round());
        Block::new_internal(
            payload,
            parent_block.id(),
            round,
            // Height is always parent's height + 1 because it's just the position in the chain.
            parent_block.height() + 1,
            timestamp_usecs,
            quorum_cert,
            validator_signer,
        )
    }

    pub fn get_payload(&self) -> &T {
        &self.payload
    }

    pub fn verify(
        &self,
        validator: &ValidatorVerifier<Ed25519PublicKey>,
    ) -> ::std::result::Result<(), BlockVerificationError> {
        if self.is_genesis_block() {
            return Ok(());
        }
        validator
            .verify_signature(self.author(), self.hash(), self.signature())
            .map_err(|_| BlockVerificationError::SigVerifyError)?;
        self.quorum_cert
            .verify(validator)
            .map_err(BlockVerificationError::QCVerificationError)
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn parent_id(&self) -> HashValue {
        self.parent_id
    }

    pub fn height(&self) -> Height {
        self.height
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.timestamp_usecs
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        &self.quorum_cert
    }

    pub fn author(&self) -> Author {
        self.author
    }

    pub fn signature(&self) -> &Ed25519Signature {
        &self.signature
    }

    pub fn is_genesis_block(&self) -> bool {
        self.id() == *GENESIS_BLOCK_ID
    }
}

impl<T> CryptoHash for Block<T>
where
    T: canonical_serialization::CanonicalSerialize,
{
    type Hasher = BlockHasher;

    fn hash(&self) -> HashValue {
        let block_internal = BlockSerializer {
            parent_id: self.parent_id,
            payload: &self.payload,
            round: self.round,
            height: self.height,
            timestamp_usecs: self.timestamp_usecs,
            quorum_cert: &self.quorum_cert,
            author: self.author,
        };
        block_internal.hash()
    }
}

// Internal use only. Contains all the fields in Block that contributes to the computation of
// Block Id
struct BlockSerializer<'a, T> {
    parent_id: HashValue,
    payload: &'a T,
    round: Round,
    height: Height,
    timestamp_usecs: u64,
    quorum_cert: &'a QuorumCert,
    author: Author,
}

impl<'a, T> CryptoHash for BlockSerializer<'a, T>
where
    T: CanonicalSerialize,
{
    type Hasher = BlockHasher;

    fn hash(&self) -> HashValue {
        let bytes =
            SimpleSerializer::<Vec<u8>>::serialize(self).expect("block serialization failed");
        let mut state = Self::Hasher::default();
        state.write(bytes.as_ref());
        state.finish()
    }
}

impl<'a, T> CanonicalSerialize for BlockSerializer<'a, T>
where
    T: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.timestamp_usecs)?
            .encode_u64(self.round)?
            .encode_u64(self.height)?
            .encode_struct(self.payload)?
            .encode_raw_bytes(self.parent_id.as_ref())?
            .encode_raw_bytes(self.quorum_cert.certified_block_id().as_ref())?
            .encode_struct(&self.author)?;
        Ok(())
    }
}

#[cfg(test)]
impl<T> Block<T>
where
    T: Default + Serialize + CanonicalSerialize,
{
    // Is this block a parent of the parameter block?
    pub fn is_parent_of(&self, block: &Self) -> bool {
        block.parent_id == self.id
    }
}

impl<T> IntoProto for Block<T>
where
    T: Serialize + Default + CanonicalSerialize,
{
    type ProtoType = ProtoBlock;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_timestamp_usecs(self.timestamp_usecs);
        proto.set_id(self.id().into());
        proto.set_parent_id(self.parent_id().into());
        proto.set_payload(
            to_vec_named(self.get_payload())
                .expect("fail to serialize payload")
                .into(),
        );
        proto.set_round(self.round());
        proto.set_height(self.height());
        proto.set_quorum_cert(self.quorum_cert().clone().into_proto());
        proto.set_signature(self.signature().to_bytes().as_ref().into());
        proto.set_author(self.author.into());
        proto
    }
}

impl<T> FromProto for Block<T>
where
    T: DeserializeOwned + CanonicalDeserialize,
{
    type ProtoType = ProtoBlock;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        let id = HashValue::from_slice(object.get_id())?;
        let parent_id = HashValue::from_slice(object.get_parent_id())?;
        let payload = from_slice(object.get_payload())?;
        let timestamp_usecs = object.get_timestamp_usecs();
        let round = object.get_round();
        let height = object.get_height();
        let quorum_cert = QuorumCert::from_proto(object.take_quorum_cert())?;
        let author = Author::try_from(object.take_author())?;
        let signature = Ed25519Signature::try_from(object.get_signature())?;
        Ok(Block {
            id,
            parent_id,
            payload,
            round,
            timestamp_usecs,
            height,
            quorum_cert,
            author,
            signature,
        })
    }
}
