// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    shared_mempool::{
        network::MempoolSyncMsg,
        tasks,
        types::{notify_subscribers, SharedMempool, SharedMempoolNotification},
    },
};
use anyhow::{ensure, Error, Result};
use itertools::Itertools;
use libra_config::{
    config::{MempoolConfig, PeerNetworkId, UpstreamConfig},
    network_id::NetworkId,
};
use libra_logger::prelude::*;
use netcore::transport::ConnectionOrigin;
use rand::seq::SliceRandom;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
    str::FromStr,
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
    broadcast_info: BroadcastInfo,
}

/// ACK status for a broadcasted batch of txns
#[derive(Clone)]
enum AckStatus {
    /// waiting for an ACK
    Pending,
    /// received ACK that marks a broadcast as retriable in the future
    Retry,
}

/// Metadata for a single broadcasted batch of txns
#[derive(Clone)]
struct BatchInfo {
    /// time when this batch was last broadcast
    pub send_time: SystemTime,
    /// latest ACK status for this batch
    pub ack_status: AckStatus,
}

/// Identifier for a broadcasted batch of txns
/// For BatchId(`start_id`, `end_id`), (`start_id`, `end_id`) is the range of timeline IDs read from
/// the core mempool timeline index that produced the txns in this batch
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
struct BatchId(u64, u64);

impl BatchId {
    /// Creates unique request id for the batch in the format "{start_id}_{end_id}"
    fn create_request_id(&self) -> String {
        format!("{}_{}", self.0, self.1)
    }
}

/// Txn broadcast-related info for a given remote peer
#[derive(Clone)]
struct BroadcastInfo {
    // sent broadcasts that have not yet received a success ACK
    pub sent_batches: BTreeMap<BatchId, BatchInfo>,
    // whether broadcasting to this peer is in backoff mode, e.g. broadcasting at longer intervals
    pub backoff_mode: bool,
}

impl BroadcastInfo {
    fn new() -> Self {
        Self {
            sent_batches: BTreeMap::new(),
            backoff_mode: false,
        }
    }
}

pub(crate) struct PeerManager {
    upstream_config: UpstreamConfig,
    mempool_config: MempoolConfig,
    peer_info: Mutex<PeerInfo>,
    // the upstream peer to failover to if all peers in the primary upstream network are dead
    // the number of failover peers is limited to 1 to avoid network competition in the failover networks
    failover_peer: Mutex<Option<PeerNetworkId>>,
}

impl FromStr for BatchId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let batch_range = s.split('_').collect::<Vec<_>>();
        ensure!(
            batch_range.len() == 2,
            "failed to parse invalid string {} to batch id",
            s
        );

        let start_range = batch_range[0].parse::<u64>()?;
        let end_range = batch_range[1].parse::<u64>()?;
        Ok(Self(start_range, end_range))
    }
}

