// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use itertools::Itertools;
use libra_config::{
    config::{PeerNetworkId, UpstreamConfig},
    network_id::NetworkId,
};
use libra_logger::prelude::*;
use netcore::transport::ConnectionOrigin;
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
    // list of peers that are eligible for this node to send sync requests to
    eligible_peers: Vec<PeerNetworkId>,
    peers: HashMap<PeerNetworkId, PeerInfo>,
    requests: BTreeMap<u64, ChunkRequestInfo>,
    upstream_config: UpstreamConfig,
    weighted_index: Option<WeightedIndex<f64>>,
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig) -> Self {
        Self {
            eligible_peers: vec![],
            peers: HashMap::new(),
            requests: BTreeMap::new(),
            upstream_config,
            weighted_index: None,
        }
    }

    pub fn enable_peer(&mut self, peer: PeerNetworkId, origin: ConnectionOrigin) {
        if !self.is_upstream_peer(&peer, origin) {
            return;
        }

        debug!("[state sync] state before: {:?}", self.peers);
        if let Some(peer_info) = self.peers.get_mut(&peer) {
            peer_info.is_alive = true;
        } else {
            self.peers.insert(peer, PeerInfo::new(true, MAX_SCORE));
        }
        self.update_peer_selection_data();
        debug!("[state sync] state after: {:?}", self.peers);
    }

    pub fn disable_peer(&mut self, peer: &PeerNetworkId) {
        if let Some(peer_info) = self.peers.get_mut(peer) {
            peer_info.is_alive = false;
        };
        self.update_peer_selection_data();
    }

    pub fn is_empty(&self) -> bool {
        self.eligible_peers.is_empty()
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
                self.update_peer_selection_data();
            }
        }
    }

    // Updates the information used to select a peer to send a chunk request to:
    // * eligible_peers
    // * weighted_index: the chance that a peer is selected from `eligible_peers` is weighted by its score
    fn update_peer_selection_data(&mut self) {
        let active_peers = self.get_active_upstream_peers();
        counters::ACTIVE_UPSTREAM_PEERS.set(active_peers.len() as i64);

        // compute weighted index based on updated eligible peers
        let mut eligible_peers = vec![];
        let weights: Vec<_> = active_peers
            .into_iter()
            .map(|(peer, peer_info)| {
                eligible_peers.push(peer.clone());
                peer_info.score
            })
            .collect();
        self.eligible_peers = eligible_peers;
        self.weighted_index = WeightedIndex::new(&weights)
            .map_err(|err| {
                error!(
                    "[state sync] (pick_peer) failed to compute weighted index, {:?}",
                    err
                );
                err
            })
            .ok();
    }

    pub fn pick_peer(&self) -> Option<PeerNetworkId> {
        if let Some(weighted_index) = &self.weighted_index {
            let mut rng = thread_rng();
            if let Some(peer) = self.eligible_peers.get(weighted_index.sample(&mut rng)) {
                return Some(peer.clone());
            }
        }
        None
    }

    fn get_active_upstream_peers(&self) -> Vec<(&PeerNetworkId, &PeerInfo)> {
        if self.upstream_config.networks.len() > 1 {
            // failover mode is enabled only if there are multiple upstream networks
            // in failover mode, we select the network of the highest preference (defined by UpstreamConfig)
            // with at least one live peer
            // We failover to the next network (in order of preference defined by UpstreamConfig) if
            // there are no live peers in each network

            // group active upstream peers by network
            let active_peers_by_network = self
                .peers
                .iter()
                .filter(|(_peer, peer_info)| peer_info.is_alive)
                .map(|(peer, peer_info)| (peer.raw_network_id(), (peer, peer_info)))
                .into_group_map();

            // find the first network with any live peers
            self.upstream_config
                .networks
                .iter()
                .find_map(|network| active_peers_by_network.get(network))
                .unwrap_or(&vec![])
                .to_vec()
        } else {
            // no failover option
            // all upstream peers belong to the same network
            self.peers
                .iter()
                .filter(|&(_peer, peer_info)| peer_info.is_alive)
                .collect()
        }
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

    pub fn process_timeout(&mut self, version: u64) {
        let peer_to_penalize = match self.requests.get(&version) {
            Some(prev_request) => prev_request.last_request_peer.clone(),
            None => {
                return;
            }
        };

        self.update_score(&peer_to_penalize, PeerScoreUpdateType::TimeOut);
    }

    fn is_upstream_peer(&self, peer: &PeerNetworkId, origin: ConnectionOrigin) -> bool {
        let is_network_upstream = self
            .upstream_config
            .get_upstream_preference(peer.raw_network_id())
            .is_some();
        // check for case whether the peer is a public downstream peer, even if the public network is upstream
        if is_network_upstream && peer.raw_network_id() == NetworkId::Public {
            origin == ConnectionOrigin::Outbound
        } else {
            is_network_upstream
        }
    }

    #[cfg(test)]
    pub fn peer_score(&self, peer: &PeerNetworkId) -> Option<f64> {
        self.peers.get(peer).map(|p| p.score)
    }
}
