// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Network API for [`Consensus`](/consensus/index.html) and [`Mempool`](/mempool/index.html)

pub use crate::protocols::rpc::error::RpcError;
use crate::{error::NetworkError, interface::NetworkNotification};
use bytes::Bytes;
use futures::{
    channel::oneshot,
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use pin_project::pin_project;
use prost::Message;
use std::{default::Default, marker::PhantomData, pin::Pin};

pub mod network_builder;

mod admission_control;
mod consensus;
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
pub use consensus::{
    ConsensusNetworkEvents, ConsensusNetworkSender, CONSENSUS_DIRECT_SEND_PROTOCOL,
    CONSENSUS_RPC_PROTOCOL,
};
pub use health_checker::HealthCheckerNetworkEvents;
use libra_types::PeerId;
pub use mempool::{MempoolNetworkEvents, MempoolNetworkSender, MEMPOOL_DIRECT_SEND_PROTOCOL};
pub use state_synchronizer::{
    StateSynchronizerEvents, StateSynchronizerSender, STATE_SYNCHRONIZER_MSG_PROTOCOL,
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
                let req_msg = TMessage::decode(rpc_req.data)?;
                Ok(Event::RpcRequest((peer_id, req_msg, rpc_req.res_tx)))
            }
            NetworkNotification::RecvMessage(peer_id, msg) => {
                let msg = TMessage::decode(msg.mdata)?;
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
