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
    counters,
    interface::{NetworkNotification, NetworkProvider, NetworkRequest},
    peer::DisconnectReason,
    protocols::{
        direct_send::Message,
        rpc::{error::RpcError, InboundRpcRequest, OutboundRpcRequest},
    },
    transport,
    transport::{Connection, ConnectionId, ConnectionMetadata},
    ProtocolId,
};
use bytes::Bytes;
use channel::{self, libra_channel};
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    sink::SinkExt,
    stream::{Fuse, FuturesUnordered, StreamExt},
};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use netcore::transport::{ConnectionOrigin, Transport};
use parity_multiaddr::Multiaddr;
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::{TryFrom, TryInto},
    fmt::Debug,
    marker::PhantomData,
    time::Duration,
};
use tokio::runtime::Handle;

pub mod conn_status_channel;
mod error;
#[cfg(test)]
mod tests;

pub use self::error::PeerManagerError;

/// Request received by PeerManager from upstream actors.
#[derive(Debug)]
pub enum PeerManagerRequest {
    /// Send an RPC request to a remote peer.
    SendRpc(PeerId, OutboundRpcRequest),
    /// Fire-and-forget style message send to a remote peer.
    SendMessage(PeerId, Message),
}

/// Notifications sent by PeerManager to upstream actors.
#[derive(Debug)]
pub enum PeerManagerNotification {
    /// A new RPC request has been received from a remote peer.
    RecvRpc(PeerId, InboundRpcRequest),
    /// A new message has been received from a remote peer.
    RecvMessage(PeerId, Message),
}

#[derive(Debug)]
pub enum ConnectionRequest {
    DialPeer(
        PeerId,
        Multiaddr,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    DisconnectPeer(PeerId, oneshot::Sender<Result<(), PeerManagerError>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionStatusNotification {
    /// Connection with a new peer has been established.
    NewPeer(PeerId, Multiaddr),
    /// Connection to a peer has been terminated. This could have been triggered from either end.
    LostPeer(PeerId, Multiaddr, DisconnectReason),
}

/// Convenience wrapper which makes it easy to issue communication requests and await the responses
/// from PeerManager.
#[derive(Clone)]
pub struct PeerManagerRequestSender {
    inner: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
}

/// Convenience wrapper which makes it easy to issue connection requests and await the responses
/// from PeerManager.
#[derive(Clone)]
pub struct ConnectionRequestSender {
    inner: libra_channel::Sender<PeerId, ConnectionRequest>,
}

impl PeerManagerRequestSender {
    /// Construct a new PeerManagerRequestSender with a raw channel::Sender
    pub fn new(inner: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>) -> Self {
        Self { inner }
    }

    /// Send a fire-and-forget direct-send message to remote peer.
    ///
    /// The function returns when the message has been enqueued on the network actor's event queue.
    /// It therefore makes no reliable delivery guarantees. An error is returned if the event queue
    /// is unexpectedly shutdown.
    pub fn send_to(
        &mut self,
        peer_id: PeerId,
        protocol: ProtocolId,
        mdata: Bytes,
    ) -> Result<(), PeerManagerError> {
        self.inner.push(
            (peer_id, protocol),
            PeerManagerRequest::SendMessage(peer_id, Message { protocol, mdata }),
        )?;
        Ok(())
    }

    /// Send the _same_ message to many recipients using the direct-send protocol.
    ///
    /// This method is an optimization so that we can avoid serializing and
    /// copying the same message many times when we want to sent a single message
    /// to many peers. Note that the `Bytes` the messages is serialized into is a
    /// ref-counted byte buffer, so we can avoid excess copies as all direct-sends
    /// will share the same underlying byte buffer.
    ///
    /// The function returns when all send requests have been enqueued on the network
    /// actor's event queue. It therefore makes no reliable delivery guarantees.
    /// An error is returned if the event queue is unexpectedly shutdown.
    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        protocol: ProtocolId,
        mdata: Bytes,
    ) -> Result<(), PeerManagerError> {
        let msg = Message { protocol, mdata };
        for recipient in recipients {
            // We return `Err` early here if the send fails. Since sending will
            // only fail if the queue is unexpectedly shutdown (i.e., receiver
            // dropped early), we know that we can't make further progress if
            // this send fails.
            self.inner.push(
                (recipient, protocol),
                PeerManagerRequest::SendMessage(recipient, msg.clone()),
            )?;
        }
        Ok(())
    }

    /// Sends a unary RPC to a remote peer and waits to either receive a response or times out.
    pub async fn send_rpc(
        &mut self,
        peer_id: PeerId,
        protocol: ProtocolId,
        req: Bytes,
        timeout: Duration,
    ) -> Result<Bytes, RpcError> {
        let (res_tx, res_rx) = oneshot::channel();
        let request = OutboundRpcRequest {
            protocol,
            data: req,
            res_tx,
            timeout,
        };
        self.inner
            .push(
                (peer_id, protocol),
                PeerManagerRequest::SendRpc(peer_id, request),
            )
            .unwrap();
        res_rx.await?
    }
}

impl ConnectionRequestSender {
    /// Construct a new ConnectionRequestSender with a raw libra_channel::Sender
    pub fn new(inner: libra_channel::Sender<PeerId, ConnectionRequest>) -> Self {
        Self { inner }
    }

    pub async fn dial_peer(
        &mut self,
        peer: PeerId,
        addr: Multiaddr,
    ) -> Result<(), PeerManagerError> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.inner
            .push(peer, ConnectionRequest::DialPeer(peer, addr, oneshot_tx))?;
        oneshot_rx.await?
    }

    pub async fn disconnect_peer(&mut self, peer: PeerId) -> Result<(), PeerManagerError> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.inner
            .push(peer, ConnectionRequest::DisconnectPeer(peer, oneshot_tx))?;
        oneshot_rx.await?
    }
}

