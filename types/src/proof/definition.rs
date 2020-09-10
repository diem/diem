// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module has definition of various proofs.

use super::{
    accumulator::InMemoryAccumulator, position::Position, verify_transaction_info,
    MerkleTreeInternalNode, SparseMerkleInternalNode, SparseMerkleLeafNode,
};
use crate::{
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfo,
    transaction::{TransactionInfo, Version},
};
use anyhow::{bail, ensure, format_err, Result};
#[cfg(any(test, feature = "fuzzing"))]
use libra_crypto::hash::TestOnlyHasher;
use libra_crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, TransactionAccumulatorHasher,
        SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// A proof that can be used authenticate an element in an accumulator given trusted root hash. For
/// example, both `LedgerInfoToTransactionInfoProof` and `TransactionInfoToEventProof` can be
/// constructed on top of this structure.
#[derive(Clone, Serialize, Deserialize)]
pub struct AccumulatorProof<H> {
    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    siblings: Vec<HashValue>,

    phantom: PhantomData<H>,
}

/// Because leaves can only take half the space in the tree, any numbering of the tree leaves must
/// not take the full width of the total space.  Thus, for a 64-bit ordering, our maximumm proof
/// depth is limited to 63.
pub type LeafCount = u64;
pub const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
pub const MAX_ACCUMULATOR_LEAVES: LeafCount = 1 << MAX_ACCUMULATOR_PROOF_DEPTH;

impl<H> AccumulatorProof<H>
where
    H: CryptoHasher,
{
    /// Constructs a new `AccumulatorProof` using a list of siblings.
    pub fn new(siblings: Vec<HashValue>) -> Self {
        AccumulatorProof {
            siblings,
            phantom: PhantomData,
        }
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[HashValue] {
        &self.siblings
    }

    /// Verifies an element whose hash is `element_hash` and version is `element_version` exists in
    /// the accumulator whose root hash is `expected_root_hash` using the provided proof.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        element_hash: HashValue,
        element_index: u64,
    ) -> Result<()> {
        ensure!(
            self.siblings.len() <= MAX_ACCUMULATOR_PROOF_DEPTH,
            "Accumulator proof has more than {} ({}) siblings.",
            MAX_ACCUMULATOR_PROOF_DEPTH,
            self.siblings.len()
        );

        let actual_root_hash = self
            .siblings
            .iter()
            .fold(
                (element_hash, element_index),
                // `index` denotes the index of the ancestor of the element at the current level.
                |(hash, index), sibling_hash| {
                    (
                        if index % 2 == 0 {
                            // the current node is a left child.
                            MerkleTreeInternalNode::<H>::new(hash, *sibling_hash).hash()
                        } else {
                            // the current node is a right child.
                            MerkleTreeInternalNode::<H>::new(*sibling_hash, hash).hash()
                        },
                        // The index of the parent at its level.
                        index / 2,
                    )
                },
            )
            .0;
        ensure!(
            actual_root_hash == expected_root_hash,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            actual_root_hash,
            expected_root_hash
        );

        Ok(())
    }
}

impl<H> std::fmt::Debug for AccumulatorProof<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccumulatorProof {{ siblings: {:?} }}", self.siblings)
    }
}

impl<H> PartialEq for AccumulatorProof<H> {
    fn eq(&self, other: &Self) -> bool {
        self.siblings == other.siblings
    }
}

impl<H> Eq for AccumulatorProof<H> {}

