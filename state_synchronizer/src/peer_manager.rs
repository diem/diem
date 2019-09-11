// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use logger::prelude::*;
use network::validator_network::StateSynchronizerSender;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

const MAX_SCORE: f64 = 100.0;
const MIN_SCORE: f64 = 1.0;

#[derive(Default, Debug)]
pub struct PeerInfo {
    is_alive: bool,
    is_upstream: bool,
    score: f64,
}

impl PeerInfo {
    pub fn new(is_alive: bool, is_upstream: bool, score: f64) -> Self {
        Self {
            is_alive,
            is_upstream,
            score,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerScoreUpdateType {
    Success,
    InvalidChunk,
    TimeOut,
}

pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    network_senders: HashMap<PeerId, StateSynchronizerSender>,
    // Latest requested block versions from a peer
    requests: HashMap<u64, (PeerId, SystemTime)>,
}

impl PeerManager {
    pub fn new(peer_ids: Vec<PeerId>) -> Self {
        let peers = peer_ids
            .into_iter()
            .map(|peer_id| (peer_id, PeerInfo::new(false, true, MAX_SCORE)))
            .collect();
        Self {
            peers,
            network_senders: HashMap::new(),
            requests: HashMap::new(),
        }
    }

    pub fn set_peers(&mut self, peer_ids: Vec<PeerId>) {
        let new_peer_ids: HashSet<_> = peer_ids.iter().collect();
        for (peer_id, info) in self.peers.iter_mut() {
            info.is_upstream = new_peer_ids.contains(peer_id);
        }
        for peer_id in new_peer_ids {
            if !self.peers.contains_key(peer_id) {
                self.peers
                    .insert(*peer_id, PeerInfo::new(false, true, MAX_SCORE));
            }
        }
        debug!("[state sync] (set_peers) state: {:?}", self.peers);
    }

    pub fn enable_peer(&mut self, peer_id: PeerId, sender: StateSynchronizerSender) {
        debug!("[state sync] state before: {:?}", self.peers);
        self.network_senders.insert(peer_id, sender);
        if let Some(peer_info) = self.peers.get_mut(&peer_id) {
            peer_info.is_alive = true;
        } else {
            self.peers
                .insert(peer_id, PeerInfo::new(true, false, MAX_SCORE));
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
        self.get_active_upstream_peers().is_empty()
    }

    pub fn update_score(&mut self, peer_id: &PeerId, update_type: PeerScoreUpdateType) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            match update_type {
                PeerScoreUpdateType::Success => {
                    let new_score = peer_info.score + 1.0;
                    peer_info.score = new_score.min(MAX_SCORE);
                }
                PeerScoreUpdateType::InvalidChunk => {
                    let new_score = peer_info.score * 0.8;
                    peer_info.score = new_score.max(MIN_SCORE);
                }
                PeerScoreUpdateType::TimeOut => {
                    let new_score = peer_info.score * 0.95;
                    peer_info.score = new_score.max(MIN_SCORE);
                }
            }
        }
    }

    pub fn pick_peer(&mut self) -> Option<(PeerId, StateSynchronizerSender)> {
        let active_peers = self.get_active_upstream_peers();
        debug!("[state sync] (pick_peer) state: {:?}", self.peers);

        if !active_peers.is_empty() {
            let weights: Vec<_> = active_peers
                .iter()
                .map(|(_, peer_info)| peer_info.score)
                .collect();

            if let Ok(weighted_index) = WeightedIndex::new(&weights) {
                let mut rng = thread_rng();
                let peer_id = *active_peers[weighted_index.sample(&mut rng)].0;

                match self.get_network_sender(&peer_id) {
                    Some(sender) => {
                        return Some((peer_id, sender));
                    }
                    None => {
                        debug!("[state sync] (pick_peer) no sender for {}", peer_id);
                    }
                }
            } else {
                error!("[state sync] (pick_peer) invalid weighted index distribution");
            }
        }
        None
    }

    fn get_active_upstream_peers(&self) -> Vec<(&PeerId, &PeerInfo)> {
        self.peers
            .iter()
            .filter(|&(_, peer_info)| peer_info.is_alive && peer_info.is_upstream)
            .collect()
    }

    pub fn get_network_sender(&self, peer_id: &PeerId) -> Option<StateSynchronizerSender> {
        self.network_senders.get(peer_id).cloned()
    }

    pub fn process_request(&mut self, version: u64, peer_id: PeerId) {
        self.requests.insert(version, (peer_id, SystemTime::now()));
    }

    pub fn process_response(&mut self, version: u64, peer_id: PeerId) {
        if let Some((id, _)) = self.requests.get(&version) {
            if *id == peer_id {
                self.requests.remove(&version);
            }
        }
    }

    pub fn process_timeout(&mut self, current_requested_version: u64, timeout: u64) {
        let request = self.requests.get(&current_requested_version).cloned();
        if let Some((peer_id, request_time)) = request {
            if let Some(timeout_threshold) =
                request_time.checked_add(Duration::from_millis(timeout))
            {
                if SystemTime::now().duration_since(timeout_threshold).is_ok() {
                    self.update_score(&peer_id, PeerScoreUpdateType::TimeOut);
                    self.requests.remove(&current_requested_version);
                }
            }
        }
    }
}
