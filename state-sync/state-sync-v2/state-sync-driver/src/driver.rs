// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    bootstrapper::Bootstrapper,
    continuous_syncer::ContinuousSyncer,
    driver_client::{ClientNotificationListener, DriverClient, DriverNotification},
    error::Error,
    notification_handlers::{ConsensusNotificationHandler, MempoolNotificationHandler},
    storage_synchronizer::{StorageStateSummary, StorageSynchronizerInterface},
};
use consensus_notifications::{
    ConsensusCommitNotification, ConsensusNotification, ConsensusSyncNotification,
};
use data_streaming_service::streaming_client::StreamingServiceClient;
use diem_config::config::{BootstrappingMode, RoleType, StateSyncDriverConfig};
use diem_data_client::{DiemDataClient, GlobalDataSummary};
use diem_infallible::Mutex;
use diem_logger::*;
use diem_metrics::{register_int_gauge_vec, IntGaugeVec};
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use event_notifications::{EventNotificationSender, EventSubscriptionService};
use futures::StreamExt;
use mempool_notifications::MempoolNotificationSender;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;

pub static VERSION: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "diem_state_sync_version",
        "Version involved in state sync v2 progress",
        &["type"]
    )
    .unwrap()
});

pub fn set_version(version_type: String, version: u64) {
    VERSION
        .with_label_values(&[&version_type])
        .set(version as i64)
}

pub fn get_version(version_type: String) -> u64 {
    VERSION.with_label_values(&[&version_type]).get() as u64
}

/// The configuration of the state sync driver
#[derive(Clone)]
pub struct DriverConfiguration {
    // The config file of the driver
    pub config: StateSyncDriverConfig,

    // The role of the node
    pub role: RoleType,

    // The trusted waypoint for the node
    pub waypoint: Waypoint,
}

impl DriverConfiguration {
    pub fn new(config: StateSyncDriverConfig, role: RoleType, waypoint: Waypoint) -> Self {
        Self {
            config,
            role,
            waypoint,
        }
    }
}

/// The state sync driver that drives synchronization progress
pub struct StateSyncDriver<D, M, S> {
    // The component that manages the initial bootrapping of the node
    bootstrapper: Bootstrapper<S>,

    // The listener for client notifications
    client_notification_listener: ClientNotificationListener,

    // The handler for notifications from consensus
    consensus_notification_handler: ConsensusNotificationHandler,

    // The component that manages the continuous syncing of the node
    continuous_syncer: ContinuousSyncer<S>,

    // The client for checking the global data summary of our peers
    diem_data_client: D,

    // The configuration for the driver
    driver_configuration: DriverConfiguration,

    // The event subscription service to notify listeners of on-chain events
    event_subscription_service: EventSubscriptionService,

    // The handler for notifications to mempool
    mempool_notification_handler: MempoolNotificationHandler<M>,

    // The storage synchronizer used to update local storage
    storage_synchronizer: Arc<Mutex<S>>,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,
}

