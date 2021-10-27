// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements an in-memory Sparse Merkle Tree that is similar to what we use in
//! storage to represent world state. This tree will store only a small portion of the state -- the
//! part of accounts that have been modified by uncommitted transactions. For example, if we
//! execute a transaction T_i on top of committed state and it modified account A, we will end up
//! having the following tree:
//! ```text
//!              S_i
//!             /   \
//!            o     y
//!           / \
//!          x   A
//! ```
//! where A has the new state of the account, and y and x are the siblings on the path from root to
//! A in the tree.
//!
//! This Sparse Merkle Tree is immutable once constructed. If the next transaction T_{i+1} modified
//! another account B that lives in the subtree at y, a new tree will be constructed and the
//! structure will look like the following:
//! ```text
//!                 S_i        S_{i+1}
//!                /   \      /       \
//!               /     y   /          \
//!              / _______/             \
//!             //                       \
//!            o                          y'
//!           / \                        / \
//!          x   A                      z   B
//! ```
//!
//! Using this structure, we are able to query the global state, taking into account the output of
//! uncommitted transactions. For example, if we want to execute another transaction T_{i+1}', we
//! can use the tree S_i. If we look for account A, we can find its new value in the tree.
//! Otherwise, we know the account does not exist in the tree, and we can fall back to storage. As
//! another example, if we want to execute transaction T_{i+2}, we can use the tree S_{i+1} that
//! has updated values for both account A and B.
//!
//! Each version of the tree holds a strong reference (an Arc<Node>) to its root as well as one to
//! its base tree (S_i is the base tree of S_{i+1} in the above example). The root node in turn,
//! recursively holds all descendant nodes created in the same version, and weak references
//! (a Weak<Node>) to all descendant nodes that was created from previous versions.
//! With this construction:
//!     1. Even if a reference to a specific tree is dropped, the nodes belonging to it won't be
//! dropped as long as trees depending on it still hold strong references to it via the chain of
//! "base trees".
//!     2. Even if a tree is not dropped, when nodes it created are persisted to DB, all of them
//! and those created by its previous versions can be dropped, which we express by calling "prune()"
//! on it which replaces the strong references to its root and its base tree with weak references.
//!     3. We can hold strong references to recently accessed nodes that have already been persisted
//! in an LRU flavor cache for less DB reads.
//!
//! This Sparse Merkle Tree serves a dual purpose. First, to support a leader based consensus
//! algorithm, we need to build a tree of transactions like the following:
//! ```text
//! Committed -> T5 -> T6  -> T7
//!              └---> T6' -> T7'
//!                    └----> T7"
//! ```
//! Once T5 is executed, we will have a tree that stores the modified portion of the state. Later
//! when we execute T6 on top of T5, the output of T5 can be visible to T6.
//!
//! Second, given this tree representation it is straightforward to compute the root hash of S_i
//! once T_i is executed. This allows us to verify the proofs we need when executing T_{i+1}.

// See https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=e9c4c53eb80b30d09112fcfb07d481e7
#![allow(clippy::let_and_return)]
// See https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=795cd4f459f1d4a0005a99650726834b
#![allow(clippy::while_let_loop)]

mod node;
mod updater;
mod utils;

pub mod batches_update;
#[cfg(test)]
mod sparse_merkle_test;
#[cfg(any(test, feature = "bench", feature = "fuzzing"))]
pub mod test_utils;

use crate::sparse_merkle::{
    node::{NodeInner, SubTree},
    updater::SubTreeUpdater,
    utils::partition,
};
use diem_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use diem_infallible::Mutex;
use diem_types::{
    nibble::{nibble_path::NibblePath, ROOT_NIBBLE_HEIGHT},
    proof::SparseMerkleProof,
};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Arc, Weak},
};

/// To help finding the oldest ancestor of any SMT, a branch tracker is created each time
/// the chain of SMTs forked (two or more SMTs updating the same parent).
#[derive(Debug)]
struct BranchTracker<V> {
    /// Current branch head, n.b. when the head just started dropping, this weak link becomes
    /// invalid, we fall back to the `next`
    head: Weak<Inner<V>>,
    /// Dealing with the edge case where the branch head just started dropping, but the branch
    /// tracker hasn't been locked and updated yet.
    next: Weak<Inner<V>>,
    /// Parent branch, if any.
    parent: Option<Arc<Mutex<BranchTracker<V>>>>,
}

