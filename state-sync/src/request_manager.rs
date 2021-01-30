// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::GetChunkRequest,
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    network::{StateSyncMessage, StateSyncSender},
};
use anyhow::{bail, format_err, Result};
use diem_config::{
    config::{PeerNetworkId, UpstreamConfig},
    network_id::{NetworkId, NodeNetworkId},
};
use diem_logger::prelude::*;
use itertools::Itertools;
use netcore::transport::ConnectionOrigin;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_SCORE: f64 = 100.0;
const MIN_SCORE: f64 = 1.0;
const PRIMARY_NETWORK_PREFERENCE: usize = 0;

#[derive(Clone, Debug)]
struct PeerInfo {
    is_alive: bool,
    score: f64,
}

impl PeerInfo {
    pub fn new(is_alive: bool, score: f64) -> Self {
        Self { is_alive, score }
    }
}

/// Basic metadata about the chunk request.
#[derive(Clone, Debug)]
pub struct ChunkRequestInfo {
    version: u64,
    first_request_time: SystemTime,
    last_request_time: SystemTime,
    multicast_level: usize,
    multicast_start_time: SystemTime,
    last_request_peers: Vec<PeerNetworkId>,
}

impl ChunkRequestInfo {
    pub fn new(version: u64, peers: Vec<PeerNetworkId>, multicast_level: usize) -> Self {
        let now = SystemTime::now();
        Self {
            version,
            first_request_time: now,
            last_request_time: now,
            multicast_level,
            multicast_start_time: now,
            last_request_peers: peers,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PeerScoreUpdateType {
    Success,
    EmptyChunk,
    // A received chunk cannot be directly applied (old / wrong version). Note that it could happen
    // that a peer would first timeout and would then be punished with ChunkVersionCannotBeApplied.
    ChunkVersionCannotBeApplied,
    InvalidChunk,
    TimeOut,
}

pub struct RequestManager {
    // list of peers that are eligible for this node to send sync requests to
    // grouped by network preference
    eligible_peers: BTreeMap<usize, (Vec<PeerNetworkId>, Option<WeightedIndex<f64>>)>,
    peers: HashMap<PeerNetworkId, PeerInfo>,
    requests: BTreeMap<u64, ChunkRequestInfo>,
    upstream_config: UpstreamConfig,
    // duration with the same version before the next attempt to get the next chunk
    request_timeout: Duration,
    // duration with the same version before multicasting, i.e. sending the next chunk request to more networks
    multicast_timeout: Duration,
    // the maximum preference level of all the networks to try to multicast the same chunk request to,
    // where network preference is specified by the upstream config
    multicast_level: usize,
    network_senders: HashMap<NodeNetworkId, StateSyncSender>,
}

impl RequestManager {
    pub fn new(
        upstream_config: UpstreamConfig,
        request_timeout: Duration,
        multicast_timeout: Duration,
        network_senders: HashMap<NodeNetworkId, StateSyncSender>,
    ) -> Self {
        counters::MULTICAST_LEVEL.set(PRIMARY_NETWORK_PREFERENCE as i64);
        Self {
            eligible_peers: BTreeMap::new(),
            peers: HashMap::new(),
            requests: BTreeMap::new(),
            upstream_config,
            request_timeout,
            multicast_timeout,
            multicast_level: PRIMARY_NETWORK_PREFERENCE,
            network_senders,
        }
    }

    pub fn enable_peer(&mut self, peer: PeerNetworkId, origin: ConnectionOrigin) {
        let is_upstream_peer = self.is_upstream_peer(&peer, origin);
        debug!(LogSchema::new(LogEntry::NewPeer)
            .peer(&peer)
            .is_upstream_peer(is_upstream_peer));

        if !is_upstream_peer {
            return;
        }

        counters::ACTIVE_UPSTREAM_PEERS
            .with_label_values(&[&peer.raw_network_id().to_string()])
            .inc();
        if let Some(peer_info) = self.peers.get_mut(&peer) {
            peer_info.is_alive = true;
        } else {
            self.peers.insert(peer, PeerInfo::new(true, MAX_SCORE));
        }
        self.update_peer_selection_data();
    }

    pub fn disable_peer(&mut self, peer: &PeerNetworkId, origin: ConnectionOrigin) {
        debug!(LogSchema::new(LogEntry::LostPeer)
            .peer(&peer)
            .is_upstream_peer(self.is_upstream_peer(&peer, origin)));

        if let Some(peer_info) = self.peers.get_mut(peer) {
            counters::ACTIVE_UPSTREAM_PEERS
                .with_label_values(&[&peer.raw_network_id().to_string()])
                .dec();
            peer_info.is_alive = false;
        }
        self.update_peer_selection_data();
    }

    pub fn no_available_peers(&self) -> bool {
        self.eligible_peers.is_empty()
    }

    fn update_score(&mut self, peer: &PeerNetworkId, update_type: PeerScoreUpdateType) {
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
        // group active peers by network
        let active_peers = self
            .peers
            .iter()
            .filter(|(_peer, peer_info)| peer_info.is_alive)
            .map(|(peer, peer_info)| {
                let network_pref = self
                    .upstream_config
                    .get_upstream_preference(peer.raw_network_id())
                    .unwrap();
                (network_pref, (peer, peer_info))
            })
            .into_group_map();

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

    /// Picks a set of peers to send chunk requests to
    /// Tries to pick one peer per network, in order of network preference (where the higher the preference,
    /// the lower the value)
    /// The set of networks selected is determined by the multicast level. The preference level of the
    /// network is as defined in `UpstreamConfig` - 0 is the numeric value for the highest preference,
    /// and the larger this level, the less preferred this network is.
    /// All networks with preference level <= multicast level are first sampled. If there are no live peers in
    /// these networks, we find the next available network with preference level greater than the current multicast
    /// level with any live peers. If such a network is found, the multicast level is also updated to
    /// the preference level of the chosen network.
    pub fn pick_peers(&mut self) -> Vec<PeerNetworkId> {
        // Strategy: pick peers using multicast level
        // if no live peers exist for this multicast level, keep failing over to next level

        let mut chosen_peers = vec![];
        let mut new_multicast_level = None;
        for (level, (peers, weighted_index)) in self.eligible_peers.iter() {
            if let Some(peer) = pick_peer(peers, weighted_index) {
                chosen_peers.push(peer)
            }
            // at the minimum go through networks with preference level <= multicast_level
            // if no peers are found for the current multicast_level, continue doing
            // best effort search of the first network with live peers to failover to
            if !chosen_peers.is_empty() && *level >= self.multicast_level {
                new_multicast_level = Some(*level);
                break;
            }
        }

        // we call `update_multicast` here instead of before the break to avoid mutable borrow conflict
        // with the outer loop
        if let Some(level) = new_multicast_level {
            self.update_multicast(level, None);
        }
        chosen_peers
    }

    pub fn send_chunk_request(&mut self, req: GetChunkRequest) -> Result<()> {
        let log = LogSchema::new(LogEntry::SendChunkRequest).chunk_request(req.clone());

        // update internal state
        let peers = self.pick_peers();
        if peers.is_empty() {
            warn!(log.event(LogEvent::MissingPeers));
            bail!("No peers to send chunk request to");
        }

        let req_info = self.add_request(req.known_version, peers.clone());
        debug!(log
            .clone()
            .event(LogEvent::ChunkRequestInfo)
            .chunk_req_info(&req_info));

        let msg = StateSyncMessage::GetChunkRequest(Box::new(req));
        let mut failed_peer_sends = vec![];

        for peer in peers {
            let sender = self
                .network_senders
                .get_mut(&peer.network_id())
                .expect("missing network sender for peer");
            let peer_id = peer.peer_id();
            let send_result = sender.send_to(peer_id, msg.clone());
            let curr_log = log.clone().peer(&peer);
            let result_label = if let Err(e) = send_result {
                failed_peer_sends.push(peer.clone());
                error!(curr_log.event(LogEvent::NetworkSendError).error(&e.into()));
                counters::SEND_FAIL_LABEL
            } else {
                debug!(curr_log.event(LogEvent::Success));
                counters::SEND_SUCCESS_LABEL
            };
            counters::REQUESTS_SENT
                .with_label_values(&[
                    &peer.raw_network_id().to_string(),
                    &peer_id.to_string(),
                    result_label,
                ])
                .inc();
        }

        if failed_peer_sends.is_empty() {
            Ok(())
        } else {
            bail!("Failed to send chunk request to: {:?}", failed_peer_sends)
        }
    }

    pub fn add_request(&mut self, version: u64, peers: Vec<PeerNetworkId>) -> ChunkRequestInfo {
        if let Some(prev_request) = self.requests.get_mut(&version) {
            let now = SystemTime::now();
            if self.multicast_level != prev_request.multicast_level {
                // restart multicast timer for this request if multicast level changed
                prev_request.multicast_level = self.multicast_level;
                prev_request.multicast_start_time = now;
            }
            prev_request.last_request_peers = peers;
            prev_request.last_request_time = now;
            prev_request.clone()
        } else {
            self.requests.insert(
                version,
                ChunkRequestInfo::new(version, peers, self.multicast_level),
            );
            self.requests
                .get(&version)
                .expect("missing chunk request that was just added")
                .clone()
        }
    }

    pub fn process_empty_chunk(&mut self, peer: &PeerNetworkId) {
        self.update_score(&peer, PeerScoreUpdateType::EmptyChunk);
    }

    pub fn process_invalid_chunk(&mut self, peer: &PeerNetworkId) {
        self.update_score(peer, PeerScoreUpdateType::InvalidChunk);
    }

    pub fn process_success_response(&mut self, peer: &PeerNetworkId) {
        // update multicast
        let peer_level = self
            .upstream_config
            .get_upstream_preference(peer.raw_network_id())
            .unwrap_or_else(|| self.upstream_config.upstream_count());
        if peer_level < self.multicast_level {
            // reduce multicast_level if we received peer from lower multicast level (= more highly
            // prioritized network)
            self.update_multicast(peer_level, None)
        }

        // update score
        self.update_score(peer, PeerScoreUpdateType::Success);
    }

    // penalize peer's score for giving chunk with starting version that doesn't match local synced version
    pub fn process_chunk_version_mismatch(
        &mut self,
        peer: &PeerNetworkId,
        chunk_version: u64,
        synced_version: u64,
    ) -> Result<()> {
        if self.is_multicast_response(chunk_version, peer) {
            // This chunk response was in response to a past multicast response that another
            // peer sent a response to earlier than this peer
            // Don't penalize if this response did not technically time out
            bail!(
                "[state sync] Received chunk for outdated request from {:?}: known_version: {}, received: {}",
                peer,
                synced_version,
                chunk_version
            );
        } else {
            self.update_score(&peer, PeerScoreUpdateType::ChunkVersionCannotBeApplied);
            bail!(
                "[state sync] Non sequential chunk from {:?}: known_version: {}, received: {}",
                peer,
                synced_version,
                chunk_version
            );
        }
    }

    fn is_multicast_response(&self, version: u64, peer: &PeerNetworkId) -> bool {
        self.requests.get(&version).map_or(false, |req| {
            req.last_request_peers.contains(peer) && req.last_request_peers.len() > 1
        })
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

    fn get_multicast_start_time(&self, version: u64) -> Option<SystemTime> {
        self.requests
            .get(&version)
            .map(|req_info| req_info.multicast_start_time)
    }

    /// Removes requests whose known_version < `version` if they are older than now - `timeout`
    /// We keep the requests that have not timed out so we don't penalize
    /// peers who send chunks after the first peer who sends the first successful chunk response for a
    /// multicasted request
    pub fn remove_requests(&mut self, version: u64) {
        // only remove requests that have timed out or sent to one peer, so we don't penalize for multicasted responses
        // that still came back on time, based on per-peer timeout
        let versions_to_remove = self
            .requests
            .range(..version)
            .filter_map(|(version, req)| {
                if is_timeout(req.last_request_time, self.request_timeout) {
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

    /// Checks whether the request sent with known_version = `version` has timed out
    /// Returns true if such a request timed out (or does not exist), else false.
    pub fn check_request_timeout(&mut self, version: u64) -> Result<bool> {
        let last_request_time = self.get_last_request_time(version).unwrap_or(UNIX_EPOCH);

        let timeout = is_timeout(last_request_time, self.request_timeout);
        if !timeout {
            return Ok(timeout);
        }

        // update peer info based on timeout
        let peers_to_penalize = match self.requests.get(&version) {
            Some(prev_request) => prev_request.last_request_peers.clone(),
            None => {
                return Ok(timeout);
            }
        };
        for peer in peers_to_penalize.iter() {
            self.update_score(peer, PeerScoreUpdateType::TimeOut);
        }

        // increment multicast level if this request is also multicast-timed-out
        let multicast_start_time = self.get_multicast_start_time(version).unwrap_or(UNIX_EPOCH);
        if is_timeout(multicast_start_time, self.multicast_timeout) {
            let new_multicast_level = std::cmp::min(
                self.multicast_level
                    .checked_add(1)
                    .ok_or_else(|| format_err!("New multicast level has overflown!"))?,
                self.upstream_config
                    .upstream_count()
                    .checked_sub(1)
                    .ok_or_else(|| format_err!("Upstream count has overflown!"))?, // multicast_level (=network preference) is 0-indexed
            );
            self.update_multicast(new_multicast_level, Some(version));
        }
        Ok(timeout)
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

    pub fn is_known_upstream_peer(&self, peer: &PeerNetworkId) -> bool {
        self.peers.contains_key(peer)
    }

    fn update_multicast(&mut self, new_level: usize, request_version: Option<u64>) {
        let old_level = self.multicast_level;
        let event = match new_level {
            l if l > old_level => LogEvent::Failover,
            l if l < old_level => LogEvent::Recover,
            _ => return,
        };

        let mut log = LogSchema::event_log(LogEntry::Multicast, event)
            .old_multicast_level(old_level)
            .new_multicast_level(new_level);
        if let Some(version) = request_version {
            log = log.request_version(version);
        }

        info!(log);
        self.multicast_level = new_level;
        counters::MULTICAST_LEVEL.set(self.multicast_level as i64);
    }
}

// Returns whether the timeout for the given params has occurred, compared to SystemTime at function call
// returns true if the timeout (=`timeout_start + timeout_duration`) has happened, else false
fn is_timeout(timeout_start: SystemTime, timeout_duration: Duration) -> bool {
    timeout_start
        .checked_add(timeout_duration)
        .map_or(false, |deadline| {
            SystemTime::now().duration_since(deadline).is_ok()
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_CHUNKS_TO_PROCESS: u64 = 50;
    const NUM_PICKS_TO_MAKE: u64 = 1000;

    #[test]
    fn test_disable_peer() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(0, 1);

        let validator_0 = validators[0].clone();

        // Verify single validator in peers
        assert!(request_manager.is_known_upstream_peer(&validator_0));
        assert!(!request_manager.no_available_peers());

        // Disable validator 0
        request_manager.disable_peer(&validator_0, ConnectionOrigin::Outbound);

        // Verify validator 0 is still known, but no longer available
        assert!(request_manager.is_known_upstream_peer(&validator_0));
        assert!(request_manager.no_available_peers());

        // Add validator 0 and verify it's now enabled
        request_manager.enable_peer(validator_0, ConnectionOrigin::Outbound);
        assert!(!request_manager.no_available_peers());
    }

    #[test]
    fn test_score_chunk_success() {
        let num_validators = 4;
        let (mut request_manager, validators) =
            generate_request_manager_and_validators(0, num_validators);

        // Process empty chunk responses from all validators
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            for validator_index in 0..num_validators {
                request_manager.process_empty_chunk(&validators[validator_index as usize]);
            }
        }

        // Process successful chunk responses from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.process_success_response(&validators[0]);
        }

        // Verify validator 0 is chosen more often than the other validators
        verify_validator_picked_most_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_chunk_timeout() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(0, 4);

        let validator_0 = vec![validators[0].clone()];

        // Process multiple request timeouts from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.add_request(1, validator_0.clone());
            assert!(request_manager.check_request_timeout(1).unwrap());
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_chunk_version_mismatch() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(0, 4);

        let validator_0 = validators[0].clone();

        // Process multiple chunk version mismatches from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.add_request(100, vec![validator_0.clone()]);
            request_manager
                .process_chunk_version_mismatch(&validator_0, 100, 0)
                .unwrap_err();
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_chunk_version_mismatch_multicast() {
        let num_validators = 4;
        let (mut request_manager, validators) =
            generate_request_manager_and_validators(0, num_validators);

        let validator_0 = validators[0].clone();
        let validator_1 = validators[1].clone();

        // Process empty chunk responses from all validators except validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            for validator_index in 1..num_validators {
                request_manager.process_empty_chunk(&validators[validator_index as usize]);
            }
        }

        // Process multiple multi-cast chunk version mismatches from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.add_request(100, vec![validator_0.clone(), validator_1.clone()]);
            request_manager
                .process_chunk_version_mismatch(&validator_0, 100, 0)
                .unwrap_err();
        }

        // Verify validator 0 is chosen more often than the other validators
        verify_validator_picked_most_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_empty_chunk() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(10, 4);

        // Process multiple empty chunk responses from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.process_empty_chunk(&validators[0]);
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_invalid_chunk() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(10, 4);

        // Process multiple invalid chunk responses from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.process_invalid_chunk(&validators[0]);
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_remove_requests() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(0, 2);

        let validator_0 = vec![validators[0].clone()];
        let validator_1 = vec![validators[1].clone()];

        // Add version requests to request manager
        request_manager.add_request(1, validator_0.clone());
        request_manager.add_request(3, validator_1.clone());
        request_manager.add_request(5, validator_0.clone());
        request_manager.add_request(10, validator_0);
        request_manager.add_request(12, validator_1);

        // Remove all request versions below 5
        request_manager.remove_requests(5);

        // Verify versions updated correctly
        assert!(request_manager.get_last_request_time(1).is_none());
        assert!(request_manager.get_last_request_time(3).is_none());
        assert!(request_manager.get_last_request_time(5).is_some());
        assert!(request_manager.get_last_request_time(10).is_some());
        assert!(request_manager.get_last_request_time(12).is_some());
    }

    #[test]
    fn test_request_times() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(0, 2);

        // Verify first request time doesn't exist for missing request
        assert!(request_manager.get_first_request_time(1).is_none());

        // Add version requests to request manager
        request_manager.add_request(1, vec![validators[0].clone()]);
        request_manager.add_request(1, vec![validators[1].clone()]);

        // Verify first request time is less than last request time
        assert!(
            request_manager.get_first_request_time(1).unwrap()
                < request_manager.get_last_request_time(1).unwrap()
        );
    }

    /// Verify that the specified validator is chosen most often (due to having a
    /// higher peer score internally).
    fn verify_validator_picked_most_often(
        request_manager: &mut RequestManager,
        validators: &[PeerNetworkId],
        validator_index: usize,
    ) {
        verify_validator_pick_frequency(request_manager, validators, validator_index, true)
    }

    /// Verify that the specified validator is chosen least often (due to having a
    /// lower peer score internally).
    fn verify_validator_picked_least_often(
        request_manager: &mut RequestManager,
        validators: &[PeerNetworkId],
        validator_index: usize,
    ) {
        verify_validator_pick_frequency(request_manager, validators, validator_index, false)
    }

    /// Verifies the picking frequency of the specified validator: if `check_highest_frequency`
    /// is true we verify the validator is chosen most often, otherwise, we verify it is
    /// chosen least often.
    fn verify_validator_pick_frequency(
        request_manager: &mut RequestManager,
        validators: &[PeerNetworkId],
        validator_index: usize,
        check_highest_frequency: bool,
    ) {
        // Calculate selection counts for validators
        let pick_counts = calculate_pick_counts_for_validators(request_manager, NUM_PICKS_TO_MAKE);

        // Verify validator frequency
        let validator_count = pick_counts.get(&validators[validator_index]).unwrap_or(&0);
        for (index, validator) in validators.iter().enumerate() {
            if validator_index != index {
                if check_highest_frequency {
                    assert!(validator_count > pick_counts.get(&validator).unwrap());
                } else {
                    assert!(validator_count < pick_counts.get(&validator).unwrap());
                }
            }
        }
    }

    /// Picks a peer to send a chunk request to (multiple times) and constructs a pick count
    /// for each of the chosen peers.
    fn calculate_pick_counts_for_validators(
        request_manager: &mut RequestManager,
        number_of_picks_to_execute: u64,
    ) -> HashMap<PeerNetworkId, u64> {
        let mut pick_counts = HashMap::new();

        for _ in 0..number_of_picks_to_execute {
            let picked_peers = request_manager.pick_peers();
            assert_eq!(1, picked_peers.len()); // Ensure only one validator per multicast level

            let picked_peer = picked_peers[0].clone();
            let counter = pick_counts.entry(picked_peer).or_insert(0);
            *counter += 1;
        }

        pick_counts
    }

    /// Generates a new request manager with a specified number of validator peers enabled.
    fn generate_request_manager_and_validators(
        request_timeout: u64,
        num_validators: u64,
    ) -> (RequestManager, Vec<PeerNetworkId>) {
        let mut request_manager = RequestManager::new(
            UpstreamConfig::default(),
            Duration::from_secs(request_timeout),
            Duration::from_secs(30),
            HashMap::new(),
        );

        let mut validators = Vec::new();
        for _ in 0..num_validators {
            let validator = PeerNetworkId::random_validator();
            request_manager.enable_peer(validator.clone(), ConnectionOrigin::Outbound);
            validators.push(validator);
        }

        (request_manager, validators)
    }
}
