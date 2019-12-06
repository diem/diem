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
//!  * An actor per Peer which owns the underlying connection and is responsible for listening for
//!  and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{common::NegotiatedSubstream, counters, protocols::identity::Identity, ProtocolId};
use channel;
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    lock::Mutex,
    sink::SinkExt,
    stream::{Fuse, FuturesUnordered, StreamExt},
};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::{
    multiplexing::StreamMultiplexer,
    negotiate::{negotiate_inbound, negotiate_outbound_interactive, negotiate_outbound_select},
    transport::{ConnectionOrigin, Transport},
};
use parity_multiaddr::Multiaddr;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};
use tokio::runtime::TaskExecutor;

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

#[derive(Debug, PartialEq, Eq)]
enum DisconnectReason {
    Requested,
    ConnectionLost,
}

#[derive(Debug)]
enum InternalEvent<TMuxer>
where
    TMuxer: StreamMultiplexer,
{
    NewConnection(Identity, Multiaddr, ConnectionOrigin, TMuxer),
    NewSubstream(PeerId, NegotiatedSubstream<TMuxer::Substream>),
    PeerDisconnected(PeerId, RoleType, ConnectionOrigin, DisconnectReason),
}

/// Responsible for handling and maintaining connections to other Peers
pub struct PeerManager<TTransport, TMuxer>
where
    TTransport: Transport,
    TMuxer: StreamMultiplexer,
{
    /// A handle to a tokio executor.
    executor: TaskExecutor,
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
    /// Internal event Receiver
    internal_event_rx: channel::Receiver<InternalEvent<TMuxer>>,
    /// Internal event Sender
    internal_event_tx: channel::Sender<InternalEvent<TMuxer>>,
    /// A map of outstanding disconnect requests
    outstanding_disconnect_requests: HashMap<PeerId, oneshot::Sender<Result<(), PeerManagerError>>>,
    /// Pin the transport type corresponding to this PeerManager instance
    phantom_transport: PhantomData<TTransport>,
    peer_ids: Arc<Mutex<HashSet<PeerId>>>,
}