pub type TransactionAccumulatorProof = AccumulatorProof<TransactionAccumulatorHasher>;
pub type EventAccumulatorProof = AccumulatorProof<EventAccumulatorHasher>;
#[cfg(any(test, feature = "fuzzing"))]
pub type TestAccumulatorProof = AccumulatorProof<TestOnlyHasher>;

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(leaf_node)`
    ///         - If `leaf_node.key` equals requested key, this is an inclusion proof and
    ///           `leaf_node.value_hash` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. `leaf_node.key` is the only key
    ///           that exists in the subtree and `leaf_node.value_hash` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    leaf: Option<SparseMerkleLeafNode>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    siblings: Vec<HashValue>,
}

impl SparseMerkleProof {
    /// Constructs a new `SparseMerkleProof` using leaf and a list of siblings.
    pub fn new(leaf: Option<SparseMerkleLeafNode>, siblings: Vec<HashValue>) -> Self {
        SparseMerkleProof { leaf, siblings }
    }

    /// Returns the leaf node in this proof.
    pub fn leaf(&self) -> Option<SparseMerkleLeafNode> {
        self.leaf
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[HashValue] {
        &self.siblings
    }

    /// If `element_blob` is present, verifies an element whose key is `element_key` and value is
    /// `element_blob` exists in the Sparse Merkle Tree using the provided proof. Otherwise
    /// verifies the proof is a valid non-inclusion proof that shows this key doesn't exist in the
    /// tree.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        element_key: HashValue,
        element_blob: Option<&AccountStateBlob>,
    ) -> Result<()> {
        ensure!(
            self.siblings.len() <= HashValue::LENGTH_IN_BITS,
            "Sparse Merkle Tree proof has more than {} ({}) siblings.",
            HashValue::LENGTH_IN_BITS,
            self.siblings.len(),
        );

        match (element_blob, self.leaf) {
            (Some(blob), Some(leaf)) => {
                // This is an inclusion proof, so the key and value hash provided in the proof
                // should match element_key and element_value_hash. `siblings` should prove the
                // route from the leaf node to the root.
                ensure!(
                    element_key == leaf.key,
                    "Keys do not match. Key in proof: {:x}. Expected key: {:x}.",
                    leaf.key,
                    element_key
                );
                let hash = blob.hash();
                ensure!(
                    hash == leaf.value_hash,
                    "Value hashes do not match. Value hash in proof: {:x}. \
                     Expected value hash: {:x}",
                    leaf.value_hash,
                    hash,
                );
            }
            (Some(_blob), None) => bail!("Expected inclusion proof. Found non-inclusion proof."),
            (None, Some(leaf)) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will break a currently existing leaf
                // node represented by `proof_key` into a branch. `siblings` should prove the
                // route from that leaf node to the root.
                ensure!(
                    element_key != leaf.key,
                    "Expected non-inclusion proof, but key exists in proof.",
                );
                ensure!(
                    element_key.common_prefix_bits_len(leaf.key) >= self.siblings.len(),
                    "Key would not have ended up in the subtree where the provided key in proof \
                     is the only existing key, if it existed. So this is not a valid \
                     non-inclusion proof.",
                );
            }
            (None, None) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will show up at a currently empty
                // position. `sibling` should prove the route from this empty position to the root.
            }
        }

        let current_hash = self
            .leaf
            .map_or(*SPARSE_MERKLE_PLACEHOLDER_HASH, |leaf| leaf.hash());
        let actual_root_hash = self
            .siblings
            .iter()
            .zip(
                element_key
                    .iter_bits()
                    .rev()
                    .skip(HashValue::LENGTH_IN_BITS - self.siblings.len()),
            )
            .fold(current_hash, |hash, (sibling_hash, bit)| {
                if bit {
                    SparseMerkleInternalNode::new(*sibling_hash, hash).hash()
                } else {
                    SparseMerkleInternalNode::new(hash, *sibling_hash).hash()
                }
            });
        ensure!(
            actual_root_hash == expected_root_hash,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            actual_root_hash,
            expected_root_hash,
        );

        Ok(())
    }
}

/// A proof that can be used to show that two Merkle accumulators are consistent -- the big one can
/// be obtained by appending certain leaves to the small one. For example, at some point in time a
/// client knows that the root hash of the ledger at version 10 is `old_root` (it could be a
/// waypoint). If a server wants to prove that the new ledger at version `N` is derived from the
/// old ledger the client knows, it can show the subtrees that represent all the new leaves. If
/// the client can verify that it can indeed obtain the new root hash by appending these new
/// leaves, it can be convinced that the two accumulators are consistent.
///
/// See [`crate::proof::accumulator::Accumulator::append_subtrees`] for more details.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorConsistencyProof {
    /// The subtrees representing the newly appended leaves.
    subtrees: Vec<HashValue>,
}

impl AccumulatorConsistencyProof {
    /// Constructs a new `AccumulatorConsistencyProof` using given `subtrees`.
    pub fn new(subtrees: Vec<HashValue>) -> Self {
        Self { subtrees }
    }

    /// Returns the subtrees.
    pub fn subtrees(&self) -> &[HashValue] {
        &self.subtrees
    }
}

/// A proof that is similar to `AccumulatorProof`, but can be used to authenticate a range of
/// leaves. For example, given the following accumulator:
///
/// ```text
///                 root
///                /     \
///              /         \
///            /             \
///           o               o
///         /   \           /   \
///        /     \         /     \
///       X       o       o       Y
///      / \     / \     / \     / \
///     o   o   a   b   c   Z   o   o
/// ```
///
/// if the proof wants to show that `[a, b, c]` exists in the accumulator, it would need `X` on the
/// left and `Y` and `Z` on the right.
#[derive(Clone, Deserialize, Serialize)]
pub struct AccumulatorRangeProof<H> {
    /// The siblings on the left of the path from the first leaf to the root. Siblings near the root
    /// are at the beginning of the vector.
    left_siblings: Vec<HashValue>,

