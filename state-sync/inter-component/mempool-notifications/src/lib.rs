// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use std::{fmt, time::Duration};

use async_trait::async_trait;
use diem_types::{account_address::AccountAddress, transaction::Transaction};
use futures::channel::{mpsc, oneshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::timeout;

const MEMPOOL_NOTIFICATION_CHANNEL_SIZE: usize = 1;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Commit notification failed: {0}")]
    CommitNotificationError(String),
    #[error("Hit the timeout waiting for mempool to respond to the notification!")]
    TimeoutWaitingForMempool,
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
}

/// The interface between state sync and mempool, allowing state sync to notify
/// mempool of events (e.g., newly committed transactions).
#[async_trait]
pub trait MempoolNotificationSender: Send {
    /// Notify mempool of the newly committed transactions at the specified block timestamp.
    async fn notify_new_commit(
        &mut self,
        committed_transactions: Vec<Transaction>,
        block_timestamp_usecs: u64,
        notification_timeout_ms: u64,
    ) -> Result<(), Error>;
}

/// The state sync component responsible for notifying mempool.
///
/// Note: When a MempoolNotifier instance is created, mempool must take and
/// listen to the receiver in the corresponding MempoolNotificationListener.
#[derive(Debug)]
pub struct MempoolNotifier {
    notification_sender: mpsc::Sender<MempoolCommitNotification>,
}

impl MempoolNotifier {
    /// Returns a new MempoolNotifier and MempoolNotificationListener (to be
    /// used in conjuction with one another).
    pub fn new() -> (Self, MempoolNotificationListener) {
        let (notification_sender, notification_receiver) =
            mpsc::channel(MEMPOOL_NOTIFICATION_CHANNEL_SIZE);

        let mempool_notifier = MempoolNotifier {
            notification_sender,
        };
        let mempool_listener = MempoolNotificationListener::new(notification_receiver);
        (mempool_notifier, mempool_listener)
    }
}

#[async_trait]
impl MempoolNotificationSender for MempoolNotifier {
    async fn notify_new_commit(
        &mut self,
        transactions: Vec<Transaction>,
        block_timestamp_usecs: u64,
        notification_timeout_ms: u64,
    ) -> Result<(), Error> {
        // Get only user transactions from committed transactions
        let user_transactions: Vec<CommittedTransaction> = transactions
            .iter()
            .filter_map(|transaction| match transaction {
                Transaction::UserTransaction(signed_txn) => Some(CommittedTransaction {
                    sender: signed_txn.sender(),
                    sequence_number: signed_txn.sequence_number(),
                }),
                _ => None,
            })
            .collect();

        // Only send a notification if user transactions have been committed
        if user_transactions.is_empty() {
            return Ok(());
        }

        // Construct a oneshot channel to receive a mempool response
        let (callback, callback_receiver) = oneshot::channel();
        let commit_notification = MempoolCommitNotification {
            transactions: user_transactions,
            block_timestamp_usecs,
            callback,
        };

        // Send the notification to mempool
        if let Err(error) = self.notification_sender.try_send(commit_notification) {
            return Err(Error::CommitNotificationError(format!(
                "Failed to notify mempool of committed transactions! Error: {:?}",
                error
            )));
        }

        // Handle any responses or a timeout
        if let Ok(response) = timeout(
            Duration::from_millis(notification_timeout_ms),
            callback_receiver,
        )
        .await
        {
            match response {
                Ok(Ok(MempoolNotificationResponse::Success)) => Ok(()),
                Ok(Err(error)) => Err(Error::UnexpectedErrorEncountered(format!(
                    "Unexpected response from mempool! Error: {:?}",
                    error
                ))),
                Err(error) => Err(Error::UnexpectedErrorEncountered(format!("{:?}", error))),
            }
        } else {
            Err(Error::TimeoutWaitingForMempool)
        }
    }
}

/// A notification for newly committed transactions sent by state sync to mempool.
#[derive(Debug)]
pub struct MempoolCommitNotification {
    pub transactions: Vec<CommittedTransaction>,
    pub block_timestamp_usecs: u64, // The timestamp of the committed block.
    pub(crate) callback: oneshot::Sender<Result<MempoolNotificationResponse, Error>>,
}

impl fmt::Display for MempoolCommitNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MempoolCommitNotification [block_timestamp_usecs: {}, txns: {:?}]",
            self.block_timestamp_usecs, self.transactions
        )
    }
}

/// A successfully executed and committed user transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedTransaction {
    pub sender: AccountAddress,
    pub sequence_number: u64,
}

impl fmt::Display for CommittedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.sender, self.sequence_number,)
    }
}

/// The mempool component responsible for responding to state sync notifications.
#[derive(Debug)]
pub struct MempoolNotificationListener {
    pub notification_receiver: mpsc::Receiver<MempoolCommitNotification>,
}

impl MempoolNotificationListener {
    pub fn new(notification_receiver: mpsc::Receiver<MempoolCommitNotification>) -> Self {
        MempoolNotificationListener {
            notification_receiver,
        }
    }

    /// Respond (succesfully) to the commit notification previously sent by state sync.
    pub async fn ack_commit_notification(
        &mut self,
        mempool_commit_notification: MempoolCommitNotification,
    ) -> Result<(), Error> {
        mempool_commit_notification
            .callback
            .send(Ok(MempoolNotificationResponse::Success))
            .map_err(|error| Error::UnexpectedErrorEncountered(format!("{:?}", error)))
    }
}

