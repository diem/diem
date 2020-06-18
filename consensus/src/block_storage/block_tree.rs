// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::account_address::AccountAddress;
use std::cmp::max;
use libra_types::block_info::Round;
use crate::counters;
use anyhow::bail;
use consensus_types::{
    executed_block::ExecutedBlock, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate,
};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use mirai_annotations::{checked_verify_eq, precondition};
use serde::Serialize;
use std::{
    collections::{vec_deque::VecDeque, HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
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
    /// Map of block id to its completed quorum certificate (2f + 1 votes)
    id_to_quorum_cert: HashMap<HashValue, Arc<QuorumCert>>,
    /// To keep the IDs of the elements that have been pruned from the tree but not cleaned up yet.
    pruned_block_ids: VecDeque<HashValue>,
    /// Num pruned blocks to keep in memory.
    max_pruned_blocks_in_mem: usize,
    
    /// Blocks that the validator voted for. Only keep the last one for each fork since it is enough to compute the marker
    voted_blocks: Vec<HashValue>,
    // The set of endorsers of a block
    id_to_endorsers: HashMap<HashValue, HashSet<AccountAddress>>,
    // genesis id, used for computing strong commit
    genesis_id: HashValue,
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

        let voted_blocks = Vec::new();
        let id_to_endorsers = HashMap::new();
        let genesis_id = root_id.clone();

        BlockTree {
            id_to_block,
            root_id,
            highest_certified_block_id: root_id,
            highest_quorum_cert: Arc::clone(&root_quorum_cert),
            highest_timeout_cert,
            highest_commit_cert: Arc::new(root_ledger_info),
            id_to_quorum_cert,
            pruned_block_ids,
            max_pruned_blocks_in_mem,
            voted_blocks,
            id_to_endorsers,
            genesis_id,
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

    // Daniel: in order to compute strong commit, for now we don't remove any block or QC from the storage, even the blocks are committed
    fn remove_block(&mut self, block_id: HashValue) {
        // Remove the block from the store
        // self.id_to_block.remove(&block_id);
        // self.id_to_quorum_cert.remove(&block_id);
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

    pub(super) fn genesis(&self) -> Arc<ExecutedBlock<T>> {
        self.get_block(&self.genesis_id).expect("Genesis must exist")
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

    pub(super) fn get_endorsers_for_block(&self, block_id: &HashValue) -> Option<HashSet<AccountAddress>> {
        self.id_to_endorsers.get(block_id).cloned()
    }

    pub(super) fn add_endorser(&mut self, block_id: HashValue, account: AccountAddress) {
        let endorser = self.id_to_endorsers
            .entry(block_id)
            .or_insert(HashSet::new());
        endorser.insert(account);
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

    pub(super) fn insert_quorum_cert(&mut self, quorumcert: QuorumCert) -> anyhow::Result<()> {
        let block_id = quorumcert.certified_block().id();
        let qc = Arc::new(quorumcert.clone());

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

        // update the endorsers of all previous blocks.
        self.update_endorsers(quorumcert)

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

    // When validator voted for a block, update the voted_block 
    pub(super) fn record_voted_block(&mut self, block_id: HashValue) -> anyhow::Result<()> {
        let mut cur_block_id = block_id;
        loop {
            // info!("daniel record voted block id {:?}", cur_block_id);
            match self.get_block(&cur_block_id) {
                Some(ref block) if block.round() <= self.root().round() => {
                    break;
                }
                Some(block) => {
                    cur_block_id = block.parent_id();
                    let mut done = false;
                    // remove the block from voted_block
                    for ind in 0..self.voted_blocks.len() {
                        if self.voted_blocks[ind] == cur_block_id {
                            self.voted_blocks.remove(ind);
                            done = true;
                            break;
                        }
                    }
                    if done { break; }
                }
                None => {
                    info!("record_voted_block: Block {:?} not found", cur_block_id);
                    bail!("Block {} not found", cur_block_id);
                }
            }
        }
        self.voted_blocks.push(block_id);

        Ok(())
    }

    pub(super) fn compute_marker(&self, block_id: HashValue) -> anyhow::Result<Round> {
        let mut marker = 0;
        for voted_block in &self.voted_blocks {
            // info!("daniel compute marker block id {:?}", voted_block);
            let result = self.conflict(*voted_block, block_id);
            match result {
                Ok(res) => {
                    if res {
                        if let Some(existing_block) = self.get_block(&voted_block) {
                            let round_number = existing_block.block_info().round();
                            marker = max(marker, round_number);
                        } else {
                            info!("compute_marker: Block {:?} not found", voted_block);
                            bail!("Block {} not found", voted_block);
                        }
                    }
                }
                Err(e) => {
                    error!("Error {:?}", e);
                }
            }
        }
        Ok(marker)
    }

    // check whether voted_block and voting_block are from different forks, voting_block always has larger round number due to the voting rule
    pub fn conflict(&self, voted_block: HashValue, voting_block: HashValue) -> anyhow::Result<bool> {
        let mut cur_block_id = voting_block;
        let mut result = true;
        loop {
            // info!("daniel record voted block id {:?}", cur_block_id);
            match self.get_block(&cur_block_id) {
                Some(ref block) if block.round() <= self.root().round() => {
                    break;
                }
                Some(block) => {
                    cur_block_id = block.parent_id();
                    // info!("conflict: round {}", block.round());
                    if cur_block_id == voted_block {
                        result = false;
                        break;
                    }
                }
                None => {
                    info!("conflict: Block {:?} not found", cur_block_id);
                    bail!("Block {} not found", cur_block_id);
                }
            }
        }
        return Ok(result);
    }

    // print voted blocks
    pub fn print_voted(&self) {
        for ind in 0..self.voted_blocks.len() {
            let mut res = vec![];
            let mut cur_block_id = self.voted_blocks[ind];
            loop {
                match self.get_block(&cur_block_id) {
                    Some(ref block) if block.round() <= self.root().round() => {
                        res.push(cur_block_id);
                        break;
                    }
                    Some(block) => {
                        res.push(cur_block_id);
                        cur_block_id = block.parent_id();
                    }
                    None => break,
                }
            }
            res.reverse();
            for block_id in res {
                if let Some(existing_block) = self.get_block(&block_id) {
                    let round_number = existing_block.block_info().round();
                    info!("daniel block round {}, id {:?}", round_number, block_id);
                } else {
                    warn!("Block {} not found", block_id);
                }
            }
            info!("****************************************");
        }
    }

    // Returns the blocks from the genesis to the current block id
    // include the current block but exclude the genesis
    // used to compute strong commit
    pub(super) fn path_from_genesis(&self, block_id: HashValue) -> Option<Vec<Arc<ExecutedBlock<T>>>> {
        let mut res = vec![];
        let mut cur_block_id = block_id;
        loop {
            match self.get_block(&cur_block_id) {
                Some(ref block) if block.round() <= self.genesis().round() => {
                    break;
                }
                Some(block) => {
                    cur_block_id = block.parent_id();
                    res.push(block);
                }
                None => return None,
            }
        }
        if cur_block_id != self.genesis_id {
            return None;
        }
        // Called `.reverse()` to get the chronically increased order.
        res.reverse();
        Some(res)
    }

    pub(super) fn update_endorsers(&mut self, qc: QuorumCert) -> anyhow::Result<()> {
        // if qc.vote_data().proposed().round()==0 {
        //     return Ok(());
        // }
        let signatures = qc.ledger_info().signatures();
        let markers = qc.ledger_info().markers();
        let blocks_from_genesis_to_highest_certified = self
            .path_from_genesis(self.highest_certified_block_id)
            .unwrap_or_else(Vec::new);
        // info!("daniel signatures {:?}, markers {:?}", signatures, markers);
        for (account_address, _) in signatures {
            let marker = markers.get(account_address).unwrap();
            for block in &blocks_from_genesis_to_highest_certified {
                // vote endorse a block if marker<block.round
                // info!("account {:?}, marker {:?}, round {:?}", account_address, *marker, block.round());
                if block.round() > *marker {
                    self.add_endorser(block.id(), *account_address);
                }
            }
        }
        self.print_endorsers();

        Ok(())
    }

    // print the round numbers and number of endorsers of blocks from genesis to current highest certified
    pub fn print_endorsers(&self) {
        info!("-------------------------print endorsers start-------------------------");
        let blocks_from_genesis_to_highest_certified = self
            .path_from_genesis(self.highest_certified_block_id)
            .unwrap_or_else(Vec::new);
        for block in blocks_from_genesis_to_highest_certified {
            let id = block.id();
            let round = block.round();
            let endorsers = self.get_endorsers_for_block(&id).unwrap();
            info!("block round {}, number of endorsers {:?}", round, endorsers.len());
        }
        info!("-------------------------print endorsers end-------------------------");
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
