// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::{PeerNetworkId, UpstreamConfig};
use std::{collections::HashMap, sync::Mutex};

/// stores only peers that receive txns from this node
pub(crate) type PeerInfo = HashMap<PeerNetworkId, PeerSyncState>;

/// state of last sync with peer
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
#[derive(Clone)]
pub(crate) struct PeerSyncState {
    pub timeline_id: u64,
    pub is_alive: bool,
    pub retry_timeline_id: Option<u64>,
}

pub(crate) struct PeerManager {
    upstream_config: UpstreamConfig,
    peer_info: Mutex<PeerInfo>,
    min_broadcast_recipient_count: usize,
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
                    retry_timeline_id: None,
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

    pub fn update_peer_broadcast(&self, peer: PeerNetworkId, timeline_id: u64) {
        self.peer_info
            .lock()
            .expect("failed to acquire peer_info lock")
            .entry(peer)
            .and_modify(|t| {
                t.timeline_id = timeline_id;
            });
    }

    pub fn set_retry(&self, peer: PeerNetworkId, new_retry_id: u64) {
        self.peer_info
            .lock()
            .expect("failed to acquire peer_info lock")
            .entry(peer)
            .and_modify(|t| {
                if let Some(retry_id) = t.retry_timeline_id {
                    t.retry_timeline_id = Some(std::cmp::min(retry_id, new_retry_id));
                } else {
                    t.retry_timeline_id = Some(new_retry_id);
                }
            });
    }

    // call this when we receive successful ACK for the retry batch
    pub fn clear_retry(&self, peer: PeerNetworkId, acked_timeline_id: u64, new_timeline_id: u64) {
        self.peer_info
            .lock()
            .expect("failed to acquire peer_info lock")
            .entry(peer)
            .and_modify(|t| {
                if let Some(pending_retry_timeline) = t.retry_timeline_id {
                    if acked_timeline_id == pending_retry_timeline {
                        t.retry_timeline_id = None;
                        t.timeline_id = new_timeline_id;
                    }
                }
            });
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
