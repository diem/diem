// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The ConnectivityManager actor is responsible for ensuring that we are
//! connected to a node if and only if it is an eligible node.
//!
//! A list of eligible nodes is received at initialization, and updates are
//! received on changes to system membership. In our current system design, the
//! Consensus actor informs the ConnectivityManager of eligible nodes.
//!
//! Different discovery sources notify the ConnectivityManager of updates to
//! peers' addresses. Currently, there are 2 discovery sources (ordered by
//! decreasing dial priority, i.e., first is highest priority):
//!
//! 1. Onchain discovery protocol
//! 2. Seed peers from config
//!
//! In other words, if a we have some addresses discovered via onchain discovery
//! and some seed addresses from our local config, we will try the onchain
//! discovery addresses first and the local seed addresses after.
//!
//! When dialing a peer with a given list of addresses, we attempt each address
//! in order with a capped exponential backoff delay until we eventually connect
//! to the peer. The backoff is capped since, for validators specifically, it is
//! absolutely important that we maintain connectivity with all peers and heal
//! any partitions asap, as we aren't currently gossiping consensus messages or
//! using a relay protocol.

use crate::{
    counters,
    logging::NetworkSchema,
    peer_manager::{self, conn_notifs_channel, ConnectionRequestSender, PeerManagerError},
};
use diem_config::network_id::NetworkContext;
use diem_crypto::x25519;
use diem_infallible::RwLock;
use diem_logger::prelude::*;
use diem_network_address::NetworkAddress;
use diem_types::PeerId;
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
};
use num_variants::NumVariants;
use rand::{
    prelude::{SeedableRng, SmallRng},
    seq::SliceRandom,
};
use serde::{export::Formatter, Serialize};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fmt, mem,
    sync::Arc,
    time::Duration,
};
use tokio::{time, time::Instant};
use tokio_retry::strategy::jitter;

pub mod builder;
#[cfg(test)]
mod test;

/// The ConnectivityManager actor.
pub struct ConnectivityManager<TTicker, TBackoff> {
    network_context: Arc<NetworkContext>,
    /// Nodes which are eligible to join the network.
    eligible: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
    /// PeerId and address of remote peers to which this peer is connected.
    connected: HashMap<PeerId, NetworkAddress>,
    /// Addresses of peers received from discovery sources.
    peer_addrs: PeerAddresses,
    /// Public key sets of peers received from discovery sources.
    peer_pubkeys: PeerPublicKeys,
    /// Ticker to trigger connectivity checks to provide the guarantees stated above.
    ticker: TTicker,
    /// Channel to send connection requests to PeerManager.
    connection_reqs_tx: ConnectionRequestSender,
    /// Channel to receive notifications from PeerManager.
    connection_notifs_rx: conn_notifs_channel::Receiver,
    /// Channel over which we receive requests from other actors.
    requests_rx: channel::Receiver<ConnectivityRequest>,
    /// Peers queued to be dialed, potentially with some delay. The dial can be canceled by
    /// sending over (or dropping) the associated oneshot sender.
    dial_queue: HashMap<PeerId, oneshot::Sender<()>>,
    /// The state of any currently executing dials. Used to keep track of what
    /// the next dial delay and dial address should be for a given peer.
    dial_states: HashMap<PeerId, DialState<TBackoff>>,
    /// Backoff strategy.
    backoff_strategy: TBackoff,
    /// Maximum delay b/w 2 consecutive attempts to connect with a disconnected peer.
    max_delay_ms: u64,
    /// A local counter incremented on receiving an incoming message. Printing this in debugging
    /// allows for easy debugging.
    event_id: u32,
    /// A way to limit the number of connected peers by outgoing dials.
    outbound_connection_limit: Option<usize>,
    /// Random for shuffling which peers will be dialed
    rng: SmallRng,
}

/// Different sources for peer addresses, ordered by priority (Onchain=highest,
/// Config=lowest).
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, NumVariants, Serialize)]
pub enum DiscoverySource {
    OnChain,
    Config,
}

