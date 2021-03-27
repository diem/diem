// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::GetChunkRequest,
    counters,
    error::Error,
    logging::{LogEntry, LogEvent, LogSchema},
    network::{StateSyncMessage, StateSyncSender},
};
use diem_config::{
    config::{PeerNetworkId, PeerRole},
    network_id::{NetworkId, NodeNetworkId},
};
use diem_logger::prelude::*;
use itertools::Itertools;
use netcore::transport::ConnectionOrigin;
use network::transport::ConnectionMetadata;
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    cmp::Ordering,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        BTreeMap, HashMap,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Scores for peer rankings based on preferences and behavior.
const MAX_SCORE: f64 = 100.0;
const MIN_SCORE: f64 = 1.0;
const STARTING_SCORE: f64 = 50.0;
const STARTING_SCORE_PREFERRED: f64 = 100.0;

/// Basic metadata about the chunk request.
#[derive(Clone, Debug)]
pub struct ChunkRequestInfo {
    version: u64,
    first_request_time: SystemTime,
    last_request_time: SystemTime,
    multicast_level: NetworkId,
    multicast_start_time: SystemTime,
    last_request_peers: Vec<PeerNetworkId>,
}

impl ChunkRequestInfo {
    pub fn new(version: u64, peers: Vec<PeerNetworkId>, multicast_level: NetworkId) -> Self {
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
    InvalidChunkRequest,
    TimeOut,
}

pub struct RequestManager {
    // Maps each peer to their peer score
    peer_scores: HashMap<PeerNetworkId, f64>,
    requests: BTreeMap<u64, ChunkRequestInfo>,
    // duration with the same version before the next attempt to get the next chunk
    request_timeout: Duration,
    // duration with the same version before multicasting, i.e. sending the next chunk request to more networks
    multicast_timeout: Duration,
    // The maximum network level that chunk requests will be sent to. The greater the network level,
    // the more networks this node will send multicast requests to. Network ordering is defined by
    // NetworkId.
    multicast_network_level: NetworkId,
    network_senders: HashMap<NodeNetworkId, StateSyncSender>,
}

impl RequestManager {
    pub fn new(
        request_timeout: Duration,
        multicast_timeout: Duration,
        network_senders: HashMap<NodeNetworkId, StateSyncSender>,
    ) -> Self {
        let multicast_network_level = NetworkId::Validator;
        update_multicast_network_counter(multicast_network_level.clone());

        Self {
            peer_scores: HashMap::new(),
            requests: BTreeMap::new(),
            request_timeout,
            multicast_timeout,
            multicast_network_level,
            network_senders,
        }
    }

    pub fn enable_peer(
        &mut self,
        peer: PeerNetworkId,
        metadata: ConnectionMetadata,
    ) -> Result<(), Error> {
        if !self.is_valid_state_sync_peer(&peer, metadata.origin) {
            return Err(Error::InvalidStateSyncPeer(
                peer.to_string(),
                metadata.origin.to_string(),
            ));
        }

        info!(LogSchema::new(LogEntry::NewPeer)
            .peer(&peer)
            .is_valid_peer(true));
        counters::ACTIVE_UPSTREAM_PEERS
            .with_label_values(&[&peer.raw_network_id().to_string()])
            .inc();

        match self.peer_scores.entry(peer) {
            Occupied(occupied_entry) => {
                warn!(LogSchema::new(LogEntry::NewPeerAlreadyExists).peer(occupied_entry.key()));
            }
            Vacant(vacant_entry) => {
                let peer_score = if metadata.role == PeerRole::PreferredUpstream {
                    STARTING_SCORE_PREFERRED
                } else {
                    STARTING_SCORE
                };
                vacant_entry.insert(peer_score);
            }
        }

        Ok(())
    }

    pub fn disable_peer(&mut self, peer: &PeerNetworkId) -> Result<(), Error> {
        info!(LogSchema::new(LogEntry::LostPeer).peer(&peer));

        if self.peer_scores.contains_key(peer) {
            counters::ACTIVE_UPSTREAM_PEERS
                .with_label_values(&[&peer.raw_network_id().to_string()])
                .dec();
            self.peer_scores.remove(peer);
        } else {
            warn!(LogSchema::new(LogEntry::LostPeerNotKnown).peer(&peer));
        }

        Ok(())
    }

    pub fn no_available_peers(&self) -> bool {
        self.peer_scores.is_empty()
    }

