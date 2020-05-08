// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod test;

use crate::{
    ledger_store::{LedgerStore, TransactionInfoIter},
    state_store::StateStore,
    transaction_store::{TransactionIter, TransactionStore},
};
use anyhow::Result;
use jellyfish_merkle::iterator::JellyfishMerkleIterator;
use libra_crypto::hash::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof, transaction::Version,
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

    /// Gets an iterator that yields a range of transaction infos.
    pub fn get_transaction_info_iter(
        &self,
        start_version: Version,
        num_transaction_infos: u64,
    ) -> Result<TransactionInfoIter> {
        self.ledger_store
            .get_transaction_info_iter(start_version, num_transaction_infos)
    }

    /// Gets an iterator which can yield all accounts in the state tree.
    pub fn get_account_iter(
        &self,
        version: Version,
    ) -> Result<Box<dyn Iterator<Item = Result<(HashValue, AccountStateBlob)>> + Send>> {
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
}
