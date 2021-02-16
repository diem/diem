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
use diem_config::{
    config::{MempoolConfig, PeerNetworkId, UpstreamConfig},
    network_id::NetworkId,
};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_types::transaction::SignedTransaction;
use itertools::Itertools;
use netcore::transport::ConnectionOrigin;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    ops::{Add, DerefMut},
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
}

pub(crate) struct PeerManager {
    upstream_config: UpstreamConfig,
    mempool_config: MempoolConfig,
    peer_states: Mutex<PeerSyncStates>,
    // The upstream peer to failover to if all peers in the primary upstream network are dead.
    // The number of failover peers is limited to 1 to avoid network competition in the failover networks.
    failover_peer: Mutex<Option<PeerNetworkId>>,
    // The set of `mempool_config.default_failover` number of peers in the non-primary networks to
    // broadcast to in addition to the primary network when the primary network is up.
    default_failovers: Mutex<HashSet<PeerNetworkId>>,
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
    pub fn new(mempool_config: MempoolConfig, upstream_config: UpstreamConfig) -> Self {
        // Primary network is always chosen at initialization.
        counters::upstream_network(PRIMARY_NETWORK_PREFERENCE);
        info!(LogSchema::new(LogEntry::UpstreamNetwork).network_level(PRIMARY_NETWORK_PREFERENCE));
        Self {
            mempool_config,
            upstream_config,
            peer_states: Mutex::new(PeerSyncStates::new()),
            failover_peer: Mutex::new(None),
            default_failovers: Mutex::new(HashSet::new()),
        }
    }

    // Returns true if `peer` is discovered for the first time, else false.
    pub fn add_peer(&self, peer: PeerNetworkId, origin: ConnectionOrigin) -> bool {
        let mut peer_states = self.peer_states.lock();
        let is_new_peer = !peer_states.contains_key(&peer);
        if self.is_upstream_peer(&peer, Some(origin)) {
            counters::active_upstream_peers(&peer.raw_network_id()).inc();
            if peer.raw_network_id() == NetworkId::Validator {
                // For a validator network, resume broadcasting from previous state.
                // We can afford to not re-broadcast here since the transaction is already in a validator.
                peer_states
                    .entry(peer)
                    .or_insert(PeerSyncState {
                        timeline_id: 0,
                        is_alive: true,
                        broadcast_info: BroadcastInfo::new(),
                    })
                    .is_alive = true;
            } else {
                // For a non-validator network, potentially re-broadcast any transactions that have not been
                // committed yet.
                // This is to ensure better reliability of transactions reaching the validator network.
                peer_states.insert(
                    peer,
                    PeerSyncState {
                        timeline_id: 0,
                        is_alive: true,
                        broadcast_info: BroadcastInfo::new(),
                    },
                );
            }
        }
        drop(peer_states);
        self.update_failover();
        is_new_peer
    }

    pub fn disable_peer(&self, peer: PeerNetworkId) {
        if let Some(state) = self.peer_states.lock().get_mut(&peer) {
            counters::active_upstream_peers(&peer.raw_network_id()).dec();
            state.is_alive = false;
        }

        {
            self.default_failovers.lock().remove(&peer);
        }
        self.update_failover();
    }

