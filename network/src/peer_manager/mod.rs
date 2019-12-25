// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The PeerManager module is responsible for establishing connections between Peers and for
//! opening/receiving new substreams on those connections.
//!
//! ## Implementation
//!
//! The PeerManager is implemented as a number of actors:
//!  * A main event loop actor which is responsible for handling requests and sending
//!  notification about new/lost Peers to the rest of the network stack.
//!  * An actor responsible for dialing and listening for new connections.
use crate::{
    common::NegotiatedSubstream,
    counters,
    peer::{Peer, PeerHandle, PeerNotification},
    protocols::identity::Identity,
    transport, ProtocolId,
};
use channel;
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    sink::SinkExt,
    stream::{Fuse, FuturesUnordered, StreamExt},
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::{
    multiplexing::StreamMultiplexer,
    transport::{ConnectionOrigin, Transport},
};
use parity_multiaddr::Multiaddr;
use std::{collections::HashMap, marker::PhantomData};
use tokio::runtime::Handle;

mod error;
#[cfg(test)]
mod tests;

pub use self::error::PeerManagerError;

/// Notifications about new/lost peers.
#[derive(Debug)]
pub enum PeerManagerNotification<TSubstream> {
    NewPeer(PeerId, Multiaddr),
    LostPeer(PeerId, Multiaddr),
    NewInboundSubstream(PeerId, NegotiatedSubstream<TSubstream>),
}

/// Request received by PeerManager from upstream actors.
#[derive(Debug)]
pub enum PeerManagerRequest<TSubstream> {
    DialPeer(
        PeerId,
        Multiaddr,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    DisconnectPeer(PeerId, oneshot::Sender<Result<(), PeerManagerError>>),
    OpenSubstream(
        PeerId,
        ProtocolId,
        oneshot::Sender<Result<TSubstream, PeerManagerError>>,
    ),
}

/// Convenience wrapper around a `channel::Sender<PeerManagerRequest>` which makes it easy to issue
/// requests and await the responses from PeerManager
pub struct PeerManagerRequestSender<TSubstream> {
    inner: channel::Sender<PeerManagerRequest<TSubstream>>,
}

impl<TSubstream> Clone for PeerManagerRequestSender<TSubstream> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<TSubstream> PeerManagerRequestSender<TSubstream> {
    /// Construct a new PeerManagerRequestSender with a raw channel::Sender
    pub fn new(sender: channel::Sender<PeerManagerRequest<TSubstream>>) -> Self {
        Self { inner: sender }
    }

    /// Request that a given Peer be dialed at the provided `Multiaddr` and synchronously wait for
    /// the request to be performed.
    pub async fn dial_peer(
        &mut self,
        peer_id: PeerId,
        addr: Multiaddr,
    ) -> Result<(), PeerManagerError> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let request = PeerManagerRequest::DialPeer(peer_id, addr, oneshot_tx);
        self.inner.send(request).await.unwrap();
        oneshot_rx.await?
    }

    /// Request that a given Peer be disconnected and synchronously wait for the request to be
    /// performed.
    pub async fn disconnect_peer(&mut self, peer_id: PeerId) -> Result<(), PeerManagerError> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let request = PeerManagerRequest::DisconnectPeer(peer_id, oneshot_tx);
        self.inner.send(request).await.unwrap();
        oneshot_rx.await?
    }

    /// Request that a new substream be opened with the given Peer and that the provided `protocol`
    /// be negotiated on that substream and synchronously wait for the request to be performed.
    pub async fn open_substream(
        &mut self,
        peer_id: PeerId,
        protocol: ProtocolId,
    ) -> Result<TSubstream, PeerManagerError> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let request = PeerManagerRequest::OpenSubstream(peer_id, protocol, oneshot_tx);
        self.inner.send(request).await.unwrap();
        oneshot_rx
            .await
            // The open_substream request can get dropped/canceled if the peer
            // connection is in the process of shutting down.
            .map_err(|_| PeerManagerError::NotConnected(peer_id))?
    }
}

