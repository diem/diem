// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use std::time::Duration;

use async_trait::async_trait;
use diem_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
};
use futures::{
    channel::{mpsc, oneshot},
    stream::FusedStream,
    SinkExt, Stream,
};
use serde::{Deserialize, Serialize};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::time::timeout;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Notification failed: {0}")]
    NotificationError(String),
    #[error("Hit the timeout waiting for state sync to respond to the notification!")]
    TimeoutWaitingForStateSync,
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
}

/// The interface between state sync and consensus, allowing consensus to send
/// synchronization notifications to state sync.
#[async_trait]
pub trait ConsensusNotificationSender: Send + Sync {
    /// Notify state sync of newly committed transactions and reconfiguration events.
    async fn notify_new_commit(
        &self,
        transactions: Vec<Transaction>,
        reconfiguration_events: Vec<ContractEvent>,
    ) -> Result<(), Error>;

    /// Notify state sync to synchronize storage to the specified target.
    async fn sync_to_target(&self, target: LedgerInfoWithSignatures) -> Result<(), Error>;
}

/// This method returns a (ConsensusNotifier, ConsensusNotificationListener) pair that can be used
/// to allow consensus and state sync to communicate.
///
/// Note: consensus should take the notifier and state sync should take the listener.
pub fn new_consensus_notifier_listener_pair(
    timeout_ms: u64,
) -> (ConsensusNotifier, ConsensusNotificationListener) {
    let (notification_sender, notification_receiver) = mpsc::unbounded();

    let consensus_notifier = ConsensusNotifier::new(notification_sender, timeout_ms);
    let consensus_listener = ConsensusNotificationListener::new(notification_receiver);

    (consensus_notifier, consensus_listener)
}

/// The consensus component responsible for sending notifications and requests to
/// state sync.
///
/// Note: When a ConsensusNotifier instance is created, state sync must take and
/// listen to the receiver in the corresponding ConsensusNotificationListener.
#[derive(Debug)]
pub struct ConsensusNotifier {
    notification_sender: mpsc::UnboundedSender<ConsensusNotification>,

    /// Timeout for state sync to respond to consensus when handling a commit
    /// notification.
    timeout_ms: u64,
}

impl ConsensusNotifier {
    fn new(
        notification_sender: mpsc::UnboundedSender<ConsensusNotification>,
        timeout_ms: u64,
    ) -> Self {
        ConsensusNotifier {
            notification_sender,
            timeout_ms,
        }
    }
}

#[async_trait]
impl ConsensusNotificationSender for ConsensusNotifier {
    async fn notify_new_commit(
        &self,
        transactions: Vec<Transaction>,
        reconfiguration_events: Vec<ContractEvent>,
    ) -> Result<(), Error> {
        // Only send a notification if transactions have been committed
        if transactions.is_empty() {
            return Ok(());
        }

        // Construct a oneshot channel to receive a state sync response
        let (callback, callback_receiver) = oneshot::channel();
        let commit_notification =
            ConsensusNotification::NotifyCommit(ConsensusCommitNotification {
                transactions,
                reconfiguration_events,
                callback,
            });

        // Send the notification to state sync
        if let Err(error) = self
            .notification_sender
            .clone()
            .send(commit_notification)
            .await
        {
            return Err(Error::NotificationError(format!(
                "Failed to notify state sync of committed transactions! Error: {:?}",
                error
            )));
        }

        // Handle any responses or a timeout
        if let Ok(response) =
            timeout(Duration::from_millis(self.timeout_ms), callback_receiver).await
        {
            match response {
                Ok(consensus_notification_response) => consensus_notification_response.result,
                Err(error) => Err(Error::UnexpectedErrorEncountered(format!("{:?}", error))),
            }
        } else {
            Err(Error::TimeoutWaitingForStateSync)
        }
    }

    async fn sync_to_target(&self, target: LedgerInfoWithSignatures) -> Result<(), Error> {
        // Construct a oneshot channel to receive a state sync response
        let (callback, callback_receiver) = oneshot::channel();
        let sync_notification =
            ConsensusNotification::SyncToTarget(ConsensusSyncNotification { target, callback });

        // Send the notification to state sync
        if let Err(error) = self
            .notification_sender
            .clone()
            .send(sync_notification)
            .await
        {
            return Err(Error::NotificationError(format!(
                "Failed to notify state sync of sync target! Error: {:?}",
                error
            )));
        }

        // Process the response
        match callback_receiver.await {
            Ok(response) => response.result,
            Err(error) => Err(Error::UnexpectedErrorEncountered(format!("{:?}", error))),
        }
    }
}

/// The state sync component responsible for handling consensus requests and
/// notifications.
#[derive(Debug)]
pub struct ConsensusNotificationListener {
    notification_receiver: mpsc::UnboundedReceiver<ConsensusNotification>,
}

