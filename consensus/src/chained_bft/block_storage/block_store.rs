// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{block_tree::BlockTree, BlockReader, InsertError, VoteReceptionResult},
        common::{Payload, Round},
        consensus_types::{block::Block, quorum_cert::QuorumCert},
        persistent_storage::PersistentStorage,
        safety::vote_msg::VoteMsg,
    },
    state_replication::{ExecutedState, StateComputer},
};
use crypto::HashValue;
use logger::prelude::*;

use crate::{chained_bft::persistent_storage::RecoveryData, state_replication::StateComputeResult};
use crypto::{ed25519::*, hash::CryptoHash};
use mirai_annotations::checked_precondition;
use std::{
    collections::{vec_deque::VecDeque, HashMap},
    sync::{Arc, RwLock},
};
use types::{ledger_info::LedgerInfo, validator_signer::ValidatorSigner};

#[cfg(test)]
#[path = "block_store_test.rs"]
mod block_store_test;

#[derive(Debug, PartialEq)]
/// Whether we need to do block retrieval if we want to insert a Quorum Cert.
pub enum NeedFetchResult {
    QCRoundBeforeRoot,
    QCAlreadyExist,
    QCBlockExist,
    NeedFetch,
}

/// Responsible for maintaining all the blocks of payload and the dependencies of those blocks
/// (parent and previous QC links).  It is expected to be accessed concurrently by multiple threads
/// and is thread-safe.
///
/// Example tree block structure based on parent links.
///                         | -> A3
/// Genesis -> B0 -> B1 -> B2 -> B3
///             | -> C1 -> C2
///                         | -> D3
///
/// Example corresponding tree block structure for the QC links (must follow QC constraints).
///                         | -> A3
/// Genesis -> B0 -> B1 -> B2 -> B3
///             | -> C1
///             | -------> C2
///             | -------------> D3
pub struct BlockStore<T> {
    inner: Arc<RwLock<BlockTree<T>>>,
    validator_signer: ValidatorSigner<Ed25519PrivateKey>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    enforce_increasing_timestamps: bool,
    /// The persistent storage backing up the in-memory data structure, every write should go
    /// through this before in-memory tree.
    storage: Arc<dyn PersistentStorage<T>>,
}

impl<T: Payload> BlockStore<T> {
    pub async fn new(
        storage: Arc<dyn PersistentStorage<T>>,
        initial_data: RecoveryData<T>,
        validator_signer: ValidatorSigner<Ed25519PrivateKey>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        enforce_increasing_timestamps: bool,
        max_pruned_blocks_in_mem: usize,
    ) -> Self {
        let (root, blocks, quorum_certs) = initial_data.take();
        let inner = Arc::new(RwLock::new(
            Self::build_block_tree(
                root,
                blocks,
                quorum_certs,
                Arc::clone(&state_computer),
                max_pruned_blocks_in_mem,
            )
            .await,
        ));
        BlockStore {
            inner,
            validator_signer,
            state_computer,
            enforce_increasing_timestamps,
            storage,
        }
    }

    async fn build_block_tree(
        root: (Block<T>, QuorumCert, QuorumCert),
        blocks: Vec<Block<T>>,
        quorum_certs: Vec<QuorumCert>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        max_pruned_blocks_in_mem: usize,
    ) -> BlockTree<T> {
        let mut tree = BlockTree::new(root.0, root.1, root.2, max_pruned_blocks_in_mem);
        let quorum_certs = quorum_certs
            .into_iter()
            .map(|qc| (qc.certified_block_id(), qc))
            .collect::<HashMap<_, _>>();
        for block in blocks {
            let compute_res = state_computer
                .compute(block.parent_id(), block.id(), block.get_payload())
                .await
                .expect("fail to rebuild scratchpad");
            let version = tree
                .get_state_for_block(block.parent_id())
                .expect("parent state does not exist")
                .version
                + compute_res.num_successful_txns;
            let executed_state = ExecutedState {
                state_id: compute_res.new_state_id,
                version,
            };
            // if this block is certified, ensure we agree with the certified state.
            if let Some(qc) = quorum_certs.get(&block.id()) {
                assert_eq!(
                    qc.certified_state(),
                    executed_state,
                    "We have inconsistent executed state with Quorum Cert for block {}",
                    block.id()
                );
            }
            tree.insert_block(block, executed_state, compute_res)
                .expect("Block insertion failed while build the tree");
        }
        quorum_certs.into_iter().for_each(|(_, qc)| {
            tree.insert_quorum_cert(qc)
                .expect("QuorumCert insertion failed while build the tree")
        });
        tree
    }

