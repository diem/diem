// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{HashReader, MerkleAccumulator, MerkleAccumulatorView};
use anyhow::{ensure, format_err, Result};
use diem_crypto::hash::{HashValue, TestOnlyHasher, ACCUMULATOR_PLACEHOLDER_HASH};
use diem_types::proof::{definition::LeafCount, position::Position};
use proptest::{collection::vec, prelude::*};
use std::collections::HashMap;

pub(crate) type InMemoryAccumulator =
    diem_types::proof::accumulator::InMemoryAccumulator<TestOnlyHasher>;
pub(crate) type TestAccumulator = MerkleAccumulator<MockHashStore, TestOnlyHasher>;

pub(crate) struct MockHashStore {
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
    pub fn verify(&self, leaves: &[HashValue]) -> Result<HashValue> {
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

pub(crate) fn verify(
    store: &MockHashStore,
    num_leaves: u64,
    root_hash: HashValue,
    leaves: &[HashValue],
    first_leaf_idx: u64,
) {
    leaves.iter().enumerate().for_each(|(i, hash)| {
        let leaf_index = first_leaf_idx + i as u64;
        let proof = TestAccumulator::get_proof(store, num_leaves, leaf_index).unwrap();
        proof.verify(root_hash, *hash, leaf_index).unwrap();
    });
}

prop_compose! {
    pub fn arb_two_hash_batches(length: usize)(
        batch1 in vec(any::<HashValue>(), 1..length),
        batch2 in vec(any::<HashValue>(), 1..length)
    ) -> (Vec<HashValue>, Vec<HashValue>) {
        (batch1, batch2)
    }
}

prop_compose! {
    pub fn arb_three_hash_batches(length: usize)(
        batch1 in vec(any::<HashValue>(), 1..length),
        batch2 in vec(any::<HashValue>(), 1..length),
        batch3 in vec(any::<HashValue>(), 1..length)
    ) -> (Vec<HashValue>, Vec<HashValue>, Vec<HashValue>) {
        (batch1, batch2, batch3)
    }
}

pub fn test_proof_impl((batch1, batch2): (Vec<HashValue>, Vec<HashValue>)) {
    let total_leaves = batch1.len() + batch2.len();
    let mut store = MockHashStore::new();

    // insert all leaves in two batches
    let (root_hash1, writes1) = TestAccumulator::append(&store, 0, &batch1).unwrap();
    store.put_many(&writes1);
    let (root_hash2, writes2) =
        TestAccumulator::append(&store, batch1.len() as LeafCount, &batch2).unwrap();
    store.put_many(&writes2);

    // verify proofs for all leaves towards current root
    verify(&store, total_leaves as u64, root_hash2, &batch1, 0);
    verify(
        &store,
        total_leaves as u64,
        root_hash2,
        &batch2,
        batch1.len() as u64,
    );

    // verify proofs for all leaves of a subtree towards subtree root
    verify(&store, batch1.len() as u64, root_hash1, &batch1, 0);
}

pub fn test_consistency_proof_impl((batch1, batch2): (Vec<HashValue>, Vec<HashValue>)) {
    let mut store = MockHashStore::new();
    let empty_in_mem_acc = InMemoryAccumulator::default();

    let (root_hash1, writes1) = TestAccumulator::append(&store, 0, &batch1).unwrap();
    store.put_many(&writes1);
    let proof1 =
        TestAccumulator::get_consistency_proof(&store, batch1.len() as LeafCount, 0).unwrap();
    let in_mem_acc1 = empty_in_mem_acc
        .append_subtrees(proof1.subtrees(), batch1.len() as LeafCount)
        .unwrap();
    assert_eq!(root_hash1, in_mem_acc1.root_hash());

    let (root_hash2, writes2) =
        TestAccumulator::append(&store, batch1.len() as LeafCount, &batch2).unwrap();
    store.put_many(&writes2);
    let proof2 = TestAccumulator::get_consistency_proof(
        &store,
        (batch1.len() + batch2.len()) as LeafCount,
        batch1.len() as LeafCount,
    )
    .unwrap();
    let in_mem_acc2 = in_mem_acc1
        .append_subtrees(proof2.subtrees(), batch2.len() as LeafCount)
        .unwrap();
    assert_eq!(root_hash2, in_mem_acc2.root_hash());
}

pub fn test_range_proof_impl(
    (batch1, batch2, batch3): (Vec<HashValue>, Vec<HashValue>, Vec<HashValue>),
) {
    let mut store = MockHashStore::new();

    let mut all_hashes = vec![];
    all_hashes.extend_from_slice(&batch1);
    all_hashes.extend_from_slice(&batch2);
    all_hashes.extend_from_slice(&batch3);

    let (root_hash, writes) = TestAccumulator::append(&store, 0, &all_hashes).unwrap();
    store.put_many(&writes);

    let first_leaf_index = if !batch2.is_empty() {
        Some(batch1.len() as u64)
    } else {
        None
    };
    let proof = TestAccumulator::get_range_proof(
        &store,
        all_hashes.len() as LeafCount,
        first_leaf_index,
        batch2.len() as LeafCount,
    )
    .unwrap();
    proof.verify(root_hash, first_leaf_index, &batch2).unwrap();
}

prop_compose! {
    pub fn arb_hash_batch(length: usize)(
        batch in vec(any::<HashValue>(), 0..length),
    ) -> Vec<HashValue> {
        batch
    }
}

pub fn test_get_frozen_subtree_hashes_impl(leaves: Vec<HashValue>) {
    let mut store = MockHashStore::new();
    let (root_hash, writes) = TestAccumulator::append(&store, 0, &leaves).unwrap();
    store.put_many(&writes);

    let frozen_subtree_hashes =
        TestAccumulator::get_frozen_subtree_hashes(&store, leaves.len() as LeafCount).unwrap();
    let in_mem_acc =
        InMemoryAccumulator::new(frozen_subtree_hashes, leaves.len() as LeafCount).unwrap();
    assert_eq!(root_hash, in_mem_acc.root_hash());
}

prop_compose! {
    pub fn arb_list_of_hash_batches(each_batch_size: usize, num_batches: usize)(
        batches in vec(vec(any::<HashValue>(), each_batch_size), num_batches)
    ) -> Vec<Vec<HashValue>> {
        batches
    }
}

pub fn test_append_many_impl(batches: Vec<Vec<HashValue>>) {
    let mut store = MockHashStore::new();

    let mut leaves: Vec<HashValue> = Vec::new();
    let mut num_leaves = 0;
    for hashes in batches.iter() {
        let (root_hash, writes) = TestAccumulator::append(&store, num_leaves, &hashes).unwrap();
        store.put_many(&writes);

        num_leaves += hashes.len() as LeafCount;
        leaves.extend(hashes.iter());
        let expected_root_hash = store.verify(&leaves).unwrap();
        assert_eq!(root_hash, expected_root_hash);
        assert_eq!(
            TestAccumulator::get_root_hash(&store, num_leaves).unwrap(),
            expected_root_hash
        );
    }
}

pub fn test_append_empty_impl(leaves: Vec<HashValue>) {
    let mut store = MockHashStore::new();

    let (root_hash, writes) = TestAccumulator::append(&store, 0, &leaves).unwrap();
    store.put_many(&writes);

    let (root_hash2, writes2) =
        TestAccumulator::append(&store, leaves.len() as LeafCount, &[]).unwrap();

    assert_eq!(root_hash, root_hash2);
    assert!(writes2.is_empty());
}
