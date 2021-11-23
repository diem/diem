// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::DriverConfiguration,
    error::Error,
    storage_synchronizer::{StorageStateSummary, StorageSynchronizerInterface},
};
use data_streaming_service::{
    data_notification::{DataNotification, DataPayload, NotificationId},
    data_stream::DataStreamListener,
    streaming_client::{DataStreamingClient, NotificationFeedback, StreamingServiceClient},
};
use diem_config::config::{BootstrappingMode, ContinuousSyncingMode, StateSyncDriverConfig};
use diem_data_client::GlobalDataSummary;
use diem_infallible::Mutex;
use diem_logger::*;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    stream::FusedStream,
    SinkExt, Stream, StreamExt,
};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::timeout;

// TODO(joshlind): make this configurable
const MAX_NOTIFICATION_WAIT_TIME_MS: u64 = 300;

/// A simple component that manages the continuous syncing of the node
pub struct ContinuousSyncer<S> {
    // The currently active data stream (provided by the data streaming service)
    active_data_stream: Option<DataStreamListener>,

    // The config of the state sync driver
    driver_configuration: DriverConfiguration,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,

    // The storage synchronizer used to update local storage
    storage_synchronizer: Arc<Mutex<S>>,
}

impl<S: StorageSynchronizerInterface> ContinuousSyncer<S> {
    pub fn new(
        driver_configuration: DriverConfiguration,
        streaming_service_client: StreamingServiceClient,
        storage_synchronizer: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            active_data_stream: None,
            driver_configuration,
            streaming_service_client,
            storage_synchronizer,
        }
    }

    /// Checks if the continuous syncer is able to make progress
    pub async fn check_progress(&mut self) -> Result<(), Error> {
        // Synchronize depending on the mode
        match self.driver_configuration.config.continuous_syncing_mode {
            ContinuousSyncingMode::ApplyTransactionOutputs => {
                self.continuously_apply_transaction_outputs().await
            }
            ContinuousSyncingMode::ExecuteTransactions => {
                self.continuously_execute_transactions().await
            }
        }
    }

    /// Attempts to make syncing progress by applying transaction outputs
    async fn continuously_apply_transaction_outputs(&mut self) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_transaction_outputs_with_proof().await
        } else {
            self.fetch_transaction_outputs_with_proofs().await
        }
    }

    /// Attempts to make syncing progress by executing transactions
    async fn continuously_execute_transactions(&mut self) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_transactions_with_proofs().await
        } else {
            self.fetch_transactions_with_proofs().await
        }
    }

    /// Returns the highest synced version and epoch in storage
    fn get_highest_synced_version_and_epoch(&self) -> Result<(Version, Version), Error> {
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let highest_synced_version = latest_storage_summary.latest_synced_version;
        let highest_synced_epoch = latest_storage_summary.latest_epoch_state.epoch;

        Ok((highest_synced_version, highest_synced_epoch))
    }

    /// Fetches all transactions starting at the current synced version
    async fn fetch_transactions_with_proofs(&mut self) -> Result<(), Error> {
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;

        // Fetch transactions starting at highest_synced_version + 1
        let next_transaction_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next transaction version has overflown!".into())
        })?;
        let transaction_stream = self
            .streaming_service_client
            .continuously_stream_transactions(next_transaction_version, highest_synced_epoch, false)
            .await?;
        self.active_data_stream = Some(transaction_stream);

        Ok(())
    }

    /// Fetches all transaction outputs starting at the current synced version
    async fn fetch_transaction_outputs_with_proofs(&mut self) -> Result<(), Error> {
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;

        // Fetch outputs starting at highest_synced_version + 1
        let next_output_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next output version has overflown!".into())
        })?;
        let output_stream = self
            .streaming_service_client
            .continuously_stream_transaction_outputs(next_output_version, highest_synced_epoch)
            .await?;
        self.active_data_stream = Some(output_stream);

        Ok(())
    }

    async fn process_transactions_with_proofs(&mut self) -> Result<(), Error> {
        // Pull data notifications from the stream
        let mut stream_feedback = None;
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Some(active_data_stream) = &mut self.active_data_stream {
            while let Some(data_notification) = get_data_notification(active_data_stream).await {
                match data_notification.data_payload {
                    DataPayload::ContinuousTransactionsWithProof(
                        ledger_info_with_sigs,
                        transactions_with_proof,
                    ) => {
                        process_transactions_payload(
                            &latest_storage_summary,
                            ledger_info_with_sigs,
                            transactions_with_proof,
                        )
                        .await?;
                    }
                    DataPayload::EndOfStream => {
                        stream_feedback = Some((
                            NotificationFeedback::EndOfStream,
                            data_notification.notification_id,
                        ));
                    }
                    _ => {
                        stream_feedback = Some((
                            NotificationFeedback::PayloadTypeIsIncorrect,
                            data_notification.notification_id,
                        ));
                        break;
                    }
                }
            }
        }

        // Handle any feedback for the stream
        self.terminate_active_stream_if_feedback(stream_feedback)
            .await
    }

    async fn process_transaction_outputs_with_proof(&mut self) -> Result<(), Error> {
        // Pull data notifications from the stream
        let mut stream_feedback = None;
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Some(active_data_stream) = &mut self.active_data_stream {
            while let Some(data_notification) = get_data_notification(active_data_stream).await {
                match data_notification.data_payload {
                    DataPayload::ContinuousTransactionOutputsWithProof(
                        ledger_info_with_sigs,
                        transaction_outputs_with_proof,
                    ) => {
                        process_transaction_outputs_payload(
                            &latest_storage_summary,
                            ledger_info_with_sigs,
                            transaction_outputs_with_proof,
                        )
                        .await?;
                    }
                    DataPayload::EndOfStream => {
                        stream_feedback = Some((
                            NotificationFeedback::EndOfStream,
                            data_notification.notification_id,
                        ));
                    }
                    _ => {
                        stream_feedback = Some((
                            NotificationFeedback::PayloadTypeIsIncorrect,
                            data_notification.notification_id,
                        ));
                        break;
                    }
                }
            }
        }

        // Handle any feedback for the stream
        self.terminate_active_stream_if_feedback(stream_feedback)
            .await
    }

    /// Terminates the currently active stream if stream feeback is provided
    async fn terminate_active_stream_if_feedback(
        &mut self,
        stream_feedback: Option<(NotificationFeedback, NotificationId)>,
    ) -> Result<(), Error> {
        if let Some(stream_feedback) = stream_feedback {
            let (notification_feedback, notification_id) = stream_feedback;
            debug!(
                "Terminating the currently active stream! Feedback: {:?}, notification ID: {:?}",
                notification_feedback, notification_id
            );
            self.streaming_service_client
                .terminate_stream_with_feedback(notification_id, notification_feedback)
                .await?;
            self.active_data_stream = None;
        }

        Ok(())
    }
}