impl<V> BranchTracker<V> {
    fn new_head_unknown(parent: Option<Arc<Mutex<Self>>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            head: Weak::new(),
            next: Weak::new(),
            parent,
        }))
    }

    fn become_oldest(&mut self, head: &Arc<Inner<V>>, next: Option<&Arc<Inner<V>>>) {
        // Detach from parent
        // n.b. the parent branch might not be dropped after this, because whenever a fork
        //      happens, the first branch shares the parent branch tracker.
        self.parent = None;

        self.head = Arc::downgrade(head);
        self.next = next.map_or_else(Weak::new, Arc::downgrade)
    }

    fn parent(&self) -> Option<Arc<Mutex<Self>>> {
        self.parent.clone()
    }

    fn head(&self) -> Option<Arc<Inner<V>>> {
        // if `head.upgrade()` failed, it's that the head is being dropped.
        self.head.upgrade().or_else(|| self.next.upgrade())
    }
}

/// Keeps track of references of children and the branch tracker of the current branch.
#[derive(Debug)]
struct InnerLinks<V> {
    children: Vec<Arc<Inner<V>>>,
    branch_tracker: Arc<Mutex<BranchTracker<V>>>,
}

impl<V> InnerLinks<V> {
    fn new(branch_tracker: Arc<Mutex<BranchTracker<V>>>) -> Mutex<Self> {
        Mutex::new(Self {
            children: Vec::new(),
            branch_tracker,
        })
    }
}

/// The inner content of a sparse merkle tree, we have this so that even if a tree is dropped, the
/// INNER of it can still live if referenced by a previous version.
#[derive(Debug)]
struct Inner<V> {
    root: SubTree<V>,
    links: Mutex<InnerLinks<V>>,
    generation: u64,
}

impl<V> Drop for Inner<V> {
    fn drop(&mut self) {
        let mut stack = self.drain_children_for_drop();

        while let Some(descendant) = stack.pop() {
            if Arc::strong_count(&descendant) == 1 {
                // The only ref is the one we are now holding, so the structure will be dropped
                // after we free the `Arc`, which results in a chain of such structures being
                // dropped recursively and that might trigger a stack overflow. To prevent that we
                // follow the chain further to disconnect things beforehand.
                stack.extend(descendant.drain_children_for_drop());
            }
        }
    }
}

impl<V> Inner<V> {
    fn new(root: SubTree<V>) -> Arc<Self> {
        let branch_tracker = BranchTracker::new_head_unknown(None);
        let me = Arc::new(Self {
            root,
            links: InnerLinks::new(branch_tracker.clone()),
            generation: 0,
        });
        branch_tracker.lock().head = Arc::downgrade(&me);

        me
    }

    fn become_oldest(self: Arc<Self>) -> Arc<Self> {
        {
            let links_locked = self.links.lock();
            let mut branch_tracker_locked = links_locked.branch_tracker.lock();
            branch_tracker_locked.become_oldest(&self, links_locked.children.first());
        }
        self
    }

    fn spawn_impl(
        &self,
        child_root: SubTree<V>,
        branch_tracker: Arc<Mutex<BranchTracker<V>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            root: child_root,
            links: InnerLinks::new(branch_tracker),
            generation: self.generation + 1,
        })
    }

    fn spawn(self: &Arc<Self>, child_root: SubTree<V>) -> Arc<Self> {
        let mut links_locked = self.links.lock();

        let child = if links_locked.children.is_empty() {
            self.spawn_impl(child_root, links_locked.branch_tracker.clone())
        } else {
            // forking a new branch
            let branch_tracker =
                BranchTracker::new_head_unknown(Some(links_locked.branch_tracker.clone()));
            let child = self.spawn_impl(child_root, branch_tracker.clone());
            branch_tracker.lock().head = Arc::downgrade(&child);
            child
        };
        links_locked.children.push(child.clone());

        child
    }

    fn get_oldest_ancestor(self: &Arc<Self>) -> Arc<Self> {
        let (mut ret, mut parent) = {
            let branch_tracker = self.links.lock().branch_tracker.clone();
            let branch_tracker_locked = branch_tracker.lock();
            (
                branch_tracker_locked
                    .head()
                    .expect("Leaf must have a head."),
                branch_tracker_locked.parent(),
            )
        };

        while let Some(branch_tracker) = parent {
            let branch_tracker_locked = branch_tracker.lock();
            if let Some(head) = branch_tracker_locked.head() {
                // Whenever it forks, the first branch shares the BranchTracker with the parent,
                // hence this
                if head.generation < self.generation {
                    ret = head;
                    parent = branch_tracker_locked.parent();
                    continue;
                }
            }
            break;
        }

        ret
    }

    fn drain_children_for_drop(&self) -> Vec<Arc<Self>> {
        self.links
            .lock()
            .children
            .drain(..)
            .map(Self::become_oldest)
            .collect()
    }
}

