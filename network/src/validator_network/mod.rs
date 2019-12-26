// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Network API for [`Consensus`](/consensus/index.html) and [`Mempool`](/mempool/index.html)

pub use crate::protocols::rpc::error::RpcError;
use crate::{
    common::NetworkPublicKeys,
    error::NetworkError,
    interface::{NetworkNotification, NetworkRequest},
    protocols::{direct_send, rpc::OutboundRpcRequest},
    utils::MessageExt,
    ProtocolId,
};
use bytes05::Bytes;
use futures::{
    channel::oneshot,
    ready,
    sink::SinkExt,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use parity_multiaddr::Multiaddr;
use pin_project::pin_project;
use prost::Message;
use std::{collections::HashMap, default::Default, marker::PhantomData, pin::Pin, time::Duration};

pub mod network_builder;

mod admission_control;
mod chain_state;
mod consensus;
mod discovery;
mod health_checker;
mod mempool;
mod state_synchronizer;

#[cfg(test)]
mod test;

// Public re-exports
pub use crate::interface::LibraNetworkProvider;
pub use admission_control::{
    AdmissionControlNetworkEvents, AdmissionControlNetworkSender, ADMISSION_CONTROL_RPC_PROTOCOL,
};
pub use chain_state::{
    ChainStateNetworkEvents, ChainStateNetworkSender, CHAIN_STATE_DIRECT_SEND_PROTOCOL,
};
pub use consensus::{
    ConsensusNetworkEvents, ConsensusNetworkSender, CONSENSUS_DIRECT_SEND_PROTOCOL,
    CONSENSUS_RPC_PROTOCOL,
};
pub use discovery::{
    DiscoveryNetworkEvents, DiscoveryNetworkSender, DISCOVERY_DIRECT_SEND_PROTOCOL,
};
pub use health_checker::{
    HealthCheckerNetworkEvents, HealthCheckerNetworkSender, HEALTH_CHECKER_RPC_PROTOCOL,
};
use libra_types::PeerId;
pub use mempool::{MempoolNetworkEvents, MempoolNetworkSender, MEMPOOL_DIRECT_SEND_PROTOCOL};
pub use state_synchronizer::{
    StateSynchronizerEvents, StateSynchronizerSender, STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL,
};

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
    NewPeer(PeerId),
    /// Peer with which we've lost our connection.
    LostPeer(PeerId),
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
            (NewPeer(pid1), NewPeer(pid2)) => pid1 == pid2,
            (LostPeer(pid1), LostPeer(pid2)) => pid1 == pid2,
            _ => false,
        }
    }
}

/// A `Stream` of `Event<TMessage>` from the lower network layer to an upper
/// network application that deserializes inbound network direct-send and rpc
/// messages into `TMessage`, which is some protobuf format implementing
/// the `prost::Message` trait.
///
/// `NetworkEvents` is really just a thin wrapper around a
/// `channel::Receiver<NetworkNotification>` that deserializes inbound messages.
#[pin_project]
pub struct NetworkEvents<TMessage: Message + Default> {
    #[pin]
    inner: channel::Receiver<NetworkNotification>,
    _marker: PhantomData<TMessage>,
}

impl<TMessage: Message + Default> NetworkEvents<TMessage> {
    pub fn new(inner: channel::Receiver<NetworkNotification>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn into_inner(self) -> channel::Receiver<NetworkNotification> {
        self.inner
    }
}

impl<TMessage: Message + Default> Stream for NetworkEvents<TMessage> {
    type Item = Result<Event<TMessage>, NetworkError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        let maybe_notif = ready!(self.project().inner.poll_next(context));

