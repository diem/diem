// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This module implements [`JellyfishMerkleTree`] backed by storage module. The tree itself doesn't
//! persist anything, but realizes the logic of R/W only. The write path will produce all the
//! intermediate results in a batch for storage layer to commit and the read path will return
//! results directly. The public APIs are only [`new`], [`put_blob_sets`], [`put_blob_set`] and
//! [`get_with_proof`]. After each put with a `blob_set` based on a known version, the tree will
//! return a new root hash with a [`TreeUpdateBatch`] containing all the new nodes and indices of
//! stale nodes.
//!
//! A Jellyfish Merkle Tree itself logically is a 256-bit sparse Merkle tree with an optimization
//! that any subtree containing 0 or 1 leaf node will be replaced by that leaf node or a placeholder
//! node with default hash value. With this optimization we can save CPU by avoiding hashing on
//! many sparse levels in the tree. Physically, the tree is structurally similar to the modified
//! Patricia Merkle tree of Ethereum but with some modifications. A standard Jellyfish Merkle tree
//! will look like the following figure:
//!
//! ```text
//!                                    .──────────────────────.
//!                            _.─────'                        `──────.
//!                       _.──'                                        `───.
//!                   _.─'                                                  `──.
//!               _.─'                                                          `──.
//!             ,'                                                                  `.
//!          ,─'                                                                      '─.
//!        ,'                                                                            `.
//!      ,'                                                                                `.
//!     ╱                                                                                    ╲
//!    ╱                                                                                      ╲
//!   ╱                                                                                        ╲
//!  ╱                                                                                          ╲
//! ;                                                                                            :
//! ;                                                                                            :
//!;                                                                                              :
//!│                                                                                              │
//!+──────────────────────────────────────────────────────────────────────────────────────────────+
//! .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.  .''.
//!/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \/    \
//!+----++----++----++----++----++----++----++----++----++----++----++----++----++----++----++----+
//! (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (
//!  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )
//! (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (
//!  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )
//! (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (
//!  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )
//! (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (
//!  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )  )
//! (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (  (
//! ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■  ■
//! ■: account_state_blob
//! ```
//!
//! A Jellyfish Merkle Tree consists of [`InternalNode`] and [`LeafNode`]. [`InternalNode`] is like
//! branch node in ethereum patricia merkle with 16 children to represent a 4-level binary tree and
//! [`LeafNode`] is similar to that in patricia merkle too. In the above figure, each `bell` in the
//! jellyfish is an [`InternalNode`] while each tentacle is a [`LeafNode`]. It is noted that
//! Jellyfish merkle doesn't have a counterpart for `extension` node of ethereum patricia merkle.
//!
//! [`JellyfishMerkleTree`]: struct.JellyfishMerkleTree.html
//! [`new`]: struct.JellyfishMerkleTree.html#method.new
//! [`put_blob_sets`]: struct.JellyfishMerkleTree.html#method.put_blob_sets
//! [`put_blob_set`]: struct.JellyfishMerkleTree.html#method.put_blob_set
//! [`get_with_proof`]: struct.JellyfishMerkleTree.html#method.get_with_proof
//! [`TreeUpdateBatch`]: struct.TreeUpdateBatch.html
//! [`InternalNode`]: node_type/struct.InternalNode.html
//! [`LeafNode`]: node_type/struct.LeafNode.html

pub mod iterator;
#[cfg(test)]
mod jellyfish_merkle_test;
#[cfg(test)]
mod mock_tree_store;
mod nibble_path;
pub mod node_type;
pub mod restore;
#[cfg(test)]
mod test_helper;
mod tree_cache;

use anyhow::{bail, ensure, format_err, Result};
use libra_crypto::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob,
    proof::{SparseMerkleProof, SparseMerkleRangeProof},
    transaction::Version,
};
use nibble_path::{skip_common_prefix, NibbleIterator, NibblePath};
use node_type::{Child, Children, InternalNode, LeafNode, Node, NodeKey};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use std::collections::{BTreeMap, BTreeSet};
use tree_cache::TreeCache;

/// The hardcoded maximum height of a [`JellyfishMerkleTree`] in nibbles.
pub const ROOT_NIBBLE_HEIGHT: usize = HashValue::LENGTH * 2;

