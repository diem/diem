// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    network::MempoolSyncMsg,
    shared_mempool::{
        tasks,
        types::{notify_subscribers, SharedMempool, SharedMempoolNotification},
    },
};
use diem_config::config::{MempoolConfig, PeerNetworkId, PeerRole, RoleType};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_types::transaction::SignedTransaction;
use itertools::Itertools;
use netcore::transport::ConnectionOrigin;
use network::transport::ConnectionMetadata;
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Add,
    time::{Duration, Instant, SystemTime},
};
use vm_validator::vm_validator::TransactionValidation;

const PRIMARY_NETWORK_PREFERENCE: usize = 0;

/// Peers that receive txns from this node.
pub(crate) type PeerSyncStates = HashMap<PeerNetworkId, PeerSyncState>;

/// State of last sync with peer:
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
#[derive(Clone)]
pub(crate) struct PeerSyncState {
    pub timeline_id: u64,
    pub is_alive: bool,
    pub broadcast_info: BroadcastInfo,
    pub metadata: ConnectionMetadata,
}

impl PeerSyncState {
    pub fn new(metadata: ConnectionMetadata) -> Self {
        PeerSyncState {
            timeline_id: 0,
            is_alive: true,
            broadcast_info: BroadcastInfo::new(),
            metadata,
        }
    }
}

pub(crate) struct PeerManager {
    /// Role of the current node
    role: RoleType,
    mempool_config: MempoolConfig,
    peer_states: Mutex<PeerSyncStates>,
    prioritized_peers: Mutex<Vec<PeerNetworkId>>,
}
/// Identifier for a broadcasted batch of txns.
/// For BatchId(`start_id`, `end_id`), (`start_id`, `end_id`) is the range of timeline IDs read from
/// the core mempool timeline index that produced the txns in this batch.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BatchId(pub u64, pub u64);

impl PartialOrd for BatchId {
    fn partial_cmp(&self, other: &BatchId) -> Option<std::cmp::Ordering> {
        Some((other.0, other.1).cmp(&(self.0, self.1)))
    }
}

impl Ord for BatchId {
    fn cmp(&self, other: &BatchId) -> std::cmp::Ordering {
        (other.0, other.1).cmp(&(self.0, self.1))
    }
}

/// Txn broadcast-related info for a given remote peer.
#[derive(Clone)]
pub struct BroadcastInfo {
    // Sent broadcasts that have not yet received an ack.
    pub sent_batches: BTreeMap<BatchId, SystemTime>,
    // Broadcasts that have received a retry ack and are pending a resend.
    pub retry_batches: BTreeSet<BatchId>,
    // Whether broadcasting to this peer is in backoff mode, e.g. broadcasting at longer intervals.
    pub backoff_mode: bool,
}

impl BroadcastInfo {
    fn new() -> Self {
        Self {
            sent_batches: BTreeMap::new(),
            retry_batches: BTreeSet::new(),
            backoff_mode: false,
        }
    }
}

impl PeerManager {
    pub fn new(role: RoleType, mempool_config: MempoolConfig) -> Self {
        // Primary network is always chosen at initialization.
        counters::upstream_network(PRIMARY_NETWORK_PREFERENCE);
        info!(LogSchema::new(LogEntry::UpstreamNetwork).network_level(PRIMARY_NETWORK_PREFERENCE));
        Self {
            role,
            mempool_config,
            peer_states: Mutex::new(PeerSyncStates::new()),
            prioritized_peers: Mutex::new(Vec::new()),
        }
    }

    // Returns true if `peer` is discovered for the first time, else false.
    pub fn add_peer(&self, peer: PeerNetworkId, metadata: ConnectionMetadata) -> bool {
        let mut peer_states = self.peer_states.lock();
        let is_new_peer = !peer_states.contains_key(&peer);
        if self.is_upstream_peer(&peer, Some(&metadata)) {
            // If we have a new peer, let's insert new data, otherwise, let's just update the current state
            if is_new_peer {
                counters::active_upstream_peers(&peer.raw_network_id()).inc();
                peer_states.insert(peer, PeerSyncState::new(metadata));
            } else if let Some(peer_state) = peer_states.get_mut(&peer) {
                if !peer_state.is_alive {
                    counters::active_upstream_peers(&peer.raw_network_id()).inc();
                }
                peer_state.is_alive = true;
                peer_state.metadata = metadata;
            }
        }
        drop(peer_states);

        // Always need to update the prioritized peers, because of `is_alive` state changes
        self.update_prioritized_peers();
        is_new_peer
    }