    pub fn is_backoff_mode(&self, peer: &PeerNetworkId) -> bool {
        self.peer_states
            .lock()
            .get(peer)
            .expect("missing peer info for peer")
            .broadcast_info
            .backoff_mode
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
        let state = peer_states
            .get_mut(&peer)
            .expect("missing peer info for peer");

        // Only broadcast to peer that is both alive and picked.
        if !state.is_alive || !self.is_picked_peer(&peer) {
            return;
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

    // Updates the peer chosen to failover to if all peers in the primary upstream network are down.
    // Tries to pick more `default_peers` until there are `mempool_config.default_peers` number of them.
    fn update_failover(&self) {
        // Failover is enabled only if there are multiple upstream networks.
        if self.upstream_config.networks.len() < 2 {
            return;
        }

        // Declare `failover` as standalone to satisfy lifetime requirement.
        let mut failover = self.failover_peer.lock();
        let current_failover = failover.deref_mut();
        let peer_states = self.peer_states.lock();
        let active_peers_by_network = peer_states
            .iter()
            .filter_map(|(peer, state)| {
                if state.is_alive {
                    Some((peer.raw_network_id(), peer))
                } else {
                    None
                }
            })
            .into_group_map();

        // Update default_failovers.
        // NOTE: this block of code, and maintaining even this concept of `default_failovers`, is a
        // *temporary* patch to improve the reliability of txn delivery in case of the primary
        // upstream network is lagging behind, and the txns delivered to it will not be ready for further
        // broadcast/consensus on that end based on its stale state.
        // So:
        // (1) don't invest too much in refactoring this code w.r.t. the rest of this function
        // (2) the logic of this function should later be migrated to a smarter networking layer
        // that can handle peer selection for mempool
        let mut default_failovers = self.default_failovers.lock();
        if default_failovers.len() < self.mempool_config.default_failovers {
            for failover_network in self.upstream_config.networks[1..].iter() {
                if let Some(active_peers) = active_peers_by_network.get(failover_network) {
                    for p in active_peers.iter().cloned() {
                        if default_failovers.insert(p.clone())
                            && default_failovers.len() >= self.mempool_config.default_failovers
                        {
                            break;
                        }
                    }
                }
            }
        }

        let primary_upstream = self
            .upstream_config
            .networks
            .get(0)
            .expect("missing primary upstream network");
        if active_peers_by_network.get(primary_upstream).is_none() {
            // There are no live peers in the primary upstream network - pick a failover peer.

            let mut failover_candidate = None;
            // Find the highest-pref'ed network (based on preference defined in upstream config)
            // with any live peer and pick a peer from that network.
            for failover_network in self.upstream_config.networks[1..].iter() {
                if let Some(active_peers) = active_peers_by_network.get(failover_network) {
                    failover_candidate = active_peers.choose(&mut rand::thread_rng());
                    if failover_candidate.is_some() {
                        break;
                    }
                }
            }

            if let Some(chosen) = current_failover {
                if let Some(candidate) = &failover_candidate {
                    if chosen.raw_network_id() == candidate.raw_network_id()
                        && peer_states
                            .get(chosen)
                            .expect("missing peer state")
                            .is_alive
                    {
                        // If current chosen failover peer is alive, then do not overwrite it
                        // with another live peer of the same network.
                        // For mempool broadcasts, broadcasting to the same peer consistently makes
                        // faster progress.
                        return;
                    }
                }
            }
            *current_failover = failover_candidate.cloned().cloned();
        } else {
            // There is at least one peer alive in the primary upstream network, so don't pick
            // a failover peer.
            *current_failover = None;
        }

        // Log/update metric for the updated failover network.
        match failover.as_ref() {
            Some(peer) => {
                let failover_network = peer.raw_network_id();
                if let Some(network_preference) = self
                    .upstream_config
                    .get_upstream_preference(failover_network.clone())
                {
                    info!(LogSchema::new(LogEntry::UpstreamNetwork)
                        .upstream_network(&failover_network)
                        .network_level(network_preference));
                    counters::upstream_network(network_preference);
                }
            }
            None => {
                info!(LogSchema::new(LogEntry::UpstreamNetwork)
                    .upstream_network(primary_upstream)
                    .network_level(PRIMARY_NETWORK_PREFERENCE));
                counters::upstream_network(PRIMARY_NETWORK_PREFERENCE);
            }
        }
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
    pub fn is_upstream_peer(&self, peer: &PeerNetworkId, origin: Option<ConnectionOrigin>) -> bool {
        if let Some(origin) = origin {
            if Self::is_public_downstream(peer.raw_network_id(), origin) {
                false
            } else {
                self.upstream_config
                    .get_upstream_preference(peer.raw_network_id())
                    .is_some()
            }
        } else {
            self.peer_states.lock().contains_key(peer)
        }
    }

    fn is_primary_upstream_peer(&self, peer: &PeerNetworkId) -> bool {
        self.upstream_config
            .get_upstream_preference(peer.raw_network_id())
            == Some(0)
    }

    fn is_public_downstream(network_id: NetworkId, origin: ConnectionOrigin) -> bool {
        network_id == NetworkId::Public && origin == ConnectionOrigin::Inbound
    }

    // Checks whether a peer is a chosen broadcast recipient:
    // - all primary peers
    // - fallback peers, if k-policy is enabled
    // This does NOT check for whether this peer is alive.
    pub fn is_picked_peer(&self, peer: &PeerNetworkId) -> bool {
        if self.is_primary_upstream_peer(&peer) {
            return true;
        }

        let failover = self.failover_peer.lock();
        if let Some(failover_peer) = failover.as_ref() {
            // In failover mode (i.e. primary network is down).
            // broadcast to all peers in the same network as this chosen upstream failover peer.

            // NOTE: originally mempool should only broadcast to one peer in the failover network. This is
            // to avoid creating too much competition for traffic in the failover network, which is in most
            // cases also used by other public clients
            // However, currently for VFN's public on-chain discovery, there is the unfortunate possibility
            // that it might discover and select itself as an upstream fallback peer, so the txns will be
            // self-broadcasted and make no actual progress.
            // So until self-connection is actively checked against for in the networking layer, mempool
            // will temporarily broadcast to all peers in its selected failover network as well
            failover_peer.raw_network_id() == peer.raw_network_id()
        } else {
            // if primary network is up, broadcast to all default_failovers in addition to it
            self.default_failovers.lock().contains(peer)
        }
    }
}