/// `TreeReader` defines the interface between
/// [`JellyfishMerkleTree`](struct.JellyfishMerkleTree.html)
/// and underlying storage holding nodes.
pub trait TreeReader {
    /// Gets node given a node key. Returns error if the node does not exist.
    fn get_node(&self, node_key: &NodeKey) -> Result<Node> {
        self.get_node_option(node_key)?
            .ok_or_else(|| format_err!("Missing node at {:?}.", node_key))
    }

    /// Gets node given a node key. Returns `None` if the node does not exist.
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>>;

    /// Gets the rightmost leaf. Note that this assumes we are in the process of restoring the tree
    /// and all nodes are at the same version.
    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode)>>;
}

pub trait TreeWriter {
    /// Writes a node batch into storage.
    fn write_node_batch(&self, node_batch: &NodeBatch) -> Result<()>;
}

/// Node batch that will be written into db atomically with other batches.
pub type NodeBatch = BTreeMap<NodeKey, Node>;
/// [`StaleNodeIndex`](struct.StaleNodeIndex.html) batch that will be written into db atomically
/// with other batches.
pub type StaleNodeIndexBatch = BTreeSet<StaleNodeIndex>;

/// Indicates a node becomes stale since `stale_since_version`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct StaleNodeIndex {
    /// The version since when the node is overwritten and becomes stale.
    pub stale_since_version: Version,
    /// The [`NodeKey`](node_type/struct.NodeKey.html) identifying the node associated with this
    /// record.
    pub node_key: NodeKey,
}

/// This is a wrapper of [`NodeBatch`](type.NodeBatch.html),
/// [`StaleNodeIndexBatch`](type.StaleNodeIndexBatch.html) and some stats of nodes that represents
/// the incremental updates of a tree and pruning indices after applying a write set,
/// which is a vector of `hashed_account_address` and `new_account_state_blob` pairs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeUpdateBatch {
    pub node_batch: NodeBatch,
    pub stale_node_index_batch: StaleNodeIndexBatch,
    pub num_new_leaves: usize,
    pub num_stale_leaves: usize,
}

/// The Jellyfish Merkle tree data structure. See [`crate`] for description.
pub struct JellyfishMerkleTree<'a, R: 'a + TreeReader> {
    reader: &'a R,
}