impl fmt::Debug for DiscoverySource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for DiscoverySource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiscoverySource::OnChain => "OnChain",
                DiscoverySource::Config => "Config",
            }
        )
    }
}

/// Requests received by the [`ConnectivityManager`] manager actor from upstream modules.
#[derive(Debug, Serialize)]
pub enum ConnectivityRequest {
    /// Request to update known addresses of peer with id `PeerId` to given list.
    UpdateAddresses(DiscoverySource, HashMap<PeerId, Vec<NetworkAddress>>),
    /// Update set of nodes eligible to join the network.
    UpdateEligibleNodes(DiscoverySource, HashMap<PeerId, HashSet<x25519::PublicKey>>),
    /// Gets current size of connected peers. This is useful in tests.
    #[serde(skip)]
    GetConnectedSize(oneshot::Sender<usize>),
    /// Gets current size of dial queue. This is useful in tests.
    #[serde(skip)]
    GetDialQueueSize(oneshot::Sender<usize>),
}

/// The set of `NetworkAddress`'s for all peers.
#[derive(Serialize)]
struct PeerAddresses(HashMap<PeerId, Addresses>);

/// A set of `NetworkAddress`'s for a single peer, bucketed by DiscoverySource in
/// priority order.
#[derive(Clone, Default, Serialize)]
struct Addresses([Vec<NetworkAddress>; DiscoverySource::NUM_VARIANTS]);

/// The sets of `x25519::PublicKey`s for all peers.
struct PeerPublicKeys(HashMap<PeerId, PublicKeys>);

/// Sets of `x25519::PublicKey`s for a single peer, bucketed by DiscoverySource
/// in priority order.
#[derive(Default)]
struct PublicKeys([HashSet<x25519::PublicKey>; DiscoverySource::NUM_VARIANTS]);

#[derive(Debug)]
enum DialResult {
    Success,
    Cancelled,
    Failed(PeerManagerError),
}

/// The state needed to compute the next dial delay and dial addr for a given
/// peer.
#[derive(Debug, Clone)]
struct DialState<TBackoff> {
    /// The current state of this peer's backoff delay.
    backoff: TBackoff,
    /// The index of the next address to dial. Index of an address in the peer's
    /// `peer_addrs` entry.
    addr_idx: usize,
}

/////////////////////////
// ConnectivityManager //
/////////////////////////

