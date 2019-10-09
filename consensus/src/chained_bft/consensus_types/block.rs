// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Round},
    consensus_types::{quorum_cert::QuorumCert, vote_data::VoteData},
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalSerialize, CanonicalSerializer, SimpleSerializer,
};
use crypto::{
    hash::{BlockHasher, CryptoHash, CryptoHasher, GENESIS_BLOCK_ID},
    HashValue,
};
use executor::{ExecutedState, StateComputeResult};
use failure::Result;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, Signature, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
};
use mirai_annotations::{assumed_postcondition, checked_precondition, checked_precondition_eq};
use rmp_serde::{from_slice, to_vec_named};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
    sync::Arc,
};

#[cfg(test)]
#[path = "block_test.rs"]
pub mod block_test;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockType<T> {
    Proposal {
        /// T of the block (e.g. one or more transaction(s)
        payload: T,
        /// Author of the block that can be validated by the author's public key and the signature
        author: Author,
        /// Signature that the hash of this block has been authored by the owner of the private key
        signature: Signature,
    },
    /// NIL blocks don't have authors or signatures: they're generated upon timeouts to fill in the
    /// gaps in the rounds.
    NilBlock,
    /// A genesis block is the first committed block in any epoch that is identically constructed on
    /// all validators by any (potentially different) LedgerInfo that justifies the epoch change
    /// from the previous epoch.  The genesis block is used as the the first root block of the
    /// BlockTree for all epochs.
    Genesis,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// Block has the core data of a consensus block that should be persistent when necessary.
/// Each block must know the id of its parent and keep the QuorurmCertificate to that parent.
pub struct Block<T> {
    /// This block's id as a hash value
    id: HashValue,
    /// Epoch number corresponds to the set of validators that are active for this block.
    epoch: u64,
    /// The round of a block is an internal monotonically increasing counter used by Consensus
    /// protocol.
    round: Round,
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
    /// If a block is a real proposal, contains its author and signature.
    block_type: BlockType<T>,
}

/// ExecutedBlocks are managed in a speculative tree, the committed blocks form a chain. Besides
/// block data, each executed block also has other derived meta data which could be regenerated from
/// blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutedBlock<T> {
    /// Block data that cannot be regenerated.
    block: Block<T>,
    /// The state compute results is calculated for all the pending blocks prior to insertion to
    /// the tree (the initial root node might not have it, because it's been already
    /// committed). The execution results are not persisted: they're recalculated again for the
    /// pending blocks upon restart.
    compute_result: Arc<StateComputeResult>,
}

impl<T: PartialEq> Display for Block<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let nil_marker = if self.block_type == BlockType::NilBlock {
            " (NIL)"
        } else {
            ""
        };
        write!(
            f,
            "[id: {}{}, round: {:02}, parent_id: {}]",
            self.id,
            nil_marker,
            self.round,
            self.quorum_cert.certified_block_id(),
        )
    }
}

impl<T: PartialEq> Display for ExecutedBlock<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.block().fmt(f)
    }
}

impl<T> Block<T> {
    pub fn payload(&self) -> Option<&T> {
        if let BlockType::Proposal { payload, .. } = &self.block_type {
            Some(payload)
        } else {
            None
        }
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert.certified_block_id()
    }

    pub fn round(&self) -> Round {
        // Round numbers:
        // - are reset to 0 periodically.
        // - do not exceed std::u64::MAX - 2 per the 3 chain safety rule
        // (ConsensusState::commit_rule_for_certified_block)
        assumed_postcondition!(self.round < std::u64::MAX - 1);
        self.round
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.timestamp_usecs
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        &self.quorum_cert
    }

    pub fn author(&self) -> Option<Author> {
        if let BlockType::Proposal { author, .. } = self.block_type {
            Some(author)
        } else {
            None
        }
    }

    pub fn signature(&self) -> Option<&Signature> {
        if let BlockType::Proposal { signature, .. } = &self.block_type {
            Some(signature)
        } else {
            None
        }
    }
}

