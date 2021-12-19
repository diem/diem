// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::DriverConfiguration, error::Error, storage_synchronizer::StorageSynchronizerInterface,
    utils,
};
use data_streaming_service::{
    data_notification::{DataNotification, DataPayload, NotificationId},
    data_stream::DataStreamListener,
    streaming_client::{DataStreamingClient, NotificationFeedback, StreamingServiceClient},
};
use diem_config::config::BootstrappingMode;
use diem_data_client::GlobalDataSummary;
use diem_infallible::Mutex;
use diem_logger::*;
use diem_types::{
    epoch_change::Verifier,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, TransactionOutputListWithProof, Version},
    waypoint::Waypoint,
};
use futures::channel::oneshot;
use std::{collections::BTreeMap, sync::Arc};
use storage_interface::DbReader;

/// A simple container for verified epoch states and epoch ending ledger infos
/// that have been fetched from the network.
struct VerifiedEpochStates {
    // If new epoch ending ledger infos have been fetched from the network
    fetched_epoch_ending_ledger_infos: bool,

    // The highest epoch ending version fetched thus far
    highest_fetched_epoch_ending_version: Version,

    // The latest epoch state that has been verified by the node
    latest_epoch_state: EpochState,

    // A map from versions to epoch ending ledger infos fetched from the network
    new_epoch_ending_ledger_infos: BTreeMap<Version, LedgerInfoWithSignatures>,

    // If the node has successfully verified the waypoint
    verified_waypoint: bool,
}

impl VerifiedEpochStates {
    pub fn new(latest_epoch_state: EpochState) -> Self {
        Self {
            fetched_epoch_ending_ledger_infos: false,
            highest_fetched_epoch_ending_version: 0,
            latest_epoch_state,
            new_epoch_ending_ledger_infos: BTreeMap::new(),
            verified_waypoint: false,
        }
    }

    /// Returns true iff the node has already fetched any new epoch
    /// ending ledger infos from the network.
    pub fn fetched_epoch_ending_ledger_infos(&self) -> bool {
        self.fetched_epoch_ending_ledger_infos
    }

    /// Sets `fetched_epoch_ending_ledger_infos` to true
    pub fn set_fetched_epoch_ending_ledger_infos(&mut self) {
        self.fetched_epoch_ending_ledger_infos = true;
    }

    /// Returns true iff the node has verified the waypoint
    pub fn verified_waypoint(&self) -> bool {
        self.verified_waypoint
    }

    /// Sets `verified_waypoint` to true
    pub fn set_verified_waypoint(&mut self) {
        self.verified_waypoint = true;
    }

    /// Verifies the given epoch ending ledger info, updates our latest
    /// trusted epoch state and attempts to verify any given waypoint.
    pub fn verify_epoch_ending_ledger_info(
        &mut self,
        epoch_ending_ledger_info: &LedgerInfoWithSignatures,
        waypoint: &Waypoint,
    ) -> Result<(), Error> {
        // Verify the ledger info against the latest epoch state
        self.latest_epoch_state
            .verify(epoch_ending_ledger_info)
            .map_err(|error| {
                Error::VerificationError(format!("Ledger info failed verification: {:?}", error))
            })?;

        // Update the latest epoch state with the next epoch
        if let Some(next_epoch_state) = epoch_ending_ledger_info.ledger_info().next_epoch_state() {
            self.highest_fetched_epoch_ending_version =
                epoch_ending_ledger_info.ledger_info().version();
            self.latest_epoch_state = next_epoch_state.clone();
            self.insert_new_epoch_ending_ledger_info(epoch_ending_ledger_info.clone());

            trace!(
                "Updated the latest epoch state to epoch: {:?}",
                self.latest_epoch_state.epoch
            );
        } else {
            return Err(Error::VerificationError(
                "The ledger info was not epoch ending!".into(),
            ));
        }

        // Check if the ledger info corresponds to the trusted waypoint
        self.verify_waypoint(epoch_ending_ledger_info, waypoint)
    }

