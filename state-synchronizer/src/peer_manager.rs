// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use itertools::Itertools;
use libra_config::{
    config::{PeerNetworkId, UpstreamConfig},
    network_id::{NetworkId, NodeNetworkId},
};
use libra_logger::prelude::*;
use netcore::transport::ConnectionOrigin;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::{Duration, SystemTime},
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
    multicast_start_time: SystemTime,
    last_request_peers: Vec<PeerNetworkId>,
}

impl ChunkRequestInfo {
    pub fn new(version: u64, peers: Vec<PeerNetworkId>) -> Self {
        let now = SystemTime::now();
        Self {
            version,
            first_request_time: now,
            last_request_time: now,
            multicast_start_time: now,
            last_request_peers: peers,
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
    // grouped by network preference
    eligible_peers: BTreeMap<usize, (Vec<PeerNetworkId>, Option<WeightedIndex<f64>>)>,
    peers: HashMap<PeerNetworkId, PeerInfo>,
    requests: BTreeMap<u64, ChunkRequestInfo>,
    upstream_config: UpstreamConfig,
    // networks that timed out in helping this node make state sync progress
    // the same chunk request will be multicasted across all these networks and an additional fallback
    // if available, until we receive a successful chunk response from the primary network
    multicast_networks: Vec<NodeNetworkId>,
}

impl PeerManager {
    pub fn new(upstream_config: UpstreamConfig) -> Self {
        Self {
            eligible_peers: BTreeMap::new(),
            peers: HashMap::new(),
            requests: BTreeMap::new(),
            upstream_config,
            multicast_networks: vec![],
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
        let mut active_peers_count = 0;
        // group active peers by network
        let active_peers = self
            .peers
            .iter()
            .filter(|(_peer, peer_info)| peer_info.is_alive)
            .map(|(peer, peer_info)| {
                active_peers_count += 1;
                let network_pref = self
                    .upstream_config
                    .get_upstream_preference(peer.raw_network_id())
                    .unwrap();
                (network_pref, (peer, peer_info))
            })
            .into_group_map();

        counters::ACTIVE_UPSTREAM_PEERS.set(active_peers_count);

        // for each network, compute peer selection data
        self.eligible_peers = active_peers
            .into_iter()
            .map(|(network_pref, peers)| {
                let mut eligible_peers = vec![];
                let weights: Vec<_> = peers
                    .iter()
                    .map(|(peer, peer_info)| {
                        eligible_peers.push((*peer).clone());
                        peer_info.score
                    })
                    .collect();
                let weighted_index = WeightedIndex::new(&weights)
                    .map_err(|err| {
                        error!(
                            "[state sync] (pick_peer) failed to compute weighted index, {:?}",
                            err
                        );
                        err
                    })
                    .ok();
                (network_pref, (eligible_peers, weighted_index))
            })
            .collect();
    }

    fn pick_peer(
        peers: &[PeerNetworkId],
        weighted_index: &Option<WeightedIndex<f64>>,
    ) -> Option<PeerNetworkId> {
        if let Some(weighted_index) = &weighted_index {
            let mut rng = thread_rng();
            if let Some(peer) = peers.get(weighted_index.sample(&mut rng)) {
                return Some(peer.clone());
            }
        }
        None
    }

    pub fn pick_peers(&self) -> Vec<PeerNetworkId> {
        let mut picked_peers = vec![];
        if self.multicast_networks.is_empty() {
            // pick from first network with available peers
            if let Some((_key, (active_peers, weighted_index))) = self.eligible_peers.iter().next()
            {
                if let Some(peer) = Self::pick_peer(active_peers, weighted_index) {
                    picked_peers.push(peer);
                }
            }
        } else {
            let multicast_network_prefs = self
                .multicast_networks
                .iter()
                .filter_map(|network| {
                    self.upstream_config
                        .get_upstream_preference(network.network_id())
                })
                .collect::<HashSet<_>>();

            picked_peers = multicast_network_prefs
                .iter()
                .filter_map(|network_pref| {
                    self.eligible_peers
                        .get(&network_pref)
                        .and_then(|(peers, weighted_index)| Self::pick_peer(peers, weighted_index))
                })
                .collect::<Vec<_>>();

            // try picking another network's peer other than multicast_networks, if available
            if let Some(failover_peer) = self
                .eligible_peers
                .iter()
                .find(|(network_pref, _peers)| !multicast_network_prefs.contains(network_pref))
                .and_then(|(_network_pref, (peers, weighted_index))| {
                    Self::pick_peer(peers, weighted_index)
                })
            {
                picked_peers.push(failover_peer);
            }
        }

        picked_peers
    }

    pub fn process_request(&mut self, version: u64, peers: Vec<PeerNetworkId>) {
        if let Some(prev_request) = self.requests.get_mut(&version) {
            let now = SystemTime::now();
            if peers.len() > prev_request.last_request_peers.len() {
                // update multicast start time if we are sending the request to more peers
                prev_request.multicast_start_time = now;
            }
            prev_request.last_request_peers = peers;
            prev_request.last_request_time = now;
        } else {
            self.requests
                .insert(version, ChunkRequestInfo::new(version, peers));
        }
    }

    pub fn process_success_response(&mut self, peer: &PeerNetworkId) {
        // update multicast
        let is_primary_upstream_peer = self
            .upstream_config
            .get_upstream_preference(peer.raw_network_id())
            == Some(0);
        if is_primary_upstream_peer {
            // if chunk from a primary upstream is successful, stop multicasting the request to failover networks
            self.multicast_networks = vec![];
        }

        // update score
        self.update_score(peer, PeerScoreUpdateType::Success);
    }

    pub fn is_multicast_response(&self, version: u64, peer: &PeerNetworkId) -> bool {
        self.requests.get(&version).map_or(false, |req| {
            req.last_request_peers.contains(peer) && req.last_request_peers.len() > 1
        })
    }

    pub fn get_last_request_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.last_request_time)
    }

    pub fn get_multicast_start_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.multicast_start_time)
    }

    pub fn get_first_request_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.first_request_time)
    }

    /// Removes requests for all versions before `version` (inclusive) if they are older than
    /// now - `multicast_timeout`
    /// We keep the requests that have not timed out for multicasting so we don't penalize
    /// peers who send chunks after the first peer who sends the first successful chunk response for a
    /// given request
    pub fn remove_requests(&mut self, version: u64, multicast_timeout: Duration) {
        // only remove requests that have multicast-timed out, so we don't penalize for multicasted responses
        // that still came back on time
        let now = SystemTime::now();
        let versions_to_remove = self
            .requests
            .range(..version + 1)
            .filter_map(|(version, req)| {
                let is_multicast_timeout = req
                    .last_request_time
                    .checked_add(multicast_timeout)
                    .map_or(false, |retry_deadline| {
                        now.duration_since(retry_deadline).is_ok()
                    });
                if is_multicast_timeout {
                    Some(*version)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for v in versions_to_remove {
            self.requests.remove(&v);
        }
    }

    pub fn process_timeout(&mut self, version: u64, is_multicast_timeout: bool) {
        let peers_to_penalize = match self.requests.get(&version) {
            Some(prev_request) => prev_request.last_request_peers.clone(),
            None => {
                return;
            }
        };
        for peer in peers_to_penalize.iter() {
            self.update_score(peer, PeerScoreUpdateType::TimeOut);
        }

        // TODO make sure is_multicast_timeout means the correct thing
        // may have to track per network
        if is_multicast_timeout {
            self.multicast_networks = peers_to_penalize
                .into_iter()
                .map(|peer| peer.network_id())
                .collect();
        }
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
