// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::{PeerNetworkId, UpstreamConfig};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Mutex,
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
    min_broadcast_recipient_count: usize,
}

#[derive(Clone)]
pub struct BroadcastInfo {
    // broadcasts that have not been ACK'ed for yet
    pub sent_batches: HashMap<String, Vec<u64>>,
    // timeline IDs of all txns that need to be retried and ACKed for
    pub total_retry_txns: BTreeSet<u64>,
}

impl BroadcastInfo {
    fn new() -> Self {
        Self {
            sent_batches: HashMap::new(),
            total_retry_txns: BTreeSet::new(),
        }
    }
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig, min_broadcast_recipient_count: usize) -> Self {
        Self {
            upstream_config,
            peer_info: Mutex::new(PeerInfo::new()),
            min_broadcast_recipient_count,
        }
    }

    // returns true if `peer` is discovered for first time, else false
    pub fn add_peer(&self, peer: PeerNetworkId) -> bool {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");
        let is_new_peer = !peer_info.contains_key(&peer);
        if self.is_upstream_peer(peer) {
            peer_info
                .entry(peer)
                .or_insert(PeerSyncState {
                    timeline_id: 0,
                    is_alive: true,
                    broadcast_info: BroadcastInfo::new(),
                })
                .is_alive = true;
        }
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
    ) {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");
        sync_state
            .broadcast_info
            .sent_batches
            .insert(batch_id, batch);
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
    ) {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        let sync_state = peer_info.get_mut(&peer).expect("missing peer sync state");

        if let Some(batch) = sync_state.broadcast_info.sent_batches.remove(&batch_id) {
            // convert retry_txns from index within a batch to actual timeline ID of txn
            let retry_timeline_ids = retry_txns
                .iter()
                .filter_map(|batch_index| batch.get(*batch_index as usize).cloned())
                .collect::<HashSet<_>>();
            for timeline_id in batch.into_iter() {
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
    }

    pub fn get_broadcast_batch(&self, peer: PeerNetworkId, batch_id: &str) -> Option<Vec<u64>> {
        self.peer_info
            .lock()
            .expect("failed to acquire lock")
            .get(&peer)
            .expect("missing peer")
            .broadcast_info
            .sent_batches
            .get(batch_id)
            .cloned()
    }

    pub fn is_upstream_peer(&self, peer: PeerNetworkId) -> bool {
        self.upstream_config.is_upstream_peer(peer)
    }

    fn is_primary_upstream_peer(&self, peer: PeerNetworkId) -> bool {
        self.upstream_config.is_primary_upstream_peer(peer)
    }

    pub fn get_peer_state(&self, peer: PeerNetworkId) -> PeerSyncState {
        self.peer_info
            .lock()
            .expect("failed to acquire peer info lock")
            .get(&peer)
            .expect("missing peer sync state")
            .clone()
    }

    // checks whether a peer is a chosen broadcast recipient:
    // - all primary peers
    // - fallback peers, if k-policy is enabled
    // this does NOT check for whether this peer is alive
    pub fn is_picked_peer(&self, peer: PeerNetworkId) -> bool {
        if self.is_primary_upstream_peer(peer) {
            return true;
        }

        let no_live_primaries = self
            .peer_info
            .lock()
            .expect("failed to acquire peer info lock")
            .iter()
            .find(|(peer, state)| self.is_primary_upstream_peer(**peer) && state.is_alive)
            .is_none();

        // for fallback peers, broadcast if k-policy is on
        // TODO change from sending to k fallback peers instead of sending to all fallback peers
        no_live_primaries && self.min_broadcast_recipient_count > 0
    }
}