    pub async fn rebuild(
        &self,
        root: (Block<T>, QuorumCert, QuorumCert),
        blocks: Vec<Block<T>>,
        quorum_certs: Vec<QuorumCert>,
    ) {
        let tree = Self::build_block_tree(
            root,
            blocks,
            quorum_certs,
            Arc::clone(&self.state_computer),
            self.inner.read().unwrap().max_pruned_blocks_in_mem(),
        )
        .await;
        let to_remove = self.inner.read().unwrap().get_all_block_id();
        if let Err(e) = self.storage.prune_tree(to_remove) {
            // it's fine to fail here, the next restart will try to clean up dangling blocks again.
            error!("fail to delete block: {:?}", e);
        }
        *self.inner.write().unwrap() = tree;
    }

    pub fn signer(&self) -> &ValidatorSigner<Ed25519PrivateKey> {
        &self.validator_signer
    }

    /// Execute and insert a block if it passes all validation tests.
    /// Returns the Arc to the block kept in the block store after persisting it to storage
    ///
    /// This function assumes that the ancestors are present (returns MissingParent otherwise).
    ///
    /// Duplicate inserts will return the previously inserted block (
    /// note that it is considered a valid non-error case, for example, it can happen if a validator
    /// receives a certificate for a block that is currently being added).
    pub async fn execute_and_insert_block(
        &self,
        block: Block<T>,
    ) -> Result<Arc<Block<T>>, InsertError> {
        if let Some(existing_block) = self.inner.read().unwrap().get_block(block.id()) {
            return Ok(existing_block);
        }
        let (parent_id, parent_exec_version) = match self.verify_and_get_parent_info(&block) {
            Ok(t) => t,
            Err(e) => {
                security_log(SecurityEvent::InvalidBlock)
                    .error(&e)
                    .data(&block)
                    .log();
                return Err(e);
            }
        };
        let compute_res = self
            .state_computer
            .compute(parent_id, block.id(), block.get_payload())
            .await
            .map_err(|e| {
                error!("Execution failure for block {}: {:?}", block, e);
                InsertError::StateComputerError
            })?;

        let version = parent_exec_version + compute_res.num_successful_txns;

        let state = ExecutedState {
            state_id: compute_res.new_state_id,
            version,
        };
        self.storage
            .save_tree(vec![block.clone()], vec![])
            .map_err(|_| InsertError::StorageFailure)?;
        self.inner
            .write()
            .unwrap()
            .insert_block(block, state, compute_res)
            .map_err(|e| e.into())
    }

    /// Check if we're far away from this ledger info and need to sync.
    /// Returns false if we have this block in the tree or the root's round is higher than the
    /// block.
    pub fn need_sync_for_quorum_cert(
        &self,
        committed_block_id: HashValue,
        qc: &QuorumCert,
    ) -> bool {
        // LedgerInfo doesn't carry the information about the round of the committed block. However,
        // the 3-chain safety rules specify that the round of the committed block must be
        // certified_block_round() - 2. In case root().round() is greater than that the committed
        // block carried by LI is older than my current commit.
        !(self.block_exists(committed_block_id)
            || self.root().round() + 2 >= qc.certified_block_round())
    }

    /// Checks if quorum certificate can be inserted in block store without RPC
    /// Returns the enum to indicate the detailed status.
    pub fn need_fetch_for_quorum_cert(&self, qc: &QuorumCert) -> NeedFetchResult {
        if qc.certified_block_round() < self.root().round() {
            return NeedFetchResult::QCRoundBeforeRoot;
        }
        if self
            .get_quorum_cert_for_block(qc.certified_block_id())
            .is_some()
        {
            return NeedFetchResult::QCAlreadyExist;
        }
        if self.block_exists(qc.certified_block_id()) {
            return NeedFetchResult::QCBlockExist;
        }
        NeedFetchResult::NeedFetch
    }