/// Responsible for handling and maintaining connections to other Peers
pub struct PeerManager<TTransport, TSocket>
where
    TTransport: Transport,
    TSocket: AsyncRead + AsyncWrite,
{
    /// A handle to a tokio executor.
    executor: Handle,
    /// PeerId of "self".
    own_peer_id: PeerId,
    /// Our node type.
    role: RoleType,
    /// Address to listen on for incoming connections.
    listen_addr: Multiaddr,
    /// Connection Listener, listening on `listen_addr`
    connection_handler: Option<ConnectionHandler<TTransport, TSocket>>,
    /// Map from PeerId to corresponding Peer object.
    active_peers: HashMap<
        PeerId,
        (
            ConnectionMetadata,
            libra_channel::Sender<ProtocolId, NetworkRequest>,
        ),
    >,
    /// Channel to receive requests from other actors.
    requests_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    /// Upstream handlers for RPC and DirectSend protocols. The handlers are promised fair delivery
    /// of messages across (PeerId, ProtocolId).
    upstream_handlers:
        HashMap<ProtocolId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    /// Channels to send NewPeer/LostPeer notifications to.
    connection_event_handlers: Vec<conn_status_channel::Sender>,
    /// Channel used to send Dial requests to the ConnectionHandler actor
    dial_request_tx: channel::Sender<ConnectionHandlerRequest>,
    /// Sender for connection events.
    connection_notifs_tx: channel::Sender<ConnectionNotification<TSocket>>,
    /// Receiver for connection requests.
    connection_reqs_rx: libra_channel::Receiver<PeerId, ConnectionRequest>,
    /// Receiver for connection events.
    connection_notifs_rx: channel::Receiver<ConnectionNotification<TSocket>>,
    /// A map of outstanding disconnect requests.
    outstanding_disconnect_requests:
        HashMap<ConnectionId, oneshot::Sender<Result<(), PeerManagerError>>>,
    /// Pin the transport type corresponding to this PeerManager instance
    phantom_transport: PhantomData<TTransport>,
    /// Maximum concurrent network requests to any peer.
    max_concurrent_network_reqs: usize,
    /// Maximum concurrent network notifications processed for a peer.
    max_concurrent_network_notifs: usize,
    /// Size of channels between different actors.
    channel_size: usize,
}