impl<TTicker, TBackoff> ConnectivityManager<TTicker, TBackoff>
where
    TTicker: Stream + FusedStream + Unpin + 'static,
    TBackoff: Iterator<Item = Duration> + Clone,
{
    /// Creates a new instance of the [`ConnectivityManager`] actor.
    pub fn new(
        network_context: Arc<NetworkContext>,
        eligible: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
        seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
        seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
        ticker: TTicker,
        connection_reqs_tx: ConnectionRequestSender,
        connection_notifs_rx: conn_notifs_channel::Receiver,
        requests_rx: channel::Receiver<ConnectivityRequest>,
        backoff_strategy: TBackoff,
        max_delay_ms: u64,
        outbound_connection_limit: Option<usize>,
    ) -> Self {
        assert!(
            eligible.read().is_empty(),
            "Eligible peers must be initially empty. eligible: {:?}",
            eligible
        );

        info!(
            NetworkSchema::new(&network_context),
            "{} Initialized connectivity manager", network_context
        );

        let mut connmgr = Self {
            network_context,
            eligible,
            connected: HashMap::new(),
            peer_addrs: PeerAddresses::new(),
            peer_pubkeys: PeerPublicKeys::new(),
            ticker,
            connection_reqs_tx,
            connection_notifs_rx,
            requests_rx,
            dial_queue: HashMap::new(),
            dial_states: HashMap::new(),
            backoff_strategy,
            max_delay_ms,
            event_id: 0,
            outbound_connection_limit,
            rng: SmallRng::from_entropy(),
        };

        // set the initial config addresses and pubkeys
        connmgr.handle_update_addresses(DiscoverySource::Config, seed_addrs);
        connmgr.handle_update_eligible_peers(DiscoverySource::Config, seed_pubkeys);

        connmgr
    }

    /// Starts the [`ConnectivityManager`] actor.
    pub async fn start(mut self) {
        // The ConnectivityManager actor is interested in 3 kinds of events:
        // 1. Ticks to trigger connecitvity check. These are implemented using a clock based
        //    trigger in production.
        // 2. Incoming requests to connect or disconnect with a peer.
        // 3. Notifications from PeerManager when we establish a new connection or lose an existing
        //    connection with a peer.
        let mut pending_dials = FuturesUnordered::new();

        // When we first startup, let's attempt to connect with our seed peers.
        self.check_connectivity(&mut pending_dials).await;

        info!(
            NetworkSchema::new(&self.network_context),
            "{} Starting ConnectivityManager actor", self.network_context
        );

        loop {
            self.event_id = self.event_id.wrapping_add(1);
            ::futures::select! {
                _ = self.ticker.select_next_some() => {
                    self.check_connectivity(&mut pending_dials).await;
                },
                req = self.requests_rx.select_next_some() => {
                    self.handle_request(req);
                },
                notif = self.connection_notifs_rx.select_next_some() => {
                    self.handle_control_notification(notif);
                },
                peer_id = pending_dials.select_next_some() => {
                    trace!(
                        NetworkSchema::new(&self.network_context)
                            .remote_peer(&peer_id),
                        "{} Dial complete to {}",
                        self.network_context,
                        peer_id.short_str(),
                    );
                    self.dial_queue.remove(&peer_id);
                },
                complete => {
                    break;
                }
            }
        }

        warn!(
            NetworkSchema::new(&self.network_context),
            "{} ConnectivityManager actor terminated", self.network_context
        );
    }

    /// Disconnect from all peers that are no longer eligible.
    ///
    /// For instance, a validator might leave the validator set after a
    /// reconfiguration. If we are currently connected to this validator, calling
    /// this function will close our connection to it.
    async fn close_stale_connections(&mut self) {
        let eligible = self.eligible.read().clone();
        let stale_connections: Vec<_> = self
            .connected
            .keys()
            .filter(|peer_id| !eligible.contains_key(peer_id))
            .cloned()
            .collect();
        for p in stale_connections.into_iter() {
            info!(
                NetworkSchema::new(&self.network_context).remote_peer(&p),
                "{} Closing stale connection to peer {}",
                self.network_context,
                p.short_str()
            );

            // Close existing connection.
            if let Err(e) = self.connection_reqs_tx.disconnect_peer(p).await {
                info!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&p),
                    error = %e,
                    "{} Failed to close stale connection to peer {} : {}",
                    self.network_context,
                    p.short_str(),
                    e
                );
            }
        }
    }

    /// Cancel all pending dials to peers that are no longer eligible.
    ///
    /// For instance, a validator might leave the validator set after a
    /// reconfiguration. If there is a pending dial to this validator, calling
    /// this function will remove it from the dial queue.
    async fn cancel_stale_dials(&mut self) {
        let eligible = self.eligible.read().clone();
        let stale_dials: Vec<_> = self
            .dial_queue
            .keys()
            .filter(|peer_id| !eligible.contains_key(peer_id))
            .cloned()
            .collect();

        for p in stale_dials.into_iter() {
            debug!(
                NetworkSchema::new(&self.network_context).remote_peer(&p),
                "{} Cancelling stale dial {}",
                self.network_context,
                p.short_str()
            );
            self.dial_queue.remove(&p);
        }
    }

    async fn dial_eligible_peers<'a>(
        &'a mut self,
        pending_dials: &'a mut FuturesUnordered<BoxFuture<'static, PeerId>>,
    ) {
        let eligible = self.eligible.read().clone();
        let to_connect: Vec<_> = self
            .peer_addrs
            .0
            .iter()
            .filter(|(peer_id, addrs)| {
                eligible.contains_key(peer_id)  // The node is eligible to be dialed.
                    && self.connected.get(peer_id).is_none() // The node is not already connected.
                    && self.dial_queue.get(peer_id).is_none() // There is no pending dial to this node.
                    && !addrs.is_empty() // There is an address to dial.
            })
            .collect();

        // Limit the number of dialed connections from a Full Node
        // This does not limit the number of incoming connections
        // It enforces that a full node cannot have more outgoing connections than `connection_limit`
        // including in flight dials.
        let to_connect_size = if let Some(conn_limit) = self.outbound_connection_limit {
            min(
                conn_limit.saturating_sub(self.connected.len() + self.dial_queue.len()),
                to_connect.len(),
            )
        } else {
            to_connect.len()
        };

        // The initial dial state; it has zero dial delay and uses the first
        // address.
        let init_dial_state = DialState::new(self.backoff_strategy.clone());

        for (p, addrs) in to_connect.choose_multiple(&mut self.rng, to_connect_size) {
            // If we're attempting to dial a Peer we must not be connected to it. This ensures that
            // newly eligible, but not connected to peers, have their counter initialized properly.
            counters::peer_connected(&self.network_context, p, 0);

            let mut connction_reqs_tx = self.connection_reqs_tx.clone();
            let peer_id = **p;
            let dial_state = self
                .dial_states
                .entry(peer_id)
                .or_insert_with(|| init_dial_state.clone());

            // Choose the next addr to dial for this peer. Currently, we just
            // round-robin the selection, i.e., try the sequence:
            // addr[0], .., addr[len-1], addr[0], ..
            let addr = dial_state.next_addr(&addrs).clone();

            // Using the DialState's backoff strategy, compute the delay until
            // the next dial attempt for this peer.
            let now = Instant::now();
            let dial_delay =
                dial_state.next_backoff_delay(Duration::from_millis(self.max_delay_ms));
            let f_delay = time::delay_for(dial_delay);

            let (cancel_tx, cancel_rx) = oneshot::channel();

            info!(
                NetworkSchema::new(&self.network_context)
                    .remote_peer(&peer_id)
                    .network_address(&addr),
                delay = dial_delay,
                "{} Create dial future {} at {} after {:?}",
                self.network_context,
                peer_id,
                addr,
                dial_delay
            );

            let network_context = self.network_context.clone();
            // Create future which completes by either dialing after calculated
            // delay or on cancellation.
            let f = async move {
                let delay = f_delay.deadline().duration_since(now);
                debug!(
                    NetworkSchema::new(&network_context)
                        .remote_peer(&peer_id)
                        .network_address(&addr),
                    delay = delay,
                    "{} Dial future started {} at {}",
                    network_context,
                    peer_id,
                    addr
                );

                // We dial after a delay. The dial can be canceled by sending to or dropping
                // `cancel_rx`.
                let dial_result = ::futures::select! {
                    _ = f_delay.fuse() => {
                        info!(
                            NetworkSchema::new(&network_context)
                                .remote_peer(&peer_id)
                                .network_address(&addr),
                            "{} Dialing peer {} at {}",
                            network_context,
                            peer_id,
                            addr
                        );
                        match connction_reqs_tx.dial_peer(peer_id, addr.clone()).await {
                            Ok(_) => DialResult::Success,
                            Err(e) => DialResult::Failed(e),
                        }
                    },
                    _ = cancel_rx.fuse() => {
                        DialResult::Cancelled
                    },
                };
                log_dial_result(network_context, peer_id, addr, dial_result);
                // Send peer_id as future result so it can be removed from dial queue.
                peer_id
            };
            pending_dials.push(f.boxed());
            self.dial_queue.insert(peer_id, cancel_tx);
        }
    }

    // Note: We do not check that the connections to older incarnations of a node are broken, and
    // instead rely on the node moving to a new epoch to break connections made from older
    // incarnations.
    async fn check_connectivity<'a>(
        &'a mut self,
        pending_dials: &'a mut FuturesUnordered<BoxFuture<'static, PeerId>>,
    ) {
        trace!(
            NetworkSchema::new(&self.network_context),
            "{} Checking connectivity",
            self.network_context
        );

        // Log the eligible peers with addresses from discovery
        sample!(SampleRate::Duration(Duration::from_secs(60)), {
            info!(
                NetworkSchema::new(&self.network_context),
                eligible_addrs = ?self.peer_addrs,
                eligible_keys = ?self.peer_pubkeys,
                "Current eligible peers"
            )
        });

        // Cancel dials to peers that are no longer eligible.
        self.cancel_stale_dials().await;
        // Disconnect from connected peers that are no longer eligible.
        self.close_stale_connections().await;
        // Dial peers which are eligible but are neither connected nor queued for dialing in the
        // future.
        self.dial_eligible_peers(pending_dials).await;
    }

    fn reset_dial_state(&mut self, peer_id: &PeerId) {
        if let Some(dial_state) = self.dial_states.get_mut(peer_id) {
            *dial_state = DialState::new(self.backoff_strategy.clone());
        }
    }

    fn handle_request(&mut self, req: ConnectivityRequest) {
        trace!(
            NetworkSchema::new(&self.network_context),
            connectivity_request = req,
            "{} Handling ConnectivityRequest",
            self.network_context
        );

        match req {
            ConnectivityRequest::UpdateAddresses(src, new_peer_addrs) => {
                trace!(
                    NetworkSchema::new(&self.network_context),
                    "{} Received updated list of peer addresses: src: {:?}",
                    self.network_context,
                    src,
                );
                self.handle_update_addresses(src, new_peer_addrs);
            }
            ConnectivityRequest::UpdateEligibleNodes(src, new_peer_pubkeys) => {
                trace!(
                    NetworkSchema::new(&self.network_context),
                    "{} Received updated list of eligible nodes: src: {:?}",
                    self.network_context,
                    src,
                );
                self.handle_update_eligible_peers(src, new_peer_pubkeys);
            }
            ConnectivityRequest::GetDialQueueSize(sender) => {
                sender.send(self.dial_queue.len()).unwrap();
            }
            ConnectivityRequest::GetConnectedSize(sender) => {
                sender.send(self.connected.len()).unwrap();
            }
        }
    }

    fn handle_update_addresses(
        &mut self,
        src: DiscoverySource,
        new_peer_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    ) {
        // TODO(philiphayes): do these two in next commit
        // 1. set peers not in update to empty
        // 2. remove all empty

        // Keep track of if any peer's addresses have actually changed, so we can
        // log without too much spam.
        let self_peer_id = self.network_context.peer_id();

        // 3. add or update intersection
        for (peer_id, new_addrs) in new_peer_addrs {
            // Do not include self_peer_id in the address list for dialing to
            // avoid pointless self-dials.
            if peer_id == self_peer_id {
                continue;
            }

            // Update peer's addresses
            let addrs = self.peer_addrs.0.entry(peer_id).or_default();
            if addrs.update(src, new_addrs) {
                info!(
                    NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                    network_addresses = addrs,
                    "{} addresses updated for peer: {}, update src: {:?}, addrs: {}",
                    self.network_context,
                    peer_id.short_str(),
                    src,
                    addrs,
                );

                // If we're currently trying to dial this peer, we reset their
                // dial state. As a result, we will begin our next dial attempt
                // from the first address (which might have changed) and from a
                // fresh backoff (since the current backoff delay might be maxed
                // out if we can't reach any of their previous addresses).
                self.reset_dial_state(&peer_id);
            }
        }
    }

    fn handle_update_eligible_peers(
        &mut self,
        src: DiscoverySource,
        new_peer_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
    ) {
        let mut have_any_changed = false;
        let self_peer_id = self.network_context.peer_id();

        // 1. set peer entries not in update to empty for this source
        for (peer_id, pubkeys) in self.peer_pubkeys.0.iter_mut() {
            if !new_peer_pubkeys.contains_key(peer_id) {
                have_any_changed |= pubkeys.update(src, HashSet::new());
            }
        }

        // 2. add or update pubkeys in intersection
        for (peer_id, new_pubkeys) in new_peer_pubkeys {
            if peer_id == self_peer_id {
                continue;
            }

            let pubkeys = self.peer_pubkeys.0.entry(peer_id).or_default();
            if pubkeys.update(src, new_pubkeys) {
                have_any_changed = true;
                info!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&peer_id)
                        .discovery_source(&src),
                    new_pubkeys = pubkeys.to_string(),
                    "{} pubkey sets updated for peer: {}, pubkeys: {}",
                    self.network_context,
                    peer_id.short_str(),
                    pubkeys
                );
                self.reset_dial_state(&peer_id);
            }
        }

        // 3. remove all peer entries where all sources are empty
        have_any_changed |= self.peer_pubkeys.remove_empty();

        // 4. set shared eligible peers to union
        if have_any_changed {
            // For each peer, union all of the pubkeys from each discovery source
            // to generate the new eligible peers set.
            let new_eligible = self.peer_pubkeys.union_all();

            // Swap in the new eligible peers set. Drop the old set after releasing
            // the write lock.
            let _old_eligible = {
                let mut eligible = self.eligible.write();
                mem::replace(&mut *eligible, new_eligible)
            };
        }
    }

    fn handle_control_notification(&mut self, notif: peer_manager::ConnectionNotification) {
        trace!(
            NetworkSchema::new(&self.network_context),
            connection_notification = notif,
            "Connection notification"
        );
        match notif {
            peer_manager::ConnectionNotification::NewPeer(peer_id, addr, _origin, _context) => {
                counters::peer_connected(&self.network_context, &peer_id, 1);
                self.connected.insert(peer_id, addr);

                // Cancel possible queued dial to this peer.
                self.dial_states.remove(&peer_id);
                self.dial_queue.remove(&peer_id);
            }
            peer_manager::ConnectionNotification::LostPeer(peer_id, addr, _origin, _reason) => {
                if let Some(stored_addr) = self.connected.get(&peer_id) {
                    // Remove node from connected peers list.

                    counters::peer_connected(&self.network_context, &peer_id, 0);

                    info!(
                        NetworkSchema::new(&self.network_context)
                            .remote_peer(&peer_id)
                            .network_address(&addr),
                        stored_addr = stored_addr,
                        "{} Removing peer '{}' addr: {}, vs event addr: {}",
                        self.network_context,
                        peer_id.short_str(),
                        stored_addr,
                        addr
                    );
                    self.connected.remove(&peer_id);
                } else {
                    info!(
                        NetworkSchema::new(&self.network_context)
                            .remote_peer(&peer_id)
                            .network_address(&addr),
                        "{} Ignoring stale lost peer event for peer: {}, addr: {}",
                        self.network_context,
                        peer_id.short_str(),
                        addr
                    );
                }
            }
        }
    }
}

