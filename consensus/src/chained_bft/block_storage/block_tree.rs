// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockTreeError, VoteReceptionResult},
        consensus_types::{
            block::Block, quorum_cert::QuorumCert, vote_data::VoteData, vote_msg::VoteMsg,
        },
    },
    counters,
    state_replication::{ExecutedState, StateComputeResult},
    util::time_service::duration_since_epoch,
};
use canonical_serialization::CanonicalSerialize;
use crypto::{hash::CryptoHash, HashValue};
use logger::prelude::*;
use mirai_annotations::checked_verify_eq;
use serde::Serialize;
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        vec_deque::VecDeque,
        HashMap,
    },
    fmt::Debug,
    sync::Arc,
    time::Duration,
};
use types::crypto_proxies::LedgerInfoWithSignatures;

/// This structure maintains a consistent block tree of parent and children links. Blocks contain
/// parent links and are immutable.  For all parent links, a child link exists. This structure
/// should only be used internally in BlockStore.
pub struct BlockTree<T> {
    /// All the blocks known to this replica (with parent links)
    id_to_block: HashMap<HashValue, Arc<Block<T>>>,
    /// All child links (i.e. reverse parent links) for easy cleaning.  Note that a block may
    /// have multiple child links.  There should be id_to_blocks.len() - 1 total
    /// id_to_child entries.
    id_to_child: HashMap<HashValue, Vec<Arc<Block<T>>>>,
    /// Mapping between proposals(Block) to execution results.
    id_to_state: HashMap<HashValue, ExecutedState>,
    /// Keeps the state compute results of the executed blocks.
    /// The state compute results is calculated for all the pending blocks prior to insertion to
    /// the tree (the initial root node might not have it, because it's been already
    /// committed). The execution results are not persisted: they're recalculated again for the
    /// pending blocks upon restart.
    id_to_compute_result: HashMap<HashValue, Arc<StateComputeResult>>,
    /// Root of the tree.
    root: Arc<Block<T>>,
    /// A certified block with highest round
    highest_certified_block: Arc<Block<T>>,
    /// The quorum certificate of highest_certified_block
    highest_quorum_cert: Arc<QuorumCert>,
    /// The quorum certificate that carries a highest ledger info
    highest_ledger_info: Arc<QuorumCert>,

    /// `id_to_votes` might keep multiple LedgerInfos per proposed block in order
    /// to tolerate non-determinism in execution: given a proposal, a QuorumCertificate is going
    /// to be collected only for all the votes that carry identical LedgerInfo.
    /// LedgerInfo digest covers the potential commit ids, as well as the vote information
    /// (including the 3-chain of a voted proposal).
    /// Thus, the structure of `id_to_votes` is as follows:
    /// HashMap<proposed_block_id, HashMap<ledger_info_digest, LedgerInfoWithSignatures>>
    id_to_votes: HashMap<HashValue, HashMap<HashValue, LedgerInfoWithSignatures>>,
    /// Map of block id to its completed quorum certificate (2f + 1 votes)
    id_to_quorum_cert: HashMap<HashValue, Arc<QuorumCert>>,
    /// To keep the IDs of the elements that have been pruned from the tree but not cleaned up yet.
    pruned_block_ids: VecDeque<HashValue>,
    /// Num pruned blocks to keep in memory.
    max_pruned_blocks_in_mem: usize,
}