    /// Validates quorum certificates and inserts it into block tree assuming dependencies exist.
    pub fn insert_single_quorum_cert(&self, qc: QuorumCert) -> Result<(), InsertError> {
        // Ensure executed state is consistent with Quorum Cert, otherwise persist the quorum's
        // state and hopefully we restart and agree with it.
        let executed_state = self
            .get_state_for_block(qc.certified_block_id())
            .ok_or_else(|| InsertError::MissingParentBlock(qc.certified_block_id()))?;
        assert_eq!(
            executed_state,
            qc.certified_state(),
            "We have inconsistent executed state with the executed state from the quorum \
             certificate for block {}, will kill this validator and rely on state synchronization \
             to try to achieve consistent state with the quorum certificate.",
            qc.certified_block_id(),
        );
        self.storage
            .save_tree(vec![], vec![qc.clone()])
            .map_err(|_| InsertError::StorageFailure)?;
        self.inner
            .write()
            .unwrap()
            .insert_quorum_cert(qc)
            .map_err(|e| e.into())
    }

    /// Adds a vote for the block.
    /// The returned value either contains the vote result (with new / old QC etc.) or a
    /// verification error.
    /// A block store does not verify that the block, which is voted for, is present locally.
    /// It returns QC, if it is formed, but does not insert it into block store, because it might
    /// not have required dependencies yet
    /// Different execution ids are treated as different blocks (e.g., if some proposal is
    /// executed in a non-deterministic fashion due to a bug, then the votes for execution result
    /// A and the votes for execution result B are aggregated separately).
    pub fn insert_vote(&self, vote_msg: VoteMsg, min_votes_for_qc: usize) -> VoteReceptionResult {
        self.inner
            .write()
            .unwrap()
            .insert_vote(&vote_msg, min_votes_for_qc)
    }

    /// Prune the tree up to next_root_id (keep next_root_id's block).  Any branches not part of
    /// the next_root_id's tree should be removed as well.
    ///
    /// For example, root = B_0
    /// B_0 -> B_1 -> B_2
    ///         |  -> B_3 -> B4
    ///
    /// prune_tree(B_3) should be left with
    /// B_3 -> B_4, root = B_3
    ///
    /// Returns the block ids of the blocks removed.
    pub fn prune_tree(&self, next_root_id: HashValue) -> VecDeque<HashValue> {
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

    /// If block id information is found, returns the ledger info placeholder, otherwise, return
    /// a placeholder with info of the genesis block.
    pub fn ledger_info_placeholder(&self, id: Option<HashValue>) -> LedgerInfo {
        let block_id = match id {
            None => return Self::zero_ledger_info_placeholder(),
            Some(id) => id,
        };
        let block = match self.get_block(block_id) {
            Some(b) => b,
            None => {
                return Self::zero_ledger_info_placeholder();
            }
        };
        let (state_id, version) = match self.get_state_for_block(block_id) {
            Some(state) => (state.state_id, state.version),
            None => {
                return Self::zero_ledger_info_placeholder();
            }
        };
        LedgerInfo::new(
            version,
            state_id,
            HashValue::zero(),
            block_id,
            0, // TODO [Reconfiguration] use the real epoch number.
            block.timestamp_usecs(),
        )
    }

    /// Used in case we're using a ledger info just as a placeholder for signing the votes / QCs
    /// and there is no real block committed.
    /// It's all pretty much zeroes.
    fn zero_ledger_info_placeholder() -> LedgerInfo {
        LedgerInfo::new(
            0,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
        )
    }

    fn verify_and_get_parent_info(
        &self,
        block: &Block<T>,
    ) -> Result<(HashValue, u64), InsertError> {
        if block.round() <= self.inner.read().unwrap().root().round() {
            return Err(InsertError::OldBlock);
        }

        let block_hash = block.hash();
        if block.id() != block_hash {
            return Err(InsertError::InvalidBlockHash);
        }

        if block.quorum_cert().certified_block_id() != block.parent_id() {
            return Err(InsertError::ParentNotCertified);
        }

        let parent = match self.inner.read().unwrap().get_block(block.parent_id()) {
            None => {
                return Err(InsertError::MissingParentBlock(block.parent_id()));
            }
            Some(parent) => parent,
        };
        if parent.height() + 1 != block.height() {
            return Err(InsertError::InvalidBlockHeight);
        }
        if parent.round() >= block.round() {
            return Err(InsertError::InvalidBlockRound);
        }
        if self.enforce_increasing_timestamps && block.timestamp_usecs() <= parent.timestamp_usecs()
        {
            return Err(InsertError::NonIncreasingTimestamp);
        }
        let parent_id = parent.id();
        match self.inner.read().unwrap().get_state_for_block(parent_id) {
            Some(ExecutedState { version, .. }) => Ok((parent.id(), version)),
            None => Err(InsertError::ParentVersionNotFound),
        }
    }
}

impl<T: Payload> BlockReader for BlockStore<T> {
    type Payload = T;

