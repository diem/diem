// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Convenience Network API for Libra

pub use crate::protocols::rpc::error::RpcError;
use crate::{
    error::NetworkError,
    peer_manager::{
        ConnectionNotification, ConnectionRequestSender, PeerManagerNotification,
        PeerManagerRequestSender,
    },
    ProtocolId,
};
use bytes::Bytes;
use channel::libra_channel;
use futures::{
    channel::oneshot,
    stream::{FusedStream, Map, Select, Stream, StreamExt},
    task::{Context, Poll},
};
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, pin::Pin, time::Duration};

pub trait Message: DeserializeOwned + Serialize {}
impl<T: DeserializeOwned + Serialize> Message for T {}

/// Events received by network clients in a validator
///
/// An enumeration of the various types of messages that the network will be sending
/// to its clients. This differs from [`NetworkNotification`] since the contents are deserialized
/// into the type `TMessage` over which `Event` is generic. Note that we assume here that for every
/// consumer of this API there's a singleton message type, `TMessage`,  which encapsulates all the
/// messages and RPCs that are received by that consumer.
///
/// [`NetworkNotification`]: crate::interface::NetworkNotification
#[derive(Debug)]
pub enum Event<TMessage> {
    /// New inbound direct-send message from peer.
    Message((PeerId, TMessage)),
    /// New inbound rpc request. The request is fulfilled by sending the
    /// serialized response `Bytes` over the `oneshot::Sender`, where the network
    /// layer will handle sending the response over-the-wire.
    RpcRequest((PeerId, TMessage, oneshot::Sender<Result<Bytes, RpcError>>)),
    /// Peer which we have a newly established connection with.
    NewPeer(PeerId, ConnectionOrigin),
    /// Peer with which we've lost our connection.
    LostPeer(PeerId, ConnectionOrigin),
}

/// impl PartialEq for simpler testing
impl<TMessage: PartialEq> PartialEq for Event<TMessage> {
    fn eq(&self, other: &Event<TMessage>) -> bool {
        use Event::*;
        match (self, other) {
            (Message((pid1, msg1)), Message((pid2, msg2))) => pid1 == pid2 && msg1 == msg2,
            // ignore oneshot::Sender in comparison
            (RpcRequest((pid1, msg1, _)), RpcRequest((pid2, msg2, _))) => {
                pid1 == pid2 && msg1 == msg2
            }
            (NewPeer(pid1, origin1), NewPeer(pid2, origin2)) => pid1 == pid2 && origin1 == origin2,
            (LostPeer(pid1, origin1), LostPeer(pid2, origin2)) => {
                pid1 == pid2 && origin1 == origin2
            }
            _ => false,
        }
    }
}

/// A `Stream` of `Event<TMessage>` from the lower network layer to an upper
/// network application that deserializes inbound network direct-send and rpc
/// messages into `TMessage`.
///
/// `NetworkEvents` is really just a thin wrapper around a
/// `channel::Receiver<NetworkNotification>` that deserializes inbound messages.
#[pin_project]
pub struct NetworkEvents<TMessage> {
    #[pin]
    event_stream: Select<
        Map<
            libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
            fn(PeerManagerNotification) -> Result<Event<TMessage>, NetworkError>,
        >,
        Map<
            libra_channel::Receiver<PeerId, ConnectionNotification>,
            fn(ConnectionNotification) -> Result<Event<TMessage>, NetworkError>,
        >,
    >,
    _marker: PhantomData<TMessage>,
}

/// Trait specifying the signature for `new()` `NetworkEvents`
pub trait NewNetworkEvents {
    fn new(
        peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        connection_notifs_rx: libra_channel::Receiver<PeerId, ConnectionNotification>,
    ) -> Self;
}

impl<TMessage: Message> NewNetworkEvents for NetworkEvents<TMessage> {
    fn new(
        peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        connection_notifs_rx: libra_channel::Receiver<PeerId, ConnectionNotification>,
    ) -> Self {
        let data_event_stream = peer_mgr_notifs_rx.map(
            peer_mgr_notif_to_event
                as fn(PeerManagerNotification) -> Result<Event<TMessage>, NetworkError>,
        );
        let control_event_stream = connection_notifs_rx.map(
            control_msg_to_event
                as fn(ConnectionNotification) -> Result<Event<TMessage>, NetworkError>,
        );
        Self {
            event_stream: ::futures::stream::select(data_event_stream, control_event_stream),
            _marker: PhantomData,
        }
    }
}

impl<TMessage> Stream for NetworkEvents<TMessage> {
    type Item = Result<Event<TMessage>, NetworkError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        self.project().event_stream.poll_next(context)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.event_stream.size_hint()
    }
}