/// The Sparse Merkle Tree implementation.
#[derive(Clone, Debug)]
pub struct SparseMerkleTree<V> {
    inner: Arc<Inner<V>>,
}

/// A type for tracking intermediate hashes at sparse merkle tree nodes in between batch
/// updates by transactions. It contains tuple (txn_id, hash_value, single_new_leaf), where
/// hash_value is the value after all the updates by transaction txn_id (txn_id-th batch)
/// and single_new_leaf is a bool that's true if the node subtree contains one new leaf.
/// (this is needed to recursively merge IntermediateHashes).
type IntermediateHashes = Vec<(usize, HashValue, bool)>;

impl<V> SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    /// Constructs a Sparse Merkle Tree with a root hash. This is often used when we restart and
    /// the scratch pad and the storage have identical state, so we use a single root hash to
    /// represent the entire state.
    pub fn new(root_hash: HashValue) -> Self {
        let root = if root_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
            SubTree::new_unknown(root_hash)
        } else {
            SubTree::new_empty()
        };

        Self {
            inner: Inner::new(root),
        }
    }

    fn get_oldest_ancestor(&self) -> Self {
        Self {
            inner: self.inner.get_oldest_ancestor(),
        }
    }

    pub fn freeze(self) -> FrozenSparseMerkleTree<V> {
        let base_smt = self.get_oldest_ancestor();
        let base_generation = base_smt.inner.generation;

        FrozenSparseMerkleTree {
            _base_smt: base_smt,
            base_generation,
            smt: self,
        }
    }

    #[cfg(test)]
    fn new_with_root(root: SubTree<V>) -> Self {
        Self {
            inner: Inner::new(root),
        }
    }

    fn root_weak(&self) -> SubTree<V> {
        self.inner.root.weak()
    }

    /// Returns the root hash of this tree.
    pub fn root_hash(&self) -> HashValue {
        self.inner.root.hash()
    }
}

/// In tests and benchmark, reference to ancestors are manually managed
#[cfg(any(feature = "fuzzing", feature = "bench", test))]
impl<V> SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    pub fn serial_update(
        &self,
        update_batch: Vec<Vec<(HashValue, &V)>>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(Vec<(HashValue, HashMap<NibblePath, HashValue>)>, Self), UpdateError> {
        self.clone()
            .freeze()
            .serial_update(update_batch, proof_reader)
            .map(|(hashes, smt)| (hashes, smt.unfreeze()))
    }

    pub fn batch_update(
        &self,
        updates: Vec<(HashValue, &V)>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<Self, UpdateError> {
        self.clone()
            .freeze()
            .batch_update(updates, proof_reader)
            .map(FrozenSparseMerkleTree::unfreeze)
    }

    pub fn get(&self, key: HashValue) -> AccountStatus<V> {
        self.clone().freeze().get(key)
    }
}

impl<V> Default for SparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    fn default() -> Self {
        SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH)
    }
}

/// `AccountStatus` describes the result of querying an account from this SparseMerkleTree.
#[derive(Debug, Eq, PartialEq)]
pub enum AccountStatus<V> {
    /// The account exists in the tree, therefore we can give its value.
    ExistsInScratchPad(V),

    /// The account does not exist in the tree, but exists in DB. This happens when the search
    /// reaches a leaf node that has the requested account, but the node has only the value hash
    /// because it was loaded into memory as part of a non-inclusion proof. When we go to DB we
    /// don't need to traverse the tree to find the same leaf, instead we can use the value hash to
    /// look up the account content directly.
    ExistsInDB,

    /// The account does not exist in either the tree or DB. This happens when the search reaches
    /// an empty node, or a leaf node that has a different account.
    DoesNotExist,

