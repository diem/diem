// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! All proofs generated in this module are not valid proofs. They are only for the purpose of
//! testing conversion between Rust and Protobuf.

use crate::proof::{
    definition::MAX_ACCUMULATOR_PROOF_DEPTH, AccumulatorConsistencyProof, AccumulatorProof,
    SparseMerkleProof,
};
use crypto::{
    hash::{ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use proptest::{collection::vec, prelude::*};
use rand::{seq::SliceRandom, thread_rng};

prop_compose! {
    fn arb_accumulator_proof()(
        non_default_siblings in vec(any::<HashValue>(), 0..MAX_ACCUMULATOR_PROOF_DEPTH),
        total_num_siblings in 0..=MAX_ACCUMULATOR_PROOF_DEPTH,
    ) -> AccumulatorProof {
        let mut siblings = non_default_siblings;
        if !siblings.is_empty() {
            let total_num_siblings = std::cmp::max(siblings.len(), total_num_siblings);
            for _ in siblings.len()..total_num_siblings {
                siblings.push(ACCUMULATOR_PLACEHOLDER_HASH.clone());
            }
            assert_eq!(siblings.len(), total_num_siblings);
            (&mut siblings[1..]).shuffle(&mut thread_rng());
        }
        AccumulatorProof::new(siblings)
    }
}

prop_compose! {
    fn arb_sparse_merkle_proof()(
        leaf in any::<Option<(HashValue, HashValue)>>(),
        non_default_siblings in vec(any::<HashValue>(), 0..256usize),
        total_num_siblings in 0..257usize,
    ) -> SparseMerkleProof {
        let mut siblings = non_default_siblings;
        if !siblings.is_empty() {
            let total_num_siblings = std::cmp::max(siblings.len(), total_num_siblings);
            for _ in siblings.len()..total_num_siblings {
                siblings.insert(0, SPARSE_MERKLE_PLACEHOLDER_HASH.clone());
            }
            assert_eq!(siblings.len(), total_num_siblings);
            (&mut siblings[0..total_num_siblings-1]).shuffle(&mut thread_rng());
        }
        SparseMerkleProof::new(leaf, siblings)
    }
}

prop_compose! {
    fn arb_accumulator_consistency_proof()(
        subtrees in vec(any::<HashValue>(), 0..=MAX_ACCUMULATOR_PROOF_DEPTH),
    ) -> AccumulatorConsistencyProof {
        AccumulatorConsistencyProof::new(subtrees)
    }
}

macro_rules! impl_arbitrary_for_proof {
    ($proof_type: ident, $arb_func: ident) => {
        impl Arbitrary for $proof_type {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
                $arb_func().boxed()
            }
        }
    };
}

impl_arbitrary_for_proof!(AccumulatorProof, arb_accumulator_proof);
impl_arbitrary_for_proof!(SparseMerkleProof, arb_sparse_merkle_proof);
impl_arbitrary_for_proof!(
    AccumulatorConsistencyProof,
    arb_accumulator_consistency_proof
);
