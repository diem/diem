// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    notification_handlers::{CommitNotification, ErrorNotification},
};
use data_streaming_service::data_notification::NotificationId;
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, TransactionOutputListWithProof, Version},
};
use executor_types::ChunkExecutorTrait;
use futures::{channel::mpsc, SinkExt, StreamExt};
use std::{
    future::Future,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use storage_interface::DbWriter;
use tokio::runtime::Runtime;

// TODO(joshlind): add structured logging support!

// The maximum number of chunks that are pending execution or commit
const MAX_PENDING_CHUNKS: usize = 50;

/// Synchronizes the storage of the node by verifying and storing new data
/// (e.g., transactions and outputs).
pub trait StorageSynchronizerInterface {
    /// Applies a batch of transaction outputs.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn apply_transaction_outputs(
        &mut self,
        notification_id: NotificationId,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Executes a batch of transactions.
    ///
    /// Note: this assumes that the ledger infos have already been verified.
    fn execute_transactions(
        &mut self,
        notification_id: NotificationId,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Initializes an account synchronizer with the specified version and
    /// expected root hash and spawns the synchronizer on the given `runtime`.
    /// If `runtime` is None, uses the current runtime.
    ///
    /// Note: this assumes `expected_root_hash` has already been verified.
    fn initialize_account_synchronizer(
        &mut self,
        expected_root_hash: HashValue,
        version: Version,
        runtime: Option<&Runtime>,
    ) -> Result<(), Error>;

    /// Returns true iff there is storage data that is still waiting
    /// to be executed/applied or committed.
    fn pending_storage_data(&self) -> bool;

    /// Saves the given account states to storage.
    ///
    /// Note: this requires that `initialize_account_synchronizer` has been
    /// called.
    fn save_account_states(
        &mut self,
        notification_id: NotificationId,
        account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error>;
}

/// The implementation of the `StorageSynchronizerInterface` used by state sync
#[derive(Clone)]
pub struct StorageSynchronizer {
    // A channel through which to notify the driver of committed data
    commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,

    // A channel through which to notify the driver of storage errors
    error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,

    // A channel through which to notify the executor of new data chunks
    executor_notifier: mpsc::Sender<StorageDataChunk>,

    // The number of storage data chunks pending execute/apply, or commit
    pending_data_chunks: Arc<AtomicU64>,

    // The channel through which to notify the state snapshot receiver of new data chunks
    state_snapshot_notifier: Option<mpsc::Sender<StorageDataChunk>>,

    // The writer to storage (required for account state syncing)
    storage: Arc<dyn DbWriter>,
}

impl StorageSynchronizer {
    pub fn new<ChunkExecutor: ChunkExecutorTrait + 'static>(
        chunk_executor: Arc<ChunkExecutor>,
        commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,
        error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,
        storage: Arc<dyn DbWriter>,
        runtime: Option<&Runtime>,
    ) -> Self {
        // Create a channel to notify the executor when data chunks are ready
        let (executor_notifier, executor_listener) = mpsc::channel(MAX_PENDING_CHUNKS);

        // Create a channel to notify the committer when executed chunks are ready
        let (committer_notifier, committer_listener) = mpsc::channel(MAX_PENDING_CHUNKS);

        // Create a shared pending data chunk counter
        let pending_transaction_chunks = Arc::new(AtomicU64::new(0));

        // Spawn the executor that executes/applies storage data chunks
        spawn_executor(
            chunk_executor.clone(),
            error_notification_sender.clone(),
            executor_listener,
            committer_notifier,
            pending_transaction_chunks.clone(),
            runtime,
        );

        // Spawn the committer that commits executed (but pending) chunks
        spawn_committer(
            chunk_executor,
            committer_listener,
            commit_notification_sender.clone(),
            error_notification_sender.clone(),
            pending_transaction_chunks.clone(),
            runtime,
        );

        Self {
            commit_notification_sender,
            error_notification_sender,
            executor_notifier,
            pending_data_chunks: pending_transaction_chunks,
            state_snapshot_notifier: None,
            storage,
        }
    }

    /// Notifies the executor of new data chunks
    fn notify_executor(&mut self, storage_data_chunk: StorageDataChunk) -> Result<(), Error> {
        if let Err(error) = self.executor_notifier.try_send(storage_data_chunk) {
            Err(Error::UnexpectedError(format!(
                "Failed to send storage data chunk to executor: {:?}",
                error
            )))
        } else {
            increment_atomic(self.pending_data_chunks.clone());
            Ok(())
        }
    }
}

impl StorageSynchronizerInterface for StorageSynchronizer {
    fn apply_transaction_outputs(
        &mut self,
        notification_id: NotificationId,
        output_list_with_proof: TransactionOutputListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let storage_data_chunk = StorageDataChunk::TransactionOutputs(
            notification_id,
            output_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        self.notify_executor(storage_data_chunk)
    }

    fn execute_transactions(
        &mut self,
        notification_id: NotificationId,
        transaction_list_with_proof: TransactionListWithProof,
        target_ledger_info: LedgerInfoWithSignatures,
        end_of_epoch_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let storage_data_chunk = StorageDataChunk::Transactions(
            notification_id,
            transaction_list_with_proof,
            target_ledger_info,
            end_of_epoch_ledger_info,
        );
        self.notify_executor(storage_data_chunk)
    }

    fn initialize_account_synchronizer(
        &mut self,
        expected_root_hash: HashValue,
        version: Version,
        runtime: Option<&Runtime>,
    ) -> Result<(), Error> {
        // Create a channel to notify the state snapshot receiver when data chunks are ready
        let (state_snapshot_notifier, state_snapshot_listener) = mpsc::channel(MAX_PENDING_CHUNKS);

        // Spawn the state snapshot receiver that commits account states
        spawn_state_snapshot_receiver(
            state_snapshot_listener,
            self.commit_notification_sender.clone(),
            expected_root_hash,
            self.error_notification_sender.clone(),
            self.pending_data_chunks.clone(),
            self.storage.clone(),
            version,
            runtime,
        );
        self.state_snapshot_notifier = Some(state_snapshot_notifier);

        Ok(())
    }

    fn pending_storage_data(&self) -> bool {
        self.pending_data_chunks.load(Ordering::Relaxed) > 0
    }

    fn save_account_states(
        &mut self,
        notification_id: NotificationId,
        account_states_with_proof: AccountStatesChunkWithProof,
    ) -> Result<(), Error> {
        let state_snapshot_notifier = &mut self
            .state_snapshot_notifier
            .as_mut()
            .expect("The state snapshot receiver has not been initialized!");
        let storage_data_chunk =
            StorageDataChunk::Accounts(notification_id, account_states_with_proof);
        if let Err(error) = state_snapshot_notifier.try_send(storage_data_chunk) {
            Err(Error::UnexpectedError(format!(
                "Failed to send storage data chunk to state snapshot listener: {:?}",
                error
            )))
        } else {
            increment_atomic(self.pending_data_chunks.clone());
            Ok(())
        }
    }
}

/// A chunk of data to be executed and/or committed to storage (i.e., accounts,
/// transactions or outputs).
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum StorageDataChunk {
    Accounts(NotificationId, AccountStatesChunkWithProof),
    Transactions(
        NotificationId,
        TransactionListWithProof,
        LedgerInfoWithSignatures,
        Option<LedgerInfoWithSignatures>,
    ),
    TransactionOutputs(
        NotificationId,
        TransactionOutputListWithProof,
        LedgerInfoWithSignatures,
        Option<LedgerInfoWithSignatures>,
    ),
}

/// Spawns a dedicated executor that executes/applies storage data chunks
fn spawn_executor<ChunkExecutor: ChunkExecutorTrait + 'static>(
    chunk_executor: Arc<ChunkExecutor>,
    error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,
    mut executor_listener: mpsc::Receiver<StorageDataChunk>,
    mut committer_notifier: mpsc::Sender<NotificationId>,
    pending_transaction_chunks: Arc<AtomicU64>,
    runtime: Option<&Runtime>,
) {
    // Create an executor
    let executor = async move {
        loop {
            ::futures::select! {
                storage_data_chunk = executor_listener.select_next_some() => {
                    // Execute/apply the storage data chunk
                    let (notification_id, result) = match storage_data_chunk {
                        StorageDataChunk::Transactions(notification_id, transactions_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                            let result = chunk_executor
                               .execute_chunk(
                                    transactions_with_proof,
                                    &target_ledger_info,
                                    end_of_epoch_ledger_info.as_ref(),
                                );
                            (notification_id, result)
                        },
                        StorageDataChunk::TransactionOutputs(notification_id, outputs_with_proof, target_ledger_info, end_of_epoch_ledger_info) => {
                            let result = chunk_executor
                                .apply_chunk(
                                    outputs_with_proof,
                                    &target_ledger_info,
                                    end_of_epoch_ledger_info.as_ref(),
                                );
                             (notification_id, result)
                        }
                        storage_data_chunk => {
                            panic!("Invalid storage data chunk sent to executor: {:?}", storage_data_chunk);
                        }
                    };

                    // Notify the committer of new executed chunks
                    match result {
                        Ok(()) => {
                            if let Err(error) = committer_notifier.try_send(notification_id) {
                                let error = format!("Failed to notify the committer! Error: {:?}", error);
                                send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                                decrement_atomic(pending_transaction_chunks.clone());
                            }
                        },
                        Err(error) => {
                            let error = format!("Failed to execute/apply the storage data chunk! Error: {:?}", error);
                            send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                            decrement_atomic(pending_transaction_chunks.clone());
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
    mut committer_listener: mpsc::Receiver<NotificationId>,
    mut commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,
    error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,
    pending_transaction_chunks: Arc<AtomicU64>,
    runtime: Option<&Runtime>,
) {
    // Create a committer
    let committer = async move {
        loop {
            ::futures::select! {
                notification_id = committer_listener.select_next_some() => {
                    // Commit the executed chunk
                    match chunk_executor.commit_chunk() {
                        Ok((events, transactions)) => {
                            // Send a commit notification to the commit listener
                            let commit_notification = CommitNotification::new_committed_transactions(events, transactions);
                            if let Err(error) = commit_notification_sender.send(commit_notification).await {
                                let error = format!("Failed to send transaction commit notification! Error: {:?}", error);
                                send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                            }
                        }
                        Err(error) => {
                            let error = format!("Failed to commit executed chunk! Error: {:?}", error);
                            send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                        }
                    };
                    decrement_atomic(pending_transaction_chunks.clone());
                }
            }
        }
    };

    // Spawn the committer
    spawn(runtime, committer);
}

/// Spawns a dedicated receiver that commits accounts from a state snapshot
fn spawn_state_snapshot_receiver(
    mut state_snapshot_listener: mpsc::Receiver<StorageDataChunk>,
    mut commit_notification_sender: mpsc::UnboundedSender<CommitNotification>,
    expected_root_hash: HashValue,
    error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,
    pending_transaction_chunks: Arc<AtomicU64>,
    storage: Arc<dyn DbWriter>,
    version: Version,
    runtime: Option<&Runtime>,
) {
    // Create a state snapshot receiver
    let receiver = async move {
        let mut state_snapshot_receiver = storage
            .get_state_snapshot_receiver(version, expected_root_hash)
            .expect("Failed to initialize the state snapshot receiver!");
        loop {
            ::futures::select! {
                storage_data_chunk = state_snapshot_listener.select_next_some() => {
                    // Commit the account states chunk
                    match storage_data_chunk {
                        StorageDataChunk::Accounts(notification_id, account_states_with_proof) => {
                            let all_accounts_synced = account_states_with_proof.proof.right_siblings().is_empty();
                            let last_committed_account_index = account_states_with_proof.last_index;

                            // Attempt to the commit the chunk
                            let commit_result = state_snapshot_receiver.add_chunk(
                                account_states_with_proof.account_blobs,
                                account_states_with_proof.proof.clone(),
                            );
                            match commit_result {
                                Ok(()) => {
                                    // Send a commit notification to the commit listener
                                    let commit_notification = CommitNotification::new_committed_accounts(all_accounts_synced, last_committed_account_index);
                                    if let Err(error) = commit_notification_sender.send(commit_notification).await {
                                        let error = format!("Failed to send account commit notification! Error: {:?}", error);
                                        send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                                    } else if all_accounts_synced {
                                        // Update the receiver and return
                                        if let Err(error) = state_snapshot_receiver.finish_box() {
                                            let error = format!("Failed to finish the account states synchronization! Error: {:?}", error);
                                            send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                                        }
                                        decrement_atomic(pending_transaction_chunks.clone());
                                        return;
                                    }
                                },
                                Err(error) => {
                                    let error = format!("Failed to commit account states chunk! Error: {:?}", error);
                                    send_storage_synchronizer_error(error_notification_sender.clone(), notification_id, error).await;
                                }
                            }
                            decrement_atomic(pending_transaction_chunks.clone());
                        },
                        storage_data_chunk => {
                            panic!("Invalid storage data chunk sent to state snapshot receiver: {:?}", storage_data_chunk);
                        }
                    }
                }
            }
        }
    };

    // Spawn the receiver
    spawn(runtime, receiver);
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

/// Increments the given atomic u64
fn increment_atomic(atomic_u64: Arc<AtomicU64>) {
    atomic_u64.fetch_add(1, Ordering::Relaxed);
}

/// Decrements the given atomic u64
fn decrement_atomic(atomic_u64: Arc<AtomicU64>) {
    atomic_u64.fetch_sub(1, Ordering::Relaxed);
}

/// Sends an error notification to the notification listener
async fn send_storage_synchronizer_error(
    mut error_notification_sender: mpsc::UnboundedSender<ErrorNotification>,
    notification_id: NotificationId,
    error_message: String,
) {
    let error_message = format!("Storage synchronizer error: {:?}", error_message);
    error!("{:?}", error_message);

    // Send an error notification
    let error_notification = ErrorNotification {
        error: Error::UnexpectedError(error_message),
        notification_id,
    };
    if let Err(error) = error_notification_sender.send(error_notification).await {
        panic!("Failed to send error notification! Error: {:?}", error);
    }
}
