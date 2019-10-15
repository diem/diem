// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod accumulator;
pub mod definition;
pub mod position;
#[cfg(any(test, feature = "testing"))]
pub mod proptest_proof;

#[cfg(test)]
#[path = "unit_tests/proof_test.rs"]
mod proof_test;

use self::accumulator::InMemoryAccumulator;
use crate::{
    contract_event::ContractEvent,
    ledger_info::LedgerInfo,
    transaction::{TransactionInfo, TransactionListWithProof, Version},
};
use crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, SparseMerkleInternalHasher,
        SparseMerkleLeafHasher, TestOnlyHasher, TransactionAccumulatorHasher,
    },
    HashValue,
};
use failure::prelude::*;
use std::{collections::VecDeque, marker::PhantomData};

pub use self::definition::{
    AccountStateProof, AccumulatorConsistencyProof, AccumulatorProof, EventAccumulatorProof,
    EventProof, SignedTransactionProof, SparseMerkleProof, TransactionAccumulatorProof,
};

#[cfg(any(test, feature = "testing"))]
pub use self::definition::TestAccumulatorProof;

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
        itertools::zip_eq(event_lists, transaction_and_infos)
            .map(|(events, (_txn, txn_info))| {
                let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
                let event_root_hash =
                    InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes)
                        .root_hash();
                ensure!(
                    event_root_hash == txn_info.event_root_hash(),
                    "Some event root hash calculated doesn't match that carried on the \
                     transaction info.",
                );
                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;
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
    ledger_info_to_transaction_info_proof: &TransactionAccumulatorProof,
) -> Result<()> {
    ensure!(
        transaction_version <= ledger_info.version(),
        "Transaction version {} is newer than LedgerInfo version {}.",
        transaction_version,
        ledger_info.version(),
    );

    let transaction_info_hash = transaction_info.hash();
    ledger_info_to_transaction_info_proof.verify(
        ledger_info.transaction_accumulator_hash(),
        transaction_info_hash,
        transaction_version,
    )?;

    Ok(())
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