impl<TTransport, TSocket> PeerManager<TTransport, TSocket>
where
    TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
    TSocket: transport::TSocket,
{
    /// Construct a new PeerManager actor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        executor: Handle,
        transport: TTransport,
        own_peer_id: PeerId,
        role: RoleType,
        listen_addr: Multiaddr,
        requests_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
        connection_reqs_rx: libra_channel::Receiver<PeerId, ConnectionRequest>,
        upstream_handlers: HashMap<
            ProtocolId,
            libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        >,
        connection_event_handlers: Vec<conn_status_channel::Sender>,
        channel_size: usize,
        max_concurrent_network_reqs: usize,
        max_concurrent_network_notifs: usize,
    ) -> Self {
        let (connection_notifs_tx, connection_notifs_rx) = channel::new(
            channel_size,
            &counters::PENDING_CONNECTION_HANDLER_NOTIFICATIONS,
        );
        let (dial_request_tx, dial_request_rx) =
            channel::new(channel_size, &counters::PENDING_PEER_MANAGER_DIAL_REQUESTS);
        //TODO now that you can only listen on a socket inside of a tokio runtime we'll need to
        // rethink how we init the PeerManager so we don't have to do this funny thing.
        let connection_handler_notifs_tx = connection_notifs_tx.clone();
        let (connection_handler, listen_addr) = executor.enter(|| {
            ConnectionHandler::new(
                transport,
                listen_addr,
                dial_request_rx,
                connection_handler_notifs_tx.clone(),
            )
        });
        Self {
            executor,
            own_peer_id,
            role,
            listen_addr,
            connection_handler: Some(connection_handler),
            active_peers: HashMap::new(),
            requests_rx,
            connection_reqs_rx,
            dial_request_tx,
            connection_notifs_tx,
            connection_notifs_rx,
            outstanding_disconnect_requests: HashMap::new(),
            phantom_transport: PhantomData,
            upstream_handlers,
            connection_event_handlers,
            max_concurrent_network_reqs,
            max_concurrent_network_notifs,
            channel_size,
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
                connection_event = self.connection_notifs_rx.select_next_some() => {
                  self.handle_connection_event(connection_event);
                }
                request = self.requests_rx.select_next_some() => {
                  self.handle_request(request).await;
                }
                connection_request = self.connection_reqs_rx.select_next_some() => {
                  self.handle_connection_request(connection_request).await;
                }
                complete => {
                  // TODO: This should be ok when running in client mode.
                  crit!("Peer manager actor terminated");
                  break;
                }
            }
        }
    }

    fn handle_connection_event(&mut self, event: ConnectionNotification<TSocket>) {
        trace!("ConnectionNotification::{:?}", event);
        match event {
            ConnectionNotification::NewConnection(conn) => {
                info!("New connection established: {:?}", conn,);
                // Update libra_network_peer counter.
                counters::LIBRA_NETWORK_PEERS
                    .with_label_values(&[self.role.as_str(), "connected"])
                    .inc();
                self.add_peer(conn);
            }
            ConnectionNotification::Disconnected(lost_conn_metadata, reason) => {
                // See: https://github.com/libra/libra/issues/3128#issuecomment-605351504 for
                // detailed reasoning on `Disconnected` events should be handled correctly.
                info!(
                    "Connection {:?} closed due to {:?}",
                    lost_conn_metadata, reason,
                );
                // Update libra_network_peer counter.
                counters::LIBRA_NETWORK_PEERS
                    .with_label_values(&[self.role.as_str(), "connected"])
                    .dec();
                let peer_id = lost_conn_metadata.peer_id();
                // If the active connection with the peer is lost, remove it from `active_peers`.
                if let Entry::Occupied(entry) = self.active_peers.entry(peer_id) {
                    let (conn_metadata, _) = entry.get();
                    if conn_metadata.connection_id() == lost_conn_metadata.connection_id() {
                        // We lost an active connection.
                        entry.remove();
                    }
                }

                // If the connection was explicitly closed by an upstream client, send an ACK.
                if let Some(oneshot_tx) = self
                    .outstanding_disconnect_requests
                    .remove(&lost_conn_metadata.connection_id())
                {
                    // The client explicitly closed the connection and it should be notified.
                    if let Err(send_err) = oneshot_tx.send(Ok(())) {
                        info!(
                            "Failed to send connection close error. Error: {:?}",
                            send_err
                        );
                    }
                }

                // Notify upstream if there's still no active connection. This might be redundant,
                // but does not affect correctness.
                if !self.active_peers.contains_key(&peer_id) {
                    self.send_lostpeer_notification(
                        peer_id,
                        lost_conn_metadata.addr().clone(),
                        reason,
                    );
                }
            }
        }
    }

    async fn handle_connection_request(&mut self, request: ConnectionRequest) {
        trace!("PeerManagerRequest::{:?}", request);
        match request {
            ConnectionRequest::DialPeer(requested_peer_id, addr, response_tx) => {
                // Only dial peers which we aren't already connected with
                if let Some((curr_connection, _)) = self.active_peers.get(&requested_peer_id) {
                    let error = PeerManagerError::AlreadyConnected(curr_connection.addr().clone());
                    debug!(
                        "Already connected with Peer {} using connection {:?}. Not dialing address {}",
                        requested_peer_id.short_str(),
                        curr_connection,
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
            ConnectionRequest::DisconnectPeer(peer_id, resp_tx) => {
                // Send a CloseConnection request to NetworkProvider and drop the send end of the
                // NetworkRequest channel.
                if let Some((conn_metadata, sender)) = self.active_peers.remove(&peer_id) {
                    // This should trigger a disconnect.
                    drop(sender);
                    // Add to outstanding disconnect requests.
                    self.outstanding_disconnect_requests
                        .insert(conn_metadata.connection_id(), resp_tx);
                } else {
                    info!(
                        "Connection with peer: {} is already closed",
                        peer_id.short_str(),
                    );
                    if let Err(err) = resp_tx.send(Err(PeerManagerError::NotConnected(peer_id))) {
                        info!(
                            "Failed to indicate that connection is already closed. Error: {:?}",
                            err
                        );
                    }
                }
            }
        }
    }

    async fn handle_request(&mut self, request: PeerManagerRequest) {
        trace!("PeerManagerRequest::{:?}", request);
        match request {
            PeerManagerRequest::SendMessage(peer_id, msg) => {
                if let Some((_, sender)) = self.active_peers.get_mut(&peer_id) {
                    if let Err(err) = sender.push(msg.protocol, NetworkRequest::SendMessage(msg)) {
                        info!(
                            "Failed to forward outbound message to downstream actor. Error:
                              {:?}",
                            err
                        );
                    }
                } else {
                    warn!("Peer {} is not connected", peer_id.short_str());
                }
            }
            PeerManagerRequest::SendRpc(peer_id, req) => {
                if let Some((_, sender)) = self.active_peers.get_mut(&peer_id) {
                    if let Err(err) = sender.push(req.protocol, NetworkRequest::SendRpc(req)) {
                        info!(
                            "Failed to forward outbound rpc to downstream actor. Error:
                            {:?}",
                            err
                        );
                    }
                } else {
                    warn!("Peer {} is not connected", peer_id.short_str());
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
            // We should never dial the same peer twice, but if we do drop the old connection
            (ConnectionOrigin::Outbound, ConnectionOrigin::Outbound) => true,
            (ConnectionOrigin::Inbound, ConnectionOrigin::Outbound) => remote_peer_id < own_peer_id,
            (ConnectionOrigin::Outbound, ConnectionOrigin::Inbound) => own_peer_id < remote_peer_id,
        }
    }

    fn add_peer(&mut self, connection: Connection<TSocket>) {
        let conn_meta = connection.metadata.clone();
        let peer_id = conn_meta.peer_id();
        assert_ne!(self.own_peer_id, peer_id);

        let mut send_new_peer_notification = true;

        // Check for and handle simultaneous dialing
        if let Entry::Occupied(active_entry) = self.active_peers.entry(peer_id) {
            let (curr_conn_metadata, _) = active_entry.get();
            if Self::simultaneous_dial_tie_breaking(
                self.own_peer_id,
                peer_id,
                curr_conn_metadata.origin(),
                conn_meta.origin(),
            ) {
                let (_, peer_handle) = active_entry.remove();
                // Drop the existing connection and replace it with the new connection
                drop(peer_handle);
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
                let drop_fut = async move {
                    let mut connection = connection;
                    if let Err(e) = tokio::time::timeout(
                        transport::TRANSPORT_TIMEOUT,
                        connection.socket.close(),
                    )
                    .await
                    {
                        error!(
                            "Closing connection with Peer {} failed with error: {}",
                            peer_id.short_str(),
                            e
                        );
                    };
                };
                self.executor.spawn(drop_fut);
                return;
            }
        }

        // Initialize a new network stack for this connection.
        let (network_reqs_tx, network_notifs_rx) = NetworkProvider::start(
            self.executor.clone(),
            connection,
            self.connection_notifs_tx.clone(),
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
            self.channel_size,
        );
        // Start background task to handle events (RPCs and DirectSend messages) received from
        // peer.
        self.spawn_peer_network_events_handler(peer_id, network_notifs_rx);
        // Save NetworkRequest sender to `active_peers`.
        self.active_peers
            .insert(peer_id, (conn_meta.clone(), network_reqs_tx));
        // Send NewPeer notification to connection event handlers.
        if send_new_peer_notification {
            for handler in self.connection_event_handlers.iter_mut() {
                handler
                    .push(
                        peer_id,
                        ConnectionStatusNotification::NewPeer(peer_id, conn_meta.addr().clone()),
                    )
                    .unwrap();
            }
        }
    }

    fn send_lostpeer_notification(
        &mut self,
        peer_id: PeerId,
        addr: Multiaddr,
        reason: DisconnectReason,
    ) {
        // Send LostPeer notification to connection event handlers.
        for handler in self.connection_event_handlers.iter_mut() {
            if let Err(e) = handler.push(
                peer_id,
                ConnectionStatusNotification::LostPeer(peer_id, addr.clone(), reason),
            ) {
                warn!(
                    "Failed to send lost peer notification to handler for peer: {}. Error: {:?}",
                    peer_id.short_str(),
                    e
                );
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

    fn spawn_peer_network_events_handler(
        &self,
        peer_id: PeerId,
        network_events: libra_channel::Receiver<ProtocolId, NetworkNotification>,
    ) {
        let mut upstream_handlers = self.upstream_handlers.clone();
        self.executor.spawn(network_events.for_each_concurrent(
            self.max_concurrent_network_reqs,
            move |inbound_event| {
                Self::handle_inbound_event(inbound_event, peer_id, &mut upstream_handlers);
                futures::future::ready(())
            },
        ));
    }

    fn handle_inbound_event(
        inbound_event: NetworkNotification,
        peer_id: PeerId,
        upstream_handlers: &mut HashMap<
            ProtocolId,
            libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        >,
    ) {
        match inbound_event {
            NetworkNotification::RecvMessage(msg) => {
                let protocol = msg.protocol;
                if let Some(handler) = upstream_handlers.get_mut(&protocol) {
                    // Send over libra channel for fairness.
                    if let Err(err) = handler.push(
                        (peer_id, protocol),
                        PeerManagerNotification::RecvMessage(peer_id, msg),
                    ) {
                        warn!(
                            "Upstream handler unable to handle messages for protocol: {:?}. Error:
                            {:?}",
                            protocol, err
                        );
                    }
                } else {
                    unreachable!("Received network event for unregistered protocol");
                }
            }
            NetworkNotification::RecvRpc(rpc_req) => {
                let protocol = rpc_req.protocol;
                if let Some(handler) = upstream_handlers.get_mut(&protocol) {
                    // Send over libra channel for fairness.
                    if let Err(err) = handler.push(
                        (peer_id, protocol),
                        PeerManagerNotification::RecvRpc(peer_id, rpc_req),
                    ) {
                        warn!(
                            "Upstream handler unable to handle rpc for protocol: {:?}. Error:
                              {:?}",
                            protocol, err
                        );
                    }
                } else {
                    unreachable!("Received network event for unregistered protocol");
                }
            }
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
pub enum ConnectionNotification<TSocket>
where
    TSocket: AsyncRead + AsyncWrite,
{
    NewConnection(Connection<TSocket>),
    Disconnected(ConnectionMetadata, DisconnectReason),
}

/// Responsible for listening for new incoming connections
struct ConnectionHandler<TTransport, TSocket>
where
    TTransport: Transport,
    TSocket: AsyncRead + AsyncWrite,
{
    /// [`Transport`] that is used to establish connections
    transport: TTransport,
    listener: Fuse<TTransport::Listener>,
    dial_request_rx: channel::Receiver<ConnectionHandlerRequest>,
    connection_notifs_tx: channel::Sender<ConnectionNotification<TSocket>>,
}

impl<TTransport, TSocket> ConnectionHandler<TTransport, TSocket>
where
    TTransport: Transport<Output = Connection<TSocket>>,
    TTransport::Listener: 'static,
    TTransport::Inbound: 'static,
    TTransport::Outbound: 'static,
    TSocket: AsyncRead + AsyncWrite + 'static,
{
    fn new(
        transport: TTransport,
        listen_addr: Multiaddr,
        dial_request_rx: channel::Receiver<ConnectionHandlerRequest>,
        connection_notifs_tx: channel::Sender<ConnectionNotification<TSocket>>,
    ) -> (Self, Multiaddr) {
        let (listener, listen_addr) = transport
            .listen_on(NetworkAddress::try_from(listen_addr).unwrap())
            .expect("Transport listen on fails");
        debug!("listening on {:?}", listen_addr);
        (
            Self {
                transport,
                listener: listener.fuse(),
                dial_request_rx,
                connection_notifs_tx,
            },
            listen_addr.try_into().unwrap(),
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
                    self.handle_completed_inbound_upgrade(upgrade, addr.try_into().unwrap()).await;
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
                Result<Connection<TSocket>, TTransport::Error>,
                Multiaddr,
                PeerId,
                oneshot::Sender<Result<(), PeerManagerError>>,
            ),
        >,
    > {
        match dial_peer_request {
            ConnectionHandlerRequest::DialPeer(peer_id, addr, response_tx) => {
                let addr = NetworkAddress::try_from(addr).unwrap();
                match self.transport.dial(addr.clone()) {
                    Ok(upgrade) => Some(
                        upgrade
                            .map(move |out| (out, addr.try_into().unwrap(), peer_id, response_tx))
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
        upgrade: Result<Connection<TSocket>, TTransport::Error>,
        addr: Multiaddr,
        peer_id: PeerId,
        response_tx: oneshot::Sender<Result<(), PeerManagerError>>,
    ) {
        match upgrade {
            Ok(connection) => {
                let dialed_peer_id = connection.metadata.peer_id();
                let response = if dialed_peer_id == peer_id {
                    debug!(
                        "Peer '{}' successfully dialed at '{}'",
                        peer_id.short_str(),
                        addr
                    );
                    let event = ConnectionNotification::NewConnection(connection);
                    // Send the new connection to PeerManager
                    self.connection_notifs_tx.send(event).await.unwrap();
                    Ok(())
                } else {
                    let e = ::anyhow::format_err!(
                        "Dialed PeerId ({}) differs from expected PeerId ({})",
                        dialed_peer_id.short_str(),
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
        upgrade: Result<Connection<TSocket>, TTransport::Error>,
        addr: Multiaddr,
    ) {
        match upgrade {
            Ok(connection) => {
                debug!(
                    "Connection from {} at {} successfully upgraded",
                    connection.metadata.peer_id().short_str(),
                    addr
                );
                let event = ConnectionNotification::NewConnection(connection);
                // Send the new connection to PeerManager
                self.connection_notifs_tx.send(event).await.unwrap();
            }
            Err(e) => {
                warn!("Connection from {} failed to upgrade {}", addr, e);
            }
        }
    }
}
