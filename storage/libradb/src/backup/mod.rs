// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod test;

use crate::{
    ledger_store::LedgerStore,
    state_store::StateStore,
    transaction_store::{TransactionIter, TransactionStore},
};
use anyhow::Result;
use jellyfish_merkle::iterator::JellyfishMerkleIterator;
use libra_crypto::hash::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfoWithSignatures,
    proof::{SparseMerkleRangeProof, TransactionInfoWithProof},
    transaction::Version,
};
use std::sync::Arc;

/// `BackupHandler` provides functionalities for LibraDB data backup.
#[derive(Clone)]
pub struct BackupHandler {
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl BackupHandler {
    pub(crate) fn new(
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            ledger_store,
            transaction_store,
            state_store,
        }
    }

    /// Gets an iterator that yields a range of transactions.
    pub fn get_transaction_iter(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<TransactionIter> {
        self.transaction_store
            .get_transaction_iter(start_version, num_transactions)
    }

    /// Gets an iterator which can yield all accounts in the state tree.
    pub fn get_account_iter(
        &self,
        version: Version,
    ) -> Result<Box<dyn Iterator<Item = Result<(HashValue, AccountStateBlob)>> + Send + Sync>> {
        let iterator = JellyfishMerkleIterator::new(
            Arc::clone(&self.state_store),
            version,
            HashValue::zero(),
        )?;
        Ok(Box::new(iterator))
    }

    /// Gets the proof that proves a range of accounts.
    pub fn get_account_state_range_proof(
        &self,
        rightmost_key: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        self.state_store
            .get_account_state_range_proof(rightmost_key, version)
    }

    /// Gets the latest version and state root hash.
    pub fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let (version, txn_info) = self.ledger_store.get_latest_transaction_info()?;
        Ok((version, txn_info.state_root_hash()))
    }

    /// Gets the proof of the state root at specified version.
    /// N.B. the `LedgerInfo` returned will always be in the same epoch of the version.
    pub fn get_state_root_proof(
        &self,
        version: Version,
    ) -> Result<(TransactionInfoWithProof, LedgerInfoWithSignatures)> {
        let epoch = self.ledger_store.get_epoch(version)?;
        let ledger_info = self.ledger_store.get_latest_ledger_info_in_epoch(epoch)?;
        let txn_info = self
            .ledger_store
            .get_transaction_info_with_proof(version, ledger_info.ledger_info().version())?;

        Ok((txn_info, ledger_info))
    }
}
