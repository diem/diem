// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::DriverConfiguration,
    error::Error,
    storage_synchronizer::{StorageStateSummary, StorageSynchronizerInterface},
};
use data_streaming_service::{
    data_notification::{DataNotification, DataPayload},
    data_stream::DataStreamListener,
    streaming_client::{DataStreamingClient, NotificationFeedback, StreamingServiceClient},
};
use diem_config::config::{BootstrappingMode, StateSyncDriverConfig};
use diem_infallible::Mutex;
use diem_logger::*;
use diem_types::ledger_info::LedgerInfoWithSignatures;
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

/// A simple component that manages the bootstrapping of the node
pub struct Bootstrapper<S> {
    // The currently active data stream (provided by the data streaming service)
    active_data_stream: Option<DataStreamListener>,

    // The channel used to notify a listener of successful bootstrapping
    bootstrap_notifier_channel: Option<oneshot::Sender<Result<(), Error>>>,

    // If the node has already been bootstrapped
    bootstrapped: bool,

    // If the node has already bootstrapped all epoch ending ledger infos
    bootstrapped_epoch_ending_ledger_infos: bool,

    // The config of the state sync driver
    driver_configuration: DriverConfiguration,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,

    // The storage synchronizer used to update local storage
    storage_synchronizer: Arc<Mutex<S>>,
}

