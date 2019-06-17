// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::{collection::vec, prelude::*};
use types::proof::verify_test_accumulator_element;

#[test]
fn test_error_on_bad_parameters() {
    let store = MockHashStore::new();
    assert!(TestAccumulator::get_proof(&store, 0, 0).is_err());
    assert!(TestAccumulator::get_proof(&store, 100, 101).is_err());
}

#[test]
fn test_one_leaf() {
    let hash = HashValue::random();
    let mut store = MockHashStore::new();
    let (root_hash, writes) = TestAccumulator::append(&store, 0, &[hash]).unwrap();
    store.put_many(&writes);

    verify(&store, 1, root_hash, &[hash], 0)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_proof(
        batch1 in vec(any::<HashValue>(), 1..100),
        batch2 in vec(any::<HashValue>(), 1..100),
    ) {
        let total_leaves = batch1.len() + batch2.len();
        let batch1_size = batch1.len() as u64;
        let mut store = MockHashStore::new();

        // insert all leaves in two batches
        let (root_hash1, writes1) = TestAccumulator::append(&store, 0, &batch1).unwrap();
        store.put_many(&writes1);
        let (root_hash2, writes2) = TestAccumulator::append(&store, batch1_size, &batch2).unwrap();
        store.put_many(&writes2);

        // verify proofs for all leaves towards current root
        verify(&store, total_leaves, root_hash2, &batch1, 0);
        verify(&store, total_leaves, root_hash2, &batch2, batch1_size);

        // verify proofs for all leaves of a subtree towards subtree root
        verify(&store, batch1.len(), root_hash1, &batch1, 0);
    }
}

fn verify(
    store: &MockHashStore,
    num_leaves: usize,
    root_hash: HashValue,
    leaves: &[HashValue],
    first_leaf_idx: u64,
) {
    leaves.iter().enumerate().for_each(|(i, hash)| {
        let leaf_index = first_leaf_idx + i as u64;
        let proof = TestAccumulator::get_proof(store, num_leaves as u64, leaf_index).unwrap();
        verify_test_accumulator_element(root_hash, *hash, leaf_index, &proof).unwrap();
    });
}