    /// The sliblings on the right of the path from the last leaf to the root. Siblings near the root
    /// are at the beginning of the vector.
    right_siblings: Vec<HashValue>,

    phantom: PhantomData<H>,
}

impl<H> AccumulatorRangeProof<H>
where
    H: CryptoHasher,
{
    /// Constructs a new `AccumulatorRangeProof` using `left_siblings` and `right_siblings`.
    pub fn new(left_siblings: Vec<HashValue>, right_siblings: Vec<HashValue>) -> Self {
        Self {
            left_siblings,
            right_siblings,
            phantom: PhantomData,
        }
    }

    /// Constructs a new `AccumulatorRangeProof` for an empty list of leaves.
    pub fn new_empty() -> Self {
        Self::new(vec![], vec![])
    }

    /// Get all the left siblngs.
    pub fn left_siblings(&self) -> &Vec<HashValue> {
        &self.left_siblings
    }

    /// Get all the right siblngs.
    pub fn right_siblings(&self) -> &Vec<HashValue> {
        &self.right_siblings
    }

    /// Verifies the proof is correct. The verifier needs to have `expected_root_hash`, the index
    /// of the first leaf and all of the leaves in possession.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        first_leaf_index: Option<u64>,
        leaf_hashes: &[HashValue],
    ) -> Result<()> {
        if first_leaf_index.is_none() {
            ensure!(
                leaf_hashes.is_empty(),
                "first_leaf_index indicated empty list while leaf_hashes is not empty.",
            );
            ensure!(
                self.left_siblings.is_empty() && self.right_siblings.is_empty(),
                "No siblings are needed.",
            );
            return Ok(());
        }

        ensure!(
            self.left_siblings.len() <= MAX_ACCUMULATOR_PROOF_DEPTH,
            "Proof has more than {} ({}) left siblings.",
            MAX_ACCUMULATOR_PROOF_DEPTH,
            self.left_siblings.len(),
        );
        ensure!(
            self.right_siblings.len() <= MAX_ACCUMULATOR_PROOF_DEPTH,
            "Proof has more than {} ({}) right siblings.",
            MAX_ACCUMULATOR_PROOF_DEPTH,
            self.right_siblings.len(),
        );
        ensure!(
            !leaf_hashes.is_empty(),
            "leaf_hashes is empty while first_leaf_index indicated non-empty list.",
        );

        let mut left_sibling_iter = self.left_siblings.iter().peekable();
        let mut right_sibling_iter = self.right_siblings.iter().peekable();

        let mut first_pos = Position::from_leaf_index(
            first_leaf_index.expect("first_leaf_index should not be None."),
        );
        let mut current_hashes = leaf_hashes.to_vec();
        let mut parent_hashes = vec![];

        // Keep reducing the list of hashes by combining all the children pairs, until there is
        // only one hash left.
        while current_hashes.len() > 1
            || left_sibling_iter.peek().is_some()
            || right_sibling_iter.peek().is_some()
        {
            let mut children_iter = current_hashes.iter();

            // If the first position on the current level is a right child, it needs to be combined
            // with a sibling on the left.
            if first_pos.is_right_child() {
                let left_hash = *left_sibling_iter.next().ok_or_else(|| {
                    format_err!("First child is a right child, but missing sibling on the left.")
                })?;
                let right_hash = *children_iter.next().expect("The first leaf must exist.");
                parent_hashes.push(MerkleTreeInternalNode::<H>::new(left_hash, right_hash).hash());
            }

            // Next we take two children at a time and compute their parents.
            let mut children_iter = children_iter.as_slice().chunks_exact(2);
            while let Some(chunk) = children_iter.next() {
                let left_hash = chunk[0];
                let right_hash = chunk[1];
                parent_hashes.push(MerkleTreeInternalNode::<H>::new(left_hash, right_hash).hash());
            }

            // Similarly, if the last position is a left child, it needs to be combined with a
            // sibling on the right.
            let remainder = children_iter.remainder();
            assert!(remainder.len() <= 1);
            if !remainder.is_empty() {
                let left_hash = remainder[0];
                let right_hash = *right_sibling_iter.next().ok_or_else(|| {
                    format_err!("Last child is a left child, but missing sibling on the right.")
                })?;
                parent_hashes.push(MerkleTreeInternalNode::<H>::new(left_hash, right_hash).hash());
            }

            first_pos = first_pos.parent();
            current_hashes.clear();
            std::mem::swap(&mut current_hashes, &mut parent_hashes);
        }

        ensure!(
            current_hashes[0] == expected_root_hash,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            current_hashes[0],
            expected_root_hash,
        );

        Ok(())
    }
}

