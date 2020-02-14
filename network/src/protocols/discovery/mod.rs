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
    proto::{DiscoveryMsg, FullNodePayload, Note, PeerInfo, SignedFullNodePayload, SignedPeerInfo},
    utils::MessageExt,
    validator_network::{DiscoveryNetworkEvents, DiscoveryNetworkSender, Event},
    NetworkPublicKeys,
};
use anyhow::anyhow;
use bytes::Bytes;
use channel;
use futures::{
    sink::SinkExt,
    stream::{FusedStream, Stream, StreamExt},
};
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::*,
    hash::{CryptoHasher, DiscoveryMsgHasher},
    HashValue, Signature,
};
use libra_logger::prelude::*;
use libra_types::{crypto_proxies::ValidatorSigner as Signer, PeerId};
use parity_multiaddr::Multiaddr;
use prost::Message;
use rand::{rngs::SmallRng, SeedableRng, Rng};
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    sync::{Arc, RwLock},
    time::SystemTime,
};

#[cfg(test)]
mod test;

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
    /// Signer for signing notes.
    signer: Signer,
    /// Current state, maintaining the most recent Note for each peer, alongside parsed PeerInfo.
    known_peers: HashMap<PeerId, VerifiedNote>,
    /// Info for seed peers.
    seed_peers: HashMap<PeerId, PeerInfo>,
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
        signer: Signer,
        seed_peers: HashMap<PeerId, PeerInfo>,
        trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
        ticker: TTicker,
        network_reqs_tx: DiscoveryNetworkSender,
        network_notifs_rx: DiscoveryNetworkEvents,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        // TODO(philiphayes): wire through config
        let dns_seed_addr = b"example.com";

        let epoch = get_unix_epoch();
        let self_verified_note =
            create_self_verified_note(&signer, self_peer_id, self_addrs, dns_seed_addr, epoch);

        let known_peers = vec![(self_peer_id, self_verified_note.clone())]
            .into_iter()
            .collect();

        Self {
            note: self_verified_note,
            peer_id: self_peer_id,
            role,
            dns_seed_addr: Bytes::from_static(dns_seed_addr),
            seed_peers,
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

    // Connect with all the seed peers. If current node is also a seed peer, remove it from the
    // list.
    async fn connect_to_seed_peers(&mut self) {
        debug!("Connecting to {} seed peers", self.seed_peers.len());
        self.record_num_discovery_notes();
        let self_peer_id = self.peer_id;
        for (peer_id, peer_info) in self
            .seed_peers
            .iter()
            .filter(|(peer_id, _)| **peer_id != self_peer_id)
        {
            self.conn_mgr_reqs_tx
                .send(ConnectivityRequest::UpdateAddresses(
                    *peer_id,
                    peer_info
                        .addrs
                        .iter()
                        .cloned()
                        .map(|addr| Multiaddr::try_from(addr).expect("Multiaddr parsing failed"))
                        .collect(),
                ))
                .await
                .expect("ConnectivityRequest::UpdateAddresses send");
        }
    }

    // Starts the main event loop for the discovery actor. We bootstrap by first dialing all the
    // seed peers, and then entering the event handling loop. Messages are received from:
    // - a ticker to trigger discovery message send to a random connected peer
    // - an incoming message from a peer wishing to send its state
    // - an internal task once it has processed incoming messages from a peer, and wishes for
    // discovery actor to update its state.
    pub async fn start(mut self) {
        // Bootstrap by connecting to seed peers.
        self.connect_to_seed_peers().await;
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

    async fn handle_network_event<'a>(
        &'a mut self,
        event: Result<Event<DiscoveryMsg>, NetworkError>,
    ) {
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
        let mut msg = DiscoveryMsg::default();
        for verified_note in self.known_peers.values() {
            msg.notes.push(verified_note.raw_note.clone());
        }
        msg
    }

    // Updates local state by reconciling with notes received from some remote peer.
    // Assumption: `remote_notes` have already been verified for signature validity and content.
    async fn reconcile(&mut self, remote_peer: PeerId, remote_notes: Vec<VerifiedNote>) {
        // If a peer is previously unknown, or has a newer epoch number, we update its
        // corresponding entry in the map.
        for mut note in remote_notes.into_iter() {
            match self.known_peers.get_mut(&note.peer_id) {
                // If we know about this peer, and receive the same or an older epoch, we do
                // nothing.
                Some(ref curr_note) if note.epoch <= curr_note.epoch => {
                    if note.epoch < curr_note.epoch {
                        debug!(
                            "Received stale note for peer: {} from peer: {}",
                            note.peer_id.short_str(),
                            remote_peer
                        );
                    }
                    continue;
                }
                _ => {
                    info!(
                        "Received updated note for peer: {} from peer: {}",
                        note.peer_id.short_str(),
                        remote_peer.short_str()
                    );
                    // It is unlikely that we receive a note with a higher epoch number on us than
                    // what we ourselves have broadcasted. However, this can happen in case of
                    // the clock being reset, or the validator process being restarted on a node
                    // with clock behind the previous node. In such scenarios, it's best to issue a
                    // newer note with an epoch number higher than what we observed (unless the
                    // issued epoch number is u64::MAX).
                    if note.peer_id == self.peer_id {
                        info!(
                            "Received an older note for self, but with higher epoch. \
                             Previous epoch: {}, current epoch: {}",
                            note.epoch, self.note.epoch
                        );
                        if note.epoch == std::u64::MAX {
                            security_log(SecurityEvent::InvalidDiscoveryMsg)
                                .data(
                                    "Older note received for self has u64::MAX epoch. \
                                     This likely means that the node's network signing key has \
                                     been compromised.",
                                )
                                .log();
                            continue;
                        }
                        note = create_self_verified_note(
                            &self.signer,
                            self.peer_id,
                            self.note.addrs.clone(),
                            &self.dns_seed_addr,
                            max(note.epoch + 1, get_unix_epoch()),
                        );
                        self.note = note.clone();
                    } else {
                        // The multiaddrs in the peer's discovery Note.
                        let mut peer_addrs: Vec<Multiaddr> = note.addrs.clone();

                        // Append the addrs in the seed PeerInfo if this peer is
                        // configured as one of our seed peers.
                        if let Some(seed_info) = self.seed_peers.get(&note.peer_id) {
                            let seed_addrs_iter = seed_info.addrs.iter().cloned().map(|addr| {
                                Multiaddr::try_from(addr).expect("Multiaddr parsing fails")
                            });
                            peer_addrs.extend(seed_addrs_iter);
                        }

                        self.conn_mgr_reqs_tx
                            .send(ConnectivityRequest::UpdateAddresses(
                                note.peer_id,
                                peer_addrs,
                            ))
                            .await
                            .expect("ConnectivityRequest::UpdateAddresses send");
                    }
                    // Update internal state of the peer with new Note.
                    self.known_peers.insert(note.peer_id, note);
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

/// The note which has been verified its validity
#[derive(Clone)]
struct VerifiedNote {
    peer_id: PeerId,
    addrs: Vec<Multiaddr>,
    epoch: u64,
    /// the raw `Note` sent from remote
    raw_note: Note,
}

// Creates a PeerInfo combining the given addresses with the current unix timestamp as epoch.
fn create_peer_info(addrs: Vec<Multiaddr>, epoch: u64) -> PeerInfo {
    let mut peer_info = PeerInfo::default();
    peer_info.epoch = epoch;
    peer_info.addrs = addrs.into_iter().map(|addr| addr.as_ref().into()).collect();
    peer_info
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

fn create_full_node_payload(dns_seed_addr: &[u8], epoch: u64) -> FullNodePayload {
    let mut full_node_payload = FullNodePayload::default();
    full_node_payload.epoch = epoch;
    full_node_payload.dns_seed_addr = dns_seed_addr.into();
    full_node_payload
}

// Creates a note by signing the given peer info, and combining the signature, peer_info and
// peer_id into a note.
fn create_self_verified_note(
    signer: &Signer,
    self_peer_id: PeerId,
    self_addrs: Vec<Multiaddr>,
    dns_seed_addr: &[u8],
    epoch: u64,
) -> VerifiedNote {
    let note = create_note(
        signer,
        self_peer_id,
        self_addrs.clone(),
        dns_seed_addr,
        epoch,
    );
    // We don't verify the self note because trusted_peers may not be populated yet
    VerifiedNote {
        peer_id: self_peer_id,
        addrs: self_addrs,
        epoch,
        raw_note: note,
    }
}

fn create_note(
    signer: &Signer,
    self_peer_id: PeerId,
    self_addrs: Vec<Multiaddr>,
    dns_seed_addr: &[u8],
    epoch: u64,
) -> Note {
    let self_peer_info = create_peer_info(self_addrs, epoch);
    let self_full_node_payload = create_full_node_payload(dns_seed_addr, epoch);

    let peer_info_bytes = self_peer_info
        .to_bytes()
        .expect("Protobuf serialization fails");
    let peer_info_signature = sign(&signer, &peer_info_bytes);

    let mut signed_peer_info = SignedPeerInfo::default();
    signed_peer_info.peer_info = peer_info_bytes.to_vec();
    signed_peer_info.signature = peer_info_signature;

    let payload_bytes = self_full_node_payload
        .to_bytes()
        .expect("Protobuf serialization fails");
    let payload_signature = sign(&signer, &payload_bytes);

    let mut signed_full_node_payload = SignedFullNodePayload::default();
    signed_full_node_payload.payload = payload_bytes.to_vec();
    signed_full_node_payload.signature = payload_signature;

    let mut note = Note::default();
    note.peer_id = self_peer_id.into();
    note.signed_peer_info = Some(signed_peer_info);
    note.signed_full_node_payload = Some(signed_full_node_payload);

    note
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
    msg.notes.iter().try_for_each(|note| {
        verify_note(&note, &trusted_peers)
            .and_then(|verified_note| {
                verified_notes.push(verified_note);
                Ok(())
            })
            .map_err(|err| {
                security_log(SecurityEvent::InvalidDiscoveryMsg)
                    .error(&err)
                    .data(&peer_id)
                    .data(&note)
                    .data(&trusted_peers)
                    .log();
                err
            })
    })?;
    Ok(verified_notes)
}

// Verifies validity of notes. Following conditions should be met for validity:
// 1. We should be able to correctly parse the peer id in each note.
// 2. The signature of the serialized peer info should be valid for the given peer_id.
// 3. The address(es) in the PeerInfo should be correctly parsable as Multiaddrs.
// 4. The signature of the serialized full node payload should be valid for the given peer_id.
fn verify_note(
    note: &Note,
    trusted_peers: &RwLock<HashMap<PeerId, NetworkPublicKeys>>,
) -> Result<VerifiedNote, NetworkError> {
    // validate PeerId

    let peer_id = PeerId::try_from(note.peer_id.clone())
        .map_err(|err| anyhow!(err).context(NetworkErrorKind::ParsingError))?;

    // validate PeerInfo

    let signed_peer_info = note.signed_peer_info.as_ref().ok_or_else(|| {
        anyhow!("Discovery Note missing signed_peer_info field")
            .context(NetworkErrorKind::ParsingError)
    })?;
    let peer_info_bytes = &signed_peer_info.peer_info;
    let peer_info_signature = &signed_peer_info.signature;
    verify_signature(
        trusted_peers,
        peer_id,
        &peer_info_signature,
        &peer_info_bytes,
    )?;

    let peer_info = PeerInfo::decode(peer_info_bytes.as_ref())?;
    let mut verified_addrs = vec![];
    for addr in &peer_info.addrs {
        verified_addrs.push(Multiaddr::try_from(addr.clone())?)
    }

    // validate FullNodePayload (optional)
    // TODO(philiphayes): actually use the FullNodePayload

    if let Some(signed_full_node_payload) = &note.signed_full_node_payload {
        verify_signature(
            trusted_peers,
            peer_id,
            &signed_full_node_payload.signature,
            &signed_full_node_payload.payload,
        )?;

        let _ = FullNodePayload::decode(signed_full_node_payload.payload.as_ref())?;

        // TODO(philiphayes): validate internal fields
    }

    Ok(VerifiedNote {
        peer_id,
        addrs: verified_addrs,
        epoch: peer_info.epoch,
        raw_note: note.clone(),
    })
}

fn get_hash(msg: &[u8]) -> HashValue {
    let mut hasher = DiscoveryMsgHasher::default();
    hasher.write(msg);
    hasher.finish()
}

fn verify_signature(
    trusted_peers: &RwLock<HashMap<PeerId, NetworkPublicKeys>>,
    signer: PeerId,
    signature: &[u8],
    msg: &[u8],
) -> Result<(), NetworkError> {
    let rlock = trusted_peers.read().unwrap();
    let pub_key = rlock
        .get(&signer)
        .ok_or_else(|| NetworkErrorKind::SignatureError)?;
    let signature = Ed25519Signature::try_from(signature)
        .map_err(|err| anyhow!(err).context(NetworkErrorKind::SignatureError))?;
    signature
        .verify(&get_hash(msg), &pub_key.signing_public_key)
        .map_err(|_| NetworkErrorKind::SignatureError)?;
    Ok(())
}

fn sign(signer: &Signer, msg: &[u8]) -> Vec<u8> {
    let signature: Ed25519Signature = signer
        .sign_message(get_hash(msg))
        .expect("Message signing fails");
    signature.to_bytes().to_vec()
}