    /// Attempts to verify the waypoint using the new epoch ending ledger info
    fn verify_waypoint(
        &mut self,
        epoch_ending_ledger_info: &LedgerInfoWithSignatures,
        waypoint: &Waypoint,
    ) -> Result<(), Error> {
        if !self.verified_waypoint {
            // Fetch the waypoint and ledger info versions
            let waypoint_version = waypoint.version();
            let ledger_info = epoch_ending_ledger_info.ledger_info();
            let ledger_info_version = ledger_info.version();

            // Verify we haven't missed the waypoint
            if ledger_info_version > waypoint_version {
                return Err(Error::VerificationError(
                    format!("Failed to verify the waypoint: ledger info version is too high! Waypoint version: {:?}, ledger info version: {:?}",
                            waypoint_version, ledger_info_version)
                ));
            }

            // Check if we've found the ledger info corresponding to the waypoint version
            if ledger_info_version == waypoint_version {
                match waypoint.verify(ledger_info) {
                    Ok(()) => self.verified_waypoint = true,
                    Err(error) => {
                        return Err(Error::VerificationError(
                            format!("Failed to verify the waypoint: {:?}! Waypoint: {:?}, given ledger info: {:?}",
                                    error, waypoint, ledger_info)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Adds an epoch ending ledger info to the new epoch ending ledger infos map
    fn insert_new_epoch_ending_ledger_info(
        &mut self,
        epoch_ending_ledger_info: LedgerInfoWithSignatures,
    ) {
        debug!(
            "Adding a new epoch to the epoch ending ledger infos: {:?}",
            &epoch_ending_ledger_info
        );

        // Insert the version to ledger info mapping
        let version = epoch_ending_ledger_info.ledger_info().version();
        if let Some(epoch_ending_ledger_info) = self
            .new_epoch_ending_ledger_infos
            .insert(version, epoch_ending_ledger_info)
        {
            panic!(
                "Duplicate epoch ending ledger info found!\
                 Version: {:?}, \
                 ledger info: {:?}",
                version, epoch_ending_ledger_info,
            );
        }
    }

    /// Returns any epoch ending ledger info associated with the given version
    pub fn get_epoch_ending_ledger_info(
        &self,
        version: Version,
    ) -> Option<LedgerInfoWithSignatures> {
        self.new_epoch_ending_ledger_infos.get(&version).cloned()
    }

    /// Returns the highest known ledger info (including the newly fetch ones)
    pub fn get_highest_known_ledger_info(
        &self,
        mut highest_known_ledger_info: LedgerInfoWithSignatures,
    ) -> LedgerInfoWithSignatures {
        // Check if we've fetched a higher versioned ledger info from the network
        if !self.new_epoch_ending_ledger_infos.is_empty() {
            let highest_fetched_ledger_info = self
                .get_epoch_ending_ledger_info(self.highest_fetched_epoch_ending_version)
                .unwrap_or_else(|| {
                    panic!(
                        "The highest known ledger info for version: {:?} was not found!",
                        self.highest_fetched_epoch_ending_version
                    )
                });

            if highest_fetched_ledger_info.ledger_info().version()
                > highest_known_ledger_info.ledger_info().version()
            {
                highest_known_ledger_info = highest_fetched_ledger_info;
            }
        }

        highest_known_ledger_info
    }

    /// Returns the next epoch ending version after the given version (if one
    /// exists).
    pub fn next_epoch_ending_version(&self, version: Version) -> Option<Version> {
        // BTreeMap keys are iterated through in increasing key orders (i.e., versions)
        for (epoch_ending_version, _) in self.new_epoch_ending_ledger_infos.iter() {
            if *epoch_ending_version > version {
                return Some(*epoch_ending_version);
            }
        }
        None
    }
}

/// A simple component that manages the bootstrapping of the node
pub struct Bootstrapper<StorageSyncer> {
    // The currently active data stream (provided by the data streaming service)
    active_data_stream: Option<DataStreamListener>,

    // The channel used to notify a listener of successful bootstrapping
    bootstrap_notifier_channel: Option<oneshot::Sender<Result<(), Error>>>,

    // If the node has completed bootstrapping
    bootstrapped: bool,

    // The config of the state sync driver
    driver_configuration: DriverConfiguration,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,

    // The interface to read from storage
    storage: Arc<dyn DbReader>,

    // The storage synchronizer used to update local storage
    storage_synchronizer: Arc<Mutex<StorageSyncer>>,

    // The epoch states verified by this node (held in memory)
    verified_epoch_states: VerifiedEpochStates,
}

impl<StorageSyncer: StorageSynchronizerInterface> Bootstrapper<StorageSyncer> {
    pub fn new(
        driver_configuration: DriverConfiguration,
        streaming_service_client: StreamingServiceClient,
        storage: Arc<dyn DbReader>,
        storage_synchronizer: Arc<Mutex<StorageSyncer>>,
    ) -> Self {
        // Load the latest epoch state from storage
        let latest_epoch_state = utils::fetch_latest_epoch_state(storage.clone())
            .expect("Unable to fetch latest epoch state!");
        let verified_epoch_states = VerifiedEpochStates::new(latest_epoch_state);

        Self {
            active_data_stream: None,
            bootstrap_notifier_channel: None,
            bootstrapped: false,
            driver_configuration,
            streaming_service_client,
            storage,
            storage_synchronizer,
            verified_epoch_states,
        }
    }

    /// Returns true iff the node has already completed bootstrapping
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Subscribes the specified channel to bootstrap completion notifications
    pub fn subscribe_to_bootstrap_notifications(
        &mut self,
        bootstrap_notifier_channel: oneshot::Sender<Result<(), Error>>,
    ) {
        if self.bootstrap_notifier_channel.is_some() {
            panic!("Only one boostrap subscriber is supported at a time!");
        }

        self.bootstrap_notifier_channel = Some(bootstrap_notifier_channel)
    }

    /// Notifies any listeners if we've now bootstrapped
    fn notify_if_bootstrapped(&mut self) -> Result<(), Error> {
        if self.bootstrapped {
            if let Some(notifier_channel) = self.bootstrap_notifier_channel.take() {
                if let Err(error) = notifier_channel.send(Ok(())) {
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
    pub async fn drive_progress(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        if self.is_bootstrapped() {
            return Err(Error::AlreadyBootstrapped(
                "The bootstrapper should not attempt to make progress!".into(),
            ));
        }

        if self.active_data_stream.is_some() {
            // We have an active data stream. Process any notifications!
            self.process_active_stream_notifications().await?;
        } else {
            // Fetch a new data stream to start streaming data
            self.initialize_active_data_stream(global_data_summary)
                .await?;
        }

        // Check if we've now bootstrapped
        self.notify_if_bootstrapped()
    }

    /// Initializes an active data stream so that we can begin to process notifications
    async fn initialize_active_data_stream(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        // Always fetch the new epoch ending ledger infos first
        if !self
            .verified_epoch_states
            .fetched_epoch_ending_ledger_infos()
            || !self.verified_epoch_states.verified_waypoint()
        {
            return self
                .fetch_epoch_ending_ledger_infos(global_data_summary)
                .await;
        }

        // Get the highest synced and known ledger info versions
        let highest_synced_version = utils::fetch_latest_synced_version(self.storage.clone())?;
        let highest_known_ledger_info = self.get_highest_known_ledger_info()?;
        let highest_known_ledger_version = highest_known_ledger_info.ledger_info().version();

        // Check if we've already fetched the required data for bootstrapping
        if highest_synced_version == highest_known_ledger_version {
            info!("The node has successfully bootstrapped!");
            self.bootstrapped = true;
            return Ok(());
        }

        // Verify we haven't synced beyond the highest ledger info
        if highest_synced_version > highest_known_ledger_version {
            unreachable!(
                "Synced beyond the highest ledger info! Synced version: {:?}, highest ledger version: {:?}", highest_synced_version, highest_known_ledger_version
            );
        }

        // Fetch all data until the epoch ending ledger info for the current epoch
        let next_version = highest_synced_version.checked_add(1).ok_or_else(|| {
            Error::IntegerOverflow("The next output version has overflown!".into())
        })?;
        let end_version = self
            .verified_epoch_states
            .next_epoch_ending_version(highest_synced_version)
            .expect("No higher epoch ending version known!");
        let data_stream = match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ApplyTransactionOutputsFromGenesis => {
                self.streaming_service_client
                    .get_all_transaction_outputs(
                        next_version,
                        end_version,
                        highest_known_ledger_version,
                    )
                    .await?
            }
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                self.streaming_service_client
                    .get_all_transactions(
                        next_version,
                        end_version,
                        highest_known_ledger_version,
                        false,
                    )
                    .await?
            }
            bootstrapping_mode => {
                unimplemented!("Bootstrapping mode not supported: {:?}", bootstrapping_mode)
            }
        };
        self.active_data_stream = Some(data_stream);

        Ok(())
    }

    /// Processes any notifications already pending on the active stream
    async fn process_active_stream_notifications(&mut self) -> Result<(), Error> {
        loop {
            // Fetch and process any data notifications
            let data_notification =
                utils::get_data_notification(self.active_data_stream.as_mut()).await?;
            match data_notification.data_payload {
                DataPayload::EpochEndingLedgerInfos(epoch_ending_ledger_infos) => {
                    self.process_epoch_ending_payload(
                        data_notification.notification_id,
                        epoch_ending_ledger_infos,
                    )
                    .await?;
                }
                DataPayload::TransactionsWithProof(transactions_with_proof) => {
                    let payload_start_version = transactions_with_proof.first_transaction_version;
                    self.process_transaction_or_output_payload(
                        data_notification.notification_id,
                        Some(transactions_with_proof),
                        None,
                        payload_start_version,
                    )
                    .await?;
                }
                DataPayload::TransactionOutputsWithProof(transaction_outputs_with_proof) => {
                    let payload_start_version =
                        transaction_outputs_with_proof.first_transaction_output_version;
                    self.process_transaction_or_output_payload(
                        data_notification.notification_id,
                        None,
                        Some(transaction_outputs_with_proof),
                        payload_start_version,
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

    /// Fetches all epoch ending ledger infos (from the current epoch to the
    /// maximum that can be found by the data streaming service).
    async fn fetch_epoch_ending_ledger_infos(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        // Verify the waypoint can be satisfied
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

        // Fetch the highest epoch end known locally
        let highest_known_ledger_info = self.get_highest_known_ledger_info()?;
        let highest_known_ledger_info = highest_known_ledger_info.ledger_info();
        let highest_local_epoch_end = if highest_known_ledger_info.ends_epoch() {
            highest_known_ledger_info.epoch()
        } else if highest_known_ledger_info.epoch() > 0 {
            highest_known_ledger_info
                .epoch()
                .checked_sub(1)
                .ok_or_else(|| {
                    Error::IntegerOverflow("The highest local epoch end has overflown!".into())
                })?
        } else {
            unreachable!("Genesis should always end epoch 0!");
        };

        // Compare the highest local epoch end to the highest advertised epoch end
        if highest_local_epoch_end > highest_advertised_epoch_end {
            let error_message =
                format!(
                    "The highest local epoch end is higher than the advertised epoch end! Local: {:?}, advertised: {:?}",
                    highest_local_epoch_end, highest_advertised_epoch_end
                );
            return Err(Error::AdvertisedDataError(error_message));
        } else if highest_local_epoch_end < highest_advertised_epoch_end {
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
        } else if self.verified_epoch_states.verified_waypoint() {
            debug!("No new epoch ending ledger infos to fetch! All peers are in the same epoch!");
            self.verified_epoch_states
                .set_fetched_epoch_ending_ledger_infos();
        } else {
            return Err(Error::AdvertisedDataError("Our waypoint is unverified, but there's no higher epoch ending ledger infos advertised!".into()));
        };

        Ok(())
    }

    /// Verifies that connected peers have advertised data beyond our waypoint
    /// or that our waypoint is trivially satisfiable.
    fn verify_waypoint_is_satisfiable(
        &mut self,
        global_data_summary: &GlobalDataSummary,
    ) -> Result<(), Error> {
        // If our storage has already synced beyond our waypoint, nothing needs to be checked
        let latest_ledger_info = utils::fetch_latest_synced_ledger_info(self.storage.clone())?;
        let waypoint_version = self.driver_configuration.waypoint.version();
        if latest_ledger_info.ledger_info().version() >= waypoint_version {
            self.verified_epoch_states.set_verified_waypoint();
            return Ok(());
        }

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

    /// Process a single epoch ending payload
    async fn process_epoch_ending_payload(
        &mut self,
        notification_id: NotificationId,
        epoch_ending_ledger_infos: Vec<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // Verify the payload isn't empty
        if epoch_ending_ledger_infos.is_empty() {
            self.terminate_active_stream(notification_id, NotificationFeedback::EmptyPayloadData)
                .await?;
            return Err(Error::VerificationError(
                "The epoch ending payload was empty!".into(),
            ));
        }

        // Verify the epoch change proofs, update our latest epoch state and
        // verify our waypoint.
        for epoch_ending_ledger_info in epoch_ending_ledger_infos {
            if let Err(error) = self.verified_epoch_states.verify_epoch_ending_ledger_info(
                &epoch_ending_ledger_info,
                &self.driver_configuration.waypoint,
            ) {
                self.terminate_active_stream(
                    notification_id,
                    NotificationFeedback::PayloadProofFailed,
                )
                .await?;
                return Err(error);
            }
        }

        // TODO(joshlind): do we want to preemptively notify certain components
        // of the new reconfigurations?

        Ok(())
    }

    /// Process a single transaction or transaction output data payload
    async fn process_transaction_or_output_payload(
        &mut self,
        notification_id: NotificationId,
        transaction_list_with_proof: Option<TransactionListWithProof>,
        transaction_outputs_with_proof: Option<TransactionOutputListWithProof>,
        payload_start_version: Option<Version>,
    ) -> Result<(), Error> {
        // Verify the payload starting version
        self.verify_payload_start_version(notification_id, payload_start_version)
            .await?;

        // Get the end of epoch ledger info if the payload ends the epoch
        let end_of_epoch_ledger_info = self
            .get_end_of_epoch_ledger_info(
                notification_id,
                payload_start_version,
                transaction_list_with_proof.as_ref(),
                transaction_outputs_with_proof.as_ref(),
            )
            .await?;

        // Get the highest known ledger info (this should be the proof ledger info)
        let highest_known_ledger_info = self.get_highest_known_ledger_info()?;

        // Execute/apply and commit the transactions/outputs
        match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ApplyTransactionOutputsFromGenesis => {
                if let Some(transaction_outputs_with_proof) = transaction_outputs_with_proof {
                    self.storage_synchronizer.lock().apply_transaction_outputs(
                        transaction_outputs_with_proof,
                        highest_known_ledger_info,
                        end_of_epoch_ledger_info,
                    )?;
                } else {
                    self.terminate_active_stream(
                        notification_id,
                        NotificationFeedback::PayloadTypeIsIncorrect,
                    )
                    .await?;
                    return Err(Error::InvalidPayload(
                        "Did not receive transaction outputs with proof!".into(),
                    ));
                }
            }
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                if let Some(transaction_list_with_proof) = transaction_list_with_proof {
                    self.storage_synchronizer.lock().execute_transactions(
                        transaction_list_with_proof,
                        highest_known_ledger_info,
                        end_of_epoch_ledger_info,
                    )?;
                } else {
                    self.terminate_active_stream(
                        notification_id,
                        NotificationFeedback::PayloadTypeIsIncorrect,
                    )
                    .await?;
                    return Err(Error::InvalidPayload(
                        "Did not receive transactions with proof!".into(),
                    ));
                }
            }
            bootstrapping_mode => {
                unimplemented!("Bootstrapping mode not supported: {:?}", bootstrapping_mode)
            }
        };

        Ok(())
    }

    /// Verifies the first payload version matches the version we wish to sync
    async fn verify_payload_start_version(
        &mut self,
        notification_id: NotificationId,
        payload_start_version: Option<Version>,
    ) -> Result<(), Error> {
        // Fetch the highest synced version
        let highest_synced_version = utils::fetch_latest_synced_version(self.storage.clone())?;

        // Compare the payload start version with the expected version
        if let Some(payload_start_version) = payload_start_version {
            let expected_version = highest_synced_version
                .checked_add(1)
                .ok_or_else(|| Error::IntegerOverflow("Expected version has overflown!".into()))?;

            if payload_start_version != expected_version {
                self.terminate_active_stream(
                    notification_id,
                    NotificationFeedback::InvalidPayloadData,
                )
                .await?;
                Err(Error::VerificationError(format!(
                    "The payload start version does not match the expected version! Start: {:?}, expected: {:?}",
                    payload_start_version, expected_version
                )))
            } else {
                Ok(())
            }
        } else {
            self.terminate_active_stream(notification_id, NotificationFeedback::EmptyPayloadData)
                .await?;
            Err(Error::VerificationError(
                "The playload starting version is missing!".into(),
            ))
        }
    }

    /// Returns the end of epoch ledger info for the given payload. If
    /// calculation fails, the active stream is terminated. Assumes the
    /// `payload_start_version` exists.
    async fn get_end_of_epoch_ledger_info(
        &mut self,
        notification_id: NotificationId,
        payload_start_version: Option<Version>,
        transaction_list_with_proof: Option<&TransactionListWithProof>,
        transaction_outputs_with_proof: Option<&TransactionOutputListWithProof>,
    ) -> Result<Option<LedgerInfoWithSignatures>, Error> {
        let payload_start_version =
            payload_start_version.expect("Payload start version should exist!");

        // Calculate the payload end version
        let num_versions = match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ApplyTransactionOutputsFromGenesis => {
                if let Some(transaction_outputs_with_proof) = transaction_outputs_with_proof {
                    transaction_outputs_with_proof
                        .transactions_and_outputs
                        .len()
                } else {
                    self.terminate_active_stream(
                        notification_id,
                        NotificationFeedback::PayloadTypeIsIncorrect,
                    )
                    .await?;
                    return Err(Error::InvalidPayload(
                        "Did not receive transaction outputs with proof!".into(),
                    ));
                }
            }
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                if let Some(transaction_list_with_proof) = transaction_list_with_proof {
                    transaction_list_with_proof.transactions.len()
                } else {
                    self.terminate_active_stream(
                        notification_id,
                        NotificationFeedback::PayloadTypeIsIncorrect,
                    )
                    .await?;
                    return Err(Error::InvalidPayload(
                        "Did not receive transactions with proof!".into(),
                    ));
                }
            }
            bootstrapping_mode => {
                unimplemented!("Bootstrapping mode not supported: {:?}", bootstrapping_mode)
            }
        };
        let payload_end_version = payload_start_version
            .checked_add(num_versions as u64)
            .and_then(|v| v.checked_sub(1))
            .ok_or_else(|| {
                Error::IntegerOverflow("The payload end version has overflown!".into())
            })?; // payload_end_version = payload_start_version + num_versions - 1

        // Find any epoch ending ledger info at the payload end version
        Ok(self
            .verified_epoch_states
            .get_epoch_ending_ledger_info(payload_end_version))
    }

    /// Returns the highest known ledger info (including the newly fetch ones)
    fn get_highest_known_ledger_info(&self) -> Result<LedgerInfoWithSignatures, Error> {
        let latest_synced_ledger_info =
            utils::fetch_latest_synced_ledger_info(self.storage.clone())?;
        Ok(self
            .verified_epoch_states
            .get_highest_known_ledger_info(latest_synced_ledger_info))
    }

    /// Handles the end of stream notification or an invalid payload by
    /// terminating the stream appropriately.
    pub async fn handle_end_of_stream_or_invalid_payload(
        &mut self,
        data_notification: DataNotification,
    ) -> Result<(), Error> {
        self.active_data_stream = None;

        utils::handle_end_of_stream_or_invalid_payload(
            &mut self.streaming_service_client,
            data_notification,
        )
        .await
    }

    /// Terminates the currently active stream with the provided feedback
    async fn terminate_active_stream(
        &mut self,
        notification_id: NotificationId,
        notification_feedback: NotificationFeedback,
    ) -> Result<(), Error> {
        self.active_data_stream = None;

        utils::terminate_stream_with_feedback(
            &mut self.streaming_service_client,
            notification_id,
            notification_feedback,
        )
        .await
    }
}
