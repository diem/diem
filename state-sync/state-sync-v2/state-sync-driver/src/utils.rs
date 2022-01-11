// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use data_streaming_service::{
    data_notification::{DataNotification, DataPayload, NotificationId},
    data_stream::DataStreamListener,
    streaming_client::{DataStreamingClient, NotificationFeedback, StreamingServiceClient},
};
use diem_logger::prelude::*;
use diem_types::{
    epoch_change::Verifier, epoch_state::EpochState, ledger_info::LedgerInfoWithSignatures,
    transaction::Version,
};
use futures::StreamExt;
use std::{sync::Arc, time::Duration};
use storage_interface::{DbReader, StartupInfo};
use tokio::time::timeout;

// TODO(joshlind): make this configurable
const MAX_NOTIFICATION_WAIT_TIME_MS: u64 = 500;

/// The speculative state that tracks a data stream of transactions or outputs.
/// This assumes all data is valid and allows the driver to speculatively verify
/// payloads flowing along the stream without having to block on the executor or
/// storage. Thus, increasing syncing performance.
pub struct SpeculativeStreamState {
    epoch_state: EpochState,
    proof_ledger_info: Option<LedgerInfoWithSignatures>,
    synced_version: Version,
}

impl SpeculativeStreamState {
    pub fn new(
        epoch_state: EpochState,
        proof_ledger_info: Option<LedgerInfoWithSignatures>,
        synced_version: Version,
    ) -> Self {
        Self {
            epoch_state,
            proof_ledger_info,
            synced_version,
        }
    }

    /// Returns the next version that we expect along the stream
    pub fn expected_next_version(&self) -> Result<Version, Error> {
        self.synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The expected next version has overflown!".into())
        })
    }

    /// Returns the proof ledger info that all data along the stream should have
    /// proofs relative to. This assumes the proof ledger info exists!
    pub fn get_proof_ledger_info(&self) -> LedgerInfoWithSignatures {
        self.proof_ledger_info
            .as_ref()
            .expect("Proof ledger info is missing!")
            .clone()
    }

    /// Updates the currently synced version of the stream
    pub fn update_synced_version(&mut self, synced_version: Version) {
        self.synced_version = synced_version;
    }

    /// Verifies the given ledger info with signatures against the current epoch
    /// state and updates the state if the validator set has changed.
    pub fn verify_ledger_info_with_signatures(
        &mut self,
        ledger_info_with_signatures: &LedgerInfoWithSignatures,
    ) -> Result<(), Error> {
        self.epoch_state
            .verify(ledger_info_with_signatures)
            .map_err(|error| {
                Error::VerificationError(format!("Ledger info failed verification: {:?}", error))
            })?;
        if let Some(epoch_state) = ledger_info_with_signatures.ledger_info().next_epoch_state() {
            self.epoch_state = epoch_state.clone();
        }
        Ok(())
    }
}

/// Fetches a data notification from the given data stream listener. Note: this
/// helper assumes the `active_data_stream` exists and throws an error if a
/// notification is not found within the timeout.
pub async fn get_data_notification(
    active_data_stream: Option<&mut DataStreamListener>,
) -> Result<DataNotification, Error> {
    let active_data_stream = active_data_stream.expect("The active data stream should exist!");

    let timeout_ms = Duration::from_millis(MAX_NOTIFICATION_WAIT_TIME_MS);
    if let Ok(data_notification) = timeout(timeout_ms, active_data_stream.select_next_some()).await
    {
        Ok(data_notification)
    } else {
        Err(Error::DataStreamNotificationTimeout(format!(
            "{:?}",
            timeout_ms
        )))
    }
}

/// Terminates the stream with the provided notification ID and feedback
pub async fn terminate_stream_with_feedback(
    streaming_service_client: &mut StreamingServiceClient,
    notification_id: NotificationId,
    notification_feedback: NotificationFeedback,
) -> Result<(), Error> {
    info!(
        "Terminating the current stream! Feedback: {:?}, notification ID: {:?}",
        notification_feedback, notification_id
    );

    streaming_service_client
        .terminate_stream_with_feedback(notification_id, notification_feedback)
        .await
        .map_err(|error| error.into())
}

/// Handles the end of stream notification or an invalid payload by terminating
/// the stream appropriately.
pub async fn handle_end_of_stream_or_invalid_payload(
    streaming_service_client: &mut StreamingServiceClient,
    data_notification: DataNotification,
) -> Result<(), Error> {
    // Terminate the stream with the appropriate feedback
    let notification_feedback = match data_notification.data_payload {
        DataPayload::EndOfStream => NotificationFeedback::EndOfStream,
        _ => NotificationFeedback::PayloadTypeIsIncorrect,
    };
    terminate_stream_with_feedback(
        streaming_service_client,
        data_notification.notification_id,
        notification_feedback,
    )
    .await?;

    // Return an error if the payload was invalid
    match data_notification.data_payload {
        DataPayload::EndOfStream => Ok(()),
        _ => Err(Error::InvalidPayload("Unexpected payload type!".into())),
    }
}

/// Fetches the latest epoch state from the specified storage
pub fn fetch_latest_epoch_state(storage: Arc<dyn DbReader>) -> Result<EpochState, Error> {
    let startup_info = fetch_startup_info(storage)?;
    Ok(startup_info.get_epoch_state().clone())
}

/// Fetches the latest synced ledger info from the specified storage
pub fn fetch_latest_synced_ledger_info(
    storage: Arc<dyn DbReader>,
) -> Result<LedgerInfoWithSignatures, Error> {
    let startup_info = fetch_startup_info(storage)?;
    Ok(startup_info.latest_ledger_info)
}

/// Fetches the latest synced version from the specified storage
pub fn fetch_latest_synced_version(storage: Arc<dyn DbReader>) -> Result<Version, Error> {
    let latest_transaction_info =
        storage
            .get_latest_transaction_info_option()
            .map_err(|error| {
                Error::StorageError(format!(
                    "Failed to get the latest transaction info from storage: {:?}",
                    error
                ))
            })?;
    latest_transaction_info
        .ok_or_else(|| Error::StorageError("Latest transaction info is missing!".into()))
        .map(|(latest_synced_version, _)| latest_synced_version)
}

/// Fetches the startup info from the specified storage
fn fetch_startup_info(storage: Arc<dyn DbReader>) -> Result<StartupInfo, Error> {
    let startup_info = storage.get_startup_info().map_err(|error| {
        Error::StorageError(format!(
            "Failed to get startup info from storage: {:?}",
            error
        ))
    })?;
    startup_info.ok_or_else(|| Error::StorageError("Missing startup info from storage".into()))
}
