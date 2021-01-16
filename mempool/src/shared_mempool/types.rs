// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Objects used by/related to shared mempool

use crate::{
    core_mempool::CoreMempool,
    shared_mempool::{network::MempoolNetworkSender, peer_manager::PeerManager},
};
use anyhow::Result;
use channel::diem_channel::Receiver;
use diem_config::{
    config::{MempoolConfig, PeerNetworkId},
    network_id::NodeNetworkId,
};
use diem_infallible::{Mutex, RwLock};
use diem_types::{
    account_address::AccountAddress,
    mempool_status::MempoolStatus,
    on_chain_config::{ConfigID, DiemVersion, OnChainConfig, OnChainConfigPayload, VMConfig},
    transaction::SignedTransaction,
    vm_status::DiscardedVMStatus,
};
use futures::{
    channel::{mpsc, mpsc::UnboundedSender, oneshot},
    future::Future,
    task::{Context, Poll},
};
use std::{collections::HashMap, fmt, pin::Pin, sync::Arc, task::Waker, time::Instant};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::runtime::Handle;
use vm_validator::vm_validator::TransactionValidation;

/// Struct that owns all dependencies required by shared mempool routines.
#[derive(Clone)]
pub(crate) struct SharedMempool<V>
where
    V: TransactionValidation + 'static,
{
    pub mempool: Arc<Mutex<CoreMempool>>,
    pub config: MempoolConfig,
    pub network_senders: HashMap<NodeNetworkId, MempoolNetworkSender>,
    pub db: Arc<dyn DbReader>,
    pub validator: Arc<RwLock<V>>,
    pub peer_manager: Arc<PeerManager>,
    pub subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SharedMempoolNotification {
    PeerStateChange,
    NewTransactions,
    ACK,
    Broadcast,
}

pub(crate) fn notify_subscribers(
    event: SharedMempoolNotification,
    subscribers: &[UnboundedSender<SharedMempoolNotification>],
) {
    for subscriber in subscribers {
        let _ = subscriber.unbounded_send(event);
    }
}

/// A future that represents a scheduled mempool txn broadcast
pub(crate) struct ScheduledBroadcast {
    /// Time of scheduled broadcast
    deadline: Instant,
    peer: PeerNetworkId,
    backoff: bool,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl ScheduledBroadcast {
    pub fn new(deadline: Instant, peer: PeerNetworkId, backoff: bool, executor: Handle) -> Self {
        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let waker_clone = waker.clone();

        if deadline > Instant::now() {
            let tokio_instant = tokio::time::Instant::from_std(deadline);
            executor.spawn(async move {
                tokio::time::delay_until(tokio_instant).await;
                let mut waker = waker_clone.lock();
                if let Some(waker) = waker.take() {
                    waker.wake()
                }
            });
        }

        Self {
            deadline,
            peer,
            backoff,
            waker,
        }
    }
}

impl Future for ScheduledBroadcast {
    type Output = (PeerNetworkId, bool); // (peer, whether this broadcast was scheduled as a backoff broadcast)

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        if Instant::now() < self.deadline {
            let waker_clone = context.waker().clone();
            let mut waker = self.waker.lock();
            *waker = Some(waker_clone);

            Poll::Pending
        } else {
            Poll::Ready((self.peer.clone(), self.backoff))
        }
    }
}

/// Message sent from consensus to mempool.
pub enum ConsensusRequest {
    /// Request to pull block to submit to consensus.
    GetBlockRequest(
        // max block size
        u64,
        // transactions to exclude from requested block
        Vec<TransactionExclusion>,
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
    /// Notifications about *rejected* committed txns.
    RejectNotification(
        Vec<CommittedTransaction>,
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
}

impl fmt::Display for ConsensusRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let payload = match self {
            ConsensusRequest::GetBlockRequest(block_size, excluded_txns, _) => {
                let mut txns_str = "".to_string();
                for tx in excluded_txns.iter() {
                    txns_str += &format!("{} ", tx);
                }
                format!(
                    "GetBlockRequest [block_size: {}, excluded_txns: {}]",
                    block_size, txns_str
                )
            }
            ConsensusRequest::RejectNotification(rejected_txns, _) => {
                let mut txns_str = "".to_string();
                for tx in rejected_txns.iter() {
                    txns_str += &format!("{} ", tx);
                }
                format!("RejectNotification [rejected_txns: {}]", txns_str)
            }
        };
        write!(f, "{}", payload)
    }
}

/// Response sent from mempool to consensus.
pub enum ConsensusResponse {
    /// Block to submit to consensus
    GetBlockResponse(
        Vec<SignedTransaction>,
    ),
    CommitResponse(),
}

/// Notification from state sync to mempool of commit event.
/// This notifies mempool to remove committed txns.
pub struct CommitNotification {
    pub transactions: Vec<CommittedTransaction>,
    /// Timestamp of committed block.
    pub block_timestamp_usecs: u64,
    pub callback: oneshot::Sender<Result<CommitResponse>>,
}

impl fmt::Display for CommitNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txns = "".to_string();
        for txn in self.transactions.iter() {
            txns += &format!("{} ", txn);
        }
        write!(
            f,
            "CommitNotification [block_timestamp_usecs: {}, txns: {}]",
            self.block_timestamp_usecs, txns
        )
    }
}

#[derive(Debug)]
pub struct CommitResponse {
    /// error msg if applicable - empty string if commit was processed successfully by mempool
    pub msg: String,
}

/// Successfully executed and committed txn
pub struct CommittedTransaction {
    pub sender: AccountAddress,
    pub sequence_number: u64,
}

impl fmt::Display for CommittedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.sender, self.sequence_number,)
    }
}

#[derive(Clone)]
pub struct TransactionExclusion {
    pub sender: AccountAddress,
    pub sequence_number: u64,
}

impl fmt::Display for TransactionExclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.sender, self.sequence_number,)
    }
}

pub type SubmissionStatus = (MempoolStatus, Option<DiscardedVMStatus>);

pub type SubmissionStatusBundle = (SignedTransaction, SubmissionStatus);

pub type MempoolClientSender =
    mpsc::Sender<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>;

const MEMPOOL_SUBSCRIBED_CONFIGS: &[ConfigID] = &[DiemVersion::CONFIG_ID, VMConfig::CONFIG_ID];

pub fn gen_mempool_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all("mempool", MEMPOOL_SUBSCRIBED_CONFIGS.to_vec(), vec![])
}
