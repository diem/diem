// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements [`SparseMerkleTree`] backed by storage module. The tree itself doesn't
//! persist anything, but realizes the logic of R/W only. The write path will produce all the
//! intermediate results in a batch for storage layer to commit and the read path will return
//! results directly. The public APIs are only [`new`](SparseMerkleTree::new),
//! [`put_blob_sets`](SparseMerkleTree::put_blob_sets),
//! [`put_keyed_blob_set`](SparseMerkleTree::put_keyed_blob_set) and
//! [`get_with_proof`](SparseMerkleTree::get_with_proof). After each put with a `keyed_blob_set`
//! based on a known root, the tree will return a new root hash with a [`TreeUpdateBatch`]
//! containing all newly generated tree nodes and blobs.
//!
//! The sparse Merkle tree itself logically is a 256-bit Merkle tree with an optimization
//! that any subtree containing 0 or 1 leaf node will be replaced by that leaf node or a placeholder
//! node with default hash value. With this optimization we can save CPU by avoiding hashing on
//! many sparse levels in the tree. Physically, the tree is structurally similar to the modified
//! Patricia Merkle tree of Ethereum, with some modifications. Please read the code for details.

#![allow(clippy::unit_arg)]

#[cfg(test)]
mod mock_tree_store;
mod nibble_path;
mod node_serde;
pub mod node_type;
#[cfg(test)]
mod sparse_merkle_test;
mod tree_cache;

use crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use failure::prelude::*;
use nibble_path::{skip_common_prefix, NibbleIterator, NibblePath};
use node_type::{BranchNode, ExtensionNode, LeafNode, Node};
use num_derive::{FromPrimitive, ToPrimitive};
use proptest_derive::Arbitrary;
use std::collections::{HashMap, HashSet};
use tree_cache::TreeCache;
use types::{
    account_state_blob::AccountStateBlob, proof::definition::SparseMerkleProof,
    transaction::Version,
};

/// The hardcoded maximum height of a [`SparseMerkleTree`] in nibbles.
const ROOT_NIBBLE_HEIGHT: usize = HashValue::LENGTH * 2;

/// TreeReader defines the interface between [`SparseMerkleTree`] and underlying storage holding
/// nodes and blobs.
pub trait TreeReader {
    /// Get state Merkle tree node given node hash
    fn get_node(&self, node_hash: HashValue) -> Result<Node>;
    /// Get state Merkle tree blob given blob hash
    fn get_blob(&self, blob_hash: HashValue) -> Result<AccountStateBlob>;
}

/// Node batch that will be written into db atomically with other batches.
pub type NodeBatch = HashMap<HashValue, Node>;
/// Blob batch that will be written into db atomically with other batches.
pub type BlobBatch = HashMap<HashValue, AccountStateBlob>;
/// RetireLog batch that will be written into db atomically with other batches.
pub type RetireLogBatch = HashSet<RetiredStateRecord>;

/// Indicates a record becomes outdated on `version_retired`.
#[derive(Arbitrary, Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetiredStateRecord {
    /// The version after which a record becomes outdated.
    pub version_retired: Version,
    /// Indicates data set that the outdated record is stored in.
    pub record_type: RetiredRecordType,
    /// The version at which the outdated record was created.
    pub version_created: Version,
    /// Hash of the outdated record, which is the key in the data set storing the record.
    pub hash: HashValue,
}

/// Types of data sets a retired record can come from.
#[derive(
    Arbitrary, Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, FromPrimitive, ToPrimitive,
)]
pub enum RetiredRecordType {
    /// The Sparse Merkle nodes data set.
    Node = 0,
    /// The account state blob data set.
    Blob = 1,
}

// TODO: remove once version_created is for real part of the key.
const DUMMY_VERSION_CREATED: Version = 0;

/// This is a wrapper of [`NodeBatch`] and [`BlobBatch`] that represents the incremental
/// updates of tree after applying a write set, which is a vector of account_address and
/// new_account_state_blob pairs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeUpdateBatch {
    node_batch: NodeBatch,
    blob_batch: BlobBatch,
    retired_record_batch: RetireLogBatch,
}

