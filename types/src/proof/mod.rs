// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod accumulator;
pub mod definition;
pub mod position;
#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_proof;

#[cfg(test)]
mod unit_tests;

use crate::{
    ledger_info::LedgerInfo,
    transaction::{TransactionInfo, Version},
};
use anyhow::{ensure, Result};
use libra_crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, SparseMerkleInternalHasher,
        TestOnlyHasher, TransactionAccumulatorHasher,
    },
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
use std::marker::PhantomData;

pub use self::definition::{
    AccountStateProof, AccumulatorConsistencyProof, AccumulatorProof, AccumulatorRangeProof,
    EventAccumulatorProof, EventProof, SparseMerkleProof, SparseMerkleRangeProof,
    TransactionAccumulatorProof, TransactionAccumulatorRangeProof, TransactionListProof,
    TransactionProof,
};

#[cfg(any(test, feature = "fuzzing"))]
pub use self::definition::{TestAccumulatorProof, TestAccumulatorRangeProof};

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

#[derive(CryptoHasher)]
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
    type Hasher = SparseMerkleLeafNodeHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.key.as_ref());
        state.write(self.value_hash.as_ref());
        state.finish()
    }
}
