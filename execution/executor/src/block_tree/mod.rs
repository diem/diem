// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! In a leader based consensus algorithm, each participant maintains a block tree that looks like
//! the following:
//! ```text
//!  Height      5      6      7      ...
//!
//! Committed -> B5  -> B6  -> B7
//!         |
//!         └--> B5' -> B6' -> B7'
//!                     |
//!                     └----> B7"
//! ```
//! This module implements `BlockTree` that is an in-memory representation of this tree.

#[cfg(test)]
mod block_tree_test;

use failure::bail_err;
use libra_crypto::HashValue;
use std::collections::{hash_map, HashMap, HashSet};

/// Each block has a unique identifier that is a `HashValue` computed by consensus. It has exactly
/// one parent and zero or more children.
pub trait Block: std::fmt::Debug {
    /// The output of executing this block.
    type Output;

    /// The signatures on this block.
    type Signature;

    /// Whether consensus has decided to commit this block. This kind of blocks are expected to be
    /// sent to storage very soon, unless execution is lagging behind.
    fn is_committed(&self) -> bool;

    /// Marks this block as committed.
    fn set_committed(&mut self);

    /// Whether this block has finished execution.
    fn is_executed(&self) -> bool;

    /// Sets the output of this block.
    fn set_output(&mut self, output: Self::Output);

    /// Sets the signatures for this block.
    fn set_signature(&mut self, signature: Self::Signature);

    /// The id of this block.
    fn id(&self) -> HashValue;

    /// The id of the parent block.
    fn parent_id(&self) -> HashValue;

    /// Adds a block as its child.
    fn add_child(&mut self, child_id: HashValue);

    /// The list of children of this block.
    fn children(&self) -> &HashSet<HashValue>;
}

/// The `BlockTree` implementation.
#[derive(Debug)]
pub struct BlockTree<B> {
    /// A map that keeps track of all existing blocks by their ids.
    id_to_block: HashMap<HashValue, B>,

    /// The blocks at the lowest height in the map. B5 and B5' in the following example.
    /// ```text
    /// Committed(B0..4) -> B5  -> B6  -> B7
    ///                |
    ///                └--> B5' -> B6' -> B7'
    ///                            |
    ///                            └----> B7"
    /// ```
    heads: HashSet<HashValue>,

    /// Id of the last committed block. B4 in the above example.
    last_committed_id: HashValue,
}

