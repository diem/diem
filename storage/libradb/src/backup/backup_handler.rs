// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ledger_store::LedgerStore,
    metrics::{
        BACKUP_EPOCH_ENDING_EPOCH, BACKUP_STATE_SNAPSHOT_LEAF_IDX, BACKUP_STATE_SNAPSHOT_VERSION,
        BACKUP_TXN_VERSION,
    },
    state_store::StateStore,
    transaction_store::TransactionStore,
};
use anyhow::{anyhow, ensure, Result};
use itertools::zip_eq;
use libra_crypto::hash::HashValue;
use libra_jellyfish_merkle::iterator::JellyfishMerkleIterator;
use libra_types::{
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfoWithSignatures,
    proof::{SparseMerkleRangeProof, TransactionAccumulatorRangeProof, TransactionInfoWithProof},
    transaction::{Transaction, TransactionInfo, Version},
};
use serde::{export::Formatter, Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

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
    pub fn get_transaction_iter<'a>(
        &'a self,
        start_version: Version,
        num_transactions: usize,
    ) -> Result<impl Iterator<Item = Result<(Transaction, TransactionInfo)>> + 'a> {
        let txn_iter = self
            .transaction_store
            .get_transaction_iter(start_version, num_transactions)?;
        let txn_info_iter = self
            .ledger_store
            .get_transaction_info_iter(start_version, num_transactions)?;
        let zipped = zip_eq(txn_iter, txn_info_iter).enumerate().map(
            move |(idx, (txn_res, txn_info_res))| {
                BACKUP_TXN_VERSION.set((start_version + idx as u64) as i64);
                Ok((txn_res?, txn_info_res?))
            },
        );
        Ok(zipped)
    }

    /// Gets the proof for a transaction chunk.
    /// N.B. the `LedgerInfo` returned will always be in the same epoch of the `last_version`.
    pub fn get_transaction_range_proof(
        &self,
        first_version: Version,
        last_version: Version,
    ) -> Result<(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)> {
        ensure!(
            last_version >= first_version,
            "Bad transaction range: [{}, {}]",
            first_version,
            last_version
        );
        let num_transactions = last_version - first_version + 1;
        let epoch = self.ledger_store.get_epoch(last_version)?;
        let ledger_info = self.ledger_store.get_latest_ledger_info_in_epoch(epoch)?;
        let accumulator_proof = self.ledger_store.get_transaction_range_proof(
            Some(first_version),
            num_transactions,
            ledger_info.ledger_info().version(),
        )?;
        Ok((accumulator_proof, ledger_info))
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
        )?
        .enumerate()
        .map(move |(idx, res)| {
            BACKUP_STATE_SNAPSHOT_VERSION.set(version as i64);
            BACKUP_STATE_SNAPSHOT_LEAF_IDX.set(idx as i64);
            res
        });
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

    /// Gets the epoch, commited version, and synced version of the DB.
    pub fn get_db_state(&self) -> Result<Option<DbState>> {
        self.ledger_store
            .get_startup_info()?
            .map(|s| {
                Ok(DbState {
                    epoch: s.get_epoch_state().epoch,
                    committed_version: s
                        .committed_tree_state
                        .num_transactions
                        .checked_sub(1)
                        .ok_or_else(|| anyhow!("Bootstrapped DB has no transactions."))?,
                    synced_version: s
                        .synced_tree_state
                        .as_ref()
                        .unwrap_or_else(|| &s.committed_tree_state)
                        .num_transactions
                        .checked_sub(1)
                        .ok_or_else(|| anyhow!("Bootstrapped DB has no transactions."))?,
                })
            })
            .transpose()
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

    pub fn get_epoch_ending_ledger_info_iter<'a>(
        &'a self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<impl Iterator<Item = Result<LedgerInfoWithSignatures>> + 'a> {
        Ok(self
            .ledger_store
            .get_epoch_ending_ledger_info_iter(start_epoch, end_epoch)?
            .enumerate()
            .map(move |(idx, li)| {
                BACKUP_EPOCH_ENDING_EPOCH.set((start_epoch + idx as u64) as i64);
                li
            }))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DbState {
    pub epoch: u64,
    pub committed_version: Version,
    pub synced_version: Version,
}

impl Display for DbState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "epoch: {}, committed_version: {}, synced_version: {}",
            self.epoch, self.committed_version, self.synced_version,
        )
    }
}
