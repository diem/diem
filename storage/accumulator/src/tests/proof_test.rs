// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::test_helpers::{
    arb_three_hash_batches, arb_two_hash_batches, test_consistency_proof_impl, test_proof_impl,
    test_range_proof_impl, verify, MockHashStore, TestAccumulator,
};

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
    fn test_proof((batch1, batch2) in arb_two_hash_batches(100)) {
        test_proof_impl((batch1, batch2));
    }

    #[test]
    fn test_consistency_proof((batch1, batch2) in arb_two_hash_batches(100)) {
        test_consistency_proof_impl((batch1, batch2));
    }

    #[test]
    fn test_range_proof((batch1, batch2, batch3) in arb_three_hash_batches(100)) {
        test_range_proof_impl((batch1, batch2, batch3));
    }
}
