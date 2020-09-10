// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! All proofs generated in this module are not valid proofs. They are only for the purpose of
//! testing conversion between Rust and Protobuf.

use crate::proof::{
    definition::MAX_ACCUMULATOR_PROOF_DEPTH, AccumulatorConsistencyProof, AccumulatorProof,
    AccumulatorRangeProof, SparseMerkleLeafNode, SparseMerkleProof, SparseMerkleRangeProof,
};
use libra_crypto::{
    hash::{CryptoHasher, ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use proptest::{collection::vec, prelude::*};

fn arb_non_placeholder_accumulator_sibling() -> impl Strategy<Value = HashValue> {
    any::<HashValue>().prop_filter("Filter out placeholder sibling.", |x| {
        *x != *ACCUMULATOR_PLACEHOLDER_HASH
    })
}

fn arb_accumulator_sibling() -> impl Strategy<Value = HashValue> {
    prop_oneof![
        arb_non_placeholder_accumulator_sibling(),
        Just(*ACCUMULATOR_PLACEHOLDER_HASH),
    ]
}

fn arb_non_placeholder_sparse_merkle_sibling() -> impl Strategy<Value = HashValue> {
    any::<HashValue>().prop_filter("Filter out placeholder sibling.", |x| {
        *x != *SPARSE_MERKLE_PLACEHOLDER_HASH
    })
}

fn arb_sparse_merkle_sibling() -> impl Strategy<Value = HashValue> {
    prop_oneof![
        arb_non_placeholder_sparse_merkle_sibling(),
        Just(*SPARSE_MERKLE_PLACEHOLDER_HASH),
    ]
}

impl<H> Arbitrary for AccumulatorProof<H>
where
    H: CryptoHasher + 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..=MAX_ACCUMULATOR_PROOF_DEPTH)
            .prop_flat_map(|len| {
                if len == 0 {
                    Just(vec![]).boxed()
                } else {
                    (
                        vec(arb_accumulator_sibling(), len - 1),
                        arb_non_placeholder_accumulator_sibling(),
                    )
                        .prop_map(|(mut siblings, last_sibling)| {
                            siblings.push(last_sibling);
                            siblings
                        })
                        .boxed()
                }
            })
            .prop_map(AccumulatorProof::<H>::new)
            .boxed()
    }
}

impl Arbitrary for SparseMerkleProof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<Option<SparseMerkleLeafNode>>(),
            (0..=256usize).prop_flat_map(|len| {
                if len == 0 {
                    Just(vec![]).boxed()
                } else {
                    (
                        arb_non_placeholder_sparse_merkle_sibling(),
                        vec(arb_sparse_merkle_sibling(), len),
                    )
                        .prop_map(|(first_sibling, mut siblings)| {
                            siblings[0] = first_sibling;
                            siblings
                        })
                        .boxed()
                }
            }),
        )
            .prop_map(|(leaf, siblings)| SparseMerkleProof::new(leaf, siblings))
            .boxed()
    }
}

impl Arbitrary for AccumulatorConsistencyProof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        vec(
            arb_non_placeholder_accumulator_sibling(),
            0..=MAX_ACCUMULATOR_PROOF_DEPTH,
        )
        .prop_map(AccumulatorConsistencyProof::new)
        .boxed()
    }
}

impl<H> Arbitrary for AccumulatorRangeProof<H>
where
    H: CryptoHasher,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            vec(
                arb_non_placeholder_accumulator_sibling(),
                0..MAX_ACCUMULATOR_PROOF_DEPTH,
            ),
            vec(arb_accumulator_sibling(), 0..MAX_ACCUMULATOR_PROOF_DEPTH),
        )
            .prop_map(|(left_siblings, right_siblings)| {
                AccumulatorRangeProof::new(left_siblings, right_siblings)
            })
            .boxed()
    }
}

impl Arbitrary for SparseMerkleRangeProof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        vec(arb_sparse_merkle_sibling(), 0..=256)
            .prop_map(Self::new)
            .boxed()
    }
}
