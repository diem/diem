// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol to discover network addresses of other peers on the Libra network
//!
//! This is for testing purposes only and should not be used in production networks.
//!
//! ## Implementation
//!
//! The gossip_discovery module is implemented as a stand-alone actor in the Network sub-system of the
//! Libra stack. The actor participates in discovery by periodically sending its observed state of
//! the network to a randomly chosen peer. Other peers are also expected to be running the same
//! protocol. Therefore, in expectation, every peer expects to hear from 1 other peer in each
//! round. On hearing from the remote peer, the local discovery module tries to reconcile its state
//! to reflect any changes. In addition to updating its state, it also passes on new information to
//! the [`ConnectivityManager`] module.
//!
//! For the initial bootstrap of a node, it sends the discovery message to a randomly chosen seed
//! peer in each round. The message only contains the identity of this peer unless it learns more
//! about the network membership from another peer.
//!
//! Currently we do not use this mechanism to detect peer failures - instead, we simply connect to
//! all the peers in the network, and hope to learn about their failure on connection errors.
//!
//! TODO: We need to handle to case of peers who may no longer be a part of the network.
//!
//! ## Future work
//!
//! - Currently, we do not try to detect/punish nodes which are just lurking (without contributing
//! to the protocol), or actively trying to spread misinformation in the network. In the future, we
//! plan to remedy this by introducing a module dedicated to detecting byzantine behavior, and by
//! making the discovery protocol itself tolerant to byzantine faults.
//!
//! [`ConnectivityManager`]: ../../connectivity_manager

use crate::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    constants::NETWORK_CHANNEL_SIZE,
    counters,
    error::NetworkError,
    logging::NetworkSchema,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{Event, NetworkEvents, NetworkSender, NewNetworkSender},
    ProtocolId,
};
use bytes::Bytes;
use channel::message_queues::QueueStyle;
use futures::{
    sink::SinkExt,
    stream::{FusedStream, Stream, StreamExt},
};
use libra_config::network_id::NetworkContext;
use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::Arc,
};

pub mod builder;
#[cfg(test)]
mod test;

/// The interface from Network to Discovery module.
///
/// `DiscoveryNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `GossipDiscoveryMsg` types. `DiscoveryNetworkEvents` is a thin wrapper
/// around a `channel::Receiver<PeerManagerNotification>`.
pub type GossipDiscoveryNetworkEvents = NetworkEvents<GossipDiscoveryMsg>;

/// The interface from Discovery to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<GossipDiscoveryMsg>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `DiscoveryNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct GossipDiscoveryNetworkSender {
    inner: NetworkSender<GossipDiscoveryMsg>,
}

/// Configuration for the network endpoints to support Discovery.
pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![],
        vec![ProtocolId::DiscoveryDirectSend],
        QueueStyle::LIFO,
        NETWORK_CHANNEL_SIZE,
        Some(&counters::PENDING_DISCOVERY_NETWORK_EVENTS),
    )
}

impl NewNetworkSender for GossipDiscoveryNetworkSender {
    /// Create a new Discovery sender
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}
impl GossipDiscoveryNetworkSender {
    /// Send a GossipDiscoveryMsg to a peer.
    pub fn send_to(&mut self, peer: PeerId, msg: GossipDiscoveryMsg) -> Result<(), NetworkError> {
        self.inner
            .send_to(peer, ProtocolId::DiscoveryDirectSend, msg)
    }
}

