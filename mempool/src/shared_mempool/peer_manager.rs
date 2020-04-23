// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::UpstreamConfig;
use libra_types::PeerId;
use rand::seq::IteratorRandom;
use std::{collections::HashMap, sync::Mutex};

/// stores only peers that receive txns from this node
pub(crate) type PeerInfo = HashMap<PeerId, PeerSyncState>;

/// state of last sync with peer
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
/// `network_id` - ID of the mempool network that this peer belongs to
#[derive(Clone)]
pub(crate) struct PeerSyncState {
    pub timeline_id: u64,
    pub is_alive: bool,
    pub network_id: PeerId,
}

pub(crate) struct PeerManager {
    upstream_config: UpstreamConfig,
    peer_info: Mutex<PeerInfo>,
    min_broadcast_recipient_count: usize,
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig, min_broadcast_recipient_count: usize) -> Self {
        let peer_info = Mutex::new(PeerInfo::new());
        Self {
            upstream_config,
            peer_info,
            min_broadcast_recipient_count,
        }
    }

    pub fn add_peer(&self, network_id: PeerId, peer_id: PeerId) {
        if self.is_upstream_peer(network_id, peer_id) {
            self.peer_info
                .lock()
                .expect("failed to acquire peer info lock")
                .entry(peer_id)
                .or_insert(PeerSyncState {
                    timeline_id: 0,
                    is_alive: true,
                    network_id,
                })
                .is_alive = true;
        }
    }

    pub fn disable_peer(&self, peer_id: PeerId) {
        if let Some(state) = self
            .peer_info
            .lock()
            .expect("failed to acquire peer info lock")
            .get_mut(&peer_id)
        {
            state.is_alive = false;
        }
    }

    // pick_peers
    // picks peers to broadcast to. Will first try to pick *all* preferred peers (= upstream as defined by config)
    // if preferred upstream peers < `k = min_broadcast_recipient_count`, pick fallback peers to fill up k
    pub fn pick_peers(&self) -> PeerInfo {
        let peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer info lock");
        // pick all preferred peers
        let mut picked_peers: PeerInfo = peer_info
            .iter()
            .filter(|(peer, state)| {
                state.is_alive
                    && self
                        .upstream_config
                        .is_primary_upstream_peer(state.network_id, **peer)
            })
            .map(|(peer, state)| (*peer, state.clone()))
            .collect();

        let picked_peers_count = picked_peers.len();
        if picked_peers_count < self.min_broadcast_recipient_count {
            // randomly select fallback peers
            // TODO add peer scoring schema to use for selecting fallback peers
            let fallback_peers: PeerInfo = peer_info
                .iter()
                .filter(|(peer, state)| !picked_peers.contains_key(&peer) && state.is_alive)
                .map(|(peer, state)| (*peer, state.clone()))
                .choose_multiple(
                    &mut rand::thread_rng(),
                    self.min_broadcast_recipient_count - picked_peers_count,
                )
                .into_iter()
                .collect();

            picked_peers.extend(fallback_peers);
        }
        picked_peers
    }

    pub fn update_peer_broadcast(&self, peer_broadcasts: Vec<(PeerId, u64)>) {
        let mut peer_info = self
            .peer_info
            .lock()
            .expect("failed to acquire peer_info lock");

        for (peer_id, new_timeline_id) in peer_broadcasts {
            peer_info.entry(peer_id).and_modify(|t| {
                t.timeline_id = new_timeline_id;
            });
        }
    }

    pub fn is_upstream_peer(&self, network_id: PeerId, peer_id: PeerId) -> bool {
        self.upstream_config.is_upstream_peer(network_id, peer_id)
    }
}