    /// Disables a peer if it can be restarted, otherwise removes it
    pub fn disable_peer(&self, peer: PeerNetworkId) {
        // Validators can be restarted ata  later time
        // TODO: Determine why there's this optimization
        // TODO: What about garbage collection of validators
        if peer.raw_network_id().is_validator_network() {
            if let Some(state) = self.peer_states.lock().get_mut(&peer) {
                counters::active_upstream_peers(&peer.raw_network_id()).dec();
                state.is_alive = false;
            }
        } else {
            // All other nodes have their state immediately restarted anyways, so let's free them
            // TODO: Why is the Validator optimization not applied here
            self.peer_states.lock().remove(&peer);
            counters::active_upstream_peers(&peer.raw_network_id()).dec();
        }

        // Always update prioritized peers to be in line with peer states
        self.update_prioritized_peers();
    }

    pub fn is_backoff_mode(&self, peer: &PeerNetworkId) -> bool {
        if let Some(state) = self.peer_states.lock().get(peer) {
            state.broadcast_info.backoff_mode
        } else {
            // If we don't have sync state, we shouldn't backoff
            false
        }
    }

    pub fn execute_broadcast<V>(
        &self,
        peer: PeerNetworkId,
        scheduled_backoff: bool,
        smp: &mut SharedMempool<V>,
    ) where
        V: TransactionValidation,
    {
        // Start timer for tracking broadcast latency.
        let start_time = Instant::now();

        let mut peer_states = self.peer_states.lock();
        let state = if let Some(state) = peer_states.get_mut(&peer) {
            state
        } else {
            // If we don't have any info about the node, we shouldn't broadcast to it
            return;
        };

        // Only broadcast to peers that are alive.
        if !state.is_alive {
            return;
        }

        // When not a validator, only broadcast to `default_failovers`
        if !self.role.is_validator() {
            let priority = self
                .prioritized_peers
                .lock()
                .iter()
                .find_position(|peer_network_id| *peer_network_id == &peer)
                .map_or(usize::MAX, |(pos, _)| pos);
            if priority > self.mempool_config.default_failovers {
                return;
            }
        }

        // If backoff mode is on for this peer, only execute broadcasts that were scheduled as a backoff broadcast.
        // This is to ensure the backoff mode is actually honored (there is a chance a broadcast was scheduled
        // in non-backoff mode before backoff mode was turned on - ignore such scheduled broadcasts).
        if state.broadcast_info.backoff_mode && !scheduled_backoff {
            return;
        }

        let batch_id: BatchId;
        let transactions: Vec<SignedTransaction>;
        let mut metric_label = None;
        {
            let mut mempool = smp.mempool.lock();

            // Sync peer's pending broadcasts with latest mempool state.
            // A pending broadcast might become empty if the corresponding txns were committed through
            // another peer, so don't track broadcasts for committed txns.
            state.broadcast_info.sent_batches = state
                .broadcast_info
                .sent_batches
                .clone()
                .into_iter()
                .filter(|(id, _batch)| !mempool.timeline_range(id.0, id.1).is_empty())
                .collect::<BTreeMap<BatchId, SystemTime>>();

            // Check for batch to rebroadcast:
            // 1. Batch that did not receive ACK in configured window of time
            // 2. Batch that an earlier ACK marked as retriable
            let mut pending_broadcasts = 0;
            let mut expired = None;

            // Find earliest batch in timeline index that expired.
            // Note that state.broadcast_info.sent_batches is ordered in decreasing order in the timeline index
            for (batch, sent_time) in state.broadcast_info.sent_batches.iter() {
                let deadline = sent_time.add(Duration::from_millis(
                    self.mempool_config.shared_mempool_ack_timeout_ms,
                ));
                if SystemTime::now().duration_since(deadline).is_ok() {
                    expired = Some(batch);
                } else {
                    pending_broadcasts += 1;
                }

                // The maximum number of broadcasts sent to a single peer that are pending a response ACK at any point.
                // If the number of un-ACK'ed un-expired broadcasts reaches this threshold, we do not broadcast anymore
                // and wait until an ACK is received or a sent broadcast expires.
                // This helps rate-limit egress network bandwidth and not overload a remote peer or this
                // node's Diem network sender.
                if pending_broadcasts >= self.mempool_config.max_broadcasts_per_peer {
                    return;
                }
            }
            let retry = state.broadcast_info.retry_batches.iter().rev().next();

            let (new_batch_id, new_transactions) = match std::cmp::max(expired, retry) {
                Some(id) => {
                    metric_label = if Some(id) == expired {
                        Some(counters::EXPIRED_BROADCAST_LABEL)
                    } else {
                        Some(counters::RETRY_BROADCAST_LABEL)
                    };

                    let txns = mempool.timeline_range(id.0, id.1);
                    (*id, txns)
                }
                None => {
                    // Fresh broadcast
                    let (txns, new_timeline_id) = mempool.read_timeline(
                        state.timeline_id,
                        self.mempool_config.shared_mempool_batch_size,
                    );
                    (BatchId(state.timeline_id, new_timeline_id), txns)
                }
            };

            batch_id = new_batch_id;
            transactions = new_transactions;
        }

        if transactions.is_empty() {
            return;
        }

        let mut network_sender = smp
            .network_senders
            .get_mut(&peer.network_id())
            .expect("[shared mempool] missing network sender")
            .clone();

        let num_txns = transactions.len();
        if let Err(e) = network_sender.send_to(
            peer.peer_id(),
            MempoolSyncMsg::BroadcastTransactionsRequest {
                request_id: bcs::to_bytes(&batch_id).expect("failed BCS serialization of batch ID"),
                transactions,
            },
        ) {
            counters::network_send_fail_inc(counters::BROADCAST_TXNS);
            error!(
                LogSchema::event_log(LogEntry::BroadcastTransaction, LogEvent::NetworkSendFail)
                    .peer(&peer)
                    .error(&e.into())
            );
            return;
        }
        // Update peer sync state with info from above broadcast.
        state.timeline_id = std::cmp::max(state.timeline_id, batch_id.1);
        // Turn off backoff mode after every broadcast.
        state.broadcast_info.backoff_mode = false;
        state
            .broadcast_info
            .sent_batches
            .insert(batch_id, SystemTime::now());
        state.broadcast_info.retry_batches.remove(&batch_id);
        notify_subscribers(SharedMempoolNotification::Broadcast, &smp.subscribers);

        let latency = start_time.elapsed();
        trace!(
            LogSchema::event_log(LogEntry::BroadcastTransaction, LogEvent::Success)
                .peer(&peer)
                .batch_id(&batch_id)
                .backpressure(scheduled_backoff)
        );
        let peer_id = peer.peer_id().short_str();
        let network_id = peer.raw_network_id();
        counters::SHARED_MEMPOOL_TRANSACTION_BROADCAST_SIZE
            .with_label_values(&[network_id.as_str(), peer_id.as_str()])
            .observe(num_txns as f64);
        counters::shared_mempool_pending_broadcasts(&peer)
            .set(state.broadcast_info.sent_batches.len() as i64);
        counters::SHARED_MEMPOOL_BROADCAST_LATENCY
            .with_label_values(&[network_id.as_str(), peer_id.as_str()])
            .observe(latency.as_secs_f64());
        if let Some(label) = metric_label {
            counters::SHARED_MEMPOOL_BROADCAST_TYPE_COUNT
                .with_label_values(&[network_id.as_str(), peer_id.as_str(), label])
                .inc();
        }
        if scheduled_backoff {
            counters::SHARED_MEMPOOL_BROADCAST_TYPE_COUNT
                .with_label_values(&[
                    network_id.as_str(),
                    peer_id.as_str(),
                    counters::BACKPRESSURE_BROADCAST_LABEL,
                ])
                .inc();
        }
    }

