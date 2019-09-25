// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Author, Round},
    consensus_types::{
        block::{Block, ExecutedBlock},
        quorum_cert::QuorumCert,
    },
};
use crypto::HashValue;
use std::sync::Arc;

mod block_store;
mod block_tree;

pub use block_store::{BlockStore, NeedFetchResult};
use executor::StateComputeResult;
use network::protocols::rpc::error::RpcError;

#[derive(Debug, PartialEq, Fail)]
/// The possible reasons for failing to retrieve a block by id from a given peer.
#[derive(Clone)]
pub enum BlockRetrievalFailure {
    /// Could not find a given author
    #[fail(display = "Unknown author: {:?}", author)]
    #[allow(dead_code)]
    UnknownAuthor { author: Author },

    /// Any sort of a network failure (should probably have an enum for network failures).
    #[fail(display = "Network failure: {:?}", msg)]
    NetworkFailure { msg: String },

    /// The remote peer did not recognize the given block id.
    #[fail(display = "Block id {:?} not recognized by the peer", block_id)]
    #[allow(dead_code)]
    UnknownBlockId { block_id: HashValue },

    /// Cannot retrieve a block from itself
    #[fail(display = "Attempt of a self block retrieval.")]
    SelfRetrieval,

    /// The event is not correctly signed.
    #[fail(display = "InvalidSignature")]
    InvalidSignature,

    /// The response is not valid: status doesn't match blocks, blocks unable to deserialize etc.
    #[fail(display = "InvalidResponse")]
    InvalidResponse,
}

impl From<RpcError> for BlockRetrievalFailure {
    fn from(source: RpcError) -> Self {
        BlockRetrievalFailure::NetworkFailure {
            msg: source.to_string(),
        }
    }
}

/// Result of the vote processing. The failure case (Verification error) is returned
/// as the Error part of the result.
#[derive(Debug, PartialEq)]
pub enum VoteReceptionResult {
    /// The vote has been added but QC has not been formed yet. Return the number of votes for
    /// the given (proposal, execution) pair.
    VoteAdded(usize),
    /// The very same vote message has been processed in past.
    DuplicateVote,
    /// This block has been already certified.
    OldQuorumCertificate(Arc<QuorumCert>),
    /// This block has just been certified after adding the vote.
    NewQuorumCertificate(Arc<QuorumCert>),
}

pub trait BlockReader: Send + Sync {
    type Payload;

    /// Check if a block with the block_id exist in the BlockTree.
    fn block_exists(&self, block_id: HashValue) -> bool;

    /// Try to get a block with the block_id, return an Arc of it if found.
    fn get_block(&self, block_id: HashValue) -> Option<Arc<ExecutedBlock<Self::Payload>>>;

    /// Try to get a compute result given the specified block id.
    ///
    /// Returns an option since all blocks should have a compute result or None for root block
    /// since it has already been committed.
    fn get_compute_result(&self, block_id: HashValue) -> Option<Arc<StateComputeResult>>;

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

    /// Generates and returns a block with the given parent and payload.
    /// Note that it does not add the block to the tree, just generates it.
    /// The main reason we want this function in the BlockStore is the fact that the signer required
    /// for signing the newly created block is held by the block store.
    /// The function panics in the following cases:
    /// * If the parent or its quorum certificate are not present in the tree,
    /// * If the given round (which is typically calculated by Pacemaker) is not greater than that
    ///   of a parent.
    fn create_block(
        &self,
        parent: &Block<Self::Payload>,
        payload: Self::Payload,
        round: Round,
        timestamp_usecs: u64,
    ) -> Block<Self::Payload>;

    /// Return the certified block with the highest round.
    fn highest_certified_block(&self) -> Arc<ExecutedBlock<Self::Payload>>;

    /// Return the quorum certificate with the highest round
    fn highest_quorum_cert(&self) -> Arc<QuorumCert>;

    /// Return the quorum certificate that carries ledger info with the highest round
    fn highest_ledger_info(&self) -> Arc<QuorumCert>;
}