impl<T> Block<T>
where
    T: Serialize + Default + CanonicalSerialize + PartialEq,
{
    // Make an empty genesis block
    pub fn make_genesis_block() -> Self {
        let ancestor_id = HashValue::zero();
        let state_id = ExecutedState::state_for_genesis().state_id;
        // Genesis carries a placeholder quorum certificate to its parent id with LedgerInfo
        // carrying information about version `0`.
        let genesis_quorum_cert = QuorumCert::new(
            VoteData::new(ancestor_id, state_id, 0, ancestor_id, 0),
            LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    0,
                    state_id,
                    HashValue::zero(),
                    HashValue::zero(),
                    0,
                    0,
                    None,
                ),
                BTreeMap::new(),
            ),
        );

        Block {
            id: *GENESIS_BLOCK_ID,
            epoch: 0,
            round: 0,
            timestamp_usecs: 0, // The beginning of UNIX TIME
            quorum_cert: genesis_quorum_cert,
            block_type: BlockType::Genesis,
        }
    }

    // Create a block directly.  Most users should prefer make_block() as it ensures correct block
    // chaining.  This functionality should typically only be used for testing.
    pub fn new_internal(
        payload: T,
        epoch: u64,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        let block_internal = BlockSerializer {
            payload: Some(&payload),
            epoch,
            round,
            timestamp_usecs,
            quorum_cert: &quorum_cert,
            author: Some(validator_signer.author()),
        };

        let id = block_internal.hash();
        let signature = validator_signer
            .sign_message(id)
            .expect("Failed to sign message");

        Block {
            id,
            epoch,
            round,
            timestamp_usecs,
            quorum_cert,
            block_type: BlockType::Proposal {
                payload,
                author: validator_signer.author(),
                signature: signature.into(),
            },
        }
    }

    pub fn make_block(
        parent_block: &Block<T>,
        payload: T,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        // A block must carry a QC to its parent.
        checked_precondition_eq!(quorum_cert.certified_block_id(), parent_block.id());
        checked_precondition!(round > parent_block.round());

        Block::new_internal(
            payload,
            parent_block.epoch(),
            round,
            timestamp_usecs,
            quorum_cert,
            validator_signer,
        )
    }

    /// The NIL blocks are special: they're not carrying any real payload and are generated
    /// independently by different validators just to fill in the round with some QC.
    pub fn make_nil_block(parent_block: &Block<T>, round: Round, quorum_cert: QuorumCert) -> Self {
        checked_precondition_eq!(quorum_cert.certified_block_id(), parent_block.id());
        checked_precondition!(round > parent_block.round());

        // We want all the NIL blocks to agree on the timestamps even though they're generated
        // independently by different validators, hence we're using the timestamp of a parent + 1.
        // The reason for artificially adding 1 usec is to support execution state synchronization,
        // which doesn't have any other way of determining the order of ledger infos rather than
        // comparing their timestamps.
        let timestamp_usecs = parent_block.timestamp_usecs + 1;
        let epoch = parent_block.epoch();
        let block_serializer = BlockSerializer::<T> {
            payload: None,
            epoch,
            round,
            timestamp_usecs,
            quorum_cert: &quorum_cert,
            // the author here doesn't really matter for as long as all the NIL Blocks are hashing
            // the same value, hence use the special genesis author for hashing.
            author: None,
        };

        let id = block_serializer.hash();

        Block {
            id,
            epoch,
            round,
            timestamp_usecs,
            quorum_cert,
            block_type: BlockType::NilBlock,
        }
    }

    /// Verifies that the proposal and the QC are correctly signed.
    /// If this is the genesis block, we skip these checks.
    pub fn validate_signatures(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        match &self.block_type {
            BlockType::Genesis => Ok(()),
            BlockType::NilBlock => self.quorum_cert.verify(validator),
            BlockType::Proposal {
                author, signature, ..
            } => {
                signature.verify(validator, *author, self.hash())?;
                self.quorum_cert.verify(validator)
            }
        }
    }

    /// Makes sure that the proposal makes sense, independently of the current state.
    /// If this is the genesis block, we skip these checks.
    pub fn verify_well_formed(&self) -> failure::Result<()> {
        if self.is_genesis_block() {
            return Ok(());
        }
        ensure!(self.id() == self.hash(), "Block id mismatch the hash");
        ensure!(
            self.quorum_cert().certified_block_round() < self.round(),
            "Block has invalid round"
        );
        Ok(())
    }

    pub fn is_genesis_block(&self) -> bool {
        self.block_type == BlockType::Genesis
    }

    pub fn is_nil_block(&self) -> bool {
        self.block_type == BlockType::NilBlock
    }
}

