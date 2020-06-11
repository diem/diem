// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The ConnectivityManager actor is responsible for ensuring that we are
//! connected to a node if and only if it is an eligible node.
//!
//! A list of eligible nodes is received at initialization, and updates are
//! received on changes to system membership. In our current system design, the
//! Consensus actor informs the ConnectivityManager of eligible nodes.
//!
//! Different discovery sources notify the ConnectivityManager of updates to
//! peers' addresses. Currently, there are 3 discovery sources (ordered by
//! decreasing dial priority, i.e., first is highest priority):
//!
//! 1. Onchain discovery protocol
//! 2. Gossip discovery protocol
//! 3. Seed peers from config
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

use crate::peer_manager::{self, conn_notifs_channel, ConnectionRequestSender, PeerManagerError};
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
};
use libra_config::network_id::NetworkContext;
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use num_variants::NumVariants;
use std::{
    cmp::min,
    collections::HashMap,
    fmt, mem,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::time;

#[cfg(test)]
mod test;

/// The ConnectivityManager actor.
pub struct ConnectivityManager<TTicker, TBackoff> {
    network_context: NetworkContext,
    /// Nodes which are eligible to join the network.
    eligible: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
    /// PeerId and address of remote peers to which this peer is connected.
    connected: HashMap<PeerId, NetworkAddress>,
    /// Addresses of peers received from discovery sources.
    peer_addresses: PeerAddresses,
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
}

/// Different sources for peer addresses, ordered by priority (Onchain=highest,
/// Config=lowest).
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, NumVariants)]
pub enum DiscoverySource {
    OnChain,
    Gossip,
    Config,
}

/// Requests received by the [`ConnectivityManager`] manager actor from upstream modules.
#[derive(Debug)]
pub enum ConnectivityRequest {
    /// Request to update known addresses of peer with id `PeerId` to given list.
    UpdateAddresses(DiscoverySource, HashMap<PeerId, Vec<NetworkAddress>>),
    /// Update set of nodes eligible to join the network.
    UpdateEligibleNodes(HashMap<PeerId, x25519::PublicKey>),
    /// Gets current size of dial queue. This is useful in tests.
    GetDialQueueSize(oneshot::Sender<usize>),
}

/// The set of `NetworkAddress`'s for all peers.
struct PeerAddresses(HashMap<PeerId, Addresses>);

/// A set of `NetworkAddress`'s for a single peer, bucketed by DiscoverySource in
/// priority order.
#[derive(Clone, Default)]
struct Addresses([Vec<NetworkAddress>; DiscoverySource::NUM_VARIANTS]);

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
    /// `peer_addresses` entry.
    addr_idx: usize,
}