/// Conversion between tuple type and `TreeUpdateBatch`.
impl From<TreeUpdateBatch> for (NodeBatch, BlobBatch, RetireLogBatch) {
    fn from(batch: TreeUpdateBatch) -> Self {
        (
            batch.node_batch,
            batch.blob_batch,
            batch.retired_record_batch,
        )
    }
}

/// The sparse Merkle tree data structure. See [`crate`] for description.
pub struct SparseMerkleTree<'a, R: 'a + TreeReader> {
    reader: &'a R,
}

impl<'a, R> SparseMerkleTree<'a, R>
where
    R: 'a + TreeReader,
{
    /// Creates a `SparseMerkleTree` backed by the given [`TreeReader`].
    pub fn new(reader: &'a R) -> Self {
        Self { reader }
    }

    /// Returns new nodes and account state blobs in a batch after applying `blob_set`. For
    /// example, if after transaction `T_i` the committed state of the tree in persistent storage
    /// looks like the following structure:
    ///
    /// ```text
    ///              S_i
    ///             /   \
    ///            .     .
    ///           .       .
    ///          /         \
    ///         o           x
    ///        / \
    ///       A   B
    ///        storage (disk)
    /// ```
    ///
    /// where `A` and `B` denote the states of two adjacent accounts, and `x` is a sibling on the
    /// path from root to A and B in the tree. Then a `blob_set` produced by the next
    /// transaction `T_{i+1}` modifies other accounts `C` and `D` that lives in
    /// the subtree under `x`, a new partial tree will be constructed in memory and the
    /// structure will be:
    ///
    /// ```text
    ///                 S_i      |      S_{i+1}
    ///                /   \     |     /       \
    ///               .     .    |    .         .
    ///              .       .   |   .           .
    ///             /         \  |  /             \
    ///            /           x | /               x'
    ///           o<-------------+-               / \
    ///          / \             |               C   D
    ///         A   B            |
    ///           storage (disk) |    cache (memory)
    /// ```
    ///
    /// With this design, we are able to query the global state in persistent storage and
    /// generate the proposed tree delta based on a specific root hash and `blob_set`. For
    /// example, if we want to execute another transaction `T_{i+1}'`, we can use the tree `S_i` in
    /// storage and apply the `blob_set` of transaction `T_{i+1}`. Then if the storage commits
    /// the returned batch, the state `S_{i+1}` is ready to be read from the tree by calling
    /// [`get_with_proof`](SparseMerkleTree::get_with_proof). Anything inside the batch is not
    /// reachable from public interfaces before being committed.
    pub fn put_blob_set(
        &self,
        blob_set: Vec<(HashValue, AccountStateBlob)>,
        version: Version,
        root_hash: HashValue,
    ) -> Result<(HashValue, TreeUpdateBatch)> {
        let (mut root_hashes, tree_update_batch) =
            self.put_blob_sets(vec![blob_set], version, root_hash)?;
        let root_hash = root_hashes.pop().expect("root hash must exist");
        assert!(
            root_hashes.is_empty(),
            "root_hashes can only have 1 root_hash inside"
        );
        Ok((root_hash, tree_update_batch))
    }

    /// This is a helper function that calls
    /// [`put_keyed_blob_set`](SparseMerkleTree::put_keyed_blob_set) with a series of
    /// `keyed_blob_set`.
    #[inline]
    pub fn put_blob_sets(
        &self,
        blob_sets: Vec<Vec<(HashValue, AccountStateBlob)>>,
        first_version: Version,
        root_hash: HashValue,
    ) -> Result<(Vec<HashValue>, TreeUpdateBatch)> {
        let mut tree_cache = TreeCache::new(self.reader, root_hash, first_version);
        for (idx, blob_set) in blob_sets.into_iter().enumerate() {
            assert!(
                !blob_set.is_empty(),
                "Transactions that output empty write set should not be included.",
            );
            let version = first_version + idx as u64;
            blob_set
                .into_iter()
                .map(|(hash, value)| Self::put(hash, value, version, &mut tree_cache))
                .collect::<Result<_>>()?;
            // Freeze the current cache to make all contents in current cache immutable
            tree_cache.freeze();
        }

        Ok(tree_cache.into())
    }

    fn put(
        key: HashValue,
        blob: AccountStateBlob,
        _version: Version,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<()> {
        let nibble_path = NibblePath::new(key.to_vec());

        // Get the root node. If this is the first operation, it would get the root node from the
        // underlying db. Otherwise it most likely would come from `cache`.
        let root = tree_cache.get_root_node()?;

        // Get the blob hash and put the value blob into `tree_cache`.
        let blob_hash = blob.hash();
        tree_cache.put_blob(blob_hash, blob)?;

        // Start insertion from the root node.
        let new_root_hash =
            Self::insert_at(root, &mut nibble_path.nibbles(), blob_hash, tree_cache)?.0;
        tree_cache.set_root_hash(new_root_hash);

        Ok(())
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// `node`. Returns the hash of the created node and a boolean indicating whether the created
    /// node is a leaf node.
    ///
    /// It is safe to use recursion here because the max depth is limited by the key length which
    /// for this tree is the length of the hash of an account address.
    fn insert_at(
        node: Option<Node>,
        nibble_iter: &mut NibbleIterator,
        value_hash: HashValue,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(HashValue, bool)> {
        match node {
            Some(Node::Branch(branch_node)) => {
                Self::insert_at_branch_node(branch_node, nibble_iter, value_hash, tree_cache)
            }
            Some(Node::Extension(extension_node)) => {
                Self::insert_at_extension_node(extension_node, nibble_iter, value_hash, tree_cache)
            }
            Some(Node::Leaf(leaf_node)) => {
                Self::insert_at_leaf_node(leaf_node, nibble_iter, value_hash, tree_cache)
            }
            None => Self::create_leaf_node(nibble_iter, value_hash, tree_cache),
        }
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// `branch_node`. Returns the hash of the created node and a boolean indicating whether the
    /// created node is a leaf node.
    fn insert_at_branch_node(
        mut branch_node: BranchNode,
        nibble_iter: &mut NibbleIterator,
        value_hash: HashValue,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(HashValue, bool)> {
        // Delete the current branch node from `tree_cache` if it exists; Otherwise it is a noop
        // since we will update this branch node anyway. Even if the new branch node is exactly the
        // same as the old one, it is okay to delete it and then put the same node back.
        tree_cache.delete_node(branch_node.hash());

        // Find the next node to visit following the next nibble as index.
        let next_node_index = nibble_iter.next().expect("Ran out of nibbles");

        // Get the next node from `tree_cache` if it exists; Otherwise it will be `None`.
        let next_node = branch_node
            .child(next_node_index)
            .map(|hash| tree_cache.get_node(hash))
            .transpose()?;

        // Traverse downwards from this branch node recursively to get the hash of the child node
        // at `next_node_index`.
        let (new_child_hash, is_new_child_leaf) = match next_node {
            Some(child) => Self::insert_at(Some(child), nibble_iter, value_hash, tree_cache)?,
            None => Self::create_leaf_node(nibble_iter, value_hash, tree_cache)?,
        };

        // Reuse the current `BranchNode` in memory to create a new branch node.
        branch_node.set_child(next_node_index, (new_child_hash, is_new_child_leaf));
        let new_node_hash = branch_node.hash();

        // Cache this new branch node with `new_node_hash` as the key.
        tree_cache.put_node(new_node_hash, branch_node.into())?;
        Ok((new_node_hash, false /* is_leaf */))
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// extension node `extension_node`. Returns the hash of the created node and a boolean
    /// indicating whether the created node is leaf node (always `false`).
    fn insert_at_extension_node(
        mut extension_node: ExtensionNode,
        nibble_iter: &mut NibbleIterator,
        value_hash: HashValue,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(HashValue, bool)> {
        // We are on an extension node but are trying to insert another node, so we may need to add
        // a new path.

        // Delete the current extension node from tree_cache if it exists; Otherwise it is a
        // noop.
        tree_cache.delete_node(extension_node.hash());

        // Determine the common prefix between this extension node and the nibble iterator of the
        // incoming key.
        let mut extension_nibble_iter = extension_node.nibble_path().nibbles();
        skip_common_prefix(&mut extension_nibble_iter, nibble_iter);

        // There are two possible cases after matching prefix:
        // 1. All the nibbles of the extension node matches the nibble path of the incoming node,
        // so just visit the next node recursively. Note: the next node must be a branch node
        // otherwise the tree is corrupted.
        if extension_nibble_iter.is_finished() {
            assert!(
                !nibble_iter.is_finished(),
                "We should never end the search on an extension node when key length is fixed."
            );
            let (inserted_child_hash, _is_leaf) = match tree_cache
                .get_node(extension_node.child())?
            {
                Node::Branch(branch_node) => {
                    Self::insert_at_branch_node(branch_node, nibble_iter, value_hash, tree_cache)?
                }
                _ => bail!("Extension node shouldn't have a non-branch node as child"),
            };
            extension_node.set_child(inserted_child_hash);
            let new_node_hash = extension_node.hash();
            tree_cache.put_node(new_node_hash, extension_node.into())?;
            return Ok((new_node_hash, false /* is_leaf */));
        }

        // 2. Not all the nibbles of the extension node match the nibble iterator of the
        // incoming key; there are several cases. Let us assume `O` denotes a nibble and `X`
        // denotes the first mismatched nibble. The nibble path of the extension node can be
        // illustrated as `(O...)X(O...)`: the extension node will be replaced by an optional
        // extension node if needed before the fork, followed by a branch node at the fork and
        // another possible extension node if needed after the fork. We create new nodes in a
        // bottom-up order.

        // 1) Cache the visited nibbles. We will use it in step 4).
        let extension_nibble_iter_before_fork = extension_nibble_iter.visited_nibbles();

        // 2) Create the branch node at the fork, i.e., `X`,  as described above.
        let mut new_branch_node_at_fork = BranchNode::default();
        let extension_node_index = extension_nibble_iter.next().expect("Ran out of nibbles");
        let new_leaf_node_index = nibble_iter.next().expect("Ran out of nibbles");
        assert_ne!(extension_node_index, new_leaf_node_index);

        // 3) Connects the two children to the branch node at fork; create if necessary.
        let extension_nibble_iter_after_fork = extension_nibble_iter.remaining_nibbles();
        // Check whether the extension node after fork is necessary.
        if extension_nibble_iter_after_fork.num_nibbles() != 0 {
            // `...XO...` case: some nibbles of extension node are left after fork.
            let new_extension_node_after_fork = Node::new_extension(
                extension_nibble_iter_after_fork.get_nibble_path(),
                extension_node.child(),
            );
            let new_extension_node_after_fork_hash = new_extension_node_after_fork.hash();
            tree_cache.put_node(
                new_extension_node_after_fork_hash,
                new_extension_node_after_fork,
            )?;
            new_branch_node_at_fork.set_child(
                extension_node_index,
                (new_extension_node_after_fork_hash, false /* is_leaf */),
            );
        } else {
            // `...X` case: the nibble at the fork is the last nibble of the extension node.
            new_branch_node_at_fork.set_child(
                extension_node_index,
                (
                    extension_node.child(),
                    false, /* is_leaf, extension node must have a branch node as child */
                ),
            );
        }

        // Set another child of the new branch node to be the new inserted leaf node.
        new_branch_node_at_fork.set_child(
            new_leaf_node_index,
            Self::create_leaf_node(nibble_iter, value_hash, tree_cache)?,
        );
        let mut new_node_hash = new_branch_node_at_fork.hash();
        tree_cache.put_node(new_node_hash, new_branch_node_at_fork.into())?;

        // 4) Check whether a extension node before the fork is necessary.
        if extension_nibble_iter_before_fork.num_nibbles() != 0 {
            let new_extension_node_before_fork = Node::new_extension(
                extension_nibble_iter_before_fork.get_nibble_path(),
                new_node_hash,
            );
            new_node_hash = new_extension_node_before_fork.hash();
            tree_cache.put_node(new_node_hash, new_extension_node_before_fork)?;
        }
        Ok((new_node_hash, false /* is_leaf */))
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// `leaf_node`. Returns the hash of the created node and a boolean indicating whether
    /// the created node is a leaf node.
    fn insert_at_leaf_node(
        existing_leaf_node: LeafNode,
        nibble_iter: &mut NibbleIterator,
        value_hash: HashValue,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(HashValue, bool)> {
        // We are on a leaf node but trying to insert another node, so we may diverge. Different
        // from insertion at branch nodes or at extension nodes, we don't delete the existing
        // leaf node here unless it has the same key as the incoming key.

        // 1. Make sure that the existing leaf nibble_path has the same prefix as the already
        // visited part of the nibble iter of the incoming key and advances the existing leaf
        // nibble iterator by the length of that prefix.
        let mut visited_nibble_iter = nibble_iter.visited_nibbles();
        let existing_leaf_nibble_path = NibblePath::new(existing_leaf_node.key().to_vec());
        let mut existing_leaf_nibble_iter = existing_leaf_nibble_path.nibbles();
        skip_common_prefix(&mut visited_nibble_iter, &mut existing_leaf_nibble_iter);
        assert!(
            visited_nibble_iter.is_finished(),
            "Leaf nodes failed to share the same visited nibbles before index {}",
            existing_leaf_nibble_iter.visited_nibbles().num_nibbles()
        );

        // 2. Determine the secondary common prefix between this leaf node and the incoming
        // key.
        let mut pruned_existing_leaf_nibble_iter = existing_leaf_nibble_iter.remaining_nibbles();
        skip_common_prefix(nibble_iter, &mut pruned_existing_leaf_nibble_iter);
        assert_eq!(
            nibble_iter.is_finished(),
            pruned_existing_leaf_nibble_iter.is_finished(),
            "key lengths mismatch."
        );
        // Both are finished. That means the incoming key already exists in the tree and we just
        // need to update its value.
        if nibble_iter.is_finished() {
            // Imply `&& pruned_existing_leaf_nibble_iter.is_finished()`.
            // Delete the old leaf and create the new one.
            // Note: it is necessary to delete the corresponding blob too.
            tree_cache.delete_blob(existing_leaf_node.value_hash());
            tree_cache.delete_node(existing_leaf_node.hash());
            return Ok(Self::create_leaf_node(nibble_iter, value_hash, tree_cache)?);
        }

        // Not both are finished. This means the incoming key branches off at some point.
        // Then create a branch node at the fork, a new leaf node for the incoming key, and
        // create an extension node if the secondary common prefix length is not 0.
        // We create new nodes in a bottom-up order.

        // 1) Keep the visited nibble iterator. We will use it in step 3).
        let pruned_existing_leaf_nibble_iter_before_fork =
            pruned_existing_leaf_nibble_iter.visited_nibbles();

        // 2) Create the branch node at the fork.
        let existing_leaf_index = pruned_existing_leaf_nibble_iter
            .next()
            .expect("Ran out of nibbles");
        let new_leaf_index = nibble_iter.next().expect("Ran out of nibbles");
        assert_ne!(existing_leaf_index, new_leaf_index);

        let mut branch_node = BranchNode::default();
        branch_node.set_child(
            existing_leaf_index,
            (existing_leaf_node.hash(), true /* is_leaf */),
        );
        branch_node.set_child(
            new_leaf_index,
            Self::create_leaf_node(nibble_iter, value_hash, tree_cache)?,
        );

        let mut new_node_hash = branch_node.hash();
        tree_cache.put_node(new_node_hash, branch_node.into())?;

        // 3) Create an extension node before the fork if necessary.
        if pruned_existing_leaf_nibble_iter_before_fork.num_nibbles() != 0 {
            let new_extension_node = Node::new_extension(
                pruned_existing_leaf_nibble_iter_before_fork.get_nibble_path(),
                new_node_hash,
            );
            new_node_hash = new_extension_node.hash();
            tree_cache.put_node(new_node_hash, new_extension_node)?;
        }
        Ok((new_node_hash, false /* is_leaf */))
    }

    /// Helper function for creating leaf nodes. Returns the hash of the newly created leaf node and
    /// a boolean indicating whether the created node is leaf node (always `true`).
    fn create_leaf_node(
        nibble_iter: &NibbleIterator,
        value_hash: HashValue,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(HashValue, bool)> {
        // Get the underlying bytes of nibble_iter which must be a key, i.e., hashed account address
        // with `HashValue::LENGTH` bytes.
        let nibble_path = nibble_iter.get_nibble_path();

        // Now create the new leaf node.
        let new_leaf = Node::new_leaf(HashValue::from_slice(nibble_path.bytes())?, value_hash);

        // Cache new leaf node and return its hash.
        let new_node_hash = new_leaf.hash();
        tree_cache.put_node(new_node_hash, new_leaf)?;
        Ok((new_node_hash, true /* is_leaf */))
    }

    /// Returns the account state blob (if applicable) and the corresponding merkle proof.
    pub fn get_with_proof(
        &self,
        key: HashValue,
        root_hash: HashValue,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        // Empty tree just returns proof with no sibling hash.
        if root_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
            return Ok((None, SparseMerkleProof::new(None, vec![])));
        }
        let mut siblings = vec![];
        let nibble_path = NibblePath::new(key.to_vec());
        let mut nibble_iter = nibble_path.nibbles();
        let mut next_hash = root_hash;

        // We limit the number of loops here deliberately to avoid potential cyclic graph bugs
        // in the tree structure.
        for _i in 0..ROOT_NIBBLE_HEIGHT {
            match self.reader.get_node(next_hash)? {
                Node::Branch(branch_node) => {
                    let queried_child_index = match nibble_iter.next() {
                        Some(nibble) => nibble,
                        // Shouldn't happen
                        None => bail!("ran out of nibbles"),
                    };
                    let (child_for_proof, mut siblings_in_branch) =
                        branch_node.get_child_for_proof_and_siblings(queried_child_index);
                    siblings.append(&mut siblings_in_branch);
                    next_hash = match child_for_proof {
                        Some(hash) => hash,
                        None => return Ok((None, SparseMerkleProof::new(None, siblings))),
                    };
                }
                Node::Extension(extension_node) => {
                    let (mut siblings_in_extension, needs_early_return) =
                        extension_node.get_siblings(&mut nibble_iter);
                    siblings.append(&mut siblings_in_extension);
                    if needs_early_return {
                        return Ok((None, SparseMerkleProof::new(None, siblings)));
                    }
                    next_hash = extension_node.child();
                }
                Node::Leaf(leaf_node) => {
                    return Ok((
                        if leaf_node.key() == key {
                            Some(self.reader.get_blob(leaf_node.value_hash())?)
                        } else {
                            None
                        },
                        SparseMerkleProof::new(
                            Some((leaf_node.key(), leaf_node.value_hash())),
                            siblings,
                        ),
                    ));
                }
            }
        }
        bail!("Sparse Merkle tree has cyclic graph inside.");
    }

    #[cfg(test)]
    pub fn get(&self, key: HashValue, root_hash: HashValue) -> Result<Option<AccountStateBlob>> {
        Ok(self.get_with_proof(key, root_hash)?.0)
    }
}