/// The actor running the gossip discovery protocol.
pub struct GossipDiscovery<TTicker> {
    /// Note for self, which is prefixed with an underscore as this is not used but is in
    /// preparation for logic that changes the advertised Note while the validator is running.
    note: Note,
    network_context: Arc<NetworkContext>,
    /// The DNS domain name other public full nodes should query to get this
    /// validator's list of full nodes.
    dns_seed_addr: Bytes,
    /// Current state, maintaining the most recent Note for each peer, alongside parsed PeerInfo.
    known_peers: HashMap<PeerId, Note>,
    /// Currently connected peers.
    connected_peers: HashSet<PeerId>,
    /// Ticker to trigger state send to a random peer. In production, the ticker is likely to be
    /// fixed duration interval timer.
    ticker: TTicker,
    /// Handle to send requests to Network.
    network_reqs_tx: GossipDiscoveryNetworkSender,
    /// Handle to receive notifications from Network.
    network_notifs_rx: GossipDiscoveryNetworkEvents,
    /// Channel to send requests to ConnectivityManager.
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    /// Random-number generator.
    rng: SmallRng,
}

impl<TTicker> GossipDiscovery<TTicker>
where
    TTicker: Stream + FusedStream + Unpin,
{
    pub fn new(
        network_context: Arc<NetworkContext>,
        self_addrs: Vec<NetworkAddress>,
        ticker: TTicker,
        network_reqs_tx: GossipDiscoveryNetworkSender,
        network_notifs_rx: GossipDiscoveryNetworkEvents,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        // TODO(philiphayes): wire through config
        let dns_seed_addr = b"example.com";
        let self_peer_id = network_context.peer_id();
        let epoch = get_unix_epoch();
        let self_note = Note::new(self_peer_id, self_addrs, dns_seed_addr, epoch);

        let known_peers = vec![(self_peer_id, self_note.clone())]
            .into_iter()
            .collect();

        Self {
            note: self_note,
            network_context,
            dns_seed_addr: Bytes::from_static(dns_seed_addr),
            known_peers,
            connected_peers: HashSet::new(),
            ticker,
            network_reqs_tx,
            network_notifs_rx,
            conn_mgr_reqs_tx,
            rng: SmallRng::from_entropy(),
        }
    }

    // Starts the main event loop for the discovery actor. We bootstrap by first dialing all the
    // seed peers, and then entering the event handling loop. Messages are received from:
    // - a ticker to trigger discovery message send to a random connected peer
    // - an incoming message from a peer wishing to send its state
    // - an internal task once it has processed incoming messages from a peer, and wishes for
    // discovery actor to update its state.
    pub async fn start(mut self) {
        // Ensure our metrics counter has an initial value.
        self.record_num_discovery_notes();

        debug!(
            NetworkSchema::new(&self.network_context),
            "{} Starting Discovery actor event loop", self.network_context
        );
        loop {
            futures::select! {
                notif = self.network_notifs_rx.select_next_some() => {
                    self.handle_network_event(notif).await;
                },
                _ = self.ticker.select_next_some() => {
                    self.handle_tick();
                }
                complete => {
                    warn!(
                        NetworkSchema::new(&self.network_context),
                        "{} Discovery actor terminated",
                        self.network_context
                    );
                    break;
                }
            }
        }
    }

    // Handles a clock "tick" by:
    // 1. Selecting a random peer to send state to.
    // 2. Compose the msg to send.
    // 3. Spawn off a new task to push the msg to the peer.
    fn handle_tick(&mut self) {
        debug!(
            NetworkSchema::new(&self.network_context),
            "{} Discovery interval tick", self.network_context
        );
        // On each tick, we choose a random neighbor and push our state to it.
        if let Some(peer) = self.choose_random_neighbor() {
            // We clone `peer_mgr_reqs_tx` member of Self, since using `self` inside fut below
            // triggers some lifetime errors.
            let mut sender = self.network_reqs_tx.clone();
            // Compose discovery msg to send.
            let msg = self.compose_discovery_msg();
            if let Err(err) = sender.send_to(peer, msg) {
                warn!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&peer)
                        .debug_error(&err),
                    "{} Failed to send discovery msg to {}; error: {:?}",
                    self.network_context,
                    peer.short_str(),
                    err
                );
            };
        }
    }

    async fn handle_network_event(
        &mut self,
        event: Result<Event<GossipDiscoveryMsg>, NetworkError>,
    ) {
        trace!(
            NetworkSchema::new(&self.network_context),
            "{} Network event::{:?}",
            self.network_context,
            event
        );
        match event {
            Ok(e) => {
                match e {
                    Event::NewPeer(peer_id, _origin) => {
                        // Add peer to connected peer list.
                        self.connected_peers.insert(peer_id);
                    }
                    Event::LostPeer(peer_id, _origin) => {
                        // Remove peer from connected peer list.
                        self.connected_peers.remove(&peer_id);
                    }
                    Event::Message((peer_id, msg)) => {
                        self.reconcile(peer_id, msg.notes).await;
                        self.record_num_discovery_notes();
                    }
                    Event::RpcRequest(req) => {
                        warn!(
                            NetworkSchema::new(&self.network_context),
                            "{} Unexpected notification from network: {:?}",
                            self.network_context,
                            req
                        );
                        debug_assert!(false);
                    }
                }
            }
            Err(err) => {
                info!(
                    NetworkSchema::new(&self.network_context).debug_error(&err),
                    "{} Received error: {}", self.network_context, err
                );
            }
        }
    }

    // Chooses a random connected neighbour.
    fn choose_random_neighbor(&mut self) -> Option<PeerId> {
        if !self.connected_peers.is_empty() {
            let peers: Vec<_> = self.connected_peers.iter().cloned().collect();
            let idx = self.rng.gen_range(0, peers.len());
            Some(peers[idx])
        } else {
            None
        }
    }

    // Creates GossipDiscoveryMsg to be sent to some remote peer.
    fn compose_discovery_msg(&self) -> GossipDiscoveryMsg {
        let notes = self.known_peers.values().cloned().collect::<Vec<_>>();
        GossipDiscoveryMsg { notes }
    }

    // Updates local state by reconciling with notes received from some remote peer.
    async fn reconcile(&mut self, remote_peer: PeerId, remote_notes: Vec<Note>) {
        let mut change_detected = false;
        // If a peer is previously unknown, or has a newer epoch number, we update its
        // corresponding entry in the map.
        for mut note in remote_notes {
            match self.known_peers.get_mut(&note.peer_id) {
                // If we know about this peer, and receive the same or an older epoch, we do
                // nothing.
                Some(ref curr_note) if note.epoch() <= curr_note.epoch() => {
                    if note.epoch() < curr_note.epoch() {
                        debug!(
                            NetworkSchema::new(&self.network_context).remote_peer(&remote_peer),
                            note_peer = note.peer_id,
                            "{} Received stale note for peer: {} from peer: {}",
                            self.network_context,
                            note.peer_id.short_str(),
                            remote_peer.short_str()
                        );
                    }
                    continue;
                }
                _ => {
                    info!(
                        NetworkSchema::new(&self.network_context).remote_peer(&remote_peer),
                        note_peer = note.peer_id,
                        "{} Received updated note for peer: {} from peer: {}",
                        self.network_context,
                        note.peer_id.short_str(),
                        remote_peer.short_str()
                    );
                    // It is unlikely that we receive a note with a higher epoch number on us than
                    // what we ourselves have broadcasted. However, this can happen in case of
                    // the clock being reset, or the validator process being restarted on a node
                    // with clock behind the previous node. In such scenarios, it's best to issue a
                    // newer note with an epoch number higher than what we observed (unless the
                    // issued epoch number is u64::MAX).
                    if note.peer_id == self.network_context.peer_id() {
                        info!(
                            NetworkSchema::new(&self.network_context),
                            previous_epoch = note.epoch(),
                            current_epoch = self.note.epoch(),
                            "{} Received an older note for self, but with higher epoch. \
                             Previous epoch: {}, current epoch: {}",
                            self.network_context,
                            note.epoch(),
                            self.note.epoch()
                        );
                        if note.epoch() == std::u64::MAX {
                            continue;
                        }
                        note = Note::new(
                            self.network_context.peer_id(),
                            self.note.addrs().clone(),
                            &self.dns_seed_addr,
                            max(note.epoch() + 1, get_unix_epoch()),
                        );
                        self.note = note.clone();
                    } else {
                        change_detected = true;
                    }
                    // Update internal state of the peer with new Note.
                    self.known_peers.insert(note.peer_id, note);
                }
            }
        }

        if change_detected {
            self.conn_mgr_reqs_tx
                .send(ConnectivityRequest::UpdateAddresses(
                    DiscoverySource::Gossip,
                    self.known_peers
                        .clone()
                        .iter()
                        .map(|(peer_id, note)| (*peer_id, note.addrs().clone()))
                        .collect(),
                ))
                .await
                .expect("ConnectivityRequest::UpdateAddresses send");
        }
    }

    // Record the number of discovery notes we have for _other_ peers
    // (not including our own note). We exclude counting our own note to be
    // consistent with the "connected_peers" metric.
    fn record_num_discovery_notes(&self) {
        let num_other_notes = self
            .known_peers
            .iter()
            .filter(|(peer_id, _)| *peer_id != &self.network_context.peer_id())
            .count();
        let num_other_notes: i64 = num_other_notes.try_into().unwrap_or(0);

        counters::LIBRA_NETWORK_DISCOVERY_NOTES
            .with_label_values(&[self.network_context.role().as_str()])
            .set(num_other_notes);
    }
}