        Poll::Ready(maybe_notif.map(|notif| match notif {
            NetworkNotification::NewPeer(peer_id) => Ok(Event::NewPeer(peer_id)),
            NetworkNotification::LostPeer(peer_id) => Ok(Event::LostPeer(peer_id)),
            NetworkNotification::RecvRpc(peer_id, rpc_req) => {
                let req_msg = TMessage::decode(rpc_req.data.as_ref())?;
                Ok(Event::RpcRequest((peer_id, req_msg, rpc_req.res_tx)))
            }
            NetworkNotification::RecvMessage(peer_id, msg) => {
                let msg = TMessage::decode(msg.mdata.as_ref())?;
                Ok(Event::Message((peer_id, msg)))
            }
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<TMessage: Message + Default> FusedStream for NetworkEvents<TMessage> {
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

/// `NetworkSender` is the generic interface from upper network applications to
/// the lower network layer. It provides the full API for network applications,
/// including sending direct-send messages, sending rpc requests, as well as
/// dialing or disconnecting from peers and updating the list of accepted public
/// keys.
///
/// `NetworkSender` is in fact a thin wrapper around a
/// `channel::Sender<NetworkRequest>`, mostly focused on providing a more
/// ergonomic API. However, network applications will usually provide their own
/// thin wrapper around `NetworkSender` that narrows the API to the specific
/// interface they need. For instance, `mempool` only requires direct-send
/// functionality so its `MempoolNetworkSender` only exposes a `send_to` function.
///
/// The `TMessage` generic is a protobuf message type (`prost::Message`).
#[derive(Clone)]
pub struct NetworkSender<TMessage: Message + Default> {
    inner: channel::Sender<NetworkRequest>,
    _marker: PhantomData<TMessage>,
}

impl<TMessage: Message + Default> NetworkSender<TMessage> {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    /// Send a fire-and-forget direct-send message to remote peer `recipient`.
    ///
    /// The returned Future simply resolves when the message has been enqueued on
    /// the network actor's event queue. It therefore makes no reliable delivery
    /// guarantees.
    ///
    /// The Future will resolve to an `Err` if the event queue is unexpectedly
    /// shutdown.
    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        protocol: ProtocolId,
        message: TMessage,
    ) -> Result<(), NetworkError> {
        self.inner
            .send(NetworkRequest::SendMessage(
                recipient,
                direct_send::Message {
                    protocol,
                    mdata: message.to_bytes().unwrap(),
                },
            ))
            .await?;

        Ok(())
    }

    /// Send the _same_ message to many `recipients` using the direct-send
    /// protocol.
    ///
    /// This method is an optimization so that we can avoid serializing and
    /// copying the same message many times when we want to sent a single message
    /// to many peers. Note that the `Bytes` the messages is serialized into is a
    /// ref-counted byte buffer, so we can avoid excess copies as all direct-sends
    /// will share the same underlying byte buffer.
    ///
    /// The returned Future simply resolves when all send requests have been
    /// enqueued on the network actor's event queue. It therefore makes no
    /// reliable delivery guarantees.
    ///
    /// The Future will resolve to an `Err` if the event queue is unexpectedly
    /// shutdown.
    pub async fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        protocol: ProtocolId,
        message: TMessage,
    ) -> Result<(), NetworkError> {
        let msg_bytes = message.to_bytes().unwrap();
        let msg = direct_send::Message {
            protocol,
            mdata: msg_bytes,
        };

        for recipient in recipients {
            // We return `Err` early here if the send fails. Since sending will
            // only fail if the queue is unexpectedly shutdown (i.e., receiver
            // dropped early), we know that we can't make further progress if
            // this send fails.
            self.inner
                .send(NetworkRequest::SendMessage(recipient, msg.clone()))
                .await?;
        }

        Ok(())
    }

    /// Send a unary rpc request to remote peer `recipient`. Handles
    /// serialization and deserialization of the message types, assuming that the
    /// request and response both have the same message type.
    pub async fn unary_rpc(
        &mut self,
        recipient: PeerId,
        protocol: ProtocolId,
        req_msg: TMessage,
        timeout: Duration,
    ) -> Result<TMessage, RpcError> {
        // serialize request
        let req_data = req_msg.to_bytes().unwrap();

        // ask network to fulfill rpc request
        let (res_tx, res_rx) = oneshot::channel();
        let req = OutboundRpcRequest {
            protocol,
            data: req_data,
            res_tx,
            timeout,
        };
        self.inner
            .send(NetworkRequest::SendRpc(recipient, req))
            .await?;

        // wait for response and deserialize
        let res_data = res_rx.await??;
        let res_msg = TMessage::decode(res_data.as_ref())?;
        Ok(res_msg)
    }

    /// Update the set of eligible nodes that the network should accept
    /// connections from.
    pub async fn update_eligible_nodes(
        &mut self,
        nodes: HashMap<PeerId, NetworkPublicKeys>,
    ) -> Result<(), NetworkError> {
        self.inner
            .send(NetworkRequest::UpdateEligibleNodes(nodes))
            .await?;
        Ok(())
    }

    /// Dial the peer with the given `PeerId` at a `Multiaddr`.
    pub async fn dial_peer(&mut self, peer: PeerId, addr: Multiaddr) -> Result<(), NetworkError> {
        let (res_tx, res_rx) = oneshot::channel();
        self.inner
            .send(NetworkRequest::DialPeer(peer, addr, res_tx))
            .await?;
        Ok(res_rx.await??)
    }

    /// Disconnect from the peer with the given `PeerId`.
    pub async fn disconnect_peer(&mut self, peer: PeerId) -> Result<(), NetworkError> {
        let (res_tx, res_rx) = oneshot::channel();
        self.inner
            .send(NetworkRequest::DisconnectPeer(peer, res_tx))
            .await?;
        Ok(res_rx.await??)
    }

    /// Unwrap the `NetworkSender` into the underlying
    /// `channel::Sender<NetworkRequest>`.
    pub fn into_inner(self) -> channel::Sender<NetworkRequest> {
        self.inner
    }

    /// Get a mutable reference to the underlying
    /// `channel::Sender<NetworkRequest>`.
    pub fn get_mut(&mut self) -> &mut channel::Sender<NetworkRequest> {
        &mut self.inner
    }
}
