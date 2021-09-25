// Copyright (c) The Diem Core Contributors
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
    constants,
    counters::{self},
    logging::*,
    peer::{Peer, PeerNotification, PeerRequest},
    transport::{
        Connection, ConnectionId, ConnectionMetadata, TSocket as TransportTSocket,
        TRANSPORT_TIMEOUT,
    },
    ProtocolId,
};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::network_id::NetworkContext;
use diem_logger::prelude::*;
use diem_rate_limiter::rate_limit::TokenBucketRateLimiter;
use diem_time_service::{TimeService, TimeServiceTrait};
use diem_types::{network_address::NetworkAddress, PeerId};
use futures::{
    channel::oneshot,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    sink::SinkExt,
    stream::StreamExt,
};
use netcore::transport::{ConnectionOrigin, Transport};
use short_hex_str::AsShortHexStr;
use std::{
    collections::{hash_map::Entry, BTreeMap},
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    time::Duration,
};
use tokio::runtime::Handle;

pub mod builder;
pub mod conn_notifs_channel;
mod error;
mod senders;
#[cfg(test)]
mod tests;
mod transport;
mod types;

pub use self::error::PeerManagerError;
use crate::{
    application::storage::PeerMetadataStorage,
    peer_manager::transport::{TransportHandler, TransportRequest},
};
use diem_config::config::{PeerRole, PeerSet};
use diem_infallible::RwLock;
pub use senders::*;
pub use types::*;

pub type IpAddrTokenBucketLimiter = TokenBucketRateLimiter<IpAddr>;

/// Responsible for handling and maintaining connections to other Peers
pub struct PeerManager<TTransport, TSocket>
where
    TTransport: Transport,
    TSocket: AsyncRead + AsyncWrite,
{
    network_context: NetworkContext,
    /// A handle to a tokio executor.
    executor: Handle,
    /// A handle to a time service for easily mocking time-related operations.
    time_service: TimeService,
    /// Address to listen on for incoming connections.
    listen_addr: NetworkAddress,
    /// Connection Listener, listening on `listen_addr`
    transport_handler: Option<TransportHandler<TTransport, TSocket>>,
    /// Map from PeerId to corresponding Peer object.
    active_peers: BTreeMap<
        PeerId,
        (
            ConnectionMetadata,
            diem_channel::Sender<ProtocolId, PeerRequest>,
        ),
    >,
    /// Shared metadata storage about peers
    peer_metadata_storage: Arc<PeerMetadataStorage>,
    /// Known trusted peers from discovery
    trusted_peers: Arc<RwLock<PeerSet>>,
    /// Channel to receive requests from other actors.
    requests_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    /// Upstream handlers for RPC and DirectSend protocols. The handlers are promised fair delivery
    /// of messages across (PeerId, ProtocolId).
    upstream_handlers:
        BTreeMap<ProtocolId, diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    /// Channels to send NewPeer/LostPeer notifications to.
    connection_event_handlers: Vec<conn_notifs_channel::Sender>,
    /// Channel used to send Dial requests to the ConnectionHandler actor
    transport_reqs_tx: channel::Sender<TransportRequest>,
    /// Sender for connection events.
    transport_notifs_tx: channel::Sender<TransportNotification<TSocket>>,
    /// Receiver for connection requests.
    connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,
    /// Receiver for connection events.
    transport_notifs_rx: channel::Receiver<TransportNotification<TSocket>>,
    /// A map of outstanding disconnect requests.
    outstanding_disconnect_requests:
        BTreeMap<ConnectionId, oneshot::Sender<Result<(), PeerManagerError>>>,
    /// Pin the transport type corresponding to this PeerManager instance
    phantom_transport: PhantomData<TTransport>,
    /// Maximum concurrent network requests to any peer.
    max_concurrent_network_reqs: usize,
    /// Size of channels between different actors.
    channel_size: usize,
    /// Max network frame size
    max_frame_size: usize,
    /// Inbound connection limit separate of outbound connections
    inbound_connection_limit: usize,
    /// Keyed storage of all inbound rate limiters
    inbound_rate_limiters: IpAddrTokenBucketLimiter,
    /// Keyed storage of all outbound rate limiters
    outbound_rate_limiters: IpAddrTokenBucketLimiter,
}

