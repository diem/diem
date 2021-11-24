// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver_client::{ClientNotificationListener, DriverNotification},
    error::Error,
    storage_synchronizer::StorageStateSummary,
};
use consensus_notifications::{
    ConsensusCommitNotification, ConsensusNotification, ConsensusNotificationListener,
    ConsensusSyncNotification,
};
use diem_types::transaction::Transaction;
use futures::{
    channel::{mpsc, oneshot},
    stream::FusedStream,
    Stream,
};
use mempool_notifications::MempoolNotificationSender;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, SystemTime},
};

// TODO(joshlind): make these configurable!
const CONSENSUS_SYNC_REQUEST_TIMEOUT_MS: u64 = 60000; // 1 minute
const MEMPOOL_COMMIT_ACK_TIMEOUT_MS: u64 = 5000; // 5 seconds

/// A simple handler for sending bootstrap notifications to client listeners
pub struct BootstrapNotificationHandler {
    // If the node has has already been bootstrapped
    bootstrapped: bool,

    // The listener for bootstrap requests from clients
    client_listener: ClientNotificationListener,

    // The channel used to notify a listener of successful bootstrapping
    notifier_channel: Option<oneshot::Sender<Result<(), Error>>>,
}

impl BootstrapNotificationHandler {
    pub fn new(client_notifications: mpsc::UnboundedReceiver<DriverNotification>) -> Self {
        let client_listener = ClientNotificationListener::new(client_notifications);

        Self {
            bootstrapped: false,
            client_listener,
            notifier_channel: None,
        }
    }

    /// Notifies the channel once the node has completed bootstrapping
    pub fn add_bootstrap_notifier(&mut self, notifier_channel: oneshot::Sender<Result<(), Error>>) {
        self.notifier_channel = Some(notifier_channel);
    }

    /// Returns true iff the node has already completed bootsrapping
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Notifies any listeners that the node has completed bootstrapping
    pub fn notify_bootstrapped(&mut self) -> Result<(), Error> {
        self.bootstrapped = true;
        if let Some(notifier_channel) = self.notifier_channel.take() {
            let notification_message = Ok(());
            if let Err(error) = notifier_channel.send(notification_message) {
                return Err(Error::CallbackSendFailed(format!(
                    "Bootstrap notification error: {:?}",
                    error
                )));
            }
        }
        Ok(())
    }
}

impl Stream for BootstrapNotificationHandler {
    type Item = DriverNotification;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().client_listener).poll_next(cx)
    }
}

impl FusedStream for BootstrapNotificationHandler {
    fn is_terminated(&self) -> bool {
        self.client_listener.is_terminated()
    }
}

/// A simple handler for consensus notifications
pub struct ConsensusNotificationHandler {
    // The listener for notifications from consensus
    consensus_listener: ConsensusNotificationListener,

    // The latest consensus sync request that has been received
    sync_request: Option<ConsensusSyncRequest>,
}

impl ConsensusNotificationHandler {
    pub fn new(consensus_listener: ConsensusNotificationListener) -> Self {
        Self {
            consensus_listener,
            sync_request: None,
        }
    }

    /// Initializes the sync request received from consensus
    pub async fn initialize_sync_request(
        &mut self,
        sync_notification: ConsensusSyncNotification,
        latest_storage_summary: StorageStateSummary,
    ) -> Result<(), Error> {
        // Get the latest committed version and the target sync version
        let sync_target_version = sync_notification.target.ledger_info().version();
        let latest_committed_version = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .version();

        // If the target version is old, return an error to consensus (something is wrong!)
        if sync_target_version < latest_committed_version {
            let error = Err(Error::OldSyncRequest(
                sync_target_version,
                latest_committed_version,
            ));
            self.respond_to_sync_notification(sync_notification, error.clone())
                .await;
            return error;
        }

        // If we're now at the target, return successfully
        if sync_target_version == latest_committed_version {
            let result = Ok(());
            self.respond_to_sync_notification(sync_notification, result.clone())
                .await;
            return result;
        }

        // Save the request so we can notify consensus once we've hit the target
        let consensus_sync_request = ConsensusSyncRequest {
            consensus_sync_notification: sync_notification,
            last_commit_timestamp: SystemTime::now(),
        };
        self.sync_request = Some(consensus_sync_request);

        Ok(())
    }

    /// Returns true iff there is a sync request currently blocking consensus
    pub fn active_sync_request(&self) -> bool {
        self.sync_request.is_some()
    }

