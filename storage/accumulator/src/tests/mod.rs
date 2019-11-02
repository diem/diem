// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod proof_test;
mod write_test;

use super::*;
use libra_crypto::hash::TestOnlyHasher;
use libra_types::proof::definition::LeafCount;
use proptest::{collection::vec, prelude::*};
use std::collections::HashMap;

type InMemoryAccumulator = libra_types::proof::accumulator::InMemoryAccumulator<TestOnlyHasher>;
type TestAccumulator = MerkleAccumulator<MockHashStore, TestOnlyHasher>;

struct MockHashStore {
    store: HashMap<Position, HashValue>,
}

impl HashReader for MockHashStore {
    fn get(&self, position: Position) -> Result<HashValue> {
        self.store
            .get(&position)
            .cloned()
            .ok_or_else(|| format_err!("Position {:?} absent.", position))
    }
}

struct Subtree {
    root_position: u64,
    max_position: u64,
    hash: HashValue,
    frozen: bool,
    num_frozen_nodes: LeafCount,
}

impl MockHashStore {
    pub fn new() -> Self {
        MockHashStore {
            store: HashMap::new(),
        }
    }

    pub fn put_many(&mut self, writes: &[(Position, HashValue)]) {
        self.store.extend(writes.iter().cloned())
    }

    fn verify_in_store_if_frozen(
        &self,
        position: u64,
        hash: HashValue,
        frozen: bool,
    ) -> Result<()> {
        if frozen {
            ensure!(
                hash == *self
                    .store
                    .get(&Position::from_inorder_index(position))
                    .ok_or_else(|| format_err!("unexpected None at position {}", position))?,
                "Hash mismatch for node at position {}",
                position
            );
        }
        Ok(())
    }

    /// Return the placeholder hash if left and right hashes both are the placeholder hash,
    /// otherwise delegate to `TestAccumulatorView::hash_internal_node`.
    ///
    /// This is needed because a real Accumulator subtree with only placeholder nodes are trimmed
    /// (never generated nor accessed).
    fn hash_internal(left: HashValue, right: HashValue) -> HashValue {
        if left == *ACCUMULATOR_PLACEHOLDER_HASH && right == *ACCUMULATOR_PLACEHOLDER_HASH {
            *ACCUMULATOR_PLACEHOLDER_HASH
        } else {
            MerkleAccumulatorView::<MockHashStore, TestOnlyHasher>::hash_internal_node(left, right)
        }
    }

    fn verify_subtree(&self, leaves: &[HashValue], min_position: u64) -> Result<Subtree> {
        assert!(leaves.len().is_power_of_two());

        let me = if leaves.len() == 1 {
            // leaf
            let root_position = min_position;
            let max_position = min_position;
            let hash = leaves[0];
            let frozen = hash != *ACCUMULATOR_PLACEHOLDER_HASH;
            let num_frozen_nodes = frozen as LeafCount;

            Subtree {
                root_position,
                max_position,
                hash,
                frozen,
                num_frozen_nodes,
            }
        } else {
            let subtree_width = leaves.len() / 2;

            let left = self.verify_subtree(&leaves[..subtree_width], min_position)?;
            let root_position = left.max_position + 1;
            let right = self.verify_subtree(&leaves[subtree_width..], root_position + 1)?;

            let max_position = right.max_position;
            let hash = Self::hash_internal(left.hash, right.hash);
            let frozen = left.frozen && right.frozen;
            let num_frozen_nodes =
                left.num_frozen_nodes + right.num_frozen_nodes + frozen as LeafCount;

            Subtree {
                root_position,
                max_position,
                hash,
                frozen,
                num_frozen_nodes,
            }
        };

        self.verify_in_store_if_frozen(me.root_position, me.hash, me.frozen)?;

        Ok(me)
    }

    /// (Naively) Verify `self.store` has in it nodes that represent an accumulator of `leaves` and
    /// only those nodes.
    ///
    /// 1. expand the accumulator tree to a virtual full binary tree by adding placeholder nodes
    /// 2. recursively:
    ///     a. in-order number each node, call it "position"
    ///     b. calculate internal node hash out of its children
    ///     c. sum up frozen nodes
    ///     d. verify frozen nodes are in store at the above mentioned "position"
    /// 4. verify number of nodes in store matches exactly number of frozen nodes.
    fn verify(&self, leaves: &[HashValue]) -> Result<HashValue> {
        if leaves.is_empty() {
            ensure!(self.store.is_empty(), "non-empty store for empty tree.");
            Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
        } else {
            // pad `leaves` with dummies, to form a full tree
            let mut full_tree_leaves = leaves.to_vec();
            full_tree_leaves.resize(
                leaves.len().next_power_of_two(),
                *ACCUMULATOR_PLACEHOLDER_HASH,
            );

            let tree = self.verify_subtree(&full_tree_leaves, 0)?;

            ensure!(
                self.store.len() as LeafCount == tree.num_frozen_nodes,
                "mismatch: items in store - {} vs expect num of frozen nodes - {}",
                self.store.len(),
                tree.num_frozen_nodes,
            );
            Ok(tree.hash)
        }
    }
}

proptest! {
    #[test]
    fn test_get_frozen_subtree_hashes(leaves in vec(any::<HashValue>(), 0..1000)) {
        let mut store = MockHashStore::new();
        let (root_hash, writes) = TestAccumulator::append(&store, 0, &leaves).unwrap();
        store.put_many(&writes);

        let frozen_subtree_hashes =
            TestAccumulator::get_frozen_subtree_hashes(&store, leaves.len() as LeafCount).unwrap();
        let in_mem_acc =
            InMemoryAccumulator::new(frozen_subtree_hashes, leaves.len() as LeafCount).unwrap();
        prop_assert_eq!(root_hash, in_mem_acc.root_hash());
    }
}