fn log_dial_result(
    network_context: Arc<NetworkContext>,
    peer_id: PeerId,
    addr: NetworkAddress,
    dial_result: DialResult,
) {
    match dial_result {
        DialResult::Success => {
            info!(
                NetworkSchema::new(&network_context)
                    .remote_peer(&peer_id)
                    .network_address(&addr),
                "{} Successfully connected to peer: {} at address: {}",
                network_context,
                peer_id.short_str(),
                addr
            );
        }
        DialResult::Cancelled => {
            info!(
                NetworkSchema::new(&network_context).remote_peer(&peer_id),
                "{} Cancelled pending dial to peer: {}",
                network_context,
                peer_id.short_str()
            );
        }
        DialResult::Failed(err) => match err {
            PeerManagerError::AlreadyConnected(a) => {
                info!(
                    NetworkSchema::new(&network_context)
                        .remote_peer(&peer_id)
                        .network_address(&a),
                    "{} Already connected to peer: {} at address: {}",
                    network_context,
                    peer_id.short_str(),
                    a
                );
            }
            e => {
                info!(
                    NetworkSchema::new(&network_context)
                        .remote_peer(&peer_id)
                        .network_address(&addr),
                    error = %e,
                    "{} Failed to connect to peer: {} at address: {}; error: {}",
                    network_context,
                    peer_id.short_str(),
                    addr,
                    e
                );
            }
        },
    }
}