    fn update_prioritized_peers(&self) {
        // Only do this if it's not a validator
        if self.role.is_validator() {
            return;
        }

        // Retrieve just what's needed for the peer ordering
        let peers: Vec<_> = {
            let peer_states = self.peer_states.lock();
            peer_states
                .iter()
                .filter(|(_, state)| state.is_alive)
                .map(|(peer, state)| (peer.clone(), state.metadata.role))
                .collect()
        };

        // Order peers by network and by type
        // Origin doesn't matter at this point, only inserted ones into peer_states are upstream
        // Validators will always have the full set
        let mut prioritized_peers = self.prioritized_peers.lock();
        let peers: Vec<_> = peers
            .iter()
            .sorted_by(|peer_a, peer_b| compare_prioritized_peers(peer_a, peer_b))
            .map(|(peer, _)| peer.clone())
            .collect();
        let _ = std::mem::replace(&mut *prioritized_peers, peers);
    }

    pub fn process_broadcast_ack(
        &self,
        peer: PeerNetworkId,
        request_id_bytes: Vec<u8>,
        retry: bool,
        backoff: bool,
        timestamp: SystemTime,
    ) {
        let batch_id = if let Ok(id) = bcs::from_bytes::<BatchId>(&request_id_bytes) {
            id
        } else {
            counters::invalid_ack_inc(&peer, counters::INVALID_REQUEST_ID);
            return;
        };

        let mut peer_states = self.peer_states.lock();

        let sync_state = if let Some(state) = peer_states.get_mut(&peer) {
            state
        } else {
            counters::invalid_ack_inc(&peer, counters::UNKNOWN_PEER);
            return;
        };

        if let Some(sent_timestamp) = sync_state.broadcast_info.sent_batches.remove(&batch_id) {
            let rtt = timestamp
                .duration_since(sent_timestamp)
                .expect("failed to calculate mempool broadcast RTT");

            let network_id = peer.raw_network_id();
            let peer_id = peer.peer_id().short_str();
            counters::SHARED_MEMPOOL_BROADCAST_RTT
                .with_label_values(&[network_id.as_str(), peer_id.as_str()])
                .observe(rtt.as_secs_f64());

            counters::shared_mempool_pending_broadcasts(&peer).dec();
        } else {
            trace!(
                LogSchema::new(LogEntry::ReceiveACK)
                    .peer(&peer)
                    .batch_id(&batch_id),
                "batch ID does not exist or expired"
            );
            return;
        }

        trace!(
            LogSchema::new(LogEntry::ReceiveACK)
                .peer(&peer)
                .batch_id(&batch_id)
                .backpressure(backoff),
            retry = retry,
        );
        tasks::update_ack_counter(&peer, counters::RECEIVED_LABEL, retry, backoff);

        if retry {
            sync_state.broadcast_info.retry_batches.insert(batch_id);
        }

        // Backoff mode can only be turned off by executing a broadcast that was scheduled
        // as a backoff broadcast.
        // This ensures backpressure request from remote peer is honored at least once.
        if backoff {
            sync_state.broadcast_info.backoff_mode = true;
        }
    }

