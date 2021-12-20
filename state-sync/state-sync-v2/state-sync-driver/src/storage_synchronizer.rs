// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, notification_handlers::CommitNotification};
use diem_infallible::RwLock;
use diem_logger::prelude::*;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, TransactionOutputListWithProof, Version},
};
use executor_types::{ChunkExecutorTrait, ExecutedTrees};
use futures::{channel::mpsc, SinkExt, StreamExt};
use std::{future::Future, sync::Arc};
use storage_interface::DbReaderWriter;
use tokio::runtime::{Builder, Runtime};

// The maximum number of chunks that are pending execution or commit
const MAX_PENDING_CHUNKS: usize = 50;

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
    /// Applies a batch of transaction outputs.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn apply_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Executes a batch of transactions.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn execute_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Returns the latest storage summary
    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error>;

    /// Saves the given account states to storage
    fn save_account_states(
        &mut self,
        account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error>;
}

/// The implementation of the `StorageSynchronizerInterface` used by state sync
pub struct StorageSynchronizer {
    // A channel through which to notify the executor of new transaction data chunks
    executor_notifier: mpsc::Sender<TransactionDataChunk>,

    // The interface to read and write to storage
    storage: Arc<RwLock<DbReaderWriter>>,

    // The runtime operating the storage synchronizer
    _storage_synchronizer_runtime: Option<Runtime>,
}

impl StorageSynchronizer {
    pub fn new<ChunkExecutor: ChunkExecutorTrait + 'static>(
        create_runtime: bool,
        chunk_executor: Arc<ChunkExecutor>,
        commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,
        storage: Arc<RwLock<DbReaderWriter>>,
    ) -> Self {
        // Create a channel to notify the executor when transaction data chunks are ready
        let (executor_notifier, executor_listener) = mpsc::channel(MAX_PENDING_CHUNKS);

        // Create a channel to notify the committer when executed chunks are ready
        let (committer_notifier, committer_listener) = mpsc::channel(MAX_PENDING_CHUNKS);

        // Create a new runtime (if required)
        let storage_synchronizer_runtime = if create_runtime {
            Some(
                Builder::new_multi_thread()
                    .thread_name("storage-synchronizer")
                    .enable_all()
                    .build()
                    .expect("Failed to create state sync v2 storage synchronizer runtime!"),
            )
        } else {
            None
        };

        // Spawn the executor that executes/applies transaction data chunks
        spawn_executor(
            chunk_executor.clone(),
            executor_listener,
            committer_notifier,
            storage_synchronizer_runtime.as_ref(),
        );

        // Spawn the committer that commits executed (but pending) chunks
        spawn_committer(
            chunk_executor,
            committer_listener,
            commit_notification_sender,
            storage_synchronizer_runtime.as_ref(),
        );

        Self {
            executor_notifier,
            storage,
            _storage_synchronizer_runtime: storage_synchronizer_runtime,
        }
    }

    /// Fetches a summary of storage by reading directly from the database
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

    /// Notifies the executor of new transaction data chunks
    fn notify_executor(
        &mut self,
        transaction_data_chunk: TransactionDataChunk,
    ) -> Result<(), Error> {
        self.executor_notifier
            .try_send(transaction_data_chunk)
            .map_err(|error| {
                Error::UnexpectedError(format!(
                    "Failed to send transaction data chunk to executor: {:?}",
                    error
                ))
            })
    }
}

impl StorageSynchronizerInterface for StorageSynchronizer {
    fn apply_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let transaction_data_chunk = TransactionDataChunk::TransactionOutputs(
            output_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        self.notify_executor(transaction_data_chunk)
    }

    fn execute_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let transaction_data_chunk = TransactionDataChunk::Transactions(
            transaction_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        self.notify_executor(transaction_data_chunk)
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

/// A chunk of data (i.e., transactions or transaction outputs) to be executed
/// and committed.
enum TransactionDataChunk {
    Transactions(
        TransactionListWithProof,
        LedgerInfoWithSignatures,
        Option<LedgerInfoWithSignatures>,
    ),
    TransactionOutputs(
        TransactionOutputListWithProof,
        LedgerInfoWithSignatures,
        Option<LedgerInfoWithSignatures>,
    ),
}

/// Spawns a dedicated executor that executes/applies transaction data chunks
fn spawn_executor<ChunkExecutor: ChunkExecutorTrait + 'static>(
    chunk_executor: Arc<ChunkExecutor>,
    mut executor_listener: mpsc::Receiver<TransactionDataChunk>,
    mut committer_notifier: mpsc::Sender<()>,
    runtime: Option<&Runtime>,
) {
    // Create an executor
    let executor = async move {
        loop {
            ::futures::select! {
                transaction_data_chunk = executor_listener.select_next_some() => {
                    // Execute/apply the transaction data chunk
                    let result = match transaction_data_chunk {
                        TransactionDataChunk::Transactions(transactions_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                            chunk_executor
                               .execute_chunk(
                                    transactions_with_proof,
                                    &target_ledger_info,
                                    end_of_epoch_ledger_info.as_ref(),
                                ).map_err(|error| {
                                     error!(
                                        "Failed to execute the transaction data chunk! Error: {:?}", error
                                    );
                                    error
                                })
                        },
                        TransactionDataChunk::TransactionOutputs(outputs_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                            chunk_executor
                                .apply_chunk(
                                    outputs_with_proof,
                                    &target_ledger_info,
                                    end_of_epoch_ledger_info.as_ref(),
                                ).map_err(|error| {
                                    error!(
                                        "Failed to apply the transaction data chunk! Error: {:?}", error
                                    );
                                    error
                                })
                        }
                    };

                    // Notify the committer of new executed chunks
                    if result.is_ok() {
                        if let Err(error) = committer_notifier.try_send(()) {
                            error!(
                                "Failed to notify the committer! Error: {:?}", error
                            );
                        }
                    }
                }
            }
        }
    };

    // Spawn the executor
    spawn(runtime, executor);
}

/// Spawns a dedicated committer that commits executed (but pending) chunks
fn spawn_committer<ChunkExecutor: ChunkExecutorTrait + 'static>(
    chunk_executor: Arc<ChunkExecutor>,
    mut committer_listener: mpsc::Receiver<()>,
    mut commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,
    runtime: Option<&Runtime>,
) {
    // Create an executor
    let committer = async move {
        loop {
            ::futures::select! {
                _ = committer_listener.select_next_some() => {
                    // Commit the executed chunk
                    match chunk_executor.commit_chunk() {
                        Ok((events, transactions)) => {
                            let commit_notification = CommitNotification::new(events, transactions);
                            if let Err(error) = commit_notification_sender.send(commit_notification).await {
                                error!("Failed to send commit notification! Error: {:?}", error);
                            }
                        }
                        Err(error) => {
                            error!("Failed to commit executed chunk! Error: {:?}", error);
                        }
                    }
                }
            }
        }
    };

    // Spawn the committer
    spawn(runtime, committer);
}

/// Spawns a future on a specified runtime. If no runtime is specified, uses
/// the current runtime.
fn spawn(runtime: Option<&Runtime>, future: impl Future<Output = ()> + Send + 'static) {
    if let Some(runtime) = runtime {
        runtime.spawn(future);
    } else {
        tokio::spawn(future);
    }
}