    /// We do not know if this account exists or not and need to go to DB to find out. This happens
    /// when the search reaches a subtree node.
    Unknown,
}

/// In the entire lifetime of this, in-mem nodes won't be dropped because a reference to the oldest
/// SMT is held inside.
#[derive(Clone, Debug)]
pub struct FrozenSparseMerkleTree<V> {
    _base_smt: SparseMerkleTree<V>,
    base_generation: u64,
    smt: SparseMerkleTree<V>,
}

impl<V> FrozenSparseMerkleTree<V>
where
    V: Clone + CryptoHash + Send + Sync,
{
    fn spawn(&self, child_root: SubTree<V>) -> Self {
        Self {
            _base_smt: self._base_smt.clone(),
            base_generation: self.base_generation,
            smt: SparseMerkleTree {
                inner: self.smt.inner.spawn(child_root),
            },
        }
    }

    pub fn unfreeze(self) -> SparseMerkleTree<V> {
        self.smt
    }

    pub fn root_hash(&self) -> HashValue {
        self.smt.root_hash()
    }

    /// Constructs a new Sparse Merkle Tree as if we are updating the existing tree multiple
    /// times with the `batch_update`. The function will return the root hash after each
    /// update and a Sparse Merkle Tree of the final state.
    ///
    /// The `serial_update` applies `batch_update' method many times, unlike a more optimized
    /// (and parallelizable) `batches_update' implementation below. It takes in a reference of
    /// value instead of an owned instance to be consistent with the `batches_update' interface.
    pub fn serial_update(
        &self,
        update_batch: Vec<Vec<(HashValue, &V)>>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<(Vec<(HashValue, HashMap<NibblePath, HashValue>)>, Self), UpdateError> {
        let mut current_state_tree = self.clone();
        let mut result = Vec::with_capacity(update_batch.len());
        for updates in update_batch {
            // sort and dedup the accounts
            let accounts = updates
                .iter()
                .map(|(account, _)| *account)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            current_state_tree = current_state_tree.batch_update(updates, proof_reader)?;
            result.push((
                current_state_tree.smt.root_hash(),
                current_state_tree.generate_node_hashes(accounts),
            ));
        }
        Ok((result, current_state_tree))
    }

    /// This is a helper function that compares an updated in-memory sparse merkle with the
    /// current on-disk jellyfish sparse merkle to get the hashes of newly generated nodes.
    pub fn generate_node_hashes(
        &self,
        // must be sorted
        touched_accounts: Vec<HashValue>,
    ) -> HashMap<NibblePath, HashValue> {
        let mut node_hashes = HashMap::new();
        let mut nibble_path = NibblePath::new(vec![]);
        self.collect_new_hashes(
            touched_accounts.as_slice(),
            self.smt.root_weak(),
            0, /* depth in nibble */
            0, /* level within a nibble*/
            &mut nibble_path,
            &mut node_hashes,
        );
        node_hashes
    }

    /// Recursively generate the partial node update batch of jellyfish merkle
    fn collect_new_hashes(
        &self,
        keys: &[HashValue],
        subtree: SubTree<V>,
        depth_in_nibble: usize,
        level_within_nibble: usize,
        cur_nibble_path: &mut NibblePath,
        node_hashes: &mut HashMap<NibblePath, HashValue>,
    ) {
        assert!(depth_in_nibble <= ROOT_NIBBLE_HEIGHT);
        if keys.is_empty() {
            return;
        }

        if level_within_nibble == 0 {
            if depth_in_nibble != 0 {
                cur_nibble_path
                    .push(NibblePath::new(keys[0].to_vec()).get_nibble(depth_in_nibble - 1));
            }
            node_hashes.insert(cur_nibble_path.clone(), subtree.hash());
        }
        match subtree
            .get_node_if_in_mem(self.base_generation)
            .expect("must exist")
            .inner()
            .borrow()
        {
            NodeInner::Internal(internal_node) => {
                let (next_nibble_depth, next_level_within_nibble) = if level_within_nibble == 3 {
                    (depth_in_nibble + 1, 0)
                } else {
                    (depth_in_nibble, level_within_nibble + 1)
                };
                let pivot = partition(
                    &keys.iter().map(|k| (*k, ())).collect::<Vec<_>>()[..],
                    depth_in_nibble * 4 + level_within_nibble,
                );
                self.collect_new_hashes(
                    &keys[..pivot],
                    internal_node.left.weak(),
                    next_nibble_depth,
                    next_level_within_nibble,
                    cur_nibble_path,
                    node_hashes,
                );
                self.collect_new_hashes(
                    &keys[pivot..],
                    internal_node.right.weak(),
                    next_nibble_depth,
                    next_level_within_nibble,
                    cur_nibble_path,
                    node_hashes,
                );
            }
            NodeInner::Leaf(leaf_node) => {
                assert_eq!(keys.len(), 1);
                assert_eq!(keys[0], leaf_node.key);
                if level_within_nibble != 0 {
                    let mut leaf_nibble_path = cur_nibble_path.clone();
                    leaf_nibble_path
                        .push(NibblePath::new(keys[0].to_vec()).get_nibble(depth_in_nibble));
                    node_hashes.insert(leaf_nibble_path, subtree.hash());
                }
            }
        }
        if level_within_nibble == 0 && depth_in_nibble != 0 {
            cur_nibble_path.pop();
        }
    }

    /// Constructs a new Sparse Merkle Tree by applying `updates`, which are considered to happen
    /// all at once. See `serial_update` and `batches_update` which take in multiple batches
    /// of updates and yields intermediate results.
    /// Since the tree is immutable, existing tree remains the same and may share parts with the
    /// new, returned tree.
    pub fn batch_update(
        &self,
        updates: Vec<(HashValue, &V)>,
        proof_reader: &impl ProofRead<V>,
    ) -> Result<Self, UpdateError> {
        // Flatten, dedup and sort the updates with a btree map since the updates between different
        // versions may overlap on the same address in which case the latter always overwrites.
        let kvs = updates
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();

        let current_root = self.smt.root_weak();
        if kvs.is_empty() {
            Ok(self.clone())
        } else {
            let root = SubTreeUpdater::update(
                current_root,
                &kvs[..],
                proof_reader,
                self.smt.inner.generation + 1,
            )?;
            Ok(self.spawn(root))
        }
    }

    /// Queries a `key` in this `SparseMerkleTree`.
    pub fn get(&self, key: HashValue) -> AccountStatus<V> {
        let mut cur = self.smt.root_weak();
        let mut bits = key.iter_bits();

        loop {
            if let Some(node) = cur.get_node_if_in_mem(self.base_generation) {
                if let NodeInner::Internal(internal_node) = node.inner() {
                    match bits.next() {
                        Some(bit) => {
                            cur = if bit {
                                internal_node.right.weak()
                            } else {
                                internal_node.left.weak()
                            };
                            continue;
                        }
                        None => panic!("Tree is deeper than {} levels.", HashValue::LENGTH_IN_BITS),
                    }
                }
            }
            break;
        }

        let ret = match cur {
            SubTree::Empty => AccountStatus::DoesNotExist,
            SubTree::NonEmpty { root, .. } => match root.get_if_in_mem() {
                None => AccountStatus::Unknown,
                Some(node) => match node.inner() {
                    NodeInner::Internal(_) => {
                        unreachable!("There is an internal node at the bottom of the tree.")
                    }
                    NodeInner::Leaf(leaf_node) => {
                        if leaf_node.key == key {
                            match &leaf_node.value.data.get_if_in_mem() {
                                Some(value) => {
                                    AccountStatus::ExistsInScratchPad(value.as_ref().clone())
                                }
                                None => AccountStatus::ExistsInDB,
                            }
                        } else {
                            AccountStatus::DoesNotExist
                        }
                    }
                },
            },
        };
        ret
    }
}

/// A type that implements `ProofRead` can provide proof for keys in persistent storage.
pub trait ProofRead<V>: Sync {
    /// Gets verified proof for this key in persistent storage.
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof<V>>;
}

/// All errors `update` can possibly return.
#[derive(Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// The update intends to insert a key that does not exist in the tree, so the operation needs
    /// proof to get more information about the tree, but no proof is provided.
    MissingProof,
    /// At `depth` a persisted subtree was encountered and a proof was requested to assist finding
    /// details about the subtree, but the result proof indicates the subtree is empty.
    ShortProof {
        key: HashValue,
        num_siblings: usize,
        depth: usize,
    },
}
