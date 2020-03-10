// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Peer actor owns the underlying connection and is responsible for listening for
//!  and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{
    common::NegotiatedSubstream, peer_manager::PeerManagerError, protocols::identity::Identity,
    transport, ProtocolId,
};
use futures::{
    channel::oneshot,
    future::BoxFuture,
    stream::{FuturesUnordered, StreamExt},
    FutureExt, SinkExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::{
    multiplexing::{Control, StreamMultiplexer},
    negotiate::{negotiate_inbound, negotiate_outbound_interactive, negotiate_outbound_select},
    transport::ConnectionOrigin,
};
use parity_multiaddr::Multiaddr;
use std::{collections::HashSet, fmt::Debug, iter::FromIterator};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Requested,
    ConnectionLost,
}

#[derive(Debug)]
pub enum PeerNotification<TSubstream> {
    NewSubstream(PeerId, NegotiatedSubstream<TSubstream>),
    PeerDisconnected(Identity, Multiaddr, ConnectionOrigin, DisconnectReason),
}

pub struct Peer<TMuxer>
where
    TMuxer: StreamMultiplexer,
{
    /// Identity of the remote peer
    identity: Identity,
    /// Address at which we are connected to the remote peer.
    address: Multiaddr,
    /// Origin of the connection.
    origin: ConnectionOrigin,
    /// Underlying connection.
    connection: Option<TMuxer>,
    /// Channel to receive requests for opening new outbound substreams.
    requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
    /// Channel to send peer notifications to PeerManager.
    peer_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    /// RPC protocols supported by self.
    rpc_protocols: HashSet<ProtocolId>,
    /// Channel to notify about new inbound RPC substreams.
    rpc_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    /// DirectSend protocols supported by self.
    direct_send_protocols: HashSet<ProtocolId>,
    /// Channel to notify about new inbound DirectSend substreams.
    direct_send_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    /// All supported protocols. This is derived from `rpc_protocols` and `direct_send_protocols`
    /// at the time this struct is constructed.
    own_supported_protocols: Vec<ProtocolId>,
    /// Flag to indicate if the actor is being shut down.
    shutting_down: bool,
}

