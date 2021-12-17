// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage_synchronizer::StorageStateSummary};
use consensus_notifications::{
    ConsensusCommitNotification, ConsensusNotification, ConsensusNotificationListener,
    ConsensusSyncNotification,
};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use event_notifications::{EventNotificationSender, EventSubscriptionService};
use futures::{channel::mpsc, stream::FusedStream, Stream};
use mempool_notifications::MempoolNotificationSender;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, SystemTime},
};

// TODO(joshlind): make these configurable!
const CONSENSUS_SYNC_REQUEST_TIMEOUT_MS: u64 = 60000; // 1 minute
const MEMPOOL_COMMIT_ACK_TIMEOUT_MS: u64 = 5000; // 5 seconds

/// A notification for new transactions and events that have been committed to
/// storage.
pub struct CommitNotification {
    pub events: Vec<ContractEvent>,
    pub transactions: Vec<Transaction>,
}

impl CommitNotification {
    pub fn new(events: Vec<ContractEvent>, transactions: Vec<Transaction>) -> Self {
        Self {
            events,
            transactions,
        }
    }

    /// Handles the commit notification by notifying mempool and the event
    /// subscription service.
    pub async fn handle_commit_notification<M: MempoolNotificationSender>(
        &self,
        latest_storage_summary: &StorageStateSummary,
        mut mempool_notification_handler: MempoolNotificationHandler<M>,
        event_subscription_service: Arc<Mutex<EventSubscriptionService>>,
    ) -> Result<(), Error> {
        let latest_synced_version = latest_storage_summary.latest_synced_version;

        // Notify mempool of the committed transactions
        debug!(
            "Notifying mempool of transactions at version: {:?}",
            latest_synced_version
        );
        let blockchain_timestamp_usecs = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .timestamp_usecs();
        mempool_notification_handler
            .notify_mempool_of_committed_transactions(
                self.transactions.clone(),
                blockchain_timestamp_usecs,
            )
            .await?;

        // Notify the event subscription service of the events
        debug!(
            "Notifying the event subscription service of events at version: {:?}",
            latest_synced_version
        );
        event_subscription_service
            .lock()
            .notify_events(latest_synced_version, self.events.clone())
            .map_err(|error| error.into())
    }
}

/// A simple wrapper for a commit notification listener
pub struct CommitNotificationListener {
    // The listener for commit notifications
    commit_notification_listener: mpsc::UnboundedReceiver<CommitNotification>,
}

impl CommitNotificationListener {
    pub fn new() -> (mpsc::UnboundedSender<CommitNotification>, Self) {
        // Create a channel to send and receive commit notifications
        let (commit_notification_sender, commit_notification_listener) = mpsc::unbounded();

        // Create and return the sender and listener
        let commit_notification_listener = Self {
            commit_notification_listener,
        };
        (commit_notification_sender, commit_notification_listener)
    }
}

impl Stream for CommitNotificationListener {
    type Item = CommitNotification;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().commit_notification_listener).poll_next(cx)
    }
}

impl FusedStream for CommitNotificationListener {
    fn is_terminated(&self) -> bool {
        self.commit_notification_listener.is_terminated()
    }
}

/// A consensus sync request for a specified target ledger info
pub struct ConsensusSyncRequest {
    consensus_sync_notification: ConsensusSyncNotification,
    last_commit_timestamp: SystemTime,
}

impl ConsensusSyncRequest {
    pub fn new(consensus_sync_notification: ConsensusSyncNotification) -> Self {
        Self {
            consensus_sync_notification,
            last_commit_timestamp: SystemTime::now(),
        }
    }

    pub fn update_last_commit_timestamp(&mut self) {
        self.last_commit_timestamp = SystemTime::now();
    }

    pub fn get_last_commit_timestamp(&self) -> SystemTime {
        self.last_commit_timestamp
    }

    pub fn get_sync_target(&self) -> LedgerInfoWithSignatures {
        self.consensus_sync_notification.target.clone()
    }
}

/// A simple handler for consensus notifications
pub struct ConsensusNotificationHandler {
    // The listener for notifications from consensus
    consensus_listener: ConsensusNotificationListener,

    // The latest consensus sync request that has been received
    consensus_sync_request: Arc<Mutex<Option<ConsensusSyncRequest>>>,
}