impl<H> std::fmt::Debug for AccumulatorRangeProof<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AccumulatorRangeProof {{ left_siblings: {:?}, right_siblings: {:?} }}",
            self.left_siblings, self.right_siblings,
        )
    }
}

impl<H> PartialEq for AccumulatorRangeProof<H> {
    fn eq(&self, other: &Self) -> bool {
        self.left_siblings == other.left_siblings && self.right_siblings == other.right_siblings
    }
}

impl<H> Eq for AccumulatorRangeProof<H> {}

pub type TransactionAccumulatorRangeProof = AccumulatorRangeProof<TransactionAccumulatorHasher>;
#[cfg(any(test, feature = "fuzzing"))]
pub type TestAccumulatorRangeProof = AccumulatorRangeProof<TestOnlyHasher>;

/// A proof that can be used authenticate a range of consecutive leaves, from the leftmost leaf to
/// a certain one, in a sparse Merkle tree. For example, given the following sparse Merkle tree:
///
/// ```text
///                   root
///                  /     \
///                 /       \
///                /         \
///               o           o
///              / \         / \
///             a   o       o   h
///                / \     / \
///               o   d   e   X
///              / \         / \
///             b   c       f   g
/// ```
///
/// if the proof wants show that `[a, b, c, d, e]` exists in the tree, it would need the siblings
/// `X` and `h` on the right.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleRangeProof {
    /// The vector of siblings on the right of the path from root to last leaf. The ones near the
    /// bottom are at the beginning of the vector. In the above example, it's `[X, h]`.
    right_siblings: Vec<HashValue>,
}

impl SparseMerkleRangeProof {
    /// Constructs a new `SparseMerkleRangeProof`.
    pub fn new(right_siblings: Vec<HashValue>) -> Self {
        Self { right_siblings }
    }

