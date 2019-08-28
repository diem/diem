// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use logger::prelude::*;
use network::validator_network::StateSynchronizerSender;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct PeerInfo {
    is_alive: bool,
    is_upstream: bool,
}

impl PeerInfo {
    pub fn new(is_alive: bool, is_upstream: bool) -> Self {
        Self {
            is_alive,
            is_upstream,
        }
    }
}

pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    network_senders: HashMap<PeerId, StateSynchronizerSender>,
}

impl PeerManager {
    pub fn new(peer_ids: Vec<PeerId>) -> Self {
        let peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::new(false, true)))
            .collect();
        Self {
            peers,
            network_senders: HashMap::new(),
        }
    }

    pub fn set_peers(&mut self, peer_ids: Vec<PeerId>) {
        let new_peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::new(true, true)))
            .collect();
        self.peers = new_peers;
        debug!("[state sync] (set_peers) state: {:?}", self.peers);
    }

    pub fn enable_peer(&mut self, peer_id: PeerId, sender: StateSynchronizerSender) {
        debug!("[state sync] state before: {:?}", self.peers);
        self.network_senders.insert(peer_id, sender);
        if let Some(peer_info) = self.peers.get_mut(&peer_id) {
            peer_info.is_alive = true;
        } else {
            self.peers.insert(peer_id, PeerInfo::new(true, false));
        }
        debug!("[state sync] state after: {:?}", self.peers);
    }

    pub fn disable_peer(&mut self, peer_id: &PeerId) {
        self.network_senders.remove(&peer_id);
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.is_alive = false;
        };
    }

    pub fn is_empty(&self) -> bool {
        self.get_active_upstream_peer_ids().is_empty()
    }

    pub fn pick_peer(&mut self) -> Option<(PeerId, StateSynchronizerSender)> {
        let active_peers = self.get_active_upstream_peer_ids();
        debug!("[state sync] (pick_peer) state: {:?}", self.peers);
        if !active_peers.is_empty() {
            let idx = thread_rng().gen_range(0, active_peers.len());
            let peer_id = *active_peers[idx];
            if let Some(sender) = self.get_network_sender(&peer_id) {
                return Some((peer_id, sender));
            } else {
                debug!("[state sync] (pick_peer) no sender for {}", peer_id);
            }
        }
        None
    }

    fn get_active_upstream_peer_ids(&self) -> Vec<&PeerId> {
        debug!(
            "[state sync] (get_active_upstream_peer_ids) state: {:?}",
            self.peers
        );
        self.peers
            .iter()
            .filter(|&(_, peer_info)| peer_info.is_alive && peer_info.is_upstream)
            .map(|(peer_id, _)| peer_id)
            .collect()
    }

    pub fn get_network_sender(&self, peer_id: &PeerId) -> Option<StateSynchronizerSender> {
        self.network_senders.get(peer_id).cloned()
    }
}
