// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::{get_version, set_version},
    error::Error,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_infallible::RwLock;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};
use executor_types::{ChunkExecutorTrait, ExecutedTrees};
use futures::{channel::mpsc, SinkExt, StreamExt};
use std::sync::Arc;
use storage_interface::default_protocol::DbReaderWriter;
use tokio::runtime::{Builder, Runtime};

/// A summary of the state of local storage at a specific snapshot (e.g., version)
#[derive(Clone, Debug)]
pub struct StorageStateSummary {
    pub latest_epoch_state: EpochState,
    pub latest_executed_trees: ExecutedTrees,
    pub latest_ledger_info: LedgerInfoWithSignatures,
    pub latest_synced_version: Version,
}

/// A set of committed events at the specified blockchain version.
/// TODO(joshlind): to support versioning of events, we'll need to do some
/// refactors. All callers currently specify version == 0.
pub struct CommittedEvents {
    pub version: u64,
    pub events: Vec<ContractEvent>,
}

/// A chunk of transactions/transaction output data to be executed and committed
enum DataChunk {
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

/// Synchronizes the storage of the node by verifying and storing new data
/// (e.g., transactions and outputs).
pub trait StorageSynchronizerInterface {
    /// Applies a batch of transaction outputs to storage.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn apply_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Executes a batch of transactions to storage.
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
pub struct StorageSynchronizer<C> {
    _chunk_executor: Arc<C>,
    data_chunk_notifier: channel::diem_channel::Sender<(), Result<DataChunk, Error>>,
    storage: Arc<RwLock<DbReaderWriter>>,
    _runtime: Option<Runtime>,

    // We cache the latest storage summary to prevent unnecessary reads to the
    // database. This should be updated after each database write.
    cached_storage_summary: Arc<RwLock<Option<StorageStateSummary>>>,
}

impl<C: ChunkExecutorTrait + 'static> StorageSynchronizer<C> {
    pub fn new(
        create_runtime: bool,
        chunk_executor: Arc<C>,
        storage: Arc<RwLock<DbReaderWriter>>,
    ) -> (Self, mpsc::UnboundedReceiver<CommittedEvents>) {
        // Create a channel to execute/apply data chunks
        let (data_chunk_notifier, data_chunk_listener) =
            diem_channel::new(QueueStyle::KLAST, 5000, None);

        // Create a channel to notify the committer for executed chunks (pending commit)
        let (executed_chunk_notifier, executed_chunk_listener) =
            diem_channel::new(QueueStyle::KLAST, 5000, None);

        // Create a channel to send committed events
        let (committed_events_sender, committed_events_receiver) = mpsc::unbounded();

        // Create the cached storage summary
        let cached_storage_summary = Arc::new(RwLock::new(None));

        // Create and spawn a new runtime for the driver (if required)
        let runtime = if create_runtime {
            let runtime = Builder::new_multi_thread()
                .thread_name("state-sync-storage-synchronizer")
                .enable_all()
                .build()
                .expect("Failed to create state sync v2 storage synchronizer runtime!");
            Some(runtime)
        } else {
            None
        };

        // Spawn a thread that executes/applies provided chunks
        spawn_executor_thread(
            chunk_executor.clone(),
            cached_storage_summary.clone(),
            data_chunk_listener,
            executed_chunk_notifier,
            runtime.as_ref(),
        );

        // Spawn a thread that commits executed chunks
        spawn_committer_thread(
            chunk_executor.clone(),
            committed_events_sender,
            executed_chunk_listener,
            runtime.as_ref(),
        );

        let storage_synchronizer = Self {
            _chunk_executor: chunk_executor,
            data_chunk_notifier,
            storage,
            _runtime: runtime,
            cached_storage_summary,
        };

        (storage_synchronizer, committed_events_receiver)
    }

