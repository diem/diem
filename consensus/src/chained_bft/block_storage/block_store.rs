// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{block_tree::BlockTree, BlockReader},
        persistent_liveness_storage::{
            PersistentLivenessStorage, RecoveryData, RootInfo, RootMetadata,
        },
    },
    counters,
    state_replication::StateComputer,
};
use anyhow::{bail, ensure, format_err, Context};
use consensus_types::{
    block::Block, common::Payload, executed_block::ExecutedBlock, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate,
};
use debug_interface::prelude::*;
use executor_types::StateComputeResult;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::ledger_info::LedgerInfoWithSignatures;
#[cfg(any(test, feature = "fuzzing"))]
use libra_types::validator_set::ValidatorSet;
use std::{
    collections::{vec_deque::VecDeque, HashMap},
    sync::{Arc, RwLock},
};
use termion::color::*;

#[cfg(test)]
#[path = "block_store_test.rs"]
mod block_store_test;

#[path = "sync_manager.rs"]
pub mod sync_manager;

/// Responsible for maintaining all the blocks of payload and the dependencies of those blocks
/// (parent and previous QC links).  It is expected to be accessed concurrently by multiple threads
/// and is thread-safe.
///
/// Example tree block structure based on parent links.
///                         ╭--> A3
/// Genesis--> B0--> B1--> B2--> B3
///             ╰--> C1--> C2
///                         ╰--> D3
///
/// Example corresponding tree block structure for the QC links (must follow QC constraints).
///                         ╭--> A3
/// Genesis--> B0--> B1--> B2--> B3
///             ├--> C1
///             ├--------> C2
///             ╰--------------> D3
pub struct BlockStore<T> {
    inner: Arc<RwLock<BlockTree<T>>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    /// The persistent storage backing up the in-memory data structure, every write should go
    /// through this before in-memory tree.
    storage: Arc<dyn PersistentLivenessStorage<T>>,
}

impl<T: Payload> BlockStore<T> {
    pub fn new(
        storage: Arc<dyn PersistentLivenessStorage<T>>,
        initial_data: RecoveryData<T>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        max_pruned_blocks_in_mem: usize,
    ) -> Self {
        let highest_tc = initial_data.highest_timeout_certificate();
        let (root, root_metadata, blocks, quorum_certs) = initial_data.take();
        let inner = Arc::new(RwLock::new(Self::build_block_tree(
            root,
            root_metadata,
            blocks,
            quorum_certs,
            highest_tc,
            Arc::clone(&state_computer),
            max_pruned_blocks_in_mem,
        )));
        BlockStore {
            inner,
            state_computer,
            storage,
        }
    }

    fn build_block_tree(
        root: RootInfo<T>,
        root_metadata: RootMetadata,
        blocks: Vec<Block<T>>,
        quorum_certs: Vec<QuorumCert>,
        highest_timeout_cert: Option<TimeoutCertificate>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        max_pruned_blocks_in_mem: usize,
    ) -> BlockTree<T> {
        let RootInfo(root_block, root_qc, root_li) = root;
        //verify root is correct
        assert_eq!(
            root_qc.certified_block().version(),
            root_metadata.version(),
            "root qc version {} doesn't match committed trees {}",
            root_qc.certified_block().version(),
            root_metadata.version(),
        );
        assert_eq!(
            root_qc.certified_block().executed_state_id(),
            root_metadata.accu_hash,
            "root qc state id {} doesn't match committed trees {}",
            root_qc.certified_block().executed_state_id(),
            root_metadata.accu_hash,
        );

        let executed_root_block = ExecutedBlock::new(
            root_block,
            // Create a dummy state_compute_result with necessary fields filled in.
            StateComputeResult::new(
                root_metadata.accu_hash,
                root_metadata.frozen_root_hashes,
                root_metadata.num_leaves, /* num_leaves */
                None,                     /* validators */
                vec![],                   /* compute_status */
                vec![],                   /* transaction_info_hashes */
            ),
        );
        let mut tree = BlockTree::new(
            executed_root_block,
            root_qc,
            root_li,
            max_pruned_blocks_in_mem,
            highest_timeout_cert.map(Arc::new),
        );
        let quorum_certs = quorum_certs
            .into_iter()
            .map(|qc| (qc.certified_block().id(), qc))
            .collect::<HashMap<_, _>>();

        for block in blocks {
            assert!(!block.is_genesis_block());
            let output = state_computer
                .compute(&block, block.parent_id())
                .unwrap_or_else(|_| {
                    panic!("fail to compute state result for block {}", block.id())
                });
            // if this block is certified, ensure we agree with the certified state.
            if let Some(qc) = quorum_certs.get(&block.id()) {
                assert_eq!(
                    qc.certified_block().executed_state_id(),
                    output.root_hash(),
                    "We have inconsistent executed state with Quorum Cert for block {}",
                    block.id()
                );
            }
            tree.insert_block(ExecutedBlock::new(block, output))
                .expect("Block insertion failed while build the tree");
        }

        quorum_certs.into_iter().for_each(|(_, qc)| {
            tree.insert_quorum_cert(qc)
                .expect("QuorumCert insertion failed while build the tree")
        });

        tree
    }

