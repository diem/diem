// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol to discover network addresses of other peers on the Libra network
//!
//! ## Implementation
//!
//! The discovery module is implemented as a stand-alone actor in the Network sub-system of the
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
    connectivity_manager::ConnectivityRequest,
    counters,
    error::{NetworkError, NetworkErrorKind},
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{Event, NetworkEvents, NetworkSender},
    validator_network::network_builder::NetworkBuilder,
    NetworkPublicKeys, ProtocolId,
};
use bytes::Bytes;
use channel::message_queues::QueueStyle;
use futures::{
    sink::SinkExt,
    stream::{FusedStream, Stream, StreamExt},
};
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::{CryptoHasher, DiscoveryMsgHasher},
    HashValue, Signature, SigningKey,
};
use libra_logger::prelude::*;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use rand::{rngs::SmallRng, FromEntropy, Rng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::{Arc, RwLock},
    time::SystemTime,
};

#[cfg(test)]
mod test;

/// The interface from Network to Discovery module.
///
/// `DiscoveryNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `DiscoveryMsg` types. `DiscoveryNetworkEvents` is a thin wrapper
/// around a `channel::Receiver<PeerManagerNotification>`.
pub type DiscoveryNetworkEvents = NetworkEvents<DiscoveryMsg>;

/// The interface from Discovery to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<Discoverymsg>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `DiscoveryNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct DiscoveryNetworkSender {
    inner: NetworkSender<DiscoveryMsg>,
}

/// Register the discovery sender and event handler with network and return interfaces for those
/// actors.
pub fn add_to_network(
    network: &mut NetworkBuilder,
) -> (DiscoveryNetworkSender, DiscoveryNetworkEvents) {
    let (sender, receiver, connection_reqs_tx, connection_notifs_rx) = network
        .add_protocol_handler(
            vec![],
            vec![ProtocolId::DiscoveryDirectSend],
            QueueStyle::LIFO,
            Some(&counters::PENDING_DISCOVERY_NETWORK_EVENTS),
        );
    (
        DiscoveryNetworkSender::new(sender, connection_reqs_tx),
        DiscoveryNetworkEvents::new(receiver, connection_notifs_rx),
    )
}

impl DiscoveryNetworkSender {
    /// Create a new Discovery sender
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }

    /// Send a DiscoveryMsg to a peer.
    pub fn send_to(&mut self, peer: PeerId, msg: DiscoveryMsg) -> Result<(), NetworkError> {
        self.inner
            .send_to(peer, ProtocolId::DiscoveryDirectSend, msg)
    }
}

/// The actor running the discovery protocol.
pub struct Discovery<TTicker> {
    /// Note for self, which is prefixed with an underscore as this is not used but is in
    /// preparation for logic that changes the advertised Note while the validator is running.
    note: VerifiedNote,
    /// PeerId for self.
    peer_id: PeerId,
    /// Our node type.
    role: RoleType,
    /// The DNS domain name other public full nodes should query to get this
    /// validator's list of full nodes.
    dns_seed_addr: Bytes,
    /// Validator for verifying signatures on messages.
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    /// Ed25519PrivateKey for signing notes.
    signer: Ed25519PrivateKey,
    /// Current state, maintaining the most recent Note for each peer, alongside parsed PeerInfo.
    known_peers: HashMap<PeerId, VerifiedNote>,
    /// Currently connected peers.
    connected_peers: HashSet<PeerId>,
    /// Ticker to trigger state send to a random peer. In production, the ticker is likely to be
    /// fixed duration interval timer.
    ticker: TTicker,
    /// Handle to send requests to Network.
    network_reqs_tx: DiscoveryNetworkSender,
    /// Handle to receive notifications from Network.
    network_notifs_rx: DiscoveryNetworkEvents,
    /// Channel to send requests to ConnectivityManager.
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    /// Random-number generator.
    rng: SmallRng,
}