impl<TTransport, TMuxer> PeerManager<TTransport, TMuxer>
where
    TTransport: Transport<Output = (Identity, TMuxer)> + Send + 'static,
    TMuxer: StreamMultiplexer + 'static,
{
    /// Construct a new PeerManager actor
    pub fn new(
        transport: TTransport,
        executor: TaskExecutor,
        own_peer_id: PeerId,
        listen_addr: Multiaddr,
        requests_rx: channel::Receiver<PeerManagerRequest<TMuxer::Substream>>,
        protocol_handlers: HashMap<
            ProtocolId,
            channel::Sender<PeerManagerNotification<TMuxer::Substream>>,
        >,
        peer_event_handlers: Vec<channel::Sender<PeerManagerNotification<TMuxer::Substream>>>,
        peer_ids: Arc<Mutex<HashSet<PeerId>>>,
    ) -> Self {
        let (internal_event_tx, internal_event_rx) =
            channel::new(1024, &counters::PENDING_PEER_MANAGER_INTERNAL_EVENTS);
        let (dial_request_tx, dial_request_rx) =
            channel::new(1024, &counters::PENDING_PEER_MANAGER_DIAL_REQUESTS);
        let (connection_handler, listen_addr) = ConnectionHandler::new(
            transport,
            listen_addr,
            dial_request_rx,
            internal_event_tx.clone(),
        );

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
            internal_event_tx,
            internal_event_rx,
            outstanding_disconnect_requests: HashMap::new(),
            phantom_transport: PhantomData,
            peer_ids,
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
                maybe_internal_event = self.internal_event_rx.next() => {
                    if let Some(event) = maybe_internal_event {
                        self.handle_internal_event(event).await;
                    }
                }
                maybe_request = self.requests_rx.next() => {
                    if let Some(request) = maybe_request {
                        self.handle_request(request).await;
                    }
                }
                complete => {
                    crit!("Peer manager actor terminated");
                    break;
                }
            }
        }
    }

    async fn handle_internal_event(&mut self, event: InternalEvent<TMuxer>) {
        trace!("InternalEvent::{:?}", event);
        match event {
            InternalEvent::NewConnection(identity, addr, origin, conn) => {
                self.add_peer(identity, addr, origin, conn).await;
            }
            InternalEvent::NewSubstream(peer_id, substream) => {
                let ch = self
                    .protocol_handlers
                    .get_mut(&substream.protocol)
                    .expect("Received substream for unknown protocol");
                let event = PeerManagerNotification::NewInboundSubstream(peer_id, substream);
                ch.send(event).await.unwrap();
            }
            InternalEvent::PeerDisconnected(peer_id, role, origin, _reason) => {
                let peer = self
                    .active_peers
                    .remove(&peer_id)
                    .expect("Should have a handle to Peer");

                self.peer_ids.lock().await.remove(&peer.peer_id());

                // If we receive a PeerDisconnected event and the connection origin isn't the same
                // as the one we have stored in PeerManager this particular event is from a Peer
                // actor which is being shutdown due to simultaneous dial tie-breaking and we don't
                // need to send a LostPeer notification to all subscribers.
                if peer.origin != origin {
                    self.active_peers.insert(peer_id.clone(), peer);
                    self.peer_ids.lock().await.insert(peer_id);
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
                    .with_label_values(&[&role.to_string(), "connected"])
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
            // The remote dialed us twice for some reason, drop the new incoming connection
            (ConnectionOrigin::Inbound, ConnectionOrigin::Inbound) => false,
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
            self.peer_ids.lock().await.remove(&peer_id);
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
                // Drop the new connection and keep the one already stored in active_peers
                connection.close().await.unwrap_or_else(|e| {
                    error!(
                        "Closing connection with Peer {} failed with error: {}",
                        peer_id.short_str(),
                        e
                    )
                });
                info!(
                    "Closing incoming connection with Peer {} to mitigate simultaneous dial",
                    peer_id.short_str()
                );
                // Put the existing connection back
                let peer_id = peer.peer_id();
                self.active_peers.insert(peer_id.clone(), peer);
                self.peer_ids.lock().await.insert(peer_id);
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
            self.internal_event_tx.clone(),
            peer_req_rx,
        );
        let peer_handle = PeerHandle::new(peer_id, address.clone(), origin, peer_req_tx);
        info!(
            "{:?} connection with peer {} established",
            origin,
            peer_id.short_str()
        );
        self.active_peers.insert(peer_id.clone(), peer_handle);
        self.peer_ids.lock().await.insert(peer_id);
        self.executor.spawn(peer.start());
        // Send NewPeer notifications to subscribers
        if send_new_peer_notification {
            // update libra_network_peer counter
            counters::LIBRA_NETWORK_PEERS
                .with_label_values(&[&role.to_string(), "connected"])
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
    internal_event_tx: channel::Sender<InternalEvent<TMuxer>>,
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
        internal_event_tx: channel::Sender<InternalEvent<TMuxer>>,
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
                internal_event_tx,
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
                    let event = InternalEvent::NewConnection(
                        identity,
                        addr,
                        ConnectionOrigin::Outbound,
                        connection,
                    );
                    // Send the new connection to PeerManager
                    self.internal_event_tx.send(event).await.unwrap();
                    Ok(())
                } else {
                    let e = ::failure::format_err!(
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
                let event = InternalEvent::NewConnection(
                    identity,
                    addr,
                    ConnectionOrigin::Inbound,
                    connection,
                );
                // Send the new connection to PeerManager
                self.internal_event_tx.send(event).await.unwrap();
            }
            Err(e) => {
                warn!("Connection from {} failed to upgrade {}", addr, e);
            }
        }
    }
}

struct PeerHandle<TSubstream> {
    peer_id: PeerId,
    sender: channel::Sender<PeerRequest<TSubstream>>,
    origin: ConnectionOrigin,
    address: Multiaddr,
    is_shutting_down: bool,
}

impl<TSubstream> PeerHandle<TSubstream> {
    pub fn new(
        peer_id: PeerId,
        address: Multiaddr,
        origin: ConnectionOrigin,
        sender: channel::Sender<PeerRequest<TSubstream>>,
    ) -> Self {
        Self {
            peer_id,
            address,
            origin,
            sender,
            is_shutting_down: false,
        }
    }

    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down
    }

    pub fn address(&self) -> &Multiaddr {
        &self.address
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn origin(&self) -> ConnectionOrigin {
        self.origin
    }

    pub async fn open_substream(
        &mut self,
        protocol: ProtocolId,
        response_tx: oneshot::Sender<Result<TSubstream, PeerManagerError>>,
    ) {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        if self
            .sender
            .send(PeerRequest::OpenSubstream(protocol, response_tx))
            .await
            .is_err()
        {
            error!(
                "Sending OpenSubstream request to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
    }

    pub async fn disconnect(&mut self) {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        if self
            .sender
            .send(PeerRequest::CloseConnection)
            .await
            .is_err()
        {
            error!(
                "Sending CloseConnection request to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
        self.is_shutting_down = true;
    }
}

#[derive(Debug)]
enum PeerRequest<TSubstream> {
    OpenSubstream(
        ProtocolId,
        oneshot::Sender<Result<TSubstream, PeerManagerError>>,
    ),
    CloseConnection,
}

struct Peer<TMuxer>
where
    TMuxer: StreamMultiplexer,
{
    /// Identity of the remote peer
    identity: Identity,
    connection: TMuxer,
    own_supported_protocols: Vec<ProtocolId>,
    internal_event_tx: channel::Sender<InternalEvent<TMuxer>>,
    requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
    origin: ConnectionOrigin,
    shutdown: bool,
}

impl<TMuxer> Peer<TMuxer>
where
    TMuxer: StreamMultiplexer + 'static,
    TMuxer::Substream: 'static,
    TMuxer::Outbound: 'static,
{
    fn new(
        identity: Identity,
        connection: TMuxer,
        origin: ConnectionOrigin,
        own_supported_protocols: Vec<ProtocolId>,
        internal_event_tx: channel::Sender<InternalEvent<TMuxer>>,
        requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
    ) -> Self {
        Self {
            identity,
            connection,
            origin,
            own_supported_protocols,
            internal_event_tx,
            requests_rx,
            shutdown: false,
        }
    }

    async fn start(mut self) {
        let mut substream_rx = self.connection.listen_for_inbound().fuse();
        let mut pending_outbound_substreams = FuturesUnordered::new();
        let mut pending_inbound_substreams = FuturesUnordered::new();

        loop {
            futures::select! {
                maybe_req = self.requests_rx.next() => {
                    if let Some(request) = maybe_req {
                        self.handle_request(&mut pending_outbound_substreams, request).await;
                    } else {
                        // This branch will only be taken if the PeerRequest sender for this Peer
                        // gets dropped.  This should never happen because PeerManager should also
                        // issue a shutdown request before dropping the sender
                        unreachable!(
                            "Peer {} PeerRequest sender gets dropped",
                            self.identity.peer_id().short_str()
                        );
                    }
                },
                maybe_substream = substream_rx.next() => {
                    match maybe_substream {
                        Some(Ok(substream)) => {
                            self.handle_inbound_substream(&mut pending_inbound_substreams, substream);
                        }
                        Some(Err(e)) => {
                            warn!("Inbound substream error {:?} with peer {}",
                                  e, self.identity.peer_id().short_str());
                            self.close_connection(DisconnectReason::ConnectionLost).await;
                        }
                        None => {
                            warn!("Inbound substreams exhausted with peer {}",
                                  self.identity.peer_id().short_str());
                            self.close_connection(DisconnectReason::ConnectionLost).await;
                        }
                    }
                },
                inbound_substream = pending_inbound_substreams.select_next_some() => {
                    match inbound_substream {
                        Ok(negotiated_substream) => {
                            let event = InternalEvent::NewSubstream(
                                self.identity.peer_id(),
                                negotiated_substream,
                            );
                            self.internal_event_tx.send(event).await.unwrap();
                        }
                        Err(e) => {
                            error!(
                                "Inbound substream negotiation for peer {} failed: {}",
                                self.identity.peer_id().short_str(), e
                            );
                        }
                    }
                },
                _ = pending_outbound_substreams.select_next_some() => {
                    // Do nothing since these futures have an output of "()"
                },
                complete => unreachable!(),
            }

            if self.shutdown {
                break;
            }
        }
        debug!(
            "Peer actor '{}' shutdown",
            self.identity.peer_id().short_str()
        );
    }

    async fn handle_request<'a>(
        &'a mut self,
        pending: &'a mut FuturesUnordered<BoxFuture<'static, ()>>,
        request: PeerRequest<TMuxer::Substream>,
    ) {
        trace!(
            "Peer {} PeerRequest::{:?}",
            self.identity.peer_id().short_str(),
            request
        );
        match request {
            PeerRequest::OpenSubstream(protocol, channel) => {
                pending.push(self.handle_open_outbound_substream_request(protocol, channel));
            }
            PeerRequest::CloseConnection => {
                self.close_connection(DisconnectReason::Requested).await;
            }
        }
    }

    fn handle_open_outbound_substream_request(
        &self,
        protocol: ProtocolId,
        channel: oneshot::Sender<Result<TMuxer::Substream, PeerManagerError>>,
    ) -> BoxFuture<'static, ()> {
        let outbound = self.connection.open_outbound();
        let optimistic_negotiation = self.identity.is_protocol_supported(&protocol);
        let negotiate = Self::negotiate_outbound_substream(
            self.identity.peer_id(),
            outbound,
            protocol,
            optimistic_negotiation,
            channel,
        );

        negotiate.boxed()
    }

    async fn negotiate_outbound_substream(
        peer_id: PeerId,
        outbound_fut: TMuxer::Outbound,
        protocol: ProtocolId,
        optimistic_negotiation: bool,
        channel: oneshot::Sender<Result<TMuxer::Substream, PeerManagerError>>,
    ) {
        let response = match outbound_fut.await {
            Ok(substream) => {
                // TODO(bmwill) Evaluate if we should still try to open and negotiate an outbound
                // substream even though we know for a fact that the Identity struct of this Peer
                // doesn't include the protocol we're interested in.
                if optimistic_negotiation {
                    negotiate_outbound_select(substream, &protocol).await
                } else {
                    warn!(
                        "Negotiating outbound substream interactively: Protocol({:?}) PeerId({})",
                        protocol,
                        peer_id.short_str()
                    );
                    negotiate_outbound_interactive(substream, [&protocol])
                        .await
                        .map(|(substream, _protocol)| substream)
                }
            }
            Err(e) => Err(e),
        }
        .map_err(Into::into);

        match response {
            Ok(_) => debug!(
                "Successfully negotiated outbound substream '{:?}' with Peer {}",
                protocol,
                peer_id.short_str()
            ),
            Err(ref e) => debug!(
                "Unable to negotiated outbound substream '{:?}' with Peer {}: {}",
                protocol,
                peer_id.short_str(),
                e
            ),
        }

        if channel.send(response).is_err() {
            warn!(
                "oneshot channel receiver dropped for new substream with peer {} for protocol {:?}",
                peer_id.short_str(),
                protocol
            );
        }
    }

    fn handle_inbound_substream<'a>(
        &'a mut self,
        pending: &'a mut FuturesUnordered<
            BoxFuture<'static, Result<NegotiatedSubstream<TMuxer::Substream>, PeerManagerError>>,
        >,
        substream: TMuxer::Substream,
    ) {
        trace!(
            "New inbound substream from peer '{}'",
            self.identity.peer_id().short_str()
        );

        let negotiate =
            Self::negotiate_inbound_substream(substream, self.own_supported_protocols.clone());
        pending.push(negotiate.boxed());
    }

    async fn negotiate_inbound_substream(
        substream: TMuxer::Substream,
        own_supported_protocols: Vec<ProtocolId>,
    ) -> Result<NegotiatedSubstream<TMuxer::Substream>, PeerManagerError> {
        let (substream, protocol) = negotiate_inbound(substream, own_supported_protocols).await?;
        Ok(NegotiatedSubstream {
            protocol,
            substream,
        })
    }

    async fn close_connection(&mut self, reason: DisconnectReason) {
        match self.connection.close().await {
            Err(e) => {
                error!(
                    "Failed to gracefully close connection with peer: {}; error: {}",
                    self.identity.peer_id().short_str(),
                    e
                );
            }
            Ok(_) => {
                info!(
                    "Closed connection with peer: {}, reason: {:?}",
                    self.identity.peer_id().short_str(),
                    reason
                );
            }
        }
        // If the graceful shutdown above fails, the connection will be forcefull terminated once
        // the connection struct is dropped. Setting the `shutdown` flag to true ensures that the
        // peer actor will terminate and close the connection in the process.
        self.shutdown = true;
        // We send a PeerDisconnected event to peer manager as a result (or in case of a failure
        // above, in anticipation of) closing the connection.

        self.internal_event_tx
            .send(InternalEvent::PeerDisconnected(
                self.identity.peer_id(),
                self.identity.role(),
                self.origin,
                reason,
            ))
            .await
            .unwrap();
    }
}
