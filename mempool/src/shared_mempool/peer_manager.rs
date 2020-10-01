// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    network::MempoolSyncMsg,
    shared_mempool::types::{notify_subscribers, SharedMempool, SharedMempoolNotification},
};
use itertools::Itertools;
use libra_config::{
    config::{MempoolConfig, PeerNetworkId, UpstreamConfig},
    network_id::NetworkId,
};
use libra_logger::prelude::*;
use libra_types::transaction::SignedTransaction;
use netcore::transport::ConnectionOrigin;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::{Add, Deref, DerefMut},
    sync::Mutex,
    time::{Duration, Instant, SystemTime},
};
use vm_validator::vm_validator::TransactionValidation;

/// stores only peers that receive txns from this node
pub(crate) type PeerInfo = HashMap<PeerNetworkId, PeerSyncState>;

/// state of last sync with peer
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
    peer_info: Mutex<PeerInfo>,
    // the upstream peer to failover to if all peers in the primary upstream network are dead
    // the number of failover peers is limited to 1 to avoid network competition in the failover networks
    failover_peer: Mutex<Option<PeerNetworkId>>,
}
/// Identifier for a broadcasted batch of txns
/// For BatchId(`start_id`, `end_id`), (`start_id`, `end_id`) is the range of timeline IDs read from
/// the core mempool timeline index that produced the txns in this batch
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Serialize)]
pub struct BatchId(pub u64, pub u64);

impl Ord for BatchId {
    fn cmp(&self, other: &BatchId) -> std::cmp::Ordering {
        (other.0, other.1).cmp(&(self.0, self.1))
    }
}