/// Fetches a data notification from the given stream listener (or hits the timeout)
async fn get_data_notification(
    stream_listener: &mut DataStreamListener,
) -> Option<DataNotification> {
    if let Ok(data_notification) = timeout(
        Duration::from_millis(MAX_NOTIFICATION_WAIT_TIME_MS),
        stream_listener.select_next_some(),
    )
    .await
    {
        Some(data_notification)
    } else {
        debug!("No data notification immediately available when listening to the stream.");
        None
    }
}

async fn process_transaction_outputs_payload(
    latest_storage_summary: &StorageStateSummary,
    ledger_info_with_sigs: LedgerInfoWithSignatures,
    transaction_outputs_with_proof: TransactionOutputListWithProof,
) -> Result<(), Error> {
    // TODO(joshlind): process the transaction output list with proof!
    debug!(
        "Received a continuous transaction output payload with proof! First version: {:?}, num outputs: {:?}, ledger info: {:?}",
        transaction_outputs_with_proof.first_transaction_output_version, transaction_outputs_with_proof.transactions_and_outputs.len(), ledger_info_with_sigs
    );

    Ok(())
}

async fn process_transactions_payload(
    latest_storage_summary: &StorageStateSummary,
    ledger_info_with_sigs: LedgerInfoWithSignatures,
    transactions_with_proof: TransactionListWithProof,
) -> Result<(), Error> {
    // TODO(joshlind): process the transaction list with proof!
    debug!(
        "Received a continuous transaction payload with proof! First version: {:?}, num outputs: {:?}, ledger info: {:?}",
        transactions_with_proof.first_transaction_version, transactions_with_proof.transactions.len(), ledger_info_with_sigs
    );

    Ok(())
}