/// Responsible for handling and maintaining connections to other Peers
pub struct PeerManager<TTransport, TMuxer>
where
    TTransport: Transport,
    TMuxer: StreamMultiplexer,
{
    /// A handle to a tokio executor.
    executor: Handle,
    /// PeerId of "self".
    own_peer_id: PeerId,
    /// Address to listen on for incoming connections.
    listen_addr: Multiaddr,
    /// Connection Listener, listening on `listen_addr`
    connection_handler: Option<ConnectionHandler<TTransport, TMuxer>>,
    /// Map from PeerId to corresponding Peer object.
    active_peers: HashMap<PeerId, PeerHandle<TMuxer::Substream>>,
    /// Channel to receive requests from other actors.
    requests_rx: channel::Receiver<PeerManagerRequest<TMuxer::Substream>>,
    /// Map from protocol to handler for substreams which want to "speak" that protocol.
    protocol_handlers:
        HashMap<ProtocolId, channel::Sender<PeerManagerNotification<TMuxer::Substream>>>,
    /// Channel to send NewPeer/LostPeer notifications to other actors.
    /// Note: NewInboundSubstream notifications are not sent via these channels.
    peer_event_handlers: Vec<channel::Sender<PeerManagerNotification<TMuxer::Substream>>>,
    /// Channel used to send Dial requests to the ConnectionHandler actor
    dial_request_tx: channel::Sender<ConnectionHandlerRequest>,
    /// Receiver for connection events.
    connection_handler_notifs_rx: channel::Receiver<ConnectionHandlerNotification<TMuxer>>,
    /// Sender for peer events.
    peer_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    /// Receiver for peer events.
    peer_notifs_rx: channel::Receiver<PeerNotification<TMuxer::Substream>>,
    /// A map of outstanding disconnect requests
    outstanding_disconnect_requests: HashMap<PeerId, oneshot::Sender<Result<(), PeerManagerError>>>,
    /// Pin the transport type corresponding to this PeerManager instance
    phantom_transport: PhantomData<TTransport>,
}