impl<'a, R> JellyfishMerkleTree<'a, R>
where
    R: 'a + TreeReader,
{
    /// Creates a `JellyfishMerkleTree` backed by the given [`TreeReader`](trait.TreeReader.html).
    pub fn new(reader: &'a R) -> Self {
        Self { reader }
    }

    /// This is a convenient function that calls
    /// [`put_blob_sets`](struct.JellyfishMerkleTree.html#method.put_blob_sets) with a single
    /// `keyed_blob_set`.
    #[cfg(test)]
    pub fn put_blob_set(
        &self,
        blob_set: Vec<(HashValue, AccountStateBlob)>,
        version: Version,
    ) -> Result<(HashValue, TreeUpdateBatch)> {
        let (root_hashes, tree_update_batch) = self.put_blob_sets(vec![blob_set], version)?;
        assert_eq!(
            root_hashes.len(),
            1,
            "root_hashes must consist of a single value.",
        );
        Ok((root_hashes[0], tree_update_batch))
    }

    /// Returns the new nodes and account state blobs in a batch after applying `blob_set`. For
    /// example, if after transaction `T_i` the committed state of tree in the persistent storage
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
    /// where `A` and `B` denote the states of two adjacent accounts, and `x` is a sibling subtree
    /// of the path from root to A and B in the tree. Then a `blob_set` produced by the next
    /// transaction `T_{i+1}` modifies other accounts `C` and `D` exist in the subtree under `x`, a
    /// new partial tree will be constructed in memory and the structure will be:
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
    /// [`get_with_proof`](struct.JellyfishMerkleTree.html#method.get_with_proof). Anything inside
    /// the batch is not reachable from public interfaces before being committed.
    pub fn put_blob_sets(
        &self,
        blob_sets: Vec<Vec<(HashValue, AccountStateBlob)>>,
        first_version: Version,
    ) -> Result<(Vec<HashValue>, TreeUpdateBatch)> {
        let mut tree_cache = TreeCache::new(self.reader, first_version)?;
        for (idx, blob_set) in blob_sets.into_iter().enumerate() {
            assert!(
                !blob_set.is_empty(),
                "Transactions that output empty write set should not be included.",
            );
            let version = first_version + idx as u64;
            blob_set
                .into_iter()
                .map(|(key, blob)| Self::put(key, blob, version, &mut tree_cache))
                .collect::<Result<_>>()?;
            // Freezes the current cache to make all contents in the current cache immutable.
            tree_cache.freeze();
        }

        Ok(tree_cache.into())
    }

    fn put(
        key: HashValue,
        blob: AccountStateBlob,
        version: Version,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<()> {
        let nibble_path = NibblePath::new(key.to_vec());

        // Get the root node. If this is the first operation, it would get the root node from the
        // underlying db. Otherwise it most likely would come from `cache`.
        let root_node_key = tree_cache.get_root_node_key();
        let mut nibble_iter = nibble_path.nibbles();

        // Start insertion from the root node.
        let (new_root_node_key, _) = Self::insert_at(
            root_node_key.clone(),
            version,
            &mut nibble_iter,
            blob,
            tree_cache,
        )?;

        tree_cache.set_root_node_key(new_root_node_key);
        Ok(())
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// [`NodeKey`](node_type/struct.NodeKey.html). Returns the newly inserted node.
    /// It is safe to use recursion here because the max depth is limited by the key length which
    /// for this tree is the length of the hash of account addresses.
    fn insert_at(
        node_key: NodeKey,
        version: Version,
        nibble_iter: &mut NibbleIterator,
        blob: AccountStateBlob,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(NodeKey, Node)> {
        let node = tree_cache.get_node(&node_key)?;
        match node {
            Node::Internal(internal_node) => Self::insert_at_internal_node(
                node_key,
                internal_node,
                version,
                nibble_iter,
                blob,
                tree_cache,
            ),
            Node::Leaf(leaf_node) => Self::insert_at_leaf_node(
                node_key,
                leaf_node,
                version,
                nibble_iter,
                blob,
                tree_cache,
            ),
            Node::Null => {
                if node_key.nibble_path().num_nibbles() != 0 {
                    bail!(
                        "Null node exists for non-root node with node_key {:?}",
                        node_key
                    );
                }
                // delete the old null node if the at the same version.
                if node_key.version() == version {
                    tree_cache.delete_node(&node_key, false /* is_leaf */);
                }
                Self::create_leaf_node(
                    NodeKey::new_empty_path(version),
                    &nibble_iter,
                    blob,
                    tree_cache,
                )
            }
        }
    }

    /// Helper function for recursive insertion into the subtree that starts from the current
    /// `internal_node`. Returns the newly inserted node with its
    /// [`NodeKey`](node_type/struct.NodeKey.html).
    fn insert_at_internal_node(
        mut node_key: NodeKey,
        internal_node: InternalNode,
        version: Version,
        nibble_iter: &mut NibbleIterator,
        blob: AccountStateBlob,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(NodeKey, Node)> {
        // We always delete the existing internal node here because it will not be referenced anyway
        // since this version.
        tree_cache.delete_node(&node_key, false /* is_leaf */);

        // Find the next node to visit following the next nibble as index.
        let child_index = nibble_iter.next().expect("Ran out of nibbles");

        // Traverse downwards from this internal node recursively to get the `node_key` of the child
        // node at `child_index`.
        let (_, new_child_node) = match internal_node.child(child_index) {
            Some(child) => {
                let child_node_key = node_key.gen_child_node_key(child.version, child_index);
                Self::insert_at(child_node_key, version, nibble_iter, blob, tree_cache)?
            }
            None => {
                let new_child_node_key = node_key.gen_child_node_key(version, child_index);
                Self::create_leaf_node(new_child_node_key, nibble_iter, blob, tree_cache)?
            }
        };

        // Reuse the current `InternalNode` in memory to create a new internal node.
        let mut children: Children = internal_node.into();
        children.insert(
            child_index,
            Child::new(new_child_node.hash(), version, new_child_node.is_leaf()),
        );
        let new_internal_node = InternalNode::new(children);

        node_key.set_version(version);

        // Cache this new internal node.
        tree_cache.put_node(node_key.clone(), new_internal_node.clone().into())?;
        Ok((node_key, new_internal_node.into()))
    }

    /// Helper function for recursive insertion into the subtree that starts from the
    /// `existing_leaf_node`. Returns the newly inserted node with its
    /// [`NodeKey`](node_type/struct.NodeKey.html).
    fn insert_at_leaf_node(
        mut node_key: NodeKey,
        existing_leaf_node: LeafNode,
        version: Version,
        nibble_iter: &mut NibbleIterator,
        blob: AccountStateBlob,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(NodeKey, Node)> {
        // We are on a leaf node but trying to insert another node, so we may diverge.
        // We always delete the existing leaf node here because it will not be referenced anyway
        // since this version.
        tree_cache.delete_node(&node_key, true /* is_leaf */);

        // 1. Make sure that the existing leaf nibble_path has the same prefix as the already
        // visited part of the nibble iter of the incoming key and advances the existing leaf
        // nibble iterator by the length of that prefix.
        let mut visited_nibble_iter = nibble_iter.visited_nibbles();
        let existing_leaf_nibble_path = NibblePath::new(existing_leaf_node.account_key().to_vec());
        let mut existing_leaf_nibble_iter = existing_leaf_nibble_path.nibbles();
        skip_common_prefix(&mut visited_nibble_iter, &mut existing_leaf_nibble_iter);

        // TODO(lightmark): Change this to corrupted error.
        assert!(
            visited_nibble_iter.is_finished(),
            "Leaf nodes failed to share the same visited nibbles before index {}",
            existing_leaf_nibble_iter.visited_nibbles().num_nibbles()
        );

        // 2. Determine the extra part of the common prefix that extends from the position where
        // step 1 ends between this leaf node and the incoming key.
        let mut existing_leaf_nibble_iter_below_internal =
            existing_leaf_nibble_iter.remaining_nibbles();
        let num_common_nibbles_below_internal =
            skip_common_prefix(nibble_iter, &mut existing_leaf_nibble_iter_below_internal);
        let mut common_nibble_path = nibble_iter.visited_nibbles().collect::<NibblePath>();

        // 2.1. Both are finished. That means the incoming key already exists in the tree and we
        // just need to update its value.
        if nibble_iter.is_finished() {
            assert!(existing_leaf_nibble_iter_below_internal.is_finished());
            // The new leaf node will have the same nibble_path with a new version as node_key.
            node_key.set_version(version);
            // Create the new leaf node with the same address but new blob content.
            return Ok(Self::create_leaf_node(
                node_key,
                nibble_iter,
                blob,
                tree_cache,
            )?);
        }

        // 2.2. both are unfinished(They have keys with same length so it's impossible to have one
        // finished and the other not). This means the incoming key forks at some point between the
        // position where step 1 ends and the last nibble, inclusive. Then create a seris of
        // internal nodes the number of which equals to the length of the extra part of the
        // common prefix in step 2, a new leaf node for the incoming key, and update the
        // [`NodeKey`] of existing leaf node. We create new internal nodes in a bottom-up
        // order.
        let existing_leaf_index = existing_leaf_nibble_iter_below_internal
            .next()
            .expect("Ran out of nibbles");
        let new_leaf_index = nibble_iter.next().expect("Ran out of nibbles");
        assert_ne!(existing_leaf_index, new_leaf_index);

        let mut children = Children::new();
        children.insert(
            existing_leaf_index,
            Child::new(existing_leaf_node.hash(), version, true /* is_leaf */),
        );
        node_key = NodeKey::new(version, common_nibble_path.clone());
        tree_cache.put_node(
            node_key.gen_child_node_key(version, existing_leaf_index),
            existing_leaf_node.into(),
        )?;

        let (_, new_leaf_node) = Self::create_leaf_node(
            node_key.gen_child_node_key(version, new_leaf_index),
            nibble_iter,
            blob,
            tree_cache,
        )?;
        children.insert(
            new_leaf_index,
            Child::new(new_leaf_node.hash(), version, true /* is_leaf */),
        );

        let internal_node = InternalNode::new(children);
        let mut next_internal_node = internal_node.clone();
        tree_cache.put_node(node_key.clone(), internal_node.into())?;

        for _i in 0..num_common_nibbles_below_internal {
            let nibble = common_nibble_path
                .pop()
                .expect("Common nibble_path below internal node ran out of nibble");
            node_key = NodeKey::new(version, common_nibble_path.clone());
            let mut children = Children::new();
            children.insert(
                nibble,
                Child::new(next_internal_node.hash(), version, false /* is_leaf */),
            );
            let internal_node = InternalNode::new(children);
            next_internal_node = internal_node.clone();
            tree_cache.put_node(node_key.clone(), internal_node.into())?;
        }

        Ok((node_key, next_internal_node.into()))
    }

    /// Helper function for creating leaf nodes. Returns the newly created leaf node.
    fn create_leaf_node(
        node_key: NodeKey,
        nibble_iter: &NibbleIterator,
        blob: AccountStateBlob,
        tree_cache: &mut TreeCache<R>,
    ) -> Result<(NodeKey, Node)> {
        // Get the underlying bytes of nibble_iter which must be a key, i.e., hashed account address
        // with `HashValue::LENGTH` bytes.
        let new_leaf_node = Node::new_leaf(
            HashValue::from_slice(nibble_iter.get_nibble_path().bytes())
                .expect("LeafNode must have full nibble path."),
            blob,
        );

        tree_cache.put_node(node_key.clone(), new_leaf_node.clone())?;
        Ok((node_key, new_leaf_node))
    }

    /// Returns the account state blob (if applicable) and the corresponding merkle proof.
    pub fn get_with_proof(
        &self,
        key: HashValue,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        // Empty tree just returns proof with no sibling hash.
        let mut next_node_key = NodeKey::new_empty_path(version);
        let mut siblings = vec![];
        let nibble_path = NibblePath::new(key.to_vec());
        let mut nibble_iter = nibble_path.nibbles();

        // We limit the number of loops here deliberately to avoid potential cyclic graph bugs
        // in the tree structure.
        for nibble_depth in 0..=ROOT_NIBBLE_HEIGHT {
            let next_node = self.reader.get_node(&next_node_key)?;
            match next_node {
                Node::Internal(internal_node) => {
                    let queried_child_index = nibble_iter
                        .next()
                        .ok_or_else(|| format_err!("ran out of nibbles"))?;
                    let (child_node_key, mut siblings_in_internal) =
                        internal_node.get_child_with_siblings(&next_node_key, queried_child_index);
                    siblings.append(&mut siblings_in_internal);
                    next_node_key = match child_node_key {
                        Some(node_key) => node_key,
                        None => {
                            return Ok((
                                None,
                                SparseMerkleProof::new(None, {
                                    siblings.reverse();
                                    siblings
                                }),
                            ))
                        }
                    };
                }
                Node::Leaf(leaf_node) => {
                    return Ok((
                        if leaf_node.account_key() == key {
                            Some(leaf_node.blob().clone())
                        } else {
                            None
                        },
                        SparseMerkleProof::new(Some(leaf_node.into()), {
                            siblings.reverse();
                            siblings
                        }),
                    ));
                }
                Node::Null => {
                    if nibble_depth == 0 {
                        return Ok((None, SparseMerkleProof::new(None, vec![])));
                    } else {
                        bail!(
                            "Non-root null node exists with node key {:?}",
                            next_node_key
                        );
                    }
                }
            }
        }
        bail!("Jellyfish Merkle tree has cyclic graph inside.");
    }

    /// Gets the proof that shows a list of keys up to `rightmost_key_to_prove` exist at `version`.
    pub fn get_range_proof(
        &self,
        rightmost_key_to_prove: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        let (account, proof) = self.get_with_proof(rightmost_key_to_prove, version)?;
        ensure!(account.is_some(), "rightmost_key_to_prove must exist.");

        let siblings = proof
            .siblings()
            .iter()
            .rev()
            .zip(rightmost_key_to_prove.iter_bits())
            .filter_map(|(sibling, bit)| {
                // We only need to keep the siblings on the right.
                if !bit {
                    Some(*sibling)
                } else {
                    None
                }
            })
            .rev()
            .collect();
        Ok(SparseMerkleRangeProof::new(siblings))
    }

    #[cfg(test)]
    pub fn get(&self, key: HashValue, version: Version) -> Result<Option<AccountStateBlob>> {
        Ok(self.get_with_proof(key, version)?.0)
    }

    pub fn get_root_hash(&self, version: Version) -> Result<HashValue> {
        self.get_root_hash_option(version)?
            .ok_or_else(|| format_err!("Root node not found for version {}.", version))
    }

    pub fn get_root_hash_option(&self, version: Version) -> Result<Option<HashValue>> {
        let root_node_key = NodeKey::new_empty_path(version);
        Ok(self
            .reader
            .get_node_option(&root_node_key)?
            .map(|root_node| root_node.hash()))
    }
}
