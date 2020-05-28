// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{Author, Round},
    quorum_cert::QuorumCert,
    vote_data::VoteData,
};
use libra_crypto::hash::HashValue;
use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use libra_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use mirai_annotations::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BlockType<T> {
    Proposal {
        /// T of the block (e.g. one or more transaction(s)
        payload: T,
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
pub struct BlockData<T> {
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

impl<T> BlockData<T> {
    pub fn author(&self) -> Option<Author> {
        if let BlockType::Proposal { author, .. } = self.block_type {
            Some(author)
        } else {
            None
        }
    }

    pub fn block_type(&self) -> &BlockType<T> {
        &self.block_type
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert.certified_block().id()
    }

    pub fn payload(&self) -> Option<&T> {
        if let BlockType::Proposal { payload, .. } = &self.block_type {
            Some(payload)
        } else {
            None
        }
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
}

impl<T> BlockData<T> {
    pub fn is_genesis_block(&self) -> bool {
        matches!(self.block_type, BlockType::Genesis)
    }

    pub fn is_nil_block(&self) -> bool {
        matches!(self.block_type, BlockType::NilBlock)
    }
}

impl<T> BlockData<T>
where
    T: Default + Serialize,
{
    pub fn new_genesis_from_ledger_info(ledger_info: &LedgerInfo) -> Self {
        assert!(ledger_info.next_epoch_state().is_some());
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
        payload: T,
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
