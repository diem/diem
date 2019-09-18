// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the functionality to restore a `JellyfishMerkleTree` from small chunks
//! of accounts.

#[cfg(test)]
mod restore_test;

use crate::{
    nibble::NibblePath,
    node_type::{Child, Children, InternalNode, LeafNode, Node, NodeKey},
    NodeBatch, TreeReader, TreeWriter, ROOT_NIBBLE_HEIGHT,
};
use crypto::{hash::CryptoHash, HashValue};
use failure::prelude::*;
use types::{account_state_blob::AccountStateBlob, transaction::Version};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ChildInfo {
    /// This child is an internal node. The hash of the internal node is stored here if it is
    /// known, otherwise it is `None`. In the process of restoring a tree, we will only know the
    /// hash of an internal node after we see all the keys that share the same prefix.
    Internal { hash: Option<HashValue> },

    /// This child is a leaf node.
    Leaf { node: LeafNode },
}

#[derive(Clone, Debug)]
struct InternalInfo {
    /// The node key of this internal node.
    node_key: NodeKey,

    /// The existing children. Every time a child appears, the corresponding position will be set
    /// to `Some`.
    children: [Option<ChildInfo>; 16],
}

impl InternalInfo {
    /// Creates an internal node whose children are all unknown.
    fn new_uninitialized(node_key: NodeKey) -> Self {
        Self {
            node_key,
            children: Default::default(),
        }
    }

    fn set_child(&mut self, index: usize, child_info: ChildInfo) {
        self.children[index] = Some(child_info);
    }

    /// Converts `self` to an internal node, assuming all of its children are already known and
    /// fully initialized.
    fn into_internal_node(self, version: Version) -> (NodeKey, InternalNode) {
        let mut children = Children::new();

        // Calling `into_iter` on an array is equivalent to calling `iter`:
        // https://github.com/rust-lang/rust/issues/25725.
        for (index, child_info_option) in self.children.iter().enumerate() {
            if let Some(child_info) = child_info_option {
                let child = match child_info {
                    ChildInfo::Internal { hash } => {
                        Child::new(
                            hash.as_ref().copied().expect("Must have been initialized."),
                            version,
                            false, /* is_leaf */
                        )
                    }
                    ChildInfo::Leaf { node } => {
                        Child::new(node.hash(), version, true /* is_leaf */)
                    }
                };
                children.insert((index as u8).into(), child);
            }
        }

        (self.node_key, InternalNode::new(children))
    }
}

pub struct JellyfishMerkleRestore<'a, S> {
    /// The underlying storage.
    store: &'a S,

    /// The version of the tree we are restoring.
    version: Version,

    /// The nodes we have partially restored. Each `partial_nodes[i-1]` is the parent of
    /// `partial_nodes[i]`. If a node `partial_nodes[i-1]` has multiple children, only the
    /// rightmost known child will appear here as `partial_nodes[i]`, because any other children on
    /// the left would have been frozen.
    ///
    /// At any point in time, the structure looks like the following:
    ///
    /// ```text
    /// +----+----+----+----+----+----+----+----+
    /// |    |    |    |    |    |    |    | C  |  partial_nodes[0]
    /// +----+----+----+----+----+----+----+----+
    ///   |         |              |
    ///   |         |              |
    ///   |         |              |
    ///   v         v              v
    /// Frozen    Frozen     +----+----+----+----+----+----+----+----+
    ///                      |    |    |    | B  |    |    | A  |    |  partial_nodes[1]
    ///                      +----+----+----+----+----+----+----+----+
    ///                             |         |
    ///                             |         |
    ///                             |         |
    ///                             v         v
    ///                            Frozen    Previously inserted account
    /// ```
    ///
    /// We insert the accounts from left to right. So if the next account appears at position `A`,
    /// it will cause the leaf at position `B` to be frozen. If it appears at position `B`, it
    /// might cause a few internal nodes to be created additionally. If it appears at position `C`,
    /// it will also cause `partial_nodes[1]` to be added to `frozen_nodes` as an internal node and
    /// be removed from `partial_nodes`.
    partial_nodes: Vec<InternalInfo>,

    /// The nodes that have been fully restored and are ready to be written to storage.
    frozen_nodes: NodeBatch,

    /// The most recently added key. With this we are able to ensure the keys come in increasing
    /// order.
    previous_key: Option<HashValue>,
}

