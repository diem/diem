// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{
    block::Block, executed_block::ExecutedBlock, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate,
};
use libra_crypto::HashValue;
use libra_types::validator_verifier::VerifyError;
use std::sync::Arc;

mod block_store;
mod block_tree;
mod pending_votes;

pub use block_store::{sync_manager::BlockRetriever, BlockStore};
pub use pending_votes::PendingVotes;

/// Result of the vote processing. The failure case (Verification error) is returned
/// as the Error part of the result.
#[derive(Debug, PartialEq)]
pub enum VoteReceptionResult {
    /// The vote has been added but QC has not been formed yet. Return the amount of voting power
    /// the given (proposal, execution) pair.
    VoteAdded(u64),
    /// The very same vote message has been processed in past.
    DuplicateVote,
    /// The very same author has already voted for another proposal in this round (equivocation).
    EquivocateVote,
    /// This block has just been certified after adding the vote.
    NewQuorumCertificate(Arc<QuorumCert>),
    /// The vote completes a new TimeoutCertificate
    NewTimeoutCertificate(Arc<TimeoutCertificate>),
    /// There might be some issues adding a vote
    ErrorAddingVote(VerifyError),
}

pub trait BlockReader: Send + Sync {
    type Payload;

    /// Check if a block with the block_id exist in the BlockTree.
    fn block_exists(&self, block_id: HashValue) -> bool;

    /// Try to get a block with the block_id, return an Arc of it if found.
    fn get_block(&self, block_id: HashValue) -> Option<Arc<ExecutedBlock<Self::Payload>>>;

    /// Retrieve the chain of descendants of a committed block.
    /// The chain ends with the given block id, and includes
    /// the LedgerInfo certifying the commit of the target block
    /// In case no such chain can be found, an empty vector is returned.
    fn get_chain_for_committed_block(&self, block_id: HashValue) -> Vec<Block<Self::Payload>>;

    /// Get the current root block of the BlockTree.
    fn root(&self) -> Arc<ExecutedBlock<Self::Payload>>;

    fn get_quorum_cert_for_block(&self, block_id: HashValue) -> Option<Arc<QuorumCert>>;

    /// Returns all the blocks between the root and the given block, including the given block
    /// but excluding the root.
    /// In case a given block is not the successor of the root, return None.
    /// For example if a tree is b0 <- b1 <- b2 <- b3, then
    /// path_from_root(b2) -> Some([b2, b1])
    /// path_from_root(b0) -> Some([])
    /// path_from_root(a) -> None
    fn path_from_root(&self, block_id: HashValue)
        -> Option<Vec<Arc<ExecutedBlock<Self::Payload>>>>;

    /// Return the certified block with the highest round.
    fn highest_certified_block(&self) -> Arc<ExecutedBlock<Self::Payload>>;

    /// Return the quorum certificate with the highest round
    fn highest_quorum_cert(&self) -> Arc<QuorumCert>;

    /// Return the quorum certificate that carries ledger info with the highest round
    fn highest_commit_cert(&self) -> Arc<QuorumCert>;

    /// Return the highest timeout certificate if available.
    fn highest_timeout_cert(&self) -> Option<Arc<TimeoutCertificate>>;
}