    /// Commit the given block id with the proof, returns the path from current root or error
    pub fn commit(
        &self,
        finality_proof: LedgerInfoWithSignatures,
    ) -> anyhow::Result<Vec<Arc<ExecutedBlock<T>>>> {
        let block_id_to_commit = finality_proof.ledger_info().consensus_block_id();
        let block_to_commit = self
            .get_block(block_id_to_commit)
            .ok_or_else(|| format_err!("Committed block id not found"))?;

        // First make sure that this commit is new.
        ensure!(
            block_to_commit.round() > self.root().round(),
            "Committed block round lower than root"
        );

        let blocks_to_commit = self
            .path_from_root(block_id_to_commit)
            .unwrap_or_else(Vec::new);

        self.state_computer
            .commit(
                blocks_to_commit.iter().map(|b| b.id()).collect(),
                finality_proof,
            )
            .expect("Failed to persist commit");
        counters::LAST_COMMITTED_ROUND.set(block_to_commit.round() as i64);
        debug!("{}Committed{} {}", Fg(Blue), Fg(Reset), *block_to_commit);
        event!("committed",
            "block_id": block_to_commit.id().short_str(),
            "round": block_to_commit.round(),
            "parent_id": block_to_commit.parent_id().short_str(),
        );
        self.prune_tree(block_to_commit.id());
        Ok(blocks_to_commit)
    }

    pub fn rebuild(
        &self,
        root: RootInfo<T>,
        root_metadata: RootMetadata,
        blocks: Vec<Block<T>>,
        quorum_certs: Vec<QuorumCert>,
    ) {
        let max_pruned_blocks_in_mem = self.inner.read().unwrap().max_pruned_blocks_in_mem();
        // Rollover the previous highest TC from the old tree to the new one.
        let prev_htc = self.highest_timeout_cert().map(|tc| tc.as_ref().clone());
        let tree = Self::build_block_tree(
            root,
            root_metadata,
            blocks,
            quorum_certs,
            prev_htc,
            Arc::clone(&self.state_computer),
            max_pruned_blocks_in_mem,
        );
        let to_remove = self.inner.read().unwrap().get_all_block_id();
        if let Err(e) = self.storage.prune_tree(to_remove) {
            // it's fine to fail here, the next restart will try to clean up dangling blocks again.
            error!("fail to delete block: {:?}", e);
        }
        *self.inner.write().unwrap() = tree;
        // If we fail to commit B_i via state computer and crash, after restart our highest commit cert
        // will not match the latest commit B_j(j<i) of state computer.
        // This introduces an inconsistent state if we send out SyncInfo and others try to sync to
        // B_i and figure out we only have B_j.
        // Here we commit up to the highest_commit_cert to maintain highest_commit_cert == state_computer.committed_trees.
        if self.highest_commit_cert().commit_info().round() > self.root().round() {
            let finality_proof = self.highest_commit_cert().ledger_info().clone();
            if let Err(e) = self.commit(finality_proof) {
                warn!("{:?}", e);
            }
        }
    }