    fn update_score(&mut self, peer: &PeerNetworkId, update_type: PeerScoreUpdateType) {
        if let Some(score) = self.peer_scores.get_mut(peer) {
            let old_score = *score;
            let new_score = match update_type {
                PeerScoreUpdateType::Success => {
                    let new_score = old_score + 1.0;
                    new_score.min(MAX_SCORE)
                }
                PeerScoreUpdateType::InvalidChunk
                | PeerScoreUpdateType::ChunkVersionCannotBeApplied => {
                    let new_score = old_score * 0.8;
                    new_score.max(MIN_SCORE)
                }
                PeerScoreUpdateType::TimeOut
                | PeerScoreUpdateType::EmptyChunk
                | PeerScoreUpdateType::InvalidChunkRequest => {
                    let new_score = old_score * 0.95;
                    new_score.max(MIN_SCORE)
                }
            };
            *score = new_score;
        }
    }

    // Calculates a weighted index for each peer per network. This is used to probabilistically
    // select a peer (per network) to send a chunk request to.
    fn calculate_weighted_peers_per_network(
        &mut self,
    ) -> BTreeMap<NetworkId, (Vec<PeerNetworkId>, Option<WeightedIndex<f64>>)> {
        // Group peers by network level
        let peers_by_network_level = self
            .peer_scores
            .iter()
            .map(|(peer, peer_score)| (peer.raw_network_id(), (peer, peer_score)))
            .into_group_map();

        // For each network, compute the weighted index
        peers_by_network_level
            .into_iter()
            .map(|(network_level, peers)| {
                let mut eligible_peers = vec![];
                let weights: Vec<_> = peers
                    .iter()
                    .map(|(peer, peer_score)| {
                        eligible_peers.push((*peer).clone());
                        *peer_score
                    })
                    .collect();
                let weighted_index = WeightedIndex::new(weights)
                    .map_err(|error| {
                        error!(
                            "Failed to compute weighted index for eligible peers: {:?}",
                            error
                        );
                        error
                    })
                    .ok();
                (network_level, (eligible_peers, weighted_index))
            })
            .collect()
    }

    /// Picks a set of peers to send chunk requests to. Here, we attempt to pick one peer
    /// per network, in order of network level preference. The set of networks selected is
    /// determined by the multicast network level. All networks with preference
    /// level <= multicast level are sampled. If there are no live peers in these networks,
    /// the multicast level is updated to the preference level of the first chosen network.
    fn pick_peers(&mut self) -> Vec<PeerNetworkId> {
        // Calculate a weighted peer selection map per network level
        let weighted_peers_per_network = self.calculate_weighted_peers_per_network();

        let mut chosen_peers = vec![];
        let mut new_multicast_network_level = None;

        for (network_level, (peers, weighted_index)) in &weighted_peers_per_network {
            if let Some(peer) = pick_peer(peers, weighted_index) {
                chosen_peers.push(peer)
            }
            // At minimum, go through networks with preference level <= multicast level.
            // If no peers are found for the current multicast level, continue doing
            // best effort search of the networks to failover to.
            if !chosen_peers.is_empty() && *network_level >= self.multicast_network_level {
                new_multicast_network_level = Some(network_level.clone());
                break;
            }
        }

        if let Some(network_level) = new_multicast_network_level {
            self.update_multicast_network_level(network_level, None);
        }

        chosen_peers
    }