impl<TTransport, TSocket> PeerManager<TTransport, TSocket>
where
    TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
    TSocket: TransportTSocket,
{
    /// Construct a new PeerManager actor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        executor: Handle,
        time_service: TimeService,
        transport: TTransport,
        network_context: NetworkContext,
        listen_addr: NetworkAddress,
        peer_metadata_storage: Arc<PeerMetadataStorage>,
        trusted_peers: Arc<RwLock<PeerSet>>,
        requests_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
        connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,
        upstream_handlers: BTreeMap<
            ProtocolId,
            diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        >,
        connection_event_handlers: Vec<conn_notifs_channel::Sender>,
        channel_size: usize,
        max_concurrent_network_reqs: usize,
        max_frame_size: usize,
        inbound_connection_limit: usize,
        inbound_rate_limiters: IpAddrTokenBucketLimiter,
        outbound_rate_limiters: IpAddrTokenBucketLimiter,
    ) -> Self {
        let (transport_notifs_tx, transport_notifs_rx) = channel::new(
            channel_size,
            &counters::PENDING_CONNECTION_HANDLER_NOTIFICATIONS,
        );
        let (transport_reqs_tx, transport_reqs_rx) =
            channel::new(channel_size, &counters::PENDING_PEER_MANAGER_DIAL_REQUESTS);
        //TODO now that you can only listen on a socket inside of a tokio runtime we'll need to
        // rethink how we init the PeerManager so we don't have to do this funny thing.
        let transport_notifs_tx_clone = transport_notifs_tx.clone();
        let _guard = executor.enter();
        let (transport_handler, listen_addr) = TransportHandler::new(
            network_context,
            time_service.clone(),
            transport,
            listen_addr,
            transport_reqs_rx,
            transport_notifs_tx_clone,
        );

        Self {
            network_context,
            executor,
            time_service,
            listen_addr,
            transport_handler: Some(transport_handler),
            active_peers: BTreeMap::new(),
            peer_metadata_storage,
            trusted_peers,
            requests_rx,
            connection_reqs_rx,
            transport_reqs_tx,
            transport_notifs_tx,
            transport_notifs_rx,
            outstanding_disconnect_requests: BTreeMap::new(),
            phantom_transport: PhantomData,
            upstream_handlers,
            connection_event_handlers,
            max_concurrent_network_reqs,
            channel_size,
            max_frame_size,
            inbound_connection_limit,
            inbound_rate_limiters,
            outbound_rate_limiters,
        }
    }

    pub fn update_connected_peers_metrics(&self) {
        let total = self.active_peers.len();
        let inbound = self
            .active_peers
            .iter()
            .filter(|(_, (metadata, _))| metadata.origin == ConnectionOrigin::Inbound)
            .count();
        let outbound = total.saturating_sub(inbound);

        counters::connections(&self.network_context, ConnectionOrigin::Inbound).set(inbound as i64);
        counters::connections(&self.network_context, ConnectionOrigin::Outbound)
            .set(outbound as i64);
    }

    fn sample_connected_peers(&self) {
        // Sample final state at most once a minute, ensuring consistent ordering
        sample!(SampleRate::Duration(Duration::from_secs(60)), {
            let peers: Vec<_> = self
                .active_peers
                .values()
                .map(|(connection, _)| {
                    (
                        connection.remote_peer_id,
                        connection.addr.clone(),
                        connection.origin,
                    )
                })
                .collect();
            info!(
                NetworkSchema::new(&self.network_context),
                peers = ?peers,
                "Current connected peers"
            )
        });
    }

    /// Get the [`NetworkAddress`] we're listening for incoming connections on
    pub fn listen_addr(&self) -> &NetworkAddress {
        &self.listen_addr
    }

    /// Start listening on the set address and return a future which runs PeerManager
    pub async fn start(mut self) {
        // Start listening for connections.
        info!(
            NetworkSchema::new(&self.network_context),
            "Start listening for incoming connections on {}", self.listen_addr
        );
        self.start_connection_listener();
        loop {
            ::futures::select! {
                connection_event = self.transport_notifs_rx.select_next_some() => {
                    self.handle_connection_event(connection_event);
                }
                connection_request = self.connection_reqs_rx.select_next_some() => {
                    self.handle_outbound_connection_request(connection_request).await;
                }
                request = self.requests_rx.select_next_some() => {
                    self.handle_outbound_request(request).await;
                }
                complete => {
                    break;
                }
            }
        }

        warn!(
            NetworkSchema::new(&self.network_context),
            "PeerManager actor terminated"
        );
    }

    fn handle_connection_event(&mut self, event: TransportNotification<TSocket>) {
        trace!(
            NetworkSchema::new(&self.network_context),
            transport_notification = format!("{:?}", event),
            "{} TransportNotification::{:?}",
            self.network_context,
            event
        );
        self.sample_connected_peers();
        match event {
            TransportNotification::NewConnection(mut conn) => {
                match conn.metadata.origin {
                    ConnectionOrigin::Outbound => {
                        // TODO: This is right now a hack around having to feed trusted peers deeper in the outbound path.  Inbound ones are assigned at Noise handshake time.
                        conn.metadata.role = self
                            .trusted_peers
                            .read()
                            .get(&conn.metadata.remote_peer_id)
                            .map_or(PeerRole::Unknown, |auth_context| auth_context.role);

                        if conn.metadata.role == PeerRole::Unknown {
                            warn!(
                                NetworkSchema::new(&self.network_context)
                                    .connection_metadata_with_address(&conn.metadata),
                                "{} Outbound connection made with unknown peer role: {}",
                                self.network_context,
                                conn.metadata
                            )
                        }
                    }
                    ConnectionOrigin::Inbound => {
                        // Everything below here is meant for unknown peers only, role comes from
                        // Noise handshake and if it's not `Unknown` it is trusted
                        if conn.metadata.role == PeerRole::Unknown {
                            // TODO: Keep track of somewhere else to not take this hit in case of DDoS
                            // Count unknown inbound connections
                            let unknown_inbound_conns = self
                                .active_peers
                                .iter()
                                .filter(|(peer_id, (metadata, _))| {
                                    metadata.origin == ConnectionOrigin::Inbound
                                        && self
                                            .trusted_peers
                                            .read()
                                            .get(peer_id)
                                            .map_or(true, |peer| peer.role == PeerRole::Unknown)
                                })
                                .count();

                            // Reject excessive inbound connections made by unknown peers
                            // We control outbound connections with Connectivity manager before we even send them
                            // and we must allow connections that already exist to pass through tie breaking.
                            if !self
                                .active_peers
                                .contains_key(&conn.metadata.remote_peer_id)
                                && unknown_inbound_conns + 1 > self.inbound_connection_limit
                            {
                                info!(
                                    NetworkSchema::new(&self.network_context)
                                        .connection_metadata_with_address(&conn.metadata),
                                    "{} Connection rejected due to connection limit: {}",
                                    self.network_context,
                                    conn.metadata
                                );
                                counters::connections_rejected(
                                    &self.network_context,
                                    conn.metadata.origin,
                                )
                                .inc();
                                self.disconnect(conn);
                                return;
                            }
                        }
                    }
                }

                // Add new peer, updating counters and all
                info!(
                    NetworkSchema::new(&self.network_context)
                        .connection_metadata_with_address(&conn.metadata),
                    "{} New connection established: {}", self.network_context, conn.metadata
                );
                self.add_peer(conn);
                self.update_connected_peers_metrics();
            }
            TransportNotification::Disconnected(lost_conn_metadata, reason) => {
                // See: https://github.com/diem/diem/issues/3128#issuecomment-605351504 for
                // detailed reasoning on `Disconnected` events should be handled correctly.
                info!(
                    NetworkSchema::new(&self.network_context)
                        .connection_metadata_with_address(&lost_conn_metadata),
                    disconnection_reason = reason,
                    "{} Connection {} closed due to {}",
                    self.network_context,
                    lost_conn_metadata,
                    reason
                );
                let peer_id = lost_conn_metadata.remote_peer_id;
                // If the active connection with the peer is lost, remove it from `active_peers`.
                if let Entry::Occupied(entry) = self.active_peers.entry(peer_id) {
                    let (conn_metadata, _) = entry.get();
                    if conn_metadata.connection_id == lost_conn_metadata.connection_id {
                        // We lost an active connection.
                        entry.remove();
                        self.peer_metadata_storage
                            .remove_connection(&lost_conn_metadata)
                    }
                }
                self.update_connected_peers_metrics();

                // If the connection was explicitly closed by an upstream client, send an ACK.
                if let Some(oneshot_tx) = self
                    .outstanding_disconnect_requests
                    .remove(&lost_conn_metadata.connection_id)
                {
                    // The client explicitly closed the connection and it should be notified.
                    if let Err(send_err) = oneshot_tx.send(Ok(())) {
                        info!(
                            NetworkSchema::new(&self.network_context),
                            error = ?send_err,
                            "{} Failed to notify upstream client of closed connection for peer {}: {:?}",
                            self.network_context,
                            peer_id,
                            send_err
                        );
                    }
                }

                let ip_addr = lost_conn_metadata
                    .addr
                    .find_ip_addr()
                    .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

                // Notify upstream if there's still no active connection. This might be redundant,
                // but does not affect correctness.
                if !self.active_peers.contains_key(&peer_id) {
                    let notif = ConnectionNotification::LostPeer(
                        lost_conn_metadata,
                        self.network_context,
                        reason,
                    );
                    self.send_conn_notification(peer_id, notif);
                }

                // Garbage collect unused rate limit buckets
                self.inbound_rate_limiters.try_garbage_collect_key(&ip_addr);
                self.outbound_rate_limiters
                    .try_garbage_collect_key(&ip_addr);
            }
        }
    }

    async fn handle_outbound_connection_request(&mut self, request: ConnectionRequest) {
        trace!(
            NetworkSchema::new(&self.network_context),
            peer_manager_request = request,
            "{} PeerManagerRequest::{:?}",
            self.network_context,
            request
        );
        self.sample_connected_peers();
        match request {
            ConnectionRequest::DialPeer(requested_peer_id, addr, response_tx) => {
                // Only dial peers which we aren't already connected with
                if let Some((curr_connection, _)) = self.active_peers.get(&requested_peer_id) {
                    let error = PeerManagerError::AlreadyConnected(curr_connection.addr.clone());
                    debug!(
                        NetworkSchema::new(&self.network_context)
                            .connection_metadata_with_address(curr_connection),
                        "{} Already connected to Peer {} with connection {:?}. Not dialing address {}",
                        self.network_context,
                        requested_peer_id.short_str(),
                        curr_connection,
                        addr
                    );
                    if let Err(send_err) = response_tx.send(Err(error)) {
                        info!(
                            NetworkSchema::new(&self.network_context)
                                .remote_peer(&requested_peer_id),
                            "{} Failed to notify that peer is already connected for Peer {}: {:?}",
                            self.network_context,
                            requested_peer_id.short_str(),
                            send_err
                        );
                    }
                } else {
                    let request = TransportRequest::DialPeer(requested_peer_id, addr, response_tx);
                    self.transport_reqs_tx.send(request).await.unwrap();
                };
            }
            ConnectionRequest::DisconnectPeer(peer_id, resp_tx) => {
                // Send a CloseConnection request to Peer and drop the send end of the
                // PeerRequest channel.
                if let Some((conn_metadata, sender)) = self.active_peers.remove(&peer_id) {
                    let connection_id = conn_metadata.connection_id;
                    self.peer_metadata_storage.remove_connection(&conn_metadata);

                    // This triggers a disconnect.
                    drop(sender);
                    // Add to outstanding disconnect requests.
                    self.outstanding_disconnect_requests
                        .insert(connection_id, resp_tx);
                } else {
                    info!(
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        "{} Connection with peer: {} was already closed",
                        self.network_context,
                        peer_id.short_str(),
                    );
                    if let Err(err) = resp_tx.send(Err(PeerManagerError::NotConnected(peer_id))) {
                        info!(
                            NetworkSchema::new(&self.network_context),
                            error = ?err,
                            "{} Failed to notify that connection was already closed for Peer {}: {:?}",
                            self.network_context,
                            peer_id,
                            err
                        );
                    }
                }
            }
        }
    }

    async fn handle_outbound_request(&mut self, request: PeerManagerRequest) {
        trace!(
            NetworkSchema::new(&self.network_context),
            peer_manager_request = request,
            "{} PeerManagerRequest::{:?}",
            self.network_context,
            request
        );
        self.sample_connected_peers();
        match request {
            PeerManagerRequest::SendDirectSend(peer_id, msg) => {
                let protocol_id = msg.protocol_id;
                if let Some((conn_metadata, sender)) = self.active_peers.get_mut(&peer_id) {
                    if let Err(err) = sender.push(msg.protocol_id, PeerRequest::SendDirectSend(msg))
                    {
                        info!(
                            NetworkSchema::new(&self.network_context).connection_metadata(conn_metadata),
                            protocol_id = %protocol_id,
                            error = ?err,
                            "{} Failed to forward outbound message to downstream actor. Error: {:?}",
                            self.network_context, err
                        );
                    }
                } else {
                    warn!(
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        protocol_id = %protocol_id,
                        "{} Can't send message to peer.  Peer {} is currently not connected",
                        self.network_context,
                        peer_id.short_str()
                    );
                }
            }
            PeerManagerRequest::SendRpc(peer_id, req) => {
                let protocol_id = req.protocol_id;
                if let Some((conn_metadata, sender)) = self.active_peers.get_mut(&peer_id) {
                    if let Err(err) = sender.push(req.protocol_id, PeerRequest::SendRpc(req)) {
                        info!(
                            NetworkSchema::new(&self.network_context)
                                .connection_metadata(conn_metadata),
                            protocol_id = %protocol_id,
                            error = ?err,
                            "{} Failed to forward outbound rpc to downstream actor. Error: {:?}",
                            self.network_context,
                            err
                        );
                    }
                } else {
                    warn!(
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        protocol_id = %protocol_id,
                        "{} Can't send RPC message to peer.  Peer {} is currently not connected",
                        self.network_context,
                        peer_id.short_str()
                    );
                }
            }
        }
    }

    fn start_connection_listener(&mut self) {
        let transport_handler = self
            .transport_handler
            .take()
            .expect("Transport handler already taken");
        self.executor.spawn(transport_handler.listen());
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

    fn disconnect(&mut self, connection: Connection<TSocket>) {
        let network_context = self.network_context;
        let time_service = self.time_service.clone();

        // Close connection, and drop it
        let drop_fut = async move {
            let mut connection = connection;
            let peer_id = connection.metadata.remote_peer_id;
            if let Err(e) = time_service
                .timeout(TRANSPORT_TIMEOUT, connection.socket.close())
                .await
            {
                error!(
                    NetworkSchema::new(&network_context)
                        .remote_peer(&peer_id),
                    error = %e,
                    "{} Closing connection with Peer {} failed with error: {}",
                    network_context,
                    peer_id.short_str(),
                    e
                );
            };
        };
        self.executor.spawn(drop_fut);
    }

    fn add_peer(&mut self, connection: Connection<TSocket>) {
        let conn_meta = connection.metadata.clone();
        let peer_id = conn_meta.remote_peer_id;

        // Make a disconnect if you've connected to yourself
        if self.network_context.peer_id() == peer_id {
            debug_assert!(false, "Self dials shouldn't happen");
            warn!(
                NetworkSchema::new(&self.network_context)
                    .connection_metadata_with_address(&conn_meta),
                "Received self-dial, disconnecting it"
            );
            self.disconnect(connection);
            return;
        }

        let mut send_new_peer_notification = true;

        // Check for and handle simultaneous dialing
        if let Entry::Occupied(active_entry) = self.active_peers.entry(peer_id) {
            let (curr_conn_metadata, _) = active_entry.get();
            if Self::simultaneous_dial_tie_breaking(
                self.network_context.peer_id(),
                peer_id,
                curr_conn_metadata.origin,
                conn_meta.origin,
            ) {
                let (_, peer_handle) = active_entry.remove();
                // Drop the existing connection and replace it with the new connection
                drop(peer_handle);
                info!(
                    NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                    "{} Closing existing connection with Peer {} to mitigate simultaneous dial",
                    self.network_context,
                    peer_id.short_str()
                );
                send_new_peer_notification = false;
            } else {
                info!(
                    NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                    "{} Closing incoming connection with Peer {} to mitigate simultaneous dial",
                    self.network_context,
                    peer_id.short_str()
                );
                // Drop the new connection and keep the one already stored in active_peers
                self.disconnect(connection);
                return;
            }
        }

        let ip_addr = connection
            .metadata
            .addr
            .find_ip_addr()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let inbound_rate_limiter = self.inbound_rate_limiters.bucket(ip_addr);
        let outbound_rate_limiter = self.outbound_rate_limiters.bucket(ip_addr);

        // TODO: Add label for peer.
        let (peer_reqs_tx, peer_reqs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            self.channel_size,
            Some(&counters::PENDING_NETWORK_REQUESTS),
        );
        // TODO: Add label for peer.
        let (peer_notifs_tx, peer_notifs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            self.channel_size,
            Some(&counters::PENDING_NETWORK_NOTIFICATIONS),
        );

        // Initialize a new Peer actor for this connection.
        let peer = Peer::new(
            self.network_context,
            self.executor.clone(),
            self.time_service.clone(),
            connection,
            self.transport_notifs_tx.clone(),
            peer_reqs_rx,
            peer_notifs_tx,
            Duration::from_millis(constants::INBOUND_RPC_TIMEOUT_MS),
            constants::MAX_CONCURRENT_INBOUND_RPCS,
            constants::MAX_CONCURRENT_OUTBOUND_RPCS,
            self.max_frame_size,
            Some(inbound_rate_limiter),
            Some(outbound_rate_limiter),
        );
        self.executor.spawn(peer.start());

        // Start background task to handle events (RPCs and DirectSend messages) received from
        // peer.
        self.spawn_peer_network_events_handler(peer_id, peer_notifs_rx);
        // Save PeerRequest sender to `active_peers`.
        self.active_peers
            .insert(peer_id, (conn_meta.clone(), peer_reqs_tx));
        self.peer_metadata_storage
            .insert_connection(conn_meta.clone());
        // Send NewPeer notification to connection event handlers.
        if send_new_peer_notification {
            let notif = ConnectionNotification::NewPeer(conn_meta, self.network_context);
            self.send_conn_notification(peer_id, notif);
        }
    }

    /// Sends a `ConnectionNotification` to all event handlers, warns on failures
    fn send_conn_notification(&mut self, peer_id: PeerId, notification: ConnectionNotification) {
        for handler in self.connection_event_handlers.iter_mut() {
            if let Err(e) = handler.push(peer_id, notification.clone()) {
                warn!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&peer_id),
                    error = ?e,
                    connection_notification = notification,
                    "{} Failed to send notification {} to handler for peer: {}. Error: {:?}",
                    self.network_context,
                    notification,
                    peer_id.short_str(),
                    e
                );
            }
        }
    }

    fn spawn_peer_network_events_handler(
        &self,
        peer_id: PeerId,
        network_events: diem_channel::Receiver<ProtocolId, PeerNotification>,
    ) {
        let mut upstream_handlers = self.upstream_handlers.clone();
        let network_context = self.network_context;
        self.executor.spawn(network_events.for_each_concurrent(
            self.max_concurrent_network_reqs,
            move |inbound_event| {
                Self::handle_inbound_event(
                    network_context,
                    inbound_event,
                    peer_id,
                    &mut upstream_handlers,
                );
                futures::future::ready(())
            },
        ));
    }

    fn handle_inbound_event(
        network_context: NetworkContext,
        inbound_event: PeerNotification,
        peer_id: PeerId,
        upstream_handlers: &mut BTreeMap<
            ProtocolId,
            diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        >,
    ) {
        match inbound_event {
            PeerNotification::RecvMessage(msg) => {
                let protocol_id = msg.protocol_id;
                if let Some(handler) = upstream_handlers.get_mut(&protocol_id) {
                    // Send over diem channel for fairness.
                    if let Err(send_err) = handler.push(
                        (peer_id, protocol_id),
                        PeerManagerNotification::RecvMessage(peer_id, msg),
                    ) {
                        warn!(
                            NetworkSchema::new(&network_context),
                            error = ?send_err,
                            protocol_id = protocol_id,
                            "{} Upstream handler unable to handle messages for protocol: {}. Error: {:?}",
                            network_context, protocol_id, send_err
                        );
                    }
                } else {
                    debug!(
                        NetworkSchema::new(&network_context),
                        message = format!("{:?}", msg),
                        "{} Received network message for unregistered protocol. Message: {:?}",
                        network_context,
                        msg,
                    );
                }
            }
            PeerNotification::RecvRpc(rpc_req) => {
                let protocol_id = rpc_req.protocol_id;
                if let Some(handler) = upstream_handlers.get_mut(&protocol_id) {
                    // Send over diem channel for fairness.
                    if let Err(err) = handler.push(
                        (peer_id, protocol_id),
                        PeerManagerNotification::RecvRpc(peer_id, rpc_req),
                    ) {
                        warn!(
                            NetworkSchema::new(&network_context),
                            error = ?err,
                            "{} Upstream handler unable to handle rpc for protocol: {}. Error: {:?}",
                            network_context, protocol_id, err
                        );
                    }
                } else {
                    debug!(
                        NetworkSchema::new(&network_context),
                        "{} Received network rpc request for unregistered protocol. RPC: {:?}",
                        network_context,
                        rpc_req,
                    );
                }
            }
        }
    }
}