    /// Execute and insert a block if it passes all validation tests.
    /// Returns the Arc to the block kept in the block store after persisting it to storage
    ///
    /// This function assumes that the ancestors are present (returns MissingParent otherwise).
    ///
    /// Duplicate inserts will return the previously inserted block (
    /// note that it is considered a valid non-error case, for example, it can happen if a validator
    /// receives a certificate for a block that is currently being added).
    pub fn execute_and_insert_block(
        &self,
        block: Block<T>,
    ) -> anyhow::Result<Arc<ExecutedBlock<T>>> {
        if let Some(existing_block) = self.get_block(block.id()) {
            return Ok(existing_block);
        }
        let executed_block = self.execute_block(block)?;
        self.storage
            .save_tree(vec![executed_block.block().clone()], vec![])
            .context("Insert block failed when saving block")?;
        self.inner.write().unwrap().insert_block(executed_block)
    }

    fn execute_block(&self, block: Block<T>) -> anyhow::Result<ExecutedBlock<T>> {
        trace_code_block!("block_store::execute_block", {"block", block.id()});
        ensure!(
            self.inner.read().unwrap().root().round() < block.round(),
            "Block with old round"
        );

        let parent_block = self
            .get_block(block.parent_id())
            .ok_or_else(|| format_err!("Block with missing parent {}", block.parent_id()))?;

        // Reconfiguration rule - if a block is a child of pending reconfiguration, it needs to be empty
        // So we roll over the executed state until it's committed and we start new epoch.
        let state_compute_result = if parent_block.compute_result().has_reconfiguration() {
            StateComputeResult::new(
                parent_block.compute_result().root_hash(),
                parent_block.compute_result().frozen_subtree_roots().clone(),
                parent_block.compute_result().num_leaves(),
                parent_block.compute_result().validators().clone(),
                vec![], /* compute_status */
                vec![], /* transaction_info_hashes */
            )
        } else {
            // Although NIL blocks don't have payload, we still send a T::default() to compute
            // because we may inject a block prologue transaction.
            self.state_computer
                .compute(&block, parent_block.id())
                .with_context(|| format!("Execution failure for block {}", block))?
        };
        Ok(ExecutedBlock::new(block, state_compute_result))
    }

    /// Validates quorum certificates and inserts it into block tree assuming dependencies exist.
    pub fn insert_single_quorum_cert(&self, qc: QuorumCert) -> anyhow::Result<()> {
        // If the parent block is not the root block (i.e not None), ensure the executed state
        // of a block is consistent with its QuorumCert, otherwise persist the QuorumCert's
        // state and on restart, a new execution will agree with it.  A new execution will match
        // the QuorumCert's state on the next restart will work if there is a memory
        // corruption, for example.
        match self.get_block(qc.certified_block().id()) {
            Some(executed_block) => {
                ensure!(
                    executed_block.block_info() == *qc.certified_block(),
                    "QC for block {} has different {:?} than local {:?}",
                    qc.certified_block().id(),
                    qc.certified_block(),
                    executed_block.block_info()
                );
            }
            None => bail!("Insert {} without having the block in store first", qc),
        }

        self.storage
            .save_tree(vec![], vec![qc.clone()])
            .context("Insert block failed when saving quorum")?;
        self.inner.write().unwrap().insert_quorum_cert(qc)
    }

    /// Replace the highest timeout certificate in case the given one has a higher round.
    /// In case a timeout certificate is updated, persist it to storage.
    pub fn insert_timeout_certificate(&self, tc: Arc<TimeoutCertificate>) -> anyhow::Result<()> {
        let cur_tc_round = self.highest_timeout_cert().map_or(0, |tc| tc.round());
        if tc.round() <= cur_tc_round {
            return Ok(());
        }
        self.storage
            .save_highest_timeout_cert(tc.as_ref().clone())
            .context("Timeout certificate insert failed when persisting to DB")?;
        self.inner.write().unwrap().replace_timeout_cert(tc);
        Ok(())
    }

