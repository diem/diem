// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::pending_votes::PendingVotes;
use crate::{
    chained_bft::block_storage::VoteReceptionResult, counters,
    util::time_service::duration_since_epoch,
};
use anyhow::bail;
use consensus_types::{
    executed_block::ExecutedBlock, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate, vote::Vote,
};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::crypto_proxies::ValidatorVerifier;
use mirai_annotations::{checked_verify_eq, precondition};
use serde::Serialize;
use std::{
    collections::{vec_deque::VecDeque, HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
    time::Duration,
};

/// This structure is a wrapper of [`ExecutedBlock`](crate::consensus_types::block::ExecutedBlock)
/// that adds `children` field to know the parent-child relationship between blocks.
struct LinkableBlock<T> {
    /// Executed block that has raw block data and execution output.
    executed_block: Arc<ExecutedBlock<T>>,
    /// The set of children for cascading pruning. Note: a block may have multiple children.
    children: HashSet<HashValue>,
}

impl<T> LinkableBlock<T> {
    pub fn new(block: ExecutedBlock<T>) -> Self {
        Self {
            executed_block: Arc::new(block),
            children: HashSet::new(),
        }
    }

    pub fn executed_block(&self) -> &Arc<ExecutedBlock<T>> {
        &self.executed_block
    }

    pub fn children(&self) -> &HashSet<HashValue> {
        &self.children
    }

    pub fn add_child(&mut self, child_id: HashValue) {
        assert!(
            self.children.insert(child_id),
            "Block {:x} already existed.",
            child_id,
        );
    }
}

impl<T> LinkableBlock<T>
where
    T: Serialize + Default + PartialEq,
{
    pub fn id(&self) -> HashValue {
        self.executed_block().id()
    }
}

/// This structure maintains a consistent block tree of parent and children links. Blocks contain
/// parent links and are immutable.  For all parent links, a child link exists. This structure
/// should only be used internally in BlockStore.
pub struct BlockTree<T> {
    /// All the blocks known to this replica (with parent links)
    id_to_block: HashMap<HashValue, LinkableBlock<T>>,
    /// Root of the tree.
    root_id: HashValue,
    /// A certified block id with highest round
    highest_certified_block_id: HashValue,

    /// The quorum certificate of highest_certified_block
    highest_quorum_cert: Arc<QuorumCert>,
    /// The highest timeout certificate (if any).
    highest_timeout_cert: Option<Arc<TimeoutCertificate>>,
    /// The quorum certificate that has highest commit info.
    highest_commit_cert: Arc<QuorumCert>,
    /// Manages pending votes to be aggregated.
    pending_votes: PendingVotes,
    /// Map of block id to its completed quorum certificate (2f + 1 votes)
    id_to_quorum_cert: HashMap<HashValue, Arc<QuorumCert>>,
    /// To keep the IDs of the elements that have been pruned from the tree but not cleaned up yet.
    pruned_block_ids: VecDeque<HashValue>,
    /// Num pruned blocks to keep in memory.
    max_pruned_blocks_in_mem: usize,
}

impl<T> BlockTree<T>
where
    T: Serialize + Default + Debug + PartialEq,
{
    pub(super) fn new(
        root: ExecutedBlock<T>,
        root_quorum_cert: QuorumCert,
        root_ledger_info: QuorumCert,
        max_pruned_blocks_in_mem: usize,
        highest_timeout_cert: Option<Arc<TimeoutCertificate>>,
    ) -> Self {
        assert_eq!(
            root.id(),
            root_ledger_info.commit_info().id(),
            "inconsistent root and ledger info"
        );
        let root_id = root.id();

        let mut id_to_block = HashMap::new();
        id_to_block.insert(root_id, LinkableBlock::new(root));
        counters::NUM_BLOCKS_IN_TREE.set(1);

        let root_quorum_cert = Arc::new(root_quorum_cert);
        let mut id_to_quorum_cert = HashMap::new();
        id_to_quorum_cert.insert(
            root_quorum_cert.certified_block().id(),
            Arc::clone(&root_quorum_cert),
        );

        let pruned_block_ids = VecDeque::with_capacity(max_pruned_blocks_in_mem);

        BlockTree {
            id_to_block,
            root_id,
            highest_certified_block_id: root_id,
            highest_quorum_cert: Arc::clone(&root_quorum_cert),
            highest_timeout_cert,
            highest_commit_cert: Arc::new(root_ledger_info),
            pending_votes: PendingVotes::new(),
            id_to_quorum_cert,
            pruned_block_ids,
            max_pruned_blocks_in_mem,
        }
    }

    // This method will only be used in this module.
    fn get_linkable_block(&self, block_id: &HashValue) -> Option<&LinkableBlock<T>> {
        self.id_to_block.get(block_id)
    }

    // This method will only be used in this module.
    fn get_linkable_block_mut(&mut self, block_id: &HashValue) -> Option<&mut LinkableBlock<T>> {
        self.id_to_block.get_mut(block_id)
    }

    // This method will only be used in this module.
    fn linkable_root(&self) -> &LinkableBlock<T> {
        self.get_linkable_block(&self.root_id)
            .expect("Root must exist")
    }

    fn remove_block(&mut self, block_id: HashValue) {
        // Remove the block from the store
        self.id_to_block.remove(&block_id);
        self.id_to_quorum_cert.remove(&block_id);
    }

    pub(super) fn block_exists(&self, block_id: &HashValue) -> bool {
        self.id_to_block.contains_key(block_id)
    }

    pub(super) fn get_block(&self, block_id: &HashValue) -> Option<Arc<ExecutedBlock<T>>> {
        self.get_linkable_block(block_id)
            .map(|lb| Arc::clone(lb.executed_block()))
    }

    pub(super) fn root(&self) -> Arc<ExecutedBlock<T>> {
        self.get_block(&self.root_id).expect("Root must exist")
    }

    pub(super) fn highest_certified_block(&self) -> Arc<ExecutedBlock<T>> {
        self.get_block(&self.highest_certified_block_id)
            .expect("Highest cerfified block must exist")
    }

    pub(super) fn highest_quorum_cert(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_quorum_cert)
    }

    pub(super) fn highest_timeout_cert(&self) -> Option<Arc<TimeoutCertificate>> {
        self.highest_timeout_cert.clone()
    }

    /// Replace highest timeout cert with the given value.
    pub(super) fn replace_timeout_cert(&mut self, tc: Arc<TimeoutCertificate>) {
        self.highest_timeout_cert.replace(tc);
    }

    pub(super) fn highest_commit_cert(&self) -> Arc<QuorumCert> {
        Arc::clone(&self.highest_commit_cert)
    }

    pub(super) fn get_quorum_cert_for_block(
        &self,
        block_id: &HashValue,
    ) -> Option<Arc<QuorumCert>> {
        self.id_to_quorum_cert.get(block_id).cloned()
    }

    pub(super) fn insert_block(
        &mut self,
        block: ExecutedBlock<T>,
    ) -> anyhow::Result<Arc<ExecutedBlock<T>>> {
        let block_id = block.id();
        if let Some(existing_block) = self.get_block(&block_id) {
            debug!("Already had block {:?} for id {:?} when trying to add another block {:?} for the same id",
                       existing_block,
                       block_id,
                       block);
            checked_verify_eq!(existing_block.compute_result(), block.compute_result());
            Ok(existing_block)
        } else {
            match self.get_linkable_block_mut(&block.parent_id()) {
                Some(parent_block) => parent_block.add_child(block_id),
                None => bail!("Parent block {} not found", block.parent_id()),
            };
            let linkable_block = LinkableBlock::new(block);
            let arc_block = Arc::clone(linkable_block.executed_block());
            assert!(self.id_to_block.insert(block_id, linkable_block).is_none());
            counters::NUM_BLOCKS_IN_TREE.inc();
            Ok(arc_block)
        }
    }

    pub(super) fn insert_quorum_cert(&mut self, qc: QuorumCert) -> anyhow::Result<()> {
        let block_id = qc.certified_block().id();
        let qc = Arc::new(qc);

        // Safety invariant: For any two quorum certificates qc1, qc2 in the block store,
        // qc1 == qc2 || qc1.round != qc2.round
        // The invariant is quadratic but can be maintained in linear time by the check
        // below.
        precondition!({
            let qc_round = qc.certified_block().round();
            self.id_to_quorum_cert.values().all(|x| {
                (*(*x).ledger_info()).ledger_info().consensus_data_hash()
                    == (*(*qc).ledger_info()).ledger_info().consensus_data_hash()
                    || x.certified_block().round() != qc_round
            })
        });

        match self.get_block(&block_id) {
            Some(block) => {
                if block.round() > self.highest_certified_block().round() {
                    self.highest_certified_block_id = block.id();
                    self.highest_quorum_cert = Arc::clone(&qc);
                }
            }
            None => bail!("Block {} not found", block_id),
        }

        self.id_to_quorum_cert
            .entry(block_id)
            .or_insert_with(|| Arc::clone(&qc));

        if self.highest_commit_cert.commit_info().round() < qc.commit_info().round() {
            self.highest_commit_cert = qc;
        }

        Ok(())
    }

    pub(super) fn insert_vote(
        &mut self,
        vote: &Vote,
        validator_verifier: &ValidatorVerifier,
    ) -> VoteReceptionResult {
        let block_id = vote.vote_data().proposed().id();
        if let Some(old_qc) = self.id_to_quorum_cert.get(&block_id) {
            return VoteReceptionResult::OldQuorumCertificate(Arc::clone(old_qc));
        }
        let res = self.pending_votes.insert_vote(vote, validator_verifier);
        if let VoteReceptionResult::NewQuorumCertificate(_) = res {
            // Note that the block might not be present locally, in which case we cannot calculate
            // time between block creation and qc
            if let Some(time_to_qc) = self.get_block(&block_id).and_then(|block| {
                duration_since_epoch().checked_sub(Duration::from_micros(block.timestamp_usecs()))
            }) {
                counters::CREATION_TO_QC_S.observe_duration(time_to_qc);
            }
        }
        res
    }

    /// Find the blocks to prune up to next_root_id (keep next_root_id's block). Any branches not
    /// part of the next_root_id's tree should be removed as well.
    ///
    /// For example, root = B0
    /// B0--> B1--> B2
    ///        ╰--> B3--> B4
    ///
    /// prune_tree(B_3) should be left with
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
        blocks_to_be_pruned.push(self.linkable_root());
        while let Some(block_to_remove) = blocks_to_be_pruned.pop() {
            // Add the children to the blocks to be pruned (if any), but stop when it reaches the
            // new root
            for child_id in block_to_remove.children() {
                if next_root_id == *child_id {
                    continue;
                }
                blocks_to_be_pruned.push(
                    self.get_linkable_block(child_id)
                        .expect("Child must exist in the tree"),
                );
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
        assert!(self.block_exists(&root_id));
        // Update the next root
        self.root_id = root_id;
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
    pub(super) fn path_from_root(&self, block_id: HashValue) -> Option<Vec<Arc<ExecutedBlock<T>>>> {
        let mut res = vec![];
        let mut cur_block_id = block_id;
        loop {
            match self.get_block(&cur_block_id) {
                Some(ref block) if block.round() <= self.root().round() => {
                    break;
                }
                Some(block) => {
                    cur_block_id = block.parent_id();
                    res.push(block);
                }
                None => return None,
            }
        }
        // At this point cur_block.round() <= self.root.round()
        if cur_block_id != self.root_id {
            return None;
        }
        // Called `.reverse()` to get the chronically increased order.
        res.reverse();
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
    T: Serialize + Default + Debug + PartialEq,
{
    /// Returns the number of blocks in the tree
    pub(super) fn len(&self) -> usize {
        // BFS over the tree to find the number of blocks in the tree.
        let mut res = 0;
        let mut to_visit = Vec::new();
        to_visit.push(self.linkable_root());
        while let Some(block) = to_visit.pop() {
            res += 1;
            for child_id in block.children() {
                to_visit.push(
                    self.get_linkable_block(child_id)
                        .expect("Child must exist in the tree"),
                );
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
