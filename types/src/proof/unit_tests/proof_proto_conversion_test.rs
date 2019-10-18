// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::{
    definition::bitmap::{AccumulatorBitmap, SparseMerkleBitmap},
    AccountStateProof, AccumulatorConsistencyProof, EventProof, SignedTransactionProof,
    SparseMerkleProof, TestAccumulatorProof,
};
use libra_crypto::{
    hash::{TestOnlyHash, ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::{collection::vec, prelude::*};
use std::convert::TryFrom;

fn accumulator_bitmap_iterator_test(bitmap_value: u64, expected_bits: Vec<bool>) {
    let bitmap = AccumulatorBitmap::new(bitmap_value);
    let bits: Vec<_> = bitmap.iter().collect();
    assert_eq!(bits, expected_bits);
    let bitmap2: AccumulatorBitmap = bits.into_iter().collect();
    let bitmap_value2: u64 = bitmap2.into();
    assert_eq!(bitmap_value, bitmap_value2);
}

#[test]
fn test_accumulator_bitmap() {
    accumulator_bitmap_iterator_test(0b0, vec![]);
    accumulator_bitmap_iterator_test(0b1, vec![true]);
    accumulator_bitmap_iterator_test(0b10_1101, vec![true, false, true, true, false, true]);
}

fn sparse_merkle_bitmap_iterator_test(bitmap_value: Vec<u8>, expected_bits: Vec<bool>) {
    let bitmap = SparseMerkleBitmap::new(bitmap_value.clone());
    let bits: Vec<_> = bitmap.iter().collect();
    assert_eq!(bits, expected_bits);
    let bitmap2: SparseMerkleBitmap = bits.into_iter().collect();
    let bitmap_value2: Vec<_> = bitmap2.into();
    assert_eq!(bitmap_value, bitmap_value2);
}

#[test]
fn test_sparse_merkle_bitmap() {
    sparse_merkle_bitmap_iterator_test(vec![], vec![]);
    sparse_merkle_bitmap_iterator_test(vec![0b1000_0000], vec![true]);
    sparse_merkle_bitmap_iterator_test(vec![0b0100_0000], vec![false, true]);
    sparse_merkle_bitmap_iterator_test(vec![0b1001_0000], vec![true, false, false, true]);
    sparse_merkle_bitmap_iterator_test(
        vec![0b0001_0011],
        vec![false, false, false, true, false, false, true, true],
    );
    sparse_merkle_bitmap_iterator_test(
        vec![0b0001_0011, 0b0010_0000],
        vec![
            false, false, false, true, false, false, true, true, false, false, true,
        ],
    );
    sparse_merkle_bitmap_iterator_test(
        vec![0b1001_0011, 0b0010_0011],
        vec![
            true, false, false, true, false, false, true, true, false, false, true, false, false,
            false, true, true,
        ],
    );
}

fn accumulator_proof_protobuf_conversion_test(
    siblings: Vec<HashValue>,
    expected_bitmap: u64,
    expected_num_non_default_siblings: usize,
) {
    let proof = TestAccumulatorProof::new(siblings);
    let compressed_proof: crate::proto::types::AccumulatorProof = proof.clone().into();
    assert_eq!(compressed_proof.bitmap, expected_bitmap);
    assert_eq!(
        compressed_proof.non_default_siblings.len(),
        expected_num_non_default_siblings
    );
    let decompressed_proof = TestAccumulatorProof::try_from(compressed_proof).unwrap();
    assert_eq!(decompressed_proof, proof);
}

#[test]
fn test_convert_accumulator_proof_to_protobuf() {
    accumulator_proof_protobuf_conversion_test(vec![], 0b0, 0);
    accumulator_proof_protobuf_conversion_test(vec![b"0".test_only_hash()], 0b1, 1);
    accumulator_proof_protobuf_conversion_test(
        vec![
            b"0".test_only_hash(),
            b"1".test_only_hash(),
            b"2".test_only_hash(),
        ],
        0b111,
        3,
    );
    accumulator_proof_protobuf_conversion_test(
        vec![
            b"0".test_only_hash(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            b"2".test_only_hash(),
        ],
        0b101,
        2,
    );
    accumulator_proof_protobuf_conversion_test(
        vec![
            b"0".test_only_hash(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            *ACCUMULATOR_PLACEHOLDER_HASH,
        ],
        0b100,
        1,
    );
}

#[test]
fn test_convert_accumulator_proof_wrong_number_of_siblings() {
    let sibling0 = b"0".test_only_hash();
    let sibling1 = b"1".test_only_hash();

    let mut compressed_proof = crate::proto::types::AccumulatorProof::default();
    compressed_proof.bitmap = 0b100;
    compressed_proof
        .non_default_siblings
        .push(sibling0.to_vec());
    compressed_proof
        .non_default_siblings
        .push(sibling1.to_vec());
    assert!(TestAccumulatorProof::try_from(compressed_proof).is_err());
}

#[test]
fn test_convert_accumulator_proof_malformed_hashes() {
    let mut sibling0 = b"0".test_only_hash().to_vec();
    sibling0.push(1);

    let mut compressed_proof = crate::proto::types::AccumulatorProof::default();
    compressed_proof.bitmap = 0b100;
    compressed_proof.non_default_siblings.push(sibling0);
    assert!(TestAccumulatorProof::try_from(compressed_proof).is_err());
}

fn sparse_merkle_proof_protobuf_conversion_test(
    leaf: Option<(HashValue, HashValue)>,
    siblings: Vec<HashValue>,
    expected_bitmap: Vec<u8>,
    expected_num_non_default_siblings: usize,
) {
    let proof = SparseMerkleProof::new(leaf, siblings);
    let compressed_proof: crate::proto::types::SparseMerkleProof = proof.clone().into();
    assert_eq!(expected_bitmap, compressed_proof.bitmap);
    assert_eq!(
        compressed_proof.non_default_siblings.len(),
        expected_num_non_default_siblings
    );
    let decompressed_proof = SparseMerkleProof::try_from(compressed_proof).unwrap();
    assert_eq!(decompressed_proof, proof);
}

#[test]
fn test_convert_sparse_merkle_proof_to_protobuf() {
    sparse_merkle_proof_protobuf_conversion_test(None, vec![], vec![], 0);
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![b"0".test_only_hash()],
        vec![0b1000_0000],
        1,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![
            b"0".test_only_hash(),
            b"1".test_only_hash(),
            b"2".test_only_hash(),
        ],
        vec![0b1110_0000],
        3,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, b"1".test_only_hash()],
        vec![0b0100_0000],
        1,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![
            b"0".test_only_hash(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            b"2".test_only_hash(),
        ],
        vec![0b1010_0000],
        2,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![
            b"0".test_only_hash(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            b"7".test_only_hash(),
        ],
        vec![0b1000_0001],
        2,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        None,
        vec![
            b"0".test_only_hash(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            b"7".test_only_hash(),
            b"8".test_only_hash(),
        ],
        vec![0b1000_0001, 0b1000_0000],
        3,
    );
    sparse_merkle_proof_protobuf_conversion_test(
        Some((HashValue::random(), HashValue::random())),
        vec![b"0".test_only_hash()],
        vec![0b1000_0000],
        1,
    );
}

#[test]
fn test_convert_sparse_merkle_proof_wrong_number_of_siblings() {
    let sibling0 = b"0".test_only_hash();
    let sibling1 = b"1".test_only_hash();

    let mut compressed_proof = crate::proto::types::SparseMerkleProof::default();
    compressed_proof.bitmap.push(0b1000_0000);
    compressed_proof
        .non_default_siblings
        .push(sibling0.to_vec());
    compressed_proof
        .non_default_siblings
        .push(sibling1.to_vec());
    assert!(SparseMerkleProof::try_from(compressed_proof).is_err());
}

#[test]
fn test_convert_sparse_merkle_proof_malformed_hashes() {
    let mut sibling0 = b"0".test_only_hash().to_vec();
    sibling0.push(1);

    let mut compressed_proof = crate::proto::types::SparseMerkleProof::default();
    compressed_proof.bitmap.push(0b1000_0000);
    compressed_proof.non_default_siblings.push(sibling0);
    assert!(SparseMerkleProof::try_from(compressed_proof).is_err());
}

#[test]
fn test_convert_sparse_merkle_proof_malformed_leaf() {
    let sibling0 = b"0".test_only_hash().to_vec();

    let mut compressed_proof = crate::proto::types::SparseMerkleProof::default();
    compressed_proof.leaf = vec![1, 2, 3];
    compressed_proof.bitmap.push(0b1000_0000);
    compressed_proof.non_default_siblings.push(sibling0);
    assert!(SparseMerkleProof::try_from(compressed_proof).is_err());
}

proptest! {
    #[test]
    fn test_accumulator_bitmap_iterator_roundtrip(value in any::<u64>()) {
        let bitmap = AccumulatorBitmap::new(value);
        let iter = bitmap.iter();
        let bitmap2 = iter.collect();
        prop_assert_eq!(bitmap, bitmap2);
    }

    #[test]
    fn test_accumulator_bitmap_iterator_inverse_roundtrip(mut value in vec(any::<bool>(), 0..63)) {
        value.insert(0, true);
        let bitmap: AccumulatorBitmap = value.iter().cloned().collect();
        let value2: Vec<_> = bitmap.iter().collect();
        prop_assert_eq!(value, value2);
    }

    #[test]
    fn test_sparse_merkle_bitmap_iterator_roundtrip(mut value in vec(any::<u8>(), 0..64)) {
        if !value.is_empty() && *value.last().unwrap() == 0 {
            *value.last_mut().unwrap() |= 0b100;
        }
        let bitmap = SparseMerkleBitmap::new(value);
        let iter = bitmap.iter();
        let bitmap2 = iter.collect();
        prop_assert_eq!(bitmap, bitmap2);
    }

    #[test]
    fn test_sparse_merkle_bitmap_iterator_inverse_roundtrip(mut value in vec(any::<bool>(), 0..255)) {
        value.push(true);
        let bitmap: SparseMerkleBitmap = value.iter().cloned().collect();
        let value2: Vec<_> = bitmap.iter().collect();
        prop_assert_eq!(value, value2);
    }

    #[test]
    fn test_accumulator_protobuf_conversion_roundtrip(proof in any::<TestAccumulatorProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccumulatorProof, TestAccumulatorProof>(&proof);
    }

    #[test]
    fn test_sparse_merkle_protobuf_conversion_roundtrip(proof in any::<SparseMerkleProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SparseMerkleProof, SparseMerkleProof>(&proof);
    }

    #[test]
    fn test_accumulator_consistency_protobuf_conversion_roundtrip(
        proof in any::<AccumulatorConsistencyProof>(),
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::AccumulatorConsistencyProof, AccumulatorConsistencyProof>(&proof);
    }

    #[test]
    fn test_signed_transaction_proof_protobuf_conversion_roundtrip(proof in any::<SignedTransactionProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SignedTransactionProof, SignedTransactionProof>(&proof);
    }

    #[test]
    fn test_account_state_proof_protobuf_conversion_roundtrip(proof in any::<AccountStateProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccountStateProof, AccountStateProof>(&proof);
    }

    #[test]
    fn test_event_proof_protobuf_conversion_roundtrip(proof in any::<EventProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EventProof, EventProof>(&proof);
    }
}
