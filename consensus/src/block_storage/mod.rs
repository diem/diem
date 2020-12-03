// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{
    executed_block::ExecutedBlock, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate,
};
use diem_crypto::HashValue;
use std::sync::Arc;

mod block_store;
mod block_tree;
pub mod tracing;

pub use block_store::{sync_manager::BlockRetriever, BlockStore};
use consensus_types::sync_info::SyncInfo;

pub trait BlockReader: Send + Sync {
    /// Check if a block with the block_id exist in the BlockTree.
    fn block_exists(&self, block_id: HashValue) -> bool;

    /// Try to get a block with the block_id, return an Arc of it if found.
    fn get_block(&self, block_id: HashValue) -> Option<Arc<ExecutedBlock>>;

    /// Get the current root block of the BlockTree.
    fn root(&self) -> Arc<ExecutedBlock>;

    fn get_quorum_cert_for_block(&self, block_id: HashValue) -> Option<Arc<QuorumCert>>;

    /// Returns all the blocks between the root and the given block, including the given block
    /// but excluding the root.
    /// In case a given block is not the successor of the root, return None.
    /// For example if a tree is b0 <- b1 <- b2 <- b3, then
    /// path_from_root(b2) -> Some([b2, b1])
    /// path_from_root(b0) -> Some([])
    /// path_from_root(a) -> None
    fn path_from_root(&self, block_id: HashValue) -> Option<Vec<Arc<ExecutedBlock>>>;

    /// Return the certified block with the highest round.
    fn highest_certified_block(&self) -> Arc<ExecutedBlock>;

    /// Return the quorum certificate with the highest round
    fn highest_quorum_cert(&self) -> Arc<QuorumCert>;

    /// Return the quorum certificate that carries ledger info with the highest round
    fn highest_commit_cert(&self) -> Arc<QuorumCert>;

    /// Return the highest timeout certificate if available.
    fn highest_timeout_cert(&self) -> Option<Arc<TimeoutCertificate>>;

    /// Return the combination of highest quorum cert, timeout cert and commit cert.
    fn sync_info(&self) -> SyncInfo;
}