impl<TTicker, TBackoff> ConnectivityManager<TTicker, TBackoff>
where
    TTicker: Stream + FusedStream + Unpin + 'static,
    TBackoff: Iterator<Item = Duration> + Clone,
{
    /// Creates a new instance of the [`ConnectivityManager`] actor.
    pub fn new(
        network_context: NetworkContext,
        eligible: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
        seed_peers: HashMap<PeerId, Vec<NetworkAddress>>,
        ticker: TTicker,
        connection_reqs_tx: ConnectionRequestSender,
        connection_notifs_rx: conn_notifs_channel::Receiver,
        requests_rx: channel::Receiver<ConnectivityRequest>,
        backoff_strategy: TBackoff,
        max_delay_ms: u64,
    ) -> Self {
        {
            // Reconcile the keysets eligible is only used to allow us to dial the remote peer
            let eligible_peers = &mut eligible.write().unwrap();
            for (id, addr) in &seed_peers {
                let key = addr[0]
                    .find_noise_proto()
                    .expect("Unable to find x25519 key in address");
                if !eligible_peers.contains_key(&id) {
                    eligible_peers.insert(*id, key);
                }
            }
        }

        // Ensure seed peers doesn't contain our own address (we want to avoid
        // pointless self-dials).
        let peer_addresses = PeerAddresses(
            seed_peers
                .into_iter()
                .filter(|(peer_id, _)| peer_id != &network_context.peer_id())
                .map(|(peer_id, seed_addrs)| {
                    (
                        peer_id,
                        Addresses::from_addrs(DiscoverySource::Config, seed_addrs),
                    )
                })
                .collect(),
        );

        info!(
            "{} ConnectivityManager init: num_seed_peers: {}, peer addresses: {}",
            network_context,
            peer_addresses.0.len(),
            peer_addresses,
        );

        Self {
            network_context,
            eligible,
            connected: HashMap::new(),
            peer_addresses,
            ticker,
            connection_reqs_tx,
            connection_notifs_rx,
            requests_rx,
            dial_queue: HashMap::new(),
            dial_states: HashMap::new(),
            backoff_strategy,
            max_delay_ms,
            event_id: 0,
        }
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

        trace!("{} Starting connection manager", self.network_context);
        loop {
            self.event_id = self.event_id.wrapping_add(1);
            ::futures::select! {
                _ = self.ticker.select_next_some() => {
                    trace!("{} Event Id: {}, type: Tick", self.network_context, self.event_id);
                    self.check_connectivity(&mut pending_dials).await;
                },
                req = self.requests_rx.select_next_some() => {
                    trace!("{} Event Id: {}, type: ConnectivityRequest, req: {:?}", self.network_context, self.event_id, req);
                    self.handle_request(req);
                },
                notif = self.connection_notifs_rx.select_next_some() => {
                    trace!("{} Event Id: {}, type: peer_manager::ConnectionNotification, notif: {:?}", self.network_context, self.event_id, notif);
                    self.handle_control_notification(notif);
                },
                peer_id = pending_dials.select_next_some() => {
                    trace!("{} Event Id: {}, type: Dial complete, peer: {}", self.network_context, self.event_id, peer_id.short_str());
                    self.dial_queue.remove(&peer_id);
                },
                complete => {
                    crit!("{} Connectivity manager actor terminated", self.network_context);
                    break;
                }
            }
        }
    }

    /// Disconnect from all peers that are no longer eligible.
    ///
    /// For instance, a validator might leave the validator set after a
    /// reconfiguration. If we are currently connected to this validator, calling
    /// this function will close our connection to it.
    async fn close_stale_connections(&mut self) {
        let eligible = self.eligible.read().unwrap().clone();
        let stale_connections: Vec<_> = self
            .connected
            .keys()
            .filter(|peer_id| !eligible.contains_key(peer_id))
            .cloned()
            .collect();
        for p in stale_connections.into_iter() {
            info!(
                "{} Should no longer be connected to peer: {}",
                self.network_context,
                p.short_str()
            );
            // Close existing connection.
            if let Err(e) = self.connection_reqs_tx.disconnect_peer(p).await {
                info!(
                    "{} Failed to disconnect from peer: {}. Error: {:?}",
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
        let eligible = self.eligible.read().unwrap().clone();
        let stale_dials: Vec<_> = self
            .dial_queue
            .keys()
            .filter(|peer_id| !eligible.contains_key(peer_id))
            .cloned()
            .collect();
        for p in stale_dials.into_iter() {
            self.dial_queue.remove(&p);
        }
    }

    async fn dial_eligible_peers<'a>(
        &'a mut self,
        pending_dials: &'a mut FuturesUnordered<BoxFuture<'static, PeerId>>,
    ) {
        let eligible = self.eligible.read().unwrap().clone();
        let to_connect: Vec<_> = self
            .peer_addresses
            .0
            .iter()
            .filter(|(peer_id, addrs)| {
                eligible.contains_key(peer_id)  // The node is eligible to be dialed.
                    && self.connected.get(peer_id).is_none() // The node is not already connected.
                    && self.dial_queue.get(peer_id).is_none() // There is no pending dial to this node.
                    && !addrs.is_empty() // There is an address to dial.
            })
            .collect();

        // We tune max delay depending on the number of peers to which we're not connected. This
        // ensures that if we're disconnected from a large fraction of peers, we keep the retry
        // window smaller.
        let max_delay = Duration::from_millis(
            (self.max_delay_ms as f64
                * (1.0
                    - ((self.dial_queue.len() + to_connect.len()) as f64
                        / eligible
                            .iter()
                            .filter(|(peer_id, _)| self.peer_addresses.0.contains_key(&peer_id))
                            .count() as f64))) as u64,
        );

        // The initial dial state; it has zero dial delay and uses the first
        // address.
        let init_dial_state = DialState::new(self.backoff_strategy.clone());

        for (p, addrs) in to_connect.into_iter() {
            let mut connction_reqs_tx = self.connection_reqs_tx.clone();
            let peer_id = *p;
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
            let dial_delay = dial_state.next_backoff_delay(max_delay);
            let f_delay = time::delay_for(dial_delay);

            let (cancel_tx, cancel_rx) = oneshot::channel();

            info!(
                "{} Create dial future: peer: {}, at address: {}, after delay: {:?}",
                self.network_context,
                peer_id.short_str(),
                addr,
                dial_delay,
            );

            let network_context = self.network_context.clone();
            // Create future which completes by either dialing after calculated
            // delay or on cancellation.
            let f = async move {
                info!(
                    "{} Dial future: dialing peer: {}, at address: {}, after delay: {:?}",
                    network_context,
                    peer_id.short_str(),
                    addr,
                    f_delay
                        .deadline()
                        .duration_since(tokio::time::Instant::from_std(now))
                );
                // We dial after a delay. The dial can be canceled by sending to or dropping
                // `cancel_rx`.
                let dial_result = ::futures::select! {
                    _ = f_delay.fuse() => {
                        info!("{} Dialing peer: {}, at addr: {}", network_context, peer_id.short_str(), addr);
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
        // Cancel dials to peers that are no longer eligible.
        self.cancel_stale_dials().await;
        // Disconnect from connected peers that are no longer eligible.
        self.close_stale_connections().await;
        // Dial peers which are eligible but are neither connected nor queued for dialing in the
        // future.
        self.dial_eligible_peers(pending_dials).await;
    }

    fn handle_request(&mut self, req: ConnectivityRequest) {
        match req {
            ConnectivityRequest::UpdateAddresses(src, address_map) => {
                // Keep track of if any peer's addresses have actually changed, so
                // we can log without too much spam.
                let mut have_any_changed = false;
                let self_peer_id = self.network_context.peer_id();

                for (peer_id, addrs) in address_map {
                    // Do not include self_peer_id in the address list for dialing
                    // to avoid pointless self-dials.
                    if peer_id == self_peer_id {
                        continue;
                    }

                    // Update peer's addresses
                    let curr_addrs = self.peer_addresses.0.entry(peer_id).or_default();
                    if curr_addrs.update(src, addrs) {
                        // At least one peer's addresses have actually changed.
                        have_any_changed = true;

                        // If we're currently trying to dial this peer, we reset
                        // their dial state. As a result, we will begin our next
                        // dial attempt from the first address (which might have
                        // changed) and from a fresh backoff (since the current
                        // backoff delay might be maxed out if we can't reach any
                        // of their previous addresses).
                        if let Some(dial_state) = self.dial_states.get_mut(&peer_id) {
                            mem::replace(dial_state, DialState::new(self.backoff_strategy.clone()));
                        }

                        // Log the change to this peer's addresses.
                        let peer_id = peer_id.short_str();
                        let addrs = curr_addrs;
                        info!(
                            "{} addresses updated for peer: {}, update src: {:?}, addrs: {}",
                            self.network_context, peer_id, src, addrs,
                        );
                    }
                }

                // Only log the total state if anything has actually changed.
                if have_any_changed {
                    let peer_addresses = &self.peer_addresses;
                    info!(
                        "{} current addresses: update src: {:?}, all peer addresses: {}",
                        self.network_context, src, peer_addresses,
                    );
                }
            }
            ConnectivityRequest::UpdateEligibleNodes(nodes) => {
                trace!(
                    "{} Received updated list of eligible nodes",
                    self.network_context
                );
                *self.eligible.write().unwrap() = nodes;
            }
            ConnectivityRequest::GetDialQueueSize(sender) => {
                sender.send(self.dial_queue.len()).unwrap();
            }
        }
    }

    fn handle_control_notification(&mut self, notif: peer_manager::ConnectionNotification) {
        match notif {
            peer_manager::ConnectionNotification::NewPeer(peer_id, addr) => {
                self.connected.insert(peer_id, addr);
                // Cancel possible queued dial to this peer.
                self.dial_states.remove(&peer_id);
                self.dial_queue.remove(&peer_id);
            }
            peer_manager::ConnectionNotification::LostPeer(peer_id, addr, _reason) => {
                match self.connected.get(&peer_id) {
                    Some(curr_addr) if *curr_addr == addr => {
                        // Remove node from connected peers list.
                        self.connected.remove(&peer_id);
                    }
                    _ => {
                        debug!(
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
}

fn log_dial_result(
    network_context: NetworkContext,
    peer_id: PeerId,
    addr: NetworkAddress,
    dial_result: DialResult,
) {
    match dial_result {
        DialResult::Success => {
            info!(
                "{} Successfully connected to peer: {} at address: {}",
                network_context,
                peer_id.short_str(),
                addr
            );
        }
        DialResult::Cancelled => {
            info!(
                "{} Cancelled pending dial to peer: {}",
                network_context,
                peer_id.short_str()
            );
        }
        DialResult::Failed(err) => match err {
            PeerManagerError::AlreadyConnected(a) => {
                info!(
                    "{} Already connected to peer: {} at address: {}",
                    network_context,
                    peer_id.short_str(),
                    a
                );
            }
            e => {
                info!(
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

impl Addresses {
    fn new() -> Self {
        Default::default()
    }

    fn from_addrs(src: DiscoverySource, src_addrs: Vec<NetworkAddress>) -> Self {
        let mut addrs = Self::new();
        addrs.update(src, src_addrs);
        addrs
    }

    fn len(&self) -> usize {
        self.0.iter().map(Vec::len).sum()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Update the addresses for the `DiscoverySource` bucket. Return `true` if
    /// the addresses have actually changed.
    fn update(&mut self, src: DiscoverySource, addrs: Vec<NetworkAddress>) -> bool {
        let src_idx = src as u8 as usize;
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
        write!(f, "{}", self)
    }
}

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
        min(max_delay, self.backoff.next().unwrap_or(max_delay))
    }
}