    // If the origin is provided, checks whether this peer is an upstream peer based on configured preferences and
    // connection origin.
    // If the origin is not provided, checks whether this peer is an upstream peer that was seen before.
    pub fn is_upstream_peer(
        &self,
        peer: &PeerNetworkId,
        metadata: Option<&ConnectionMetadata>,
    ) -> bool {
        // Validator network is always upstream
        if peer.raw_network_id().is_validator_network() {
            return true;
        }

        // Outbound connections are upstream on non-P2P networks
        if let Some(metadata) = metadata {
            metadata.origin == ConnectionOrigin::Outbound
        } else {
            // If we already know about the peer, it's upstream
            // TODO: If this is actually used it seems kinda pointless?
            self.peer_states.lock().contains_key(peer)
        }
    }
}

/// Provides ordering for prioritized peers
fn compare_prioritized_peers(
    peer_a: &(PeerNetworkId, PeerRole),
    peer_b: &(PeerNetworkId, PeerRole),
) -> Ordering {
    let network_a = peer_a.0.raw_network_id();
    let network_b = peer_b.0.raw_network_id();

    // Sort by NetworkId
    match network_a.cmp(&network_b) {
        Ordering::Equal => {
            // Then sort by Role
            let role_a = peer_a.1;
            let role_b = peer_b.1;
            match role_a.cmp(&role_b) {
                // Then tiebreak by PeerId for stability
                Ordering::Equal => {
                    let peer_id_a = peer_a.0.peer_id();
                    let peer_id_b = peer_b.0.peer_id();
                    peer_id_a.cmp(&peer_id_b)
                }
                ordering => ordering,
            }
        }
        ordering => ordering,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use diem_config::network_id::{NetworkId, NodeNetworkId};
    use diem_types::PeerId;

    fn peer_network_id(peer_id: PeerId, network: NetworkId) -> PeerNetworkId {
        PeerNetworkId(NodeNetworkId::new(network, 0), peer_id)
    }

    #[test]
    fn check_peer_prioritization() {
        let peer_id_1 = PeerId::from_hex_literal("0x1").unwrap();
        let peer_id_2 = PeerId::from_hex_literal("0x2").unwrap();
        let val_1 = (
            peer_network_id(peer_id_1, NetworkId::vfn_network()),
            PeerRole::Validator,
        );
        let val_2 = (
            peer_network_id(peer_id_2, NetworkId::vfn_network()),
            PeerRole::Validator,
        );
        let vfn_1 = (
            peer_network_id(peer_id_1, NetworkId::Public),
            PeerRole::ValidatorFullNode,
        );
        let preferred_1 = (
            peer_network_id(peer_id_1, NetworkId::Public),
            PeerRole::PreferredUpstream,
        );

        // NetworkId ordering
        assert_eq!(Ordering::Greater, compare_prioritized_peers(&vfn_1, &val_1));
        assert_eq!(Ordering::Less, compare_prioritized_peers(&val_1, &vfn_1));

        // PeerRole ordering
        assert_eq!(
            Ordering::Greater,
            compare_prioritized_peers(&vfn_1, &preferred_1)
        );
        assert_eq!(
            Ordering::Less,
            compare_prioritized_peers(&preferred_1, &vfn_1)
        );

        // Tiebreaker on peer_id
        assert_eq!(Ordering::Greater, compare_prioritized_peers(&val_2, &val_1));
        assert_eq!(Ordering::Less, compare_prioritized_peers(&val_1, &val_2));

        // Same the only equal case
        assert_eq!(Ordering::Equal, compare_prioritized_peers(&val_1, &val_1));
    }
}