impl ConsensusNotificationListener {
    fn new(notification_receiver: mpsc::UnboundedReceiver<ConsensusNotification>) -> Self {
        ConsensusNotificationListener {
            notification_receiver,
        }
    }

    /// Respond to the commit notification previously sent by consensus.
    pub async fn respond_to_commit_notification(
        &mut self,
        consensus_commit_notification: ConsensusCommitNotification,
        result: Result<(), Error>,
    ) -> Result<(), Error> {
        consensus_commit_notification
            .callback
            .send(ConsensusNotificationResponse { result })
            .map_err(|error| Error::UnexpectedErrorEncountered(format!("{:?}", error)))
    }

    /// Respond to the sync notification previously sent by consensus.
    pub async fn respond_to_sync_notification(
        &mut self,
        consensus_sync_notification: ConsensusSyncNotification,
        result: Result<(), Error>,
    ) -> Result<(), Error> {
        consensus_sync_notification
            .callback
            .send(ConsensusNotificationResponse { result })
            .map_err(|error| Error::UnexpectedErrorEncountered(format!("{:?}", error)))
    }
}

impl Stream for ConsensusNotificationListener {
    type Item = ConsensusNotification;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().notification_receiver).poll_next(cx)
    }
}

impl FusedStream for ConsensusNotificationListener {
    fn is_terminated(&self) -> bool {
        self.notification_receiver.is_terminated()
    }
}

#[derive(Debug)]
pub enum ConsensusNotification {
    NotifyCommit(ConsensusCommitNotification),
    SyncToTarget(ConsensusSyncNotification),
}

/// A commit notification to notify state sync of new commits.
#[derive(Debug)]
pub struct ConsensusCommitNotification {
    pub transactions: Vec<Transaction>,
    pub reconfiguration_events: Vec<ContractEvent>,
    pub(crate) callback: oneshot::Sender<ConsensusNotificationResponse>,
}

impl ConsensusCommitNotification {
    pub fn new(
        transactions: Vec<Transaction>,
        reconfiguration_events: Vec<ContractEvent>,
    ) -> (Self, oneshot::Receiver<ConsensusNotificationResponse>) {
        let (callback, callback_receiver) = oneshot::channel();
        let commit_notification = ConsensusCommitNotification {
            transactions,
            reconfiguration_events,
            callback,
        };

        (commit_notification, callback_receiver)
    }
}

/// The result returned by state sync for a consensus notification.
#[derive(Debug)]
pub struct ConsensusNotificationResponse {
    pub result: Result<(), Error>,
}

/// A commit notification to notify state sync to sync to the specified target.
#[derive(Debug)]
pub struct ConsensusSyncNotification {
    pub target: LedgerInfoWithSignatures,
    pub(crate) callback: oneshot::Sender<ConsensusNotificationResponse>,
}

