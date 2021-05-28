// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    proof::{AccumulatorConsistencyProof, MerkleTreeInternalNode, TransactionAccumulatorSummary},
    transaction::Version,
};
use diem_crypto::hash::{
    CryptoHash, HashValue, TransactionAccumulatorHasher, ACCUMULATOR_PLACEHOLDER_HASH,
};
use std::{cell::RefCell, collections::HashMap};

type Children = (HashValue, Option<HashValue>);

/// An immutable transaction accumulator (not a summary, since it stores all leaf
/// nodes and caches internal nodes) that allows for easily computing root hashes,
/// consistency proofs, etc... at any versions in the range `0..=self.version()`.
///
/// This implementation intentionally eschews the existing storage implementation
/// and `Position` APIs to be as different as possible so we can better guarantee
/// correctness across implementations. Here, we mostly use "naive" recursive
/// algorithms (with internal caches).
///
/// Note: intended for test-only code; panics whenever something goes wrong.
#[derive(Clone, Debug)]
pub struct MockTransactionAccumulator {
    /// The list of leaf nodes.
    leaves: Vec<HashValue>,
    /// Cache that maps child nodes to their parent node.
    c2p: RefCell<HashMap<Children, HashValue>>,
    /// Cache that maps from a parent node to its child nodes.
    p2c: RefCell<HashMap<HashValue, Children>>,
}

impl MockTransactionAccumulator {
    /// Create a full transaction accumulator from a list of leaf node hashes.
    pub fn from_leaves(leaves: Vec<HashValue>) -> Self {
        assert!(!leaves.is_empty());
        Self {
            leaves,
            c2p: RefCell::new(HashMap::new()),
            p2c: RefCell::new(HashMap::new()),
        }
    }

    /// Create an accumulator with some mock leaf hash values at the given version.
    pub fn with_version(version: Version) -> Self {
        Self::from_leaves(mock_txn_hashes(version))
    }

    pub fn version(&self) -> Version {
        self.leaves.len() as u64 - 1
    }

    fn hash_internal_inner(pair: Children) -> HashValue {
        let (left, maybe_right) = pair;
        let right = maybe_right.unwrap_or(*ACCUMULATOR_PLACEHOLDER_HASH);
        MerkleTreeInternalNode::<TransactionAccumulatorHasher>::new(left, right).hash()
    }

    /// Compute the parent node hash from a pair of siblings while also updating
    /// internal caches.
    fn hash_internal(&self, pair: Children) -> HashValue {
        let parent = *self
            .c2p
            .borrow_mut()
            .entry(pair)
            .or_insert_with(|| Self::hash_internal_inner(pair));
        self.p2c.borrow_mut().insert(parent, pair);
        parent
    }

    /// Get the accumulator root hash at a specific version. Note that this method
    /// has the side effect of seeding the parent<->child caches.
    pub fn get_root_hash(&self, version: Version) -> HashValue {
        assert!(version <= self.version());
        let leaves_view = &self.leaves[0..=version as usize];

        let mut level = leaves_view.to_vec();
        while level.len() != 1 {
            let siblings_iter = level.chunks(2);
            level = siblings_iter
                .map(|pair| self.hash_internal((pair[0], pair.get(1).copied())))
                .collect();
        }

        level.into_iter().next().unwrap()
    }

    fn children(&self, parent: HashValue) -> Option<Children> {
        // precondition: parent in internal node set
        self.p2c.borrow().get(&parent).copied()
    }

    fn height(&self, subtree_root: HashValue) -> u64 {
        // precondition: subtree_root in internal node set
        match self.children(subtree_root) {
            // leaves have no children
            None => 0,
            Some((left, _)) => self.height(left) + 1,
        }
    }