/// A Discovery message contains notes collected from other peers within the system.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GossipDiscoveryMsg {
    notes: Vec<Note>,
}

/// A `Note` contains a validator's signed `PeerInfo` as well as a signed
/// `FullNodePayload`, which provides relevant discovery info for public full
/// nodes and clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Note {
    /// Id of the peer.
    peer_id: PeerId,
    /// The validator node's signed `PeerInfo`.
    peer_info: PeerInfo,
    /// The validator node's signed `FullNodePayload`.
    full_node_info: FullNodeInfo,
}

impl Note {
    fn new(peer_id: PeerId, addrs: Vec<NetworkAddress>, dns_seed_addr: &[u8], epoch: u64) -> Self {
        Self {
            peer_id,
            peer_info: PeerInfo { addrs, epoch },
            full_node_info: FullNodeInfo {
                dns_seed_addr: dns_seed_addr.to_vec(),
                epoch,
            },
        }
    }

    /// Shortcut to the addrs embedded within the Note
    fn addrs(&self) -> &Vec<NetworkAddress> {
        &self.peer_info.addrs
    }

    /// The current implementation derives epoch from the PeerInfo.
    fn epoch(&self) -> u64 {
        self.peer_info.epoch
    }
}

/// A `PeerInfo` represents the network address(es) of a Peer.
#[derive(Clone, Debug, Deserialize, Serialize, CryptoHasher, LCSCryptoHash)]
pub struct PeerInfo {
    /// Network addresses this peer can be reached at.
    addrs: Vec<NetworkAddress>,
    /// Monotonically increasing incarnation number used to allow peers to issue
    /// updates to their `PeerInfo` and prevent attackers from propagating old
    /// `PeerInfo`s. This is usually a timestamp.
    epoch: u64,
}

/// Discovery information relevant to public full nodes and clients.
#[derive(Clone, Debug, Deserialize, Serialize, CryptoHasher, LCSCryptoHash)]
pub struct FullNodeInfo {
    /// The DNS domain name other public full nodes should query to get this
    /// validator's list of full nodes.
    dns_seed_addr: Vec<u8>,
    /// Monotonically increasing incarnation number used to allow peers to issue
    /// updates to their `FullNodePayload` and prevent attackers from propagating
    /// old `FullNodePayload`s. This is usually a timestamp.
    epoch: u64,
}

fn get_unix_epoch() -> u64 {
    // TODO: Currently, SystemTime::now() in Rust is not guaranteed to use a monotonic clock.
    // At the moment, it's unclear how to do this in a platform-agnostic way. For Linux, we
    // could use something like the [timerfd trait](https://docs.rs/crate/timerfd/1.0.0).
    libra_time::duration_since_epoch().as_millis() as u64
}