impl ConsensusSyncNotification {
    pub fn new(
        target: LedgerInfoWithSignatures,
    ) -> (Self, oneshot::Receiver<ConsensusNotificationResponse>) {
        let (callback, callback_receiver) = oneshot::channel();
        let sync_notification = ConsensusSyncNotification { target, callback };

        (sync_notification, callback_receiver)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ConsensusNotification, ConsensusNotificationSender, Error};
    use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, SigningKey, Uniform};
    use diem_types::{
        account_address::AccountAddress,
        block_info::BlockInfo,
        chain_id::ChainId,
        contract_event::ContractEvent,
        event::EventKey,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        transaction::{RawTransaction, Script, SignedTransaction, Transaction, TransactionPayload},
    };
    use futures::executor::block_on;
    use move_core_types::language_storage::TypeTag;
    use std::{collections::BTreeMap, time::Duration};
    use tokio::runtime::{Builder, Runtime};

    const CONSENSUS_NOTIFICATION_TIMEOUT: u64 = 1000;

    #[test]
    fn test_commit_state_sync_not_listening() {
        // Create runtime and consensus notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (consensus_notifier, mut consensus_listener) =
            crate::new_consensus_notifier_listener_pair(CONSENSUS_NOTIFICATION_TIMEOUT);

        // Send a notification and expect a timeout (no listener)
        let notify_result =
            block_on(consensus_notifier.notify_new_commit(vec![create_user_transaction()], vec![]));
        assert!(matches!(
            notify_result,
            Err(Error::TimeoutWaitingForStateSync)
        ));

        // Drop the receiver and try again
        consensus_listener.notification_receiver.close();
        let notify_result =
            block_on(consensus_notifier.notify_new_commit(vec![create_user_transaction()], vec![]));
        assert!(matches!(notify_result, Err(Error::NotificationError(_))));
    }

    #[test]
    fn test_commit_no_transactions() {
        // Create runtime and consensus notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (consensus_notifier, _consensus_listener) =
            crate::new_consensus_notifier_listener_pair(CONSENSUS_NOTIFICATION_TIMEOUT);

        // Send a notification
        let notify_result = block_on(consensus_notifier.notify_new_commit(vec![], vec![]));
        assert!(notify_result.is_ok());
    }

    #[test]
    fn test_consensus_notification_arrives() {
        // Create runtime and consensus notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (consensus_notifier, mut consensus_listener) =
            crate::new_consensus_notifier_listener_pair(CONSENSUS_NOTIFICATION_TIMEOUT);

        // Send a commit notification
        let transactions = vec![create_user_transaction()];
        let reconfiguration_events = vec![create_contract_event()];
        let _ = block_on(
            consensus_notifier
                .notify_new_commit(transactions.clone(), reconfiguration_events.clone()),
        );

        // Verify the notification arrives at the receiver
        match consensus_listener.notification_receiver.try_next() {
            Ok(Some(consensus_notification)) => match consensus_notification {
                ConsensusNotification::NotifyCommit(commit_notification) => {
                    assert_eq!(transactions, commit_notification.transactions);
                    assert_eq!(
                        reconfiguration_events,
                        commit_notification.reconfiguration_events
                    );
                }
                result => panic!(
                    "Expected consensus commit notification but got: {:?}",
                    result
                ),
            },
            result => panic!("Expected consensus notification but got: {:?}", result),
        };

        // Send a sync notification
        let _thread = std::thread::spawn(move || {
            let _result = block_on(consensus_notifier.sync_to_target(create_ledger_info()));
        });

        // Give the thread enough time to spawn and send the notification
        std::thread::sleep(Duration::from_millis(1000));

        // Verify the notification arrives at the receiver
        match consensus_listener.notification_receiver.try_next() {
            Ok(Some(consensus_notification)) => match consensus_notification {
                ConsensusNotification::SyncToTarget(sync_notification) => {
                    assert_eq!(create_ledger_info(), sync_notification.target);
                }
                result => panic!("Expected consensus sync notification but got: {:?}", result),
            },
            result => panic!("Expected consensus notification but got: {:?}", result),
        };
    }

    #[test]
    fn test_consensus_notification_responses() {
        // Create runtime and consensus notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (consensus_notifier, mut consensus_listener) =
            crate::new_consensus_notifier_listener_pair(CONSENSUS_NOTIFICATION_TIMEOUT);

        // Spawn a new thread to handle any messages on the receiver
        let _handler = std::thread::spawn(move || loop {
            match consensus_listener.notification_receiver.try_next() {
                Ok(Some(ConsensusNotification::NotifyCommit(commit_notification))) => {
                    let _result = block_on(
                        consensus_listener
                            .respond_to_commit_notification(commit_notification, Ok(())),
                    );
                }
                Ok(Some(ConsensusNotification::SyncToTarget(sync_notification))) => {
                    let _result = block_on(consensus_listener.respond_to_sync_notification(
                        sync_notification,
                        Err(Error::UnexpectedErrorEncountered("Oops?".into())),
                    ));
                }
                _ => { /* Do nothing */ }
            }
        });

        // Send a commit notification and verify a successful response
        let notify_result =
            block_on(consensus_notifier.notify_new_commit(vec![create_user_transaction()], vec![]));
        assert!(notify_result.is_ok());

        // Send a sync notification and very an error response
        let notify_result = block_on(consensus_notifier.sync_to_target(create_ledger_info()));
        assert!(notify_result.is_err());
    }

    fn create_user_transaction() -> Transaction {
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let public_key = private_key.public_key();

        let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
        let raw_transaction = RawTransaction::new(
            AccountAddress::random(),
            0,
            transaction_payload,
            0,
            0,
            "".into(),
            0,
            ChainId::new(10),
        );
        let signed_transaction = SignedTransaction::new(
            raw_transaction.clone(),
            public_key,
            private_key.sign(&raw_transaction),
        );

        Transaction::UserTransaction(signed_transaction)
    }

    fn create_contract_event() -> ContractEvent {
        ContractEvent::new(
            EventKey::new_from_address(&AccountAddress::random(), 0),
            0,
            TypeTag::Bool,
            b"some event bytes".to_vec(),
        )
    }

    fn create_ledger_info() -> LedgerInfoWithSignatures {
        LedgerInfoWithSignatures::new(
            LedgerInfo::new(BlockInfo::empty(), HashValue::zero()),
            BTreeMap::new(),
        )
    }

    fn create_runtime() -> Runtime {
        Builder::new_multi_thread().enable_all().build().unwrap()
    }
}
