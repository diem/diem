// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{Author, Payload, Round},
    quorum_cert::QuorumCert,
    vote_data::VoteData,
};
use diem_crypto::hash::HashValue;
use diem_crypto_derive::{CryptoHasher, LCSCryptoHash};
use diem_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use mirai_annotations::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockType {
    Proposal {
        /// T of the block (e.g. one or more transaction(s)
        payload: Payload,
        /// Author of the block that can be validated by the author's public key and the signature
        author: Author,
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, CryptoHasher, LCSCryptoHash)]
/// Block has the core data of a consensus block that should be persistent when necessary.
/// Each block must know the id of its parent and keep the QuorurmCertificate to that parent.
pub struct BlockData {
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
    ///   1. Time Monotonicity: Time is monotonically increasing in the block chain.
    ///      (i.e. If H1 < H2, H1.Time < H2.Time).
    ///   2. If a block of transactions B is agreed on with timestamp T, then at least
    ///      f+1 honest validators think that T is in the past. An honest validator will
    ///      only vote on a block when its own clock >= timestamp T.
    ///   3. If a block of transactions B has a QC with timestamp T, an honest validator
    ///      will not serve such a block to other validators until its own clock >= timestamp T.
    ///   4. Current: an honest validator is not issuing blocks with a timestamp in the
    ///       future. Currently we consider a block is malicious if it was issued more
    ///       that 5 minutes in the future.
    timestamp_usecs: u64,
    /// Contains the quorum certified ancestor and whether the quorum certified ancestor was
    /// voted on successfully
    quorum_cert: QuorumCert,
    /// If a block is a real proposal, contains its author and signature.
    block_type: BlockType,
}

impl BlockData {
    pub fn author(&self) -> Option<Author> {
        if let BlockType::Proposal { author, .. } = self.block_type {
            Some(author)
        } else {
            None
        }
    }

    pub fn block_type(&self) -> &BlockType {
        &self.block_type
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert.certified_block().id()
    }

    pub fn payload(&self) -> Option<&Payload> {
        if let BlockType::Proposal { payload, .. } = &self.block_type {
            Some(payload)
        } else {
            None
        }
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

    pub fn is_genesis_block(&self) -> bool {
        matches!(self.block_type, BlockType::Genesis)
    }

    pub fn is_nil_block(&self) -> bool {
        matches!(self.block_type, BlockType::NilBlock)
    }

    pub fn new_genesis_from_ledger_info(ledger_info: &LedgerInfo) -> Self {
        assert!(ledger_info.ends_epoch());
        let ancestor = BlockInfo::new(
            ledger_info.epoch(),
            0,                 /* round */
            HashValue::zero(), /* parent block id */
            ledger_info.transaction_accumulator_hash(),
            ledger_info.version(),
            ledger_info.timestamp_usecs(),
            None,
        );

        // Genesis carries a placeholder quorum certificate to its parent id with LedgerInfo
        // carrying information about version from the last LedgerInfo of previous epoch.
        let genesis_quorum_cert = QuorumCert::new(
            VoteData::new(ancestor.clone(), ancestor.clone()),
            LedgerInfoWithSignatures::new(
                LedgerInfo::new(ancestor, HashValue::zero()),
                BTreeMap::new(),
            ),
        );

        BlockData::new_genesis(ledger_info.timestamp_usecs(), genesis_quorum_cert)
    }

    #[cfg(any(test, feature = "fuzzing"))]
    // This method should only used by tests and fuzzers to produce arbitrary BlockData types.
    pub fn new_for_testing(
        epoch: u64,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        block_type: BlockType,
    ) -> Self {
        Self {
            epoch,
            round,
            timestamp_usecs,
            quorum_cert,
            block_type,
        }
    }

    pub fn new_genesis(timestamp_usecs: u64, quorum_cert: QuorumCert) -> Self {
        assume!(quorum_cert.certified_block().epoch() < u64::max_value()); // unlikely to be false in this universe
        Self {
            epoch: quorum_cert.certified_block().epoch() + 1,
            round: 0,
            timestamp_usecs,
            quorum_cert,
            block_type: BlockType::Genesis,
        }
    }

    pub fn new_nil(round: Round, quorum_cert: QuorumCert) -> Self {
        // We want all the NIL blocks to agree on the timestamps even though they're generated
        // independently by different validators, hence we're using the timestamp of a parent + 1.
        assume!(quorum_cert.certified_block().timestamp_usecs() < u64::max_value()); // unlikely to be false in this universe
        let timestamp_usecs = quorum_cert.certified_block().timestamp_usecs();

        Self {
            epoch: quorum_cert.certified_block().epoch(),
            round,
            timestamp_usecs,
            quorum_cert,
            block_type: BlockType::NilBlock,
        }
    }

    pub fn new_proposal(
        payload: Payload,
        author: Author,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
    ) -> Self {
        Self {
            epoch: quorum_cert.certified_block().epoch(),
            round,
            timestamp_usecs,
            quorum_cert,
            block_type: BlockType::Proposal { payload, author },
        }
    }
}