impl<S: StorageSynchronizerInterface> Bootstrapper<S> {
    pub fn new(
        driver_configuration: DriverConfiguration,
        streaming_service_client: StreamingServiceClient,
        storage_synchronizer: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            active_data_stream: None,
            bootstrap_notifier_channel: None,
            bootstrapped: false,
            bootstrapped_epoch_ending_ledger_infos: false,
            driver_configuration,
            streaming_service_client,
            storage_synchronizer,
        }
    }

    /// Returns true iff the node has already completed bootsrapping
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Notifies the given channel when the node has completed bootstrapping
    pub fn notify_when_bootstrapped(
        &mut self,
        bootstrap_notifier_channel: oneshot::Sender<Result<(), Error>>,
    ) -> Result<(), Error> {
        self.bootstrap_notifier_channel = Some(bootstrap_notifier_channel);
        self.check_if_bootstrapped()
    }

    /// Checks if the node has successfully bootstrapped
    fn check_if_bootstrapped(&mut self) -> Result<(), Error> {
        if self.bootstrapped {
            if let Some(notifier_channel) = self.bootstrap_notifier_channel.take() {
                let notification_message = Ok(());
                if let Err(error) = notifier_channel.send(notification_message) {
                    return Err(Error::CallbackSendFailed(format!(
                        "Bootstrap notification error: {:?}",
                        error
                    )));
                }
            }
        }

        Ok(())
    }

    /// Checks if the bootstrapper is able to make progress
    pub async fn check_progress(&mut self) -> Result<(), Error> {
        if self.is_bootstrapped() {
            return Err(Error::AlreadyBootstrapped(
                "The bootstrapper should not attempt to make progress!".into(),
            ));
        }

        // Identify how to make progress based on the bootstrapping mode
        match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                self.execute_transactions_from_genesis().await?;
            }
            mode => unimplemented!("Bootstrapping mode {:?} is not supported!", mode),
        }

        // Check if we've now bootstrapped
        self.check_if_bootstrapped()
    }

    /// Attempts to make bootstrapping progress by executing transactions since genesis
    async fn execute_transactions_from_genesis(&mut self) -> Result<(), Error> {
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;

        // Check if there are already active streams
        if self.active_data_stream.is_some() {
            if self.bootstrapped_epoch_ending_ledger_infos {
                return self
                    .process_transactions_with_proofs(&latest_storage_summary)
                    .await;
            } else {
                return self
                    .process_epoch_ending_ledger_infos(&latest_storage_summary)
                    .await;
            }
        }

        // If we've already fetched the epoch ending ledger infos, fetch the transactions
        if self.bootstrapped_epoch_ending_ledger_infos {
            return self
                .fetch_transactions_with_proofs(&latest_storage_summary)
                .await;
        }

        // Fetch the epoch ending ledger infos
        if let Err(error) = self
            .fetch_epoch_ending_ledger_infos(&latest_storage_summary)
            .await
        {
            // TODO(joshlind): handle this error!
            error!("Unable to fetch the epoch ending ledger infos: {:?}", error);
        }

        Ok(())
    }

    /// Fetches all epoch ending ledger infos: from the current epoch to the
    /// maximum that can be found by the data streaming service.
    async fn fetch_epoch_ending_ledger_infos(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        let highest_epoch = latest_storage_summary.latest_epoch_state.epoch;
        let epoch_ending_stream = self
            .streaming_service_client
            .get_all_epoch_ending_ledger_infos(highest_epoch)
            .await?;
        self.active_data_stream = Some(epoch_ending_stream);

        Ok(())
    }

    /// Fetches all transactions: from the highest synced transaction version to
    /// the highest stored ledger info version.
    async fn fetch_transactions_with_proofs(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        let highest_transaction_version = latest_storage_summary.latest_synced_version;
        let highest_ledger_info_version = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .version();

        // Ensure that the ledger info version is higher than our highest transaction
        if highest_ledger_info_version < highest_transaction_version {
            return Err(Error::UnexpectedError(format!(
                "Highest ledger info version ({:?}) < highest transaction version ({:?})",
                highest_ledger_info_version, highest_transaction_version
            )));
        }

        // Check if we've bootstrapped
        if highest_ledger_info_version == highest_transaction_version {
            // TODO(joshlind): log me!
            info!("The node has successfully bootstrapped!");
            self.bootstrapped = true; // We've bootstrapped!
            return Ok(());
        }

        // Fetch all transactions until the ledger info version
        let transaction_stream = self
            .streaming_service_client
            .get_all_transactions(
                highest_transaction_version,
                highest_ledger_info_version,
                highest_ledger_info_version,
                false,
            )
            .await?;
        self.active_data_stream = Some(transaction_stream);

        Ok(())
    }

    async fn process_epoch_ending_ledger_infos(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        let mut data_notification_id = None;
        let mut stream_feedback = None;

        // Pull data notifications from the stream
        if let Some(active_data_stream) = &mut self.active_data_stream {
            loop {
                let data_notification = get_data_notification(active_data_stream).await;
                if let Some(data_notification) = data_notification {
                    data_notification_id = Some(data_notification.notification_id);

                    match data_notification.data_payload {
                        DataPayload::EpochEndingLedgerInfos(epoch_ending_ledger_infos) => {
                            Self::process_epoch_ending_payload(
                                latest_storage_summary,
                                epoch_ending_ledger_infos,
                            );
                        }
                        DataPayload::EndOfStream => {
                            let latest_ledger_info =
                                latest_storage_summary.latest_ledger_info.ledger_info();
                            let waypoint = self.driver_configuration.waypoint;
                            if latest_ledger_info.version() >= waypoint.version() {
                                // The waypoint was verified successfully. We've bootstrapped.
                                self.bootstrapped_epoch_ending_ledger_infos = true;
                            }
                            stream_feedback = Some(NotificationFeedback::EndOfStream);
                        }
                        _ => {
                            stream_feedback = Some(NotificationFeedback::PayloadTypeIsIncorrect);
                        }
                    }
                } else {
                    break; // TODO(joshlind): log the fact that nothing was found on the stream
                }
            }
        }

        // Terminate the currently active stream
        if let Some(stream_feedback) = stream_feedback {
            if let Some(data_notification_id) = data_notification_id {
                self.streaming_service_client
                    .terminate_stream_with_feedback(data_notification_id, stream_feedback)
                    .await?;
                self.active_data_stream = None;
                // TODO(joshlind): log the stream termination and reason
            }
        }

        Ok(())
    }

    async fn process_epoch_ending_payload(
        latest_storage_summary: &StorageStateSummary,
        epoch_ending_ledger_infos: Vec<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // TODO(joshlind): process the epoch ending ledger infos!
        // TODO(joshlind): verify the waypoint!

        Ok(())
    }

    async fn process_transactions_with_proofs(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        unimplemented!()
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
        None
    }
}