    fn block_exists(&self, block_id: HashValue) -> bool {
        self.inner.read().unwrap().block_exists(block_id)
    }

    fn get_block(&self, block_id: HashValue) -> Option<Arc<Block<Self::Payload>>> {
        self.inner.read().unwrap().get_block(block_id)
    }

    fn get_state_for_block(&self, block_id: HashValue) -> Option<ExecutedState> {
        self.inner.read().unwrap().get_state_for_block(block_id)
    }

    fn get_compute_result(&self, block_id: HashValue) -> Option<Arc<StateComputeResult>> {
        self.inner.read().unwrap().get_compute_result(block_id)
    }

    fn root(&self) -> Arc<Block<Self::Payload>> {
        self.inner.read().unwrap().root()
    }

    fn get_quorum_cert_for_block(&self, block_id: HashValue) -> Option<Arc<QuorumCert>> {
        self.inner
            .read()
            .unwrap()
            .get_quorum_cert_for_block(block_id)
    }

    fn path_from_root(&self, block: Arc<Block<T>>) -> Option<Vec<Arc<Block<T>>>> {
        self.inner.read().unwrap().path_from_root(block)
    }

    fn create_block(
        &self,
        parent: Arc<Block<Self::Payload>>,
        payload: Self::Payload,
        round: Round,
        timestamp_usecs: u64,
    ) -> Block<Self::Payload> {
        if self.enforce_increasing_timestamps {
            checked_precondition!(parent.timestamp_usecs() < timestamp_usecs);
        }
        let quorum_cert = self
            .get_quorum_cert_for_block(parent.id())
            .expect("Parent for the newly created block is not certified!")
            .as_ref()
            .clone();
        Block::make_block(
            parent.as_ref(),
            payload,
            round,
            timestamp_usecs,
            quorum_cert,
            &self.validator_signer,
        )
    }

    fn highest_certified_block(&self) -> Arc<Block<Self::Payload>> {
        self.inner.read().unwrap().highest_certified_block()
    }

    fn highest_quorum_cert(&self) -> Arc<QuorumCert> {
        self.inner.read().unwrap().highest_quorum_cert()
    }

    fn highest_ledger_info(&self) -> Arc<QuorumCert> {
        self.inner.read().unwrap().highest_ledger_info()
    }
}

#[cfg(test)]
impl<T: Payload> BlockStore<T> {
    /// Returns the number of blocks in the tree
    fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// Returns the number of child links in the tree
    fn child_links(&self) -> usize {
        self.inner.read().unwrap().child_links()
    }

    /// The number of pruned blocks that are still available in memory
    pub(super) fn pruned_blocks_in_mem(&self) -> usize {
        self.inner.read().unwrap().pruned_blocks_in_mem()
    }

    /// Helper to insert vote and qc
    /// Can't be used in production, because production insertion potentially requires state sync
    pub fn insert_vote_and_qc(&self, vote_msg: VoteMsg, qc_size: usize) -> VoteReceptionResult {
        let r = self.insert_vote(vote_msg, qc_size);
        if let VoteReceptionResult::NewQuorumCertificate(ref qc) = r {
            self.insert_single_quorum_cert(qc.as_ref().clone()).unwrap();
        }
        r
    }

    /// Helper function to insert the block with the qc together
    pub async fn insert_block_with_qc(
        &self,
        block: Block<T>,
    ) -> Result<Arc<Block<T>>, InsertError> {
        self.insert_single_quorum_cert(block.quorum_cert().clone())?;
        Ok(self.execute_and_insert_block(block).await?)
    }
}