impl<B> BlockTree<B>
where
    B: Block,
{
    /// Constructs a new `BlockTree`.
    pub fn new(last_committed_id: HashValue) -> Self {
        BlockTree {
            id_to_block: HashMap::new(),
            heads: HashSet::new(),
            last_committed_id,
        }
    }

    /// Adds a new block to the tree.
    pub fn add_block(&mut self, block: B) -> Result<(), AddBlockError<B>> {
        assert!(!self.id_to_block.contains_key(&self.last_committed_id));

        let id = block.id();
        if self.id_to_block.contains_key(&id) {
            bail_err!(AddBlockError::BlockAlreadyExists { block });
        }

        let parent_id = block.parent_id();
        if parent_id == self.last_committed_id {
            assert!(self.heads.insert(id), "Block already existed in heads.");
            self.id_to_block.insert(id, block);
            return Ok(());
        }

        match self.id_to_block.entry(parent_id) {
            hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().add_child(id);
                assert!(
                    self.id_to_block.insert(id, block).is_none(),
                    "Block {:x} already existed.",
                    id,
                );
            }
            hash_map::Entry::Vacant(_) => bail_err!(AddBlockError::ParentNotFound { block }),
        }

        Ok(())
    }

    /// Returns a reference to a specific block, if it exists in the tree.
    pub fn get_block(&self, id: HashValue) -> Option<&B> {
        self.id_to_block.get(&id)
    }

    /// Returns a mutable reference to a specific block, if it exists in the tree.
    pub fn get_block_mut(&mut self, id: HashValue) -> Option<&mut B> {
        self.id_to_block.get_mut(&id)
    }

    /// Returns id of a block that is ready to be sent to VM for execution (its parent has finished
    /// execution), if such block exists in the tree.
    pub fn get_block_to_execute(&mut self) -> Option<HashValue> {
        let mut to_visit: Vec<HashValue> = self.heads.iter().cloned().collect();

        while let Some(id) = to_visit.pop() {
            let block = self
                .id_to_block
                .get(&id)
                .expect("Missing block in id_to_block.");
            if !block.is_executed() {
                return Some(id);
            }
            to_visit.extend(block.children().iter().cloned());
        }

        None
    }

    /// Marks given block and all its uncommitted ancestors as committed. This does not cause these
    /// blocks to be sent to storage immediately.
    pub fn mark_as_committed(
        &mut self,
        id: HashValue,
        signature: B::Signature,
    ) -> Result<(), CommitBlockError> {
        // First put the signatures in the block. Note that if this causes multiple blocks to be
        // marked as committed, only the last one will have the signatures.
        match self.id_to_block.get_mut(&id) {
            Some(block) => {
                if block.is_committed() {
                    bail_err!(CommitBlockError::BlockAlreadyMarkedAsCommitted { id });
                } else {
                    block.set_signature(signature);
                }
            }
            None => bail_err!(CommitBlockError::BlockNotFound { id }),
        }

        // Mark the current block as committed. Go to parent block and repeat until a committed
        // block is found, or no more blocks.
        let mut current_id = id;
        while let Some(block) = self.id_to_block.get_mut(&current_id) {
            if block.is_committed() {
                break;
            }

            block.set_committed();
            current_id = block.parent_id();
        }

        Ok(())
    }

    /// Removes all blocks in the tree that conflict with committed blocks. Returns a list of
    /// blocks that are ready to be sent to storage (all the committed blocks that have been
    /// executed).
    pub fn prune(&mut self) -> Vec<B> {
        let mut blocks_to_store = vec![];

        // First find if there is a committed block in current heads. Since these blocks are at the
        // same height, at most one of them can be committed. If all of them are pending we have
        // nothing to do here.  Otherwise, one of the branches is committed. Throw away the rest of
        // them and advance to the next height.
        let mut current_heads = self.heads.clone();
        while let Some(committed_head) = self.get_committed_head(&current_heads) {
            assert!(
                current_heads.remove(&committed_head),
                "committed_head should exist.",
            );
            for id in current_heads {
                self.remove_branch(id);
            }

            match self.id_to_block.entry(committed_head) {
                hash_map::Entry::Occupied(entry) => {
                    current_heads = entry.get().children().clone();
                    let current_id = *entry.key();
                    let parent_id = entry.get().parent_id();
                    if entry.get().is_executed() {
                        // If this block has been executed, all its proper ancestors must have
                        // finished execution and present in `blocks_to_store`.
                        self.heads = current_heads.clone();
                        self.last_committed_id = current_id;
                        blocks_to_store.push(entry.remove());
                    } else {
                        // The current block has not finished execution. If the parent block does
                        // not exist in the map, that means parent block (also committed) has been
                        // executed and removed. Otherwise self.heads does not need to be changed.
                        if !self.id_to_block.contains_key(&parent_id) {
                            self.heads = HashSet::new();
                            self.heads.insert(current_id);
                        }
                    }
                }
                hash_map::Entry::Vacant(_) => unreachable!("committed_head_id should exist."),
            }
        }

        blocks_to_store
    }

    /// Given a list of heads, returns the committed one if it exists.
    fn get_committed_head(&self, heads: &HashSet<HashValue>) -> Option<HashValue> {
        let mut committed_head = None;
        for head in heads {
            let block = self
                .id_to_block
                .get(head)
                .expect("Head should exist in id_to_block.");
            if block.is_committed() {
                assert!(
                    committed_head.is_none(),
                    "Conflicting blocks are both committed.",
                );
                committed_head = Some(*head);
            }
        }
        committed_head
    }

    /// Removes a branch at block `head`.
    fn remove_branch(&mut self, head: HashValue) {
        let mut remaining = vec![head];
        while let Some(current_block_id) = remaining.pop() {
            let block = self
                .id_to_block
                .remove(&current_block_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Trying to remove a non-existing block {:x}.",
                        current_block_id,
                    )
                });
            assert!(
                !block.is_committed(),
                "Trying to remove a committed block {:x}.",
                current_block_id,
            );
            remaining.extend(block.children().iter());
        }
    }

    /// Removes the entire subtree at block `id`.
    pub fn remove_subtree(&mut self, id: HashValue) {
        self.heads.remove(&id);
        self.remove_branch(id);
    }

    /// Resets the block tree with a new `last_committed_id`. This removes all the in-memory
    /// blocks.
    pub fn reset(&mut self, last_committed_id: HashValue) {
        let mut new_block_tree = BlockTree::new(last_committed_id);
        std::mem::swap(self, &mut new_block_tree);
    }
}

/// An error returned by `add_block`. The error contains the block being added so the caller does
/// not lose it.
#[derive(Debug, Eq, PartialEq)]
pub enum AddBlockError<B: Block> {
    ParentNotFound { block: B },
    BlockAlreadyExists { block: B },
}

impl<B> AddBlockError<B>
where
    B: Block,
{
    pub fn into_block(self) -> B {
        match self {
            AddBlockError::ParentNotFound { block } => block,
            AddBlockError::BlockAlreadyExists { block } => block,
        }
    }
}

impl<B> std::fmt::Display for AddBlockError<B>
where
    B: Block,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AddBlockError::ParentNotFound { block } => {
                write!(f, "Parent block {:x} was not found.", block.parent_id())
            }
            AddBlockError::BlockAlreadyExists { block } => {
                write!(f, "Block {:x} already exists.", block.id())
            }
        }
    }
}

/// An error returned by `mark_as_committed`. The error contains id of the block the caller wants
/// to commit.
#[derive(Debug, Eq, PartialEq)]
pub enum CommitBlockError {
    BlockNotFound { id: HashValue },
    BlockAlreadyMarkedAsCommitted { id: HashValue },
}

impl std::fmt::Display for CommitBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommitBlockError::BlockNotFound { id } => write!(f, "Block {:x} was not found.", id),
            CommitBlockError::BlockAlreadyMarkedAsCommitted { id } => {
                write!(f, "Block {:x} was already marked as committed.", id)
            }
        }
    }
}