    /// Checks to see if the sync request has been successfully fulfilled
    pub async fn check_sync_request_progress(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        if let Some(sync_request) = self.sync_request.as_mut() {
            // Fetch the latest committed version and the target sync version
            let sync_target_version = sync_request
                .consensus_sync_notification
                .target
                .ledger_info()
                .version();
            let latest_committed_version = latest_storage_summary
                .latest_ledger_info
                .ledger_info()
                .version();

            // Check if we've synced beyond the target
            if latest_committed_version > sync_target_version {
                return Err(Error::SyncedBeyondTarget(
                    latest_committed_version,
                    sync_target_version,
                ));
            }

            // Check if we've hit the target
            if latest_committed_version == sync_target_version {
                if let Some(sync_request) = self.sync_request.take() {
                    self.respond_to_sync_notification(
                        sync_request.consensus_sync_notification,
                        Ok(()),
                    )
                    .await;
                }
                return Ok(());
            }

            // Check if the sync deadline has been exceeded (timed out since the last commit)
            let max_time_between_commits = Duration::from_millis(CONSENSUS_SYNC_REQUEST_TIMEOUT_MS);
            let next_commit_deadline = sync_request
                .last_commit_timestamp
                .checked_add(max_time_between_commits)
                .ok_or_else(|| {
                    Error::IntegerOverflow("The new commit deadline has overflown!".into())
                })?;
            if SystemTime::now()
                .duration_since(next_commit_deadline)
                .is_ok()
            {
                // TODO(joshlind): log this!

                // Remove the sync request and notify consensus that the request timed out
                if let Some(sync_request) = self.sync_request.take() {
                    self.respond_to_sync_notification(
                        sync_request.consensus_sync_notification,
                        Err(Error::UnexpectedError(
                            "Sync request timed out! Hit the max commit time!".into(),
                        )),
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    /// Responds to consensus for a sync notification using the specified result
    pub async fn respond_to_sync_notification(
        &mut self,
        sync_notification: ConsensusSyncNotification,
        result: Result<(), Error>,
    ) {
        // Wrap the result in an error that consensus can process
        let message = result.map_err(|error| {
            consensus_notifications::Error::UnexpectedErrorEncountered(format!("{:?}", error))
        });

        // Send the result
        self.consensus_listener
            .respond_to_sync_notification(sync_notification, message)
            .await
            .map_err(|error| {
                // TODO(joshlind): log me!
                let _ = Error::CallbackSendFailed(format!(
                    "Consensus sync request response error: {:?}",
                    error
                ));
            });
    }

    /// Responds successfully to consensus for a commit notification
    pub async fn respond_to_commit_notification(
        &mut self,
        commit_notification: ConsensusCommitNotification,
        result: Result<(), Error>,
    ) {
        // Wrap the result in an error that consensus can process
        let message = result.map_err(|error| {
            consensus_notifications::Error::UnexpectedErrorEncountered(format!("{:?}", error))
        });

        // Send the result
        self.consensus_listener
            .respond_to_commit_notification(commit_notification, message)
            .await
            .map_err(|error| {
                // TODO(joshlind): log me!
                Error::CallbackSendFailed(format!("Consensus commit response error: {:?}", error))
            });
    }
}

impl Stream for ConsensusNotificationHandler {
    type Item = ConsensusNotification;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().consensus_listener).poll_next(cx)
    }
}

impl FusedStream for ConsensusNotificationHandler {
    fn is_terminated(&self) -> bool {
        self.consensus_listener.is_terminated()
    }
}

/// A consensus sync request for a specified target ledger info
pub struct ConsensusSyncRequest {
    pub consensus_sync_notification: ConsensusSyncNotification,
    pub last_commit_timestamp: SystemTime,
}

/// A simple handler for sending notifications to mempool
pub struct MempoolNotificationHandler<M> {
    mempool_notification_sender: M,
}

impl<M: MempoolNotificationSender> MempoolNotificationHandler<M> {
    pub fn new(mempool_notification_sender: M) -> Self {
        Self {
            mempool_notification_sender,
        }
    }
    /// Notifies mempool that transactions have been committed.
    pub async fn notify_mempool_of_committed_transactions(
        &mut self,
        committed_transactions: Vec<Transaction>,
        block_timestamp_usecs: u64,
    ) -> Result<(), Error> {
        let result = self
            .mempool_notification_sender
            .notify_new_commit(
                committed_transactions,
                block_timestamp_usecs,
                MEMPOOL_COMMIT_ACK_TIMEOUT_MS,
            )
            .await;

        if let Err(error) = result {
            // TODO(joshlind): log this!
            Err(Error::NotifyMempoolError(format!("{:?}", error)))
        } else {
            Ok(())
        }
    }
}