fn peer_mgr_notif_to_event<TMessage: Message>(
    notif: PeerManagerNotification,
) -> Result<Event<TMessage>, NetworkError> {
    match notif {
        PeerManagerNotification::RecvRpc(peer_id, rpc_req) => {
            let req_msg: TMessage = lcs::from_bytes(&rpc_req.data)?;
            Ok(Event::RpcRequest((peer_id, req_msg, rpc_req.res_tx)))
        }
        PeerManagerNotification::RecvMessage(peer_id, msg) => {
            let msg: TMessage = lcs::from_bytes(&msg.mdata)?;
            Ok(Event::Message((peer_id, msg)))
        }
    }
}

fn control_msg_to_event<TMessage>(
    notif: ConnectionNotification,
) -> Result<Event<TMessage>, NetworkError> {
    match notif {
        ConnectionNotification::NewPeer(peer_id, _addr, origin, _context) => {
            Ok(Event::NewPeer(peer_id, origin))
        }
        ConnectionNotification::LostPeer(peer_id, _addr, origin, _reason) => {
            Ok(Event::LostPeer(peer_id, origin))
        }
    }
}

impl<TMessage> FusedStream for NetworkEvents<TMessage> {
    fn is_terminated(&self) -> bool {
        self.event_stream.is_terminated()
    }
}

/// `NetworkSender` is the generic interface from upper network applications to
/// the lower network layer. It provides the full API for network applications,
/// including sending direct-send messages, sending rpc requests, as well as
/// dialing or disconnecting from peers and updating the list of accepted public
/// keys.
///
/// `NetworkSender` is in fact a thin wrapper around a `PeerManagerRequestSender`, which in turn is
/// a thin wrapper on `libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>`,
/// mostly focused on providing a more ergonomic API. However, network applications will usually
/// provide their own thin wrapper around `NetworkSender` that narrows the API to the specific
/// interface they need. For instance, `mempool` only requires direct-send functionality so its
/// `MempoolNetworkSender` only exposes a `send_to` function.
///
/// Provide Protobuf wrapper over `[peer_manager::PeerManagerRequestSender]`
#[derive(Clone)]
pub struct NetworkSender<TMessage> {
    peer_mgr_reqs_tx: PeerManagerRequestSender,
    connection_reqs_tx: ConnectionRequestSender,
    _marker: PhantomData<TMessage>,
}

/// Trait specifying the signature for `new()` `NetworkSender`s
pub trait NewNetworkSender {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self;
}

impl<TMessage> NewNetworkSender for NetworkSender<TMessage> {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            peer_mgr_reqs_tx,
            connection_reqs_tx,
            _marker: PhantomData,
        }
    }
}

impl<TMessage> NetworkSender<TMessage> {
    /// Request that a given Peer be dialed at the provided `NetworkAddress` and
    /// synchronously wait for the request to be performed.
    pub async fn dial_peer(
        &mut self,
        peer: PeerId,
        addr: NetworkAddress,
    ) -> Result<(), NetworkError> {
        self.connection_reqs_tx.dial_peer(peer, addr).await?;
        Ok(())
    }

    /// Request that a given Peer be disconnected and synchronously wait for the request to be
    /// performed.
    pub async fn disconnect_peer(&mut self, peer: PeerId) -> Result<(), NetworkError> {
        self.connection_reqs_tx.disconnect_peer(peer).await?;
        Ok(())
    }
}

impl<TMessage: Message> NetworkSender<TMessage> {
    /// Send a protobuf message to a single recipient. Provides a wrapper over
    /// `[peer_manager::PeerManagerRequestSender::send_to]`.
    pub fn send_to(
        &mut self,
        recipient: PeerId,
        protocol: ProtocolId,
        message: TMessage,
    ) -> Result<(), NetworkError> {
        let mdata = lcs::to_bytes(&message)?.into();
        self.peer_mgr_reqs_tx.send_to(recipient, protocol, mdata)?;
        Ok(())
    }

    /// Send a protobuf message to a many recipients. Provides a wrapper over
    /// `[peer_manager::PeerManagerRequestSender::send_to_many]`.
    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        protocol: ProtocolId,
        message: TMessage,
    ) -> Result<(), NetworkError> {
        // Serialize message.
        let mdata = lcs::to_bytes(&message)?.into();
        self.peer_mgr_reqs_tx
            .send_to_many(recipients, protocol, mdata)?;
        Ok(())
    }

    /// Send a protobuf rpc request to a single recipient while handling
    /// serialization and deserialization of the request and response respectively.
    /// Assumes that the request and response both have the same message type.
    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        protocol: ProtocolId,
        req_msg: TMessage,
        timeout: Duration,
    ) -> Result<TMessage, RpcError> {
        // serialize request
        let req_data = lcs::to_bytes(&req_msg)?.into();
        let res_data = self
            .peer_mgr_reqs_tx
            .send_rpc(recipient, protocol, req_data, timeout)
            .await?;
        let res_msg: TMessage = lcs::from_bytes(&res_data)?;
        Ok(res_msg)
    }
}
