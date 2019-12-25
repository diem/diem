// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Peer actor owns the underlying connection and is responsible for listening for
//!  and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{
    common::NegotiatedSubstream, peer_manager::PeerManagerError, protocols::identity::Identity,
    transport, ProtocolId,
};
use channel;
use futures::{
    channel::oneshot,
    future::{BoxFuture, FutureExt},
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::{
    multiplexing::StreamMultiplexer,
    negotiate::{negotiate_inbound, negotiate_outbound_interactive, negotiate_outbound_select},
    transport::ConnectionOrigin,
};
use parity_multiaddr::Multiaddr;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum PeerRequest<TSubstream> {
    OpenSubstream(
        ProtocolId,
        oneshot::Sender<Result<TSubstream, PeerManagerError>>,
    ),
    CloseConnection,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Requested,
    ConnectionLost,
}

#[derive(Debug)]
pub enum PeerNotification<TSubstream> {
    NewSubstream(PeerId, NegotiatedSubstream<TSubstream>),
    PeerDisconnected(PeerId, RoleType, ConnectionOrigin, DisconnectReason),
}

pub struct Peer<TMuxer>
where
    TMuxer: StreamMultiplexer,
{
    /// Identity of the remote peer
    identity: Identity,
    /// Underlying connection.
    connection: TMuxer,
    /// Protocols supported by self.
    own_supported_protocols: Vec<ProtocolId>,
    /// channel to notify about new inbound substreams.
    peer_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    /// channel to receive requests for opening new outbound substreams.
    requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
    /// Origin of the connection.
    origin: ConnectionOrigin,
    /// Flag to indicate if the actor is being shut down.
    shutdown: bool,
}

impl<TMuxer> Peer<TMuxer>
where
    TMuxer: StreamMultiplexer + 'static,
    TMuxer::Substream: 'static,
    TMuxer::Outbound: 'static,
{
    pub fn new(
        identity: Identity,
        connection: TMuxer,
        origin: ConnectionOrigin,
        own_supported_protocols: Vec<ProtocolId>,
        peer_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
        requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
    ) -> Self {
        Self {
            identity,
            connection,
            origin,
            own_supported_protocols,
            peer_notifs_tx,
            requests_rx,
            shutdown: false,
        }
    }

    pub async fn start(mut self) {
        let mut substream_rx = self.connection.listen_for_inbound().fuse();
        let mut pending_outbound_substreams = FuturesUnordered::new();
        let mut pending_inbound_substreams = FuturesUnordered::new();
        while !self.shutdown {
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
                            let event = PeerNotification::NewSubstream(
                                self.identity.peer_id(),
                                negotiated_substream,
                            );
                            self.peer_notifs_tx.send(event).await.unwrap();
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
        match tokio::time::timeout(transport::TRANSPORT_TIMEOUT, self.connection.close()).await {
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

        self.peer_notifs_tx
            .send(PeerNotification::PeerDisconnected(
                self.identity.peer_id(),
                self.identity.role(),
                self.origin,
                reason,
            ))
            .await
            .unwrap();
    }
}

pub struct PeerHandle<TSubstream> {
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
