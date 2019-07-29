// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod definition;
pub mod position;
#[cfg(any(test, feature = "testing"))]
pub mod proptest_proof;
pub mod treebits;

#[cfg(test)]
#[path = "unit_tests/proof_test.rs"]
mod proof_test;
#[cfg(test)]
mod unit_tests;

use crate::{
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    ledger_info::LedgerInfo,
    transaction::{TransactionInfo, TransactionListWithProof, Version},
};
use crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, SparseMerkleInternalHasher,
        SparseMerkleLeafHasher, TestOnlyHasher, TransactionAccumulatorHasher,
        ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
use failure::prelude::*;
use std::{collections::VecDeque, marker::PhantomData};

pub use crate::proof::definition::{
    AccountStateProof, AccumulatorProof, EventProof, SignedTransactionProof, SparseMerkleProof,
};

/// Verifies that a `SignedTransaction` with hash value of `signed_transaction_hash`
/// is the version `transaction_version` transaction in the ledger using the provided proof.
/// If event_root_hash is provided, it's also verified against the proof.
pub fn verify_signed_transaction(
    ledger_info: &LedgerInfo,
    signed_transaction_hash: HashValue,
    event_root_hash: Option<HashValue>,
    transaction_version: Version,
    signed_transaction_proof: &SignedTransactionProof,
) -> Result<()> {
    let transaction_info = signed_transaction_proof.transaction_info();

    ensure!(
        signed_transaction_hash == transaction_info.signed_transaction_hash(),
        "The hash of signed transaction does not match the transaction info in proof. \
         Transaction hash: {:x}. Transaction hash provided by proof: {:x}.",
        signed_transaction_hash,
        transaction_info.signed_transaction_hash()
    );

    if let Some(event_root_hash) = event_root_hash {
        ensure!(
            event_root_hash == transaction_info.event_root_hash(),
            "Event root hash ({}) doesn't match that in the transaction info ({}).",
            event_root_hash,
            transaction_info.event_root_hash(),
        );
    }

    verify_transaction_info(
        ledger_info,
        transaction_version,
        transaction_info,
        signed_transaction_proof.ledger_info_to_transaction_info_proof(),
    )?;
    Ok(())
}

/// Verifies that the state of an account at version `state_version` is correct using the provided
/// proof.  If `account_state_blob` is present, we expect the account to exist, otherwise we
/// expect the account to not exist.
pub fn verify_account_state(
    ledger_info: &LedgerInfo,
    state_version: Version,
    account_address_hash: HashValue,
    account_state_blob: &Option<AccountStateBlob>,
    account_state_proof: &AccountStateProof,
) -> Result<()> {
    let transaction_info = account_state_proof.transaction_info();

    verify_sparse_merkle_element(
        transaction_info.state_root_hash(),
        account_address_hash,
        account_state_blob,
        account_state_proof.transaction_info_to_account_proof(),
    )?;

    verify_transaction_info(
        ledger_info,
        state_version,
        transaction_info,
        account_state_proof.ledger_info_to_transaction_info_proof(),
    )?;
    Ok(())
}

/// Verifies that a given event is correct using provided proof.
pub(crate) fn verify_event(
    ledger_info: &LedgerInfo,
    event_hash: HashValue,
    transaction_version: Version,
    event_version_within_transaction: Version,
    event_proof: &EventProof,
) -> Result<()> {
    let transaction_info = event_proof.transaction_info();

    verify_event_accumulator_element(
        transaction_info.event_root_hash(),
        event_hash,
        event_version_within_transaction,
        event_proof.transaction_info_to_event_proof(),
    )?;

    verify_transaction_info(
        ledger_info,
        transaction_version,
        transaction_info,
        event_proof.ledger_info_to_transaction_info_proof(),
    )?;

    Ok(())
}