    pub fn send_chunk_request(&mut self, req: GetChunkRequest) -> Result<(), Error> {
        let log = LogSchema::new(LogEntry::SendChunkRequest).chunk_request(req.clone());

        let peers = self.pick_peers();
        if peers.is_empty() {
            warn!(log.event(LogEvent::MissingPeers));
            return Err(Error::NoAvailablePeers(
                "No peers to send chunk request to".into(),
            ));
        }

        let req_info = self.add_request(req.known_version, peers.clone());
        debug!(log
            .clone()
            .event(LogEvent::ChunkRequestInfo)
            .chunk_req_info(&req_info));

        let msg = StateSyncMessage::GetChunkRequest(Box::new(req));
        let mut failed_peer_sends = vec![];

        for peer in peers {
            let mut sender = self.get_network_sender(&peer);
            let peer_id = peer.peer_id();
            let send_result = sender.send_to(peer_id, msg.clone());
            let curr_log = log.clone().peer(&peer);
            let result_label = if let Err(e) = send_result {
                failed_peer_sends.push(peer.clone());
                error!(curr_log.event(LogEvent::NetworkSendError).error(&e));
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
            Err(Error::UnexpectedError(format!(
                "Failed to send chunk request to: {:?}",
                failed_peer_sends
            )))
        }
    }

    fn get_network_sender(&mut self, peer: &PeerNetworkId) -> StateSyncSender {
        self.network_senders
            .get_mut(&peer.network_id())
            .unwrap_or_else(|| {
                panic!(
                    "Missing network sender for network: {:?}",
                    peer.network_id()
                )
            })
            .clone()
    }

    pub fn send_chunk_response(
        &mut self,
        peer: &PeerNetworkId,
        message: StateSyncMessage,
    ) -> Result<(), Error> {
        self.get_network_sender(peer)
            .send_to(peer.peer_id(), message)
    }

    pub fn add_request(&mut self, version: u64, peers: Vec<PeerNetworkId>) -> ChunkRequestInfo {
        if let Some(prev_request) = self.requests.get_mut(&version) {
            let now = SystemTime::now();
            if self.multicast_network_level != prev_request.multicast_level {
                // restart multicast timer for this request if multicast level changed
                prev_request.multicast_level = self.multicast_network_level.clone();
                prev_request.multicast_start_time = now;
            }
            prev_request.last_request_peers = peers;
            prev_request.last_request_time = now;
            prev_request.clone()
        } else {
            let chunk_request_info =
                ChunkRequestInfo::new(version, peers, self.multicast_network_level.clone());
            self.requests.insert(version, chunk_request_info.clone());
            chunk_request_info
        }
    }

    pub fn process_chunk_from_downstream(&mut self, peer: &PeerNetworkId) {
        self.update_score(&peer, PeerScoreUpdateType::InvalidChunk);
    }

    pub fn process_empty_chunk(&mut self, peer: &PeerNetworkId) {
        self.update_score(&peer, PeerScoreUpdateType::EmptyChunk);
    }

    pub fn process_invalid_chunk(&mut self, peer: &PeerNetworkId) {
        self.update_score(peer, PeerScoreUpdateType::InvalidChunk);
    }

    pub fn process_invalid_chunk_request(&mut self, peer: &PeerNetworkId) {
        self.update_score(peer, PeerScoreUpdateType::InvalidChunkRequest);
    }

    pub fn process_success_response(&mut self, peer: &PeerNetworkId) {
        // Update the multicast level if appropriate
        let peer_network_level = peer.raw_network_id();
        if peer_network_level < self.multicast_network_level {
            // Reduce the multicast network level as we received a chunk response from a
            // peer in a lower (that is, higher priority) network.
            self.update_multicast_network_level(peer_network_level, None)
        }

        // Update the peer's score
        self.update_score(peer, PeerScoreUpdateType::Success);
    }

    // Penalize the peer for giving a chunk with a starting version that doesn't match
    // the local synced version.
    pub fn process_chunk_version_mismatch(
        &mut self,
        peer: &PeerNetworkId,
        chunk_version: u64,
        synced_version: u64,
    ) -> Result<(), Error> {
        if self.is_multicast_response(chunk_version, peer) {
            // If the chunk is a stale multicast response (for a request that another peer sent
            // a response to earlier) don't penalize the peer (no mismatch occurred -- it's just slow).
            Err(Error::ReceivedChunkForOutdatedRequest(
                peer.to_string(),
                synced_version.to_string(),
                chunk_version.to_string(),
            ))
        } else {
            self.update_score(&peer, PeerScoreUpdateType::ChunkVersionCannotBeApplied);
            Err(Error::ReceivedNonSequentialChunk(
                peer.to_string(),
                synced_version.to_string(),
                chunk_version.to_string(),
            ))
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
    pub fn has_request_timed_out(&mut self, version: u64) -> Result<bool, Error> {
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

        // Increase the multicast network level if this request has also hit a multicast timeout
        let multicast_start_time = self.get_multicast_start_time(version).unwrap_or(UNIX_EPOCH);
        if is_timeout(multicast_start_time, self.multicast_timeout) {
            // Move to the next multicast network level
            let new_multicast_network_level = match self.multicast_network_level {
                NetworkId::Validator => NetworkId::vfn_network(),
                _ => NetworkId::Public,
            };
            self.update_multicast_network_level(new_multicast_network_level, Some(version));
        }
        Ok(timeout)
    }

    /// The validator network and vfn network are always considered valid when state syncing, as
    /// multicasting will prioritize sending chunk requests to the most important network first.
    /// If the peer is on the public network, we only consider it a valid peer to send chunk
    /// requests to if we connected to it.
    fn is_valid_state_sync_peer(&self, peer: &PeerNetworkId, origin: ConnectionOrigin) -> bool {
        peer.raw_network_id().is_validator_network()
            || peer.raw_network_id().is_vfn_network()
            || origin == ConnectionOrigin::Outbound
    }

    pub fn is_known_state_sync_peer(&self, peer: &PeerNetworkId) -> bool {
        self.peer_scores.contains_key(peer)
    }

    fn update_multicast_network_level(
        &mut self,
        new_level: NetworkId,
        request_version: Option<u64>,
    ) {
        // Update level if the new level is different
        let current_level = self.multicast_network_level.clone();
        let log_event = match new_level.cmp(&current_level) {
            Ordering::Equal => return,
            Ordering::Greater => LogEvent::Failover,
            Ordering::Less => LogEvent::Recover,
        };
        self.multicast_network_level = new_level.clone();

        // Update the counters and logs
        update_multicast_network_counter(self.multicast_network_level.clone());
        let mut log_event = LogSchema::event_log(LogEntry::Multicast, log_event)
            .old_multicast_level(current_level)
            .new_multicast_level(new_level);
        if let Some(version) = request_version {
            log_event = log_event.request_version(version);
        }
        info!(log_event);
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

// TODO(joshlind): Right now, the internal NetworkId state is leaking into state
// sync (and other places in the code/other components, too). For example, this mapping between
// NetworkId and integer for the purpose of maintaining visible counters should be done
// by NetworkId and not here. Look into updating the NetworkId interface to expose this
// conversion, as well as provide clearer interfaces around max, min network values as
// well as moving between network levels.
fn update_multicast_network_counter(multicast_network_level: NetworkId) {
    let network_counter_value = if multicast_network_level.is_validator_network() {
        0
    } else if multicast_network_level.is_vfn_network() {
        1
    } else {
        2
    };
    counters::MULTICAST_LEVEL.set(network_counter_value);
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
        assert!(request_manager.is_known_state_sync_peer(&validator_0));
        assert!(!request_manager.no_available_peers());

        // Disable validator 0
        request_manager.disable_peer(&validator_0).unwrap();

        // Verify validator 0 is now unknown
        assert!(!request_manager.is_known_state_sync_peer(&validator_0));
        assert!(request_manager.no_available_peers());

        // Add validator 0 and verify it's now re-enabled
        add_validator_to_request_manager(&mut request_manager, &validator_0, PeerRole::Validator);
        assert!(request_manager.is_known_state_sync_peer(&validator_0));
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
            assert!(request_manager.has_request_timed_out(1).unwrap());
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
    fn test_score_invalid_chunk_request() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(10, 4);

        // Process multiple invalid chunk requests from validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.process_invalid_chunk_request(&validators[0]);
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_chunk_from_downstream() {
        let (mut request_manager, validators) = generate_request_manager_and_validators(10, 4);

        // Process multiple chunk responses from downstream validator 0
        for _ in 0..NUM_CHUNKS_TO_PROCESS {
            request_manager.process_chunk_from_downstream(&validators[0]);
        }

        // Verify validator 0 is chosen less often than the other validators
        verify_validator_picked_least_often(&mut request_manager, &validators, 0);
    }

    #[test]
    fn test_score_preferred() {
        let num_validators = 4;
        let mut request_manager = generate_request_manager(0);

        // Create the validators and add them to the request manager. Note that validator at
        // index 0 is the only preferred validator.
        let mut validators = Vec::new();
        for validator_index in 0..num_validators {
            let validator = PeerNetworkId::random_validator();
            let peer_role = if validator_index == 0 {
                PeerRole::PreferredUpstream
            } else {
                PeerRole::Validator
            };
            add_validator_to_request_manager(&mut request_manager, &validator, peer_role);
            validators.push(validator);
        }

        // Verify validator 0 is chosen more often than the other validators
        verify_validator_picked_most_often(&mut request_manager, &validators, 0);
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

    /// Generates a new request manager with the given request_timeout.
    fn generate_request_manager(request_timeout: u64) -> RequestManager {
        RequestManager::new(
            Duration::from_secs(request_timeout),
            Duration::from_secs(30),
            HashMap::new(),
        )
    }

    /// Generates a new request manager with a specified number of validator peers enabled.
    fn generate_request_manager_and_validators(
        request_timeout: u64,
        num_validators: u64,
    ) -> (RequestManager, Vec<PeerNetworkId>) {
        let mut request_manager = generate_request_manager(request_timeout);

        let mut validators = Vec::new();
        for _ in 0..num_validators {
            let validator = PeerNetworkId::random_validator();
            add_validator_to_request_manager(&mut request_manager, &validator, PeerRole::Validator);
            validators.push(validator);
        }

        (request_manager, validators)
    }

    /// Adds the given validator to the specified request manager using the peer role.
    fn add_validator_to_request_manager(
        request_manager: &mut RequestManager,
        validator: &PeerNetworkId,
        peer_role: PeerRole,
    ) {
        let connection_metadata = ConnectionMetadata::mock_with_role_and_origin(
            validator.peer_id(),
            peer_role,
            ConnectionOrigin::Inbound,
        );
        request_manager
            .enable_peer(validator.clone(), connection_metadata)
            .unwrap();
    }
}
