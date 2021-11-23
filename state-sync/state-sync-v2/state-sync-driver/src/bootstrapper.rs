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
use diem_config::config::{BootstrappingMode, StateSyncDriverConfig};
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

/// A simple component that manages the bootstrapping of the node
pub struct Bootstrapper<S> {
    // The currently active data stream (provided by the data streaming service)
    active_data_stream: Option<DataStreamListener>,

    // The channel used to notify a listener of successful bootstrapping
    bootstrap_notifier_channel: Option<oneshot::Sender<Result<(), Error>>>,

    // If the node has completed bootstrapped
    bootstrapped: bool,

    // If the node has already bootstrapped all epoch ending ledger infos
    bootstrapped_epoch_ending_ledger_infos: bool,

    // The config of the state sync driver
    driver_configuration: DriverConfiguration,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,

    // The storage synchronizer used to update local storage
    storage_synchronizer: Arc<Mutex<S>>,

    // If the node has successfully verified the waypoint it was given
    verified_waypoint: bool,
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
            verified_waypoint: false,
        }
    }

    /// Returns true iff the node has already completed bootsrapping
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Notifies the given channel when the node has completed bootstrapping
    pub fn subscribe_to_bootstrap_notifications(
        &mut self,
        bootstrap_notifier_channel: oneshot::Sender<Result<(), Error>>,
    ) -> Result<(), Error> {
        self.bootstrap_notifier_channel = Some(bootstrap_notifier_channel);
        self.notify_if_bootstrapped()
    }

    /// Checks if the node has successfully bootstrapped
    fn notify_if_bootstrapped(&mut self) -> Result<(), Error> {
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
    pub async fn check_progress(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.is_bootstrapped() {
            return Err(Error::AlreadyBootstrapped(
                "The bootstrapper should not attempt to make progress!".into(),
            ));
        }

        // Bootstrap the epoch ending ledger infos first
        if !self.bootstrapped_epoch_ending_ledger_infos {
            return if self.active_data_stream.is_some() {
                self.process_epoch_ending_ledger_infos().await
            } else {
                self.fetch_epoch_ending_ledger_infos(global_data_summary)
                    .await
            };
        }

        // Once we have the epoch ending ledger infos, bootstrap using the mode
        match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ApplyTransactionOutputsFromGenesis => {
                self.apply_transaction_outputs_from_genesis(global_data_summary)
                    .await?;
            }
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                self.execute_transactions_from_genesis(global_data_summary)
                    .await?;
            }
            BootstrappingMode::DownloadLatestAccountStates => {
                self.download_latest_account_states(global_data_summary)
                    .await?;
            }
        }

        // Check if we've now bootstrapped
        self.notify_if_bootstrapped()
    }

    /// Attempts to make bootstrapping progress by applying transaction outputs since genesis
    async fn apply_transaction_outputs_from_genesis(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_transaction_outputs_with_proof().await
        } else {
            self.fetch_transaction_outputs_with_proofs().await
        }
    }

    /// Attempts to make bootstrapping progress by executing transactions since genesis
    async fn execute_transactions_from_genesis(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_transactions_with_proofs().await
        } else {
            self.fetch_transactions_with_proofs().await
        }
    }

    /// Attempts to make bootstrapping progress by downloading the latest account states
    async fn download_latest_account_states(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.active_data_stream.is_some() {
            self.process_account_states_with_proofs().await
        } else {
            self.fetch_account_states_with_proofs().await
        }
    }

    /// Fetches all accounts: at the highest epoch ending ledger info version
    async fn fetch_account_states_with_proofs(&mut self) -> Result<(), Error> {
        // Fetch the highest ledger info stored locally
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let latest_ledger_info = latest_storage_summary.latest_ledger_info.ledger_info();
        let latest_ledger_info_version = latest_ledger_info.version();

        // Fetch the accounts at the latest ledger info version
        debug!(
            "Fetching the account states stream at version: {:?}",
            latest_ledger_info_version
        );
        // TODO(joshlind): identify the start index and keep track of how many accounts we've written!
        let start_index = 0;
        let account_states_stream = self
            .streaming_service_client
            .get_all_accounts(latest_ledger_info_version, Some(start_index))
            .await?;
        self.active_data_stream = Some(account_states_stream);

        Ok(())
    }

    /// Fetches all epoch ending ledger infos: from the current epoch to the
    /// maximum that can be found by the data streaming service.
    async fn fetch_epoch_ending_ledger_infos(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        // Verify the waypoint can be satisfied by our peers
        self.verify_waypoint_is_satisfiable(global_data_summary)?;

        // Get the highest advertised epoch that has ended
        let highest_advertised_epoch_end = global_data_summary
            .advertised_data
            .highest_epoch_ending_ledger_info()
            .ok_or_else(|| {
                Error::AdvertisedDataError(
                    "No highest advertised epoch end found in the network!".into(),
                )
            })?;

        // Fetch the highest epoch end stored locally
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let latest_ledger_info = latest_storage_summary.latest_ledger_info.ledger_info();
        let highest_local_epoch_end = if latest_ledger_info.ends_epoch() {
            Some(latest_ledger_info.epoch())
        } else if latest_ledger_info.epoch() > 0 {
            Some(latest_ledger_info.epoch().checked_sub(1).ok_or_else(|| {
                Error::IntegerOverflow("The highest local epoch end has overflown!".into())
            })?)
        } else {
            None
        };

        // Compare the highest local epoch end to the highest advertised epoch end
        if let Some(highest_local_epoch_end) = highest_local_epoch_end {
            if highest_local_epoch_end < highest_advertised_epoch_end {
                debug!("Found higher epoch ending ledger infos in the network! Local: {:?}, advertised: {:?}",
                       highest_local_epoch_end, highest_advertised_epoch_end);
                let next_epoch_end = highest_local_epoch_end.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("The next epoch end has overflown!".into())
                })?;
                let epoch_ending_stream = self
                    .streaming_service_client
                    .get_all_epoch_ending_ledger_infos(next_epoch_end)
                    .await?;
                self.active_data_stream = Some(epoch_ending_stream);
            } else if self.verified_waypoint || self.driver_configuration.waypoint.version() == 0 {
                debug!("We've bootstrapped the epoch ending ledger infos!");
                self.bootstrapped_epoch_ending_ledger_infos = true;
            } else {
                error!("Our waypoint is unverified, but there's no higher epoch ending ledger infos advertised!");
            };
        } else {
            debug!("No highest local epoch end found! Starting to sync at epoch 0.");
            let epoch_ending_stream = self
                .streaming_service_client
                .get_all_epoch_ending_ledger_infos(0)
                .await?;
            self.active_data_stream = Some(epoch_ending_stream);
        };

        Ok(())
    }

    /// Verifies that connected peers have advertised data beyond our waypoint
    fn verify_waypoint_is_satisfiable(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        panic!("Given waypoint! {:?}", self.driver_configuration.waypoint);

        // Get the highest advertised synced ledger info version
        let highest_advertised_ledger_info = global_data_summary
            .advertised_data
            .highest_synced_ledger_info()
            .ok_or_else(|| {
                Error::AdvertisedDataError(
                    "No highest advertised ledger info found in the network!".into(),
                )
            })?;
        let highest_advertised_version = highest_advertised_ledger_info.ledger_info().version();

        // Compare the highest advertised version with our waypoint
        let waypoint_version = self.driver_configuration.waypoint.version();
        if highest_advertised_version < waypoint_version {
            Err(Error::AdvertisedDataError(
                format!(
                    "No advertised version higher than our waypoint! Highest version: {:?}, waypoint version: {:?}",
                        highest_advertised_version, waypoint_version
                )
            ))
        } else {
            Ok(())
        }
    }

    /// Returns the highest synced version and ledger version in storage
    fn get_highest_synced_and_ledger_versions(&self) -> Result<(Version, Version), Error> {
        // Fetch the highest synced and ledger info versions
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let highest_synced_version = latest_storage_summary.latest_synced_version;
        let highest_ledger_info_version = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .version();

        // Ensure that the ledger info version is >= the synced version
        if highest_ledger_info_version < highest_synced_version {
            Err(Error::UnexpectedError(format!(
                "Highest ledger info version ({:?}) < highest synced version ({:?})",
                highest_ledger_info_version, highest_synced_version
            )))
        } else {
            Ok((highest_synced_version, highest_ledger_info_version))
        }
    }

    /// Fetches all transactions: from the highest synced transaction version + 1
    /// to the highest stored ledger info version.
    async fn fetch_transactions_with_proofs(&mut self) -> Result<(), Error> {
        let (highest_synced_version, highest_ledger_info_version) =
            self.get_highest_synced_and_ledger_versions()?;

        // Check if we've bootstrapped
        if highest_ledger_info_version == highest_synced_version {
            info!("The node has successfully bootstrapped!");
            self.bootstrapped = true;
            return Ok(());
        }

        // Fetch all transactions until the ledger info version
        let next_transaction_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next transaction version has overflown!".into())
        })?;
        let transaction_stream = self
            .streaming_service_client
            .get_all_transactions(
                next_transaction_version,
                highest_ledger_info_version,
                highest_ledger_info_version,
                false,
            )
            .await?;
        self.active_data_stream = Some(transaction_stream);

        Ok(())
    }

    /// Fetches all transaction outputs: from the highest synced transaction
    /// version + 1 to the highest stored ledger info version.
    async fn fetch_transaction_outputs_with_proofs(&mut self) -> Result<(), Error> {
        let (highest_synced_version, highest_ledger_info_version) =
            self.get_highest_synced_and_ledger_versions()?;

        // Check if we've bootstrapped
        if highest_ledger_info_version == highest_synced_version {
            info!("The node has successfully bootstrapped!");
            self.bootstrapped = true;
            return Ok(());
        }

        // Fetch all transaction outputs until the ledger info version
        let next_output_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next output version has overflown!".into())
        })?;
        let transaction_stream = self
            .streaming_service_client
            .get_all_transaction_outputs(
                next_output_version,
                highest_ledger_info_version,
                highest_ledger_info_version,
            )
            .await?;
        self.active_data_stream = Some(transaction_stream);

        Ok(())
    }

    async fn process_account_states_with_proofs(&mut self) -> Result<(), Error> {
        // Pull data notifications from the stream
        let mut stream_feedback = None;
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Some(active_data_stream) = &mut self.active_data_stream {
            while let Some(data_notification) = get_data_notification(active_data_stream).await {
                match data_notification.data_payload {
                    DataPayload::AccountStatesWithProof(account_states_with_proof) => {
                        process_account_states_payload(
                            &latest_storage_summary,
                            account_states_with_proof,
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

    async fn process_epoch_ending_ledger_infos(&mut self) -> Result<(), Error> {
        // Pull data notifications from the stream
        let mut stream_feedback = None;
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Some(active_data_stream) = &mut self.active_data_stream {
            while let Some(data_notification) = get_data_notification(active_data_stream).await {
                match data_notification.data_payload {
                    DataPayload::EpochEndingLedgerInfos(epoch_ending_ledger_infos) => {
                        process_epoch_ending_payload(
                            &latest_storage_summary,
                            epoch_ending_ledger_infos,
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

    async fn process_transactions_with_proofs(&mut self) -> Result<(), Error> {
        // Pull data notifications from the stream
        let mut stream_feedback = None;
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Some(active_data_stream) = &mut self.active_data_stream {
            while let Some(data_notification) = get_data_notification(active_data_stream).await {
                match data_notification.data_payload {
                    DataPayload::TransactionsWithProof(transactions_with_proof) => {
                        process_transactions_payload(
                            &latest_storage_summary,
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
                    DataPayload::TransactionOutputsWithProof(transaction_outputs_with_proof) => {
                        process_transaction_outputs_payload(
                            &latest_storage_summary,
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

async fn process_account_states_payload(
    latest_storage_summary: &StorageStateSummary,
    account_states_with_proof: AccountStatesChunkWithProof,
) -> Result<(), Error> {
    debug!(
        "Received an account states with proof! {:?}",
        account_states_with_proof
    );
    // TODO(joshlind): process the transaction list with proof!
    // TODO(joshlind): verify the waypoint!

    Ok(())
}

async fn process_epoch_ending_payload(
    latest_storage_summary: &StorageStateSummary,
    epoch_ending_ledger_infos: Vec<LedgerInfoWithSignatures>,
) -> Result<(), Error> {
    debug!(
        "Received an epoch ending payload! {:?}",
        epoch_ending_ledger_infos
    );
    // TODO(joshlind): process the epoch ending ledger infos!
    // TODO(joshlind): verify the waypoint!

    Ok(())
}

async fn process_transaction_outputs_payload(
    latest_storage_summary: &StorageStateSummary,
    transaction_outputs_with_proof: TransactionOutputListWithProof,
) -> Result<(), Error> {
    debug!(
        "Received a transaction output with proof! {:?}",
        transaction_outputs_with_proof
    );
    // TODO(joshlind): process the transaction list with proof!
    // TODO(joshlind): verify the waypoint!

    Ok(())
}

async fn process_transactions_payload(
    latest_storage_summary: &StorageStateSummary,
    transactions_with_proof: TransactionListWithProof,
) -> Result<(), Error> {
    debug!(
        "Received a transaction with proof! {:?}",
        transactions_with_proof
    );
    // TODO(joshlind): process the transaction list with proof!
    // TODO(joshlind): verify the waypoint!

    Ok(())
}