    /// A node is frozen if it is complete. A node is complete if it's a leaf node
    /// or both of its children are complete. Note that a complete right child
    /// also implies that the left child is complete (since this is an accumulator).
    fn is_frozen(&self, subtree_root: HashValue) -> bool {
        // precondition: subtree_root in internal node set
        match self.children(subtree_root) {
            // leaf node ==> frozen
            None => true,
            // right subtree complete ==> left subtree also complete ==> frozen
            Some((_, Some(right))) => self.is_frozen(right),
            // not complete ==> not frozen
            Some((_, None)) => false,
        }
    }

    /// f(n) := if n.is_frozen => [n]
    ///         else           => f(n.left) || f(n.right)
    fn frozen_subtrees(&self, subtree_root: HashValue) -> Vec<HashValue> {
        // precondition: subtree_root in internal node set
        if self.is_frozen(subtree_root) {
            vec![subtree_root]
        } else {
            let (left, maybe_right) = self.children(subtree_root).unwrap();
            let mut left_subtrees = self.frozen_subtrees(left);
            let right_subtrees = maybe_right
                .map(|right| self.frozen_subtrees(right))
                .unwrap_or_else(Vec::new);
            left_subtrees.extend(right_subtrees);
            left_subtrees
        }
    }

    pub fn get_accumulator_summary(&self, version: Version) -> TransactionAccumulatorSummary {
        assert!(version <= self.version());

        let genesis_consistency_proof = self.get_consistency_proof(None, version);
        TransactionAccumulatorSummary::try_from_genesis_proof(genesis_consistency_proof, version)
            .unwrap()
    }

    pub fn get_consistency_proof(
        &self,
        start_version: Option<Version>,
        end_version: Version,
    ) -> AccumulatorConsistencyProof {
        assert!(start_version <= Some(end_version));
        assert!(end_version <= self.version());

        let maybe_old_root = start_version.map(|v| self.get_root_hash(v));
        let new_root = self.get_root_hash(end_version);

        let height_diff = maybe_old_root
            .clone()
            .map(|old_root| self.height(new_root) - self.height(old_root))
            .unwrap_or(0);

        let subtrees = self.frozen_subtree_diff(maybe_old_root, new_root, height_diff);
        AccumulatorConsistencyProof::new(subtrees)
    }

    /// Given a subtree root at an older version and a subtree root at a newer
    /// version, find the frozen subtree siblings needed to bring the frozen
    /// subtrees at the old subtree to the frozen subtrees at the new subtree.
    ///
    /// This method is recursive, typically starting at the root hashes for the
    /// old and new accumulators. Note that the new accumulator root can start at
    /// a greater height. After recursing left `height_diff` times, the old and
    /// new subtree roots will be tracking the same position. When old and new
    /// subtrees are tracking the same position, `height_diff == 0`.
    ///
    /// The old subtree parameter is an Option; when None, it means there is no
    /// subtree at this position in the old accumulator.
    ///
    /// ```not_rust
    /// // base case:
    /// // position does not exist in old subtree
    /// f(None, new, _) := frozen_subtrees(new)
    /// // node is frozen in both accumulators
    /// f(Some(same), same, 0) := []
    ///
    /// // inductive step:
    /// // same position
    /// f(Some(old), new, 0)
    ///     := f(Some(old.left), new.left, 0) || new.maybe_right.map(|new_right| f(old.maybe_right, new_right, 0))
    /// // new.height > old.height ==> old is in new's left descendents
    /// f(Some(old), new, dh > 0)
    ///     := f(Some(old), new.left, dh - 1) || f(None, new.right, _)
    /// ```
    fn frozen_subtree_diff(
        &self,
        maybe_old_subtree: Option<HashValue>,
        new_subtree: HashValue,
        height_diff: u64,
    ) -> Vec<HashValue> {
        // precondition: Some(old_subtree) and new_subtree in internal node set

        let old_subtree = match maybe_old_subtree {
            Some(old_subtree) => old_subtree,
            // When we are in a disjoint subtree from the old accumulator, we just
            // need the frozen subtrees.
            None => return self.frozen_subtrees(new_subtree),
        };

        if height_diff == 0 {
            // the old subtree and new subtree are at the same position

            if old_subtree == new_subtree {
                // the same subtree is frozen in both accumulators, don't need
                // anything here.
                vec![]
            } else {
                // this subtree is different between the old and new accumulator

                let (old_left, maybe_old_right) = self
                    .children(old_subtree)
                    .expect("cannot have two different leaf nodes");
                let (new_left, maybe_new_right) = self
                    .children(new_subtree)
                    .expect("cannot have two different leaf nodes");

                // recurse left and right
                let mut left_subtrees = self.frozen_subtree_diff(Some(old_left), new_left, 0);
                let right_subtrees = maybe_new_right
                    .map(|new_right| self.frozen_subtree_diff(maybe_old_right, new_right, 0))
                    .unwrap_or_else(Vec::new);
                left_subtrees.extend(right_subtrees);
                left_subtrees
            }
        } else {
            // the new subtree is still above the old subtree (which is somewhere
            // in the new subtree's left descendents).

            let (left, maybe_right) = self
                .children(new_subtree)
                .expect("non-zero height_diff implies new_subtree is an internal node, which must have children");
            let right = maybe_right
                .expect("non-zero height_diff implies new subtree must have a right child");
            // recurse to the left
            let mut left_subtrees =
                self.frozen_subtree_diff(Some(old_subtree), left, height_diff - 1);
            // this is effectively just frozen_subtrees(new.right), since the right
            // subtree will always be disjoint
            let right_subtrees = self.frozen_subtree_diff(None, right, 0);
            left_subtrees.extend(right_subtrees);
            left_subtrees
        }
    }
}