    fn invalidate_cached_storage_summary(&mut self) {
        *self.cached_storage_summary.write() = None;
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
        *self.cached_storage_summary.write() = Some(storage_state_summary.clone());

        Ok(storage_state_summary)
    }
}

impl<C: ChunkExecutorTrait + 'static> StorageSynchronizerInterface for StorageSynchronizer<C> {
    fn apply_transaction_outputs(
        &mut self,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let data_chunk = DataChunk::TransactionOutputs(
            output_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        if let Err(error) = self.data_chunk_notifier.push((), Ok(data_chunk)) {
            panic!("Failed to execute transactions! Error: {:?}", error);
        }

        Ok(())
    }

    fn execute_transactions(
        &mut self,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let data_chunk = DataChunk::Transactions(
            transaction_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        if let Err(error) = self.data_chunk_notifier.push((), Ok(data_chunk)) {
            panic!("Failed to execute transactions! Error: {:?}", error);
        }

        Ok(())
    }

    fn get_storage_summary(&mut self) -> Result<StorageStateSummary, Error> {
        if let Some(cached_storage_summary) = &*self.cached_storage_summary.read() {
            return Ok(cached_storage_summary.clone());
        }

        self.refresh_cached_storage_summary()
    }

    fn save_account_states(
        &mut self,
        _account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error> {
        unimplemented!("Saving account states to storage is not currently supported!")
    }
}

/// Spawns a dedicated thread that commits already executed chunks
fn spawn_committer_thread<C: ChunkExecutorTrait + 'static>(
    chunk_executor: Arc<C>,
    mut committed_events_sender: mpsc::UnboundedSender<CommittedEvents>,
    mut executed_chunk_listener: channel::diem_channel::Receiver<(), Result<u64, Error>>,
    runtime: Option<&Runtime>,
) {
    let committer_thread = async move {
        loop {
            ::futures::select! {
                num_transactions = executed_chunk_listener.select_next_some() => {
                    match num_transactions {
                        Ok(num_transactions) => {
                            match chunk_executor.commit_chunk() {
                                Ok(committed_events) => {
                                    let committed_events = CommittedEvents {
                                        version: 0,
                                        events: committed_events,
                                    };
                                    if let Err(error) = committed_events_sender.send(committed_events).await {
                                        panic!("Failed to send committed events! Error: {:?}", error);
                                    }

                                    // Update the metrics
                                    set_metrics(num_transactions);
                                }
                                Err(error) => {
                                    panic!("Failed to commit pending chunk! Error: {:?}", error);
                                }
                            }
                        }
                        Err(error) => {
                            panic!("Received error from executed chunk listener! Error: {:?}", error);
                        }
                    }
                }
            }
        }
    };

    // Spawn the thread
    if let Some(runtime) = runtime {
        runtime.spawn(committer_thread);
    } else {
        tokio::spawn(committer_thread);
    }
}

/// Spawns a dedicated thread that executes/applies chunks
fn spawn_executor_thread<C: ChunkExecutorTrait + 'static>(
    chunk_executor: Arc<C>,
    cached_storage_summary: Arc<RwLock<Option<StorageStateSummary>>>,
    mut data_chunk_listener: channel::diem_channel::Receiver<(), Result<DataChunk, Error>>,
    executed_chunk_notifier: channel::diem_channel::Sender<(), Result<u64, Error>>,
    runtime: Option<&Runtime>,
) {
    let executor_thread = async move {
        loop {
            ::futures::select! {
                data_chunk = data_chunk_listener.select_next_some() => {
                    // Execute/apply the data chunk
                    let num_transactions = match data_chunk {
                        Err(error) => {
                            panic!("Received an error data chunk: {:?}", error);
                        }
                        Ok(data_chunk) => {
                            match data_chunk {
                                DataChunk::Transactions(transactions_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                                    let num_transactions = transactions_with_proof.transactions.len() as u64;
                                    let _ = chunk_executor
                                        .execute_chunk(
                                            transactions_with_proof,
                                            &target_ledger_info,
                                            end_of_epoch_ledger_info.as_ref(),
                                        )
                                        .map_err(|error| {
                                            panic!("Error when executing chunk: {:?}", error);
                                        });
                                    num_transactions
                                },
                                DataChunk::TransactionOutputs(outputs_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                                    let num_transactions = outputs_with_proof.transactions_and_outputs.len() as u64;
                                    let _ = chunk_executor
                                        .apply_chunk(
                                            outputs_with_proof,
                                            &target_ledger_info,
                                            end_of_epoch_ledger_info.as_ref(),
                                        )
                                        .map_err(|error| {
                                            panic!("Error when applying chunk: {:?}", error);
                                        });
                                    num_transactions
                                }
                            }
                        }
                    };

                    // Notify the listener of new chunks!
                    notify_executed_chunks_listener(&executed_chunk_notifier, num_transactions);

                    // Update the metrics
                    // set_metrics(num_transactions);

                    // Invalid the cache
                    *cached_storage_summary.write() = None;
                }
            }
        }
    };

    // Spawn the thread
    if let Some(runtime) = runtime {
        runtime.spawn(executor_thread);
    } else {
        tokio::spawn(executor_thread);
    }
}

/// Sets the metrics required for CT benchmarking
fn set_metrics(num_transactions: u64) {
    let current_synced_version = get_version("synced".into());
    let new_synced_version = current_synced_version + num_transactions;
    set_version("synced".into(), new_synced_version);
    set_version("committed".into(), new_synced_version);
}

/// Notifies the executed chunks listener that executed chunks are ready to
/// be committed.
fn notify_executed_chunks_listener(
    executed_chunk_notifier: &channel::diem_channel::Sender<(), Result<u64, Error>>,
    num_transactions: u64,
) {
    if let Err(error) = executed_chunk_notifier.push((), Ok(num_transactions)) {
        panic!(
            "Failed to notify the executed chunk listener! Error: {:?}",
            error
        );
    }
}