impl<
        D: DiemDataClient + Send + Clone + 'static,
        M: MempoolNotificationSender,
        S: StorageSynchronizerInterface,
    > StateSyncDriver<D, M, S>
{
    pub fn new(
        client_notification_listener: ClientNotificationListener,
        consensus_notification_handler: ConsensusNotificationHandler,
        driver_configuration: DriverConfiguration,
        event_subscription_service: EventSubscriptionService,
        mempool_notification_handler: MempoolNotificationHandler<M>,
        storage_synchronizer: S,
        diem_data_client: D,
        streaming_service_client: StreamingServiceClient,
    ) -> Self {
        let storage_synchronizer = Arc::new(Mutex::new(storage_synchronizer));
        let bootstrapper = Bootstrapper::new(
            driver_configuration.clone(),
            streaming_service_client.clone(),
            storage_synchronizer.clone(),
        );
        let continuous_syncer = ContinuousSyncer::new(
            driver_configuration.clone(),
            streaming_service_client.clone(),
            storage_synchronizer.clone(),
        );

        Self {
            bootstrapper,
            client_notification_listener,
            continuous_syncer,
            consensus_notification_handler,
            diem_data_client,
            driver_configuration,
            event_subscription_service,
            mempool_notification_handler,
            storage_synchronizer,
            streaming_service_client,
        }
    }

    /// Starts the state sync driver
    pub async fn start_driver(mut self) {
        let mut progress_check_interval = IntervalStream::new(interval(Duration::from_millis(
            self.driver_configuration.config.progress_check_interval_ms,
        )))
        .fuse();

        loop {
            ::futures::select! {
                notification = self.consensus_notification_handler.select_next_some() => {
                    self.handle_consensus_notification(notification).await;
                }
                notification = self.client_notification_listener.select_next_some() => {
                    self.handle_client_notification(notification);
                },
                _ = progress_check_interval.select_next_some() => {
                    self.check_progress().await;
                }
            }
        }
    }

    /// Handles a notification sent by consensus
    async fn handle_consensus_notification(&mut self, notification: ConsensusNotification) {
        debug!("Received a consensus notification: {:?}", notification);
        // Handle the notification
        let result = match notification {
            ConsensusNotification::NotifyCommit(commit_notification) => {
                self.handle_consensus_commit_notification(commit_notification)
                    .await
            }
            ConsensusNotification::SyncToTarget(sync_notification) => {
                self.handle_consensus_sync_notification(sync_notification)
                    .await
            }
        };

        // Log any errors from notification handling
        if let Err(_error) = result {
            // TODO(joshlind): log the error!
        }
    }

    // TODO(joshlind): can we get consensus to forward the events?
    /// Handles a commit notification sent by consensus
    async fn handle_consensus_commit_notification(
        &mut self,
        commit_notification: ConsensusCommitNotification,
    ) -> Result<(), Error> {
        debug!(
            "Handling the consensus commit notification: {:?}",
            commit_notification
        );

        // Full nodes shouldn't receive commit notifications
        if self.driver_configuration.role == RoleType::FullNode {
            let error = Err(Error::FullNodeConsensusNotification(
                "Received a commit notification!".into(),
            ));
            self.consensus_notification_handler
                .respond_to_commit_notification(commit_notification, error.clone())
                .await;
            return error;
        }

        // Respond to consensus successfully
        let committed_transactions = commit_notification.transactions.clone();
        let reconfiguration_events = commit_notification.reconfiguration_events.clone();
        self.consensus_notification_handler
            .respond_to_commit_notification(commit_notification, Ok(()))
            .await;

        // Notify mempool of the new commit
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        let blockchain_timestamp_usecs = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .timestamp_usecs();
        self.mempool_notification_handler
            .notify_mempool_of_committed_transactions(
                committed_transactions,
                blockchain_timestamp_usecs,
            )
            .await;

        // Publish the reconfiguration notifications
        if let Err(error) = self.event_subscription_service.notify_events(
            latest_storage_summary.latest_synced_version,
            reconfiguration_events,
        ) {
            return Err(Error::EventNotificationError(format!("{:?}", error)));
        }

        Ok(())
    }

    /// Handles a consensus notification to sync to a specified target
    async fn handle_consensus_sync_notification(
        &mut self,
        sync_notification: ConsensusSyncNotification,
    ) -> Result<(), Error> {
        debug!(
            "Handling the consensus sync notification: {:?}",
            sync_notification
        );

        // Full nodes don't support sync requests
        if self.driver_configuration.role == RoleType::FullNode {
            let error = Err(Error::FullNodeConsensusNotification(
                "Received a sync request!".into(),
            ));
            self.consensus_notification_handler
                .respond_to_sync_notification(sync_notification, error.clone())
                .await;
            return error;
        }

        // Consensus should only send sync requests after bootstrapping
        if !self.bootstrapper.is_bootstrapped() {
            let error = Err(Error::BootstrapNotComplete(
                "Consensus shouldn't be running! Unable to process sync notification!".into(),
            ));
            self.consensus_notification_handler
                .respond_to_sync_notification(sync_notification, error.clone())
                .await;
            return error;
        }

        // Initialize a new sync request
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        if let Err(error) = self
            .consensus_notification_handler
            .initialize_sync_request(sync_notification, latest_storage_summary)
            .await
        {
            return Err(error);
        }

        Ok(())
    }

    /// Handles a client notification sent by the driver client
    fn handle_client_notification(
        &mut self,
        notification: DriverNotification,
    ) -> Result<(), Error> {
        debug!("Received a client notification!");

        // Extract the bootstrap notifier channel
        let DriverNotification::NotifyOnceBootstrapped(notifier_channel) = notification;

        // Subscribe the bootstrap notifier channel
        self.bootstrapper
            .subscribe_to_bootstrap_notifications(notifier_channel)
    }

    /// Checks if the node has successfully reached the sync target
    async fn check_sync_request_progress(&mut self) -> Result<(), Error> {
        let latest_storage_summary = self.storage_synchronizer.lock().get_storage_summary()?;
        self.consensus_notification_handler
            .check_sync_request_progress(&latest_storage_summary)
            .await
    }

    /// Returns true iff consensus is currently executing
    fn check_if_consensus_executing(&self) -> bool {
        self.driver_configuration.role == RoleType::Validator
            && self.bootstrapper.is_bootstrapped()
            && !self.consensus_notification_handler.active_sync_request()
    }

    /// Periodically checks that state sync is making progress
    async fn check_progress(&mut self) {
        //debug!("Checking progress of the state sync driver!");

        // If consensus is executing, there's nothing to do.
        if self.check_if_consensus_executing() {
            debug!("Consensus is executing. There's nothing to do.");
            return;
        }

        // Fetch the global data summary and verify we have active peers
        let global_data_summary = self.diem_data_client.get_global_data_summary();
        if global_data_summary.is_empty() {
            debug!("The global data summary is empty! It's likely that we have no active peers.");
            return;
        }

        // Check the progress of any sync requests
        if self.consensus_notification_handler.active_sync_request() {
            debug!("There's an active sync request!");

            // Check if we've hit the sync request target or timed out
            if let Err(error) = self.check_sync_request_progress().await {
                error!(
                    "Error found when checking the sync request progress: {:?}",
                    error
                );
            }
        }

        // Check progress depending on if we're bootstrapping or continuously syncing
        if self.bootstrapper.is_bootstrapped() {
            let sync_request_target = self
                .consensus_notification_handler
                .get_sync_request_target();
            if let Err(error) = self
                .continuous_syncer
                .check_progress(sync_request_target)
                .await
            {
                error!(
                    "Error found when checking the continuous syncer progress: {:?}",
                    error
                );
            }
        } else if let Err(error) = self.bootstrapper.check_progress(&global_data_summary).await {
            error!(
                "Error found when checking the bootstrapper progress: {:?}",
                error
            );
        };
    }
}