fn mock_txn_hashes(version: Version) -> Vec<HashValue> {
    (0..=version).map(HashValue::from_u64).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_info::BlockInfo, ledger_info::LedgerInfo};
    use proptest::prelude::*;

    fn mock_ledger_info(version: Version, root_hash: HashValue) -> LedgerInfo {
        LedgerInfo::new(
            BlockInfo::new(0, 0, HashValue::zero(), root_hash, version, 0, None),
            HashValue::zero(),
        )
    }

    #[test]
    fn test_mock_accumulator() {
        let end = 255;

        // A
        let accumulator = MockTransactionAccumulator::with_version(end);
        assert_eq!(accumulator.version(), end);

        let end_li = mock_ledger_info(end, accumulator.get_root_hash(end));

        // check for { m1, m2 : 0 <= m1 <= m2 <= end } that accumulators A and B have the same root hash
        // |------------------------------------------->|        A: accumulator
        // |--------------->|------------>|------------>|        B: B_{None,m1}.extend(B_{m1,m2}).extend(B_{m2,end})
        // 0               m1            m2            end
        //    B_{None,m1}      B_{m1,m2}     B_{m2,end}    B_{i,j}: accumulator.consistency_proof(i, j)
        proptest!(|((m1, m2) in (0..=end).prop_flat_map(|m1| (Just(m1), m1..=end)))| {
            // B_{None,m1}
            let accumulator_summary = accumulator.get_accumulator_summary(m1);
            assert_eq!(accumulator_summary.version(), m1);
            assert_eq!(accumulator_summary.root_hash(), accumulator.get_root_hash(m1));

            // B_{m1,m2}
            let mid1_to_mid2 = accumulator.get_consistency_proof(Some(m1), m2);
            let mid2_li = mock_ledger_info(m2, accumulator.get_root_hash(m2));

            // B_{None,m1}.extend(B_{m1,m2})
            let accumulator_summary = accumulator_summary
                .try_extend_with_proof(&mid1_to_mid2, &mid2_li)
                .unwrap();

            // B_{m2,end}
            let mid2_to_end = accumulator.get_consistency_proof(Some(m2), end);

            // B_{None,m1}.extend(B_{m1,m2}).extend(B_{m2,end})
            accumulator_summary.try_extend_with_proof(&mid2_to_end, &end_li).unwrap();
        });
    }
}
