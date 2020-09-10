// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use itertools::Itertools;
use libra_config::{
    config::{PeerNetworkId, UpstreamConfig},
    network_id::NetworkId,
};
use netcore::transport::ConnectionOrigin;
use rand::seq::SliceRandom;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    ops::{Deref, DerefMut},
    sync::Mutex,
    time::Instant,
};

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
    peer_info: Mutex<PeerInfo>,
    // the upstream peer to failover to if all peers in the primary upstream network are dead
    // the number of failover peers is limited to 1 to avoid network competition in the failover networks
    failover_peer: Mutex<Option<PeerNetworkId>>,
}

#[derive(Clone)]
pub struct BroadcastInfo {
    // broadcasts that have not been ACK'ed for yet
    pub sent_batches: HashMap<String, BatchInfo>,
    // timeline IDs of all txns that need to be retried and ACKed for
    pub total_retry_txns: BTreeSet<u64>,
    // whether broadcasts are in backoff/backpressure mode, e.g. broadcasting at longer intervals
    pub backoff_mode: bool,
}

impl BroadcastInfo {
    fn new() -> Self {
        Self {
            sent_batches: HashMap::new(),
            total_retry_txns: BTreeSet::new(),
            backoff_mode: false,
        }
    }
}

#[derive(Clone)]
pub struct BatchInfo {
    /// Timeline IDs of broadcasted txns
    pub timeline_ids: Vec<u64>,
    /// timestamp of broadcast
    pub timestamp: Instant,
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig) -> Self {
        Self {
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

    pub fn update_peer_broadcast(
        &self,
        peer: PeerNetworkId,
        // ID of broadcast request
        batch_id: String,
        // timeline IDs of txns broadcasted
        batch: Vec<u64>,
        // the new timeline ID to read from for next broadcast
        timeline_id: u64,
        // timeline ID of first txn in timeline, used to remove potentially expired retry_txns
        earliest_timeline_id: u64,
        // timestamp of broadcast
        timestamp: Instant,
    ) {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");
        sync_state.broadcast_info.sent_batches.insert(
            batch_id,
            BatchInfo {
                timeline_ids: batch,
                timestamp,
            },
        );
        sync_state.timeline_id = std::cmp::max(sync_state.timeline_id, timeline_id);

        // clean up expired retriable txns
        let gc_retry_txns = sync_state
            .broadcast_info
            .total_retry_txns
            .iter()
            .filter(|x| *x >= &earliest_timeline_id)
            .cloned()
            .collect::<BTreeSet<_>>();

        sync_state.broadcast_info.total_retry_txns = gc_retry_txns;
    }

    pub fn process_broadcast_ack(
        &self,
        peer: PeerNetworkId,
        batch_id: String,
        retry_txns: Vec<u64>,
        backoff: bool,
        // timestamp of ACK received
        timestamp: Instant,
    ) {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");

        if let Some(batch) = sync_state.broadcast_info.sent_batches.remove(&batch_id) {
            // update ACK counter
            counters::SHARED_MEMPOOL_PENDING_BROADCASTS_COUNT
                .with_label_values(&[&peer.peer_id().to_string()])
                .dec();

            // track broadcast roundtrip latency
            if let Some(rtt) = timestamp.checked_duration_since(batch.timestamp) {
                counters::SHARED_MEMPOOL_BROADCAST_RTT
                    .with_label_values(&[&peer.peer_id().to_string()])
                    .observe(rtt.as_secs_f64());
            }

            // convert retry_txns from index within a batch to actual timeline ID of txn
            let retry_timeline_ids = retry_txns
                .iter()
                .filter_map(|batch_index| batch.timeline_ids.get(*batch_index as usize).cloned())
                .collect::<HashSet<_>>();
            for timeline_id in batch.timeline_ids.into_iter() {
                if retry_timeline_ids.contains(&timeline_id) {
                    // add this retriable txn's timeline ID to peer info's retry_txns
                    sync_state
                        .broadcast_info
                        .total_retry_txns
                        .insert(timeline_id);
                } else {
                    // this txn was successfully ACK'ed for - aggressively remove from retry_txns
                    sync_state
                        .broadcast_info
                        .total_retry_txns
                        .remove(&timeline_id);
                }
            }
        }
        sync_state.broadcast_info.backoff_mode = backoff;
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

    pub fn get_peer_state(&self, peer: &PeerNetworkId) -> PeerSyncState {
        self.peer_info
            .lock()
            .expect("failed to acquire peer info lock")
            .get(peer)
            .expect("missing peer sync state")
            .clone()
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
