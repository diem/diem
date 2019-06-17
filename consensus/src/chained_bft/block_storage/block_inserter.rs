// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{block_tree::BlockTree, InsertError},
        common::Payload,
        consensus_types::block::Block,
        persistent_storage::PersistentStorage,
    },
    state_replication::{ExecutedState, StateComputer},
};
use crypto::{hash::CryptoHash, HashValue};
use futures::compat::Future01CompatExt;
use logger::prelude::*;
use std::sync::{Arc, RwLock};

/// The max number of the parallel executions a BlockInserterGuard allows.
const MAX_PARALLEL_EXECUTIONS: u8 = 32;

/// BlockInserterGuard is using futures_locks asynchronous locks in order to synchronize
/// async insertions from different concurrent tasks.
/// It keeps multiple instances of rwlocks in order to reduce unnecessary serialization of
/// independent blocks: the guard is determined by a given block id.
pub struct BlockInserterGuard<T> {
    // The guards is a vector of async locks to the very same instance of inserter.
    // The guard is chosen as a function of a block id: same blocks operations must be serialized.
    guards: Vec<futures_locks::RwLock<Arc<BlockInserter<T>>>>,
}

impl<T: Payload> BlockInserterGuard<T> {
    pub fn new(
        inner: Arc<RwLock<BlockTree<T>>>,
        state_computer: Arc<StateComputer<Payload = T>>,
        enforce_increasing_timestamps: bool,
        storage: Arc<dyn PersistentStorage<T>>,
    ) -> Self {
        let inserter_ref = Arc::new(BlockInserter::new(
            inner,
            state_computer,
            enforce_increasing_timestamps,
            storage,
        ));
        let mut guards = vec![];
        for _ in 0..MAX_PARALLEL_EXECUTIONS {
            guards.push(futures_locks::RwLock::new(Arc::clone(&inserter_ref)));
        }

        Self { guards }
    }

    /// Execute and insert a block if it passes all validation tests.
    /// Returns the Arc to the block kept in the block tree.
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
        // Choose a guard deterministically as a function of a block id: different requests for the
        // same block must be serialized.
        let guard_idx = (*block.id().to_vec().last().unwrap() % MAX_PARALLEL_EXECUTIONS) as usize;
        let inserter = self
            .guards
            .get(guard_idx)
            .unwrap()
            .write()
            .compat()
            .await
            .unwrap();
        inserter.execute_and_insert_block(block).await
    }
}

struct BlockInserter<T> {
    inner: Arc<RwLock<BlockTree<T>>>,
    state_computer: Arc<StateComputer<Payload = T>>,
    enforce_increasing_timestamps: bool,
    /// The persistent storage backing up the in-memory data structure, every write should go
    /// through this before in-memory tree.
    storage: Arc<dyn PersistentStorage<T>>,
}

impl<T: Payload> BlockInserter<T> {
    fn new(
        inner: Arc<RwLock<BlockTree<T>>>,
        state_computer: Arc<StateComputer<Payload = T>>,
        enforce_increasing_timestamps: bool,
        storage: Arc<dyn PersistentStorage<T>>,
    ) -> Self {
        Self {
            inner,
            state_computer,
            enforce_increasing_timestamps,
            storage,
        }
    }

    async fn execute_and_insert_block(
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

    /// All the verifications of a block that is going to be added to the tree.
    /// We assume that all the ancestors are present, returns MissingParent error otherwise.
    /// Returns parent id and version in case of success.
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
        if self.enforce_increasing_timestamps && parent.timestamp_usecs() >= block.timestamp_usecs()
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
