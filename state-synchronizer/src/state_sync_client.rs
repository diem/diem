// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{format_err, Result};
use futures::channel::{mpsc, oneshot};
use futures::{future::Future, SinkExt};
use libra_mempool::CommitResponse;
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use std::time::{Duration, SystemTime};
use tokio::time::timeout;

/// struct created when a sync is requested
/// in order to get back to the caller with the response once the sync is done
pub(crate) struct SyncRequest {
    // The Result value returned to the caller is Error in case the StateSynchronizer failed to
    // reach the target (the LI in the storage remains unchanged as if nothing happened).
    pub callback: oneshot::Sender<Result<()>>,
    pub target: LedgerInfoWithSignatures,
    pub last_progress_tst: SystemTime,
}

/// message used by StateSyncClient for communication with Coordinator
pub(crate) enum CoordinatorMessage {
    // used to initiate new sync
    Request(Box<SyncRequest>),
    // used to notify about new txn commit
    Commit(
        // committed transactions
        Vec<Transaction>,
        // reconfiguration events
        Vec<ContractEvent>,
        // callback for recipient to send response back to this sender
        oneshot::Sender<Result<CommitResponse>>,
    ),
    GetState(oneshot::Sender<SynchronizerState>),
    // Receive a notification via a given channel when coordinator is initialized.
    WaitInitialize(oneshot::Sender<Result<()>>),
}

/// A StateSyncClient can be used to send asynchronous requests to state-sync
pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncClient {
    /// Create a new StateSyncClient from a coordinator sender
    pub(crate) fn new(coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>) -> Self {
        Self { coordinator_sender }
    }

    /// Sync validator's state to target.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    /// It is up to state synchronizer to decide about the specific criteria for the failure
    /// (e.g., lack of progress with all of the peer validators).
    pub fn sync_to(&self, target: LedgerInfoWithSignatures) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        let (callback, cb_receiver) = oneshot::channel();
        let request = SyncRequest {
            callback,
            target,
            last_progress_tst: SystemTime::now(),
        };
        async move {
            sender
                .send(CoordinatorMessage::Request(Box::new(request)))
                .await?;
            cb_receiver.await?
        }
    }

    /// Notifies state synchronizer about new version
    pub fn commit(
        &self,
        // *successfully* committed transactions
        committed_txns: Vec<Transaction>,
        reconfig_events: Vec<ContractEvent>,
    ) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        async move {
            let (callback, callback_rcv) = oneshot::channel();
            sender
                .send(CoordinatorMessage::Commit(
                    committed_txns,
                    reconfig_events,
                    callback,
                ))
                .await?;

            match timeout(Duration::from_secs(5), callback_rcv).await {
              Err(_) => {
                  Err(format_err!("[state sync client] failed to receive commit ACK from state synchronizer on time"))
              }
              Ok(resp) => {
                  let CommitResponse { msg } = resp??;
                  if msg != "" {
                      Err(format_err!("[state sync client] commit failed: {:?}", msg))
                  } else {
                      Ok(())
                  }
              }
          }
        }
    }

    /// Returns information about StateSynchronizer internal state
    pub fn get_state(&self) -> impl Future<Output = Result<SynchronizerState>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender.send(CoordinatorMessage::GetState(cb_sender)).await?;
            let info = cb_receiver.await?;
            Ok(info)
        }
    }
}