/// Txn broadcast-related info for a given remote peer
#[derive(Clone)]
pub struct BroadcastInfo {
    // sent broadcasts that have not yet received an ACK
    pub sent_batches: BTreeMap<BatchId, SystemTime>,
    // broadcasts that have received a retry ACK and are pending a resend
    pub retry_batches: BTreeSet<BatchId>,
    // whether braodcasting to this peer is in backoff mode, e.g. broadcasting at longer intervals
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
        Self {
            mempool_config,
            upstream_config,
            peer_info: Mutex::new(PeerInfo::new()),
            failover_peer: Mutex::new(None),
        }
    }

    // returns true if `peer` is discovered for first time, else false
    pub fn add_peer(&self, peer: PeerNetworkId, origin: ConnectionOrigin) -> bool {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");
        let is_new_peer = !peer_info.contains_key(&peer);
        if self.is_upstream_peer(&peer, Some(origin)) {
            if peer.raw_network_id() == NetworkId::Validator {
                // For a validator network, resume broadcasting from previous state
                // we can afford to not re-broadcast here since the transaction is already in a validator
                peer_info
                    .entry(peer)
                    .or_insert(PeerSyncState {
                        timeline_id: 0,
                        is_alive: true,
                        broadcast_info: BroadcastInfo::new(),
                    })
                    .is_alive = true;
            } else {
                // For a non-validator network, potentially re-broadcast any transactions that have not been
                // committed yet
                // This is to ensure better reliability of transactions reaching the validator network
                peer_info.insert(
                    peer,
                    PeerSyncState {
                        timeline_id: 0,
                        is_alive: true,
                        broadcast_info: BroadcastInfo::new(),
                    },
                );
            }
        }
        drop(peer_info);
        self.update_failover();
        is_new_peer
    }

    pub fn disable_peer(&self, peer: PeerNetworkId) {
        if let Some(state) = self
            .peer_info
            .lock()
            .expect("failed to get peer info lock")
            .get_mut(&peer)
        {
            state.is_alive = false;
        }
        self.update_failover();
    }

    pub fn is_backoff_mode(&self, peer: &PeerNetworkId) -> bool {
        self.peer_info
            .lock()
            .expect("failed to acquire peer info lock")
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
        // start timer for tracking broadcast latency
        let start_time = Instant::now();

        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer info lock");
        let state = peer_info
            .get_mut(&peer)
            .expect("missing peer info for peer");

        // only broadcast to peer that is both alive and picked
        if !state.is_alive || !self.is_picked_peer(&peer) {
            return;
        }

        // If backoff mode is on for this peer, only execute broadcasts that were scheduled as a backoff broadcast
        // This is to ensure the backoff mode is actually honored (there is a chance a broadcast was scheduled
        // in non-backoff mode before backoff mode was turned on - ignore such scheduled broadcasts)
        if state.broadcast_info.backoff_mode && !scheduled_backoff {
            return;
        }

        let batch_id: BatchId;
        let transactions: Vec<SignedTransaction>;
        {
            let mut mempool = smp
                .mempool
                .lock()
                .expect("failed to acquire core mempool lock");

            // Sync peer's pending broadcasts with latest mempool state
            // A pending broadcast might become empty if the corresponding txns were committed through
            // another peer, so don't track broadcasts for committed txns
            state.broadcast_info.sent_batches = state
                .broadcast_info
                .sent_batches
                .clone()
                .into_iter()
                .filter(|(id, _batch)| !mempool.timeline_range(id.0, id.1).is_empty())
                .collect::<BTreeMap<BatchId, SystemTime>>();

            // check for batch to rebroadcast:
            // 1. batch that did not receive ACK in configured window of time
            // 2. batch that an earlier ACK marked as retriable
            let expired = state
                .broadcast_info
                .sent_batches
                .iter()
                .find(|(_, time)| {
                    let deadline = time.add(Duration::from_millis(
                        self.mempool_config.shared_mempool_ack_timeout_ms,
                    ));
                    SystemTime::now().duration_since(deadline).is_ok()
                })
                .map(|(id, _)| id);
            let retry = state.broadcast_info.retry_batches.iter().next();

            let (new_batch_id, new_transactions) = match std::cmp::max(expired, retry) {
                Some(id) => {
                    let txns = mempool.timeline_range(id.0, id.1);
                    (*id, txns)
                }
                None => {
                    // fresh broadcast
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

        // execute actual network send
        let mut network_sender = smp
            .network_senders
            .get_mut(&peer.network_id())
            .expect("[shared mempool] missing network sender")
            .clone();

        let num_txns = transactions.len();
        let peer_id_str = peer.peer_id().to_string();
        if let Err(e) = network_sender.send_to(
            peer.peer_id(),
            MempoolSyncMsg::BroadcastTransactionsRequest {
                request_id: lcs::to_bytes(&batch_id).expect("failed LCS serialization of batch ID"),
                transactions,
            },
        ) {
            counters::NETWORK_SEND_FAIL
                .with_label_values(&[counters::BROADCAST_TXNS])
                .inc();
            error!(
                LogSchema::event_log(LogEntry::BroadcastTransaction, LogEvent::NetworkSendFail)
                    .peer(&peer)
                    .error(&e.into())
            );
            return;
        }
        // update peer sync state with info from above broadcast
        state.timeline_id = std::cmp::max(state.timeline_id, batch_id.1);
        // turn off backoff mode after every broadcast
        state.broadcast_info.backoff_mode = false;
        state
            .broadcast_info
            .sent_batches
            .insert(batch_id, SystemTime::now());
        state.broadcast_info.retry_batches.remove(&batch_id);
        notify_subscribers(SharedMempoolNotification::Broadcast, &smp.subscribers);

        counters::SHARED_MEMPOOL_TRANSACTION_BROADCAST
            .with_label_values(&[&peer_id_str])
            .observe(num_txns as f64);
        counters::SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT
            .with_label_values(&[&peer_id_str])
            .inc();
        counters::SHARED_MEMPOOL_BROADCAST_LATENCY
            .with_label_values(&[&peer_id_str])
            .observe(start_time.elapsed().as_secs_f64());
    }

    // updates the peer chosen to failover to if all peers in the primary upstream network are down
    fn update_failover(&self) {
        // failover is enabled only if there are multiple upstream networks
        if self.upstream_config.networks.len() < 2 {
            return;
        }

        // declare `failover` as standalone to satisfy lifetime requirement
        let mut failover = self
            .failover_peer
            .lock()
            .expect("failed to acquire failover lock");
        let current_failover = failover.deref_mut();
        let peer_info = self.peer_info.lock().expect("can not get peer info lock");
        let active_peers_by_network = peer_info
            .iter()
            .filter_map(|(peer, state)| {
                if state.is_alive {
                    Some((peer.raw_network_id(), peer))
                } else {
                    None
                }
            })
            .into_group_map();

        let primary_upstream = self
            .upstream_config
            .networks
            .get(0)
            .expect("missing primary upstream network");
        if active_peers_by_network.get(primary_upstream).is_none() {
            // there are no live peers in the primary upstream network - pick a failover peer

            let mut failover_candidate = None;
            // find the highest-pref'ed network (based on preference defined in upstream config)
            // with any live peer and pick a peer from that network
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
                        && peer_info.get(chosen).expect("missing peer state").is_alive
                    {
                        // if current chosen failover peer is alive, then do not overwrite it
                        // with another live peer of the same network
                        // for mempool broadcasts, broadcasting to the same peer consistently makes
                        // faster progress
                        return;
                    }
                }
            }

            *current_failover = failover_candidate.cloned().cloned();
        } else {
            // there is at least one peer alive in the primary upstream network, so don't pick
            // a failover peer
            *current_failover = None;
        }
    }

    pub fn process_broadcast_ack(
        &self,
        peer: PeerNetworkId,
        request_id_bytes: Vec<u8>,
        retry_txns: Vec<u64>,
        backoff: bool,
        // timestamp of ACK received
        timestamp: SystemTime,
    ) {
        let batch_id = if let Ok(id) = lcs::from_bytes::<BatchId>(&request_id_bytes) {
            id
        } else {
            counters::INVALID_ACK_RECEIVED_COUNT
                .with_label_values(&[&peer.peer_id().to_string()])
                .inc();
            return;
        };

        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");

        if let Some(sent_timestamp) = sync_state.broadcast_info.sent_batches.remove(&batch_id) {
            // track broadcast roundtrip latency
            let rtt = timestamp
                .duration_since(sent_timestamp)
                .expect("failed to calculate mempool broadcast RTT");
            counters::SHARED_MEMPOOL_BROADCAST_RTT
                .with_label_values(&[&peer.peer_id().to_string()])
                .observe(rtt.as_secs_f64());

            // update ACK counter
            counters::SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT
                .with_label_values(&[&peer.peer_id().to_string()])
                .dec();
        } else {
            // log and return
            // sample log
            return;
        }

        if !retry_txns.is_empty() {
            sync_state.broadcast_info.retry_batches.insert(batch_id);
        }

        // if backoff mode can only be turned off by executing a broadcast that was scheduled
        // as a backoff broadcast
        // This ensures backpressure request from remote peer is honored at least once
        if backoff {
            sync_state.broadcast_info.backoff_mode = true;
        }
    }

    // if the origin is provided, checks whether this peer is an upstream peer based on configured preferences and
    // connection origin
    // if the origin is not provided, checks whether this peer is an upstream peer that was seen before
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
            self.peer_info
                .lock()
                .expect("failed to acquire peer lock")
                .contains_key(peer)
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

    // checks whether a peer is a chosen broadcast recipient:
    // - all primary peers
    // - fallback peers, if k-policy is enabled
    // this does NOT check for whether this peer is alive
    pub fn is_picked_peer(&self, peer: &PeerNetworkId) -> bool {
        if self.is_primary_upstream_peer(&peer) {
            return true;
        }

        // checks whether this peer is a chosen upstream failover peer
        // NOTE: originally mempool should only broadcast to one peer in the failover network. This is
        // to avoid creating too much competition for traffic in the failover network, which is in most
        // cases also used by other public clients
        // However, currently for VFN's public on-chain discovery, there is the unfortunate possibility
        // that it might discover and select itself as an upstream fallback peer, so the txns will be
        // self-broadcasted and make no actual progress.
        // So until self-connection is actively checked against for in the networking layer, mempool
        // will temporarily broadcast to all peers in its selected failover network as well
        self.failover_peer
            .lock()
            .expect("failed to get failover peer")
            .deref()
            .as_ref()
            .map_or(false, |failover| {
                failover.raw_network_id() == peer.raw_network_id()
            })
    }
}