/// A response from mempool for a notification.
///
/// Note: failure responses are not currently used.
#[derive(Debug)]
enum MempoolNotificationResponse {
    Success,
}

#[cfg(test)]
mod tests {
    use crate::{CommittedTransaction, Error, MempoolNotificationSender, MempoolNotifier};
    use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, SigningKey, Uniform};
    use diem_types::{
        account_address::AccountAddress,
        block_metadata::BlockMetadata,
        chain_id::ChainId,
        transaction::{
            ChangeSet, RawTransaction, Script, SignedTransaction, Transaction, TransactionPayload,
            WriteSetPayload,
        },
        write_set::WriteSetMut,
    };
    use futures::executor::block_on;
    use tokio::runtime::{Builder, Runtime};

    #[test]
    fn test_mempool_not_listening() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut mempool_listener) = MempoolNotifier::new();

        // Send a notification and expect a timeout (no listener)
        let notify_result =
            block_on(mempool_notifier.notify_new_commit(vec![create_user_transaction()], 0, 1000));
        assert!(matches!(
            notify_result,
            Err(Error::TimeoutWaitingForMempool)
        ));

        // Drop the receiver and try again
        mempool_listener.notification_receiver.close();
        let notify_result =
            block_on(mempool_notifier.notify_new_commit(vec![create_user_transaction()], 0, 1000));
        assert!(matches!(
            notify_result,
            Err(Error::CommitNotificationError(_))
        ));
    }

    #[test]
    fn test_zero_timeout() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut _mempool_listener) = MempoolNotifier::new();

        // Send a notification and expect a timeout (zero timeout)
        let notify_result =
            block_on(mempool_notifier.notify_new_commit(vec![create_user_transaction()], 0, 0));
        assert!(matches!(
            notify_result,
            Err(Error::TimeoutWaitingForMempool)
        ));
    }

    #[test]
    fn test_no_transactions() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut _mempool_listener) = MempoolNotifier::new();

        // Send a notification and verify no timeout because no notification was sent!
        let notify_result = block_on(mempool_notifier.notify_new_commit(vec![], 0, 1000));
        assert!(notify_result.is_ok());
    }

    #[test]
    fn test_transaction_filtering() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut _mempool_listener) = MempoolNotifier::new();

        // Create several transactions that should be filtered out
        let mut transactions = vec![];
        for _ in 0..5 {
            transactions.push(create_block_metadata_transaction())
        }
        for _ in 0..5 {
            transactions.push(create_genesis_transaction())
        }

        // Send a notification and verify no timeout because no notification was sent!
        let notify_result =
            block_on(mempool_notifier.notify_new_commit(transactions.clone(), 0, 1000));
        assert!(notify_result.is_ok());

        // Send another notification with a single user transaction now included.
        transactions.push(create_user_transaction());
        let notify_result = block_on(mempool_notifier.notify_new_commit(transactions, 0, 1000));
        assert!(matches!(
            notify_result,
            Err(Error::TimeoutWaitingForMempool)
        ));
    }

    #[test]
    fn test_commit_notification_arrives() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut mempool_listener) = MempoolNotifier::new();

        // Send a notification
        let user_transaction = create_user_transaction();
        let transactions = vec![user_transaction.clone()];
        let block_timestamp_usecs = 101;
        let _ =
            block_on(mempool_notifier.notify_new_commit(transactions, block_timestamp_usecs, 1000));

        // Verify the notification arrives at the receiver
        match mempool_listener.notification_receiver.try_next() {
            Ok(Some(mempool_commit_notification)) => match user_transaction {
                Transaction::UserTransaction(signed_transaction) => {
                    assert_eq!(
                        mempool_commit_notification.transactions,
                        vec![CommittedTransaction {
                            sender: signed_transaction.sender(),
                            sequence_number: signed_transaction.sequence_number(),
                        }]
                    );
                    assert_eq!(
                        mempool_commit_notification.block_timestamp_usecs,
                        block_timestamp_usecs
                    );
                }
                result => panic!("Expected user transaction but got: {:?}", result),
            },
            result => panic!("Expected mempool commit notification but got: {:?}", result),
        };
    }

    #[test]
    fn test_mempool_success_response() {
        // Create runtime and mempool notifier
        let runtime = create_runtime();
        let _enter = runtime.enter();
        let (mut mempool_notifier, mut mempool_listener) = MempoolNotifier::new();

        // Spawn a new thread to handle any messages on the receiver
        let _handler = std::thread::spawn(move || loop {
            if let Ok(Some(mempool_commit_notification)) =
                mempool_listener.notification_receiver.try_next()
            {
                let _result =
                    block_on(mempool_listener.ack_commit_notification(mempool_commit_notification));
            }
        });

        // Send a notification and verify a successful response
        let notify_result = block_on(mempool_notifier.notify_new_commit(
            vec![create_user_transaction()],
            101,
            1000,
        ));
        assert!(notify_result.is_ok());
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

    fn create_block_metadata_transaction() -> Transaction {
        Transaction::BlockMetadata(BlockMetadata::new(
            HashValue::new([0; HashValue::LENGTH]),
            1,
            300000001,
            vec![],
            AccountAddress::random(),
        ))
    }

    fn create_genesis_transaction() -> Transaction {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(ChangeSet::new(
            WriteSetMut::new(vec![])
                .freeze()
                .expect("freeze cannot fail"),
            vec![],
        )))
    }

    fn create_runtime() -> Runtime {
        Builder::new_multi_thread().enable_all().build().unwrap()
    }
}