impl Ord for BatchId {
    fn cmp(&self, other: &BatchId) -> Ordering {
        (self.0, self.1).cmp(&(other.0, other.1))
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

    /// Returns whether backpressure mode is on for broadcasting to this peer
    pub fn is_backoff_mode(&self, peer: &PeerNetworkId) -> bool {
        self.peer_info
            .lock()
            .expect("failed to acquire peer info lock")
            .get(peer)
            .expect("missing peer info for peer")
            .broadcast_info
            .backoff_mode
    }

    /// Broadcasts txns to `peer`
    ///
    /// Broadcast strategy:
    /// Prioritize resending older broadcasts that timed out (i.e. didn't receive ACK in set window of time)
    /// or is retriable over sending more new broadcasts
    /// 1. clear pending (i.e. unACK'ed) broadcasts if the corresponding timeline index read is empty
    /// 2. find earliest broadcast that timed out or is retriable - rebroadcast if any
    /// 3. else, broadcast fresh batch of txns
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
            .collect();

        // check for batch to rebroadcast:
        // 1. batch that did not receive ACK in configured window of time
        // 2. batch that an earlier ACK marked as retriable
        let rebroadcast = state
            .broadcast_info
            .sent_batches
            .iter()
            .find(|(_id, batch)| {
                batch
                    .send_time
                    .checked_add(Duration::from_millis(
                        self.mempool_config.shared_mempool_ack_timeout_ms,
                    ))
                    .map_or(false, |deadline| {
                        SystemTime::now().duration_since(deadline).is_ok()
                    })
                    || matches!(batch.ack_status, AckStatus::Retry)
            })
            .map(|(id, _batch)| (*id, mempool.timeline_range(id.0, id.1)));

        let (batch_id, transactions) = match rebroadcast {
            Some((batch_id, txns)) => {
                debug!("rebroadcasting {:?}", batch_id);
                (batch_id, txns)
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

        if transactions.is_empty() {
            return;
        }

        // don't hold mempool lock during network send
        drop(mempool);

        // execute actual network send
        let mut network_sender = smp
            .network_senders
            .get_mut(&peer.network_id())
            .expect("[shared mempool] missing network sender")
            .clone();
        let num_txns = transactions.len();
        let send_time = SystemTime::now();
        if let Err(e) = tasks::send_mempool_sync_msg(
            MempoolSyncMsg::BroadcastTransactionsRequest {
                request_id: batch_id.create_request_id(),
                transactions,
            },
            peer.peer_id(),
            &mut network_sender,
        ) {
            error!(
                "[shared mempool] error broadcasting transactions to peer {:?}: {}",
                peer, e
            );
        } else {
            let peer_id = &peer.peer_id().to_string();
            counters::SHARED_MEMPOOL_BROADCAST_LATENCY
                .with_label_values(&[peer_id])
                .observe(start_time.elapsed().as_secs_f64());
            counters::SHARED_MEMPOOL_TRANSACTION_BROADCAST
                .with_label_values(&[peer_id])
                .observe(num_txns as f64);
            counters::SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT
                .with_label_values(&[peer_id])
                .inc();

            // update peer sync state with info from above broadcast
            state.timeline_id = std::cmp::max(state.timeline_id, batch_id.1);
            // turn off backoff mode after every broadcast
            state.broadcast_info.backoff_mode = false;
            state.broadcast_info.sent_batches.insert(
                batch_id,
                BatchInfo {
                    send_time,
                    ack_status: AckStatus::Pending,
                },
            );
            notify_subscribers(SharedMempoolNotification::Broadcast, &smp.subscribers);
        }
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
        batch_id: String,
        retry_txns: Vec<u64>,
        backoff: bool,
        // timestamp of ACK received
        timestamp: SystemTime,
    ) {
        let batch_id = match BatchId::from_str(&batch_id) {
            Ok(id) => id,
            Err(e) => {
                error!(
                    "[mempool] received ack with invalid batch id {}: {}",
                    batch_id, e
                );
                return;
            }
        };

        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");

        if let Some(batch) = sync_state.broadcast_info.sent_batches.get(&batch_id) {
            // track broadcast roundtrip latency
            if let Ok(rtt) = timestamp.duration_since(batch.send_time) {
                counters::SHARED_MEMPOOL_BROADCAST_RTT
                    .with_label_values(&[&peer.peer_id().to_string()])
                    .observe(rtt.as_secs_f64());
            }

            // track ACK counter
            counters::SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT
                .with_label_values(&[&peer.peer_id().to_string()])
                .dec();
        } else {
            error!(
                "[mempool] received ACK for unknown batch id {:?} from {:?}",
                batch_id, peer
            );
            return;
        }

        // For a fancier broadcasting strategy, we can only try to resend retriable txns
        // versus resending the entire batch
        // However, per-transaction batch analysis is not scalable CPU-wise/not worth the complexity,
        // so for efficiency, we resend the entire batch if any of its txns are retriable
        if retry_txns.is_empty() {
            // remove broadcast from pending
            sync_state.broadcast_info.sent_batches.remove(&batch_id);
        } else {
            sync_state
                .broadcast_info
                .sent_batches
                .entry(batch_id)
                .and_modify(|batch| batch.ack_status = AckStatus::Retry);
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
    fn is_picked_peer(&self, peer: &PeerNetworkId) -> bool {
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
