// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Peer actor owns the underlying connection and is responsible for listening for
//! and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{
    common::NegotiatedSubstream,
    peer_manager::PeerManagerError,
    protocols::{identity::Identity, wire::messaging::v1::NetworkMessage},
    sink::NetworkSinkExt,
    transport, ProtocolId,
};
use futures::{
    self,
    channel::oneshot,
    future::BoxFuture,
    io::{AsyncRead, AsyncWrite},
    stream::{FuturesUnordered, StreamExt},
    FutureExt, SinkExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::{
    compat::IoCompat,
    multiplexing::{yamux::Yamux, Control, StreamMultiplexer},
    negotiate::{negotiate_inbound, negotiate_outbound_interactive, negotiate_outbound_select},
    transport::ConnectionOrigin,
};
use parity_multiaddr::Multiaddr;
use std::{collections::HashSet, fmt::Debug, iter::FromIterator};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum PeerRequest {
    SendMessage(
        NetworkMessage,
        ProtocolId,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    CloseConnection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Requested,
    ConnectionLost,
}

#[derive(Debug)]
pub enum PeerNotification {
    NewMessage(NetworkMessage),
    PeerDisconnected(Identity, Multiaddr, ConnectionOrigin, DisconnectReason),
}

pub struct Peer<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + Sync + 'static,
{
    /// Identity of the remote peer
    identity: Identity,
    /// Address at which we are connected to the remote peer.
    address: Multiaddr,
    /// Origin of the connection.
    origin: ConnectionOrigin,
    /// Underlying connection.
    connection: Option<TSocket>,
    /// Channel to receive requests for opening new outbound substreams.
    requests_rx: channel::Receiver<PeerRequest>,
    /// Channel to send peer notifications to PeerManager.
    peer_notifs_tx: channel::Sender<PeerNotification>,
    /// RPC protocols supported by self.
    rpc_protocols: HashSet<ProtocolId>,
    /// Channel to notify about new inbound RPC substreams.
    rpc_notifs_tx: channel::Sender<PeerNotification>,
    /// DirectSend protocols supported by self.
    direct_send_protocols: HashSet<ProtocolId>,
    /// Channel to notify about new inbound DirectSend substreams.
    direct_send_notifs_tx: channel::Sender<PeerNotification>,
    /// All supported protocols. This is derived from `rpc_protocols` and `direct_send_protocols`
    /// at the time this struct is constructed.
    own_supported_protocols: Vec<ProtocolId>,
    /// Flag to indicate if the actor is being shut down.
    shutting_down: bool,
}

impl<TSocket> Peer<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + Sync + 'static,
{
    pub fn new(
        identity: Identity,
        address: Multiaddr,
        origin: ConnectionOrigin,
        connection: TSocket,
        requests_rx: channel::Receiver<PeerRequest>,
        peer_notifs_tx: channel::Sender<PeerNotification>,
        rpc_protocols: HashSet<ProtocolId>,
        rpc_notifs_tx: channel::Sender<PeerNotification>,
        direct_send_protocols: HashSet<ProtocolId>,
        direct_send_notifs_tx: channel::Sender<PeerNotification>,
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
        let self_peer_id = self.identity.peer_id();
        info!(
            "Starting Peer actor for peer: {:?}",
            self_peer_id.short_str()
        );
        let connection = {
            match Yamux::upgrade_connection(self.connection.take().unwrap(), self.origin).await {
                Err(e) => {
                    warn!("yamux handshake failed. Error: {:?}", e);
                    return;
                }
                Ok(connection) => connection,
            }
        };
        let (substream_rx, control) = connection.start().await;
        let mut substream_rx = substream_rx.fuse();
        let mut pending_outbound_substreams = FuturesUnordered::new();
        let mut pending_inbound_substreams = FuturesUnordered::new();
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
                    if let Err(err) = self.handle_negotiated_substream(negotiated_subststream).await {
                        warn!("Error in handling inbound substream from peer: {:?}. Error: {:?}",
                            self_peer_id.short_str(), err);
                    }
                },
                () = pending_outbound_substreams.select_next_some() => {
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
        negotiated_subststream: Result<
            NegotiatedSubstream<<Yamux<TSocket> as StreamMultiplexer>::Substream>,
            PeerManagerError,
        >,
    ) -> Result<(), PeerManagerError> {
        match negotiated_subststream {
            Ok(negotiated_substream) => {
                let protocol = negotiated_substream.protocol;
                debug!(
                    "Successfully negotiated inbound substream '{:?}' with Peer {}",
                    protocol,
                    self.identity.peer_id().short_str()
                );
                let mut stream = Framed::new(
                    IoCompat::new(negotiated_substream.substream),
                    LengthDelimitedCodec::new(),
                );
                // Read inbound message from stream.
                let message = stream
                    .next()
                    .await
                    .ok_or_else(|| anyhow::anyhow!("stream terminated"))??;
                let message = message.freeze();
                let message: NetworkMessage = lcs::from_bytes(&message)?;
                let event = PeerNotification::NewMessage(message);
                debug!("Notifying upstream about new event: {:?}", event);
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
                // Half-close our end of stream.
                stream.close().await?;
            }
            Err(e) => {
                error!(
                    "Inbound substream negotiation for peer {} failed: {}",
                    self.identity.peer_id().short_str(),
                    e
                );
            }
        }
        Ok(())
    }

    async fn handle_request<'a>(
        &'a mut self,
        control: <Yamux<TSocket> as StreamMultiplexer>::Control,
        pending: &'a mut FuturesUnordered<BoxFuture<'static, ()>>,
        request: PeerRequest,
    ) {
        trace!(
            "Peer {} PeerRequest::{:?}",
            self.identity.peer_id().short_str(),
            request
        );
        match request {
            PeerRequest::SendMessage(message, protocol, channel) => {
                let identity = self.identity.clone();
                let f = async move {
                    let result =
                        Self::send_message(message, protocol, identity.clone(), control).await.map_err(|e| {
                            info!("Failed to send message for protocol {:?} to peer: {:?}. Error: {:?}",
                                protocol, identity.peer_id().short_str(), e);
                            e
                        });
                    if channel.send(result).is_err() {
                        info!(
                            "oneshot channel receiver dropped for new substream with peer {} for protocol {:?}",
                            identity.peer_id().short_str(),
                            protocol
                        );
                    }
                };
                pending.push(f.boxed());
            }
            PeerRequest::CloseConnection => {
                self.close_connection(control, DisconnectReason::Requested)
                    .await;
            }
        }
    }

    async fn send_message(
        message: NetworkMessage,
        protocol: ProtocolId,
        identity: Identity,
        mut control: <Yamux<TSocket> as StreamMultiplexer>::Control,
    ) -> Result<(), PeerManagerError> {
        let peer_id = identity.peer_id();
        // Open substream.
        let substream = control.open_stream().await?;
        // Negotiate protocol on opened substream.
        let negotiated_stream = {
            let optimistic_negotiation = identity.is_protocol_supported(protocol);
            // TODO(bmwill) Evaluate if we should still try to open and negotiate an outbound
            // substream even though we know for a fact that the Identity struct of this Peer
            // doesn't include the protocol we're interested in.
            if optimistic_negotiation {
                negotiate_outbound_select(substream, lcs::to_bytes(&protocol)?).await
            } else {
                warn!(
                    "Negotiating outbound substream interactively: Protocol({:?}) PeerId({})",
                    protocol,
                    peer_id.short_str()
                );
                negotiate_outbound_interactive(substream, &[lcs::to_bytes(&protocol)?])
                    .await
                    .map(|(substream, _protocol)| substream)
            }
        }
        .map_err(|e| {
            debug!(
                "Unable to negotiated outbound substream '{:?}' with Peer {}: {}",
                protocol,
                peer_id.short_str(),
                e
            );
            e
        })?;
        debug!(
            "Successfully negotiated outbound substream '{:?}' with Peer {}",
            protocol,
            peer_id.short_str()
        );
        // Frame substream sends by length prefixing.
        let mut stream = Framed::new(
            IoCompat::new(negotiated_stream),
            LengthDelimitedCodec::new(),
        );
        // Send the message.
        stream
            .buffered_send(lcs::to_bytes(&message)?.into())
            .await?;
        // Half-close our end of stream.
        stream.close().await?;

        Ok(())
    }

    async fn negotiate_inbound_substream(
        substream: <Yamux<TSocket> as StreamMultiplexer>::Substream,
        own_supported_protocols: Vec<ProtocolId>,
    ) -> Result<
        NegotiatedSubstream<<Yamux<TSocket> as StreamMultiplexer>::Substream>,
        PeerManagerError,
    > {
        let (substream, protocol) = negotiate_inbound(
            substream,
            own_supported_protocols
                .into_iter()
                .map(|p| lcs::to_bytes(&p))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .await?;
        Ok(NegotiatedSubstream {
            protocol: lcs::from_bytes(&protocol)?,
            substream,
        })
    }

    async fn close_connection(
        &mut self,
        mut control: <Yamux<TSocket> as StreamMultiplexer>::Control,
        reason: DisconnectReason,
    ) {
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

pub struct PeerHandle {
    peer_id: PeerId,
    sender: channel::Sender<PeerRequest>,
    address: Multiaddr,
}

impl Clone for PeerHandle {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            sender: self.sender.clone(),
            address: self.address.clone(),
        }
    }
}

impl PeerHandle {
    pub fn new(peer_id: PeerId, address: Multiaddr, sender: channel::Sender<PeerRequest>) -> Self {
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

    pub async fn send_message(
        &mut self,
        message: NetworkMessage,
        protocol: ProtocolId,
    ) -> Result<(), PeerManagerError> {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        if self
            .sender
            .send(PeerRequest::SendMessage(message, protocol, oneshot_tx))
            .await
            .is_err()
        {
            error!(
                "Sending message to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
        oneshot_rx
            .await
            // The send_message request can get dropped/canceled if the peer
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