impl<TTransport, TMuxer> PeerManager<TTransport, TMuxer>
where
    TTransport: Transport<Output = (Identity, TMuxer)> + Send + 'static,
    TMuxer: StreamMultiplexer + 'static,
{
    /// Construct a new PeerManager actor
    pub fn new(
        transport: TTransport,
        executor: Handle,
        own_peer_id: PeerId,
        listen_addr: Multiaddr,
        requests_rx: channel::Receiver<PeerManagerRequest<TMuxer::Substream>>,
        protocol_handlers: HashMap<
            ProtocolId,
            channel::Sender<PeerManagerNotification<TMuxer::Substream>>,
        >,
        peer_event_handlers: Vec<channel::Sender<PeerManagerNotification<TMuxer::Substream>>>,
    ) -> Self {
        let (connection_handler_notifs_tx, connection_handler_notifs_rx) =
            channel::new(1024, &counters::PENDING_CONNECTION_HANDLER_NOTIFICATIONS);
        let (peer_notifs_tx, peer_notifs_rx) =
            channel::new(1024, &counters::PENDING_PEER_NOTIFICATIONS);
        let (dial_request_tx, dial_request_rx) =
            channel::new(1024, &counters::PENDING_PEER_MANAGER_DIAL_REQUESTS);
        //TODO now that you can only listen on a socket inside of a tokio runtime we'll need to
        // rethink how we init the PeerManager so we don't have to do this funny thing.
        let (connection_handler, listen_addr) =
            futures::executor::block_on(executor.spawn(async move {
                ConnectionHandler::new(
                    transport,
                    listen_addr,
                    dial_request_rx,
                    connection_handler_notifs_tx,
                )
            }))
            .unwrap();

        Self {
            executor,
            own_peer_id,
            listen_addr,
            connection_handler: Some(connection_handler),
            active_peers: HashMap::new(),
            requests_rx,
            protocol_handlers,
            peer_event_handlers,
            dial_request_tx,
            connection_handler_notifs_rx,
            peer_notifs_rx,
            peer_notifs_tx,
            outstanding_disconnect_requests: HashMap::new(),
            phantom_transport: PhantomData,
        }
    }

    /// Get the [`Multiaddr`] we're listening for incoming connections on
    pub fn listen_addr(&self) -> &Multiaddr {
        &self.listen_addr
    }

    /// Start listening on the set address and return a future which runs PeerManager
    pub async fn start(mut self) {
        // Start listening for connections.
        self.start_connection_listener();
        loop {
            ::futures::select! {
                connection_event = self.connection_handler_notifs_rx.select_next_some() => {
                  self.handle_connection_event(connection_event).await;
                }
                peer_event = self.peer_notifs_rx.select_next_some() => {
                  self.handle_peer_event(peer_event).await;
                }
                request = self.requests_rx.select_next_some() => {
                  self.handle_request(request).await;
                }
                complete => {
                    crit!("Peer manager actor terminated");
                    break;
                }
            }
        }
    }

    async fn handle_connection_event(&mut self, event: ConnectionHandlerNotification<TMuxer>) {
        trace!("ConnectionHandlerNotification::{:?}", event);
        match event {
            ConnectionHandlerNotification::NewConnection(identity, addr, origin, conn) => {
                self.add_peer(identity, addr, origin, conn).await;
            }
        }
    }

    async fn handle_peer_event(&mut self, event: PeerNotification<TMuxer::Substream>) {
        trace!("PeerEvent::{:?}", event);
        match event {
            PeerNotification::NewSubstream(peer_id, substream) => {
                let ch = self
                    .protocol_handlers
                    .get_mut(&substream.protocol)
                    .expect("Received substream for unknown protocol");
                let event = PeerManagerNotification::NewInboundSubstream(peer_id, substream);
                ch.send(event).await.unwrap();
            }
            PeerNotification::PeerDisconnected(peer_id, role, origin, _reason) => {
                let peer = self
                    .active_peers
                    .remove(&peer_id)
                    .expect("Should have a handle to Peer");

                // If we receive a PeerDisconnected event and the connection origin isn't the same
                // as the one we have stored in PeerManager this particular event is from a Peer
                // actor which is being shutdown due to simultaneous dial tie-breaking and we don't
                // need to send a LostPeer notification to all subscribers.
                if peer.origin() != origin {
                    self.active_peers.insert(peer_id, peer);
                    return;
                }
                info!("Disconnected from peer: {}", peer_id.short_str());
                if let Some(oneshot_tx) = self.outstanding_disconnect_requests.remove(&peer_id) {
                    if oneshot_tx.send(Ok(())).is_err() {
                        error!("oneshot channel receiver dropped");
                    }
                }
                // update libra_network_peer counter
                counters::LIBRA_NETWORK_PEERS
                    .with_label_values(&[role.as_str(), "connected"])
                    .dec();
                // Send LostPeer notifications to subscribers
                for ch in &mut self.peer_event_handlers {
                    ch.send(PeerManagerNotification::LostPeer(
                        peer_id,
                        peer.address().clone(),
                    ))
                    .await
                    .unwrap();
                }
            }
        }
    }

    async fn handle_request(&mut self, request: PeerManagerRequest<TMuxer::Substream>) {
        trace!("PeerManagerRequest::{:?}", request);
        match request {
            PeerManagerRequest::DialPeer(requested_peer_id, addr, response_tx) => {
                // Only dial peers which we aren't already connected with
                if let Some(peer) = self.active_peers.get(&requested_peer_id) {
                    let error = if peer.is_shutting_down() {
                        PeerManagerError::ShuttingDownPeer
                    } else {
                        PeerManagerError::AlreadyConnected(peer.address().to_owned())
                    };
                    debug!(
                        "Already connected with Peer {} at address {}, not dialing address {}",
                        peer.peer_id().short_str(),
                        peer.address(),
                        addr
                    );

                    if response_tx.send(Err(error)).is_err() {
                        warn!(
                            "Receiver for DialPeer {} dropped",
                            requested_peer_id.short_str()
                        );
                    }
                } else {
                    self.dial_peer(requested_peer_id, addr, response_tx).await;
                };
            }
            PeerManagerRequest::DisconnectPeer(peer_id, response_tx) => {
                self.disconnect_peer(peer_id, response_tx).await;
            }
            PeerManagerRequest::OpenSubstream(peer_id, protocol, request_tx) => {
                match self.active_peers.get_mut(&peer_id) {
                    Some(ref mut peer) if !peer.is_shutting_down() => {
                        peer.open_substream(protocol, request_tx).await;
                    }
                    _ => {
                        // If we don't have a connection open with this peer, or if the connection
                        // is currently undergoing shutdown we should return an error to the
                        // requester
                        if request_tx
                            .send(Err(PeerManagerError::NotConnected(peer_id)))
                            .is_err()
                        {
                            warn!(
                                "Request for substream to peer {} failed, but receiver dropped too",
                                peer_id.short_str()
                            );
                        }
                    }
                }
            }
        }
    }

    fn start_connection_listener(&mut self) {
        let connection_handler = self
            .connection_handler
            .take()
            .expect("Connection handler already taken");
        self.executor.spawn(connection_handler.listen());
    }

    /// In the event two peers simultaneously dial each other we need to be able to do
    /// tie-breaking to determine which connection to keep and which to drop in a deterministic
    /// way. One simple way is to compare our local PeerId with that of the remote's PeerId and
    /// keep the connection where the peer with the greater PeerId is the dialer.
    ///
    /// Returns `true` if the existing connection should be dropped and `false` if the new
    /// connection should be dropped.
    fn simultaneous_dial_tie_breaking(
        own_peer_id: PeerId,
        remote_peer_id: PeerId,
        existing_origin: ConnectionOrigin,
        new_origin: ConnectionOrigin,
    ) -> bool {
        match (existing_origin, new_origin) {
            // If the remote dials while an existing connection is open, the older connection is
            // dropped.
            (ConnectionOrigin::Inbound, ConnectionOrigin::Inbound) => true,
            (ConnectionOrigin::Inbound, ConnectionOrigin::Outbound) => remote_peer_id < own_peer_id,
            (ConnectionOrigin::Outbound, ConnectionOrigin::Inbound) => own_peer_id < remote_peer_id,
            // We should never dial the same peer twice, but if we do drop the new connection
            (ConnectionOrigin::Outbound, ConnectionOrigin::Outbound) => false,
        }
    }

    async fn add_peer(
        &mut self,
        identity: Identity,
        address: Multiaddr,
        origin: ConnectionOrigin,
        connection: TMuxer,
    ) {
        let peer_id = identity.peer_id();
        let role = identity.role();
        assert_ne!(self.own_peer_id, peer_id);

        let mut send_new_peer_notification = true;

        // Check for and handle simultaneous dialing
        if let Some(mut peer) = self.active_peers.remove(&peer_id) {
            if Self::simultaneous_dial_tie_breaking(
                self.own_peer_id,
                peer.peer_id(),
                peer.origin(),
                origin,
            ) {
                // Drop the existing connection and replace it with the new connection
                peer.disconnect().await;
                info!(
                    "Closing existing connection with Peer {} to mitigate simultaneous dial",
                    peer_id.short_str()
                );
                send_new_peer_notification = false;
            } else {
                info!(
                    "Closing incoming connection with Peer {} to mitigate simultaneous dial",
                    peer_id.short_str()
                );
                // Drop the new connection and keep the one already stored in active_peers
                if let Err(e) =
                    tokio::time::timeout(transport::TRANSPORT_TIMEOUT, connection.close()).await
                {
                    error!(
                        "Closing connection with Peer {} failed with error: {}",
                        peer_id.short_str(),
                        e
                    );
                };
                // Put the existing connection back
                self.active_peers.insert(peer.peer_id(), peer);
                return;
            }
        }

        let (peer_req_tx, peer_req_rx) = channel::new(
            1024,
            &counters::OP_COUNTERS
                .peer_gauge(&counters::PENDING_PEER_REQUESTS, &peer_id.short_str()),
        );
        let peer = Peer::new(
            identity,
            connection,
            origin,
            self.protocol_handlers.keys().cloned().collect(),
            self.peer_notifs_tx.clone(),
            peer_req_rx,
        );
        let peer_handle = PeerHandle::new(peer_id, address.clone(), origin, peer_req_tx);
        info!(
            "{:?} connection with peer {} established",
            origin,
            peer_id.short_str()
        );
        self.active_peers.insert(peer_id, peer_handle);
        self.executor.spawn(peer.start());
        // Send NewPeer notifications to subscribers
        if send_new_peer_notification {
            // update libra_network_peer counter
            counters::LIBRA_NETWORK_PEERS
                .with_label_values(&[role.as_str(), "connected"])
                .inc();

            for ch in &mut self.peer_event_handlers {
                ch.send(PeerManagerNotification::NewPeer(peer_id, address.clone()))
                    .await
                    .unwrap();
            }
        }
    }

    async fn dial_peer(
        &mut self,
        peer_id: PeerId,
        address: Multiaddr,
        response_tx: oneshot::Sender<Result<(), PeerManagerError>>,
    ) {
        let request = ConnectionHandlerRequest::DialPeer(peer_id, address, response_tx);
        self.dial_request_tx.send(request).await.unwrap();
    }

    // Send a Disconnect request to the Peer actor corresponding with `peer_id`.
    async fn disconnect_peer(
        &mut self,
        peer_id: PeerId,
        response_tx: oneshot::Sender<Result<(), PeerManagerError>>,
    ) {
        if let Some(peer) = self.active_peers.get_mut(&peer_id) {
            peer.disconnect().await;
            self.outstanding_disconnect_requests
                .insert(peer_id, response_tx);
        } else if response_tx
            .send(Err(PeerManagerError::NotConnected(peer_id)))
            .is_err()
        {
            info!(
                "Failed to disconnect from peer {}, but result receiver dropped",
                peer_id.short_str()
            );
        }
    }
}

