// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::test_helpers::{
    arb_hash_batch, arb_list_of_hash_batches, test_append_empty_impl, test_append_many_impl,
    MockHashStore, TestAccumulator,
};
use diem_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use diem_types::proof::definition::LeafCount;

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
    for v in 0..100 {
        let hash = HashValue::random();
        let (root_hash, writes) =
            TestAccumulator::append(&store, leaves.len() as LeafCount, &[hash]).unwrap();
        store.put_many(&writes);

        leaves.push(hash);
        let expected_root_hash = store.verify(&leaves).unwrap();

        assert_eq!(root_hash, expected_root_hash);
        assert_eq!(
            TestAccumulator::get_root_hash(&store, v + 1).unwrap(),
            expected_root_hash
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_append_many(batches in arb_list_of_hash_batches(10, 10)) {
        test_append_many_impl(batches);
    }

    #[test]
    fn test_append_empty(leaves in arb_hash_batch(100)) {
        test_append_empty_impl(leaves)
    }
}
