// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use diem_infallible::RwLock;
use diem_types::{
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{default_protocol::TransactionListWithProof, Version},
};
use executor_types::{ChunkExecutor, ExecutedTrees};
use std::sync::Arc;
use storage_interface::default_protocol::DbReaderWriter;

/// A summary of the state of local storage at a specific snapshot (e.g., version)
#[derive(Clone, Debug)]
pub struct StorageStateSummary {
    pub latest_epoch_state: EpochState,
    pub latest_executed_trees: ExecutedTrees,
    pub latest_ledger_info: LedgerInfoWithSignatures,
    pub latest_synced_version: Version,
}

/// Synchronizes the storage of the node by verifying and storing new data
/// (e.g., transactions and outputs).
pub trait StorageSynchronizerInterface {
    /// Returns the latest storage summary
    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error>;

    /// Executes and commit a batch of transactions
    fn execute_and_commit_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;
}

/// The implementation of the `StorageSynchronizerInterface` used by state sync
pub struct StorageSynchronizer {
    chunk_executor: Box<dyn ChunkExecutor>,
    storage: Arc<RwLock<DbReaderWriter>>,

    // We cache the latest storage summary to prevent unnecessary reads to the
    // database. This should be updated after each database write.
    cached_storage_summary: Option<StorageStateSummary>,
}

impl StorageSynchronizer {
    pub fn new(
        chunk_executor: Box<dyn ChunkExecutor>,
        storage: Arc<RwLock<DbReaderWriter>>,
    ) -> Self {
        Self {
            chunk_executor,
            storage,
            cached_storage_summary: None,
        }
    }

    fn refresh_cached_storage_summary(&mut self) -> Result<StorageStateSummary, Error> {
        // Fetch the startup info from storage
        let startup_info = self
            .storage
            .read()
            .reader
            .get_startup_info()
            .map_err(|error| {
                Error::StorageError(format!(
                    "Failed to get startup info from storage: {:?}",
                    error
                ))
            })?;
        let storage_info = startup_info
            .ok_or_else(|| Error::StorageError("Missing startup info from storage".into()))?;

        // Grab the latest epoch state, executed trees and ledger info
        let latest_epoch_state = storage_info.get_epoch_state().clone();
        let latest_executed_trees = if let Some(synced_tree_state) = storage_info.synced_tree_state
        {
            ExecutedTrees::from(synced_tree_state)
        } else {
            ExecutedTrees::from(storage_info.committed_tree_state)
        };
        let latest_ledger_info = storage_info.latest_ledger_info;

        // Fetch the latest synced version
        let latest_transaction_info = self
            .storage
            .read()
            .reader
            .get_latest_transaction_info_option()
            .map_err(|error| {
                Error::StorageError(format!(
                    "Failed to get the latest transaction info from storage: {:?}",
                    error
                ))
            })?;
        let (latest_synced_version, _) = latest_transaction_info
            .ok_or_else(|| Error::StorageError("Latest transaction info is missing!".into()))?;

        // Create the storage summary and save it in the cache
        let storage_state_summary = StorageStateSummary {
            latest_epoch_state,
            latest_executed_trees,
            latest_ledger_info,
            latest_synced_version,
        };
        self.cached_storage_summary = Some(storage_state_summary.clone());

        Ok(storage_state_summary)
    }
}

impl StorageSynchronizerInterface for StorageSynchronizer {
    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error> {
        if let Some(cached_storage_summary) = &self.cached_storage_summary {
            Ok(cached_storage_summary.clone())
        } else {
            self.refresh_cached_storage_summary()
        }
    }

    fn execute_and_commit_transactions(
        &mut self,
        _transaction_list_with_proof: TransactionListWithProof,
        _verified_target_li: LedgerInfoWithSignatures,
        _intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        unimplemented!();
    }
}