impl<TMuxer> Peer<TMuxer>
where
    TMuxer: StreamMultiplexer + 'static,
    TMuxer::Substream: 'static,
{
    pub fn new(
        identity: Identity,
        address: Multiaddr,
        origin: ConnectionOrigin,
        connection: TMuxer,
        requests_rx: channel::Receiver<PeerRequest<TMuxer::Substream>>,
        peer_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
        rpc_protocols: HashSet<ProtocolId>,
        rpc_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
        direct_send_protocols: HashSet<ProtocolId>,
        direct_send_notifs_tx: channel::Sender<PeerNotification<TMuxer::Substream>>,
    ) -> Self {
        let own_supported_protocols = Vec::from_iter(
            rpc_protocols
                .iter()
                .chain(direct_send_protocols.iter())
                .cloned(),
        );
        Self {
            identity,
            address,
            origin,
            connection: Some(connection),
            requests_rx,
            peer_notifs_tx,
            rpc_protocols,
            rpc_notifs_tx,
            direct_send_protocols,
            direct_send_notifs_tx,
            own_supported_protocols,
            shutting_down: false,
        }
    }

    pub async fn start(mut self) {
        let (substream_rx, control) = self.connection.take().unwrap().start().await;
        let mut substream_rx = substream_rx.fuse();
        let mut pending_outbound_substreams = FuturesUnordered::new();
        let mut pending_inbound_substreams = FuturesUnordered::new();
        let self_peer_id = self.identity.peer_id();
        while !self.shutting_down {
            futures::select! {
                maybe_req = self.requests_rx.next() => {
                    if let Some(request) = maybe_req {
                        self.handle_request(control.clone(), &mut pending_outbound_substreams, request).await;
                    } else {
                        // This branch will only be taken if all PeerRequest senders for this Peer
                        // get dropped.
                        break;
                    }
                },
                maybe_substream = substream_rx.next() => {
                    match maybe_substream {
                        Some(Ok(substream)) => {
                            trace!("New inbound substream from peer: {}", self_peer_id.short_str());
                            pending_inbound_substreams.push(
                                Self::negotiate_inbound_substream(
                                    substream, self.own_supported_protocols.clone()));
                        }
                        Some(Err(e)) => {
                            warn!("Inbound substream error {:?} with peer {}",
                                e, self_peer_id.short_str());
                            self.close_connection(control.clone(), DisconnectReason::ConnectionLost).await;
                        }
                        None => {
                            warn!("Inbound substreams exhausted with peer {}",
                                self_peer_id.short_str());
                            self.close_connection(control.clone(), DisconnectReason::ConnectionLost).await;
                        }
                    }
                },
                negotiated_subststream = pending_inbound_substreams.select_next_some() => {
                    self.handle_negotiated_substream(negotiated_subststream).await;
                },
                _ = pending_outbound_substreams.select_next_some() => {
                    // Do nothing since these futures have an output of "()"
                },
            }
        }
        debug!(
            "Peer actor '{}' shutdown",
            self.identity.peer_id().short_str()
        );
    }

    async fn handle_negotiated_substream(
        &mut self,
        negotiated_subststream: Result<NegotiatedSubstream<TMuxer::Substream>, PeerManagerError>,
    ) {
        match negotiated_subststream {
            Ok(negotiated_substream) => {
                let protocol = negotiated_substream.protocol.clone();
                let event =
                    PeerNotification::NewSubstream(self.identity.peer_id(), negotiated_substream);
                if self.rpc_protocols.contains(&protocol) {
                    if let Err(err) = self.rpc_notifs_tx.send(event).await {
                        warn!("Failed to send notification to RPC actor. Error: {:?}", err);
                    }
                } else if self.direct_send_protocols.contains(&protocol) {
                    if let Err(err) = self.direct_send_notifs_tx.send(event).await {
                        warn!(
                            "Failed to send notification to DirectSend actor. Error: {:?}",
                            err
                        );
                    }
                } else {
                    // We should only be able to negotiate supported protocols, and therefore this
                    // branch is unreachable.
                    unreachable!("Negotiated unsupported protocol");
                }
            }
            Err(e) => {
                error!(
                    "Inbound substream negotiation for peer {} failed: {}",
                    self.identity.peer_id().short_str(),
                    e
                );
            }
        }
    }

    async fn handle_request<'a>(
        &'a mut self,
        control: TMuxer::Control,
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
                pending
                    .push(self.handle_open_outbound_substream_request(protocol, control, channel));
            }
            PeerRequest::CloseConnection => {
                self.close_connection(control, DisconnectReason::Requested)
                    .await;
            }
        }
    }

    fn handle_open_outbound_substream_request(
        &self,
        protocol: ProtocolId,
        control: TMuxer::Control,
        channel: oneshot::Sender<Result<TMuxer::Substream, PeerManagerError>>,
    ) -> BoxFuture<'static, ()> {
        let optimistic_negotiation = self.identity.is_protocol_supported(&protocol);
        Self::negotiate_outbound_substream(
            self.identity.peer_id(),
            control,
            protocol,
            optimistic_negotiation,
            channel,
        )
        .boxed()
    }

    async fn negotiate_outbound_substream(
        peer_id: PeerId,
        mut control: TMuxer::Control,
        protocol: ProtocolId,
        optimistic_negotiation: bool,
        channel: oneshot::Sender<Result<TMuxer::Substream, PeerManagerError>>,
    ) {
        let response = match control.open_stream().await {
            Ok(substream) => {
                // TODO(bmwill) Evaluate if we should still try to open and negotiate an outbound
                // substream even though we know for a fact that the Identity struct of this Peer
                // doesn't include the protocol we're interested in.
                if optimistic_negotiation {
                    negotiate_outbound_select(substream, lcs::to_bytes(&protocol).unwrap()).await
                } else {
                    warn!(
                        "Negotiating outbound substream interactively: Protocol({:?}) PeerId({})",
                        protocol,
                        peer_id.short_str()
                    );
                    negotiate_outbound_interactive(substream, &[lcs::to_bytes(&protocol).unwrap()])
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

    async fn negotiate_inbound_substream(
        substream: TMuxer::Substream,
        own_supported_protocols: Vec<ProtocolId>,
    ) -> Result<NegotiatedSubstream<TMuxer::Substream>, PeerManagerError> {
        let (substream, protocol) = negotiate_inbound(
            substream,
            own_supported_protocols
                .into_iter()
                .map(|p| lcs::to_bytes(&p).unwrap())
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(NegotiatedSubstream {
            protocol: lcs::from_bytes(&protocol).unwrap(),
            substream,
        })
    }

    async fn close_connection(&mut self, mut control: TMuxer::Control, reason: DisconnectReason) {
        match tokio::time::timeout(transport::TRANSPORT_TIMEOUT, control.close()).await {
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
        // If the graceful shutdown above fails, the connection will be forcefully terminated once
        // the connection struct is dropped. Setting the `shutdown` flag to true ensures that the
        // peer actor will terminate and close the connection in the process.
        self.shutting_down = true;
        // We send a PeerDisconnected event to NetworkProvider as a result (or in case of a failure
        // above, in anticipation of) closing the connection.
        if let Err(e) = self
            .peer_notifs_tx
            .send(PeerNotification::PeerDisconnected(
                self.identity.clone(),
                self.address.clone(),
                self.origin,
                reason,
            ))
            .await
        {
            warn!(
                "Failed to notify upstream about disconnection of peer: {}; error: {:?}",
                self.identity.peer_id().short_str(),
                e
            );
        }
    }
}

pub struct PeerHandle<TSubstream> {
    peer_id: PeerId,
    sender: channel::Sender<PeerRequest<TSubstream>>,
    address: Multiaddr,
}

impl<TSubstream> Clone for PeerHandle<TSubstream> {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            sender: self.sender.clone(),
            address: self.address.clone(),
        }
    }
}

impl<TSubstream> PeerHandle<TSubstream> {
    pub fn new(
        peer_id: PeerId,
        address: Multiaddr,
        sender: channel::Sender<PeerRequest<TSubstream>>,
    ) -> Self {
        Self {
            peer_id,
            address,
            sender,
        }
    }

    pub fn address(&self) -> &Multiaddr {
        &self.address
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub async fn open_substream(
        &mut self,
        protocol: ProtocolId,
    ) -> Result<TSubstream, PeerManagerError> {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        if self
            .sender
            .send(PeerRequest::OpenSubstream(protocol, oneshot_tx))
            .await
            .is_err()
        {
            error!(
                "Sending OpenSubstream request to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
        oneshot_rx
            .await
            // The open_substream request can get dropped/canceled if the peer
            // connection is in the process of shutting down.
            .map_err(|_| PeerManagerError::NotConnected(self.peer_id))?
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
    }
}
