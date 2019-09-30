// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, PeerId};
use libra_logger::prelude::*;
use libra_network::validator_network::StateSynchronizerSender;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::SystemTime,
};

const MAX_SCORE: f64 = 100.0;
const MIN_SCORE: f64 = 1.0;

#[derive(Default, Debug, Clone)]
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
    requests: BTreeMap<u64, (PeerId, SystemTime)>,
    weighted_index: Option<WeightedIndex<f64>>,
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
            requests: BTreeMap::new(),
            weighted_index: None,
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
        self.compute_weighted_index();
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
        self.compute_weighted_index();
        debug!("[state sync] state after: {:?}", self.peers);
    }

    pub fn disable_peer(&mut self, peer_id: &PeerId) {
        self.network_senders.remove(&peer_id);
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.is_alive = false;
        };
        self.compute_weighted_index();
    }

    pub fn is_empty(&self) -> bool {
        self.get_active_upstream_peers().is_empty()
    }

    pub fn update_score(&mut self, peer_id: &PeerId, update_type: PeerScoreUpdateType) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            let old_score = peer_info.score;
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
            if (old_score - peer_info.score).abs() > std::f64::EPSILON {
                self.compute_weighted_index();
            }
        }
    }

    fn compute_weighted_index(&mut self) {
        let active_peers = self.get_active_upstream_peers();
        counters::ACTIVE_UPSTREAM_PEERS.set(active_peers.len() as i64);

        if !active_peers.is_empty() {
            let weights: Vec<_> = active_peers
                .iter()
                .map(|(_, peer_info)| peer_info.score)
                .collect();
            match WeightedIndex::new(&weights) {
                Ok(weighted_index) => {
                    self.weighted_index = Some(weighted_index);
                }
                Err(e) => {
                    error!(
                        "[state sync] (pick_peer) failed to compute weighted index, {:?}",
                        e
                    );
                }
            }
        }
    }

    pub fn pick_peer(&self) -> Option<(PeerId, StateSynchronizerSender)> {
        let active_peers = self.get_active_upstream_peers();
        debug!("[state sync] (pick_peer) state: {:?}", self.peers);

        if let Some(weighted_index) = &self.weighted_index {
            let mut rng = thread_rng();
            if let Some(peer) = active_peers.get(weighted_index.sample(&mut rng)) {
                let peer_id = *peer.0;
                if let Some(sender) = self.get_network_sender(&peer_id) {
                    return Some((peer_id, sender));
                } else {
                    debug!("[state sync] (pick_peer) no sender for {}", peer_id);
                }
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

    pub fn get_request_time(&self, version: u64) -> Option<SystemTime> {
        self.requests.get(&version).map(|(_, tst)| tst).cloned()
    }

    pub fn process_response(&mut self, version: u64, peer_id: PeerId) {
        if let Some((id, _)) = self.requests.get(&version) {
            if *id == peer_id {
                self.requests.remove(&version);
            }
        }
    }

    pub fn has_requested(&self, version: u64, peer_id: PeerId) -> bool {
        if let Some((id, _)) = self.requests.get(&version) {
            return *id == peer_id;
        }
        false
    }

    pub fn remove_requests(&mut self, version: u64) {
        self.requests = self.requests.split_off(&(version + 1));
    }

    pub fn process_timeout(&mut self, version: u64, penalize: bool) {
        if let Some((peer_id, _)) = self.requests.remove(&version) {
            if penalize {
                self.update_score(&peer_id, PeerScoreUpdateType::TimeOut);
            }
        }
    }
}