impl<TTicker> Discovery<TTicker>
where
    TTicker: Stream + FusedStream + Unpin,
{
    pub fn new(
        self_peer_id: PeerId,
        role: RoleType,
        self_addrs: Vec<Multiaddr>,
        signer: Ed25519PrivateKey,
        trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
        ticker: TTicker,
        network_reqs_tx: DiscoveryNetworkSender,
        network_notifs_rx: DiscoveryNetworkEvents,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        // TODO(philiphayes): wire through config
        let dns_seed_addr = b"example.com";

        let epoch = get_unix_epoch();
        let self_note = Note::new(&signer, self_peer_id, self_addrs, dns_seed_addr, epoch);
        let self_verified_note = VerifiedNote(self_note);

        let known_peers = vec![(self_peer_id, self_verified_note.clone())]
            .into_iter()
            .collect();

        Self {
            note: self_verified_note,
            peer_id: self_peer_id,
            role,
            dns_seed_addr: Bytes::from_static(dns_seed_addr),
            trusted_peers,
            signer,
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

        debug!("Starting Discovery actor event loop");
        loop {
            futures::select! {
                notif = self.network_notifs_rx.select_next_some() => {
                    self.handle_network_event(notif).await;
                },
                _ = self.ticker.select_next_some() => {
                    self.handle_tick();
                }
                complete => {
                    crit!("Discovery actor terminated");
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
        debug!("Discovery interval tick");
        // On each tick, we choose a random neighbor and push our state to it.
        if let Some(peer) = self.choose_random_neighbor() {
            // We clone `peer_mgr_reqs_tx` member of Self, since using `self` inside fut below
            // triggers some lifetime errors.
            let mut sender = self.network_reqs_tx.clone();
            // Compose discovery msg to send.
            let msg = self.compose_discovery_msg();
            if let Err(err) = sender.send_to(peer, msg) {
                warn!(
                    "Failed to send discovery msg to {}; error: {:?}",
                    peer.short_str(),
                    err
                );
            };
        }
    }

    async fn handle_network_event(&mut self, event: Result<Event<DiscoveryMsg>, NetworkError>) {
        trace!("Network event::{:?}", event);
        match event {
            Ok(e) => {
                match e {
                    Event::NewPeer(peer_id) => {
                        // Add peer to connected peer list.
                        self.connected_peers.insert(peer_id);
                    }
                    Event::LostPeer(peer_id) => {
                        // Remove peer from connected peer list.
                        self.connected_peers.remove(&peer_id);
                    }
                    Event::Message((peer_id, msg)) => {
                        match handle_discovery_msg(msg, self.trusted_peers.clone(), peer_id) {
                            Ok(verified_notes) => {
                                self.reconcile(peer_id, verified_notes).await;
                                self.record_num_discovery_notes();
                            }
                            Err(e) => {
                                warn!(
                                    "Failure in processing stream from peer: {}. Error: {:?}",
                                    peer_id.short_str(),
                                    e
                                );
                            }
                        }
                    }
                    Event::RpcRequest(req) => {
                        warn!("Unexpected notification from network: {:?}", req);
                        debug_assert!(false);
                    }
                }
            }
            Err(err) => {
                info!("Received error: {}", err);
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

    // Creates DiscoveryMsg to be sent to some remote peer.
    fn compose_discovery_msg(&self) -> DiscoveryMsg {
        let notes = self
            .known_peers
            .values()
            .map(|note| note.as_note())
            .cloned()
            .collect::<Vec<_>>();
        DiscoveryMsg { notes }
    }

    // Updates local state by reconciling with notes received from some remote peer.
    // Assumption: `remote_notes` have already been verified for signature validity and content.
    async fn reconcile(&mut self, remote_peer: PeerId, remote_notes: Vec<VerifiedNote>) {
        // If a peer is previously unknown, or has a newer epoch number, we update its
        // corresponding entry in the map.
        for mut note in remote_notes.into_iter() {
            match self.known_peers.get_mut(&note.as_note().peer_id) {
                // If we know about this peer, and receive the same or an older epoch, we do
                // nothing.
                Some(ref curr_note) if note.as_note().epoch() <= curr_note.as_note().epoch() => {
                    if note.as_note().epoch() < curr_note.as_note().epoch() {
                        debug!(
                            "Received stale note for peer: {} from peer: {}",
                            note.as_note().peer_id.short_str(),
                            remote_peer
                        );
                    }
                    continue;
                }
                _ => {
                    info!(
                        "Received updated note for peer: {} from peer: {}",
                        note.as_note().peer_id.short_str(),
                        remote_peer.short_str()
                    );
                    // It is unlikely that we receive a note with a higher epoch number on us than
                    // what we ourselves have broadcasted. However, this can happen in case of
                    // the clock being reset, or the validator process being restarted on a node
                    // with clock behind the previous node. In such scenarios, it's best to issue a
                    // newer note with an epoch number higher than what we observed (unless the
                    // issued epoch number is u64::MAX).
                    if note.as_note().peer_id == self.peer_id {
                        info!(
                            "Received an older note for self, but with higher epoch. \
                             Previous epoch: {}, current epoch: {}",
                            note.as_note().epoch(),
                            self.note.as_note().epoch()
                        );
                        if note.as_note().epoch() == std::u64::MAX {
                            security_log(SecurityEvent::InvalidDiscoveryMsg)
                                .data(
                                    "Older note received for self has u64::MAX epoch. \
                                     This likely means that the node's network signing key has \
                                     been compromised.",
                                )
                                .log();
                            continue;
                        }
                        let unverified_note = Note::new(
                            &self.signer,
                            self.peer_id,
                            self.note.as_note().addrs().clone(),
                            &self.dns_seed_addr,
                            max(note.as_note().epoch() + 1, get_unix_epoch()),
                        );
                        self.note = VerifiedNote(unverified_note);
                        note = self.note.clone();
                    } else {
                        // The multiaddrs in the peer's discovery Note.
                        let peer_addrs = note.as_note().addrs().clone();

                        self.conn_mgr_reqs_tx
                            .send(ConnectivityRequest::UpdateAddresses(
                                note.as_note().peer_id,
                                peer_addrs,
                            ))
                            .await
                            .expect("ConnectivityRequest::UpdateAddresses send");
                    }
                    // Update internal state of the peer with new Note.
                    self.known_peers.insert(note.as_note().peer_id, note);
                }
            }
        }
    }

    // Record the number of verified discovery notes we have for _other_ peers
    // (not including our own note). We exclude counting our own note to be
    // consistent with the "connected_peers" metric.
    fn record_num_discovery_notes(&self) {
        let num_other_notes = self
            .known_peers
            .iter()
            .filter(|(peer_id, _)| *peer_id != &self.peer_id)
            .count();
        let num_other_notes: i64 = num_other_notes.try_into().unwrap_or(0);

        counters::LIBRA_NETWORK_DISCOVERY_NOTES
            .with_label_values(&[self.role.as_str()])
            .set(num_other_notes);
    }
}

/// A Discovery message contains notes collected from other peers within the system.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryMsg {
    notes: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifiedNote(Note);

impl VerifiedNote {
    /// Give access to the internal note
    fn as_note(&self) -> &Note {
        &self.0
    }
}

/// A `Note` contains a validator's signed `PeerInfo` as well as a signed
/// `FullNodePayload`, which provides relevant discovery info for public full
/// nodes and clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Note {
    /// Id of the peer.
    peer_id: PeerId,
    /// The validator node's signed `PeerInfo`.
    signed_peer_info: SignedPeerInfo,
    /// The validator node's signed `FullNodePayload`.
    signed_full_node_info: SignedFullNodeInfo,
}

impl Note {
    fn new(
        signer: &Ed25519PrivateKey,
        peer_id: PeerId,
        addrs: Vec<Multiaddr>,
        dns_seed_addr: &[u8],
        epoch: u64,
    ) -> Self {
        let peer_info = PeerInfo { addrs, epoch };
        let signed_peer_info = peer_info.sign(&signer);

        let full_node_info = FullNodeInfo {
            dns_seed_addr: dns_seed_addr.to_vec(),
            epoch,
        };
        let signed_full_node_info = full_node_info.sign(&signer);

        Note {
            peer_id,
            signed_peer_info,
            signed_full_node_info,
        }
    }

    /// Verifies validity of notes. Most fields have already been verified during initial
    /// deserialization, so this only verifies that the note contains properly signed infos.
    fn verify(self, pub_key: &Ed25519PublicKey) -> Result<VerifiedNote, NetworkError> {
        self.signed_peer_info.verify(&pub_key)?;
        self.signed_full_node_info.verify(&pub_key)?;
        Ok(VerifiedNote(self))
    }

    /// Shortcut to the addrs embedded within the Note
    fn addrs(&self) -> &Vec<Multiaddr> {
        &self.signed_peer_info.peer_info.addrs
    }

    /// The current implementation derives epoch from the PeerInfo.
    fn epoch(&self) -> u64 {
        self.signed_peer_info.peer_info.epoch
    }
}

/// A `PeerInfo` authenticated by the peer's root `network_signing_key` stored on-chain.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedPeerInfo {
    /// A `PeerInfo` represents the network address(es) of a Peer.
    peer_info: PeerInfo,
    /// A signature over the above serialzed `PeerInfo`, signed by the validator's
    /// `network_signing_key` referred to by the `peer_id` account address.
    signature: Ed25519Signature,
}

impl SignedPeerInfo {
    fn verify(&self, pub_key: &Ed25519PublicKey) -> Result<(), NetworkError> {
        let peer_info_bytes =
            lcs::to_bytes(&self.peer_info).map_err(|_| NetworkErrorKind::ParsingError)?;
        self.signature
            .verify(&get_hash(&peer_info_bytes), &pub_key)
            .map_err(|_| NetworkErrorKind::SignatureError)?;
        Ok(())
    }
}

/// A `PeerInfo` represents the network address(es) of a Peer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    /// Network addresses this peer can be reached at.
    addrs: Vec<Multiaddr>,
    /// Monotonically increasing incarnation number used to allow peers to issue
    /// updates to their `PeerInfo` and prevent attackers from propagating old
    /// `PeerInfo`s. This is usually a timestamp.
    epoch: u64,
}

impl PeerInfo {
    fn sign(self, signer: &Ed25519PrivateKey) -> SignedPeerInfo {
        let peer_info_bytes = lcs::to_bytes(&self).expect("LCS serialization fails");
        let signature = signer.sign_message(&get_hash(&peer_info_bytes));
        SignedPeerInfo {
            peer_info: self,
            signature,
        }
    }
}

/// A `FullNodeInfo` authenticated by the peer's root `network_signing_key` stored on-chain.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedFullNodeInfo {
    full_node_info: FullNodeInfo,
    /// A signature over the above serialzed `FullNodeInfo`, signed by the validator's
    /// `network_signing_key` referred to by the `peer_id` account address.
    signature: Ed25519Signature,
}

impl SignedFullNodeInfo {
    fn verify(&self, pub_key: &Ed25519PublicKey) -> Result<(), NetworkError> {
        let full_node_info_bytes =
            lcs::to_bytes(&self.full_node_info).map_err(|_| NetworkErrorKind::ParsingError)?;
        self.signature
            .verify(&get_hash(&full_node_info_bytes), &pub_key)
            .map_err(|_| NetworkErrorKind::SignatureError)?;
        Ok(())
    }
}

/// Discovery information relevant to public full nodes and clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullNodeInfo {
    /// The DNS domain name other public full nodes should query to get this
    /// validator's list of full nodes.
    dns_seed_addr: Vec<u8>,
    /// Monotonically increasing incarnation number used to allow peers to issue
    /// updates to their `FullNodePayload` and prevent attackers from propagating
    /// old `FullNodePayload`s. This is usually a timestamp.
    epoch: u64,
}

impl FullNodeInfo {
    fn sign(self, signer: &Ed25519PrivateKey) -> SignedFullNodeInfo {
        let full_node_info_bytes = lcs::to_bytes(&self).expect("LCS serialization failed");
        let signature = signer.sign_message(&get_hash(&full_node_info_bytes));
        SignedFullNodeInfo {
            full_node_info: self,
            signature,
        }
    }
}

fn get_unix_epoch() -> u64 {
    // TODO: Currently, SystemTime::now() in Rust is not guaranteed to use a monotonic clock.
    // At the moment, it's unclear how to do this in a platform-agnostic way. For Linux, we
    // could use something like the [timerfd trait](https://docs.rs/crate/timerfd/1.0.0).
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System clock reset to before unix epoch")
        .as_millis() as u64
}

// Handles an inbound message from a remote peer as follows:
// Verifies signatures on all notes contained in the message.
fn handle_discovery_msg(
    msg: DiscoveryMsg,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    peer_id: PeerId,
) -> Result<Vec<VerifiedNote>, NetworkError> {
    // Check that all received `Note`s are valid -- reject the whole message
    // if any `Note` is invalid.
    let mut verified_notes = vec![];
    for note in msg.notes.into_iter() {
        let rlock = trusted_peers.read().unwrap();
        let pub_key = rlock
            .get(&note.peer_id)
            .ok_or_else(|| NetworkErrorKind::SignatureError)?;
        let pub_key = &pub_key.signing_public_key;
        let verified_note = note.verify(&pub_key).map_err(|err| {
            security_log(SecurityEvent::InvalidDiscoveryMsg)
                .error(&err)
                .data(&peer_id)
                .data(&trusted_peers)
                .log();
            err
        })?;
        verified_notes.push(verified_note);
    }
    Ok(verified_notes)
}

fn get_hash(msg: &[u8]) -> HashValue {
    let mut hasher = DiscoveryMsgHasher::default();
    hasher.write(msg);
    hasher.finish()
}
