// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_info::BlockInfo,
    common::{Author, Round},
    quorum_cert::QuorumCert,
    vote_data::VoteData,
};
use executor::{ExecutedTrees, ProcessedVMOutput, StateComputeResult};
use failure::ensure;
use libra_crypto::{
    hash::{BlockHasher, CryptoHash, CryptoHasher},
    HashValue,
};
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, Signature, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
};
use mirai_annotations::{
    assumed_postcondition, checked_precondition, checked_precondition_eq, debug_checked_verify_eq,
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

#[path = "block_test_utils.rs"]
#[cfg(any(test, feature = "testing"))]
pub mod block_test_utils;

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

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
/// Block has the core data of a consensus block that should be persistent when necessary.
/// Each block must know the id of its parent and keep the QuorurmCertificate to that parent.
pub struct Block<T> {
    /// This block's id as a hash value
    #[serde(skip)]
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
#[derive(Clone, Debug)]
pub struct ExecutedBlock<T> {
    /// Block data that cannot be regenerated.
    block: Block<T>,
    /// The processed output needed by executor.
    output: Arc<ProcessedVMOutput>,
    /// The state compute result is calculated for all the pending blocks prior to insertion to
    /// the tree (the initial root node might not have it, because it's been already
    /// committed). The execution results are not persisted: they're recalculated again for the
    /// pending blocks upon restart.
    compute_result: Arc<StateComputeResult>,
}

impl<T: PartialEq> PartialEq for ExecutedBlock<T> {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && self.compute_result == other.compute_result
    }
}

impl<T: Eq> Eq for ExecutedBlock<T> where T: PartialEq {}

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
            self.quorum_cert.certified_block().id(),
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
        self.quorum_cert.certified_block().id()
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
    T: Serialize + Default + PartialEq,
{
    #[cfg(any(test, feature = "testing"))]
    pub fn make_genesis_block() -> Self {
        Self::make_genesis_block_from_ledger_info(&LedgerInfo::genesis())
    }

    /// Construct new genesis block for next epoch deterministically from the end-epoch LedgerInfo
    /// We carry over most fields except round and block id
    pub fn make_genesis_block_from_ledger_info(ledger_info: &LedgerInfo) -> Self {
        assert!(ledger_info.next_validator_set().is_some());
        let ancestor = BlockInfo::new(
            ledger_info.epoch(),
            0,
            HashValue::zero(),
            ledger_info.transaction_accumulator_hash(),
            ledger_info.version(),
            ledger_info.timestamp_usecs(),
            ledger_info.next_validator_set().cloned(),
        );

        // Genesis carries a placeholder quorum certificate to its parent id with LedgerInfo
        // carrying information about version from the last LedgerInfo of previous epoch.
        let genesis_quorum_cert = QuorumCert::new(
            VoteData::new(ancestor.clone(), ancestor),
            LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    ledger_info.version(),
                    ledger_info.transaction_accumulator_hash(),
                    HashValue::zero(),
                    HashValue::zero(),
                    ledger_info.epoch(),
                    ledger_info.timestamp_usecs(),
                    None,
                ),
                BTreeMap::new(),
            ),
        );

        let block_internal = BlockSerializer::<T> {
            payload: None,
            epoch: ledger_info.epoch() + 1,
            round: 0,
            timestamp_usecs: ledger_info.timestamp_usecs(),
            quorum_cert: &genesis_quorum_cert,
            author: None,
        };

        Block {
            id: block_internal.hash(),
            epoch: ledger_info.epoch() + 1,
            round: 0,
            timestamp_usecs: ledger_info.timestamp_usecs(),
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
        let author = validator_signer.author();
        let block_internal = BlockSerializer {
            payload: Some(&payload),
            epoch,
            round,
            timestamp_usecs,
            quorum_cert: &quorum_cert,
            author: Some(&author),
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
                author,
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
        checked_precondition_eq!(quorum_cert.certified_block().id(), parent_block.id());
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
        checked_precondition_eq!(quorum_cert.certified_block().id(), parent_block.id());
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
        debug_checked_verify_eq!(self.id(), self.hash(), "Block id mismatch the hash");
        ensure!(
            self.quorum_cert().certified_block().round() < self.round(),
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
    pub fn new(
        block: Block<T>,
        output: ProcessedVMOutput,
        compute_result: StateComputeResult,
    ) -> Self {
        Self {
            block,
            output: Arc::new(output),
            compute_result: Arc::new(compute_result),
        }
    }

    pub fn output(&self) -> &Arc<ProcessedVMOutput> {
        &self.output
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
    T: Serialize + Default + PartialEq,
{
    pub fn payload(&self) -> Option<&T> {
        self.block().payload()
    }

    pub fn id(&self) -> HashValue {
        self.block().id()
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert().certified_block().id()
    }

    pub fn round(&self) -> Round {
        self.block().round()
    }

    pub fn epoch(&self) -> u64 {
        self.block().epoch()
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

    pub fn executed_trees(&self) -> &ExecutedTrees {
        self.output.executed_trees()
    }
}

impl<T> CryptoHash for Block<T>
where
    T: Serialize,
{
    type Hasher = BlockHasher;

    fn hash(&self) -> HashValue {
        // The author value used by NIL blocks for calculating the hash is genesis.
        let author = match &self.block_type {
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

// Internal use only. Contains all the fields in Block that contribute to the computation of
// Block Id
#[derive(Serialize)]
struct BlockSerializer<'a, T> {
    timestamp_usecs: u64,
    epoch: u64,
    round: Round,
    payload: Option<&'a T>,
    quorum_cert: &'a QuorumCert,
    author: Option<&'a Author>,
}

impl<'a, T> CryptoHash for BlockSerializer<'a, T>
where
    T: Serialize,
{
    type Hasher = BlockHasher;

    fn hash(&self) -> HashValue {
        let bytes = lcs::to_bytes(self).expect("block serialization failed");
        let mut state = Self::Hasher::default();
        state.write(bytes.as_ref());
        state.finish()
    }
}

#[cfg(test)]
impl<T> Block<T> {
    // Is this block a parent of the parameter block?
    pub fn is_parent_of(&self, block: &Self) -> bool {
        block.parent_id() == self.id
    }
}

impl<T> TryFrom<network::proto::Block> for Block<T>
where
    T: DeserializeOwned + Serialize,
{
    type Error = failure::Error;

    fn try_from(proto: network::proto::Block) -> failure::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl<T> TryFrom<Block<T>> for network::proto::Block
where
    T: Serialize + Default + PartialEq,
{
    type Error = failure::Error;

    fn try_from(block: Block<T>) -> failure::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&block)?,
        })
    }
}

impl<'de, T: DeserializeOwned + Serialize> Deserialize<'de> for Block<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BlockWithoutId<T> {
            epoch: u64,
            round: Round,
            timestamp_usecs: u64,
            quorum_cert: QuorumCert,
            block_type: BlockType<T>,
        };
        let BlockWithoutId {
            epoch,
            round,
            timestamp_usecs,
            quorum_cert,
            block_type,
        } = BlockWithoutId::deserialize(deserializer)?;

        let (payload, author) = match &block_type {
            BlockType::Proposal {
                payload, author, ..
            } => (Some(payload), Some(author)),
            _ => (None, None),
        };
        let id = BlockSerializer::<T> {
            epoch,
            round,
            timestamp_usecs,
            quorum_cert: &quorum_cert,
            payload,
            author,
        }
        .hash();
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