impl ConsensusNotificationHandler {
    pub fn new(consensus_listener: ConsensusNotificationListener) -> Self {
        Self {
            consensus_listener,
            consensus_sync_request: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns true iff there is a sync request currently blocking consensus
    pub fn active_sync_request(&self) -> bool {
        self.consensus_sync_request.lock().is_some()
    }

    /// Returns the active sync request that consensus is waiting on
    pub fn get_consensus_sync_request(&self) -> Arc<Mutex<Option<ConsensusSyncRequest>>> {
        self.consensus_sync_request.clone()
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
                .await?;
            return error;
        }

        // If we're now at the target, return successfully
        if sync_target_version == latest_committed_version {
            info!("We're already at the requested sync target version! Returning early.");
            let result = Ok(());
            self.respond_to_sync_notification(sync_notification, result.clone())
                .await?;
            return result;
        }

        // Save the request so we can notify consensus once we've hit the target
        let consensus_sync_request = ConsensusSyncRequest::new(sync_notification);
        self.consensus_sync_request = Arc::new(Mutex::new(Some(consensus_sync_request)));

        Ok(())
    }

    /// Checks to see if the sync request has been successfully fulfilled
    pub async fn check_sync_request_progress(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        // Fetch the sync target version
        let consensus_sync_request = self.get_consensus_sync_request();
        let sync_target_version = consensus_sync_request.lock().as_ref().map(|sync_request| {
            sync_request
                .consensus_sync_notification
                .target
                .ledger_info()
                .version()
        });

        // Compare our local state to the target version
        if let Some(sync_target_version) = sync_target_version {
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
                let consensus_sync_request = self.get_consensus_sync_request().lock().take();
                if let Some(consensus_sync_request) = consensus_sync_request {
                    self.respond_to_sync_notification(
                        consensus_sync_request.consensus_sync_notification,
                        Ok(()),
                    )
                    .await?;
                }
                return Ok(());
            }

            // Check if the commit deadline has been exceeded (timed out since the last commit)
            let max_time_between_commits = Duration::from_millis(CONSENSUS_SYNC_REQUEST_TIMEOUT_MS);
            let last_commit_timestamp = self
                .get_consensus_sync_request()
                .lock()
                .as_ref()
                .expect("The sync request should exist!")
                .get_last_commit_timestamp();
            let next_commit_deadline = last_commit_timestamp
                .checked_add(max_time_between_commits)
                .ok_or_else(|| {
                    Error::IntegerOverflow("The new commit deadline has overflown!".into())
                })?;
            if SystemTime::now()
                .duration_since(next_commit_deadline)
                .is_ok()
            {
                // Remove the sync request and notify consensus that the request timed out
                let error = Error::UnexpectedError(format!(
                    "Sync request timed out! Hit the max time between commits: {:?}",
                    max_time_between_commits
                ));
                let consensus_sync_request = self.get_consensus_sync_request().lock().take();
                if let Some(consensus_sync_request) = consensus_sync_request {
                    self.respond_to_sync_notification(
                        consensus_sync_request.consensus_sync_notification,
                        Err(error.clone()),
                    )
                    .await?;
                }
                return Err(error);
            }
        }

        Ok(())
    }

    /// Responds to consensus for a sync notification using the specified result
    pub async fn respond_to_sync_notification(
        &mut self,
        sync_notification: ConsensusSyncNotification,
        result: Result<(), Error>,
    ) -> Result<(), Error> {
        // Wrap the result in an error that consensus can process
        let message = result.map_err(|error| {
            consensus_notifications::Error::UnexpectedErrorEncountered(format!("{:?}", error))
        });

        // Send the result
        self.consensus_listener
            .respond_to_sync_notification(sync_notification, message)
            .await
            .map_err(|error| {
                Error::CallbackSendFailed(format!(
                    "Consensus sync request response error: {:?}",
                    error
                ))
            })
    }

    /// Responds successfully to consensus for a commit notification
    pub async fn respond_to_commit_notification(
        &mut self,
        commit_notification: ConsensusCommitNotification,
        result: Result<(), Error>,
    ) -> Result<(), Error> {
        // Wrap the result in an error that consensus can process
        let message = result.map_err(|error| {
            consensus_notifications::Error::UnexpectedErrorEncountered(format!("{:?}", error))
        });

        // Send the result
        self.consensus_listener
            .respond_to_commit_notification(commit_notification, message)
            .await
            .map_err(|error| {
                Error::CallbackSendFailed(format!("Consensus commit response error: {:?}", error))
            })
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

/// A simple handler for sending notifications to mempool
#[derive(Clone)]
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
