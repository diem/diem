// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::ensure;
#[cfg(test)]
use libra_crypto::hash::{CryptoHash, TestOnlyHasher, ACCUMULATOR_PLACEHOLDER_HASH};
use libra_crypto::hash::{CryptoHasher, HashValue};
#[cfg(test)]
use libra_types::proof::TestAccumulatorInternalNode;
use libra_types::proof::{accumulator::InMemoryAccumulator, definition::LeafCount};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// A proof that first verifies that establishes correct computation of the root and then
/// returns the new tree to acquire a new root and version. Note: this is used internally by
/// VoteProposal hence why it exists within consensus-types andd not libra-types.
/// @TODO This should contain AccumulatorConsistencyProof once that is code complete
#[derive(Clone, Deserialize, Serialize)]
pub struct AccumulatorExtensionProof<H> {
    /// Represents the roots of all the full subtrees from left to right in the original accumulator.
    frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in original accumulator.
    num_leaves: LeafCount,
    /// The values representing the newly appended leaves.
    leaves: Vec<HashValue>,

    hasher: PhantomData<H>,
}

impl<H: CryptoHasher> AccumulatorExtensionProof<H> {
    pub fn new(
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        leaves: Vec<HashValue>,
    ) -> Self {
        Self {
            frozen_subtree_roots,
            num_leaves,
            leaves,
            hasher: PhantomData,
        }
    }

    pub fn verify(&self, original_root: HashValue) -> anyhow::Result<InMemoryAccumulator<H>> {
        let original_tree =
            InMemoryAccumulator::<H>::new(self.frozen_subtree_roots.clone(), self.num_leaves)?;
        ensure!(
            original_tree.root_hash() == original_root,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            original_tree.root_hash(),
            original_root
        );

        Ok(original_tree.append(self.leaves.as_slice()))
    }
}

// This test does the following:
// 1) Test that empty has a well defined definition
// 2) Test a single value
// 3) Test multiple values
// 4) Random nonsense returns an error
#[test]
fn test_accumulator_extension_proof() {
    // Test empty
    let empty = AccumulatorExtensionProof::<TestOnlyHasher>::new(vec![], 0, vec![]);

    let derived_tree = empty.verify(*ACCUMULATOR_PLACEHOLDER_HASH).unwrap();
    assert_eq!(*ACCUMULATOR_PLACEHOLDER_HASH, derived_tree.root_hash());
    assert_eq!(derived_tree.version(), 0);

    // Test a single value
    HashValue::zero();
    let one_tree =
        AccumulatorExtensionProof::<TestOnlyHasher>::new(vec![], 0, vec![HashValue::zero()]);

    let derived_tree = one_tree.verify(*ACCUMULATOR_PLACEHOLDER_HASH).unwrap();
    assert_eq!(HashValue::zero(), derived_tree.root_hash());
    assert_eq!(derived_tree.version(), 0);

    // Test multiple values
    let two_tree = AccumulatorExtensionProof::<TestOnlyHasher>::new(
        vec![HashValue::zero()],
        1,
        vec![HashValue::zero()],
    );

    let derived_tree = two_tree.verify(HashValue::zero()).unwrap();
    let two_hash = TestAccumulatorInternalNode::new(HashValue::zero(), HashValue::zero()).hash();
    assert_eq!(two_hash, derived_tree.root_hash());
    assert_eq!(derived_tree.version(), 1);

    // Test nonsense breaks
    let derived_tree_err = two_tree.verify(*ACCUMULATOR_PLACEHOLDER_HASH);
    assert!(derived_tree_err.is_err());
}
