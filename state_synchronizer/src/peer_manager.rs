// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
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
}

impl PeerManager {
    pub fn new(peer_ids: Vec<PeerId>) -> Self {
        let upstream_peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::default()))
            .collect();
        Self { upstream_peers }
    }

    pub fn set_peers(&mut self, peer_ids: Vec<PeerId>) {
        let new_peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::new(true)))
            .collect();
        self.upstream_peers = new_peers;
    }

    pub fn enable_peer(&mut self, peer_id: &PeerId) {
        if let Some(peer_info) = self.upstream_peers.get_mut(peer_id) {
            peer_info.is_alive = true;
        };
    }

    pub fn disable_peer(&mut self, peer_id: &PeerId) {
        if let Some(peer_info) = self.upstream_peers.get_mut(peer_id) {
            peer_info.is_alive = false;
        };
    }

    pub fn is_empty(&self) -> bool {
        self.get_active_peer_ids().is_empty()
    }

    pub fn pick_peer(&mut self) -> Option<AccountAddress> {
        let active_peers = self.get_active_peer_ids();
        if !active_peers.is_empty() {
            let idx = thread_rng().gen_range(0, active_peers.len());
            Some(*active_peers[idx])
        } else {
            None
        }
    }

    fn get_active_peer_ids(&self) -> Vec<&AccountAddress> {
        self.upstream_peers
            .iter()
            .filter(|&(_, peer_info)| peer_info.is_alive)
            .map(|(peer_id, _)| peer_id)
            .collect()
    }
}
