// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockTreeError, VoteReceptionResult},
        consensus_types::{block::Block, quorum_cert::QuorumCert},
        safety::vote_msg::VoteMsg,
    },
    counters,
    state_replication::{ExecutedState, SpeculationResult},
    util::time_service::duration_since_epoch,
};
use canonical_serialization::CanonicalSerialize;
use crypto::HashValue;
use logger::prelude::*;
use mirai_annotations::checked_verify_eq;
use serde::Serialize;
use std::{
    collections::{vec_deque::VecDeque, HashMap},
    fmt::Debug,
    sync::Arc,
    time::Duration,
};
use types::ledger_info::LedgerInfoWithSignatures;

/// This structure maintains a consistent block tree of parent and children links. Blocks contain
/// parent links and are immutable.  For all parent links, a child link exists. This structure
/// should only be used internally in BlockStore.
pub struct BlockTree<T> {
    /// All the blocks keyed by its id known to this replica (with parent links)
    id_to_block: HashMap<HashValue, Block<T>>,
    /// Root of the tree.
    root_id: HashValue,
    /// A certified block id with highest round
    highest_certified_block_id: HashValue,
    /// The quorum certificate of highest_certified_block
    highest_quorum_cert: Arc<QuorumCert>,
    /// The quorum certificate that carries a highest ledger info
    highest_ledger_info: Arc<QuorumCert>,
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
        root_ledger_info: QuorumCert,
        max_pruned_blocks_in_mem: usize,
    ) -> Self {
        assert_eq!(
            root.id(),
            root_ledger_info
                .ledger_info_with_sigs()
                .ledger_info()
                .consensus_block_id(),
            "inconsistent root and ledger info"
        );
        let root_id = root.id();
        let highest_quorum_cert = Arc::clone(root.quorum_cert());
        let mut id_to_block = HashMap::new();
        id_to_block.insert(root_id, root);
        counters::NUM_BLOCKS_IN_TREE.set(1);

        let pruned_block_ids = VecDeque::with_capacity(max_pruned_blocks_in_mem);

        BlockTree {
            id_to_block,
            root_id,
            highest_certified_block_id: root_id,
            highest_quorum_cert,
            highest_ledger_info: Arc::new(root_ledger_info),
            pruned_block_ids,
            max_pruned_blocks_in_mem,
        }
    }

    fn remove_block(&mut self, block_id: HashValue) {
        self.id_to_block.remove(&block_id);
    }

    pub(super) fn block_exists(&self, block_id: HashValue) -> bool {
        self.id_to_block.contains_key(&block_id)
    }

    pub(super) fn get_block(&self, block_id: HashValue) -> Option<&Block<T>> {
        self.id_to_block.get(&block_id)
    }

    fn get_block_mut(&self, block_id: HashValue) -> Option<&mut Block<T>> {
        self.id_to_block.get_mut(&block_id)
    }

    fn get_votes_mut(&self, block_id: HashValue) -> Option<&mut Block<T>> {
        self.get_block_mut(&block_id).map(|b| b.votes)
    }

    pub(super) fn get_state_for_block(&self, block_id: HashValue) -> Option<ExecutedState> {
        self.id_to_state.get(&block_id).cloned()
    }

    pub(super) fn get_speculation_result(
        &self,
        block_id: HashValue,
    ) -> Option<Arc<SpeculationResult>> {
        self.get_block(&block_id).cloned()
    }

    pub(super) fn root(&self) -> &Block<T> {
        self.root.clone()
    }

    pub(super) fn highest_certified_block(&self) -> &Block<T> {
        self.get_block(&self.highest_certified_block)
            .expect("Highest certified block must exist.")
    }

    pub(super) fn highest_quorum_cert(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_quorum_cert)
    }

    pub(super) fn highest_ledger_info(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_ledger_info)
    }

    pub(super) fn get_quorum_cert(&self, block_id: HashValue) -> Option<Arc<QuorumCert>> {
        self.get_block(&block_id).map(|b| b.quorum_cert().clone())
    }

    pub(super) fn insert_block(&mut self, block: Block<T>) -> Result<&Block<T>, BlockTreeError> {
        let block_id = block.id();
        match self.get_block.get(&block_id) {
            Some(previous_block) => {
                debug!("Already had block {:?} for id {:?} when trying to add another block {:?} for the same id",
                       previous_block,
                       block.id(),
                       block);
                checked_verify_eq!(
                    previous_block.speculative_state(),
                    block.speculative_state()
                );
                Ok(previous_block)
            }
            None => match self.get_block_mut(&block.parent_id()) {
                Some(parent_block) => parent_block.add_child(block_id),
                None => bail_err!(BlockTreeError::BlockNotFound {
                    id: block.parent_id(),
                }),
            },
        };
        counters::NUM_BLOCKS_IN_TREE.inc();
        self.id_to_block.insert(block_id, block);
        Ok(self.get_block(block_id).expect("Must exist"))
    }

    pub(super) fn insert_quorum_cert(&mut self, qc: Arc<QuorumCert>) -> Result<(), BlockTreeError> {
        let block_id = qc.certified_block_id();
        match self.get_block_mut(&block_id) {
            Some(block) => {
                if block.round() > self.highest_certified_block().round() {
                    self.highest_certified_block = block.clone();
                    self.highest_quorum_cert = Arc::clone(&qc);
                }
                block.set_quorum_cert(Arc::clone(&qc));
            }
            None => return Err(BlockTreeError::BlockNotFound { id: block_id }),
        }

        let committed_block_id = qc
            .ledger_info_with_sigs()
            .ledger_info()
            .consensus_block_id();
        if let Some(block) = self.get_block(&committed_block_id) {
            if block.round()
                > self
                    .get_block(
                        &self
                            .highest_ledger_info
                            .ledger_info_with_sigs()
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
        let block_id = vote_msg.proposed_block_id();
        if let Some(old_qc) = self.get_quorum_cert(&block_id) {
            return VoteReceptionResult::OldQuorumCertificate(Arc::clone(old_qc));
        }

        // All the votes collected for all the execution results of a given proposal.
        let block_votes = self
            .id_to_votes
            .entry(block_id)
            .or_insert_with(HashMap::new);

        // Note that the digest covers not just the proposal id, but also the resulting
        // state id as well as the round number. In other words, if two different voters have the
        // same digest then they reached the same state following the same proposals.
        let digest = vote_msg.vote_hash();
        let ledger_info_with_sig = self
            .get_block_mut(&block_id)
            .get_votes_mut()
            .entry(digest)
            .or_insert_with(|| {
                LedgerInfoWithSignatures::new(vote_msg.ledger_info().clone(), HashMap::new())
            });
        let author = vote_msg.author();
        if ledger_info_with_sig.signatures().contains_key(&author) {
            return VoteReceptionResult::DuplicateVote;
        }
        ledger_info_with_sig.add_signature(author, vote_msg.signature().clone());

        let num_votes = ledger_info_with_sig.signatures().len();
        if num_votes >= min_votes_for_qc {
            let quorum_cert = QuorumCert::new(
                block_id,
                vote_msg.executed_state(),
                vote_msg.round(),
                ledger_info_with_sig.clone(),
                vote_msg.parent_block_id(),
                vote_msg.parent_block_round(),
                vote_msg.grandparent_block_id(),
                vote_msg.grandparent_block_round(),
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

    /// Find the blocks to prune up to next_root_id (keep next_root_id's block). Any branches being
    /// not part of the next_root_id's subtree should be removed as well.
    ///
    /// For example, root = B0
    /// B0--> B1--> B2
    ///        └--> B3 --> B4
    ///
    /// prune_tree(B3) should be left with
    /// B3--> B4, root = B3
    ///
    /// Note this function is read-only, use with process_pruned_blocks to do the actual prune.
    pub(super) fn find_blocks_to_prune(&self, next_root_id: HashValue) -> VecDeque<HashValue> {
        // Nothing to do if this is the root
        if next_root_id == self.root_id {
            return VecDeque::new();
        }

        let mut blocks_pruned = VecDeque::new();
        let mut blocks_to_be_pruned = Vec::new();
        blocks_to_be_pruned.push(self.root.clone());
        while let Some(block_to_remove) = blocks_to_be_pruned.pop() {
            // Add the children to the blocks to be pruned (if any), but stop when it reaches the
            // new root
            for child in block_to_remove.children() {
                if next_root_id == *child {
                    continue;
                }

                blocks_to_be_pruned.push(child.clone());
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
            .get_block(&root_id)
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
    /// While generally the provided block should always belong to the active tree, there might be
    /// a race, in which the root of the tree is propagated forward between retrieving the block
    /// and getting its path from root (e.g., at proposal generator). Hence, we don't want to panic
    /// and prefer to return None instead.
    pub(super) fn path_from_root(&self, block: &Block<T>) -> Option<Vec<&Block<T>>> {
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
        if cur_block.id() != self.root_id {
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

#[cfg(test)]
impl<T> BlockTree<T>
where
    T: Serialize + Default + Debug + CanonicalSerialize + PartialEq,
{
    /// Returns the number of blocks in the tree
    pub(super) fn len(&self) -> usize {
        // BFS over the tree to find the number of blocks in the tree.
        let mut res = 0;
        let mut to_visit = Vec::new();
        to_visit.push(self.root_id);
        while let Some(block_id) = to_visit.pop() {
            res += 1;
            for child_id in self.get_block(&block_id).children() {
                to_visit.push(*child_id);
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
