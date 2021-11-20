// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver_client::DriverNotification,
    error::Error,
    notification_handlers::{
        BootstrapNotificationHandler, ConsensusNotificationHandler, MempoolNotificationHandler,
    },
    storage_synchronizer::{StorageStateSummary, StorageSynchronizerInterface},
};
use consensus_notifications::{
    ConsensusCommitNotification, ConsensusNotification, ConsensusSyncNotification,
};
use data_streaming_service::streaming_client::StreamingServiceClient;
use diem_config::config::{RoleType, StateSyncDriverConfig};
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use event_notifications::{EventNotificationSender, EventSubscriptionService};
use futures::StreamExt;
use mempool_notifications::MempoolNotificationSender;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;

// TODO(joshlind): make these configurable!
/// Constants for the driver
const PROGRESS_CHECK_INTERVAL_MS: u64 = 100;

/// The configuration of the state sync driver
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
pub struct StateSyncDriver<M, S> {
    // The component used to notify listeners of successful bootstrapping
    bootstrap_notification_handler: BootstrapNotificationHandler,

    // The handler for notifications from consensus
    consensus_notification_handler: ConsensusNotificationHandler,

    // The configuration for the driver
    driver_configuration: DriverConfiguration,

    // The event subscription service to notify listeners of on-chain events
    event_subscription_service: EventSubscriptionService,

    // The handler for notifications to mempool
    mempool_notification_handler: MempoolNotificationHandler<M>,

    // The storage synchronizer used to update local storage
    storage_synchronizer: S,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,
}

impl<M: MempoolNotificationSender, S: StorageSynchronizerInterface> StateSyncDriver<M, S> {
    pub fn new(
        bootstrap_notification_handler: BootstrapNotificationHandler,
        consensus_notification_handler: ConsensusNotificationHandler,
        driver_configuration: DriverConfiguration,
        event_subscription_service: EventSubscriptionService,
        mempool_notification_handler: MempoolNotificationHandler<M>,
        storage_synchronizer: S,
        streaming_service_client: StreamingServiceClient,
    ) -> Self {
        Self {
            bootstrap_notification_handler,
            consensus_notification_handler,
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
                notification = self.bootstrap_notification_handler.select_next_some() => {
                    self.handle_bootstrap_notification(notification);
                },
                _ = progress_check_interval.select_next_some() => {
                    self.check_progress().await;
                }
            }
        }
    }

    /// Handles a notification sent by consensus
    async fn handle_consensus_notification(&mut self, notification: ConsensusNotification) {
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
        let latest_storage_summary = self.storage_synchronizer.get_storage_summary()?;
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

        // Check if we're now bootstrapped or if we've hit the sync target
        self.check_if_bootstrapped(&latest_storage_summary)?;
        self.check_sync_request_progress(&latest_storage_summary)
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
        if !self.bootstrap_notification_handler.is_bootstrapped() {
            let error = Err(Error::BootstrapNotComplete(
                "Consensus shouldn't be running! Unable to process sync notification!".into(),
            ));
            self.consensus_notification_handler
                .respond_to_sync_notification(sync_notification, error.clone())
                .await;
            return error;
        }

        // Initialize a new sync request
        let latest_storage_summary = self.storage_synchronizer.get_storage_summary()?;
        if let Err(error) = self
            .consensus_notification_handler
            .initialize_sync_request(sync_notification, latest_storage_summary)
            .await
        {
            return Err(error);
        }

        // TODO(joshlind): Start syncing to the sync target!

        Ok(())
    }

    /// Handles a bootstrap notification sent by the driver client
    fn handle_bootstrap_notification(
        &mut self,
        notification: DriverNotification,
    ) -> Result<(), Error> {
        // Extract the bootstrap notifier channel
        let DriverNotification::NotifyOnceBootstrapped(notifier_channel) = notification;

        // Set the bootstrap notifier channel
        self.bootstrap_notification_handler
            .add_bootstrap_notifier(notifier_channel);

        // Check if we're already bootstrapped
        let latest_storage_summary = self.storage_synchronizer.get_storage_summary()?;
        if let Err(_error) = self.check_if_bootstrapped(&latest_storage_summary) {
            // TODO(joshlind): log the error!
        }

        Ok(())
    }

    // TODO(joshlind): improve this so that bootstrapping is not just checking
    // the local state against the waypoint version. Bootstrapping should be
    // based on the network around us.
    /// Checks if the node has successfully bootstrapped
    fn check_if_bootstrapped(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        let latest_committed_version = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .version();
        if self.driver_configuration.waypoint.version() <= latest_committed_version {
            // We've bootstrapped! Notify the handler
            self.bootstrap_notification_handler.notify_bootstrapped()?;
        }

        Ok(())
    }

    /// Checks if the node has successfully reached the sync target
    async fn check_sync_request_progress(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        self.consensus_notification_handler
            .check_sync_request_progress(latest_storage_summary)
            .await
    }

    /// Returns true iff consensus is currently executing
    fn check_if_consensus_executing(&self) -> bool {
        self.driver_configuration.role == RoleType::Validator
            && self.bootstrap_notification_handler.is_bootstrapped()
            && !self.consensus_notification_handler.active_sync_request()
    }

    /// Periodically checks that state sync is making progress
    async fn check_progress(&mut self) -> Result<(), Error> {
        if self.check_if_consensus_executing() {
            return Ok(()); // No need to check progress or issue any requests
        }

        // Check if the sync request has timed out (i.e., if we aren't committing fast enough)
        self.consensus_notification_handler
            .check_sync_request_timeout()
            .await?;

        // TODO(joshlind): check the data streaming service for messages and
        // identify how to make progress going forward.

        Ok(())
    }
}