impl<'a, S> JellyfishMerkleRestore<'a, S>
where
    S: 'a + TreeReader + TreeWriter,
{
    pub fn new(store: &'a S, version: Version) -> Result<Self> {
        let partial_nodes = match store.get_rightmost_leaf()? {
            Some((node_key, _leaf_node)) => {
                // If the system crashed in the middle of the previous restoration attempt, we need
                // to recover the partial nodes to the state right before the crash.
                Self::recover_partial_nodes(store, version, node_key)?
            }
            None => {
                // If no rightmost leaf exists, it means this is the first time we start and
                // storage is still empty. We use a single root node in this case.
                vec![InternalInfo::new_uninitialized(NodeKey::new_empty_path(
                    version,
                ))]
            }
        };

        Ok(Self {
            store,
            version,
            partial_nodes,
            frozen_nodes: NodeBatch::new(),
            previous_key: None,
        })
    }

    /// Recovers partial nodes from storage. We do this by looking at all the ancestors of the
    /// rightmost leaf. The ones do not exist in storage are the partial nodes.
    fn recover_partial_nodes(
        store: &'a S,
        version: Version,
        rightmost_leaf_node_key: NodeKey,
    ) -> Result<Vec<InternalInfo>> {
        ensure!(
            rightmost_leaf_node_key.nibble_path().num_nibbles() > 0,
            "Root node would not be written until entire restoration process has completed \
             successfully.",
        );

        // Start from the parent of the rightmost leaf. If this internal node exists in storage, it
        // is not a partial node. Go to the parent node and repeat until we see a node that does
        // not exist. This node and all its ancestors will be the partial nodes.
        let mut node_key = rightmost_leaf_node_key.gen_parent_node_key();
        while store.get_node_option(&node_key)?.is_some() {
            node_key = node_key.gen_parent_node_key();
        }

        // Next we reconstruct all the partial nodes up to the root node.
        let mut partial_nodes = vec![];
        let mut previous_child_index = None;

        loop {
            let mut internal_info = InternalInfo::new_uninitialized(node_key.clone());

            // Scan all its possible children and see if they exist in storage. We need to do this
            // from index 0 to the previous child index. For the bottom node we just try to find
            // all its children.
            for i in 0..previous_child_index.unwrap_or(16) {
                let child_node_key = node_key.gen_child_node_key(version, (i as u8).into());
                if let Some(node) = store.get_node_option(&child_node_key)? {
                    let child_info = match node {
                        Node::Internal(internal_node) => ChildInfo::Internal {
                            hash: Some(internal_node.hash()),
                        },
                        Node::Leaf(leaf_node) => ChildInfo::Leaf { node: leaf_node },
                        Node::Null => bail!("Null node should not appear in storage."),
                    };
                    internal_info.set_child(i, child_info);
                }
            }

            if let Some(index) = previous_child_index {
                // Set the hash of this child to `None` because it is a partial node and we do not
                // know its hash yet.
                internal_info.set_child(index, ChildInfo::Internal { hash: None });
            }

            partial_nodes.push(internal_info);
            if node_key.nibble_path().num_nibbles() == 0 {
                break;
            }
            previous_child_index = node_key.nibble_path().last().map(|x| u8::from(x) as usize);
            node_key = node_key.gen_parent_node_key();
        }

        partial_nodes.reverse();
        Ok(partial_nodes)
    }

    /// Restores a chunk of accounts. This function assumes that the given chunk has been validated
    /// and comes in the correct order.
    pub fn add_chunk(&mut self, chunk: Vec<(HashValue, AccountStateBlob)>) -> Result<()> {
        for (key, value) in chunk {
            if let Some(ref prev_key) = self.previous_key {
                ensure!(
                    key > *prev_key,
                    "Account keys must come in increasing order.",
                )
            }
            self.add_one(key, value)?;
            self.previous_key.replace(key);
        }

        // Write the frozen nodes to storage.
        self.store.write_node_batch(self.frozen_nodes.clone())?;
        self.frozen_nodes.clear();

        Ok(())
    }

    /// Restores one account.
    fn add_one(&mut self, new_key: HashValue, new_value: AccountStateBlob) -> Result<()> {
        let nibble_path = NibblePath::new(new_key.to_vec());
        let mut nibbles = nibble_path.nibbles();

        for i in 0..ROOT_NIBBLE_HEIGHT {
            let child_index = u8::from(nibbles.next().expect("This nibble must exist.")) as usize;

            match self.partial_nodes[i].children[child_index] {
                Some(ref child_info) => {
                    // If the next node is an internal node, we just continue the loop with the
                    // next nibble. Here we deal with the leaf case. We may need to insert multiple
                    // internal nodes depending on the length of the common prefix of the existing
                    // key and the new key.
                    if let ChildInfo::Leaf { node } = child_info {
                        assert_eq!(i, self.partial_nodes.len() - 1);

                        let existing_leaf = node.clone();

                        // The node at this position becomes an internal node. Since we may insert
                        // more nodes at this position in the future, we do not know its hash yet.
                        self.partial_nodes[i]
                            .set_child(child_index, ChildInfo::Internal { hash: None });

                        let common_prefix_len =
                            existing_leaf.account_key().common_prefix_bits_len(new_key) / 4;

                        // All these internal node will now have a single internal node child.
                        for _ in i + 1..common_prefix_len {
                            let visited_nibbles = nibbles.visited_nibbles().collect();
                            let next_nibble = nibbles.next().expect("This nibble must exist.");
                            let new_node_key = NodeKey::new(self.version, visited_nibbles);

                            let mut internal_info = InternalInfo::new_uninitialized(new_node_key);
                            internal_info.set_child(
                                u8::from(next_nibble) as usize,
                                ChildInfo::Internal { hash: None },
                            );
                            self.partial_nodes.push(internal_info);
                        }

                        // The last internal node will have two leaf node children.
                        let visited_nibbles = nibbles.visited_nibbles().collect();
                        let new_node_key = NodeKey::new(self.version, visited_nibbles);
                        let mut internal_info = InternalInfo::new_uninitialized(new_node_key);

                        // Next we put the existing leaf as a child of this internal node.
                        let existing_child_index =
                            existing_leaf.account_key().get_nibble(common_prefix_len);
                        internal_info.set_child(
                            existing_child_index as usize,
                            ChildInfo::Leaf {
                                node: existing_leaf,
                            },
                        );

                        // Do not set the new child for now. We always call `freeze` first, then
                        // set the new child later, because this way it's easier in `freeze` to
                        // find the right leaf -- it's always the rightmost leaf on the lowest
                        // level.
                        self.partial_nodes.push(internal_info);
                        self.freeze(self.partial_nodes.len());

                        // Now we set the new child.
                        let new_child_index = new_key.get_nibble(common_prefix_len);
                        assert!(
                            new_child_index > existing_child_index,
                            "New leaf must be on the right.",
                        );
                        self.partial_nodes
                            .last_mut()
                            .expect("This node must exist.")
                            .set_child(
                                new_child_index as usize,
                                ChildInfo::Leaf {
                                    node: LeafNode::new(new_key, new_value),
                                },
                            );

                        break;
                    }
                }
                None => {
                    // This means that we are going to put a leaf in this position. For all the
                    // descendants on the left, they are now frozen.
                    self.freeze(i + 1);

                    // Mark this position as a leaf child.
                    self.partial_nodes[i].set_child(
                        child_index,
                        ChildInfo::Leaf {
                            node: LeafNode::new(new_key, new_value),
                        },
                    );

                    // We do not add this leaf node to self.frozen_nodes because we don't know its
                    // node key yet. We will know its node key when the next account comes.
                    break;
                }
            }
        }

        Ok(())
    }

    /// Puts the nodes that will not be changed in `self.frozen_nodes`.
    fn freeze(&mut self, target_len: usize) {
        // Freeze the rightmost leaf node on the lowest level. This node was inserted in the
        // previous `restore_one` call.
        let last_node = self
            .partial_nodes
            .last()
            .expect("Must have at least one partial node.");
        for i in (0..16).rev() {
            if let Some(ref child_info) = last_node.children[i] {
                if let ChildInfo::Leaf { node } = child_info {
                    let child_node_key = last_node
                        .node_key
                        .gen_child_node_key(self.version, (i as u8).into());
                    self.frozen_nodes
                        .insert(child_node_key, node.clone().into());
                    break;
                }
            }
        }

        // Freeze extra internal nodes.
        while self.partial_nodes.len() > target_len {
            let last_node = self.partial_nodes.pop().expect("This node must exist.");
            let (node_key, internal_node) = last_node.into_internal_node(self.version);
            let node_hash = internal_node.hash();
            self.frozen_nodes.insert(node_key, internal_node.into());

            // Now that we have computed the hash of the internal node above, we will also update
            // its parent unless it is root node.
            if let Some(parent_node) = self.partial_nodes.last_mut() {
                // This internal node must be the rightmost child of its parent at the moment.
                for i in (0..16).rev() {
                    if let Some(ref mut child_info) = parent_node.children[i] {
                        match child_info {
                            ChildInfo::Internal { ref mut hash } => {
                                assert_eq!(hash.replace(node_hash), None);
                            }
                            ChildInfo::Leaf { .. } => {
                                panic!("The rightmost child must not be a leaf.");
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Finishes the restoration process. This tells the code that there is no more account,
    /// otherwise we can not freeze the rightmost leaf and its ancestors.
    pub fn finish(mut self) -> Result<()> {
        // Deal with the special case when the entire tree has a single leaf.
        if self.partial_nodes.len() == 1 {
            let mut num_children = 0;
            let mut leaf = None;
            for i in 0..16 {
                if let Some(ref child_info) = self.partial_nodes[0].children[i] {
                    num_children += 1;
                    if let ChildInfo::Leaf { node } = child_info {
                        leaf = Some(node.clone());
                    }
                }
            }

            if num_children == 1 {
                if let Some(node) = leaf {
                    let node_key = NodeKey::new_empty_path(self.version);
                    assert!(self.frozen_nodes.is_empty());
                    self.frozen_nodes.insert(node_key, node.into());
                    self.store.write_node_batch(self.frozen_nodes)?;
                    return Ok(());
                }
            }
        }

        self.freeze(0);
        self.store.write_node_batch(self.frozen_nodes)
    }
}