pub(crate) fn verify_transaction_list(
    ledger_info: &LedgerInfo,
    transaction_list_with_proof: &TransactionListWithProof,
) -> Result<()> {
    let (transaction_and_infos, event_lists, first_transaction_version, first_proof, last_proof) = (
        &transaction_list_with_proof.transaction_and_infos,
        transaction_list_with_proof.events.as_ref(),
        transaction_list_with_proof.first_transaction_version,
        transaction_list_with_proof
            .proof_of_first_transaction
            .as_ref(),
        transaction_list_with_proof
            .proof_of_last_transaction
            .as_ref(),
    );

    let num_txns = transaction_and_infos.len();
    if let Some(event_lists) = event_lists {
        ensure!(
            num_txns == event_lists.len(),
            "Number of the event lists doesn't match that of the transactions: {} vs {}",
            num_txns,
            event_lists.len(),
        );
    }

    // 1. Empty list;
    if num_txns == 0 {
        ensure!(
            first_proof.is_none(),
            "List is empty but proof of the first transaction is provided."
        );
        ensure!(
            last_proof.is_none(),
            "List is empty but proof of the last transaction is provided."
        );
        ensure!(
            first_transaction_version.is_none(),
            "List is empty but expecting first transaction to exist.",
        );
        return Ok(());
    }

    // 2. Non-empty list.
    let first_version = first_transaction_version.ok_or_else(|| {
        format_err!("Invalid TransactionListWithProof: First_transaction_version is None.")
    })?;
    let siblings_of_first_txn = first_proof
        .ok_or_else(|| {
            format_err!("Invalid TransactionListWithProof: First transaction proof is None")
        })?
        .siblings();
    let siblings_of_last_txn = match (num_txns, last_proof) {
        (1, None) => siblings_of_first_txn,
        (_, Some(last_proof)) => last_proof.siblings(),
        _ => bail!(
            "Invalid TransactionListWithProof: Last transaction proof is_none:{}, num_txns:{}",
            last_proof.is_none(),
            num_txns
        ),
    };

    // Verify event root hashes match what is carried on the transaction infos.
    if let Some(event_lists) = event_lists {
        itertools::zip_eq(event_lists, transaction_and_infos).map(|(events, (_txn, txn_info))| {
            let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
            let event_root_hash = get_accumulator_root_hash::<EventAccumulatorHasher>(&event_hashes);
            ensure!(
                event_root_hash == txn_info.event_root_hash(),
                "Some event root hash calculated doesn't match that carried on the transaction info.",
            );
            Ok(())
        }).collect::<Result<Vec<_>>>()?;
    }

    // Get the hashes of all nodes at the accumulator leaf level.
    let mut hashes = transaction_and_infos
        .iter()
        .map(|(txn, txn_info)| {
            // Verify all transaction_infos and signed_transactions are consistent.
            ensure!(
                txn.hash() == txn_info.signed_transaction_hash(),
                "Some hash of signed transaction does not match the corresponding transaction info in proof"
            );
            Ok(txn_info.hash())
        })
        .collect::<Result<VecDeque<_>>>()?;

    let mut first_index = first_version;

    // Verify level by level from the leaf level upwards.
    for (first_sibling, last_sibling) in siblings_of_first_txn
        .iter()
        .zip(siblings_of_last_txn.iter())
        .rev()
    {
        assert!(!hashes.is_empty());
        let num_nodes = hashes.len();

        if num_nodes > 1 {
            let last_index = first_index + num_nodes as u64 - 1;
            if last_index % 2 == 0 {
                // if `last_index` is even, it is the left child of its parent so the sibling is not
                // in `hashes`, we have to append it to `hashes` generate parent nodes' hashes.
                hashes.push_back(*last_sibling);
            } else {
                // Otherwise, the sibling should be the second to last hash.
                // Note: if we check `first_index` first we cannot use num_nodes to index because
                // hashes length may change.
                ensure!(hashes[num_nodes - 2] == *last_sibling,
                        "Invalid TransactionListWithProof: Last transaction proof doesn't match provided siblings");
            }
            // We haven't reached the first common ancester of all transactions in the list.
            if first_index % 2 == 0 {
                // if `first_index` is even, it is the left child of its parent so the sibling must
                // be the next node.
                ensure!(hashes[1] == *first_sibling,
                            "Invalid TransactionListWithProof: First transaction proof doesn't match provided siblings");
            } else {
                // Otherwise, the sibling is not in `hashes`, we have to prepend it to `hashes` to
                // generate parent nodes' hashes.
                hashes.push_front(*first_sibling);
            }
        } else {
            // We have reached the first common ancestor of all the transactions in the list.
            ensure!(
                first_sibling == last_sibling,
                "Invalid TransactionListWithProof: Either proof is invalid."
            );
            if first_index % 2 == 0 {
                hashes.push_back(*first_sibling);
            } else {
                hashes.push_front(*first_sibling);
            }
        }
        let mut hash_iter = hashes.into_iter();
        let mut parent_hashes = VecDeque::new();
        while let Some(left) = hash_iter.next() {
            let right = hash_iter.next().expect("Can't be None");
            parent_hashes.push_back(
                MerkleTreeInternalNode::<TransactionAccumulatorHasher>::new(left, right).hash(),
            )
        }
        hashes = parent_hashes;
        // The parent node index at its level should be floor(index / 2)
        first_index /= 2;
    }
    assert!(hashes.len() == 1);
    let expected_root_hash = ledger_info.transaction_accumulator_hash();
    ensure!(
        hashes[0] == expected_root_hash,
        "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
        hashes[0],
        expected_root_hash
    );
    Ok(())
}

