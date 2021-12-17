// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use diem_infallible::RwLock;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, TransactionOutputListWithProof, Version},
};
use executor_types::{ChunkExecutorTrait, ExecutedTrees};
use std::sync::Arc;
use storage_interface::DbReaderWriter;

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
    /// Applies and commits a batch of transaction outputs to storage.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn apply_and_commit_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>, Error>;

    /// Executes and commits a batch of transactions to storage.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn execute_and_commit_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>, Error>;

    /// Returns the latest storage summary
    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error>;

    /// Saves the given account states to storage
    fn save_account_states(
        &mut self,
        account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error>;
}

/// The implementation of the `StorageSynchronizerInterface` used by state sync
pub struct StorageSynchronizer<ChunkExecutor> {
    chunk_executor: Arc<ChunkExecutor>,
    storage: Arc<RwLock<DbReaderWriter>>,
}

impl<ChunkExecutor: ChunkExecutorTrait> StorageSynchronizer<ChunkExecutor> {
    pub fn new(chunk_executor: Arc<ChunkExecutor>, storage: Arc<RwLock<DbReaderWriter>>) -> Self {
        Self {
            chunk_executor,
            storage,
        }
    }

    fn fetch_storage_summary(&mut self) -> Result<StorageStateSummary, Error> {
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

        // Create the storage summary
        let storage_state_summary = StorageStateSummary {
            latest_epoch_state,
            latest_executed_trees,
            latest_ledger_info,
            latest_synced_version,
        };
        Ok(storage_state_summary)
    }
}

impl<ChunkExecutor: ChunkExecutorTrait> StorageSynchronizerInterface
    for StorageSynchronizer<ChunkExecutor>
{
    fn apply_and_commit_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>, Error> {
        let committed_events = self
            .chunk_executor
            .apply_and_commit_chunk(
                output_list_with_proof,
                &target_ledger_info,
                end_of_epoch_ledger_info.as_ref(),
            )
            .map_err(|error| {
                Error::UnexpectedError(format!("Apply and commit chunk failed: {}", error))
            })?;

        Ok(committed_events)
    }

    fn execute_and_commit_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>, Error> {
        let committed_events = self
            .chunk_executor
            .execute_and_commit_chunk(
                transaction_list_with_proof,
                &target_ledger_info,
                end_of_epoch_ledger_info.as_ref(),
            )
            .map_err(|error| {
                Error::UnexpectedError(format!("Execute and commit chunk failed: {}", error))
            })?;

        Ok(committed_events)
    }

    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error> {
        self.fetch_storage_summary()
    }

    fn save_account_states(
        &mut self,
        _account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error> {
        unimplemented!("Saving account states to storage is not currently supported!")
    }
}
