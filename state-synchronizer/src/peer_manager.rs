// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use libra_config::config::{PeerNetworkId, UpstreamConfig};
use libra_logger::prelude::*;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::{BTreeMap, HashMap},
    time::SystemTime,
};

const MAX_SCORE: f64 = 100.0;
const MIN_SCORE: f64 = 1.0;

#[derive(Default, Debug, Clone)]
pub struct PeerInfo {
    is_alive: bool,
    score: f64,
}

impl PeerInfo {
    pub fn new(is_alive: bool, score: f64) -> Self {
        Self { is_alive, score }
    }
}

/// Basic metadata about the chunk request.
#[derive(Debug, Clone)]
pub struct ChunkRequestInfo {
    version: u64,
    first_request_time: SystemTime,
    last_request_time: SystemTime,
    last_request_peer: PeerNetworkId,
}

impl ChunkRequestInfo {
    pub fn new(version: u64, peer: PeerNetworkId) -> Self {
        let now = SystemTime::now();
        Self {
            version,
            first_request_time: now,
            last_request_time: now,
            last_request_peer: peer,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerScoreUpdateType {
    Success,
    EmptyChunk,
    // A received chunk cannot be directly applied (old / wrong version). Note that it could happen
    // that a peer would first timeout and would then be punished with ChunkVersionCannotBeApplied.
    ChunkVersionCannotBeApplied,
    InvalidChunk,
    TimeOut,
}

pub struct PeerManager {
    peers: HashMap<PeerNetworkId, PeerInfo>,
    requests: BTreeMap<u64, ChunkRequestInfo>,
    weighted_index: Option<WeightedIndex<f64>>,
    upstream_config: UpstreamConfig,
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig) -> Self {
        Self {
            peers: HashMap::new(),
            requests: BTreeMap::new(),
            weighted_index: None,
            upstream_config,
        }
    }

    pub fn enable_peer(&mut self, peer: PeerNetworkId) {
        if !self.is_upstream_peer(peer) {
            return;
        }

        debug!("[state sync] state before: {:?}", self.peers);
        if let Some(peer_info) = self.peers.get_mut(&peer) {
            peer_info.is_alive = true;
        } else {
            self.peers.insert(peer, PeerInfo::new(true, MAX_SCORE));
        }
        self.compute_weighted_index();
        debug!("[state sync] state after: {:?}", self.peers);
    }

    pub fn disable_peer(&mut self, peer: &PeerNetworkId) {
        if let Some(peer_info) = self.peers.get_mut(peer) {
            peer_info.is_alive = false;
        };
        self.compute_weighted_index();
    }

    pub fn is_empty(&self) -> bool {
        self.get_active_upstream_peers().is_empty()
    }

    pub fn update_score(&mut self, peer: &PeerNetworkId, update_type: PeerScoreUpdateType) {
        if let Some(peer_info) = self.peers.get_mut(peer) {
            let old_score = peer_info.score;
            match update_type {
                PeerScoreUpdateType::Success => {
                    let new_score = peer_info.score + 1.0;
                    peer_info.score = new_score.min(MAX_SCORE);
                }
                PeerScoreUpdateType::InvalidChunk
                | PeerScoreUpdateType::ChunkVersionCannotBeApplied => {
                    let new_score = peer_info.score * 0.8;
                    peer_info.score = new_score.max(MIN_SCORE);
                }
                PeerScoreUpdateType::TimeOut | PeerScoreUpdateType::EmptyChunk => {
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

    pub fn pick_peer(&self) -> Option<PeerNetworkId> {
        let active_peers = self.get_active_upstream_peers();
        debug!("[state sync] (pick_peer) state: {:?}", self.peers);

        if let Some(weighted_index) = &self.weighted_index {
            let mut rng = thread_rng();
            if let Some(peer) = active_peers.get(weighted_index.sample(&mut rng)) {
                return Some(*peer.0);
            }
        }
        None
    }

    fn get_active_upstream_peers(&self) -> Vec<(&PeerNetworkId, &PeerInfo)> {
        self.peers
            .iter()
            .filter(|&(peer, peer_info)| peer_info.is_alive && self.is_upstream_peer(*peer))
            .collect()
    }

    pub fn process_request(&mut self, version: u64, peer: PeerNetworkId) {
        if let Some(prev_request) = self.requests.get_mut(&version) {
            prev_request.last_request_peer = peer;
            prev_request.last_request_time = SystemTime::now();
        } else {
            self.requests
                .insert(version, ChunkRequestInfo::new(version, peer));
        }
    }

    pub fn get_last_request_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.last_request_time)
    }

    pub fn get_first_request_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.first_request_time)
    }

    pub fn remove_requests(&mut self, version: u64) {
        self.requests = self.requests.split_off(&(version + 1));
    }

    pub fn process_timeout(&mut self, version: u64, penalize: bool) {
        if !penalize {
            return;
        }
        let peer_to_penalize = match self.requests.get(&version) {
            Some(prev_request) => prev_request.last_request_peer,
            None => {
                return;
            }
        };

        self.update_score(&peer_to_penalize, PeerScoreUpdateType::TimeOut);
    }

    fn is_upstream_peer(&self, peer: PeerNetworkId) -> bool {
        self.upstream_config
            .is_upstream_peer(peer.network_id(), peer.peer_id())
    }

    #[cfg(test)]
    pub fn peer_score(&self, peer: &PeerNetworkId) -> Option<f64> {
        self.peers.get(peer).map(|p| p.score)
    }
}
