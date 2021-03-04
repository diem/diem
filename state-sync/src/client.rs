// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, error::Error, shared_components::SyncState};
use diem_mempool::CommitResponse;
use diem_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};
use std::time::{Duration, SystemTime};
use tokio::time::timeout;

/// A sync request for a specified target ledger info.
pub struct SyncRequest {
    pub callback: oneshot::Sender<Result<(), Error>>,
    pub last_commit_timestamp: SystemTime,
    pub target: LedgerInfoWithSignatures,
}

/// A commit notification to notify state sync of new commits.
pub struct CommitNotification {
    pub callback: oneshot::Sender<Result<CommitResponse, Error>>,
    pub committed_transactions: Vec<Transaction>,
    pub reconfiguration_events: Vec<ContractEvent>,
}

/// Messages used by the StateSyncClient for communication with the StateSyncCoordinator.
pub enum CoordinatorMessage {
    SyncRequest(Box<SyncRequest>), // Initiate a new sync request for a given target.
    CommitNotification(Box<CommitNotification>), // Notify state sync about committed transactions.
    GetSyncState(oneshot::Sender<SyncState>), // Return the local sync state.
    WaitForInitialization(oneshot::Sender<Result<(), Error>>), // Wait until state sync is initialized to the waypoint.
}

/// A client used for communicating with a StateSyncCoordinator.
pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,

    /// Timeout for the StateSyncClient to receive an ack when executing commit().
    commit_timeout_ms: u64,
}

impl StateSyncClient {
    pub fn new(
        coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
        commit_timeout_ms: u64,
    ) -> Self {
        Self {
            coordinator_sender,
            commit_timeout_ms,
        }
    }

    /// Sync node's state to target ledger info (LI).
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    pub fn sync_to(
        &self,
        target: LedgerInfoWithSignatures,
    ) -> impl Future<Output = Result<(), Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        let request = SyncRequest {
            callback: cb_sender,
            target,
            last_commit_timestamp: SystemTime::now(),
        };

        async move {
            sender
                .send(CoordinatorMessage::SyncRequest(Box::new(request)))
                .await?;
            cb_receiver.await?
        }
    }

    /// Notifies state sync about newly committed transactions.
    pub fn commit(
        &self,
        committed_txns: Vec<Transaction>,
        reconfig_events: Vec<ContractEvent>,
    ) -> impl Future<Output = Result<(), Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        let commit_timeout_ms = self.commit_timeout_ms;
        let notification = CommitNotification {
            callback: cb_sender,
            committed_transactions: committed_txns,
            reconfiguration_events: reconfig_events,
        };

        async move {
            sender
                .send(CoordinatorMessage::CommitNotification(Box::new(
                    notification,
                )))
                .await?;

            match timeout(Duration::from_millis(commit_timeout_ms), cb_receiver).await {
                Err(_) => {
                    counters::COMMIT_FLOW_FAIL
                        .with_label_values(&[counters::STATE_SYNC_LABEL])
                        .inc();
                    Err(Error::UnexpectedError(
                        "State sync client timeout: failed to receive commit() ack in time!".into(),
                    ))
                }
                Ok(response) => {
                    let response = response??; // Unwrap the futures result to get the body
                    if response.success {
                        Ok(())
                    } else {
                        Err(Error::UnexpectedError(format!(
                            "State sync client failed: commit() returned an error: {:?}",
                            response.error_message
                        )))
                    }
                }
            }
        }
    }

    /// Returns information about the state sync internal state. This should only
    /// be used by tests.
    // TODO(joshlind): remove this once unit tests are added!
    pub fn get_state(&self) -> impl Future<Output = Result<SyncState, Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::GetSyncState(cb_sender))
                .await?;
            cb_receiver.await.map_err(|error| error.into())
        }
    }

    /// Waits until state sync is caught up with the waypoint specified in the local config.
    pub fn wait_until_initialized(&self) -> impl Future<Output = Result<(), Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::WaitForInitialization(cb_sender))
                .await?;
            cb_receiver.await?
        }
    }
}