    /// Returns the siblings.
    pub fn right_siblings(&self) -> &[HashValue] {
        &self.right_siblings
    }
}

/// `TransactionInfo` and a `TransactionAccumulatorProof` connecting it to the ledger root.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TransactionInfoWithProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    transaction_info: TransactionInfo,
}

impl TransactionInfoWithProof {
    /// Constructs a new `TransactionWithProof` object using given
    /// `ledger_info_to_transaction_info_proof`.
    pub fn new(
        ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,
        transaction_info: TransactionInfo,
    ) -> Self {
        Self {
            ledger_info_to_transaction_info_proof,
            transaction_info,
        }
    }

    /// Returns the `ledger_info_to_transaction_info_proof` object in this proof.
    pub fn ledger_info_to_transaction_info_proof(&self) -> &TransactionAccumulatorProof {
        &self.ledger_info_to_transaction_info_proof
    }

    /// Returns the `transaction_info` object in this proof.
    pub fn transaction_info(&self) -> &TransactionInfo {
        &self.transaction_info
    }

    /// Verifies that the `TransactionInfo` exists in the ledger represented by the `LedgerInfo`
    /// at specified version.
    pub fn verify(&self, ledger_info: &LedgerInfo, transaction_version: Version) -> Result<()> {
        verify_transaction_info(
            ledger_info,
            transaction_version,
            &self.transaction_info,
            &self.ledger_info_to_transaction_info_proof,
        )?;
        Ok(())
    }
}

/// The complete proof used to authenticate the state of an account. This structure consists of the
/// `AccumulatorProof` from `LedgerInfo` to `TransactionInfo`, the `TransactionInfo` object and the
/// `SparseMerkleProof` from state root to the account.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountStateProof {
    transaction_info_with_proof: TransactionInfoWithProof,

    /// The sparse merkle proof from state root to the account state.
    transaction_info_to_account_proof: SparseMerkleProof,
}

impl AccountStateProof {
    /// Constructs a new `AccountStateProof` using given `ledger_info_to_transaction_info_proof`,
    /// `transaction_info` and `transaction_info_to_account_proof`.
    pub fn new(
        transaction_info_with_proof: TransactionInfoWithProof,
        transaction_info_to_account_proof: SparseMerkleProof,
    ) -> Self {
        AccountStateProof {
            transaction_info_with_proof,
            transaction_info_to_account_proof,
        }
    }

    /// Returns the `transaction_info_with_proof` object in this proof.
    pub fn transaction_info_with_proof(&self) -> &TransactionInfoWithProof {
        &self.transaction_info_with_proof
    }

    /// Returns the `transaction_info_to_account_proof` object in this proof.
    pub fn transaction_info_to_account_proof(&self) -> &SparseMerkleProof {
        &self.transaction_info_to_account_proof
    }

    /// Verifies that the state of an account at version `state_version` is correct using the
    /// provided proof. If `account_state_blob` is present, we expect the account to exist,
    /// otherwise we expect the account to not exist.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        state_version: Version,
        account_address_hash: HashValue,
        account_state_blob: Option<&AccountStateBlob>,
    ) -> Result<()> {
        self.transaction_info_to_account_proof.verify(
            self.transaction_info_with_proof
                .transaction_info
                .state_root_hash(),
            account_address_hash,
            account_state_blob,
        )?;

        self.transaction_info_with_proof
            .verify(ledger_info, state_version)?;

        Ok(())
    }
}

/// The complete proof used to authenticate a contract event. This structure consists of the
/// `AccumulatorProof` from `LedgerInfo` to `TransactionInfo`, the `TransactionInfo` object and the
/// `AccumulatorProof` from event accumulator root to the event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventProof {
    transaction_info_with_proof: TransactionInfoWithProof,

    /// The accumulator proof from event root to the actual event.
    transaction_info_to_event_proof: EventAccumulatorProof,
}