/////////////////////
// DiscoverySource //
/////////////////////

impl DiscoverySource {
    fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

///////////////////
// PeerAddresses //
///////////////////

impl PeerAddresses {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

impl fmt::Display for PeerAddresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the normal HashMap-style debug format, but shorten the peer_id's
        // so the output isn't as noisy.
        f.debug_map()
            .entries(
                self.0
                    .iter()
                    .map(|(peer_id, addrs)| (peer_id.short_str(), addrs)),
            )
            .finish()
    }
}

impl fmt::Debug for PeerAddresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

///////////////
// Addresses //
///////////////

impl Addresses {
    fn len(&self) -> usize {
        self.0.iter().map(Vec::len).sum()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Update the addresses for the `DiscoverySource` bucket. Return `true` if
    /// the addresses have actually changed.
    fn update(&mut self, src: DiscoverySource, addrs: Vec<NetworkAddress>) -> bool {
        let src_idx = src.as_usize();
        if self.0[src_idx] != addrs {
            self.0[src_idx] = addrs;
            true
        } else {
            false
        }
    }

    fn get(&self, idx: usize) -> Option<&NetworkAddress> {
        self.0.iter().flatten().nth(idx)
    }
}

impl fmt::Display for Addresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write without the typical "Addresses(..)" around the output to reduce
        // debug noise.
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Debug for Addresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

////////////////////
// PeerPublicKeys //
////////////////////

impl PeerPublicKeys {
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Remove all empty `PublicKeys`. Returns `true` if any `PublicKeys`
    /// were actually removed.
    fn remove_empty(&mut self) -> bool {
        let pre_retain_len = self.0.len();
        self.0.retain(|_, pubkeys| !pubkeys.is_empty());
        assert!(
            pre_retain_len >= self.0.len(),
            "retain should only remove items, never add: pre len: {}, post len: {}",
            pre_retain_len,
            self.0.len()
        );
        let num_removed = pre_retain_len - self.0.len();
        num_removed > 0
    }

