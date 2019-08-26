// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use network::validator_network::StateSynchronizerSender;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use types::account_address::AccountAddress;

#[derive(Default)]
pub struct PeerInfo {
    is_alive: bool,
}

impl PeerInfo {
    pub fn new(is_alive: bool) -> Self {
        Self { is_alive }
    }
}

pub struct PeerManager {
    upstream_peers: HashMap<PeerId, PeerInfo>,
    network_senders: HashMap<PeerId, StateSynchronizerSender>,
}

impl PeerManager {
    pub fn new(peer_ids: Vec<PeerId>) -> Self {
        let upstream_peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::default()))
            .collect();
        Self {
            upstream_peers,
            network_senders: HashMap::new(),
        }
    }

    pub fn set_peers(&mut self, peer_ids: Vec<PeerId>) {
        let new_peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::new(true)))
            .collect();
        self.upstream_peers = new_peers;
    }

    pub fn enable_peer(&mut self, peer_id: PeerId, sender: StateSynchronizerSender) {
        self.network_senders.insert(peer_id, sender);
        if let Some(peer_info) = self.upstream_peers.get_mut(&peer_id) {
            peer_info.is_alive = true;
        };
    }

    pub fn disable_peer(&mut self, peer_id: &PeerId) {
        self.network_senders.remove(&peer_id);
        if let Some(peer_info) = self.upstream_peers.get_mut(peer_id) {
            peer_info.is_alive = false;
        };
    }

    pub fn is_empty(&self) -> bool {
        self.get_active_peer_ids().is_empty()
    }

    pub fn pick_peer(&mut self) -> Option<(AccountAddress, StateSynchronizerSender)> {
        let active_peers = self.get_active_peer_ids();
        if !active_peers.is_empty() {
            let idx = thread_rng().gen_range(0, active_peers.len());
            let peer_id = *active_peers[idx];
            if let Some(sender) = self.get_network_sender(&peer_id) {
                return Some((peer_id, sender));
            }
        }
        None
    }

    fn get_active_peer_ids(&self) -> Vec<&AccountAddress> {
        self.upstream_peers
            .iter()
            .filter(|&(_, peer_info)| peer_info.is_alive)
            .map(|(peer_id, _)| peer_id)
            .collect()
    }

    pub fn get_network_sender(&self, peer_id: &PeerId) -> Option<StateSynchronizerSender> {
        self.network_senders.get(peer_id).cloned()
    }
}