#[derive(Debug)]
enum ConnectionHandlerRequest {
    DialPeer(
        PeerId,
        Multiaddr,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
}

#[derive(Debug)]
enum ConnectionHandlerNotification<TMuxer>
where
    TMuxer: StreamMultiplexer,
{
    NewConnection(Identity, Multiaddr, ConnectionOrigin, TMuxer),
}

/// Responsible for listening for new incoming connections
struct ConnectionHandler<TTransport, TMuxer>
where
    TTransport: Transport,
    TMuxer: StreamMultiplexer,
{
    /// [`Transport`] that is used to establish connections
    transport: TTransport,
    listener: Fuse<TTransport::Listener>,
    dial_request_rx: channel::Receiver<ConnectionHandlerRequest>,
    connection_handler_notifs_tx: channel::Sender<ConnectionHandlerNotification<TMuxer>>,
}

impl<TTransport, TMuxer> ConnectionHandler<TTransport, TMuxer>
where
    TTransport: Transport<Output = (Identity, TMuxer)>,
    TTransport::Listener: 'static,
    TTransport::Inbound: 'static,
    TTransport::Outbound: 'static,
    TMuxer: StreamMultiplexer + 'static,
{
    fn new(
        transport: TTransport,
        listen_addr: Multiaddr,
        dial_request_rx: channel::Receiver<ConnectionHandlerRequest>,
        connection_handler_notifs_tx: channel::Sender<ConnectionHandlerNotification<TMuxer>>,
    ) -> (Self, Multiaddr) {
        let (listener, listen_addr) = transport
            .listen_on(listen_addr)
            .expect("Transport listen on fails");
        debug!("listening on {:?}", listen_addr);

        (
            Self {
                transport,
                listener: listener.fuse(),
                dial_request_rx,
                connection_handler_notifs_tx,
            },
            listen_addr,
        )
    }

    async fn listen(mut self) {
        let mut pending_inbound_connections = FuturesUnordered::new();
        let mut pending_outbound_connections = FuturesUnordered::new();

        debug!("Incoming connections listener Task started");

        loop {
            futures::select! {
                dial_request = self.dial_request_rx.select_next_some() => {
                    if let Some(fut) = self.dial_peer(dial_request) {
                        pending_outbound_connections.push(fut);
                    }
                },
                incoming_connection = self.listener.select_next_some() => {
                    match incoming_connection {
                        Ok((upgrade, addr)) => {
                            debug!("Incoming connection from {}", addr);
                            pending_inbound_connections.push(upgrade.map(|out| (out, addr)));
                        }
                        Err(e) => {
                            warn!("Incoming connection error {}", e);
                        }
                    }
                },
                (upgrade, addr, peer_id, response_tx) = pending_outbound_connections.select_next_some() => {
                    self.handle_completed_outbound_upgrade(upgrade, addr, peer_id, response_tx).await;
                },
                (upgrade, addr) = pending_inbound_connections.select_next_some() => {
                    self.handle_completed_inbound_upgrade(upgrade, addr).await;
                },
                complete => break,
            }
        }

        error!("Incoming connections listener Task ended");
    }

    fn dial_peer(
        &self,
        dial_peer_request: ConnectionHandlerRequest,
    ) -> Option<
        BoxFuture<
            'static,
            (
                Result<(Identity, TMuxer), TTransport::Error>,
                Multiaddr,
                PeerId,
                oneshot::Sender<Result<(), PeerManagerError>>,
            ),
        >,
    > {
        match dial_peer_request {
            ConnectionHandlerRequest::DialPeer(peer_id, address, response_tx) => {
                match self.transport.dial(address.clone()) {
                    Ok(upgrade) => Some(
                        upgrade
                            .map(move |out| (out, address, peer_id, response_tx))
                            .boxed(),
                    ),
                    Err(error) => {
                        if response_tx
                            .send(Err(PeerManagerError::from_transport_error(error)))
                            .is_err()
                        {
                            warn!(
                                "Receiver for DialPeer {} request dropped",
                                peer_id.short_str()
                            );
                        }
                        None
                    }
                }
            }
        }
    }

    async fn handle_completed_outbound_upgrade(
        &mut self,
        upgrade: Result<(Identity, TMuxer), TTransport::Error>,
        addr: Multiaddr,
        peer_id: PeerId,
        response_tx: oneshot::Sender<Result<(), PeerManagerError>>,
    ) {
        match upgrade {
            Ok((identity, connection)) => {
                let response = if identity.peer_id() == peer_id {
                    debug!(
                        "Peer '{}' successfully dialed at '{}'",
                        peer_id.short_str(),
                        addr
                    );
                    let event = ConnectionHandlerNotification::NewConnection(
                        identity,
                        addr,
                        ConnectionOrigin::Outbound,
                        connection,
                    );
                    // Send the new connection to PeerManager
                    self.connection_handler_notifs_tx.send(event).await.unwrap();
                    Ok(())
                } else {
                    let e = ::anyhow::format_err!(
                        "Dialed PeerId ({}) differs from expected PeerId ({})",
                        identity.peer_id().short_str(),
                        peer_id.short_str()
                    );

                    warn!("{}", e);

                    Err(PeerManagerError::from_transport_error(e))
                };

                if response_tx.send(response).is_err() {
                    warn!(
                        "Receiver for DialPeer {} request dropped",
                        peer_id.short_str()
                    );
                }
            }
            Err(error) => {
                error!("Error dialing Peer {} at {}", peer_id.short_str(), addr);

                if response_tx
                    .send(Err(PeerManagerError::from_transport_error(error)))
                    .is_err()
                {
                    warn!(
                        "Receiver for DialPeer {} request dropped",
                        peer_id.short_str()
                    );
                }
            }
        }
    }

    async fn handle_completed_inbound_upgrade(
        &mut self,
        upgrade: Result<(Identity, TMuxer), TTransport::Error>,
        addr: Multiaddr,
    ) {
        match upgrade {
            Ok((identity, connection)) => {
                debug!("Connection from {} successfully upgraded", addr);
                let event = ConnectionHandlerNotification::NewConnection(
                    identity,
                    addr,
                    ConnectionOrigin::Inbound,
                    connection,
                );
                // Send the new connection to PeerManager
                self.connection_handler_notifs_tx.send(event).await.unwrap();
            }
            Err(e) => {
                warn!("Connection from {} failed to upgrade {}", addr, e);
            }
        }
    }
}