impl<T> ExecutedBlock<T> {
    pub fn new(block: Block<T>, compute_result: StateComputeResult) -> Self {
        Self {
            block,
            compute_result: Arc::new(compute_result),
        }
    }

    pub fn block(&self) -> &Block<T> {
        &self.block
    }

    pub fn compute_result(&self) -> &Arc<StateComputeResult> {
        &self.compute_result
    }
}

impl<T> ExecutedBlock<T>
where
    T: Serialize + Default + CanonicalSerialize + PartialEq,
{
    pub fn payload(&self) -> Option<&T> {
        self.block().payload()
    }

    pub fn id(&self) -> HashValue {
        self.block().id()
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert().certified_block_id()
    }

    pub fn round(&self) -> Round {
        self.block().round()
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.block().timestamp_usecs()
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        self.block().quorum_cert()
    }

    pub fn is_nil_block(&self) -> bool {
        self.block().is_nil_block()
    }
}

impl<T> CryptoHash for Block<T>
where
    T: canonical_serialization::CanonicalSerialize,
{
    type Hasher = BlockHasher;

    fn hash(&self) -> HashValue {
        // The author value used by NIL blocks for calculating the hash is genesis.
        let author = match self.block_type {
            BlockType::Proposal { author, .. } => Some(author),
            BlockType::NilBlock | BlockType::Genesis => None,
        };
        let block_internal = BlockSerializer {
            payload: self.payload(),
            epoch: self.epoch,
            round: self.round,
            timestamp_usecs: self.timestamp_usecs,
            quorum_cert: &self.quorum_cert,
            author,
        };
        block_internal.hash()
    }
}

// Internal use only. Contains all the fields in Block that contributes to the computation of
// Block Id
struct BlockSerializer<'a, T> {
    payload: Option<&'a T>,
    epoch: u64,
    round: Round,
    timestamp_usecs: u64,
    quorum_cert: &'a QuorumCert,
    author: Option<Author>,
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
            .encode_u64(self.epoch)?
            .encode_u64(self.round)?
            .encode_optional(&self.payload)?
            .encode_struct(self.quorum_cert)?
            .encode_optional(&self.author)?;
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
        block.parent_id() == self.id
    }
}

impl<T> TryFrom<network::proto::Block> for Block<T>
where
    T: DeserializeOwned + CanonicalDeserialize,
{
    type Error = failure::Error;

    fn try_from(proto: network::proto::Block) -> failure::Result<Self> {
        let id = HashValue::from_slice(proto.id.as_ref())?;
        let timestamp_usecs = proto.timestamp_usecs;
        let epoch = proto.epoch;
        let round = proto.round;
        let quorum_cert = proto
            .quorum_cert
            .ok_or_else(|| format_err!("Missing quorum_cert"))?
            .try_into()?;
        let block_type = if proto.round == 0 {
            BlockType::Genesis
        } else if proto.author.is_empty() {
            BlockType::NilBlock
        } else {
            BlockType::Proposal {
                payload: from_slice(&proto.payload)?,
                author: Author::try_from(proto.author)?,
                signature: Signature::try_from(&proto.signature)?,
            }
        };
        Ok(Block {
            id,
            epoch,
            round,
            timestamp_usecs,
            quorum_cert,
            block_type,
        })
    }
}

impl<T> From<Block<T>> for network::proto::Block
where
    T: Serialize + Default + CanonicalSerialize + PartialEq,
{
    fn from(block: Block<T>) -> Self {
        let (payload, signature, author) = if let BlockType::Proposal {
            payload,
            author,
            signature,
        } = block.block_type
        {
            (
                to_vec_named(&payload).expect("fail to serialize payload"),
                signature.to_bytes(),
                author.into(),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        Self {
            id: block.id.to_vec(),
            payload,
            epoch: block.epoch,
            round: block.round,
            timestamp_usecs: block.timestamp_usecs,
            quorum_cert: Some(block.quorum_cert.into()),
            signature,
            author,
        }
    }
}