    fn union_all(&self) -> HashMap<PeerId, HashSet<x25519::PublicKey>> {
        self.0
            .iter()
            .map(|(peer_id, pubkeys)| (*peer_id, pubkeys.union()))
            .collect()
    }
}

impl fmt::Display for PeerPublicKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the normal HashMap-style debug format, but shorten the peer_id's
        // so the output isn't as noisy.
        f.debug_map()
            .entries(
                self.0
                    .iter()
                    .map(|(peer_id, pubkeys)| (peer_id.short_str(), pubkeys)),
            )
            .finish()
    }
}

impl fmt::Debug for PeerPublicKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

////////////////
// PublicKeys //
////////////////

impl PublicKeys {
    fn len(&self) -> usize {
        self.0.iter().map(HashSet::len).sum()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn update(&mut self, src: DiscoverySource, pubkeys: HashSet<x25519::PublicKey>) -> bool {
        let src_idx = src.as_usize();
        if self.0[src_idx] != pubkeys {
            self.0[src_idx] = pubkeys;
            true
        } else {
            false
        }
    }

    fn union(&self) -> HashSet<x25519::PublicKey> {
        self.0.iter().flatten().copied().collect()
    }
}

impl fmt::Display for PublicKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write without the typical "PublicKeys(..)" around the output to reduce
        // debug noise.
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Debug for PublicKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

///////////////
// DialState //
///////////////

impl<TBackoff> DialState<TBackoff>
where
    TBackoff: Iterator<Item = Duration> + Clone,
{
    fn new(backoff: TBackoff) -> Self {
        Self {
            backoff,
            addr_idx: 0,
        }
    }

    fn next_addr<'a>(&mut self, addrs: &'a Addresses) -> &'a NetworkAddress {
        assert!(!addrs.is_empty());

        let addr_idx = self.addr_idx;
        self.addr_idx = self.addr_idx.wrapping_add(1);

        addrs.get(addr_idx % addrs.len()).unwrap()
    }

    fn next_backoff_delay(&mut self, max_delay: Duration) -> Duration {
        let jitter = jitter(Duration::from_millis(100));

        min(max_delay, self.backoff.next().unwrap_or(max_delay)) + jitter
    }
}
