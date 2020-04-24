// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Objects used by/related to shared mempool

use crate::{
    core_mempool::CoreMempool,
    shared_mempool::{network::MempoolNetworkSender, peer_manager::PeerManager},
};
use anyhow::Result;
use futures::{
    channel::{mpsc, mpsc::UnboundedSender, oneshot},
    Stream,
};
use libra_config::config::MempoolConfig;
use libra_types::{
    account_address::AccountAddress,
    mempool_status::MempoolStatus,
    on_chain_config::{ConfigID, LibraVersion, OnChainConfig, VMConfig},
    transaction::SignedTransaction,
    vm_error::VMStatus,
    PeerId,
};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
};
use storage_client::StorageRead;
use tokio::sync::RwLock;
use vm_validator::vm_validator::TransactionValidation;

pub(crate) const DEFAULT_MIN_BROADCAST_RECIPIENT_COUNT: usize = 0;

/// Struct that owns all dependencies required by shared mempool routines
#[derive(Clone)]
pub(crate) struct SharedMempool<V>
where
    V: TransactionValidation + 'static,
{
    pub mempool: Arc<Mutex<CoreMempool>>,
    pub config: MempoolConfig,
    pub network_senders: HashMap<PeerId, MempoolNetworkSender>,
    pub storage_read_client: Arc<dyn StorageRead>,
    pub validator: Arc<RwLock<V>>,
    pub peer_manager: Arc<PeerManager>,
    /// optional `k-policy` enforcer
    /// if None, we rely only on TTL to remove transactions from mempool
    pub ack_policy: Arc<Option<Mutex<AckPolicy>>>,
    pub subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SharedMempoolNotification {
    Sync,
    PeerStateChange,
    NewTransactions,
    ACK,
}

#[derive(Debug)]
pub(crate) struct SyncEvent;

pub(crate) type IntervalStream = Pin<Box<dyn Stream<Item = SyncEvent> + Send + 'static>>;

pub(crate) fn notify_subscribers(
    event: SharedMempoolNotification,
    subscribers: &[UnboundedSender<SharedMempoolNotification>],
) {
    for subscriber in subscribers {
        let _ = subscriber.unbounded_send(event);
    }
}

/// represents `k-policy` to ensure reliable txn delivery:
/// - removes txn from mempool if at least `k` peers have successfully ACK'ed for it
#[derive(Clone)]
pub(crate) struct AckPolicy {
    // tracks how many ACKs have been received for a pending broadcasted txn
    // (k, v) = (txn timeline_id, # ACKs received)
    pub ack_counts: HashMap<u64, usize>,
    pub min_acks: NonZeroUsize,
}

impl AckPolicy {
    pub fn new(min_acks: NonZeroUsize) -> Self {
        Self {
            ack_counts: HashMap::new(),
            min_acks,
        }
    }

    // updates ACK count for txn with `timeline_id`
    // returns whether txn has enough ACKs after this update
    pub fn is_ack_enough(&mut self, timeline_id: u64) -> bool {
        let acks = self
            .ack_counts
            .entry(timeline_id)
            .and_modify(|acks| *acks += 1)
            .or_insert(1);
        if acks >= &mut self.min_acks.get() {
            // remove
            self.ack_counts.remove(&timeline_id);
            true
        } else {
            false
        }
    }
}

/// Message sent from consensus to mempool
pub enum ConsensusRequest {
    /// request to pull block to submit to consensus
    GetBlockRequest(
        // max block size
        u64,
        // transactions to exclude from requested block
        Vec<TransactionExclusion>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
    /// notifications about *rejected* committed txns
    RejectNotification(
        // committed transactions
        Vec<CommittedTransaction>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
}

/// Response setn from mempool to consensus
pub enum ConsensusResponse {
    /// block to submit to consensus
    GetBlockResponse(
        // transactions in block
        Vec<SignedTransaction>,
    ),
    /// ACK for commit notification
    CommitResponse(),
}

/// notification from state sync to mempool of commit event
/// This notifies mempool to remove committed txns
pub struct CommitNotification {
    /// committed transactions
    pub transactions: Vec<CommittedTransaction>,
    /// timestamp of committed block
    pub block_timestamp_usecs: u64,
    /// callback to send back response from mempool to State Sync
    pub callback: oneshot::Sender<Result<CommitResponse>>,
}

/// ACK response to commit notification
#[derive(Debug)]
pub struct CommitResponse {
    /// error msg if applicable - empty string if commit was processed successfully by mempool
    pub msg: String,
}

/// successfully executed and committed txn
pub struct CommittedTransaction {
    /// sender
    pub sender: AccountAddress,
    /// sequence number
    pub sequence_number: u64,
}

/// excluded txn
pub struct TransactionExclusion {
    /// sender
    pub sender: AccountAddress,
    /// sequence number
    pub sequence_number: u64,
}

/// Submission Status is represented as combination of vm_validator internal status and core mempool insertion status
pub type SubmissionStatus = (MempoolStatus, Option<VMStatus>);

/// sender type: used to enqueue new transactions to shared mempool by client endpoints
pub type MempoolClientSender =
    mpsc::Sender<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>;

/// On-chain configs that mempool subscribes to for reconfiguration
pub const MEMPOOL_SUBSCRIBED_CONFIGS: &[ConfigID] = &[LibraVersion::CONFIG_ID, VMConfig::CONFIG_ID];
