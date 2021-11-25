// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::{get_version, set_version, DriverConfiguration},
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
    epoch_change::Verifier,
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
    pub async fn check_progress(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // Synchronize depending on the mode
        match self.driver_configuration.config.continuous_syncing_mode {
            ContinuousSyncingMode::ApplyTransactionOutputs => {
                self.continuously_apply_transaction_outputs(sync_request_target)
                    .await
            }
            ContinuousSyncingMode::ExecuteTransactions => {
                self.continuously_execute_transactions(sync_request_target)
                    .await
            }
        }
    }

    /// Attempts to make syncing progress by applying transaction outputs
    async fn continuously_apply_transaction_outputs(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_continuous_transaction_outputs_with_proof(sync_request_target)
                .await
        } else {
            self.fetch_transaction_outputs_with_proofs(sync_request_target)
                .await
        }
    }

    /// Attempts to make syncing progress by executing transactions
    async fn continuously_execute_transactions(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_continuous_transactions_with_proofs(sync_request_target)
                .await
        } else {
            self.fetch_transactions_with_proofs(sync_request_target)
                .await
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
    async fn fetch_transactions_with_proofs(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;

        // Fetch transactions starting at highest_synced_version + 1
        let next_transaction_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next transaction version has overflown!".into())
        })?;
        let transaction_stream = self
            .streaming_service_client
            .continuously_stream_transactions(
                next_transaction_version,
                highest_synced_epoch,
                false,
                sync_request_target,
            )
            .await?;
        self.active_data_stream = Some(transaction_stream);

        Ok(())
    }

    /// Fetches all transaction outputs starting at the current synced version
    async fn fetch_transaction_outputs_with_proofs(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;

        // Fetch outputs starting at highest_synced_version + 1
        let next_output_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next output version has overflown!".into())
        })?;
        let output_stream = self
            .streaming_service_client
            .continuously_stream_transaction_outputs(
                next_output_version,
                highest_synced_epoch,
                sync_request_target,
            )
            .await?;
        self.active_data_stream = Some(output_stream);

        Ok(())
    }

    async fn process_continuous_transactions_with_proofs(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        loop {
            let data_notification = self.get_data_notification().await?;
            if data_notification.is_none() {
                return Ok(()); // Nothing to do!
            }

            // Process the notification
            let data_notification = data_notification.expect("A data notification should exist!");
            match data_notification.data_payload {
                DataPayload::ContinuousTransactionsWithProof(
                    ledger_info_with_sigs,
                    transactions_with_proof,
                ) => {
                    self.process_continuous_transactions_payload(
                        sync_request_target.as_ref(),
                        data_notification.notification_id,
                        ledger_info_with_sigs,
                        transactions_with_proof,
                    )
                    .await?;
                }
                _ => {
                    return self
                        .handle_end_of_stream_or_invalid_payload(data_notification)
                        .await
                }
            }
        }
    }

    async fn process_continuous_transaction_outputs_with_proof(
        &mut self,
        sync_request_target: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        loop {
            let data_notification = self.get_data_notification().await?;
            if data_notification.is_none() {
                return Ok(()); // Nothing to do!
            }

            // Process the notification
            let data_notification = data_notification.expect("A data notification should exist!");
            match data_notification.data_payload {
                DataPayload::ContinuousTransactionOutputsWithProof(
                    ledger_info_with_sigs,
                    transaction_outputs_with_proof,
                ) => {
                    self.process_continuous_transaction_outputs_payload(
                        sync_request_target.as_ref(),
                        data_notification.notification_id,
                        ledger_info_with_sigs,
                        transaction_outputs_with_proof,
                    )
                    .await?;
                }
                _ => {
                    return self
                        .handle_end_of_stream_or_invalid_payload(data_notification)
                        .await
                }
            }
        }
    }

    /// Fetches a data notification from the `active_data_stream`
    async fn get_data_notification(&mut self) -> Result<Option<DataNotification>, Error> {
        let active_data_stream = self
            .active_data_stream
            .as_mut()
            .expect("The active data stream should exist!");

        let data_notification = if let Ok(data_notification) = timeout(
            Duration::from_millis(MAX_NOTIFICATION_WAIT_TIME_MS),
            active_data_stream.select_next_some(),
        )
        .await
        {
            Some(data_notification)
        } else {
            //debug!(
            //    "No data notification immediately found when listening to the active data stream."
            //);
            None
        };

        Ok(data_notification)
    }

    /// Handles the end of stream notification, or an invalid payload
    async fn handle_end_of_stream_or_invalid_payload(
        &mut self,
        data_notification: DataNotification,
    ) -> Result<(), Error> {
        match data_notification.data_payload {
            DataPayload::EndOfStream => {
                self.terminate_active_stream(
                    data_notification.notification_id,
                    NotificationFeedback::EndOfStream,
                )
                .await
            }
            _ => {
                self.terminate_active_stream(
                    data_notification.notification_id,
                    NotificationFeedback::PayloadTypeIsIncorrect,
                )
                .await
            }
        }
    }

    /// Processes a continuous transaction outputs payload received from an active data stream
    async fn process_continuous_transaction_outputs_payload(
        &mut self,
        sync_request_target: Option<&LedgerInfoWithSignatures>,
        notification_id: NotificationId,
        ledger_info_with_signatures: LedgerInfoWithSignatures,
        transaction_outputs_with_proof: TransactionOutputListWithProof,
    ) -> Result<(), Error> {
        //debug!(
        //    "Received a continuous transaction output payload with proof! ID: {:?}, first version: {:?}, num outputs: {:?}, ledger info: {:?}",
        //    notification_id, transaction_outputs_with_proof.first_transaction_output_version, transaction_outputs_with_proof.transactions_and_outputs.len(), ledger_info_with_signatures
        //);

        // TODO(joshlind): remove this hack!
        // Set the version of the counter
        let num_outputs = transaction_outputs_with_proof
            .transactions_and_outputs
            .len() as u64;
        let current_synced_version = get_version("synced".into());
        let new_synced_version = current_synced_version + num_outputs;
        set_version("synced".into(), new_synced_version);
        set_version("committed".into(), new_synced_version);
        //return Ok(());

        // Sanity check the payload
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;
        if let Some(first_output_version) =
            transaction_outputs_with_proof.first_transaction_output_version
        {
            let expected_version = highest_synced_version
                .checked_add(1)
                .ok_or_else(|| Error::IntegerOverflow("Expected version has overflown!".into()))?;

            if first_output_version != expected_version {
                // The first output version doesn't match the expected!
                //return self
                //    .terminate_active_stream(
                //        notification_id,
                //        NotificationFeedback::InvalidPayloadData,
                //    )
                //    .await;
            }
        } else {
            // The output list is empty!
            //return self
            //    .terminate_active_stream(notification_id, NotificationFeedback::InvalidPayloadData)
            //    .await;
        }

        // If we're syncing to a specific target, validate the chunk
        if let Some(sync_request_target) = sync_request_target {
            let sync_request_version = sync_request_target.ledger_info().version();
            let output_proof_version = ledger_info_with_signatures.ledger_info().version();
            if sync_request_version < output_proof_version {
                //error!("Transaction output proof version is higher than the sync target. Proof version: {:?}, sync version: {:?}.",
                //       output_proof_version,
                //       sync_request_version);
                // The output proof version is too high!
                //return self
                //    .terminate_active_stream(
                //        notification_id,
                //        NotificationFeedback::PayloadProofFailed,
                //    )
                //    .await;
            }
        }

        // Verify the ledger info
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let trusted_state = latest_storage_summary.latest_epoch_state;
        let _ = trusted_state.verify(&ledger_info_with_signatures);
        //if let Err(error) = trusted_state.verify(&ledger_info_with_signatures) {
        // Failed to verify the ledger info!
        //return self
        //    .terminate_active_stream(notification_id, NotificationFeedback::PayloadProofFailed)
        //    .await;
        //}

        // Execute and commit the chunk
        //let result = self
        //    .storage_synchronizer
        //    .lock()
        //    .apply_and_commit_transaction_outputs(
        //        transaction_outputs_with_proof,
        //        ledger_info_with_signatures,
        //        None,
        //    );

        //if let Err(error) = result {
        // Failed to apply and commit the transactions
        //error!(
        //    "Failed to apply and commit the transactions list with proof: {:?}",
        //    error
        //);
        //self.terminate_active_stream(notification_id, NotificationFeedback::InvalidPayloadData)
        //    .await
        //} else {
        //Ok(())
        //}

        Ok(())
    }

    /// Processes a continuous transactions payload received from an active data stream
    async fn process_continuous_transactions_payload(
        &mut self,
        sync_request_target: Option<&LedgerInfoWithSignatures>,
        notification_id: NotificationId,
        ledger_info_with_signatures: LedgerInfoWithSignatures,
        transactions_with_proof: TransactionListWithProof,
    ) -> Result<(), Error> {
        //debug!(
        //    "Received a continuous transaction output payload with proof! First version: {:?}, num transactions: {:?}, ledger info: {:?}",
        //    transactions_with_proof.first_transaction_version, transactions_with_proof.transactions.len(), ledger_info_with_signatures
        //);

        // TODO(joshlind): remove this hack!
        // Set the version of the counter
        let num_outputs = transactions_with_proof.transactions.len() as u64;
        let current_synced_version = get_version("synced".into());
        let new_synced_version = current_synced_version + num_outputs;
        set_version("synced".into(), new_synced_version);
        set_version("committed".into(), new_synced_version);
        return Ok(());

        // Sanity check the payload
        let (highest_synced_version, highest_synced_epoch) =
            self.get_highest_synced_version_and_epoch()?;
        if let Some(first_transaction_version) = transactions_with_proof.first_transaction_version {
            let expected_version = highest_synced_version
                .checked_add(1)
                .ok_or_else(|| Error::IntegerOverflow("Expected version has overflown!".into()))?;

            if first_transaction_version != expected_version {
                // The first transaction version doesn't match the expected!
                return self
                    .terminate_active_stream(
                        notification_id,
                        NotificationFeedback::InvalidPayloadData,
                    )
                    .await;
            }
        } else {
            // The transaction list is empty!
            return self
                .terminate_active_stream(notification_id, NotificationFeedback::InvalidPayloadData)
                .await;
        }

        // If we're syncing to a specific target, validate the chunk
        if let Some(sync_request_target) = sync_request_target {
            let sync_request_version = sync_request_target.ledger_info().version();
            let transaction_proof_version = ledger_info_with_signatures.ledger_info().version();
            if sync_request_version < transaction_proof_version {
                error!("Transaction proof version is higher than the sync target. Proof version: {:?}, sync version: {:?}.",
                               transaction_proof_version,
                               sync_request_version);
                // The transaction proof version is too high!
                return self
                    .terminate_active_stream(
                        notification_id,
                        NotificationFeedback::PayloadProofFailed,
                    )
                    .await;
            }
        }

        // Verify the ledger info
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let trusted_state = latest_storage_summary.latest_epoch_state;
        if let Err(error) = trusted_state.verify(&ledger_info_with_signatures) {
            // Failed to verify the ledger info!
            return self
                .terminate_active_stream(notification_id, NotificationFeedback::PayloadProofFailed)
                .await;
        }

        // Execute and commit the chunk
        let result = self
            .storage_synchronizer
            .lock()
            .execute_and_commit_transactions(
                transactions_with_proof,
                ledger_info_with_signatures,
                None,
            );

        if let Err(error) = result {
            // Failed to execute and commit the transactions
            error!(
                "Failed to execute and commit the transactions list with proof: {:?}",
                error
            );
            self.terminate_active_stream(notification_id, NotificationFeedback::InvalidPayloadData)
                .await
        } else {
            Ok(())
        }
    }

    /// Terminates the currently active stream with the provided feedback
    async fn terminate_active_stream(
        &mut self,
        notification_id: NotificationId,
        notification_feedback: NotificationFeedback,
    ) -> Result<(), Error> {
        debug!(
            "Terminating the currently active stream! Feedback: {:?}, notification ID: {:?}",
            notification_feedback, notification_id
        );

        self.active_data_stream = None;
        self.streaming_service_client
            .terminate_stream_with_feedback(notification_id, notification_feedback)
            .await?;

        Ok(())
    }
}