impl EventProof {
    /// Constructs a new `EventProof` using given `ledger_info_to_transaction_info_proof`,
    /// `transaction_info` and `transaction_info_to_event_proof`.
    pub fn new(
        transaction_info_with_proof: TransactionInfoWithProof,
        transaction_info_to_event_proof: EventAccumulatorProof,
    ) -> Self {
        EventProof {
            transaction_info_with_proof,
            transaction_info_to_event_proof,
        }
    }

    /// Returns the `transaction_info_with_proof` object in this proof.
    pub fn transaction_info_with_proof(&self) -> &TransactionInfoWithProof {
        &self.transaction_info_with_proof
    }

    /// Verifies that a given event is correct using provided proof.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        event_hash: HashValue,
        transaction_version: Version,
        event_version_within_transaction: Version,
    ) -> Result<()> {
        self.transaction_info_to_event_proof.verify(
            self.transaction_info_with_proof
                .transaction_info()
                .event_root_hash(),
            event_hash,
            event_version_within_transaction,
        )?;

        self.transaction_info_with_proof
            .verify(ledger_info, transaction_version)?;

        Ok(())
    }
}

/// The complete proof used to authenticate a list of consecutive transactions.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TransactionListProof {
    /// The accumulator range proof from ledger info root to leaves that authenticates the hashes
    /// of all `TransactionInfo` objects.
    ledger_info_to_transaction_infos_proof: TransactionAccumulatorRangeProof,

    /// The `TransactionInfo` objects that correspond to all the transactions.
    transaction_infos: Vec<TransactionInfo>,
}

impl TransactionListProof {
    /// Constructs a new `TransactionListProof` using `ledger_info_to_transaction_info_proof` and
    /// `transaction_infos`.
    pub fn new(
        ledger_info_to_transaction_infos_proof: TransactionAccumulatorRangeProof,
        transaction_infos: Vec<TransactionInfo>,
    ) -> Self {
        Self {
            ledger_info_to_transaction_infos_proof,
            transaction_infos,
        }
    }

    /// Constructs a proof for an empty list of transactions.
    pub fn new_empty() -> Self {
        Self::new(AccumulatorRangeProof::new_empty(), vec![])
    }

    /// Returns the list of `TransactionInfo` objects.
    pub fn transaction_infos(&self) -> &[TransactionInfo] {
        &self.transaction_infos
    }

    pub fn left_siblings(&self) -> &Vec<HashValue> {
        self.ledger_info_to_transaction_infos_proof.left_siblings()
    }

    pub fn unpack(self) -> (TransactionAccumulatorRangeProof, Vec<TransactionInfo>) {
        (
            self.ledger_info_to_transaction_infos_proof,
            self.transaction_infos,
        )
    }

    /// Verifies the list of transactions are correct using the proof. The verifier needs to have
    /// the ledger info and the version of the first transaction in possession.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        first_transaction_version: Option<Version>,
        transaction_hashes: &[HashValue],
    ) -> Result<()> {
        ensure!(
            self.transaction_infos.len() == transaction_hashes.len(),
            "The number of TransactionInfo objects ({}) does not match the number of \
             transactions ({}).",
            self.transaction_infos.len(),
            transaction_hashes.len(),
        );

        itertools::zip_eq(transaction_hashes, &self.transaction_infos)
            .map(|(txn_hash, txn_info)| {
                ensure!(
                    *txn_hash == txn_info.transaction_hash(),
                    "The hash of transaction does not match the transaction info in proof. \
                     Transaction hash: {:x}. Transaction hash in txn_info: {:x}.",
                    txn_hash,
                    txn_info.transaction_hash(),
                );
                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;

        let txn_info_hashes: Vec<_> = self
            .transaction_infos
            .iter()
            .map(CryptoHash::hash)
            .collect();
        self.ledger_info_to_transaction_infos_proof.verify(
            ledger_info.transaction_accumulator_hash(),
            first_transaction_version,
            &txn_info_hashes,
        )?;
        Ok(())
    }
}

/// A proof that first verifies that establishes correct computation of the root and then
/// returns the new tree to acquire a new root and version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