    /// Prune the tree up to next_root_id (keep next_root_id's block).  Any branches not part of
    /// the next_root_id's tree should be removed as well.
    ///
    /// For example, root = B0
    /// B0--> B1--> B2
    ///        ╰--> B3--> B4
    ///
    /// prune_tree(B3) should be left with
    /// B3--> B4, root = B3
    ///
    /// Returns the block ids of the blocks removed.
    fn prune_tree(&self, next_root_id: HashValue) -> VecDeque<HashValue> {
        let id_to_remove = self
            .inner
            .read()
            .unwrap()
            .find_blocks_to_prune(next_root_id);
        if let Err(e) = self
            .storage
            .prune_tree(id_to_remove.clone().into_iter().collect())
        {
            // it's fine to fail here, as long as the commit succeeds, the next restart will clean
            // up dangling blocks, and we need to prune the tree to keep the root consistent with
            // executor.
            error!("fail to delete block: {:?}", e);
        }
        self.inner
            .write()
            .unwrap()
            .process_pruned_blocks(next_root_id, id_to_remove.clone());
        id_to_remove
    }
}

impl<T: Payload> BlockReader for BlockStore<T> {
    type Payload = T;

    fn block_exists(&self, block_id: HashValue) -> bool {
        self.inner.read().unwrap().block_exists(&block_id)
    }

    fn get_block(&self, block_id: HashValue) -> Option<Arc<ExecutedBlock<T>>> {
        self.inner.read().unwrap().get_block(&block_id)
    }

    fn root(&self) -> Arc<ExecutedBlock<T>> {
        self.inner.read().unwrap().root()
    }

    fn get_quorum_cert_for_block(&self, block_id: HashValue) -> Option<Arc<QuorumCert>> {
        self.inner
            .read()
            .unwrap()
            .get_quorum_cert_for_block(&block_id)
    }

    fn path_from_root(&self, block_id: HashValue) -> Option<Vec<Arc<ExecutedBlock<T>>>> {
        self.inner.read().unwrap().path_from_root(block_id)
    }

    fn highest_certified_block(&self) -> Arc<ExecutedBlock<Self::Payload>> {
        self.inner.read().unwrap().highest_certified_block()
    }

    fn highest_quorum_cert(&self) -> Arc<QuorumCert> {
        self.inner.read().unwrap().highest_quorum_cert()
    }

    fn highest_commit_cert(&self) -> Arc<QuorumCert> {
        self.inner.read().unwrap().highest_commit_cert()
    }

    fn highest_timeout_cert(&self) -> Option<Arc<TimeoutCertificate>> {
        self.inner.read().unwrap().highest_timeout_cert()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl<T: Payload> BlockStore<T> {
    /// Returns the number of blocks in the tree
    pub(crate) fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// Returns the number of child links in the tree
    pub(crate) fn child_links(&self) -> usize {
        self.inner.read().unwrap().child_links()
    }

    /// The number of pruned blocks that are still available in memory
    pub(super) fn pruned_blocks_in_mem(&self) -> usize {
        self.inner.read().unwrap().pruned_blocks_in_mem()
    }

    /// Helper function to insert the block with the qc together
    pub fn insert_block_with_qc(&self, block: Block<T>) -> anyhow::Result<Arc<ExecutedBlock<T>>> {
        self.insert_single_quorum_cert(block.quorum_cert().clone())?;
        Ok(self.execute_and_insert_block(block)?)
    }

    /// Helper function to insert a reconfiguration block
    pub fn insert_reconfiguration_block(
        &self,
        block: Block<T>,
    ) -> anyhow::Result<Arc<ExecutedBlock<T>>> {
        self.insert_single_quorum_cert(block.quorum_cert().clone())?;
        let executed_block = self.execute_block(block)?;
        let compute_result = executed_block.compute_result();
        Ok(self
            .inner
            .write()
            .unwrap()
            .insert_block(ExecutedBlock::new(
                executed_block.block().clone(),
                StateComputeResult::new(
                    compute_result.root_hash(),
                    compute_result.frozen_subtree_roots().clone(),
                    compute_result.num_leaves(),
                    Some(ValidatorSet::new(vec![])),
                    compute_result.compute_status().clone(),
                    compute_result.transaction_info_hashes().clone(),
                ),
            ))?)
    }
}
