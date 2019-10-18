// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module has definition of various proofs.

#[cfg(test)]
#[path = "unit_tests/proof_proto_conversion_test.rs"]
mod proof_proto_conversion_test;

use self::bitmap::{AccumulatorBitmap, SparseMerkleBitmap};
use super::{
    verify_transaction_info, MerkleTreeInternalNode, SparseMerkleInternalNode, SparseMerkleLeafNode,
};
use crate::{
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfo,
    transaction::{TransactionInfo, Version},
};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use libra_crypto::hash::TestOnlyHasher;
use libra_crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, TransactionAccumulatorHasher,
        ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

/// A proof that can be used authenticate an element in an accumulator given trusted root hash. For
/// example, both `LedgerInfoToTransactionInfoProof` and `TransactionInfoToEventProof` can be
/// constructed on top of this structure.
#[derive(Clone)]
pub struct AccumulatorProof<H> {
    /// All siblings in this proof, including the default ones. Siblings near the root are at the
    /// beginning of the vector.
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
        // The sibling list could be empty in case the accumulator is empty or has a single
        // element. When it's not empty, the top most sibling will never be default, otherwise the
        // accumulator should have collapsed to a smaller one.
        if let Some(first_sibling) = siblings.first() {
            assert_ne!(*first_sibling, *ACCUMULATOR_PLACEHOLDER_HASH);
        }

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
            .rev()
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

impl<H> TryFrom<crate::proto::types::AccumulatorProof> for AccumulatorProof<H>
where
    H: CryptoHasher,
{
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccumulatorProof) -> Result<Self> {
        let bitmap = proto_proof.bitmap;
        let num_non_default_siblings = bitmap.count_ones() as usize;
        ensure!(
            num_non_default_siblings == proto_proof.non_default_siblings.len(),
            "Malformed proof. Bitmap indicated {} non-default siblings. Found {} siblings.",
            num_non_default_siblings,
            proto_proof.non_default_siblings.len()
        );

        let mut proto_siblings = proto_proof.non_default_siblings.into_iter();
        // Iterate from the leftmost 1-bit to LSB in the bitmap. If a bit is set, the corresponding
        // sibling is non-default and we take the sibling from proto_siblings.  Otherwise the
        // sibling on this position is default.
        let siblings = AccumulatorBitmap::new(bitmap)
            .iter()
            .map(|x| {
                if x {
                    let hash_bytes = proto_siblings
                        .next()
                        .expect("Unexpected number of siblings.");
                    HashValue::from_slice(&hash_bytes)
                } else {
                    Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(AccumulatorProof::new(siblings))
    }
}

impl<H> From<AccumulatorProof<H>> for crate::proto::types::AccumulatorProof {
    fn from(proof: AccumulatorProof<H>) -> Self {
        let mut proto_proof = Self::default();
        // Iterate over all siblings. For each non-default sibling, add to protobuf struct and set
        // the corresponding bit in the bitmap.
        let bitmap: AccumulatorBitmap = proof
            .siblings
            .into_iter()
            .map(|sibling| {
                if sibling != *ACCUMULATOR_PLACEHOLDER_HASH {
                    proto_proof.non_default_siblings.push(sibling.to_vec());
                    true
                } else {
                    false
                }
            })
            .collect();
        proto_proof.bitmap = bitmap.into();
        proto_proof
    }
}

pub type TransactionAccumulatorProof = AccumulatorProof<TransactionAccumulatorHasher>;
pub type EventAccumulatorProof = AccumulatorProof<EventAccumulatorHasher>;
#[cfg(any(test, feature = "testing"))]
pub type TestAccumulatorProof = AccumulatorProof<TestOnlyHasher>;

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(HashValue, HashValue)`
    ///         - If the first `HashValue` equals requested key, this is an inclusion proof and the
    ///           second `HashValue` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. The first `HashValue` is the only key
    ///           that exists in the subtree and the second `HashValue` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    leaf: Option<(HashValue, HashValue)>,

    /// All siblings in this proof, including the default ones. Siblings near the root are at the
    /// beginning of the vector.
    siblings: Vec<HashValue>,
}

impl SparseMerkleProof {
    /// Constructs a new `SparseMerkleProof` using leaf and a list of siblings.
    pub fn new(leaf: Option<(HashValue, HashValue)>, siblings: Vec<HashValue>) -> Self {
        // The sibling list could be empty in case the Sparse Merkle Tree is empty or has a single
        // element. When it's not empty, the bottom most sibling will never be default, otherwise a
        // leaf and a default sibling should have collapsed to a leaf.
        if let Some(last_sibling) = siblings.last() {
            assert_ne!(*last_sibling, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        }

        SparseMerkleProof { leaf, siblings }
    }

    /// Returns the leaf node in this proof.
    pub fn leaf(&self) -> Option<(HashValue, HashValue)> {
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
            (Some(blob), Some((proof_key, proof_value_hash))) => {
                // This is an inclusion proof, so the key and value hash provided in the proof
                // should match element_key and element_value_hash. `siblings` should prove the
                // route from the leaf node to the root.
                ensure!(
                    element_key == proof_key,
                    "Keys do not match. Key in proof: {:x}. Expected key: {:x}.",
                    proof_key,
                    element_key
                );
                let hash = blob.hash();
                ensure!(
                    hash == proof_value_hash,
                    "Value hashes do not match. Value hash in proof: {:x}. \
                     Expected value hash: {:x}",
                    proof_value_hash,
                    hash,
                );
            }
            (Some(_blob), None) => bail!("Expected inclusion proof. Found non-inclusion proof."),
            (None, Some((proof_key, _))) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will break a currently existing leaf
                // node represented by `proof_key` into a branch. `siblings` should prove the
                // route from that leaf node to the root.
                ensure!(
                    element_key != proof_key,
                    "Expected non-inclusion proof, but key exists in proof.",
                );
                ensure!(
                    element_key.common_prefix_bits_len(proof_key) >= self.siblings.len(),
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

        let current_hash = match self.leaf {
            Some((key, value_hash)) => SparseMerkleLeafNode::new(key, value_hash).hash(),
            None => *SPARSE_MERKLE_PLACEHOLDER_HASH,
        };
        let actual_root_hash = self
            .siblings
            .iter()
            .rev()
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

impl TryFrom<crate::proto::types::SparseMerkleProof> for SparseMerkleProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::SparseMerkleProof) -> Result<Self> {
        let proto_leaf = proto_proof.leaf;
        let leaf = if proto_leaf.is_empty() {
            None
        } else if proto_leaf.len() == HashValue::LENGTH * 2 {
            let key = HashValue::from_slice(&proto_leaf[0..HashValue::LENGTH])?;
            let value_hash = HashValue::from_slice(&proto_leaf[HashValue::LENGTH..])?;
            Some((key, value_hash))
        } else {
            bail!(
                "Mailformed proof. Leaf has {} bytes. Expect 0 or {} bytes.",
                proto_leaf.len(),
                HashValue::LENGTH * 2
            );
        };

        let bitmap = proto_proof.bitmap;
        if let Some(last_byte) = bitmap.last() {
            ensure!(
                *last_byte != 0,
                "Malformed proof. The last byte of the bitmap is zero."
            );
        }
        let num_non_default_siblings = bitmap.iter().fold(0, |total, x| total + x.count_ones());
        ensure!(
            num_non_default_siblings as usize == proto_proof.non_default_siblings.len(),
            "Malformed proof. Bitmap indicated {} non-default siblings. Found {} siblings.",
            num_non_default_siblings,
            proto_proof.non_default_siblings.len()
        );

        let mut proto_siblings = proto_proof.non_default_siblings.into_iter();
        // Iterate from the MSB of the first byte to the rightmost 1-bit in the bitmap. If a bit is
        // set, the corresponding sibling is non-default and we take the sibling from
        // proto_siblings. Otherwise the sibling on this position is default.
        let siblings: Result<Vec<_>> = SparseMerkleBitmap::new(bitmap)
            .iter()
            .map(|x| {
                if x {
                    let hash_bytes = proto_siblings
                        .next()
                        .expect("Unexpected number of siblings.");
                    HashValue::from_slice(&hash_bytes)
                } else {
                    Ok(*SPARSE_MERKLE_PLACEHOLDER_HASH)
                }
            })
            .collect();

        Ok(SparseMerkleProof::new(leaf, siblings?))
    }
}

impl From<SparseMerkleProof> for crate::proto::types::SparseMerkleProof {
    fn from(proof: SparseMerkleProof) -> Self {
        let mut proto_proof = Self::default();
        // If a leaf is present, we write the key and value hash as a single byte array of 64
        // bytes. Otherwise we write an empty byte array.
        if let Some((key, value_hash)) = proof.leaf {
            proto_proof.leaf.extend_from_slice(key.as_ref());
            proto_proof.leaf.extend_from_slice(value_hash.as_ref());
        }
        // Iterate over all siblings. For each non-default sibling, add to protobuf struct and set
        // the corresponding bit in the bitmap.
        let bitmap: SparseMerkleBitmap = proof
            .siblings
            .into_iter()
            .map(|sibling| {
                if sibling != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                    proto_proof.non_default_siblings.push(sibling.to_vec());
                    true
                } else {
                    false
                }
            })
            .collect();
        proto_proof.bitmap = bitmap.into();
        proto_proof
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
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl TryFrom<crate::proto::types::AccumulatorConsistencyProof> for AccumulatorConsistencyProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccumulatorConsistencyProof) -> Result<Self> {
        let subtrees = proto_proof
            .subtrees
            .into_iter()
            .map(|hash_bytes| HashValue::from_slice(&hash_bytes))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(subtrees))
    }
}

impl From<AccumulatorConsistencyProof> for crate::proto::types::AccumulatorConsistencyProof {
    fn from(proof: AccumulatorConsistencyProof) -> Self {
        Self {
            subtrees: proof.subtrees.iter().map(HashValue::to_vec).collect(),
        }
    }
}

/// The complete proof used to authenticate a `SignedTransaction` object.  This structure consists
/// of an `AccumulatorProof` from `LedgerInfo` to `TransactionInfo` the verifier needs to verify
/// the correctness of the `TransactionInfo` object, and the `TransactionInfo` object that is
/// supposed to match the `SignedTransaction`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct SignedTransactionProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    transaction_info: TransactionInfo,
}

impl SignedTransactionProof {
    /// Constructs a new `SignedTransactionProof` object using given
    /// `ledger_info_to_transaction_info_proof`.
    pub fn new(
        ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,
        transaction_info: TransactionInfo,
    ) -> Self {
        SignedTransactionProof {
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

    /// Verifies that a `SignedTransaction` with hash value of `signed_transaction_hash`
    /// is the version `transaction_version` transaction in the ledger using the provided proof.
    /// If event_root_hash is provided, it's also verified against the proof.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        signed_transaction_hash: HashValue,
        event_root_hash: Option<HashValue>,
        transaction_version: Version,
    ) -> Result<()> {
        ensure!(
            signed_transaction_hash == self.transaction_info.signed_transaction_hash(),
            "The hash of signed transaction does not match the transaction info in proof. \
             Transaction hash: {:x}. Transaction hash provided by proof: {:x}.",
            signed_transaction_hash,
            self.transaction_info.signed_transaction_hash()
        );

        if let Some(event_root_hash) = event_root_hash {
            ensure!(
                event_root_hash == self.transaction_info.event_root_hash(),
                "Event root hash ({}) doesn't match that in the transaction info ({}).",
                event_root_hash,
                self.transaction_info.event_root_hash(),
            );
        }

        verify_transaction_info(
            ledger_info,
            transaction_version,
            &self.transaction_info,
            &self.ledger_info_to_transaction_info_proof,
        )?;
        Ok(())
    }
}

impl TryFrom<crate::proto::types::SignedTransactionProof> for SignedTransactionProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::SignedTransactionProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;

        Ok(SignedTransactionProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
        ))
    }
}

impl From<SignedTransactionProof> for crate::proto::types::SignedTransactionProof {
    fn from(proof: SignedTransactionProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof.into(),
            ),
            transaction_info: Some(proof.transaction_info.into()),
        }
    }
}

/// The complete proof used to authenticate the state of an account. This structure consists of the
/// `AccumulatorProof` from `LedgerInfo` to `TransactionInfo`, the `TransactionInfo` object and the
/// `SparseMerkleProof` from state root to the account.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct AccountStateProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    transaction_info: TransactionInfo,

    /// The sparse merkle proof from state root to the account state.
    transaction_info_to_account_proof: SparseMerkleProof,
}

impl AccountStateProof {
    /// Constructs a new `AccountStateProof` using given `ledger_info_to_transaction_info_proof`,
    /// `transaction_info` and `transaction_info_to_account_proof`.
    pub fn new(
        ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,
        transaction_info: TransactionInfo,
        transaction_info_to_account_proof: SparseMerkleProof,
    ) -> Self {
        AccountStateProof {
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_account_proof,
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
            self.transaction_info.state_root_hash(),
            account_address_hash,
            account_state_blob,
        )?;

        verify_transaction_info(
            ledger_info,
            state_version,
            &self.transaction_info,
            &self.ledger_info_to_transaction_info_proof,
        )?;
        Ok(())
    }
}

impl TryFrom<crate::proto::types::AccountStateProof> for AccountStateProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccountStateProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;
        let transaction_info_to_account_proof = proto_proof
            .transaction_info_to_account_proof
            .ok_or_else(|| format_err!("Missing transaction_info_to_account_proof"))?
            .try_into()?;

        Ok(AccountStateProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_account_proof,
        ))
    }
}

impl From<AccountStateProof> for crate::proto::types::AccountStateProof {
    fn from(proof: AccountStateProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof.into(),
            ),
            transaction_info: Some(proof.transaction_info.into()),
            transaction_info_to_account_proof: Some(proof.transaction_info_to_account_proof.into()),
        }
    }
}

/// The complete proof used to authenticate a contract event. This structure consists of the
/// `AccumulatorProof` from `LedgerInfo` to `TransactionInfo`, the `TransactionInfo` object and the
/// `AccumulatorProof` from event accumulator root to the event.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct EventProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    transaction_info: TransactionInfo,

    /// The accumulator proof from event root to the actual event.
    transaction_info_to_event_proof: EventAccumulatorProof,
}

impl EventProof {
    /// Constructs a new `EventProof` using given `ledger_info_to_transaction_info_proof`,
    /// `transaction_info` and `transaction_info_to_event_proof`.
    pub fn new(
        ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,
        transaction_info: TransactionInfo,
        transaction_info_to_event_proof: EventAccumulatorProof,
    ) -> Self {
        EventProof {
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_event_proof,
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

    /// Returns the `transaction_info_to_event_proof` object in this proof.
    pub fn transaction_info_to_event_proof(&self) -> &EventAccumulatorProof {
        &self.transaction_info_to_event_proof
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
            self.transaction_info.event_root_hash(),
            event_hash,
            event_version_within_transaction,
        )?;

        verify_transaction_info(
            ledger_info,
            transaction_version,
            &self.transaction_info,
            &self.ledger_info_to_transaction_info_proof,
        )?;

        Ok(())
    }
}

impl TryFrom<crate::proto::types::EventProof> for EventProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::EventProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;
        let transaction_info_to_event_proof = proto_proof
            .transaction_info_to_event_proof
            .ok_or_else(|| format_err!("Missing transaction_info_to_account_proof"))?
            .try_into()?;

        Ok(EventProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_event_proof,
        ))
    }
}

impl From<EventProof> for crate::proto::types::EventProof {
    fn from(proof: EventProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof.into(),
            ),
            transaction_info: Some(proof.transaction_info.into()),
            transaction_info_to_event_proof: Some(proof.transaction_info_to_event_proof.into()),
        }
    }
}

mod bitmap {
    /// The bitmap indicating which siblings are default in a compressed accumulator proof. 1 means
    /// non-default and 0 means default.  The LSB corresponds to the sibling at the bottom of the
    /// accumulator. The leftmost 1-bit corresponds to the sibling at the top of the accumulator,
    /// since this one is always non-default.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct AccumulatorBitmap(u64);

    impl AccumulatorBitmap {
        pub fn new(bitmap: u64) -> Self {
            AccumulatorBitmap(bitmap)
        }

        pub fn iter(self) -> AccumulatorBitmapIterator {
            AccumulatorBitmapIterator::new(self.0)
        }
    }

    impl std::convert::From<AccumulatorBitmap> for u64 {
        fn from(bitmap: AccumulatorBitmap) -> u64 {
            bitmap.0
        }
    }

    /// Given a u64 bitmap, this iterator generates one bit at a time starting from the leftmost
    /// 1-bit.
    pub struct AccumulatorBitmapIterator {
        bitmap: AccumulatorBitmap,
        mask: u64,
    }

    impl AccumulatorBitmapIterator {
        fn new(bitmap: u64) -> Self {
            let num_leading_zeros = bitmap.leading_zeros();
            let mask = if num_leading_zeros >= 64 {
                0
            } else {
                1 << (63 - num_leading_zeros)
            };
            AccumulatorBitmapIterator {
                bitmap: AccumulatorBitmap(bitmap),
                mask,
            }
        }
    }

    impl std::iter::Iterator for AccumulatorBitmapIterator {
        type Item = bool;

        fn next(&mut self) -> Option<bool> {
            if self.mask == 0 {
                return None;
            }
            let ret = self.bitmap.0 & self.mask != 0;
            self.mask >>= 1;
            Some(ret)
        }
    }

    impl std::iter::FromIterator<bool> for AccumulatorBitmap {
        fn from_iter<I>(iter: I) -> Self
        where
            I: std::iter::IntoIterator<Item = bool>,
        {
            let mut bitmap = 0;
            for (i, bit) in iter.into_iter().enumerate() {
                if i == 0 {
                    assert!(bit, "The first bit should always be set.");
                } else if i > 63 {
                    panic!("Trying to put more than 64 bits in AccumulatorBitmap.");
                }
                bitmap <<= 1;
                bitmap |= bit as u64;
            }
            AccumulatorBitmap::new(bitmap)
        }
    }

    /// The bitmap indicating which siblings are default in a compressed sparse merkle proof. 1
    /// means non-default and 0 means default.  The MSB of the first byte corresponds to the
    /// sibling at the top of the Sparse Merkle Tree. The rightmost 1-bit of the last byte
    /// corresponds to the sibling at the bottom, since this one is always non-default.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct SparseMerkleBitmap(Vec<u8>);

    impl SparseMerkleBitmap {
        pub fn new(bitmap: Vec<u8>) -> Self {
            SparseMerkleBitmap(bitmap)
        }

        pub fn iter(&self) -> SparseMerkleBitmapIterator {
            SparseMerkleBitmapIterator::new(&self.0)
        }
    }

    impl std::convert::From<SparseMerkleBitmap> for Vec<u8> {
        fn from(bitmap: SparseMerkleBitmap) -> Vec<u8> {
            bitmap.0
        }
    }

    /// Given a `Vec<u8>` bitmap, this iterator generates one bit at a time starting from the MSB
    /// of the first byte. All trailing zeros of the last byte are discarded.
    pub struct SparseMerkleBitmapIterator<'a> {
        bitmap: &'a [u8],
        index: usize,
        len: usize,
    }

    impl<'a> SparseMerkleBitmapIterator<'a> {
        fn new(bitmap: &'a [u8]) -> Self {
            match bitmap.last() {
                Some(last_byte) => {
                    assert_ne!(
                        *last_byte, 0,
                        "The last byte of the bitmap should never be zero."
                    );
                    SparseMerkleBitmapIterator {
                        bitmap,
                        index: 0,
                        len: bitmap.len() * 8 - last_byte.trailing_zeros() as usize,
                    }
                }
                None => SparseMerkleBitmapIterator {
                    bitmap,
                    index: 0,
                    len: 0,
                },
            }
        }
    }

    impl<'a> std::iter::Iterator for SparseMerkleBitmapIterator<'a> {
        type Item = bool;

        fn next(&mut self) -> Option<bool> {
            // We are past the last useful bit.
            if self.index >= self.len {
                return None;
            }

            let pos = self.index / 8;
            let bit = self.index % 8;
            let ret = self.bitmap[pos] >> (7 - bit) & 1 != 0;
            self.index += 1;
            Some(ret)
        }
    }

    impl std::iter::FromIterator<bool> for SparseMerkleBitmap {
        fn from_iter<I>(iter: I) -> Self
        where
            I: std::iter::IntoIterator<Item = bool>,
        {
            let mut bitmap = vec![];
            for (i, bit) in iter.into_iter().enumerate() {
                let pos = i % 8;
                if pos == 0 {
                    bitmap.push(0);
                }
                let last_byte = bitmap
                    .last_mut()
                    .expect("The bitmap vector should not be empty");
                *last_byte |= (bit as u8) << (7 - pos);
            }
            SparseMerkleBitmap::new(bitmap)
        }
    }
}
