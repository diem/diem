// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The ConnectivityManager actor is responsible for ensuring that we are connected to a node
//! if and only if it is an eligible node.
//! A list of eligible nodes is received at initialization, and updates are received on changes
//! to system membership.
//!
//! In our current system design, the Consensus actor informs the ConnectivityManager of
//! eligible nodes, and the Discovery actor infroms it about updates to addresses of eligible
//! nodes.
use crate::{
    common::NetworkPublicKeys,
    peer_manager::{PeerManagerError, PeerManagerNotification, PeerManagerRequestSender},
};
use channel;
use futures::stream::{FusedStream, Stream, StreamExt};
use logger::prelude::*;
use parity_multiaddr::Multiaddr;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};
use types::PeerId;

#[cfg(test)]
mod test;

/// The ConnectivityManager actor.
pub struct ConnectivityManager<TTicker, TSubstream> {
    /// Nodes which are eligible to join the network.
    eligible: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    /// Nodes we are connected to, and the address we are connected at.
    connected: HashMap<PeerId, Multiaddr>,
    /// Addresses of peers received from Discovery module.
    peer_addresses: HashMap<PeerId, Vec<Multiaddr>>,
    /// Ticker to trigger connectivity checks to provide the guarantees stated above.
    ticker: TTicker,
    /// Channel to send requests to PeerManager.
    peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    /// Channel to receive notifications from PeerManager.
    peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
    /// Channel over which we receive requests from other actors.
    requests_rx: channel::Receiver<ConnectivityRequest>,
    /// A local counter incremented on receiving an incoming message. Printing this in debugging
    /// allows for easy debugging.
    event_id: u32,
}

/// Requests received by the [`ConnectivityManager`] manager actor from upstream modules.
#[derive(Debug)]
pub enum ConnectivityRequest {
    /// Request to update known addresses of peer with id `PeerId` to given list.
    UpdateAddresses(PeerId, Vec<Multiaddr>),
    /// Update set of nodes eligible to join the network.
    UpdateEligibleNodes(HashMap<PeerId, NetworkPublicKeys>),
}

impl<TTicker, TSubstream> ConnectivityManager<TTicker, TSubstream>
where
    TTicker: Stream + FusedStream + Unpin + 'static,
    TSubstream: Debug,
{
    /// Creates a new instance of the [`ConnectivityManager`] actor.
    pub fn new(
        eligible: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
        ticker: TTicker,
        peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        requests_rx: channel::Receiver<ConnectivityRequest>,
    ) -> Self {
        Self {
            eligible,
            connected: HashMap::new(),
            peer_addresses: HashMap::new(),
            ticker,
            peer_mgr_reqs_tx,
            peer_mgr_notifs_rx,
            requests_rx,
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
        loop {
            self.event_id += 1;
            ::futures::select! {
                _ = self.ticker.select_next_some() => {
                    trace!("Event Id: {}, type: Tick", self.event_id);
                    self.check_connectivity().await;
                }
                req = self.requests_rx.select_next_some() => {
                    trace!("Event Id: {}, type: ConnectivityRequest, req: {:?}", self.event_id, req);
                    self.handle_request(req);
                }
                notif = self.peer_mgr_notifs_rx.select_next_some() => {
                    trace!("Event Id: {}, type: PeerManagerNotification, notif: {:?}", self.event_id, notif);
                    self.handle_peer_mgr_notification(notif);
                }
                complete => {
                    crit!("Connectivity manager actor terminated");
                    break;
                }
            }
        }
    }

    // Note: We do not check that the connections to older incarnations of a node are broken, and
    // instead rely on the node moving to a new epoch to break connections made from older
    // incarnations.
    async fn check_connectivity(&mut self) {
        // Ensure we are only connected to eligible peers.
        let stale_connections: Vec<_> = self
            .connected
            .keys()
            .filter(|peer_id| !self.eligible.read().unwrap().contains_key(peer_id))
            .cloned()
            .collect();
        for p in stale_connections.into_iter() {
            info!("Should no longer be connected to peer: {}", p.short_str());
            match self.peer_mgr_reqs_tx.disconnect_peer(p).await {
                // If we disconnect successfully, or if the connection has already broken, we
                // eagerly remove the peer from list of connected peers, without
                // waiting for the LostPeer notification from PeerManager.
                Ok(_) | Err(PeerManagerError::NotConnected(_)) => {
                    self.connected.remove(&p);
                }
                e => {
                    info!(
                        "Failed to disconnect from peer: {}. Error: {:?}",
                        p.short_str(),
                        e
                    );
                }
            }
        }
        // If we have the address of an eligible peer, but are not connected to it, we dial it.
        let pending_connections: Vec<_> = self
            .peer_addresses
            .iter()
            .filter(|(peer_id, _)| {
                self.eligible.read().unwrap().contains_key(peer_id)
                    && self.connected.get(peer_id).is_none()
            })
            .collect();
        for (p, addrs) in pending_connections.into_iter() {
            info!(
                "Should be connected to peer: {} at addr(s): {:?}",
                p.short_str(),
                addrs,
            );
            match self.peer_mgr_reqs_tx.dial_peer(*p, addrs[0].clone()).await {
                // If the dial succeeded, or if are somehow already connected to the peer by the
                // time we make the dial request, we eagerly add the peer to list of connected
                // peers, without waiting for the NewPeer notification from PeerManager.
                Ok(_) => {
                    self.connected.insert(*p, addrs[0].clone());
                }
                Err(PeerManagerError::AlreadyConnected(a)) => {
                    // We ignore whether `a` is actually the address we dialed.
                    self.connected.insert(*p, a);
                }
                e => {
                    info!(
                        "Failed to connect to peer: {} at address: {}. Error: {:?}",
                        p.short_str(),
                        addrs[0].clone(),
                        e
                    );
                }
            }
        }
    }

    fn handle_request(&mut self, req: ConnectivityRequest) {
        match req {
            ConnectivityRequest::UpdateAddresses(peer_id, addrs) => {
                self.peer_addresses.insert(peer_id, addrs);
            }
            ConnectivityRequest::UpdateEligibleNodes(nodes) => {
                *self.eligible.write().unwrap() = nodes;
            }
        }
    }

    fn handle_peer_mgr_notification(&mut self, notif: PeerManagerNotification<TSubstream>) {
        match notif {
            PeerManagerNotification::NewPeer(peer_id, addr) => {
                self.connected.insert(peer_id, addr);
            }
            PeerManagerNotification::LostPeer(peer_id, addr) => {
                match self.connected.get(&peer_id) {
                    Some(curr_addr) if *curr_addr == addr => {
                        // Remove node from connected peers list.
                        self.connected.remove(&peer_id);
                    }
                    _ => {
                        debug!(
                            "Ignoring stale lost peer event for peer: {}, addr: {}",
                            peer_id.short_str(),
                            addr
                        );
                    }
                }
            }
            _ => {
                panic!("Received unexpected notification from peer manager");
            }
        }
    }
}
