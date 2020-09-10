// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use libra_types::proof::definition::LeafCount;
use proptest::{collection::vec, prelude::*};

#[test]
fn test_append_empty_on_empty() {
    let store = MockHashStore::new();
    assert_eq!(
        TestAccumulator::append(&store, 0, &[]).unwrap(),
        (*ACCUMULATOR_PLACEHOLDER_HASH, Vec::new())
    );
}

#[test]
fn test_append_one() {
    let mut store = MockHashStore::new();
    store.verify(&[]).unwrap();

    let mut leaves = Vec::new();
    for _ in 0..100 {
        let hash = HashValue::random();
        let (root_hash, writes) =
            TestAccumulator::append(&store, leaves.len() as LeafCount, &[hash]).unwrap();
        store.put_many(&writes);

        leaves.push(hash);
        let expected_root_hash = store.verify(&leaves).unwrap();

        assert_eq!(root_hash, expected_root_hash)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_append_many(batches in vec(vec(any::<HashValue>(), 10), 10)) {
        let mut store = MockHashStore::new();

        let mut leaves: Vec<HashValue> = Vec::new();
        let mut num_leaves = 0;
        for hashes in batches.iter() {
            let (root_hash, writes) =
                TestAccumulator::append(&store, num_leaves, &hashes).unwrap();
            store.put_many(&writes);

            num_leaves += hashes.len() as LeafCount;
            leaves.extend(hashes.iter());
            let expected_root_hash = store.verify(&leaves).unwrap();
            assert_eq!(root_hash, expected_root_hash)
        }
    }

    #[test]
    fn test_append_empty(leaves in vec(any::<HashValue>(), 100)) {
        let mut store = MockHashStore::new();

        let (root_hash, writes) = TestAccumulator::append(&store, 0, &leaves).unwrap();
        store.put_many(&writes);

        let (root_hash2, writes2) =
            TestAccumulator::append(&store, leaves.len() as LeafCount, &[]).unwrap();

        assert_eq!(root_hash, root_hash2);
        assert!(writes2.is_empty());
    }
}