/// Verifies that a given `transaction_info` exists in the ledger using provided proof.
fn verify_transaction_info(
    ledger_info: &LedgerInfo,
    transaction_version: Version,
    transaction_info: &TransactionInfo,
    ledger_info_to_transaction_info_proof: &AccumulatorProof,
) -> Result<()> {
    ensure!(
        transaction_version <= ledger_info.version(),
        "Transaction version {} is newer than LedgerInfo version {}.",
        transaction_version,
        ledger_info.version(),
    );

    let transaction_info_hash = transaction_info.hash();
    verify_transaction_accumulator_element(
        ledger_info.transaction_accumulator_hash(),
        transaction_info_hash,
        transaction_version,
        ledger_info_to_transaction_info_proof,
    )?;

    Ok(())
}

/// Verifies an element whose hash is `element_hash` and version is `element_version` exists in the
/// accumulator whose root hash is `expected_root_hash` using the provided proof.
fn verify_accumulator_element<H: Clone + CryptoHasher>(
    expected_root_hash: HashValue,
    element_hash: HashValue,
    element_index: u64,
    accumulator_proof: &AccumulatorProof,
) -> Result<()> {
    let siblings = accumulator_proof.siblings();
    ensure!(
        siblings.len() <= 63,
        "Accumulator proof has more than 63 ({}) siblings.",
        siblings.len()
    );

    let actual_root_hash = siblings
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

pub(crate) fn get_accumulator_root_hash<H: Clone + CryptoHasher>(
    element_hashes: &[HashValue],
) -> HashValue {
    if element_hashes.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let mut next_level: Vec<HashValue>;
    let mut current_level: &[HashValue] = element_hashes;

    while current_level.len() > 1 {
        next_level = current_level
            .chunks(2)
            .map(|t| {
                if t.len() == 2 {
                    MerkleTreeInternalNode::<H>::new(t[0], t[1]).hash()
                } else {
                    MerkleTreeInternalNode::<H>::new(t[0], *ACCUMULATOR_PLACEHOLDER_HASH).hash()
                }
            })
            .collect();

        current_level = &next_level;
    }

    current_level[0]
}

type AccumulatorElementVerifier = fn(
    expected_root_hash: HashValue,
    element_hash: HashValue,
    element_version: Version,
    accumulator_proof: &AccumulatorProof,
) -> Result<()>;

#[allow(non_upper_case_globals)]
pub const verify_event_accumulator_element: AccumulatorElementVerifier =
    verify_accumulator_element::<EventAccumulatorHasher>;

#[allow(non_upper_case_globals)]
pub const verify_transaction_accumulator_element: AccumulatorElementVerifier =
    verify_accumulator_element::<TransactionAccumulatorHasher>;

#[allow(non_upper_case_globals)]
pub const verify_test_accumulator_element: AccumulatorElementVerifier =
    verify_accumulator_element::<TestOnlyHasher>;

/// If `element_blob` is present, verifies an element whose key is `element_key` and value
/// is `element_blob` exists in the Sparse Merkle Tree using the provided proof.
/// Otherwise verifies the proof is a valid non-inclusion proof that shows this key doesn't exist
/// in the tree.
pub fn verify_sparse_merkle_element(
    expected_root_hash: HashValue,
    element_key: HashValue,
    element_blob: &Option<AccountStateBlob>,
    sparse_merkle_proof: &SparseMerkleProof,
) -> Result<()> {
    let siblings = sparse_merkle_proof.siblings();
    ensure!(
        siblings.len() <= HashValue::LENGTH_IN_BITS,
        "Sparse Merkle Tree proof has more than {} ({}) siblings.",
        HashValue::LENGTH_IN_BITS,
        siblings.len()
    );

    match (element_blob, sparse_merkle_proof.leaf()) {
        (Some(blob), Some((proof_key, proof_value_hash))) => {
            // This is an inclusion proof, so the key and value hash provided in the proof should
            // match element_key and element_value_hash.
            ensure!(
                element_key == proof_key,
                "Keys do not match. Key in proof: {:x}. Expected key: {:x}.",
                proof_key,
                element_key
            );
            let hash = blob.hash();
            ensure!(
                hash == proof_value_hash,
                "Value hashes do not match. Value hash in proof: {:x}. Expected value hash: {:x}",
                proof_value_hash,
                hash,
            );
        }
        (Some(_blob), None) => bail!("Expected inclusion proof. Found non-inclusion proof."),
        (None, Some((proof_key, _))) => {
            // The proof intends to show that proof_key is the only key in a subtree and
            // element_key would have ended up in the same subtree if it existed in the tree.
            ensure!(
                element_key != proof_key,
                "Expected non-inclusion proof, but key exists in proof."
            );
            ensure!(
                element_key.common_prefix_bits_len(proof_key) >= siblings.len(),
                "Key would not have ended up in the subtree where the provided key in proof is \
                 the only existing key, if it existed. So this is not a valid non-inclusion proof."
            );
        }
        (None, None) => (),
    }

    let leaf_hash = match sparse_merkle_proof.leaf() {
        Some((key, value_hash)) => SparseMerkleLeafNode::new(key, value_hash).hash(),
        None => *SPARSE_MERKLE_PLACEHOLDER_HASH,
    };
    let actual_root_hash = siblings
        .iter()
        .rev()
        .zip(
            element_key
                .iter_bits()
                .rev()
                .skip(HashValue::LENGTH_IN_BITS - siblings.len()),
        )
        .fold(leaf_hash, |hash, (sibling_hash, bit)| {
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
        expected_root_hash
    );

    Ok(())
}

/// Computes the root hash of an accumulator using the root hashes of the frozen subtrees and their
/// corresponding sibling nodes. As an example, given the following accumulator that has 10
/// elements, the frozen subtree roots and the corresponding siblings are annotated below.
///
/// ```text
///                                        root
///                                      /      \
///                                    /          \
///                                  /              \
///                                /                  \
///                              /                      \
///                            /                          \
///                          /                              \
///                        /                                  \
/// frozen_subtree_roots[0]                                    o
///                    /   \                                  / \
///                   /     \                                /   \
///                  o       o                              o     siblings[0]
///                 / \     / \                            / \
///                o   o   o   o    frozen_subtree_roots[1]   sibling[1]
///               / \ / \ / \ / \                       / \
///               o o o o o o o o                       o o
/// ```
///
/// This function has at least two possible use cases:
///   1) Given an accumulator represented by all of its frozen subtrees, we want to compute the
///      root hash by combining the subtrees with placeholder nodes. In this case the caller can
///      use a list of placeholder nodes as the `siblings` parameter.
///   2) To prove that an accumulator A can be obtained by appending some elements to another
///      accumulator B, the proof can provide a list of siblings and we can combine B's frozen
///      subtrees with these siblings.
pub fn get_accumulator_root_hash_by_frozen_subtrees_and_siblings<H: CryptoHasher>(
    frozen_subtree_roots: &[HashValue],
    num_leaves: u64,
    siblings: &[HashValue],
) -> HashValue {
    // Special cases when there is zero or one subtree.
    match frozen_subtree_roots.len() {
        0 => return *ACCUMULATOR_PLACEHOLDER_HASH,
        1 => return frozen_subtree_roots[0],
        _ => (),
    }

    // For accumulators that have more than one frozen subtrees, the rightmost/smallest frozen
    // subtree will always be combined with a sibling on the right.
    let mut subtree_iter = frozen_subtree_roots.iter().rev();
    let mut sibling_iter = siblings.iter().rev();
    let mut current_hash = MerkleTreeInternalNode::<H>::new(
        *subtree_iter.next().expect("Missing subtree hash."),
        *sibling_iter.next().expect("Missing sibling hash."),
    )
    .hash();

    // Remove the trailing zeros since they do not matter. Also remove one more bit since we have
    // processed the lowest level.
    let mut bitmap = num_leaves >> (num_leaves.trailing_zeros() + 1);
    // Using the above example, now the bitmap becomes 0b10 (was 0b1010 originally). We check each
    // bit from right to left.  If a bit is 0, it means there is no existing subtree on this level,
    // so we combine current hash with a sibling.  Otherwise we combine it with the corresponding
    // subtree.
    while bitmap > 0 {
        current_hash = if bitmap & 1 == 0 {
            MerkleTreeInternalNode::<H>::new(
                current_hash,
                *sibling_iter.next().expect("Missing sibling hash."),
            )
        } else {
            MerkleTreeInternalNode::<H>::new(
                *subtree_iter.next().expect("Missing subtree hash."),
                current_hash,
            )
        }
        .hash();
        bitmap >>= 1;
    }

    current_hash
}

/// For an accumulator that has `num_leaves` leaves, computes the total number of corresponding
/// siblings that can be used to compute root hash. For example, the following accumulator with 5
/// leaves requires 2 siblings to compute its root hash.
/// ```text
///            o
///           / \
///          /   \
///         /     \
///        o       o
///       / \     / \
///      o   o   o   siblings[0]
///     / \ / \ / \
///     o o o o o siblings[1]
/// ```
pub fn get_num_siblings_for_frozen_subtrees(num_leaves: u64) -> usize {
    if num_leaves == 0 {
        return 0;
    }

    // For any accumulator, if we add two children to every existing leaf, the positions of the
    // frozen subtrees do no change, so the trailing zeros do not matter.
    let num_leaves = num_leaves >> num_leaves.trailing_zeros();
    // Because every time we combine two hashes to compute its parent, the tree can be made one
    // level smaller. So the total number of hashes needed to compute the root hash is
    // `num_levels`.
    let num_levels = num_leaves.next_power_of_two().trailing_zeros() + 1;
    let num_frozen_subtrees = num_leaves.count_ones();
    (num_levels - num_frozen_subtrees) as usize
}

pub struct MerkleTreeInternalNode<H> {
    left_child: HashValue,
    right_child: HashValue,
    hasher: PhantomData<H>,
}

impl<H: CryptoHasher> MerkleTreeInternalNode<H> {
    pub fn new(left_child: HashValue, right_child: HashValue) -> Self {
        Self {
            left_child,
            right_child,
            hasher: PhantomData,
        }
    }
}

impl<H: CryptoHasher> CryptoHash for MerkleTreeInternalNode<H> {
    type Hasher = H;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.left_child.as_ref());
        state.write(self.right_child.as_ref());
        state.finish()
    }
}

pub type SparseMerkleInternalNode = MerkleTreeInternalNode<SparseMerkleInternalHasher>;
pub type TransactionAccumulatorInternalNode = MerkleTreeInternalNode<TransactionAccumulatorHasher>;
pub type EventAccumulatorInternalNode = MerkleTreeInternalNode<EventAccumulatorHasher>;
pub type TestAccumulatorInternalNode = MerkleTreeInternalNode<TestOnlyHasher>;

pub struct SparseMerkleLeafNode {
    key: HashValue,
    value_hash: HashValue,
}

impl SparseMerkleLeafNode {
    pub fn new(key: HashValue, value_hash: HashValue) -> Self {
        SparseMerkleLeafNode { key, value_hash }
    }
}

impl CryptoHash for SparseMerkleLeafNode {
    type Hasher = SparseMerkleLeafHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.key.as_ref());
        state.write(self.value_hash.as_ref());
        state.finish()
    }
}