impl<T> BlockTree<T>
where
    T: Serialize + Default + Debug + CanonicalSerialize + PartialEq,
{
    pub(super) fn new(
        root: Block<T>,
        root_quorum_cert: QuorumCert,
        root_ledger_info: QuorumCert,
        max_pruned_blocks_in_mem: usize,
    ) -> Self {
        assert_eq!(
            root.id(),
            root_ledger_info
                .ledger_info()
                .ledger_info()
                .consensus_block_id(),
            "inconsistent root and ledger info"
        );
        let root = Arc::new(root);
        let mut id_to_block = HashMap::new();
        id_to_block.insert(root.id(), root.clone());
        counters::NUM_BLOCKS_IN_TREE.set(1);

        let root_quorum_cert = Arc::new(root_quorum_cert);
        let mut id_to_quorum_cert = HashMap::new();
        id_to_quorum_cert.insert(
            root_quorum_cert.certified_block_id(),
            Arc::clone(&root_quorum_cert),
        );

        let mut id_to_state = HashMap::new();
        id_to_state.insert(root.id(), root_quorum_cert.certified_state());

        let pruned_block_ids = VecDeque::with_capacity(max_pruned_blocks_in_mem);

        BlockTree {
            id_to_block,
            id_to_child: HashMap::new(),
            id_to_state,
            id_to_compute_result: HashMap::new(),
            root: Arc::clone(&root),
            highest_certified_block: Arc::clone(&root),
            highest_quorum_cert: Arc::clone(&root_quorum_cert),
            highest_ledger_info: Arc::new(root_ledger_info),
            id_to_votes: HashMap::new(),
            id_to_quorum_cert,
            pruned_block_ids,
            max_pruned_blocks_in_mem,
        }
    }

    fn remove_block(&mut self, block_id: HashValue) {
        // Delete my child links
        self.id_to_child.remove(&block_id);
        // Remove the block from the store
        self.id_to_block.remove(&block_id);
        self.id_to_state.remove(&block_id);
        self.id_to_compute_result.remove(&block_id);
        self.id_to_votes.remove(&block_id);
        self.id_to_quorum_cert.remove(&block_id);
    }

    pub(super) fn block_exists(&self, block_id: HashValue) -> bool {
        self.id_to_block.contains_key(&block_id)
    }

    pub(super) fn get_block(&self, block_id: HashValue) -> Option<Arc<Block<T>>> {
        self.id_to_block.get(&block_id).cloned()
    }

    pub(super) fn get_state_for_block(&self, block_id: HashValue) -> Option<ExecutedState> {
        self.id_to_state.get(&block_id).cloned()
    }

    pub(super) fn get_compute_result(
        &self,
        block_id: HashValue,
    ) -> Option<Arc<StateComputeResult>> {
        self.id_to_compute_result.get(&block_id).cloned()
    }

    pub(super) fn root(&self) -> Arc<Block<T>> {
        self.root.clone()
    }

    pub(super) fn highest_certified_block(&self) -> Arc<Block<T>> {
        Arc::clone(&self.highest_certified_block)
    }

    pub(super) fn highest_quorum_cert(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_quorum_cert)
    }

    pub(super) fn highest_ledger_info(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_ledger_info)
    }

    pub(super) fn get_quorum_cert_for_block(&self, block_id: HashValue) -> Option<Arc<QuorumCert>> {
        self.id_to_quorum_cert.get(&block_id).cloned()
    }

    pub(super) fn insert_block(
        &mut self,
        block: Block<T>,
        state: ExecutedState,
        compute_result: StateComputeResult,
    ) -> Result<Arc<Block<T>>, BlockTreeError> {
        if !self.block_exists(block.parent_id()) {
            return Err(BlockTreeError::BlockNotFound {
                id: block.parent_id(),
            });
        }
        let block = Arc::new(block);

        match self.id_to_block.get(&block.id()) {
            Some(previous_block) => {
                debug!("Already had block {:?} for id {:?} when trying to add another block {:?} for the same id",
                       previous_block,
                       block.id(),
                       block);
                checked_verify_eq!(*self.id_to_state.get(&block.id()).unwrap(), state);
                Ok(previous_block.clone())
            }
            _ => {
                let children = match self.id_to_child.entry(block.parent_id()) {
                    Vacant(entry) => entry.insert(Vec::new()),
                    Occupied(entry) => entry.into_mut(),
                };
                children.push(block.clone());
                counters::NUM_BLOCKS_IN_TREE.inc();
                self.id_to_block.insert(block.id(), block.clone());
                self.id_to_state.insert(block.id(), state);
                self.id_to_compute_result
                    .insert(block.id(), Arc::new(compute_result));
                Ok(block)
            }
        }
    }

    pub(super) fn insert_quorum_cert(&mut self, qc: QuorumCert) -> Result<(), BlockTreeError> {
        let block_id = qc.certified_block_id();
        let qc = Arc::new(qc);
        match self.id_to_block.get(&block_id) {
            Some(block) => {
                if block.round() > self.highest_certified_block.round() {
                    self.highest_certified_block = block.clone();
                    self.highest_quorum_cert = Arc::clone(&qc);
                }
            }
            None => return Err(BlockTreeError::BlockNotFound { id: block_id }),
        }

        self.id_to_quorum_cert
            .entry(block_id)
            .or_insert_with(|| Arc::clone(&qc));

        let committed_block_id = qc.ledger_info().ledger_info().consensus_block_id();
        if let Some(block) = self.id_to_block.get(&committed_block_id) {
            if block.round()
                > self
                    .id_to_block
                    .get(
                        &self
                            .highest_ledger_info
                            .ledger_info()
                            .ledger_info()
                            .consensus_block_id(),
                    )
                    .expect("Highest ledger info's block should exist")
                    .round()
            {
                self.highest_ledger_info = qc;
            }
        }
        Ok(())
    }

    pub(super) fn insert_vote(
        &mut self,
        vote_msg: &VoteMsg,
        min_votes_for_qc: usize,
    ) -> VoteReceptionResult {
        let block_id = vote_msg.block_id();
        if let Some(old_qc) = self.id_to_quorum_cert.get(&block_id) {
            return VoteReceptionResult::OldQuorumCertificate(Arc::clone(old_qc));
        }

        // All the votes collected for all the execution results of a given proposal.
        let block_votes = self
            .id_to_votes
            .entry(block_id)
            .or_insert_with(HashMap::new);

        // Note that the digest covers the ledger info information, which is also indirectly
        // covering vote data hash (in its `consensus_data_hash` field).
        let digest = vote_msg.ledger_info().hash();
        let li_with_sig = block_votes.entry(digest).or_insert_with(|| {
            LedgerInfoWithSignatures::new(vote_msg.ledger_info().clone(), HashMap::new())
        });
        let author = vote_msg.author();
        if li_with_sig.signatures().contains_key(&author) {
            return VoteReceptionResult::DuplicateVote;
        }
        vote_msg.signature().clone().add_to_li(author, li_with_sig);

        let num_votes = li_with_sig.signatures().len();
        if num_votes >= min_votes_for_qc {
            let quorum_cert = QuorumCert::new(
                VoteData::new(
                    block_id,
                    vote_msg.executed_state(),
                    vote_msg.block_round(),
                    vote_msg.parent_block_id(),
                    vote_msg.parent_block_round(),
                    vote_msg.grandparent_block_id(),
                    vote_msg.grandparent_block_round(),
                ),
                li_with_sig.clone(),
            );
            // Note that the block might not be present locally, in which case we cannot calculate
            // time between block creation and qc
            if let Some(block) = self.get_block(block_id) {
                if let Some(time_to_qc) = duration_since_epoch()
                    .checked_sub(Duration::from_micros(block.timestamp_usecs()))
                {
                    counters::CREATION_TO_QC_S.observe_duration(time_to_qc);
                }
            }
            return VoteReceptionResult::NewQuorumCertificate(Arc::new(quorum_cert));
        }
        VoteReceptionResult::VoteAdded(num_votes)
    }

    /// Find the blocks to prune up to next_root_id (keep next_root_id's block). Any branches not
    /// part of the next_root_id's tree should be removed as well.
    ///
    /// For example, root = B_0
    /// B_0 -> B_1 -> B_2
    ///         |  -> B_3 -> B4
    ///
    /// prune_tree(B_3) should be left with
    /// B_3 -> B_4, root = B_3
    ///
    /// Note this function is read-only, use with process_pruned_blocks to do the actual prune.
    pub(super) fn find_blocks_to_prune(&self, next_root_id: HashValue) -> VecDeque<HashValue> {
        // Nothing to do if this is the root
        if next_root_id == self.root.id() {
            return VecDeque::new();
        }

        let mut blocks_pruned = VecDeque::new();
        let mut blocks_to_be_pruned = Vec::new();
        blocks_to_be_pruned.push(self.root.clone());
        while let Some(block_to_remove) = blocks_to_be_pruned.pop() {
            // Add the children to the blocks to be pruned (if any), but stop when it reaches the
            // new root
            if let Some(children) = self.id_to_child.get(&block_to_remove.id()) {
                for child in children {
                    if next_root_id == child.id() {
                        continue;
                    }

                    blocks_to_be_pruned.push(child.clone());
                }
            }
            // Track all the block ids removed
            blocks_pruned.push_back(block_to_remove.id());
        }
        blocks_pruned
    }

    /// Process the data returned by the prune_tree, they're separated because caller might
    /// be interested in doing extra work e.g. delete from persistent storage.
    /// Note that we do not necessarily remove the pruned blocks: they're kept in a separate buffer
    /// for some time in order to enable other peers to retrieve the blocks even after they've
    /// been committed.
    pub(super) fn process_pruned_blocks(
        &mut self,
        root_id: HashValue,
        mut newly_pruned_blocks: VecDeque<HashValue>,
    ) {
        // Update the next root
        self.root = self
            .id_to_block
            .get(&root_id)
            .expect("next_root_id must exist")
            .clone();

        counters::NUM_BLOCKS_IN_TREE.sub(newly_pruned_blocks.len() as i64);
        // The newly pruned blocks are pushed back to the deque pruned_block_ids.
        // In case the overall number of the elements is greater than the predefined threshold,
        // the oldest elements (in the front of the deque) are removed from the tree.
        self.pruned_block_ids.append(&mut newly_pruned_blocks);
        if self.pruned_block_ids.len() > self.max_pruned_blocks_in_mem {
            let num_blocks_to_remove = self.pruned_block_ids.len() - self.max_pruned_blocks_in_mem;
            for _ in 0..num_blocks_to_remove {
                if let Some(id) = self.pruned_block_ids.pop_front() {
                    self.remove_block(id);
                }
            }
        }
    }

    /// Returns all the blocks between the root and the given block, including the given block
    /// but excluding the root.
    /// In case a given block is not the successor of the root, return None.
    /// While generally the provided blocks should always belong to the active tree, there might be
    /// a race, in which the root of the tree is propagated forward between retrieving the block
    /// and getting its path from root (e.g., at proposal generator). Hence, we don't want to panic
    /// and prefer to return None instead.
    pub(super) fn path_from_root(&self, block: Arc<Block<T>>) -> Option<Vec<Arc<Block<T>>>> {
        let mut res = vec![];
        let mut cur_block = block;
        while cur_block.round() > self.root.round() {
            res.push(Arc::clone(&cur_block));
            cur_block = match self.get_block(cur_block.parent_id()) {
                None => {
                    return None;
                }
                Some(b) => b,
            };
        }
        // At this point cur_block.round() <= self.root.round()
        if cur_block.id() != self.root.id() {
            return None;
        }
        Some(res)
    }

    pub(super) fn max_pruned_blocks_in_mem(&self) -> usize {
        self.max_pruned_blocks_in_mem
    }

    pub(super) fn get_all_block_id(&self) -> Vec<HashValue> {
        self.id_to_block.keys().cloned().collect()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl<T> BlockTree<T>
where
    T: Serialize + Default + Debug + CanonicalSerialize + PartialEq,
{
    /// Returns the number of blocks in the tree
    pub(super) fn len(&self) -> usize {
        // BFS over the tree to find the number of blocks in the tree.
        let mut res = 0;
        let mut to_visit = Vec::new();
        to_visit.push(Arc::clone(&self.root));
        while let Some(block) = to_visit.pop() {
            res += 1;
            if let Some(children) = self.id_to_child.get(&block.id()) {
                for child in children {
                    to_visit.push(Arc::clone(&child));
                }
            }
        }
        res
    }

    /// Returns the number of child links in the tree
    pub(super) fn child_links(&self) -> usize {
        self.len() - 1
    }

    /// The number of pruned blocks that are still available in memory
    pub(super) fn pruned_blocks_in_mem(&self) -> usize {
        self.pruned_block_ids.len()
    }
}
